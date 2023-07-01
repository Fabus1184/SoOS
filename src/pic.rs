use core::arch::asm;

use crate::asm::{inb, outb};

pub fn remap_pic(offset1: u8, offset2: u8) {
    let a1 = unsafe { inb(0x21) };
    let a2 = unsafe { inb(0xA1) };

    unsafe {
        outb(0x20, 0x11);
        outb(0xA0, 0x11);
        outb(0x21, offset1);
        outb(0xA1, offset2);
        outb(0x21, 0x04);
        outb(0xA1, 0x02);
        outb(0x21, 0x01);
        outb(0xA1, 0x01);
        outb(0x21, a1);
        outb(0xA1, a2);
    }
}

pub fn eoi(irq: u8) {
    if irq >= 8 {
        unsafe {
            outb(0xA0, 0x20);
        }
    }
    unsafe {
        outb(0x20, 0x20);
    }
}

pub fn init() {
    remap_pic(0x20, 0x28);

    // unmask all PIC interrupts
    unsafe {
        outb(0x21, 0x00);
        outb(0xA1, 0x00);
    }

    unsafe {
        asm!("sti");
    }
}
