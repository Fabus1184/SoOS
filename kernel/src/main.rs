#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]
#![feature(never_type)]
#![warn(clippy::pedantic)]
#![warn(clippy::style)]
#![warn(clippy::correctness)]
#![warn(clippy::perf)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::similar_names)]

extern crate alloc;

mod driver;
mod elf;
mod font;
mod idt;
mod io;
mod kernel;
mod pic;
mod process;
mod stuff;
mod syscall;
mod term;
mod vfs;

use core::arch::asm;

use limine::request::{HhdmRequest, MemoryMapRequest, PagingModeRequest};
use log::{debug, info, LevelFilter};

use x86_64::{
    instructions::tables,
    registers::segmentation::{Segment, CS, DS, ES, FS, GS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        paging::mapper::CleanUp,
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::{
    driver::i8253,
    kernel::paging::{self, SoosFrameAllocator, SoosPaging},
    stuff::memmap::SoosMemmap,
    term::TERM,
};

static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

static PAGING_MODE_REQUEST: PagingModeRequest =
    PagingModeRequest::new().with_mode(limine::paging::Mode::FOUR_LEVEL);

static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

static KERNEL_PAGING: spin::Lazy<spin::Mutex<SoosPaging>> = spin::Lazy::new(|| {
    spin::Mutex::new(SoosPaging::offset_page_table(0, unsafe {
        core::mem::MaybeUninit::zeroed().assume_init()
    }))
});

static FILE_SYSTEM: spin::Lazy<spin::Mutex<vfs::Directory>> =
    spin::Lazy::new(|| spin::Mutex::new(vfs::Directory::new(&["home", "bin"])));

static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];
const KERNEL_STACK_SIZE: usize = 4192 * 100;
const KERNEL_STACK_POINTER: fn() -> u64 =
    || unsafe { KERNEL_STACK.as_mut_ptr() as u64 + KERNEL_STACK.len() as u64 - 1 };

extern "C" {
    static KERNEL_MEMORY_START: u8;
    static KERNEL_MEMORY_END: u8;
}

const KERNEL_HEAP_SIZE: usize = 0x1_000_000; // 16 MiB
static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    kernel::logger::init(LevelFilter::Debug);

    static mut TSS: TaskStateSegment = TaskStateSegment::new();
    TSS.privilege_stack_table = [
        VirtAddr::new(KERNEL_STACK_POINTER()),
        VirtAddr::zero(),
        VirtAddr::zero(),
    ];
    for i in 0..7 {
        TSS.interrupt_stack_table[i] = VirtAddr::new(KERNEL_STACK_POINTER());
    }

    static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
    let cs = GDT.append(Descriptor::kernel_code_segment());
    let ds = GDT.append(Descriptor::kernel_data_segment());
    let ucs = GDT.append(Descriptor::user_code_segment());
    let uds = GDT.append(Descriptor::user_data_segment());
    let tss = GDT.append(Descriptor::tss_segment(&TSS));

    GDT.load();

    CS::set_reg(cs);
    DS::set_reg(ds);
    ES::set_reg(ds);
    FS::set_reg(ds);
    GS::set_reg(ds);
    SS::set_reg(ds);

    log::info!("SoOS version {}", env!("CARGO_PKG_VERSION"));

    log::debug!(
        "kernel memory {:#x} - {:#x}",
        (&raw const KERNEL_MEMORY_START) as u64,
        (&raw const KERNEL_MEMORY_END) as u64
    );

    let rip = x86_64::registers::read_rip();
    let rsp: u64;
    asm!("mov {}, rsp", out(reg) rsp);
    debug!("rsp: {rsp:#x}, rip: {rip:#x}");
    debug!(
        "UCS: {:#x}, UDS: {:#x}, KCS: {:#x}, KDS: {:#x}",
        ucs.0, uds.0, cs.0, ds.0
    );

    tables::load_tss(tss);

    let paging = PAGING_MODE_REQUEST
        .get_response()
        .expect("Failed to get paging mode!");
    assert!(
        paging.mode() == limine::paging::Mode::FOUR_LEVEL,
        "Bootloader did not set up 4-level paging!"
    );

    {
        let fb = &term::TERM.framebuffer;
        log::debug!(
            "framebuffer {}x{} at {:#x}",
            fb.width(),
            fb.height(),
            fb.addr() as u64
        );
    }

    let hhdm = HHDM_REQUEST.get_response().expect("Failed to get HHDM!");

    let current_page_table = paging::current_page_table();
    log::debug!("Current page table: {:#x}", current_page_table as u64);

    // copy page table

    {
        static mut KERNEL_PAGE_TABLE: x86_64::structures::paging::PageTable =
            x86_64::structures::paging::PageTable::new();

        log::debug!(
            "Copying current page table at {:#x} to kernel page table at {:#x}",
            current_page_table as u64,
            &raw const KERNEL_PAGE_TABLE as u64
        );

        *KERNEL_PAGING.lock() =
            SoosPaging::offset_page_table(hhdm.offset(), &mut KERNEL_PAGE_TABLE);
    }

    {
        let mut paging = KERNEL_PAGING.lock();

        paging.load();

        log::debug!("kernel paging loaded");

        // clean up lower half memory
        cleanup(paging.offset_page_table.level_4_table_mut());

        x86_64::instructions::tlb::flush_all();
    }

    let limine_memmap = MEMMAP_REQUEST
        .get_response()
        .expect("Failed to get memmap!");
    let memmap = SoosMemmap::from(limine_memmap);
    SoosFrameAllocator::init_empty(&memmap);
    log::info!("memory map");
    for entry in memmap.iter() {
        log::info!(
            "  {:#x} {:#} - {:?}",
            entry.base,
            byte_unit::Byte::from_u64(entry.len),
            entry.type_
        );
    }

    {
        KERNEL_PAGING
            .lock()
            .offset_page_table
            .clean_up(kernel::paging::SOOS_FRAME_ALLOCATOR.as_mut().unwrap());
    }

    // no allocation before this point!
    kernel::allocator::init_kernel_heap(KERNEL_HEAP.as_mut_ptr(), KERNEL_HEAP_SIZE);

    idt::load_idt();
    pic::init();

    i8253::TIMER0.init(
        10,
        i8253::Channel::CH0,
        i8253::AccessMode::LoHiByte,
        i8253::OperatingMode::RateGenerator,
        i8253::BCDMode::Binary,
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

    {
        process::PROCESSES
            .lock()
            .push_back(process::Process::user_from_elf(
                hhdm.offset(),
                ucs,
                uds,
                0x202,
                include_bytes_aligned::include_bytes_aligned!(32, "../../build/userspace/bin/sosh"),
            ));
    }

    {
        let mut fs = FILE_SYSTEM.lock();
        debug!("VFS: ");
        fs.create_file("/home/test", vfs::File::regular(b"Hello World!"));
        fs.create_file(
            "/proc/pci/devices",
            vfs::File::special(
                |_self, offset, writer| {
                    if offset != 0 {
                        return Err(crate::io::WriteError::InvalidOffset);
                    }

                    let mut written = 0;

                    for dev in driver::pci::scan().expect("Failed to scan PCI devices!") {
                        let line = alloc::format!(
                            "bus {} device {} function {} class {:?}\n",
                            dev.bus,
                            dev.device,
                            dev.function,
                            dev.header.class
                        );

                        writer.write(line.as_bytes())?;
                        written += line.len();
                    }

                    writer.write(b"test\n")?;

                    Ok(written)
                },
                |_, _, _| panic!("Not implemented!"),
            ),
        );
        fs.print();
    }

    //loop {}

    process::try_schedule().unwrap_or_else(|| {
        panic!("No process ready to run!");
    });
}

extern "C" {
    pub fn do_iret(cs: u64, ds: u64, flags: u64, rip: u64, regs: *const idt::GPRegisters) -> !;
}

fn cleanup(page_table: &mut x86_64::structures::paging::PageTable) {
    for (l4_index, l4_entry) in page_table.iter_mut().enumerate().take(256) {
        if l4_entry
            .flags()
            .contains(x86_64::structures::paging::PageTableFlags::PRESENT)
        {
            let l3_table = l4_entry.addr().as_u64() as *mut x86_64::structures::paging::PageTable;
            let l3_table = unsafe { &mut *l3_table };
            for (l3_index, l3_entry) in l3_table.iter_mut().enumerate() {
                if l3_entry
                    .flags()
                    .contains(x86_64::structures::paging::PageTableFlags::HUGE_PAGE)
                {
                    huge_pages
                        .push(l3_entry)
                        .expect("Failed to push huge page to vector");
                    continue;
                }

                if l3_entry
                    .flags()
                    .contains(x86_64::structures::paging::PageTableFlags::PRESENT)
                {
                    let l2_table =
                        l3_entry.addr().as_u64() as *mut x86_64::structures::paging::PageTable;
                    let l2_table = unsafe { &mut *l2_table };
                    for (l2_index, l2_entry) in l2_table.iter_mut().enumerate() {
                        if l2_entry
                            .flags()
                            .contains(x86_64::structures::paging::PageTableFlags::HUGE_PAGE)
                        {
                            huge_pages
                                .push(l2_entry)
                                .expect("Failed to push huge page to vector");
                            continue;
                        }

                        if l2_entry
                            .flags()
                            .contains(x86_64::structures::paging::PageTableFlags::PRESENT)
                        {
                            let l1_table = l2_entry.addr().as_u64()
                                as *mut x86_64::structures::paging::PageTable;
                            let l1_table = unsafe { &mut *l1_table };
                            for (l1_index, l1_entry) in l1_table.iter_mut().enumerate() {
                                let virt_addr = (l4_index as u64) << 39
                                    | (l3_index as u64) << 30
                                    | (l2_index as u64) << 21
                                    | (l1_index as u64) << 12;

                                if virt_addr >= 0x1_000_000
                                    && l1_entry.flags().contains(
                                        x86_64::structures::paging::PageTableFlags::PRESENT,
                                    )
                                {
                                    log::trace!(
                                        "removing page {:#x} -> {:#x} at frame {:#x}",
                                        virt_addr,
                                        l1_entry.addr().as_u64(),
                                        core::ptr::from_ref(l1_entry) as u64,
                                    );

                                    l1_entry.set_unused();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
