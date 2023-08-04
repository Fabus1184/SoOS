use core::arch::asm;

use log::{info, trace, warn};
use x86_64::{
    set_general_handler,
    structures::{
        idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
        port::PortRead,
    },
    VirtAddr,
};

use crate::{
    driver, syscall::Syscall, KERNEL_CODE_SEGMENT, KERNEL_PAGING, KERNEL_STACK,
    KERNEL_STACK_SEGMENT, SCHEDULER,
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

        set_general_handler!(&mut IDT, irq_handler, 32..=47);

        IDT[0x80]
            .set_handler_addr(VirtAddr::new(syscall_handler_asm_stub as u64))
            .set_privilege_level(x86_64::PrivilegeLevel::Ring3);

        IDT.load();
    }
}

extern "x86-interrupt" {
    fn syscall_handler_asm_stub();
}

/*
extern "C" fn _run_scheduler() {
    unsafe { SCHEDULER.run().expect("failed to run scheduler") };
}

fn run_scheduler() -> Option<!> {
    trace!("run_scheduler: loading kernel paging");
    unsafe {
        KERNEL_PAGING
            .as_mut()
            .expect("kernel paging not set")
            .load()
    };

    if unsafe { !SCHEDULER.running() } {
        unsafe {
            asm!(
                "push {kds:r}",
                "push {stack:r}",
                "push {rflags:r}",
                "push {kcs:r}",
                "push {func:r}",
                "mov ax, {kds:x}",
                "mov ds, ax",
                "mov es, ax",
                "mov fs, ax",
                "mov gs, ax",
                "iretq",
                kds = in(reg) (KERNEL_STACK_SEGMENT.expect("kernel stack segment not set").index() * 8) | 0,
                kcs = in(reg) (KERNEL_CODE_SEGMENT.expect("kernel code segment not set").index() * 8) | 0,
                // out("ax") _,
                stack = in(reg) KERNEL_STACK.as_mut_ptr() as u64 + KERNEL_STACK.len() as u64,
                rflags = in(reg) 0x202,
                func = in(reg) _run_scheduler as u64,
                options(noreturn),
            )
        };
    } else {
        None
    }
}*/

#[no_mangle]
pub extern "C" fn syscall_handler(rax: u64, rbx: u64, stack_frame: InterruptStackFrame) {
    trace!(
        "syscall_handler: rax {:?}, rbx {:?}, stack_frame {:?}",
        rax,
        rbx,
        stack_frame
    );

    unsafe {
        SCHEDULER
            .try_lock()
            .map(|s| {
                s.update_current_process_stack_frame(&stack_frame);

                Syscall::from_regs(rax, rbx)
                    .expect("failed to read syscall from registers")
                    .execute(s);

                s.run(|| SCHEDULER.unlock());
            })
            .expect("syscall_handler: failed to lock scheduler");
    };
}

fn irq_handler(stack_frame: InterruptStackFrame, irq: u8, _error_code: Option<u64>) {
    trace!("irq_handler: irq {:?}, stack_frame {:?}", irq, stack_frame);

    let irq = irq - 32;

    match irq {
        0 => unsafe {
            driver::i8253::TIMER0.tick();

            SCHEDULER
                .with_locked(|s| {
                    s.update_current_process_stack_frame(&stack_frame);
                    s.run(|| {
                        crate::pic::eoi(irq);
                        SCHEDULER.unlock();
                    });
                })
                .unwrap_or_else(|| warn!("irq_handler: failed to lock scheduler"));
        },
        1 => unsafe {
            let scancode: u8 = PortRead::read_from_port(0x60);
            info!("scancode: {}", scancode);
            crate::pic::eoi(irq);
        },
        _ => {
            info!("irq: {}", irq);
            crate::pic::eoi(irq);
        }
    }

    trace!("irq_handler: end");
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
        "EXCEPTION: GENERAL PROTECTION FAULT {:#?}\n Error code: {}",
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
    panic!(
        "EXCEPTION: PAGE FAULT {:#?}\n Error code: {:?}",
        stack_frame, err
    );
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
