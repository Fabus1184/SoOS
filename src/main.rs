#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod allocator;
mod font;
mod idt;
mod isr;
mod panic;
mod term;

use alloc::format;
use core::arch::asm;

use crate::{allocator::ALLOCATOR, term::TERM};

static MEMMAP_REQUEST: limine::LimineMemmapRequest = limine::LimineMemmapRequest::new(0);

static BOOT_TIME: limine::LimineBootTimeRequest = limine::LimineBootTimeRequest::new(0);

static BOOT_INFO: limine::LimineBootInfoRequest = limine::LimineBootInfoRequest::new(0);

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    let memmap = MEMMAP_REQUEST
        .get_response()
        .get()
        .expect("Failed to get memmap!");
    unsafe { ALLOCATOR.load_limine_memmap(memmap) };

    TERM.fg = 0xFF00FF00;
    printk!("Hello, world!\n");

    let boot_time = BOOT_TIME
        .get_response()
        .get()
        .map(|x| x.boot_time)
        .unwrap_or(0);
    printk!(
        "Boot time: {:?}\n",
        chrono::NaiveDateTime::from_timestamp_opt(boot_time as i64, 0).expect("Invalid time!")
    );

    unsafe {
        let boot_info = BOOT_INFO
            .get_response()
            .get()
            .expect("Failed to get boot info!");
        let name = boot_info
            .name
            .as_ptr()
            .and_then(|x| core::ffi::CStr::from_ptr(x).to_str().ok())
            .unwrap_or("<failed to get name>");
        let version = boot_info
            .version
            .as_ptr()
            .and_then(|x| core::ffi::CStr::from_ptr(x).to_str().ok())
            .unwrap_or("<failed to get version>");
        printk!(
            "Bootloader: {} {} rev {}\n",
            name,
            version,
            boot_info.revision
        );
    }

    {
        printk!("memory map:\n");
        for i in 0..memmap.entry_count {
            let entry = unsafe { memmap.entries.as_ptr().wrapping_offset(i as isize).read() };
            printk!(
                "  base: {:#16x}, len: {:#14x}, type: {:?}\n",
                entry.base,
                entry.len,
                entry.typ
            );
        }
    }

    idt::load_idt();
    printk!("IDT loaded!\n");

    loop {
        unsafe { asm!("nop") };
    }

    panic!("Kernel returned from main!");
}
