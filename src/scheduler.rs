use core::sync::atomic::{AtomicBool, Ordering};

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
    lock: AtomicBool,
    current_process: Option<*mut Process<'a>>,
}

impl<'a> SoosScheduler<'a> {
    pub const fn new() -> Self {
        Self {
            processes: Vec::new(),
            lock: AtomicBool::new(false),
            current_process: None,
        }
    }

    pub fn running(&self) -> bool {
        self.lock.load(Ordering::SeqCst)
    }

    pub unsafe fn schedule(&mut self, process: Process<'a>) {
        self.lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .expect("Scheduler already running!");

        info!("Scheduling process {:?}...", process.pid);
        self.processes.push(process);

        self.lock
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .expect("Scheduler not running!");
    }

    pub unsafe fn run(&mut self) -> Option<!> {
        self.lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .ok()?;
        {
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
        }
        self.lock
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .expect("Scheduler not running!");

        loop {
            if self
                .lock
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                self.processes.rotate_left(1);

                if match self.processes.first().unwrap().state {
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
                    break;
                } else {
                    debug!(
                        "Skipping process {:?} because of state {:?}...",
                        self.processes.first().unwrap().pid,
                        self.processes.first().unwrap().state
                    );

                    self.lock
                        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                        .expect("Scheduler not running!");
                }
            }
        }

        self.current_process = Some(self.processes.first_mut().unwrap() as *mut _);

        debug!("Running process {:?}...", self.current_process.unwrap());

        unsafe {
            self.processes.first_mut().unwrap().run(|| {
                self.lock
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .expect("Scheduler not running!");
            });
        };
    }

    pub unsafe fn sleep(&mut self, ms: u64) {
        self.lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .expect("Scheduler already running!");

        trace!("Sleeping current process for {}ms...", ms);
        self.current_process
            .as_mut()
            .map(|p| {
                (&mut **p).state = ProcessState::Waiting(WaitingState::Timer(TIMER0.ticks() + ms));
            })
            .unwrap_or_else(|| warn!("No current process!"));

        self.lock
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .expect("Scheduler not running!");
    }

    pub fn update_current_process_stack(&mut self, stack_frame: &InterruptStackFrame) {
        self.lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .expect("Scheduler already running!");

        self.current_process
            .as_mut()
            .map(|p| {
                (unsafe { &mut **p }).stack = **stack_frame;
            })
            .unwrap_or_else(|| warn!("No current process!"));

        self.lock
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .expect("Scheduler not running!");
    }
}
