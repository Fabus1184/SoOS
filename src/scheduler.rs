use alloc::vec::Vec;
use log::trace;
use x86_64::structures::idt::InterruptStackFrame;

use crate::process::{Pid, Process, ProcessState, WaitingState};

#[derive(Debug)]
pub struct SoosScheduler<'a> {
    processes: Vec<Process<'a>>,
    current_process: Option<Pid>,
}

impl<'a> SoosScheduler<'a> {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            current_process: None,
        }
    }

    pub unsafe fn schedule(&mut self, process: Process<'a>) {
        trace!("Scheduling process {:?}...", process.pid);
        self.processes.push(process);
    }

    pub unsafe fn run(&mut self) {
        if self.processes.is_empty() {
            trace!("No processes to run!");
            return;
        }

        while ((&self.processes.first().expect("No processes to run!").state)
            as *const ProcessState)
            .read_volatile()
            != ProcessState::Ready
        {
            trace!(
                "Skipping process {:?} because of state {:?}...",
                self.processes.first().expect("No processes to run!").pid,
                self.processes.first().expect("No processes to run!").state
            );
            self.processes.rotate_left(1);
        }

        self.current_process = Some(self.processes.first().expect("No processes to run!").pid);

        trace!(
            "Running process {:?}...",
            self.current_process.expect("No current process!")
        );

        unsafe {
            self.processes
                .first_mut()
                .expect("No processes to run!")
                .run()
        }
    }

    pub unsafe fn sleep(&mut self, ms: i64) {
        trace!("Sleeping current process for {}ms...", ms);
        ((&mut self
            .processes
            .iter_mut()
            .find(|p| p.pid == self.current_process.expect("No current process!"))
            .expect("Current process not found!")
            .state) as *mut ProcessState)
            .write_volatile(ProcessState::Waiting(WaitingState::Timer(ms)));
    }

    pub fn timer_tick(&mut self) {
        trace!("Timer tick!");
        self.processes.iter_mut().for_each(|p| {
            match unsafe { (&p.state as *const ProcessState).read_volatile() } {
                ProcessState::Waiting(WaitingState::Timer(0)) => {
                    unsafe {
                        (&mut p.state as *mut ProcessState).write_volatile(ProcessState::Ready)
                    };
                }
                ProcessState::Waiting(WaitingState::Timer(ms)) => {
                    unsafe {
                        (&mut p.state as *mut ProcessState).write_volatile(if ms < 0 {
                            ProcessState::Ready
                        } else {
                            ProcessState::Waiting(WaitingState::Timer(ms - 1))
                        })
                    };
                }
                _ => {}
            }
        });
    }

    pub fn update_current_process_stack(&mut self, stack_frame: &InterruptStackFrame) {
        if self.current_process.is_some() {
            self.processes
                .iter_mut()
                .find(|p| p.pid == self.current_process.expect("No current process!"))
                .expect("Current process not found!")
                .stack = **stack_frame;
        }
    }
}
