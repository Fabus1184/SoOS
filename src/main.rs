#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]
#![feature(never_type)]

extern crate alloc;

mod driver;
mod elf;
mod font;
mod idt;
mod kernel;
mod pic;
mod process;
mod stuff;
mod term;

use core::arch::asm;

use include_bytes_aligned::include_bytes_aligned;
use log::{info, LevelFilter};

use x86_64::{
    instructions::tables,
    registers::segmentation::{Segment, CS, DS, ES, FS, GS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        paging::{FrameAllocator, Mapper, Page, PageSize, PageTableFlags, Size4KiB},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::{
    driver::i8253,
    kernel::{
        logger::KernelLogger,
        paging::{self, SoosFrameAllocator, SoosPaging},
    },
    process::{Process, PROCESSES},
    stuff::memmap::SoosMemmap,
    term::TERM,
};

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);

static BOOT_INFO_REQUEST: limine::BootInfoRequest = limine::BootInfoRequest::new(0);

static PAGING_MODE_REQUEST: limine::PagingModeRequest =
    limine::PagingModeRequest::new(0).mode(limine::PagingMode::Lvl4);

static HHDM_REQUEST: limine::HhdmRequest = limine::HhdmRequest::new(0);

const KERNEL_STACK_SIZE: usize = 4192 * 100;
pub static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    let kernel_stack_pointer = KERNEL_STACK.as_mut_ptr() as u64 + KERNEL_STACK.len() as u64;

    static mut TSS: TaskStateSegment = TaskStateSegment::new();
    TSS.privilege_stack_table = [
        VirtAddr::new(kernel_stack_pointer),
        VirtAddr::zero(),
        VirtAddr::zero(),
    ];
    for i in 0..7 {
        TSS.interrupt_stack_table[i] = VirtAddr::new(kernel_stack_pointer);
    }

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

    let memmap = SoosMemmap::from(limine_memmap);

    let frame_allocator = SoosFrameAllocator::get_or_init_with_current_pagetable(memmap);

    let kernel_page_table = paging::current_page_table();
    let mut paging = SoosPaging::offset_page_table(hhdm.offset, &mut *kernel_page_table);

    // no allocation before this point!
    kernel::allocator::init_kernel_heap(&mut paging, frame_allocator)
        .expect("Failed to init kernel heap!");

    KernelLogger::new(LevelFilter::Debug).init();

    info!("Logger initialized!");

    idt::load_idt();
    pic::init();

    i8253::TIMER0.init(
        10,
        i8253::Channel::CH0,
        i8253::AccessMode::LoHiByte,
        i8253::OperatingMode::RateGenerator,
        i8253::BCDMode::Binary,
    );

    let rip = x86_64::registers::read_rip();
    info!("RIP: {:#x}", rip);
    let mut rsp: u64;
    asm!("mov {}, rsp", out(reg) rsp);
    info!("RSP: {:#x}", rsp);

    info!(
        "User code segment: {:#x}, user data segment: {:#x}",
        ucs.0, uds.0
    );

    driver::pci::scan()
        .expect("Failed to scan PCI devices!")
        .into_iter()
        .for_each(|dev| {
            info!(
                "Found PCI device: bus {} device {} function {} class {:?}",
                dev.bus, dev.device, dev.function, dev.header.class
            );
        });

    let userspace_address = elf::load(
        &mut paging,
        frame_allocator,
        include_bytes_aligned!(32, "../userspace/build/test"),
        VirtAddr::new(0x0000_1234_0000_0000),
    );

    info!("ELF loaded at {:?}", userspace_address);

    let stack_size_pages = 10;
    let stack_address = VirtAddr::new(0x0000_1000_0000_0000);
    for i in 0..stack_size_pages {
        let page = Page::containing_address(stack_address + i * Size4KiB::SIZE);
        let frame = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!");
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::USER_ACCESSIBLE
            | PageTableFlags::NO_EXECUTE;
        unsafe {
            paging
                .offset_page_table
                .map_to(page, frame, flags, frame_allocator)
                .expect("Failed to map page!")
                .flush()
        };
    }
    let stack_pointer = stack_address + stack_size_pages * Size4KiB::SIZE - 0x100_u64;

    {
        let mut processes = PROCESSES.lock();

        processes.push(Process::new(
            1234,
            userspace_address.as_u64(),
            stack_pointer.as_u64(),
            uds.0 as u64,
            ucs.0 as u64,
            0x202,
        ));
    }

    process::schedule()
}
