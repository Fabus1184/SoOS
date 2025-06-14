use alloc::{string::String, vec};
use core::fmt::Write as _;
use log::trace;
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator as _, Mapper, Page, PageSize, PageTableFlags, Size4KiB,
    Translate,
};

use crate::process::{MappedPage, PROCESSES};

pub mod generated;

fn copy_string_from_user(string: generated::string_const_t) -> String {
    let mut bytes = vec![0; string.len as usize];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = unsafe { (string.ptr as *const u8).add(i).read_volatile() };
    }
    String::from_utf8(bytes).expect("Invalid UTF-8 string")
}

fn print(pid: u32, arg: &mut generated::syscall_print_t) {
    let string = copy_string_from_user(arg.message);
    write!(crate::TERM.writer(), "{string}").expect("Failed to write to terminal");

    log::debug!("[{pid}]: {string}");
}

/// sleep for the number of milliseconds in rbx
fn sleep(pid: u32, arg: &mut generated::syscall_sleep_t) {
    PROCESSES.with_process_mut(pid, |p| {
        trace!(
            "syscall_handler: sleep for {} milliseconds",
            arg.milliseconds
        );

        p.state = crate::process::State::Sleeping(unsafe {
            crate::i8253::TIMER0.ticks() + arg.milliseconds as u64 / 10
        });
    });
}

/// Exit the current process with the exit code in rbx
fn exit(pid: u32, arg: &mut generated::syscall_exit_t) {
    PROCESSES.with_process_mut(pid, |p| {
        trace!("syscall_handler: exit {:#x}", arg.status);
        p.state = crate::process::State::Terminated(u64::from(arg.status));

        log::debug!("Process {} exited with code {}", p.pid(), arg.status);
    });

    for process in PROCESSES.processes_mut().iter_mut() {
        match process.state {
            crate::process::State::WaitingForChild {
                pid: child_pid,
                arg: child_arg,
            } if child_pid == pid => {
                process.load_paging();
                (unsafe { &mut *child_arg }).return_value.status = arg.status;
                process.state = crate::process::State::Ready;
            }
            _ => {}
        }
    }
}

/// Get the name of the entry at index rdx in the directory at path in rbx
/// Returns the name to the pointer in r8 and the length of the name in rax
fn list_directory(pid: u32, arg: &mut generated::syscall_listdir_t) {
    let process = PROCESSES.process(pid);

    let path = copy_string_from_user(arg.path);

    let mut fs = crate::FILE_SYSTEM
        .try_lock()
        .expect("Failed to lock file system");

    drop(process); // Drop the process to avoid borrowing issues

    let Some(dir) = fs.directory_mut(&path) else {
        log::debug!("Directory not found: {path}");
        arg.return_value.entries_count = 0;
        arg.return_value.error = generated::SYSCALL_LISTDIR_ERROR_NOT_FOUND;
        return;
    };

    if dir.files().len() + dir.directories().len() > arg.entries_len as usize {
        log::debug!(
            "Directory {path} has more entries than the buffer can hold: {} > {}",
            dir.files().len() + dir.directories().len(),
            arg.entries_len
        );
        arg.return_value.entries_count = 0;
        arg.return_value.error = generated::SYSCALL_LISTDIR_ERROR_BUFFER_TOO_SMALL;
        return;
    }

    let mut i = 0;

    for name in dir.files().keys() {
        let entry = unsafe { &mut *arg.entries.add(i) };
        if name.len() > entry.name.len as usize {
            log::debug!(
                "File name '{name}' is too long for the entry at index {i}: {} > {}",
                name.len(),
                entry.name.len
            );
            arg.return_value.entries_count = 0;
            arg.return_value.error = generated::SYSCALL_LISTDIR_ERROR_BUFFER_TOO_SMALL;
            return;
        }

        unsafe {
            core::ptr::copy_nonoverlapping(name.as_ptr(), entry.name.ptr.cast::<u8>(), name.len());
        }

        entry.name.len = name.len() as u32;
        entry.type_ = generated::SYSCALL_LISTDIR_ENTRY_TYPE_FILE;
        i += 1;
    }

    for name in dir.directories().keys() {
        let entry = unsafe { &mut *arg.entries.add(i) };
        if name.len() > entry.name.len as usize {
            log::debug!(
                "Directory name '{name}' is too long for the entry at index {i}: {} > {}",
                name.len(),
                entry.name.len
            );
            arg.return_value.entries_count = 0;
            arg.return_value.error = generated::SYSCALL_LISTDIR_ERROR_BUFFER_TOO_SMALL;
            return;
        }

        unsafe {
            core::ptr::copy_nonoverlapping(name.as_ptr(), entry.name.ptr.cast::<u8>(), name.len());
        }

        entry.name.len = name.len() as u32;
        entry.type_ = generated::SYSCALL_LISTDIR_ENTRY_TYPE_DIR;
        i += 1;
    }

    arg.return_value.entries_count = i as u32;
    arg.return_value.error = generated::SYSCALL_LISTDIR_ERROR_NONE;
}

/// Read from the file descriptor in rbx into the buffer in rcx with length rdx
fn read(pid: u32, arg: &mut generated::syscall_read_t) {
    log::trace!(
        "syscall_handler: read fd {}, buffer {:x}, length {}",
        arg.fd,
        arg.buf as u64,
        arg.len
    );

    let mut process = crate::process::PROCESSES.process_mut(pid);
    let Some(fd) = process.file_descriptor_mut(arg.fd) else {
        log::debug!(
            "pid {pid}, invalid file descriptor: {arg:x?} ({:#x})",
            core::ptr::from_ref(arg) as u64
        );
        arg.return_value.bytes_read = 0;
        arg.return_value.error = generated::SYSCALL_READ_ERROR_INVALID_FD;
        return;
    };

    match fd {
        crate::process::FileDescriptor::Regular { path, offset } => {
            let offset = *offset;
            let path = path.clone();
            drop(process);

            let mut fs = crate::FILE_SYSTEM
                .try_lock()
                .expect("Failed to lock file system");
            let file = fs.file_mut(&path).expect("File not found");

            let mut buffer = vec![0; arg.len as usize];

            let read_result = file.read(offset, crate::io::Cursor::new(&mut buffer));

            log::trace!(
                "read_result for file descriptor {:?}: {read_result:?}",
                arg.fd
            );

            PROCESSES.with_process_mut(pid, |p| {
                let crate::process::FileDescriptor::Regular { offset, .. } = p
                    .file_descriptor_mut(arg.fd)
                    .expect("File descriptor not found")
                else {
                    panic!("Expected a regular file descriptor")
                };

                match read_result {
                    Ok(n) => {
                        unsafe {
                            core::ptr::copy_nonoverlapping(
                                buffer.as_ptr(),
                                arg.buf.cast::<u8>(),
                                n,
                            );
                        }
                        *offset += n;

                        arg.return_value.bytes_read = n as u32;
                        arg.return_value.error = generated::SYSCALL_READ_ERROR_NONE;
                    }
                    Err(e) => {
                        log::debug!("Failed to read from file descriptor {:?}: {e}", arg.fd);
                        arg.return_value.bytes_read = 0;
                        arg.return_value.error = generated::SYSCALL_READ_ERROR_INVALID_FD;
                    }
                }
            });
        }
        crate::process::FileDescriptor::OwnedStream { buffer, .. } => {
            if buffer.is_empty() {
                if (arg.options & generated::SYSCALL_READ_OPTION_NON_BLOCKING) != 0 {
                    arg.return_value.bytes_read = 0;
                    arg.return_value.error = generated::SYSCALL_READ_OPTION_NONE;
                    process.state = crate::process::State::Ready;
                } else {
                    process.state = crate::process::State::WaitingForStream(arg.fd);
                }
            } else {
                let bytes = buffer.drain(0..(arg.len as usize).min(buffer.len()));
                let len = bytes.len();

                for (i, byte) in bytes.enumerate() {
                    unsafe { arg.buf.cast::<u8>().add(i).write_volatile(byte) };

                    log::trace!("Read {i} bytes from stdin");
                }

                arg.return_value.bytes_read = len as u32;
                arg.return_value.error = generated::SYSCALL_READ_ERROR_NONE;

                process.state = crate::process::State::Ready;
            }
        }
        crate::process::FileDescriptor::ForeignStream { stream_type } => match stream_type {
            &mut crate::process::ForeignStreamType::Process {
                pid: other_pid,
                file_descriptor,
            } => {
                drop(process);
                let mut other_process = PROCESSES.process_mut(other_pid);
                let fd = other_process
                    .file_descriptor_mut(file_descriptor)
                    .expect("Foreign stream file descriptor not found");

                match fd {
                    crate::process::FileDescriptor::Regular { .. } => todo!(),
                    crate::process::FileDescriptor::ForeignStream { .. } => todo!(),
                    crate::process::FileDescriptor::OwnedStream { buffer, .. } => {
                        if buffer.is_empty() {
                            drop(other_process);
                            if (arg.options & generated::SYSCALL_READ_OPTION_NON_BLOCKING) != 0 {
                                arg.return_value.bytes_read = 0;
                                arg.return_value.error = generated::SYSCALL_READ_OPTION_NONE;
                                PROCESSES.process_mut(pid).state = crate::process::State::Ready;
                            } else {
                                PROCESSES.process_mut(pid).state =
                                    crate::process::State::WaitingForStream(arg.fd);
                            }
                        } else {
                            let bytes = buffer.drain(0..(arg.len as usize).min(buffer.len()));
                            let len = bytes.len();

                            for (i, byte) in bytes.enumerate() {
                                unsafe {
                                    arg.buf.cast::<u8>().add(i).write_volatile(byte);
                                }
                            }

                            arg.return_value.bytes_read = len as u32;
                            arg.return_value.error = generated::SYSCALL_READ_ERROR_NONE;

                            drop(other_process);
                            PROCESSES.process_mut(pid).state = crate::process::State::Ready;
                        }
                    }
                    crate::process::FileDescriptor::Terminal => todo!(),
                }
            }
        },
        crate::process::FileDescriptor::Terminal => todo!(),
    }
}

/// Fork the current process
/// The new process will have the same registers and state as the current process
/// The new process will be added to the process list
/// return the pid of each process in rax
fn fork(pid: u32, arg: &mut generated::syscall_fork_t) {
    let new_process = PROCESSES.process(pid).fork();

    // return 0 in the new process
    let new_pid = new_process.pid();
    PROCESSES.add_process(new_process);

    // return the pid of the new process in rax
    let process = PROCESSES.process_mut(pid);
    process.load_paging();
    arg.return_value.child_pid = new_pid;
}

/// Open a file at the path in rbx, with the length in rcx
/// Returns the file descriptor in rax
fn open(pid: u32, arg: &mut generated::syscall_open_t) {
    let path = copy_string_from_user(arg.path);

    log::trace!("syscall_handler: open '{path}'");

    if let Some(file) = crate::FILE_SYSTEM
        .try_lock()
        .expect("Failed to lock file system")
        .file_mut(&path)
    {
        let mut process = PROCESSES.process_mut(pid);

        let fd = match file {
            &mut crate::vfs::File::OwnedStream { stream_type } => {
                crate::process::FileDescriptor::OwnedStream {
                    buffer: alloc::collections::VecDeque::with_capacity(1024),
                    max_size: 1024,
                    stream_type,
                }
            }
            crate::vfs::File::Regular { .. } | crate::vfs::File::Special { .. } => {
                crate::process::FileDescriptor::Regular { path, offset: 0 }
            }
            &mut crate::vfs::File::ForeignStream { stream_type } => {
                crate::process::FileDescriptor::ForeignStream { stream_type }
            }
        };

        let fd = process.new_file_descriptor(fd);

        arg.return_value.fd = fd;
        arg.return_value.error = generated::SYSCALL_OPEN_ERROR_NONE;
    } else {
        log::debug!("File not found: {path}");

        arg.return_value.fd = -1; // indicate error
        arg.return_value.error = generated::SYSCALL_OPEN_ERROR_NOT_FOUND;
    }
}

/// Close the file descriptor in rbx
/// Returns 0 in rax on success, -1 on error
fn close(pid: u32, arg: &mut generated::syscall_close_t) {
    let mut process = PROCESSES.process_mut(pid);

    if process.close_file_descriptor(arg.fd).is_some() {
        log::trace!("syscall_handler: closed file descriptor {}", arg.fd);

        arg.return_value.error = generated::SYSCALL_CLOSE_ERROR_NONE;
    } else {
        log::debug!("Failed to close file descriptor {}", process.registers.rbx);
        arg.return_value.error = generated::SYSCALL_CLOSE_ERROR_INVALID_FD;
    }
}

/// map a new page into the process and return the address in rax
/// rbx contains the address to map to or 0
/// return the address in rax, length in rbx
fn mmap(pid: u32, arg: &mut generated::syscall_mmap_t) {
    const START_ADDRESS: u64 = 0x6942_0000_0000;

    assert!(arg.size == 4096, "mmap only supports mapping 4096 bytes");

    let mut process = PROCESSES.process_mut(pid);

    let address = process
        .mapped_pages
        .iter()
        .filter(|&m| m.page.start_address().as_u64() >= START_ADDRESS)
        .max_by_key(|&m| m.page.start_address().as_u64())
        .map_or(START_ADDRESS, |&m| m.page.start_address().as_u64() + 0x1000);
    let page = Page::containing_address(x86_64::VirtAddr::new(address));

    log::trace!(
        "mmap process {}, address {address:#x}, page {page:?}",
        process.pid()
    );

    let mut kernel_paging = crate::kernel_paging();
    let phys_frame = kernel_paging
        .allocate_frame()
        .expect("Failed to allocate frame");

    let frame_virt_addr =
        kernel_paging.page_table().phys_offset() + phys_frame.start_address().as_u64();
    (unsafe { *frame_virt_addr.as_mut_ptr::<[u8; 4096]>() }).fill(0);

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::USER_ACCESSIBLE
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_EXECUTE;

    unsafe {
        process
            .paging
            .page_table
            .map_to(page, phys_frame, flags, &mut *kernel_paging)
            .expect("Failed to map page")
            .flush();
    }

    process.mapped_pages.push(MappedPage {
        name: "heap",
        page,
        flags,
    });

    arg.return_value.addr = address as *mut _;
    arg.return_value.error = generated::SYSCALL_MMAP_ERROR_NONE;
}

/// Unmap the page at the address in rbx
fn munmap(pid: u32, arg: &mut generated::syscall_munmap_t) {
    let mut process = PROCESSES.process_mut(pid);

    let page = Page::<Size4KiB>::containing_address(x86_64::VirtAddr::new(arg.addr as u64));

    log::debug!(
        "munmap process {}, address {:#x}, page {page:?}",
        process.pid(),
        arg.addr as u64,
    );

    match process.paging.page_table.unmap(page) {
        Ok((frame, flush)) => {
            flush.flush();

            let mut kernel_paging = crate::kernel_paging();
            unsafe { kernel_paging.deallocate_frame(frame) };

            arg.return_value.error = generated::SYSCALL_MUNMAP_ERROR_NONE;
        }
        Err(e) => {
            log::warn!("Failed to unmap page at {:#x}: {e:?}", arg.addr as u64);
            process.state = crate::process::State::Terminated(2);

            arg.return_value.error = generated::SYSCALL_MUNMAP_ERROR_INVALID_ADDR;
        }
    }

    process.mapped_pages.retain(|m| m.page != page);
}

/// Execute a new program at the path in rbx (length in rcx), with the number of arguments in rdx
/// arguments in r8 is a pointer to the list of length-prefixed strings
fn execve(pid: u32, arg: &mut generated::syscall_execve_t) {
    let path = copy_string_from_user(arg.path);

    let mut argv = vec![String::new(); arg.argv_len as usize];
    for i in 0..arg.argv_len as usize {
        argv[i] = copy_string_from_user(unsafe { arg.argv.add(i).read_volatile() });
    }

    log::debug!(
        "syscall_handler: execve '{}', argc: {}, argv: {}",
        path,
        arg.argv_len,
        argv.join(", ")
    );

    match crate::FILE_SYSTEM
        .try_lock()
        .expect("Failed to lock file system")
        .file(&path)
    {
        Some(crate::vfs::File::Regular { contents }) => {
            PROCESSES.process_mut(pid).execve(contents, &argv);
        }
        Some(_) => {
            log::debug!("Cannot execute special file: {path}");
            arg.return_value.error = generated::SYSCALL_EXECVE_ERROR_NOT_FOUND;
        }
        None => {
            log::debug!("File not found: {path}");
            arg.return_value.error = generated::SYSCALL_EXECVE_ERROR_NOT_FOUND;
        }
    }
}

fn map_framebuffer(pid: u32, arg: &mut generated::syscall_map_framebuffer_t) {
    log::debug!("syscall_handler: map framebuffer");

    let mut kernel_paging = crate::kernel_paging();
    let mut process = PROCESSES.process_mut(pid);

    let size = crate::term::TERM.framebuffer.height * crate::term::TERM.framebuffer.width * 4;

    let start_phys_address = kernel_paging
        .translate_addr(x86_64::VirtAddr::new(
            crate::term::TERM.framebuffer.ptr as u64,
        ))
        .expect("Failed to translate framebuffer address");
    let start_phys_frame = start_phys_address.align_down(Size4KiB::SIZE);

    let start_address = 0x3333_4444_0000;

    log::debug!(
        "mapping framebuffer at {start_address:#x} to {start_phys_frame:#x} ({start_phys_address:#x}) with size {size} bytes",
    );

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE
        | PageTableFlags::NO_EXECUTE
        // WT bit so PAT index is 1
        | PageTableFlags::WRITE_THROUGH;

    let mut mapped = 0;
    while mapped < size {
        let frame = x86_64::structures::paging::PhysFrame::<Size4KiB>::containing_address(
            start_phys_frame + mapped,
        );

        unsafe {
            process
                .paging
                .page_table
                .map_to(
                    Page::<Size4KiB>::containing_address(x86_64::VirtAddr::new(
                        start_address + mapped,
                    )),
                    frame,
                    flags,
                    &mut *kernel_paging,
                )
                .expect("Failed to map framebuffer page")
                .flush();
        }

        mapped += Size4KiB::SIZE; // 4KiB
    }

    arg.return_value = generated::syscall_map_framebuffer_return_t {
        addr: (start_address + start_phys_address.as_u64() % 0x1000) as *mut _,
        width: crate::term::TERM.framebuffer.width as u32,
        height: crate::term::TERM.framebuffer.height as u32,
    };
}

fn write(pid: u32, arg: &mut generated::syscall_write_t) {
    log::trace!(
        "syscall_handler: write fd {}, buffer {:x}, length {}",
        arg.fd,
        arg.buf as u64,
        arg.len
    );

    let mut process = PROCESSES.process_mut(pid);
    let Some(mut fd) = process.file_descriptor_mut(arg.fd) else {
        log::debug!("Invalid file descriptor: {}", arg.fd);
        arg.return_value.bytes_written = 0;
        arg.return_value.error = generated::SYSCALL_WRITE_ERROR_INVALID_FD;
        return;
    };

    if let &mut crate::process::FileDescriptor::ForeignStream { stream_type } = fd {
        drop(process);
        let (other_pid, other_fd) =
            resolve_foreign_stream(stream_type).expect("failed to resolve fd");
        process = PROCESSES.process_mut(other_pid);
        fd = process
            .file_descriptor_mut(other_fd)
            .expect("Foreign stream file descriptor not found");
    }

    match fd {
        crate::process::FileDescriptor::Regular { path, offset } => {
            todo!(
                "write to regular file, pid {pid}, fd {}, path {path}, offset {offset}",
                arg.fd
            )
        }
        crate::process::FileDescriptor::OwnedStream {
            buffer, max_size, ..
        } => {
            if buffer.len() >= *max_size {
                arg.return_value.bytes_written = 0;
                arg.return_value.error = generated::SYSCALL_WRITE_ERROR_NONE;
                return;
            }

            let bytes_to_write = (arg.len as usize).min(*max_size - buffer.len());
            for i in 0..bytes_to_write {
                let byte = unsafe { arg.buf.cast::<u8>().add(i).read_volatile() };
                buffer.push_back(byte);
            }

            arg.return_value.bytes_written = bytes_to_write as u32;
            arg.return_value.error = generated::SYSCALL_WRITE_ERROR_NONE;
        }
        crate::process::FileDescriptor::ForeignStream { .. } => {
            unreachable!("foreign streams should be resolved before writing")
        }
        crate::process::FileDescriptor::Terminal => {
            let mut writer = crate::term::TERM.writer();

            let mut vec = vec![0; arg.len as usize];
            unsafe {
                core::ptr::copy_nonoverlapping(
                    arg.buf.cast::<u8>(),
                    vec.as_mut_ptr(),
                    arg.len as usize,
                );
            }

            let str = String::from_utf8(vec).unwrap_or_else(|e| {
                panic!(
                    "Invalid UTF-8 string in syscall_write: len {}: {}",
                    arg.len,
                    e.utf8_error()
                )
            });
            writer.write_str(&str).expect("Failed to write to terminal");

            arg.return_value.bytes_written = arg.len;
            arg.return_value.error = generated::SYSCALL_WRITE_ERROR_NONE;
        }
    }
}

fn resolve_foreign_stream(stream: crate::process::ForeignStreamType) -> Option<(u32, i32)> {
    log::debug!("resolving foreign stream: {stream:?}");
    match stream {
        crate::process::ForeignStreamType::Process {
            pid,
            file_descriptor,
        } => {
            let process = PROCESSES.process(pid);
            match process.file_descriptor(file_descriptor) {
                Some(crate::process::FileDescriptor::OwnedStream { .. }) => {
                    Some((pid, file_descriptor))
                }
                Some(&crate::process::FileDescriptor::ForeignStream { stream_type }) => {
                    drop(process);
                    resolve_foreign_stream(stream_type)
                }
                _ => None,
            }
        }
    }
}

fn waitpid(pid: u32, arg: &mut generated::syscall_waitpid_t) {
    PROCESSES.process_mut(pid).state = crate::process::State::WaitingForChild {
        pid: arg.pid,
        arg: core::ptr::from_mut(arg),
    };
}

/// Handle system calls
/// return true if the process still exists, false if it was terminated
pub fn handle_syscall(pid: u32) {
    let (rax, rbx) = crate::process::PROCESSES.with_process(pid, |p| {
        log::trace!("process {pid} called syscall {:#x}", p.registers.rax);

        p.load_paging();

        (p.registers.rax, p.registers.rbx)
    });

    match rax {
        0 => print(pid, unsafe { &mut *(rbx as *mut _) }),
        1 => sleep(pid, unsafe { &mut *(rbx as *mut _) }),
        2 => exit(pid, unsafe { &mut *(rbx as *mut _) }),
        3 => list_directory(pid, unsafe { &mut *(rbx as *mut _) }),
        4 => read(pid, unsafe { &mut *(rbx as *mut _) }),
        5 => fork(pid, unsafe { &mut *(rbx as *mut _) }),
        6 => open(pid, unsafe { &mut *(rbx as *mut _) }),
        7 => close(pid, unsafe { &mut *(rbx as *mut _) }),
        8 => mmap(pid, unsafe { &mut *(rbx as *mut _) }),
        9 => munmap(pid, unsafe { &mut *(rbx as *mut _) }),
        10 => execve(pid, unsafe { &mut *(rbx as *mut _) }),
        11 => map_framebuffer(pid, unsafe { &mut *(rbx as *mut _) }),
        12 => write(pid, unsafe { &mut *(rbx as *mut _) }),
        13 => waitpid(pid, unsafe { &mut *(rbx as *mut _) }),
        n => panic!("unknown syscall: {n:#x}"),
    }
}
