use alloc::{
    string::{String, ToString},
    vec,
};
use core::fmt::Write as _;
use log::trace;
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator as _, Mapper, Page, PageTableFlags, Size4KiB,
};

use crate::process::{FileDescriptor, MappedPage, PROCESSES};

/// Print the string in rbx with length rcx
fn print(pid: u32) {
    let process = PROCESSES.process_mut(pid);

    let mut bytes = vec![0; process.registers.rcx as usize];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = unsafe { (process.registers.rbx as *const u8).add(i).read() };
    }

    let string = String::from_utf8(bytes).expect("Invalid UTF-8 string");

    write!(crate::TERM.writer(), "{string}").expect("Failed to write to terminal");
}

/// sleep for the number of milliseconds in rbx
fn sleep(pid: u32) {
    PROCESSES.with_process_mut(pid, |p| {
        trace!(
            "syscall_handler: sleep for {} milliseconds",
            p.registers.rbx
        );

        p.state = crate::process::State::Sleeping(unsafe {
            crate::i8253::TIMER0.ticks() + p.registers.rbx / 100
        });
    });
}

/// Exit the current process with the exit code in rbx
fn exit(pid: u32) {
    PROCESSES.with_process_mut(pid, |p| {
        trace!("syscall_handler: exit {:#x}", p.registers.rbx);
        p.state = crate::process::State::Terminated(p.registers.rbx);

        log::debug!("Process {} exited with code {}", p.pid(), p.registers.rbx);
    });
}

/// Get the name of the entry at index rdx in the directory at path in rbx
/// Returns the name to the pointer in r8 and the length of the name in rax
fn list_directory(pid: u32) {
    let process = PROCESSES.process(pid);

    let mut path = vec![0; process.registers.rcx as usize];
    for (i, byte) in path.iter_mut().enumerate() {
        *byte = unsafe { (process.registers.rbx as *const u8).add(i).read() };
    }
    let path_str = core::str::from_utf8(&path).expect("Invalid UTF-8 string");
    let index = process.registers.rdx as usize;
    let return_ptr = process.registers.r8 as *mut u8;

    trace!("syscall_handler: list directory '{path_str}', index {index}, return to {return_ptr:?}");

    let mut fs = crate::FILE_SYSTEM
        .try_lock()
        .expect("Failed to lock file system");

    drop(process); // Drop the process to avoid borrowing issues

    let rax = if let Some(dir) = fs.directory_mut(path_str) {
        let files_len = dir.files().len();
        let directories_len = dir.directories().len();

        if index < files_len {
            let file = dir.files().iter().nth(index).unwrap();
            for (i, byte) in file.0.as_bytes().iter().enumerate() {
                unsafe { return_ptr.add(i).write_volatile(*byte) };
            }
            file.0.len() as u64
        } else if index < files_len + directories_len {
            let dir = dir.directories().iter().nth(index - files_len).unwrap();
            for (i, byte) in dir.0.as_bytes().iter().enumerate() {
                unsafe { return_ptr.add(i).write_volatile(*byte) };
            }
            dir.0.len() as u64
        } else {
            0
        }
    } else {
        log::debug!("Directory not found: {path_str}");
        0
    };

    PROCESSES.process_mut(pid).registers.rax = rax;
}

/// Read from the file descriptor in rbx into the buffer in rcx with length rdx
fn read(pid: u32) {
    let (fd, dest, length) = PROCESSES.with_process(pid, |process| {
        (
            process.registers.rbx,
            process.registers.rcx as usize,
            process.registers.rdx as usize,
        )
    });

    log::trace!("syscall_handler: read fd {fd}, buffer {dest:#0x}, length {length}");

    match fd {
        // 0 is stdin
        0 => {
            PROCESSES.with_process_mut(pid, |p| {
                if p.stdin.is_empty() {
                    p.state = crate::process::State::WaitingForStdin;
                    log::trace!("Process {} is waiting for stdin", p.pid());
                } else {
                    for i in 0..length {
                        if let Some(byte) = p.stdin.pop_front() {
                            unsafe { ((dest + i) as *mut u8).write_volatile(byte) };
                        } else {
                            p.registers.rax = i as u64; // Number of bytes read
                            log::trace!("Read {i} bytes from stdin");
                            break;
                        }
                    }
                }
            });
        }
        fd => {
            log::trace!("syscall_handler: read from file descriptor {fd}");

            let process = PROCESSES.process(pid);
            let descriptor = process.file_descriptor(FileDescriptor::from_u64(fd));

            if let Some(file_descriptor) = descriptor {
                let offset = file_descriptor.offset;
                let path = file_descriptor.path.clone();
                drop(process);

                let mut fs = crate::FILE_SYSTEM
                    .try_lock()
                    .expect("Failed to lock file system");
                let file = fs.file_mut(&path).expect("File not found");

                let mut buffer = vec![0; length];

                log::debug!("reading from file '{path}' at offset {offset}, length {length}");

                let read_result = file.read(offset, crate::io::Cursor::new(&mut buffer));

                log::debug!("read_result for file descriptor {fd:?}: {read_result:?}");

                PROCESSES.with_process_mut(pid, |p| {
                    let fd = p
                        .file_descriptor_mut(FileDescriptor::from_u64(fd))
                        .expect("File descriptor not found");

                    match read_result {
                        Ok(n) => {
                            for i in 0..n {
                                unsafe { ((dest + i) as *mut u8).write_volatile(buffer[i]) };
                            }
                            fd.offset += n;
                            p.registers.rax = n as u64; // Number of bytes read
                        }
                        Err(e) => {
                            log::debug!("Failed to read from file descriptor {fd:?}: {e}");
                            p.registers.rax = -1i64 as u64;
                        }
                    }
                });
            } else {
                log::debug!("Invalid file descriptor: {fd}");
                PROCESSES.with_process_mut(pid, |p| {
                    p.registers.rax = -1i64 as u64; // indicate error
                });
            }
        }
    }
}

/// Fork the current process
/// The new process will have the same registers and state as the current process
/// The new process will be added to the process list
/// return the pid of each process in rax
fn fork(pid: u32) {
    let mut new_process = PROCESSES.process(pid).fork();

    // return 0 in the new process
    new_process.registers.rax = 0;
    PROCESSES.add_process(new_process);

    // return the pid of the new process in rax
    PROCESSES.process_mut(pid).registers.rax = u64::from(pid);
}

/// Open a file at the path in rbx, with the length in rcx
/// Returns the file descriptor in rax
fn open(pid: u32) {
    let process = PROCESSES.process(pid);

    let mut path = vec![0; process.registers.rcx as usize];
    for (i, byte) in path.iter_mut().enumerate() {
        *byte = unsafe { (process.registers.rbx as *const u8).add(i).read() };
    }
    let path_str = core::str::from_utf8(&path).expect("Invalid UTF-8 string");

    log::trace!("syscall_handler: open '{path_str}'");

    drop(process);

    if let Some(_file) = crate::FILE_SYSTEM
        .try_lock()
        .expect("Failed to lock file system")
        .file_mut(path_str)
    {
        let mut process = PROCESSES.process_mut(pid);

        let fd = process.new_file_descriptor(crate::process::ProcessFileDescriptor {
            path: path_str.to_string(),
            offset: 0,
        });

        process.registers.rax = fd.as_u64();
    } else {
        log::debug!("File not found: {path_str}");

        PROCESSES.process_mut(pid).registers.rax = -1i64 as u64; // indicate error
    }
}

/// Close the file descriptor in rbx
/// Returns 0 in rax on success, -1 on error
fn close(pid: u32) {
    let mut process = PROCESSES.process_mut(pid);
    let fd = process.registers.rbx;

    match process.close_file_descriptor(FileDescriptor::from_u64(fd)) {
        Some(_) => {
            log::trace!(
                "syscall_handler: closed file descriptor {}",
                process.registers.rbx
            );
            process.registers.rax = 0; // success
        }
        None => {
            log::debug!("Failed to close file descriptor {}", process.registers.rbx);
            process.registers.rax = -1i64 as u64; // indicate error
        }
    }
}

/// map a new page into the process and return the address in rax
/// rbx contains the address to map to or 0
/// return the address in rax, length in rbx
fn mmap(pid: u32) {
    const START_ADDRESS: u64 = 0x6942_0000_0000;

    let mut process = PROCESSES.process_mut(pid);

    let address = if process.registers.rbx == 0 {
        process
            .mapped_pages
            .iter()
            .filter(|&m| m.page.start_address().as_u64() >= START_ADDRESS)
            .max_by_key(|&m| m.page.start_address().as_u64())
            .map_or(START_ADDRESS, |&m| m.page.start_address().as_u64() + 4096)
    } else {
        process.registers.rbx
    };
    let page = Page::containing_address(x86_64::VirtAddr::new(address));

    log::debug!(
        "mmap process {}, address {address:#x}, page {page:?}",
        process.pid()
    );

    let mut kernel_paging = crate::kernel_paging();
    let phys_frame = kernel_paging
        .frame_allocator
        .allocate_frame()
        .expect("Failed to allocate frame");

    let frame_virt_addr =
        kernel_paging.page_table.phys_offset() + phys_frame.start_address().as_u64();
    (unsafe { *frame_virt_addr.as_mut_ptr::<[u8; 4096]>() }).fill(0);

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::USER_ACCESSIBLE
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_EXECUTE;

    unsafe {
        process
            .paging
            .page_table
            .map_to(page, phys_frame, flags, &mut kernel_paging.frame_allocator)
            .expect("Failed to map page")
            .flush();
    }

    process.mapped_pages.push(MappedPage {
        name: "heap",
        page,
        flags,
    });

    process.registers.rax = address;
    process.registers.rbx = 4096;
}

/// Unmap the page at the address in rbx
fn munmap(pid: u32) {
    let mut process = PROCESSES.process_mut(pid);

    let address = process.registers.rbx;
    let page = Page::<Size4KiB>::containing_address(x86_64::VirtAddr::new(address));

    log::debug!(
        "munmap process {}, address {address:#x}, page {page:?}",
        process.pid()
    );

    match process.paging.page_table.unmap(page) {
        Ok((frame, flush)) => {
            flush.flush();

            let mut kernel_paging = crate::kernel_paging();
            unsafe { kernel_paging.frame_allocator.deallocate_frame(frame) };
        }
        Err(e) => {
            log::warn!("Failed to unmap page at {address:#x}: {e:?}");
            process.state = crate::process::State::Terminated(2);
        }
    }

    process.mapped_pages.retain(|m| m.page != page);
}

/// Execute a new program at the path in rbx (length in rcx), with the number of arguments in rdx
/// arguments in r8 is a pointer to the list of length-prefixed strings
fn execve(pid: u32) {
    let process = PROCESSES.process(pid);

    let mut path = vec![0; process.registers.rcx as usize];
    for (i, byte) in path.iter_mut().enumerate() {
        *byte = unsafe { (process.registers.rbx as *const u8).add(i).read() };
    }
    let path_str = core::str::from_utf8(&path).expect("Invalid UTF-8 string");

    let argc = process.registers.rdx as usize;

    log::debug!(
        "syscall_handler: execve '{path_str}', argc: {argc}, argv: {:#x?}",
        process.registers.r8
    );

    let mut argv = alloc::vec::Vec::with_capacity(argc);
    for i in 0..argc {
        #[repr(C)]
        struct Arg {
            len: u64,
            ptr: *const u64,
        }

        let arg = unsafe { (process.registers.r8 as *const Arg).add(i).read_volatile() };

        log::debug!(
            "syscall_handler: execve arg {i}: len = {}, ptr = {:#x?}",
            arg.len,
            arg.ptr
        );

        let mut string = alloc::vec::Vec::with_capacity(arg.len as usize);
        for j in 0..arg.len {
            let byte = unsafe { arg.ptr.cast::<u8>().add(j as usize).read_volatile() };
            string.push(byte);
        }

        let arg_str = core::str::from_utf8(&string).expect("Invalid UTF-8 string");

        argv.push(arg_str.to_string());
    }

    drop(process); // Drop the process to avoid borrowing issues

    match crate::FILE_SYSTEM
        .try_lock()
        .expect("Failed to lock file system")
        .file(path_str)
    {
        Some(crate::vfs::File::Regular { contents }) => {
            PROCESSES.process_mut(pid).execve(contents);
        }
        Some(_) => {
            log::debug!("Cannot execute special file: {path_str}");
            PROCESSES.process_mut(pid).registers.rax = -1i64 as u64; // indicate error
        }
        None => {
            log::debug!("File not found: {path_str}");
            PROCESSES.process_mut(pid).registers.rax = -1i64 as u64; // indicate error
        }
    }
}

/// Handle system calls
/// return true if the process still exists, false if it was terminated
pub fn handle_syscall(pid: u32) {
    let rax = crate::process::PROCESSES.with_process(pid, |p| {
        log::trace!(
            "syscall_handler: process {pid} called syscall {:#x}",
            p.registers.rax
        );

        p.registers.rax
    });

    match rax {
        0 => print(pid),
        1 => sleep(pid),
        2 => exit(pid),
        3 => list_directory(pid),
        4 => read(pid),
        5 => fork(pid),
        6 => open(pid),
        7 => close(pid),
        8 => mmap(pid),
        9 => munmap(pid),
        10 => execve(pid),
        n => panic!("unknown syscall: {n:#x}"),
    }
}
