#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]

extern crate alloc;

mod driver;
mod elf;
mod fifo;
mod font;
mod idt;
mod kernel;
mod logger;
mod pic;
mod process;
mod scheduler;
mod spinlock;
mod stuff;
mod syscall;
mod term;

use fifo::BoundedAtomicFifo;
use log::info;

use spinlock::Spinlock;
use x86_64::{
    instructions::tables,
    registers::segmentation::{Segment, CS, DS, ES, FS, GS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::{
    kernel::paging::{self, SoosFrameAllocator, SoosPaging},
    process::Process,
    scheduler::SoosScheduler,
    stuff::memmap::SoosMemmap,
    term::TERM,
};

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);

static BOOT_INFO_REQUEST: limine::BootInfoRequest = limine::BootInfoRequest::new(0);

static PAGING_MODE_REQUEST: limine::PagingModeRequest =
    limine::PagingModeRequest::new(0).mode(limine::PagingMode::Lvl4);

static HHDM_REQUEST: limine::HhdmRequest = limine::HhdmRequest::new(0);

const KERNEL_STACK_SIZE: usize = 4192 * 100;
static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

static mut KERNEL_PAGING: Option<SoosPaging> = None;

static mut KERNEL_STACK_SEGMENT: Option<SegmentSelector> = None;
static mut KERNEL_CODE_SEGMENT: Option<SegmentSelector> = None;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    static mut TSS: TaskStateSegment = TaskStateSegment::new();
    TSS.privilege_stack_table = [
        VirtAddr::new(KERNEL_STACK.as_mut_ptr() as u64 + KERNEL_STACK.len() as u64),
        VirtAddr::zero(),
        VirtAddr::zero(),
    ];
    for i in 0..7 {
        TSS.interrupt_stack_table[i] =
            VirtAddr::new(KERNEL_STACK.as_mut_ptr() as u64 + KERNEL_STACK.len() as u64);
    }

    static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
    let cs = GDT.add_entry(Descriptor::kernel_code_segment());
    let ds = GDT.add_entry(Descriptor::kernel_data_segment());
    let ucs = GDT.add_entry(Descriptor::user_code_segment());
    let uds = GDT.add_entry(Descriptor::user_data_segment());
    let tss = GDT.add_entry(Descriptor::tss_segment(&TSS));

    KERNEL_STACK_SEGMENT = Some(ds);
    KERNEL_CODE_SEGMENT = Some(cs);

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

    let mut frame_allocator = SoosFrameAllocator::get_or_init_with_current_pagetable(memmap);

    let kernel_page_table = paging::current_page_table();
    KERNEL_PAGING = Some(SoosPaging::offset_page_table(
        hhdm.offset,
        &mut *kernel_page_table,
    ));

    // no allocation before this point!
    kernel::allocator::init_kernel_heap(
        KERNEL_PAGING.as_mut().expect("Failed to init kernel heap!"),
        &mut frame_allocator,
    )
    .expect("Failed to init kernel heap!");

    log::set_logger(&logger::KernelLogger {}).expect("Failed to set logger!");
    log::set_max_level(log::LevelFilter::Trace);
    info!("Logger initialized!");

    idt::load_idt();
    pic::init();

    driver::i8253::TIMER0.init(
        10,
        driver::i8253::Channel::CH0,
        driver::i8253::AccessMode::LoHiByte,
        driver::i8253::OperatingMode::RateGenerator,
        driver::i8253::BCDMode::Binary,
    );

    {
        let process = Process::from_elf_bytes(
            include_bytes!("../userspace/main.elf"),
            hhdm.offset,
            kernel_page_table,
            frame_allocator,
            ucs,
            uds,
            VirtAddr::new(0x0000_5555_ABBA_0000),
            VirtAddr::new(0x0000_5555_ACDC_0000),
            100,
        );
        info!("Process {:?} loaded!", process.pid);
        KERNEL_PAGING
            .as_mut()
            .map(|paging| paging.load())
            .expect("Kernel paging not initialized!");
        SCHEDULER.inner_unsafe().schedule(process);
    }
    /*{
        let process = Process::from_elf_bytes(
            include_bytes!("../userspace/main.elf"),
            hhdm.offset,
            kernel_page_table,
            frame_allocator,
            ucs,
            uds,
            VirtAddr::new(0x0000_4444_ABBA_0000),
            VirtAddr::new(0x0000_4444_ACDC_0000),
            100,
        );
        info!("Process {:?} loaded!", process.pid);
        KERNEL_PAGING
            .as_mut()
            .map(|paging| paging.load())
            .expect("Kernel paging not initialized!");
        SCHEDULER.schedule(process);
    }*/

    SCHEDULER.inner_unsafe().run();
}

static mut SCHEDULER: SoosScheduler = SoosScheduler::new();
