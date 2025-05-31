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

use alloc::{borrow::ToOwned, format};
use include_bytes_aligned::include_bytes_aligned;
use log::{info, LevelFilter};

use x86_64::{
    instructions::tables,
    registers::segmentation::{Segment, CS, DS, ES, FS, GS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        paging::{FrameAllocator, PageTable},
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
    stuff::memmap::SoosMemmap,
    term::TERM,
};

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);

static PAGING_MODE_REQUEST: limine::PagingModeRequest =
    limine::PagingModeRequest::new(0).mode(limine::PagingMode::Lvl4);

static HHDM_REQUEST: limine::HhdmRequest = limine::HhdmRequest::new(0);

const BANNER: &[&str] = &[
    r#""#,
    r#".d88888b            .88888.  .d88888b "#,
    r#"88.    "'          d8'   `8b 88.    "'"#,
    r#"`Y88888b. .d8888b. 88     88 `Y88888b."#,
    r#"      `8b 88'  `88 88     88       `8b"#,
    r#"d8'   .8P 88.  .88 Y8.   .8P d8'   .8P"#,
    r#" Y88888P  `88888P'  `8888P'   Y88888P "#,
    r#""#,
];

static mut KERNEL_PAGING: *mut SoosPaging = core::ptr::null_mut();

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];
    const KERNEL_STACK_SIZE: usize = 4192 * 100;
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

    TERM.set_color(0xFF00FF00, 0xFF000000);

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
    let mut _kernel_paging = SoosPaging::offset_page_table(hhdm.offset, &mut *kernel_page_table);
    KERNEL_PAGING = &mut _kernel_paging;
    (*KERNEL_PAGING).load();

    let heap = memmap
        .iter()
        .find(|entry| entry.len >= 100 * 1024 * 1024)
        .expect("Failed to find a suitable heap location!");

    // no allocation before this point!
    kernel::allocator::init_kernel_heap(heap.base).expect("Failed to init kernel heap!");

    KernelLogger::new(LevelFilter::Debug).init();

    for line in BANNER {
        TERM.println(line);
    }
    TERM.print("Version ");
    TERM.println(env!("CARGO_PKG_VERSION"));

    TERM.println("Memory map:");
    for entry in memmap.iter() {
        TERM.println(&format!(
            "  {:#x} {:#} - {:?}",
            entry.base,
            byte_unit::Byte::from_u64(entry.len),
            entry.type_
        ));
    }

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

    let rsp: u64;
    asm!("mov {}, rsp", out(reg) rsp);
    info!("RSP: {:#x}", rsp);

    info!(
        "UCS: {:#x}, UDS: {:#x}, KCS: {:#x}, KDS: {:#x}",
        ucs.0, uds.0, cs.0, ds.0
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

    let process_page_table = frame_allocator
        .allocate_frame()
        .expect("Failed to allocate frame!")
        .start_address()
        .as_u64() as *mut PageTable;

    (*KERNEL_PAGING)
        .offset_page_table
        .level_4_table()
        .clone_into(&mut *process_page_table);

    info!(
        "Address of page table: {:#x}",
        process_page_table as *const _ as u64
    );
    let mut process_paging = SoosPaging::offset_page_table(hhdm.offset, &mut *process_page_table);

    let (userspace_address, userspace_stack) = elf::load(
        &mut process_paging,
        &mut *KERNEL_PAGING,
        frame_allocator,
        include_bytes_aligned!(32, "../userspace/build/fib"),
        VirtAddr::new(0x0000_1234_0000_0000),
    );

    info!(
        "ELF loaded at {:?}, stack at {:?}",
        userspace_address, userspace_stack
    );

    {
        process::PROCESSES
            .lock()
            .push(process::Process::user_process(
                123,
                process_paging,
                ucs,
                uds,
                0x202,
                userspace_address.as_u64(),
                userspace_stack.as_u64(),
            ));
    }

    {
        process::PROCESSES
            .lock()
            .push(process::Process::kernel_process(
                0,
                CS::get_reg(),
                DS::get_reg(),
                0x202,
                kernel_process as usize as u64,
                kernel_stack_pointer,
            ));
    }

    process::try_schedule().unwrap_or_else(|| {
        panic!("No process ready to run!");
    });
}

fn kernel_process() {
    log::trace!("Kernel process running...");
    for _ in 0..100_000 {
        core::hint::spin_loop();
    }
    process::try_schedule().expect("Failed to schedule a process!");
}

extern "C" {
    pub fn do_iret(cs: u64, ds: u64, flags: u64, rip: u64, regs: *const idt::GPRegisters) -> !;
}
