#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]

extern crate alloc;

mod allocator;
mod asm;
mod font;
mod idt;
mod panic;
mod pic;
mod term;
mod time;

use core::arch::asm;

use alloc::string::ToString;

use crate::{allocator::ALLOCATOR, term::TERM};

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);

static BOOT_INFO_REQUEST: limine::BootInfoRequest = limine::BootInfoRequest::new(0);

static PAGING_INFO_REQUEST: limine::Level5PagingRequest = limine::Level5PagingRequest::new(0);

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    TERM.fg = 0xFF00FF00;

    {
        let paging = PAGING_INFO_REQUEST
            .get_response()
            .get()
            .expect("Failed to get paging info!");
        printk!("paging info: {:#?}\n", paging);
    }

    time::i8253::TIMER0.init(
        100,
        time::i8253::Channel::CH0,
        time::i8253::AccessMode::LoHiByte,
        time::i8253::OperatingMode::RateGenerator,
        time::i8253::BCDMode::Binary,
    );

    let memmap = MEMMAP_REQUEST
        .get_response()
        .get()
        .expect("Failed to get memmap!");
    unsafe { ALLOCATOR.load_limine_memmap(memmap) };

    printk!("Hello, world!\n");
    printk!("Time: {}\n", time::rtc::get_time());

    unsafe {
        let boot_info = BOOT_INFO_REQUEST
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
    pic::init();

    {
        let cpuid = raw_cpuid::CpuId::new();
        printk!(
            "Vendor: {:?}\n",
            cpuid
                .get_vendor_info()
                .map(|x| x.as_str().to_string())
                .unwrap_or("<unknown>".to_string())
        );
        printk!(
            "Name: {:?}\n",
            cpuid
                .get_processor_brand_string()
                .map(|s| s.as_str().to_string())
                .unwrap_or("<unknown>".to_string())
        );
    }

    loop {
        time::i8253::TIMER0.sleep(2000);
        printk!("Time: {}\n", time::rtc::get_time());
    }

    loop {
        unsafe { asm!("nop") };
    }

    panic!("Kernel returned from main!");
}
