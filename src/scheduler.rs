use alloc::vec::Vec;
use log::{debug, info, trace, warn};
use x86_64::structures::idt::InterruptStackFrame;

use crate::{
    driver::i8253::TIMER0,
    process::{Process, ProcessState, WaitingState},
};

#[derive(Debug)]
pub struct SoosScheduler<'a> {
    processes: Vec<Process<'a>>,
    pub current_process: Option<*mut Process<'a>>,
}

impl<'a> SoosScheduler<'a> {
    pub const fn new() -> Self {
        Self {
            processes: Vec::new(),
            current_process: None,
        }
    }

    pub unsafe fn schedule(&mut self, process: Process<'a>) {
        info!("Scheduling process {:?}...", process.pid);
        self.processes.push(process);
    }

    pub unsafe fn run(&mut self) -> ! {
        debug!("Running scheduler...");

        if self.processes.is_empty() {
            panic!("No processes to run!");
        }

        self.current_process
            .as_mut()
            .map(|p| {
                if (&mut **p).state == ProcessState::Running {
                    (&mut **p).state = ProcessState::Ready;
                }
            })
            .unwrap_or_else(|| warn!("No current process!"));

        while match self.processes.first().unwrap().state {
            ProcessState::Running => unreachable!(),
            ProcessState::Waiting(WaitingState::Timer(ts)) => {
                if TIMER0.ticks() > ts {
                    true
                } else {
                    false
                }
            }
            ProcessState::Ready => true,
            _ => false,
        } {
            self.processes.rotate_left(1);
        }

        self.current_process = Some(self.processes.first_mut().unwrap() as *mut _);

        debug!(
            "Running process {:?}...",
            (&mut *self.current_process.unwrap()).pid
        );
        self.current_process.unwrap().as_mut().unwrap().run();
    }

    pub unsafe fn sleep(&mut self, ms: u64) {
        trace!("Sleeping current process for {}ms...", ms);
        self.current_process
            .as_mut()
            .map(|p| {
                (&mut **p).state = ProcessState::Waiting(WaitingState::Timer(TIMER0.ticks() + ms));
            })
            .unwrap_or_else(|| warn!("No current process!"));
    }

    pub fn update_current_process_stack_frame(&mut self, stack_frame: &InterruptStackFrame) {
        debug!(
            "Updating stack frame for current process to {:?}",
            stack_frame
        );
        self.current_process
            .as_mut()
            .map(|p| {
                (unsafe { &mut **p }).stack = **stack_frame;
            })
            .unwrap_or_else(|| warn!("No current process!"));
    }
}
