use core::sync::atomic::AtomicU32;

use alloc::vec::Vec;

use crate::kernel::paging::SoosPaging;

pub enum State {
    Ready,
    Sleeping(u64),
}

enum Paging {
    Paging(SoosPaging<'static>),
    KernelPaging,
}

impl Paging {
    pub fn load(&mut self) {
        unsafe {
            match self {
                Paging::Paging(paging) => paging.load(),
                Paging::KernelPaging => (*crate::KERNEL_PAGING).load(),
            }
        }
    }
}

pub struct Process {
    pid: u32,
    state: State,
    paging: Paging,
    cs: x86_64::structures::gdt::SegmentSelector,
    ds: x86_64::structures::gdt::SegmentSelector,
    flags: u64,
    rip: u64,
    registers: crate::idt::GPRegisters,
}

pub static PROCESSES: spin::Lazy<spin::Mutex<Vec<Process>>> =
    spin::Lazy::new(|| spin::Mutex::new(Vec::new()));
pub static CURRENT_PROCESS: AtomicU32 = AtomicU32::new(0);

impl Process {
    pub fn user_process(
        pid: u32,
        paging: SoosPaging<'static>,
        cs: x86_64::structures::gdt::SegmentSelector,
        ds: x86_64::structures::gdt::SegmentSelector,
        flags: u64,
        rip: u64,
        rsp: u64,
    ) -> Self {
        Process {
            pid,
            state: State::Ready,
            paging: Paging::Paging(paging),
            cs,
            ds,
            flags,
            rip,
            registers: crate::idt::GPRegisters {
                rsp,
                ..Default::default()
            },
        }
    }

    pub fn kernel_process(
        pid: u32,
        cs: x86_64::structures::gdt::SegmentSelector,
        ds: x86_64::structures::gdt::SegmentSelector,
        flags: u64,
        rip: u64,
        rsp: u64,
    ) -> Self {
        Process {
            pid,
            state: State::Ready,
            paging: Paging::KernelPaging,
            cs,
            ds,
            flags,
            rip,
            registers: crate::idt::GPRegisters {
                rsp,
                ..Default::default()
            },
        }
    }

    fn ready(&self) -> bool {
        match self.state {
            State::Ready => true,
            State::Sleeping(target) => {
                let ticks = unsafe { crate::i8253::TIMER0.ticks() };
                ticks >= target
            }
        }
    }
}

pub fn set_current_process_state(state: State) {
    let pid = CURRENT_PROCESS.load(core::sync::atomic::Ordering::Relaxed);

    let mut processes = PROCESSES.try_lock().expect("Failed to lock processes");
    let process = processes
        .iter_mut()
        .find(|p| p.pid == pid)
        .expect("Process not found");

    process.state = state;
}

pub fn terminate_current_process() {
    let pid = CURRENT_PROCESS.load(core::sync::atomic::Ordering::Relaxed);

    let mut processes = PROCESSES.try_lock().expect("Failed to lock processes");
    if let Some(index) = processes.iter().position(|p| p.pid == pid) {
        processes.remove(index);
    } else {
        panic!("Process not found for termination");
    }

    if processes.is_empty() {
        panic!("No more processes left to run!");
    }

    CURRENT_PROCESS.store(0, core::sync::atomic::Ordering::Relaxed);
}

pub fn store_current_process_registers(
    stack_frame: x86_64::structures::idt::InterruptStackFrame,
    registers: crate::idt::GPRegisters,
) {
    let pid = CURRENT_PROCESS.load(core::sync::atomic::Ordering::Relaxed);

    let mut processes = PROCESSES.try_lock().expect("Failed to lock processes");
    let process = processes
        .iter_mut()
        .find(|p| p.pid == pid)
        .expect("Process not found");

    process.flags = stack_frame.cpu_flags;
    process.rip = stack_frame.instruction_pointer.as_u64();
    process.registers = crate::idt::GPRegisters {
        rsp: stack_frame.stack_pointer.as_u64(),
        ..registers
    };
}

pub fn try_schedule() -> Option<!> {
    match PROCESSES.try_lock() {
        Some(mut processes) => {
            let process = match processes.iter_mut().find(|p| p.ready()) {
                None => panic!("No process ready to run!"),
                Some(process) => process,
            };

            CURRENT_PROCESS.store(process.pid, core::sync::atomic::Ordering::Relaxed);

            process.paging.load();

            let cs = process.cs.0 as u64;
            let ds = process.ds.0 as u64;
            let flags = process.flags;
            let rip = process.rip;
            let registers = process.registers;
            drop(processes);

            unsafe {
                crate::do_iret(cs, ds, flags, rip, &registers);
            };
        }
        None => None,
    }
}
