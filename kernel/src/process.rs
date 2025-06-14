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
            next_pid: AtomicU32::new(1),
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
    WaitingForChild {
        pid: u32,
        arg: *mut crate::syscall::generated::syscall_waitpid_t,
    },
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
    pub cs: x86_64::structures::gdt::SegmentSelector,
    pub ds: x86_64::structures::gdt::SegmentSelector,
    pub flags: u64,
    pub rip: u64,
    pub registers: crate::idt::GPRegisters,
    pub xsave: xsave::XSave,
    pub mapped_pages: Vec<MappedPage>,
    file_descriptors: alloc::collections::BTreeMap<i32, FileDescriptor>,
}

#[derive(Debug, Clone)]
pub enum FileDescriptor {
    Regular {
        path: alloc::string::String,
        offset: usize,
    },
    ForeignStream {
        stream_type: ForeignStreamType,
    },
    OwnedStream {
        buffer: VecDeque<u8>,
        max_size: usize,
        stream_type: OwnedStreamType,
    },
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnedStreamType {
    Stdin,
    Stdout,
    Keyboard,
    Mouse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForeignStreamType {
    Process { pid: u32, file_descriptor: i32 },
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
                    .unwrap_or_else(|| panic!("process {pid} not found"))
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
                    .unwrap_or_else(|| panic!("process {pid} not found"))
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
            .map_or_else(|| panic!("process {pid} not found"), f)
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
            .map_or_else(|| panic!("process {pid} not found"), f)
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

    #[track_caller]
    pub fn current(&self) -> Option<core::cell::Ref<'_, Process>> {
        let pid = self.current_pid.load(core::sync::atomic::Ordering::Relaxed);

        core::cell::Ref::filter_map(self.processes(), |processes| {
            processes.iter().find(|p| p.pid == pid)
        })
        .ok()
    }

    #[track_caller]
    pub fn current_mut(&self) -> Option<core::cell::RefMut<'_, Process>> {
        let pid = self.current_pid.load(core::sync::atomic::Ordering::Relaxed);

        core::cell::RefMut::filter_map(self.processes_mut(), |processes| {
            processes.iter_mut().find(|p| p.pid == pid)
        })
        .ok()
    }

    pub fn add_process(&self, process: Process) {
        self.processes.borrow_mut().push_back(process);
    }
}
unsafe impl Sync for Processes {}

pub fn store_state(
    registers: crate::idt::GPRegisters,
    stack_frame: &x86_64::structures::idt::InterruptStackFrame,
) -> Option<u32> {
    let mut process = PROCESSES.current_mut()?;

    process.flags = stack_frame.cpu_flags.bits();
    process.rip = stack_frame.instruction_pointer.as_u64();
    process.registers = crate::idt::GPRegisters {
        rsp: stack_frame.stack_pointer.as_u64(),
        ..registers
    };
    process.xsave.save();

    let pid = PROCESSES
        .current_pid
        .load(core::sync::atomic::Ordering::Relaxed);
    PROCESSES
        .current_pid
        .store(0, core::sync::atomic::Ordering::Relaxed);

    Some(pid)
}

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
            crate::elf::load::<&str>(&mut userspace_paging, &mut kernel_paging, elf, &[]);

        log::debug!("elf for pid {pid} loaded at address {userspace_address:#x}, stack at {userspace_stack:#x}");

        let mut file_descriptors = alloc::collections::BTreeMap::new();
        file_descriptors.insert(
            0,
            FileDescriptor::OwnedStream {
                buffer: alloc::collections::vec_deque::VecDeque::new(),
                max_size: 1024,
                stream_type: OwnedStreamType::Stdin,
            },
        );
        file_descriptors.insert(
            1,
            FileDescriptor::OwnedStream {
                buffer: alloc::collections::vec_deque::VecDeque::new(),
                max_size: 1024,
                stream_type: OwnedStreamType::Stdout,
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
            xsave: xsave::XSave::default(),
            mapped_pages,
            file_descriptors,
        }
    }

    pub fn execve<T: AsRef<str>>(&mut self, elf: &[u8], args: &[T]) {
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
                kernel_paging.deallocate_frame(frame);
            }
        }

        let (userspace_address, userspace_stack, mapped_pages) =
            crate::elf::load(&mut self.paging, &mut kernel_paging, elf, args);

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
                let file_descriptor = process
                    .file_descriptors
                    .get(&fd)
                    .expect("File descriptor not found");

                match file_descriptor {
                    FileDescriptor::OwnedStream { buffer, .. } => {
                        if !buffer.is_empty() {
                            drop(process);
                            crate::syscall::handle_syscall(pid);
                            PROCESSES.process_mut(pid).state = State::Ready;
                        }
                    }
                    FileDescriptor::Regular { .. } => {
                        panic!("cannot wait for regular file descriptor")
                    }
                    FileDescriptor::ForeignStream { stream_type } => match stream_type {
                        &ForeignStreamType::Process {
                            pid: other_pid,
                            file_descriptor,
                        } => {
                            drop(process);
                            let other_process = PROCESSES.process(other_pid);
                            let fd = other_process
                                .file_descriptor(file_descriptor)
                                .expect("File descriptor not found");
                            match fd {
                                FileDescriptor::OwnedStream { buffer, .. } => {
                                    if !buffer.is_empty() {
                                        drop(other_process);
                                        crate::syscall::handle_syscall(pid);
                                        PROCESSES.process_mut(pid).state = State::Ready;
                                    }
                                }
                                e => panic!("cannot wait for foreign file descriptor: {:?}", e,),
                            }
                        }
                    },
                    FileDescriptor::Terminal => todo!(),
                }
            }
            _ => {}
        }
    }

    pub fn load_paging(&self) {
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
            xsave: self.xsave,
            mapped_pages: self.mapped_pages.clone(),
            file_descriptors: self.file_descriptors.clone(),
        }
    }

    pub fn new_file_descriptor(&mut self, fd: FileDescriptor) -> i32 {
        let file_descriptor = self.file_descriptors.keys().max().map_or(0, |max| max + 1);
        self.file_descriptors.insert(file_descriptor, fd);
        file_descriptor
    }

    pub fn close_file_descriptor(&mut self, fd: i32) -> Option<FileDescriptor> {
        self.file_descriptors.remove(&fd)
    }

    pub fn file_descriptor(&self, fd: i32) -> Option<&FileDescriptor> {
        self.file_descriptors.get(&fd)
    }

    pub fn file_descriptor_mut(&mut self, fd: i32) -> Option<&mut FileDescriptor> {
        self.file_descriptors.get_mut(&fd)
    }

    pub fn file_descriptors(&self) -> impl Iterator<Item = (&i32, &FileDescriptor)> {
        self.file_descriptors.iter()
    }

    pub fn file_descriptors_mut(&mut self) -> impl Iterator<Item = (&i32, &mut FileDescriptor)> {
        self.file_descriptors.iter_mut()
    }

    pub fn redirect_stdout_to_term(&mut self) {
        *self
            .file_descriptor_mut(1)
            .expect("File descriptor 1 not found") = FileDescriptor::Terminal;
    }

    pub fn redirect_keyboard_to_stdin(&mut self) {
        *self
            .file_descriptor_mut(0)
            .expect("File descriptor 0 not found") = FileDescriptor::OwnedStream {
            buffer: alloc::collections::vec_deque::VecDeque::new(),
            max_size: 1024,
            stream_type: OwnedStreamType::Keyboard,
        };
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
                kernel_paging.deallocate_frame(frame);
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
            log::warn!("no processes left to schedule, halting...");
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
                let pid = process.pid;

                processes.rotate_left(1);
                drop(processes);

                iret(pid)
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

extern "C" {
    pub fn do_iret(
        cs: u64,
        ds: u64,
        flags: u64,
        rip: u64,
        regs: *const crate::idt::GPRegisters,
    ) -> !;
}

pub fn iret(pid: u32) -> ! {
    x86_64::instructions::interrupts::disable();

    PROCESSES
        .current_pid
        .store(pid, core::sync::atomic::Ordering::Relaxed);

    let process = PROCESSES.process(pid);

    let flags = process.flags;
    let rip = process.rip;
    let registers = process.registers;
    let cs = process.cs;
    let ds = process.ds;

    process.xsave.load();

    process.load_paging();

    drop(process);

    unsafe {
        do_iret(
            u64::from(cs.0),
            u64::from(ds.0),
            flags,
            rip,
            &raw const registers,
        );
    }
}
