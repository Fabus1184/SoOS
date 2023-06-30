use x86_64::structures::idt::InterruptStackFrame;

use crate::printk;

pub static ISRS: [extern "x86-interrupt" fn(InterruptStackFrame); 32] = [
    isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7, isr8, isr9, isr10, isr11, isr12, isr13, isr14,
    isr15, isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23, isr24, isr25, isr26, isr27,
    isr28, isr29, isr30, isr31,
];

fn isr(n: u8, stack_frame: InterruptStackFrame) {
    printk!("isr{}: {:?}", n, stack_frame);
}

extern "x86-interrupt" fn isr0(stack_frame: InterruptStackFrame) {
    isr(0, stack_frame);
}

extern "x86-interrupt" fn isr1(stack_frame: InterruptStackFrame) {
    isr(1, stack_frame);
}

extern "x86-interrupt" fn isr2(stack_frame: InterruptStackFrame) {
    isr(2, stack_frame);
}

extern "x86-interrupt" fn isr3(stack_frame: InterruptStackFrame) {
    isr(3, stack_frame);
}

extern "x86-interrupt" fn isr4(stack_frame: InterruptStackFrame) {
    isr(4, stack_frame);
}

extern "x86-interrupt" fn isr5(stack_frame: InterruptStackFrame) {
    isr(5, stack_frame);
}

extern "x86-interrupt" fn isr6(stack_frame: InterruptStackFrame) {
    isr(6, stack_frame);
}

extern "x86-interrupt" fn isr7(stack_frame: InterruptStackFrame) {
    isr(7, stack_frame);
}

extern "x86-interrupt" fn isr8(stack_frame: InterruptStackFrame) {
    isr(8, stack_frame);
}

extern "x86-interrupt" fn isr9(stack_frame: InterruptStackFrame) {
    isr(9, stack_frame);
}

extern "x86-interrupt" fn isr10(stack_frame: InterruptStackFrame) {
    isr(10, stack_frame);
}

extern "x86-interrupt" fn isr11(stack_frame: InterruptStackFrame) {
    isr(11, stack_frame);
}

extern "x86-interrupt" fn isr12(stack_frame: InterruptStackFrame) {
    isr(12, stack_frame);
}

extern "x86-interrupt" fn isr13(stack_frame: InterruptStackFrame) {
    isr(13, stack_frame);
}

extern "x86-interrupt" fn isr14(stack_frame: InterruptStackFrame) {
    isr(14, stack_frame);
}

extern "x86-interrupt" fn isr15(stack_frame: InterruptStackFrame) {
    isr(15, stack_frame);
}

extern "x86-interrupt" fn isr16(stack_frame: InterruptStackFrame) {
    isr(16, stack_frame);
}

extern "x86-interrupt" fn isr17(stack_frame: InterruptStackFrame) {
    isr(17, stack_frame);
}

extern "x86-interrupt" fn isr18(stack_frame: InterruptStackFrame) {
    isr(18, stack_frame);
}

extern "x86-interrupt" fn isr19(stack_frame: InterruptStackFrame) {
    isr(19, stack_frame);
}

extern "x86-interrupt" fn isr20(stack_frame: InterruptStackFrame) {
    isr(20, stack_frame);
}

extern "x86-interrupt" fn isr21(stack_frame: InterruptStackFrame) {
    isr(21, stack_frame);
}

extern "x86-interrupt" fn isr22(stack_frame: InterruptStackFrame) {
    isr(22, stack_frame);
}

extern "x86-interrupt" fn isr23(stack_frame: InterruptStackFrame) {
    isr(23, stack_frame);
}

extern "x86-interrupt" fn isr24(stack_frame: InterruptStackFrame) {
    isr(24, stack_frame);
}

extern "x86-interrupt" fn isr25(stack_frame: InterruptStackFrame) {
    isr(25, stack_frame);
}

extern "x86-interrupt" fn isr26(stack_frame: InterruptStackFrame) {
    isr(26, stack_frame);
}

extern "x86-interrupt" fn isr27(stack_frame: InterruptStackFrame) {
    isr(27, stack_frame);
}

extern "x86-interrupt" fn isr28(stack_frame: InterruptStackFrame) {
    isr(28, stack_frame);
}

extern "x86-interrupt" fn isr29(stack_frame: InterruptStackFrame) {
    isr(29, stack_frame);
}

extern "x86-interrupt" fn isr30(stack_frame: InterruptStackFrame) {
    isr(30, stack_frame);
}

extern "x86-interrupt" fn isr31(stack_frame: InterruptStackFrame) {
    isr(31, stack_frame);
}
