use core::{cell::RefCell, sync::atomic::AtomicU32};

use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use anyhow::Context;
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
    WaitingForStream(i32),
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
    pub mapped_pages: Vec<MappedPage>,
    file_descriptors: alloc::collections::BTreeMap<i32, Processi32>,
}

#[derive(Debug, Clone)]
pub enum Processi32 {
    Regular {
        path: alloc::string::String,
        offset: usize,
    },
    Stream {
        buffer: VecDeque<u8>,
        max_size: usize,
        stream_type: StreamType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamType {
    Stdin,
    Keyboard,
    Mouse,
}

pub struct Processes {
    processes: RefCell<VecDeque<Process>>,
    current_pid: AtomicU32,
}

impl Processes {
    #[track_caller]
    pub fn process(&self, pid: u32) -> impl core::ops::Deref<Target = Process> + '_ {
        core::cell::Ref::map(
            self.processes
                .try_borrow()
                .with_context(|| core::panic::Location::caller())
                .expect("Failed to borrow processes"),
            |processes| {
                processes
                    .iter()
                    .find(|p| p.pid == pid)
                    .expect("Process not found")
            },
        )
    }

    #[track_caller]
    pub fn process_mut(&self, pid: u32) -> impl core::ops::DerefMut<Target = Process> + '_ {
        core::cell::RefMut::map(
            self.processes
                .try_borrow_mut()
                .with_context(|| core::panic::Location::caller())
                .expect("Failed to borrow processes"),
            |processes| {
                processes
                    .iter_mut()
                    .find(|p| p.pid == pid)
                    .expect("Process not found")
            },
        )
    }

    #[track_caller]
    pub fn with_process<F, R>(&self, pid: u32, f: F) -> R
    where
        F: FnOnce(&Process) -> R,
    {
        let processes = self
            .processes
            .try_borrow()
            .with_context(|| core::panic::Location::caller())
            .expect("Failed to borrow processes");
        processes
            .iter()
            .find(|p| p.pid == pid)
            .map(f)
            .expect("Process not found")
    }

    #[track_caller]
    pub fn with_process_mut<F, R>(&self, pid: u32, f: F) -> R
    where
        F: FnOnce(&mut Process) -> R,
    {
        let mut processes = self
            .processes
            .try_borrow_mut()
            .with_context(|| core::panic::Location::caller())
            .expect("Failed to borrow processes");
        processes
            .iter_mut()
            .find(|p| p.pid == pid)
            .map(f)
            .expect("Process not found")
    }

    #[track_caller]
    pub fn processes(&self) -> core::cell::Ref<'_, VecDeque<Process>> {
        self.processes
            .try_borrow()
            .with_context(|| core::panic::Location::caller())
            .expect("Failed to borrow processes")
    }

    #[track_caller]
    pub fn processes_mut(&self) -> core::cell::RefMut<'_, VecDeque<Process>> {
        self.processes
            .try_borrow_mut()
            .with_context(|| core::panic::Location::caller())
            .expect("Failed to borrow processes")
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

        let mut file_descriptors = alloc::collections::BTreeMap::new();
        file_descriptors.insert(
            0,
            Processi32::Stream {
                buffer: alloc::collections::vec_deque::VecDeque::new(),
                max_size: 1024,
                stream_type: StreamType::Keyboard,
            },
        );

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
            mapped_pages,
            file_descriptors,
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
            State::WaitingForStream(fd) => {
                let fd = process
                    .file_descriptors
                    .get(&fd)
                    .expect("File descriptor not found");

                match fd {
                    Processi32::Stream { buffer, .. } => {
                        if !buffer.is_empty() {
                            drop(process);
                            crate::syscall::handle_syscall(pid);
                            PROCESSES.process_mut(pid).state = State::Ready;
                        }
                    }
                    Processi32::Regular { .. } => {
                        panic!("cannot wait for regular file descriptor")
                    }
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
            mapped_pages: self.mapped_pages.clone(),
            file_descriptors: self.file_descriptors.clone(),
        }
    }

    pub fn new_file_descriptor(&mut self, fd: Processi32) -> i32 {
        let file_descriptor = self.file_descriptors.keys().max().map_or(0, |max| max + 1);
        self.file_descriptors.insert(file_descriptor, fd);
        file_descriptor
    }

    pub fn close_file_descriptor(&mut self, fd: i32) -> Option<Processi32> {
        self.file_descriptors.remove(&fd)
    }

    pub fn file_descriptor(&self, fd: i32) -> Option<&Processi32> {
        self.file_descriptors.get(&fd)
    }

    pub fn file_descriptor_mut(&mut self, fd: i32) -> Option<&mut Processi32> {
        self.file_descriptors.get_mut(&fd)
    }

    pub fn file_descriptors(&self) -> impl Iterator<Item = (&i32, &Processi32)> {
        self.file_descriptors.iter()
    }

    pub fn file_descriptors_mut(&mut self) -> impl Iterator<Item = (&i32, &mut Processi32)> {
        self.file_descriptors.iter_mut()
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

        assert!(!processes.is_empty(), "No processes left to schedule!");

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
