use alloc::{
    string::{String, ToString},
    vec,
};
use core::fmt::Write as _;
use log::trace;
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator as _, Mapper, Page, PageTableFlags, Size4KiB, Translate,
};

use crate::process;

/// Print the string in rbx with length rcx
fn print(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    let process = process_lock.get();

    let mut bytes = vec![0; process.registers.rcx as usize];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = unsafe { (process.registers.rbx as *const u8).add(i).read() };
    }

    let string = String::from_utf8(bytes).expect("Invalid UTF-8 string");

    write!(crate::TERM.writer(), "{}", string).expect("Failed to write to terminal");
}

/// sleep for the number of milliseconds in rbx
fn sleep(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    let process = process_lock.get();

    process.state = crate::process::State::Sleeping(unsafe {
        crate::i8253::TIMER0.ticks() + process.registers.rbx / 100
    });
}

/// Exit the current process with the exit code in rbx
fn exit(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    let process = process_lock.get();

    trace!("syscall_handler: exit {:#x}", process.registers.rbx);
    process.state = crate::process::State::Terminated(process.registers.rbx);

    log::debug!(
        "Process {} exited with code {}",
        process.pid(),
        process.registers.rbx
    );
}

/// Get the name of the entry at index rdx in the directory at path in rbx
/// Returns the name to the pointer in r8 and the length of the name in rax
fn list_directory(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    let process = process_lock.get();

    let mut path = vec![0; process.registers.rcx as usize];
    for (i, byte) in path.iter_mut().enumerate() {
        *byte = unsafe { (process.registers.rbx as *const u8).add(i).read() };
    }
    let path_str = core::str::from_utf8(&path).expect("Invalid UTF-8 string");
    let index = process.registers.rdx as usize;
    let return_ptr = process.registers.r8 as *mut u8;

    trace!(
        "syscall_handler: list directory '{}', index {}, return to {:?}",
        path_str,
        index,
        return_ptr
    );

    let fs = crate::FILE_SYSTEM
        .try_lock()
        .expect("Failed to lock file system");
    process.registers.rax = match fs.directory(path_str) {
        Some(crate::vfs::Directory { files, directories }) => {
            if index < files.len() {
                let file = files.iter().nth(index).unwrap();
                for (i, byte) in file.0.as_bytes().iter().enumerate() {
                    unsafe { return_ptr.add(i).write_volatile(*byte) };
                }
                file.0.len() as u64
            } else if index < files.len() + directories.len() {
                let dir = directories.iter().nth(index - files.len()).unwrap();
                for (i, byte) in dir.0.as_bytes().iter().enumerate() {
                    unsafe { return_ptr.add(i).write_volatile(*byte) };
                }
                dir.0.len() as u64
            } else {
                0
            }
        }
        None => {
            log::debug!("Directory not found: {}", path_str);
            0
        }
    };
}

/// Read from the file descriptor in rbx into the buffer in rcx with length rdx
fn read(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    let process = process_lock.get();

    let fd = process.registers.rbx;
    let dest = process.registers.rcx as usize;
    let length = process.registers.rdx as usize;

    log::trace!("syscall_handler: read fd {fd}, buffer {dest:#0x}, length {length}");

    match fd {
        // 0 is stdin
        0 => {
            if process.stdin.is_empty() {
                process.state = crate::process::State::WaitingForStdin;
                log::trace!("Process {} is waiting for stdin", process.pid());
            } else {
                for i in 0..length {
                    match process.stdin.pop_front() {
                        Some(byte) => {
                            unsafe { ((dest + i) as *mut u8).write_volatile(byte) };
                        }
                        None => {
                            process.registers.rax = i as u64; // Number of bytes read
                            log::trace!("Read {i} bytes from stdin");
                            break;
                        }
                    }
                }
            }
        }
        fd => {
            log::trace!("syscall_handler: read from file descriptor {fd}");

            if let Some(fd) = process.file_descriptor_mut(process::FileDescriptor::from_u64(fd)) {
                let mut fs = crate::FILE_SYSTEM
                    .try_lock()
                    .expect("Failed to lock file system");
                let file = fs.file_mut(&fd.path).expect("File not found");

                let mut buffer = vec![0; length];

                let n = file
                    .read(fd.offset, crate::io::Cursor::new(&mut buffer))
                    .expect("Failed to read file");

                for i in 0..n {
                    unsafe { ((dest + i) as *mut u8).write_volatile(buffer[i]) };
                }

                fd.offset += n;
                process.registers.rax = n as u64;
            } else {
                log::debug!("Invalid file descriptor: {}", fd);
                process.registers.rax = -1i64 as u64; // indicate error
            }
        }
    }
}

/// Fork the current process
/// The new process will have the same registers and state as the current process
/// The new process will be added to the process list
/// return the pid of each process in rax
fn fork(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    let mut new_process = process_lock.get().fork();
    let pid = new_process.pid();
    // return 0 in the new process
    new_process.registers.rax = 0;
    process_lock.get_processes().push_back(new_process);

    // return the pid of the new process in rax
    process_lock.get().registers.rax = pid as u64;
}

/// Open a file at the path in rbx, with the length in rcx
/// Returns the file descriptor in rax
fn open(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    let process = process_lock.get();

    let mut path = vec![0; process.registers.rcx as usize];
    for (i, byte) in path.iter_mut().enumerate() {
        *byte = unsafe { (process.registers.rbx as *const u8).add(i).read() };
    }
    let path_str = core::str::from_utf8(&path).expect("Invalid UTF-8 string");

    log::trace!("syscall_handler: open '{}'", path_str);

    if let Some(_file) = crate::FILE_SYSTEM
        .try_lock()
        .expect("Failed to lock file system")
        .file_mut(path_str)
    {
        let fd = process.new_file_descriptor(crate::process::ProcessFileDescriptor {
            path: path_str.to_string(),
            offset: 0,
        });

        process.registers.rax = fd.as_u64();
    } else {
        log::debug!("File not found: {}", path_str);
        process.registers.rax = -1i64 as u64; // indicate error
    }
}

/// Close the file descriptor in rbx
/// Returns 0 in rax on success, -1 on error
fn close(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    let process = process_lock.get();

    match process.close_file_descriptor(crate::process::FileDescriptor::from_u64(
        process.registers.rbx,
    )) {
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
fn mmap(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    const START_ADDRESS: u64 = 0x6942_0000_0000;

    let process = process_lock.get();

    let address = if process.registers.rbx == 0 {
        process
            .mapped_pages
            .iter()
            .filter(|&(page, _)| page.start_address().as_u64() >= START_ADDRESS)
            .max_by_key(|&(page, _)| page.start_address().as_u64())
            .map_or(START_ADDRESS, |(page, _)| {
                page.start_address().as_u64() + 4096
            })
    } else {
        process.registers.rbx
    };
    let page = Page::containing_address(x86_64::VirtAddr::new(address));

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

    process.mapped_pages.push((page, flags));

    process.registers.rax = address;
    process.registers.rbx = 4096;
}

/// Unmap the page at the address in rbx
fn munmap(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    let process = process_lock.get();

    let address = process.registers.rbx;
    let page = Page::<Size4KiB>::containing_address(x86_64::VirtAddr::new(address));

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
}

/// Handle system calls
/// return true if the process still exists, false if it was terminated
pub fn handle_syscall(process_lock: &mut crate::process::IndexedProcessGuard<'_>) {
    log::trace!(
        "syscall_handler: process {} called syscall {:#x}",
        process_lock.get().pid(),
        process_lock.get().registers.rax
    );

    match process_lock.get().registers.rax {
        0 => print(process_lock),
        1 => sleep(process_lock),
        2 => exit(process_lock),
        3 => list_directory(process_lock),
        4 => read(process_lock),
        5 => fork(process_lock),
        6 => open(process_lock),
        7 => close(process_lock),
        8 => mmap(process_lock),
        9 => munmap(process_lock),
        n => panic!("unknown syscall: {n:#x}"),
    }
}
