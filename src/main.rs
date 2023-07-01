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

use crate::{allocator::ALLOCATOR, term::TERM};

static MEMMAP_REQUEST: limine::LimineMemmapRequest = limine::LimineMemmapRequest::new(0);

static BOOT_INFO_REQUEST: limine::LimineBootInfoRequest = limine::LimineBootInfoRequest::new(0);

static PAGING_INFO_REQUEST: limine::Limine5LevelPagingRequest =
    limine::Limine5LevelPagingRequest::new(0);

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    TERM.fg = 0xFF00FF00;

    {
        let ptr = PAGING_INFO_REQUEST.get_response().as_ptr();
        printk!("Paging info: {:#?}\n", ptr);
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
    printk!("IDT loaded!\n");

    pic::init();
    printk!("APIC initialized!\n");

    {
        let cpuid = raw_cpuid::CpuId::new();
        printk!("Vendor: {:?}\n", cpuid.get_vendor_info());
        printk!("Feature Info: {:?}\n", cpuid.get_feature_info());
        printk!(
            "Extended Feature Info: {:?}\n",
            cpuid.get_extended_processor_and_feature_identifiers()
        );
    }

    loop {
        unsafe { asm!("nop") };
    }

    panic!("Kernel returned from main!");
}
