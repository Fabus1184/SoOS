use core::{arch::asm, sync::atomic::AtomicU32};

use alloc::{sync::Arc, vec::Vec};
use log::trace;
use spin::{Lazy, Mutex};

use crate::driver::i8253::{SystemTime, TIMER0};

pub struct Process {
    pub pid: u32,
    pub descriptors: Vec<Vec<u8>>,
    pub sleep: Option<SystemTime>,
    pub state: ProcessState,
}

#[derive(Debug, Default, Clone)]
pub struct ProcessState {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub rip: u64,
    pub flags: u64,
    pub ss: u64,
    pub cs: u64,
}

pub static mut PROCESSES: Lazy<Arc<Mutex<Vec<Process>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));
pub static mut CURRENT_PROCESS: AtomicU32 = AtomicU32::new(0);

pub unsafe fn schedule() -> ! {
    let mut run = None;

    let mut processes = PROCESSES.lock();
    for process in processes.iter_mut() {
        if process.ready() {
            run = Some(process.run());
            break;
        }
    }
    drop(processes);

    match run {
        Some(r) => {
            trace!("running process");
            r()
        }
        None => {
            trace!("no process to run, hlt");
            Process::new(
                0,
                hlt as usize as u64,
                HLT_STACK.as_ptr() as u64 + 4096,
                0x10,
                0x08,
                0x202,
            )
            .run()()
        }
    }
}

static mut HLT_STACK: [u8; 4096] = [0; 4096];

unsafe fn hlt() {
    loop {
        asm!("hlt");
    }
}

impl Process {
    pub fn new(pid: u32, rip: u64, rsp: u64, ss: u64, cs: u64, flags: u64) -> Self {
        Self {
            pid,
            descriptors: Vec::new(),
            sleep: None,
            state: ProcessState {
                rsp,
                rip,
                flags,
                ss,
                cs,
                ..Default::default()
            },
        }
    }

    pub fn ready(&mut self) -> bool {
        trace!("checking if process {} is ready", self.pid);
        match self.sleep {
            Some(end) => unsafe { TIMER0.time() >= end },
            None => true,
        }
    }

    /// Returns a closure that will run the process
    /// Make sure to only call this while the process still exists
    pub unsafe fn run(&self) -> impl FnOnce() -> ! {
        let pid = self.pid;
        let state = self.state.clone();
        move || {
            CURRENT_PROCESS.store(pid, core::sync::atomic::Ordering::Relaxed);
            asm!(
                "cli",
                "push {uds:r}",
                "push {stack_pointer:r}",
                "push {flags:r}",
                "push {ucs:r}",
                "push {instruction_pointer:r}",
                "mov ax, {uds:x}",
                "mov ds, ax",
                "mov es, ax",
                "mov fs, ax",
                "mov gs, ax",
                "iretq",
                uds = in(reg) state.ss,
                ucs = in(reg) state.cs,
                flags = in(reg) state.flags,
                stack_pointer = in(reg) state.rsp,
                instruction_pointer = in(reg) state.rip,
                options(noreturn)
            )
        }
    }
}
