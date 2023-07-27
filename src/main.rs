#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]

extern crate alloc;

mod asm;
mod driver;
mod font;
mod idt;
mod kernel;
mod pic;
mod stuff;
mod term;

use alloc::string::ToString;
use core::{arch::asm, slice};
use x86_64::{
    instructions::{self, interrupts, tables},
    registers::{
        rflags::{self, RFlags},
        segmentation::{Segment, CS, DS, ES, FS, GS, SS},
    },
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        paging::{FrameAllocator, Mapper},
        tss::TaskStateSegment,
    },
    PrivilegeLevel, VirtAddr,
};

use crate::term::TERM;

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);

static BOOT_INFO_REQUEST: limine::BootInfoRequest = limine::BootInfoRequest::new(0);

static PAGING_MODE_REQUEST: limine::PagingModeRequest =
    limine::PagingModeRequest::new(0).mode(limine::PagingMode::Lvl4);

static HHDM_REQUEST: limine::HhdmRequest = limine::HhdmRequest::new(0);

static mut STACK1: [u8; 4096] = [0; 4096];

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    static mut TSS: TaskStateSegment = TaskStateSegment::new();
    TSS.privilege_stack_table = [
        VirtAddr::new(STACK1.as_mut_ptr() as u64 + STACK1.len() as u64),
        VirtAddr::zero(),
        VirtAddr::zero(),
    ];

    static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
    let cs = GDT.add_entry(Descriptor::kernel_code_segment());
    let ds = GDT.add_entry(Descriptor::kernel_data_segment());
    let ucs = GDT.add_entry(Descriptor::user_code_segment());
    let uds = GDT.add_entry(Descriptor::user_data_segment());
    let tss = GDT.add_entry(Descriptor::tss_segment(&TSS));

    GDT.load();

    CS::set_reg(cs);
    DS::set_reg(ds);
    ES::set_reg(ds);
    FS::set_reg(ds);
    GS::set_reg(ds);
    SS::set_reg(ds);

    tables::load_tss(tss);

    TERM.fg = 0xFF00FF00;

    let paging = PAGING_MODE_REQUEST
        .get_response()
        .get()
        .expect("Failed to get paging mode!");
    if paging.mode != limine::PagingMode::Lvl4 {
        panic!("Failed to enable 4-level paging!");
    }

    let hhdm = HHDM_REQUEST
        .get_response()
        .get()
        .expect("Failed to get HHDM!");

    let limine_memmap = MEMMAP_REQUEST
        .get_response()
        .get()
        .expect("Failed to get memmap!");

    let memmap = stuff::memmap::SoosMemmap::from(limine_memmap);

    let frame_allocator = kernel::paging::SoosFrameAllocator::new(memmap);
    let mut paging = kernel::paging::SoosPaging::new(hhdm.offset, frame_allocator);

    // no allocation before this point!
    kernel::allocator::init_kernel_heap(&mut paging).expect("Failed to init kernel heap!");

    idt::load_idt();
    pic::init();

    driver::i8253::TIMER0.init(
        100,
        driver::i8253::Channel::CH0,
        driver::i8253::AccessMode::LoHiByte,
        driver::i8253::OperatingMode::RateGenerator,
        driver::i8253::BCDMode::Binary,
    );

    asm!(
        "cli",
        "mov ax, {uds:x}",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "push {uds:r}",
        "push {stack:r}",
        "push {rflags:r}",
        "push {ucs:r}",
        "push {userland_function:r}",
        "iretq",
        uds = in(reg) ((uds.index() * 8) | 3),
        ucs = in(reg) ((ucs.index() * 8) | 3),
        stack = in(reg) 0xabcdef,
        rflags = in(reg) 0x200,
        userland_function = in(reg) 0x1234,
        options(noreturn),
    );

    printk!("Hello, world!\n");
    printk!("Time: {}\n", driver::rtc::get_time());

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
        driver::i8253::TIMER0.sleep(2000);
        printk!("Time: {}\n", driver::rtc::get_time());
    }
}
