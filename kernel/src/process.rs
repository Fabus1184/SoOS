use core::{cell::RefCell, sync::atomic::AtomicU32};

use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use x86_64::structures::paging::{FrameDeallocator, Mapper};

use crate::kernel::paging::UserspacePaging;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedPage {
    pub name: &'static str,
    pub page: x86_64::structures::paging::Page,
    pub flags: x86_64::structures::paging::PageTableFlags,
}

pub struct Process {
    pid: u32,
    pub state: State,
    pub paging: UserspacePaging<'static>,
    cs: x86_64::structures::gdt::SegmentSelector,
    ds: x86_64::structures::gdt::SegmentSelector,
    pub flags: u64,
    pub rip: u64,
    pub registers: crate::idt::GPRegisters,
    pub stdin: alloc::collections::VecDeque<u8>,
    pub mapped_pages: Vec<MappedPage>,
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

pub struct Processes {
    processes: RefCell<VecDeque<Process>>,
    current_pid: AtomicU32,
}

impl Processes {
    pub fn process(&self, pid: u32) -> impl core::ops::Deref<Target = Process> + '_ {
        core::cell::Ref::map(self.processes.borrow(), |processes| {
            processes
                .iter()
                .find(|p| p.pid == pid)
                .expect("Process not found")
        })
    }

    pub fn process_mut(&self, pid: u32) -> impl core::ops::DerefMut<Target = Process> + '_ {
        core::cell::RefMut::map(self.processes.borrow_mut(), |processes| {
            processes
                .iter_mut()
                .find(|p| p.pid == pid)
                .expect("Process not found")
        })
    }

    pub fn with_process<F, R>(&self, pid: u32, f: F) -> R
    where
        F: FnOnce(&Process) -> R,
    {
        let processes = self.processes.borrow();
        processes
            .iter()
            .find(|p| p.pid == pid)
            .map(f)
            .expect("Process not found")
    }

    pub fn with_process_mut<F, R>(&self, pid: u32, f: F) -> R
    where
        F: FnOnce(&mut Process) -> R,
    {
        let mut processes = self.processes.borrow_mut();
        processes
            .iter_mut()
            .find(|p| p.pid == pid)
            .map(f)
            .expect("Process not found")
    }

    pub fn processes(&self) -> core::cell::Ref<'_, VecDeque<Process>> {
        self.processes.borrow()
    }

    pub fn processes_mut(&self) -> core::cell::RefMut<'_, VecDeque<Process>> {
        self.processes.borrow_mut()
    }

    pub fn current(&self) -> core::cell::Ref<'_, Process> {
        let pid = self.current_pid.load(core::sync::atomic::Ordering::Relaxed);

        core::cell::Ref::map(self.processes(), |processes| {
            processes
                .iter()
                .find(|p| p.pid == pid)
                .expect("Current process not found")
        })
    }

    pub fn current_mut(&self) -> core::cell::RefMut<'_, Process> {
        let pid = self.current_pid.load(core::sync::atomic::Ordering::Relaxed);

        core::cell::RefMut::map(self.processes_mut(), |processes| {
            processes
                .iter_mut()
                .find(|p| p.pid == pid)
                .expect("Current process not found")
        })
    }

    pub fn add_process(&self, process: Process) {
        self.processes.borrow_mut().push_back(process);
    }
}
unsafe impl Sync for Processes {}

pub static PROCESSES: Processes = Processes {
    processes: RefCell::new(VecDeque::new()),
    current_pid: AtomicU32::new(0),
};

impl Process {
    pub fn user_from_elf(
        cs: x86_64::structures::gdt::SegmentSelector,
        ds: x86_64::structures::gdt::SegmentSelector,
        flags: u64,
        elf: &[u8],
    ) -> Self {
        let pid = PID_FACTORY.next_pid();

        log::debug!("loading process with pid {pid}");

        let mut kernel_paging = crate::kernel_paging();

        let mut userspace_paging = kernel_paging.make_userspace_paging();

        let (userspace_address, userspace_stack, mapped_pages) =
            crate::elf::load(&mut userspace_paging, &mut kernel_paging, elf);

        log::debug!("elf for pid {pid} loaded at address {userspace_address:#x}, stack at {userspace_stack:#x}");

        Process {
            pid: PID_FACTORY.next_pid(),
            state: State::Ready,
            paging: userspace_paging,
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

    pub fn execve(&mut self, elf: &[u8]) {
        log::debug!("execve for pid {}", self.pid);

        let mut kernel_paging = crate::kernel_paging();

        for page in &self.mapped_pages {
            let (frame, flush) = self
                .paging
                .page_table
                .unmap(page.page)
                .expect("Failed to unmap page");
            flush.flush();
            unsafe {
                kernel_paging.frame_allocator.deallocate_frame(frame);
            }
        }

        let (userspace_address, userspace_stack, mapped_pages) =
            crate::elf::load(&mut self.paging, &mut kernel_paging, elf);

        log::debug!(
            "elf for pid {} loaded at address {:#x}, stack at {:#x}",
            self.pid,
            userspace_address.as_u64(),
            userspace_stack.as_u64()
        );

        self.rip = userspace_address.as_u64();
        self.registers = crate::idt::GPRegisters {
            rsp: userspace_stack.as_u64(),
            ..Default::default()
        };
        self.mapped_pages = mapped_pages;
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    fn update_state(pid: u32) {
        let mut process = PROCESSES.process_mut(pid);

        match process.state {
            State::Sleeping(target) => {
                let ticks = unsafe { crate::i8253::TIMER0.ticks() };
                if ticks >= target {
                    process.state = State::Ready;
                }
            }
            State::WaitingForStdin => {
                if !process.stdin.is_empty() {
                    drop(process);

                    crate::syscall::handle_syscall(pid);

                    PROCESSES.process_mut(pid).state = State::Ready;
                }
            }
            _ => {}
        }
    }

    pub fn load_paging(&mut self) {
        self.paging.load();
    }

    pub fn fork(&self) -> Process {
        let mut kernel_paging = crate::kernel_paging();

        let forked_paging = self.paging.fork(&mut kernel_paging, &self.mapped_pages);

        Process {
            pid: PID_FACTORY.next_pid(),
            state: self.state,
            paging: forked_paging,
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

    pub fn file_descriptor(&self, fd: FileDescriptor) -> Option<&ProcessFileDescriptor> {
        self.file_descriptors.get(&fd)
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
        let mut kernel_paging = crate::kernel_paging();

        for &MappedPage { page, .. } in &self.mapped_pages {
            let (frame, flush) = self
                .paging
                .page_table
                .unmap(page)
                .expect("Failed to unmap page");

            flush.flush();

            unsafe {
                kernel_paging.frame_allocator.deallocate_frame(frame);
            }
        }
    }
}

pub fn schedule() -> ! {
    loop {
        x86_64::instructions::interrupts::disable();
        log::trace!("scheduling...");

        let mut processes = PROCESSES.processes_mut();

        processes.retain(|p| !matches!(p.state, State::Terminated(_)));

        if processes.is_empty() {
            log::error!("No user processes left, halting forever!. Goodbye!");
            x86_64::instructions::interrupts::disable();
            x86_64::instructions::hlt();
        }

        let len = processes.len();
        drop(processes);

        for i in 0..len {
            let pid = PROCESSES.processes().get(i).expect("Process not found").pid;
            Process::update_state(pid);
        }

        let mut processes = PROCESSES.processes_mut();

        // rotate through processes until we find one that is ready
        for _ in 0..processes.len() {
            let process = processes.front_mut().expect("No processes left!");

            if process.state == State::Ready {
                log::trace!("scheduling {}", process.pid);

                PROCESSES
                    .current_pid
                    .store(process.pid, core::sync::atomic::Ordering::Relaxed);

                let cs = u64::from(process.cs.0);
                let ds = u64::from(process.ds.0);
                let flags = process.flags;
                let rip = process.rip;
                let registers = process.registers;

                x86_64::instructions::interrupts::disable();
                process.load_paging();

                processes.rotate_left(1);
                drop(processes);

                unsafe {
                    crate::do_iret(cs, ds, flags, rip, &raw const registers);
                }
            } else {
                processes.rotate_left(1);
            }
        }

        drop(processes);

        log::trace!("no ready processes found, sleeping...");

        x86_64::instructions::interrupts::enable();
        x86_64::instructions::hlt();
    }
}
