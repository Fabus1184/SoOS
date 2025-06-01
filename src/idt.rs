use log::{debug, trace, warn};
use pc_keyboard::{KeyboardLayout, ScancodeSet};
use x86_64::{
    structures::{
        idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
        port::PortRead,
    },
    VirtAddr,
};

use crate::{
    driver::{self},
    process,
    syscall::handle_syscall,
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

extern "x86-interrupt" fn other_handler(stack_frame: InterruptStackFrame) {
    panic!("Unhandled interrupt: {:#?}", stack_frame);
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
    let mut process_lock =
        crate::process::current_process_mut().expect("failed to get current process");
    let process = process_lock.get();

    process.flags = stack_frame.cpu_flags.bits();
    process.rip = stack_frame.instruction_pointer.as_u64();
    process.registers = crate::idt::GPRegisters {
        rsp: stack_frame.stack_pointer.as_u64(),
        ..registers
    };

    handle_syscall(&mut process_lock);

    drop(process_lock);

    crate::process::try_schedule().expect("Failed to schedule a process");
}

#[no_mangle]
extern "C" fn irq_handler(
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
    trace!(
        "irq_handler begin: irq {:0x?}, stack_frame {:0x?}, registers {:0x?}",
        irq,
        stack_frame,
        registers
    );

    match irq {
        0 => unsafe {
            driver::i8253::TIMER0.tick();
            trace!("timer tick");
        },
        1 => unsafe {
            let scancode: u8 = PortRead::read_from_port(0x60);
            trace!("scancode: {}", scancode);

            static mut KEYBOARD: pc_keyboard::ScancodeSet1 = pc_keyboard::ScancodeSet1::new();
            match KEYBOARD.advance_state(scancode) {
                Ok(Some(key_event)) => {
                    static mut MODIFIERS: pc_keyboard::Modifiers = pc_keyboard::Modifiers {
                        lshift: false,
                        rshift: false,
                        lctrl: false,
                        rctrl: false,
                        numlock: false,
                        capslock: false,
                        lalt: false,
                        ralt: false,
                        rctrl2: false,
                    };

                    let key = pc_keyboard::layouts::De105Key {}.map_keycode(
                        key_event.code,
                        &MODIFIERS,
                        pc_keyboard::HandleControl::Ignore,
                    );

                    match key {
                        pc_keyboard::DecodedKey::RawKey(key) => match key {
                            pc_keyboard::KeyCode::LShift => {
                                MODIFIERS.lshift = key_event.state == pc_keyboard::KeyState::Down;
                            }
                            pc_keyboard::KeyCode::LAlt => {
                                MODIFIERS.lalt = key_event.state == pc_keyboard::KeyState::Down;
                            }
                            pc_keyboard::KeyCode::LControl => {
                                MODIFIERS.lctrl = key_event.state == pc_keyboard::KeyState::Down;
                            }
                            _ => {
                                log::debug!("unhandled raw key: {:?}", key);
                            }
                        },
                        pc_keyboard::DecodedKey::Unicode(char) => {
                            if MODIFIERS.lctrl {
                                if let Some(digit) = char.to_digit(10) {
                                    // switch to tty
                                }
                            } else if key_event.state == pc_keyboard::KeyState::Down {
                                if char.is_ascii() {
                                    for process in crate::process::PROCESSES
                                        .try_lock()
                                        .expect("Failed to lock processes")
                                        .iter_mut()
                                    {
                                        process.stdin.push_back(char as u8);
                                    }
                                } else {
                                    warn!(
                                        "Non-ASCII character received from keyboard: '{}'",
                                        char.escape_unicode()
                                    );
                                }
                            }
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("Failed to advance keyboard state: {:?}", e);
                }
            }
        },
        _ => {
            debug!("irq: {}", irq);
        }
    }

    crate::pic::eoi(irq);

    trace!("irq_handler end");

    unsafe {
        crate::do_iret(
            stack_frame.code_segment.0 as u64,
            stack_frame.stack_segment.0 as u64,
            stack_frame.cpu_flags.bits(),
            stack_frame.instruction_pointer.as_u64(),
            &GPRegisters {
                rsp: stack_frame.stack_pointer.as_u64(),
                ..registers
            },
        )
    };
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
    let address = x86_64::registers::control::Cr2::read().expect("Failed to read CR2 register");

    if err.contains(PageFaultErrorCode::USER_MODE) {
        let mut process_lock =
            crate::process::current_process_mut().expect("Failed to get current process");

        let process = process_lock.get();
        process.state = crate::process::State::Terminated(1);
        log::warn!(
            "Page fault in process {} ({:?}): {:#0x?}, caused by address {:#0x}",
            process.pid(),
            err,
            stack_frame,
            address
        );

        drop(process_lock);

        process::try_schedule().expect("Failed to schedule a process after page fault");
    } else {
        panic!(
            "Page fault in kernel mode: {:#0x?}\nError code: {:?}\nAddress: {:#0x}",
            stack_frame, err, address
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
