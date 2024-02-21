use core::time::Duration;

use alloc::vec;
use log::{debug, error, info, trace};
use x86_64::{
    structures::{
        idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
        port::PortRead,
    },
    VirtAddr,
};

use crate::{
    driver::{self, i8253::TIMER0},
    process::{self, ProcessState, CURRENT_PROCESS, PROCESSES},
    term::TERM,
    KERNEL_PAGING,
};

pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn load_idt() {
    unsafe {
        IDT.alignment_check.set_handler_fn(alignment_check_handler);
        IDT.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded_handler);
        IDT.breakpoint.set_handler_fn(breakpoint_handler);
        IDT.debug.set_handler_fn(debug_handler);
        IDT.device_not_available
            .set_handler_fn(device_not_available_handler);
        IDT.divide_error.set_handler_fn(divide_error_handler);
        IDT.double_fault.set_handler_fn(double_fault_handler);
        IDT.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        IDT.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        IDT.invalid_tss.set_handler_fn(invalid_tss_handler);
        IDT.machine_check.set_handler_fn(machine_check_handler);
        IDT.non_maskable_interrupt
            .set_handler_fn(non_maskable_interrupt_handler);
        IDT.overflow.set_handler_fn(overflow_handler);

        IDT.page_fault.set_handler_fn(page_fault_handler);

        IDT.security_exception
            .set_handler_fn(security_exception_handler);
        IDT.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        IDT.simd_floating_point
            .set_handler_fn(simd_floating_point_handler);
        IDT.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        IDT.virtualization.set_handler_fn(virtualization_handler);
        IDT.vmm_communication_exception
            .set_handler_fn(vmm_communication_exception_handler);

        IDT[0x20].set_handler_addr(VirtAddr::new(irq0 as usize as u64));
        IDT[0x21].set_handler_addr(VirtAddr::new(irq1 as usize as u64));
        IDT[0x22].set_handler_addr(VirtAddr::new(irq2 as usize as u64));
        IDT[0x23].set_handler_addr(VirtAddr::new(irq3 as usize as u64));
        IDT[0x24].set_handler_addr(VirtAddr::new(irq4 as usize as u64));
        IDT[0x25].set_handler_addr(VirtAddr::new(irq5 as usize as u64));
        IDT[0x26].set_handler_addr(VirtAddr::new(irq6 as usize as u64));
        IDT[0x27].set_handler_addr(VirtAddr::new(irq7 as usize as u64));
        IDT[0x28].set_handler_addr(VirtAddr::new(irq8 as usize as u64));
        IDT[0x29].set_handler_addr(VirtAddr::new(irq9 as usize as u64));
        IDT[0x2a].set_handler_addr(VirtAddr::new(irq10 as usize as u64));
        IDT[0x2b].set_handler_addr(VirtAddr::new(irq11 as usize as u64));
        IDT[0x2c].set_handler_addr(VirtAddr::new(irq12 as usize as u64));
        IDT[0x2d].set_handler_addr(VirtAddr::new(irq13 as usize as u64));
        IDT[0x2e].set_handler_addr(VirtAddr::new(irq14 as usize as u64));
        IDT[0x2f].set_handler_addr(VirtAddr::new(irq15 as usize as u64));

        IDT[0x80]
            .set_handler_addr(VirtAddr::new(syscall_handler_asm_stub as usize as u64))
            .set_privilege_level(x86_64::PrivilegeLevel::Ring3);

        IDT.load();
    }
}

extern "C" {
    fn syscall_handler_asm_stub();
    fn irq0();
    fn irq1();
    fn irq2();
    fn irq3();
    fn irq4();
    fn irq5();
    fn irq6();
    fn irq7();
    fn irq8();
    fn irq9();
    fn irq10();
    fn irq11();
    fn irq12();
    fn irq13();
    fn irq14();
    fn irq15();
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct GPRegisters {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

unsafe fn store_process_state(registers: GPRegisters, stack_frame: InterruptStackFrame) {
    {
        let mut processes = PROCESSES.lock();
        let current_pid = CURRENT_PROCESS.load(core::sync::atomic::Ordering::Relaxed);
        if let Some(current) = processes.iter_mut().find(|p| p.pid == current_pid) {
            current.state = ProcessState {
                gp: GPRegisters {
                    rsp: stack_frame.stack_pointer.as_u64(),
                    ..registers
                },
                rip: stack_frame.instruction_pointer.as_u64(),
                flags: stack_frame.cpu_flags,
                ds: stack_frame.stack_segment,
                cs: stack_frame.code_segment,
            };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn syscall_handler(
    _rdi: u64,
    _rsi: u64,
    _rdx: u64,
    _rcx: u64,
    _r8: u64,
    _r9: u64,
    registers: GPRegisters,
    stack_frame: InterruptStackFrame,
) {
    unsafe {
        (*KERNEL_PAGING).load();
        store_process_state(registers, stack_frame);
    };

    match registers.rax {
        0 => {
            trace!(
                "syscall_handler: print {:#x} {:#x}",
                registers.rbx,
                registers.rcx
            );

            // print
            let mut bytes = vec![0; registers.rcx as usize];

            {
                let mut processes = PROCESSES.lock();
                let pid = CURRENT_PROCESS.load(core::sync::atomic::Ordering::Relaxed);
                let current_process = processes
                    .iter_mut()
                    .find(|p| p.pid == pid)
                    .expect("failed to find current process");
                if let Some(paging) = current_process.paging.as_mut() {
                    paging.load();
                }

                debug!(
                    "Copying from {:#x} to {:#x}",
                    registers.rbx,
                    registers.rbx + registers.rcx
                );

                for i in 0..registers.rcx {
                    bytes[i as usize] = core::ptr::read((registers.rbx + i) as *const u8);
                }

                (*KERNEL_PAGING).load();
            }

            debug!("Printing: {:?}", bytes);

            let str = core::str::from_utf8(&bytes).unwrap();
            unsafe { TERM.print(str) };
        }
        1 => {
            // sleep for rax ms
            trace!("syscall_handler: sleep {:#x}", registers.rbx);

            let mut processes = PROCESSES.lock();
            if let Some(process) = processes
                .iter_mut()
                .find(|p| p.pid == CURRENT_PROCESS.load(core::sync::atomic::Ordering::Relaxed))
            {
                process.sleep = Some(TIMER0.time() + Duration::from_millis(registers.rbx));
            }
        }
        2 => {
            // exit
            trace!("syscall_handler: exit {:#x}", registers.rbx);

            let mut processes = PROCESSES.lock();
            let current_pid = CURRENT_PROCESS.load(core::sync::atomic::Ordering::Relaxed);
            if let Some(index) = processes.iter().position(|p| p.pid == current_pid) {
                processes.remove(index);
            }

            info!("process {} exited with code {}", current_pid, registers.rbx);

            process::schedule();
        }
        n => {
            error!("syscall_handler: unknown syscall: {:#x}", n);
        }
    }

    process::schedule();
}

#[no_mangle]
pub extern "C" fn irq_handler(
    _rdi: u64,
    _rsi: u64,
    _rdx: u64,
    _rcx: u64,
    _r8: u64,
    _r9: u64,
    registers: GPRegisters,
    irq: u8,
    stack_frame: InterruptStackFrame,
) {
    trace!("irq_handler: irq {:?}, stack_frame {:?}", irq, stack_frame);

    unsafe {
        (*KERNEL_PAGING).load();
        store_process_state(registers, stack_frame);
    };

    match irq {
        0 => unsafe {
            driver::i8253::TIMER0.tick();
            trace!("timer tick");
        },
        1 => unsafe {
            let scancode: u8 = PortRead::read_from_port(0x60);
            debug!("scancode: {}", scancode);
        },
        _ => {
            debug!("irq: {}", irq);
        }
    }

    crate::pic::eoi(irq);

    unsafe { process::schedule() };
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, err: u64) {
    panic!(
        "EXCEPTION: ALIGNMENT CHECK {:#?}\n Error code: {}",
        stack_frame, err
    );
}

extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BOUND RANGE EXCEEDED {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BREAKPOINT {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEBUG {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEVICE NOT AVAILABLE {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, err: u64) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT {:#?}\n Error code: {}",
        stack_frame, err
    );
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    err: u64,
) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT {:#x?}\n Error code: {}",
        stack_frame, err
    );
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, err: u64) {
    panic!(
        "EXCEPTION: INVALID TSS {:#?}\n Error code: {}",
        stack_frame, err
    );
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("EXCEPTION: MACHINE CHECK {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NON MASKABLE INTERRUPT {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    err: PageFaultErrorCode,
) {
    let mut processes = unsafe { PROCESSES.lock() };
    let current_pid = CURRENT_PROCESS.load(core::sync::atomic::Ordering::Relaxed);
    if let Some(i) = processes.iter().position(|p| p.pid == current_pid) {
        unsafe {
            (*KERNEL_PAGING).load();
        }

        info!(
            "Page fault in process {}: {:#x?}, aborting",
            current_pid, stack_frame
        );

        processes.remove(i);
        CURRENT_PROCESS.store(0, core::sync::atomic::Ordering::Relaxed);

        drop(processes);

        loop {}

        unsafe { process::schedule() };
    } else {
        panic!(
            "EXCEPTION: KERNEL PAGE FAULT {:#x?}\n Error code: {:?}",
            stack_frame, err
        );
    }
}

extern "x86-interrupt" fn security_exception_handler(stack_frame: InterruptStackFrame, err: u64) {
    panic!(
        "EXCEPTION: SECURITY EXCEPTION {:#?}\n Error code: {}",
        stack_frame, err
    );
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, err: u64) {
    panic!(
        "EXCEPTION: SEGMENT NOT PRESENT {:#?}\n Error code: {}",
        stack_frame, err
    );
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: SIMD FLOATING POINT {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, err: u64) {
    panic!(
        "EXCEPTION: STACK SEGMENT FAULT {:#?}\n Error code: {}",
        stack_frame, err
    );
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: VIRTUALIZATION {:#?}\n", stack_frame);
}

extern "x86-interrupt" fn vmm_communication_exception_handler(
    stack_frame: InterruptStackFrame,
    err: u64,
) {
    panic!(
        "EXCEPTION: VMM COMMUNICATION EXCEPTION {:#?}\n Error code: {}",
        stack_frame, err
    );
}
