use core::{arch::asm, sync::atomic::AtomicU32};

use alloc::{sync::Arc, vec::Vec};
use log::{debug, trace};
use spin::{Lazy, Mutex};

use crate::{
    driver::i8253::{SystemTime, TIMER0},
    idt::GPRegisters,
    kernel::paging::SoosPaging,
};

pub struct Process {
    pub pid: u32,
    pub descriptors: Vec<Vec<u8>>,
    pub sleep: Option<SystemTime>,
    pub state: ProcessState,
    pub paging: Option<SoosPaging<'static>>,
}

#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct ProcessState {
    pub gp: GPRegisters,
    pub rip: u64,
    pub flags: u64,
    pub ds: u64,
    pub cs: u64,
}

pub static mut PROCESSES: Lazy<Arc<Mutex<Vec<Process>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));
pub static CURRENT_PROCESS: AtomicU32 = AtomicU32::new(0);

pub unsafe fn schedule() -> ! {
    let mut run = None;

    let mut processes = PROCESSES.lock();
    for process in processes.iter_mut() {
        if process.ready() {
            run = Some((process.run(), process.pid));
            break;
        }
    }
    drop(processes);

    match run {
        Some((run, pid)) => {
            debug!("running process {pid}");
            run()
        }
        None => {
            debug!("no process to run, hlt");
            Process::new(
                0,
                None,
                hlt as usize as u64,
                HLT_STACK.as_ptr() as u64 + 4096,
                0x10,
                0x8,
                0x202,
            )
            .run()()
        }
    }
}

static mut HLT_STACK: [u8; 4096] = [0; 4096];

unsafe fn hlt() -> ! {
    loop {
        debug!("hlt");
        asm!("hlt");
    }
}

extern "C" {
    fn run_process(cs: u64, ds: u64, flags: u64, rip: u64, regs: *const GPRegisters) -> !;
}

impl Process {
    pub fn new(
        pid: u32,
        paging: Option<SoosPaging<'static>>,
        rip: u64,
        rsp: u64,
        ds: u64,
        cs: u64,
        flags: u64,
    ) -> Self {
        Self {
            pid,
            paging,
            descriptors: Vec::new(),
            sleep: None,
            state: ProcessState {
                gp: GPRegisters {
                    rsp,
                    ..Default::default()
                },
                rip,
                flags,
                ds,
                cs,
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
    pub unsafe fn run(&mut self) -> impl FnOnce() -> ! {
        let pid = self.pid;
        let state = self.state.clone();
        let load_paging = self.paging.as_mut().map(|p| p.load_fn());

        move || {
            asm!("cli");

            CURRENT_PROCESS.store(pid, core::sync::atomic::Ordering::Relaxed);

            if let Some(load) = load_paging {
                load();
            }

            run_process(
                state.cs,
                state.ds,
                state.flags,
                state.rip,
                &state.gp as *const GPRegisters,
            )
        }
    }
}
