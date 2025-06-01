use core::sync::atomic::AtomicU32;

use alloc::{borrow::ToOwned as _, collections::vec_deque::VecDeque, vec::Vec};
use x86_64::structures::paging::{FrameAllocator as _, Mapper};

use crate::kernel::paging::SoosPaging;

struct PidFactory {
    next_pid: AtomicU32,
}
impl PidFactory {
    pub const fn new() -> Self {
        PidFactory {
            next_pid: AtomicU32::new(0),
        }
    }

    pub fn next_pid(&self) -> u32 {
        self.next_pid
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed)
    }
}

static PID_FACTORY: PidFactory = PidFactory::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Ready,
    Sleeping(u64),
    WaitingForStdin,
    Terminated(u64),
}

pub struct Process {
    pid: u32,
    pub state: State,
    paging: SoosPaging<'static>,
    cs: x86_64::structures::gdt::SegmentSelector,
    ds: x86_64::structures::gdt::SegmentSelector,
    pub flags: u64,
    pub rip: u64,
    pub registers: crate::idt::GPRegisters,
    pub stdin: alloc::collections::VecDeque<u8>,
    mapped_pages: Vec<(
        x86_64::structures::paging::Page,
        x86_64::structures::paging::PageTableFlags,
    )>,
    file_descriptors: alloc::collections::BTreeMap<FileDescriptor, ProcessFileDescriptor>,
}

#[derive(Debug, Clone)]
pub struct ProcessFileDescriptor {
    pub path: alloc::string::String,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct FileDescriptor(u64);

impl FileDescriptor {
    const fn from_index(fd: usize) -> Self {
        FileDescriptor(fd as u64 + 3) // 0, 1, 2 are reserved for stdin, stdout, stderr
    }

    pub const fn from_u64(fd: u64) -> Self {
        FileDescriptor(fd)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

pub static PROCESSES: spin::Lazy<spin::Mutex<VecDeque<Process>>> =
    spin::Lazy::new(|| spin::Mutex::new(VecDeque::new()));
pub static CURRENT_PROCESS: AtomicU32 = AtomicU32::new(0);

impl Process {
    pub fn user_from_elf(
        hhdm_offset: u64,
        cs: x86_64::structures::gdt::SegmentSelector,
        ds: x86_64::structures::gdt::SegmentSelector,
        flags: u64,
        elf: &[u8],
    ) -> Self {
        let pid = PID_FACTORY.next_pid();

        let frame_allocator = unsafe {
            crate::kernel::paging::SOOS_FRAME_ALLOCATOR
                .as_mut()
                .expect("Frame allocator not initialized!")
        };

        let process_page_table = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!")
            .start_address()
            .as_u64()
            as *mut x86_64::structures::paging::PageTable;

        (unsafe { &*crate::KERNEL_PAGING })
            .offset_page_table
            .level_4_table()
            .clone_into(unsafe { &mut *process_page_table });

        log::debug!(
            "page table for pid {pid}: {:#x}",
            process_page_table as *const _ as u64
        );

        let mut paging =
            SoosPaging::offset_page_table(hhdm_offset, unsafe { &mut *process_page_table });

        let (userspace_address, userspace_stack, mapped_pages) = crate::elf::load(
            &mut paging,
            unsafe { &mut *crate::KERNEL_PAGING },
            frame_allocator,
            elf,
            x86_64::VirtAddr::new(0x0000_1234_0000_0000),
        );

        log::debug!("elf for pid {pid} loaded at address {userspace_address:#x}, stack at {userspace_stack:#x}");

        Process {
            pid: PID_FACTORY.next_pid(),
            state: State::Ready,
            paging,
            cs,
            ds,
            flags,
            rip: userspace_address.as_u64(),
            registers: crate::idt::GPRegisters {
                rsp: userspace_stack.as_u64(),
                ..Default::default()
            },
            stdin: alloc::collections::VecDeque::new(),
            mapped_pages,
            file_descriptors: alloc::collections::BTreeMap::new(),
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    fn update_state(process_lock: &mut IndexedProcessGuard<'_>) {
        match process_lock.get().state {
            State::Sleeping(target) => {
                let ticks = unsafe { crate::i8253::TIMER0.ticks() };
                if ticks >= target {
                    process_lock.get().state = State::Ready;
                }
            }
            State::WaitingForStdin => {
                if !process_lock.get().stdin.is_empty() {
                    crate::syscall::handle_syscall(process_lock);
                    process_lock.get().state = State::Ready;
                }
            }
            _ => {}
        }
    }

    pub fn load_paging(&mut self) {
        self.paging.load();
    }

    pub fn fork(&self) -> Process {
        let paging = self.paging.fork(self.mapped_pages.as_slice());

        Process {
            pid: PID_FACTORY.next_pid(),
            state: self.state,
            paging,
            cs: self.cs,
            ds: self.ds,
            flags: self.flags,
            rip: self.rip,
            registers: self.registers,
            stdin: self.stdin.clone(),
            mapped_pages: self.mapped_pages.clone(),
            file_descriptors: self.file_descriptors.clone(),
        }
    }

    pub fn new_file_descriptor(&mut self, fd: ProcessFileDescriptor) -> FileDescriptor {
        let index = self.file_descriptors.len();
        let file_descriptor = FileDescriptor::from_index(index);
        self.file_descriptors.insert(file_descriptor, fd);
        file_descriptor
    }

    pub fn close_file_descriptor(&mut self, fd: FileDescriptor) -> Option<ProcessFileDescriptor> {
        self.file_descriptors.remove(&fd)
    }

    pub fn file_descriptor_mut(
        &mut self,
        fd: FileDescriptor,
    ) -> Option<&mut ProcessFileDescriptor> {
        self.file_descriptors.get_mut(&fd)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        for &(page, _flags) in &self.mapped_pages {
            self.paging
                .offset_page_table
                .unmap(page)
                .expect("Failed to unmap page")
                .1
                .flush();
        }
    }
}

pub struct IndexedProcessGuard<'a> {
    processes: spin::MutexGuard<'a, VecDeque<Process>>,
    index: usize,
}

impl IndexedProcessGuard<'_> {
    pub fn get(&mut self) -> &mut Process {
        &mut self.processes[self.index]
    }

    pub fn get_processes(&mut self) -> &mut VecDeque<Process> {
        &mut self.processes
    }
}

pub fn current_process_mut() -> Result<IndexedProcessGuard<'static>, ()> {
    let pid = CURRENT_PROCESS.load(core::sync::atomic::Ordering::Relaxed);

    let mut processes = PROCESSES.try_lock().ok_or(())?;

    Ok(processes
        .iter_mut()
        .position(|p| p.pid == pid)
        .map(|index| IndexedProcessGuard { processes, index })
        .expect("Current process not found"))
}

pub fn try_schedule() -> Option<!> {
    loop {
        match PROCESSES.try_lock() {
            Some(mut processes) => {
                processes.retain(|p| !matches!(p.state, State::Terminated(_)));

                if processes.len() == 0 {
                    log::error!("No user processes left, halting forever!. Goodbye!");
                    x86_64::instructions::interrupts::disable();
                    x86_64::instructions::hlt();
                }

                let len = processes.len();
                let mut lock = IndexedProcessGuard {
                    processes,
                    index: 0,
                };
                for index in 0..len {
                    lock.index = index;
                    Process::update_state(&mut lock);
                }

                let mut processes = lock.processes;

                // rotate through processes until we find one that is ready
                for _ in 0..processes.len() {
                    let process = processes.front_mut().expect("No processes left!");

                    if process.state == State::Ready {
                        log::trace!("Scheduling {}", process.pid);

                        CURRENT_PROCESS.store(process.pid, core::sync::atomic::Ordering::Relaxed);

                        process.load_paging();

                        let cs = process.cs.0 as u64;
                        let ds = process.ds.0 as u64;
                        let flags = process.flags;
                        let rip = process.rip;
                        let registers = process.registers;

                        processes.rotate_left(1);
                        drop(processes);

                        unsafe {
                            crate::do_iret(cs, ds, flags, rip, &registers);
                        };
                    } else {
                        processes.rotate_left(1);
                    }
                }

                drop(processes);
                x86_64::instructions::interrupts::enable_and_hlt();
                x86_64::instructions::interrupts::disable();
                continue;
            }
            None => return None,
        }
    }
}
