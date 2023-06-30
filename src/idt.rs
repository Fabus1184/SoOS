use core::arch::asm;

use crate::{isr::ISRS, printk};

#[repr(C)]
#[derive(Copy, Clone)]
struct InterruptDescriptor {
    offset_low: u16,
    selector: u16,
    ist_index: u8,
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

#[repr(C)]
struct InterruptDescriptorTable {
    limit: u16,
    base: u64,
}

static mut IDS: [InterruptDescriptor; 32] = [InterruptDescriptor {
    offset_low: 0,
    selector: 0,
    ist_index: 0,
    flags: 0,
    offset_mid: 0,
    offset_high: 0,
    zero: 0,
}; 32];

pub fn load_idt() {
    for (i, isr) in ISRS.iter().enumerate() {
        unsafe {
            IDS[i] = InterruptDescriptor {
                offset_low: (isr as *const _ as u64 & 0xFFFF) as u16,
                selector: 0x08,
                ist_index: 0,
                flags: 0x8E,
                offset_mid: ((isr as *const _ as u64 >> 16) & 0xFFFF) as u16,
                offset_high: ((isr as *const _ as u64 >> 32) & 0xFFFFFFFF) as u32,
                zero: 0,
            };
        }
    }

    let idt = InterruptDescriptorTable {
        limit: (core::mem::size_of::<InterruptDescriptor>() * 32 - 1) as u16,
        base: unsafe { IDS.as_ptr() as u64 },
    };

    unsafe {
        asm!("cli");
        asm!("lidt [{}]", in(reg) &idt);
        asm!("sti");
    }
}
