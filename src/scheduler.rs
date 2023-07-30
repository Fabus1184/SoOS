use alloc::vec::Vec;
use log::{debug, info, trace};
use x86_64::structures::idt::InterruptStackFrame;

use crate::{
    process::{Pid, Process, ProcessState, WaitingState},
    spinlock::Spinlock,
};

#[derive(Debug)]
pub struct SoosScheduler<'a> {
    processes: Vec<Process<'a>>,
    current_process_pid: Option<Pid>,
}

impl<'a> SoosScheduler<'a> {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            current_process_pid: None,
        }
    }

    pub unsafe fn schedule(&mut self, process: Process<'a>) {
        info!("Scheduling process {:?}...", process.pid);
        self.processes.push(process);
    }

    pub unsafe fn run_unlock(&mut self, spinlock: &mut Spinlock<*mut Self>) {
        debug!("Running scheduler...");

        if self.processes.is_empty() {
            debug!("No processes to run!");
            return;
        }

        if self.current_process_pid.is_some() {
            let current_process = self
                .processes
                .iter_mut()
                .find(|p| p.pid == self.current_process_pid.unwrap())
                .unwrap();

            if current_process.state == ProcessState::Running {
                current_process.state = ProcessState::Ready;
            }
        }

        self.processes.rotate_left(1);

        while self.processes.first().unwrap().state != ProcessState::Ready {
            trace!(
                "Skipping process {:?} because of state {:?}...",
                self.processes.first().unwrap().pid,
                self.processes.first().unwrap().state
            );
            self.processes.rotate_left(1);
        }

        self.current_process_pid = Some(self.processes.first().unwrap().pid);

        debug!("Running process {:?}...", self.current_process_pid.unwrap());

        unsafe {
            self.processes
                .first_mut()
                .unwrap()
                .run(|| spinlock.unlock())
        };
    }

    pub unsafe fn sleep(&mut self, ms: i64) {
        trace!("Sleeping current process for {}ms...", ms);
        self.processes
            .iter_mut()
            .find(|p| p.pid == self.current_process_pid.expect("No current process!"))
            .expect("Current process not found!")
            .state = ProcessState::Waiting(WaitingState::Timer(ms));
    }

    pub fn timer_tick(&mut self) {
        trace!("Timer tick!");
        self.processes.iter_mut().for_each(|p| match p.state {
            ProcessState::Waiting(WaitingState::Timer(0)) => {
                p.state = ProcessState::Ready;
            }
            ProcessState::Waiting(WaitingState::Timer(ms)) => {
                p.state = if ms < 0 {
                    ProcessState::Ready
                } else {
                    ProcessState::Waiting(WaitingState::Timer(ms - 1))
                };
            }
            _ => {}
        });
    }

    pub fn update_current_process_stack(&mut self, stack_frame: &InterruptStackFrame) {
        if self.current_process_pid.is_some() {
            self.processes
                .iter_mut()
                .find(|p| p.pid == self.current_process_pid.expect("No current process!"))
                .expect("Current process not found!")
                .stack = **stack_frame;
        }
    }
}
