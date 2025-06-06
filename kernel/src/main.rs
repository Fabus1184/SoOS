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

use ringbuffer::RingBuffer;
use x86_64::{
    instructions::tables,
    registers::segmentation::{Segment, CS, DS, ES, FS, GS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        paging::{OffsetPageTable, Page},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::{
    driver::i8253,
    kernel::paging::{self, KernelPaging},
    stuff::memmap::SoosMemmap,
    term::TERM,
};

static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

static PAGING_MODE_REQUEST: PagingModeRequest =
    PagingModeRequest::new().with_mode(limine::paging::Mode::FOUR_LEVEL);

static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

static FILE_SYSTEM: spin::Lazy<spin::Mutex<vfs::Directory>> =
    spin::Lazy::new(|| spin::Mutex::new(vfs::Directory::new(&["home", "bin"])));

static KERNEL_MEMORY_START_ADDR: spin::Lazy<u64> = spin::Lazy::new(|| {
    extern "C" {
        static KERNEL_MEMORY_START: u8;
    }
    &raw const KERNEL_MEMORY_START as u64
});
static KERNEL_MEMORY_END_ADDR: spin::Lazy<u64> = spin::Lazy::new(|| {
    extern "C" {
        static KERNEL_MEMORY_END: u8;
    }
    &raw const KERNEL_MEMORY_END as u64
});

const KERNEL_STACK_SIZE: usize = 4192 * 1000;
static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];
const KERNEL_STACK_POINTER: fn() -> u64 =
    || unsafe { (KERNEL_STACK.as_mut_ptr() as u64 + KERNEL_STACK.len() as u64 - 0xF) & !0xF };

static KERNEL_PAGING: spin::Once<spin::Mutex<KernelPaging>> = spin::Once::new();

fn kernel_paging() -> spin::MutexGuard<'static, KernelPaging> {
    KERNEL_PAGING
        .get()
        .expect("Kernel paging not initialized!")
        .try_lock()
        .expect("Failed to lock kernel paging!")
}

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    asm!("
        mov rsp, {0}
        jmp main
        ",
        in(reg) KERNEL_STACK_POINTER(), options(noreturn)
    );
}

#[no_mangle]
unsafe extern "C" fn main() -> ! {
    kernel::logger::KERNEL_LOGGER.init(LevelFilter::Debug);

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
        *KERNEL_MEMORY_START_ADDR,
        *KERNEL_MEMORY_END_ADDR
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
            "framebuffer {}x{} (pitch {}) at {:#x}",
            fb.width,
            fb.height,
            fb.pitch,
            fb.ptr as u64
        );
    }

    let hhdm = HHDM_REQUEST.get_response().expect("Failed to get HHDM!");

    let current_page_table = paging::current_page_table();
    log::debug!("Current page table: {:#x}", current_page_table as u64);

    let limine_memmap = MEMMAP_REQUEST
        .get_response()
        .expect("Failed to get memmap!");
    let memmap = SoosMemmap::from(limine_memmap);
    log::info!("memory map");
    for entry in memmap.iter() {
        log::info!(
            "{:#12x} {:>#12} - {:?}",
            entry.base,
            byte_unit::Byte::from_u64(entry.len),
            entry.type_
        );
    }

    idt::load_idt();
    pic::init();

    let offset = hhdm.offset();

    let mut current_page_table =
        OffsetPageTable::new(&mut *current_page_table, VirtAddr::new(offset));

    KERNEL_PAGING.call_once(|| {
        spin::Mutex::new(KernelPaging::make_kernel_paging(
            &memmap,
            &mut current_page_table,
            Page::range_inclusive(
                Page::containing_address(VirtAddr::new(*KERNEL_MEMORY_START_ADDR)),
                Page::containing_address(VirtAddr::new(*KERNEL_MEMORY_END_ADDR)),
            )
            .chain(Page::range_inclusive(
                Page::containing_address(VirtAddr::new(term::TERM.framebuffer.ptr as u64)),
                Page::containing_address(VirtAddr::new(
                    term::TERM.framebuffer.ptr as u64
                        + term::TERM.framebuffer.width * term::TERM.framebuffer.height * 4,
                )),
            ))
            .chain(Page::range(
                Page::containing_address(VirtAddr::new(
                    kernel::paging::KERNEL_FRAME_MAPPING_ADDRESS,
                )),
                Page::containing_address(VirtAddr::new(
                    kernel::paging::KERNEL_FRAME_MAPPING_ADDRESS + (1 << 20) * 0x1000,
                )),
            )),
        ))
    });

    // no allocation before this point!
    {
        const KERNEL_HEAP_SIZE: usize = 0x1_000_000; // 16 MiB
        static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
        kernel::allocator::init_kernel_heap(KERNEL_HEAP.as_mut_ptr(), KERNEL_HEAP_SIZE);
    }

    kernel::logger::KERNEL_LOGGER.init_ringbuffer();

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
            .try_write()
            .expect("Failed to lock processes")
            .push_back(process::Process::user_from_elf(
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
            "/var/log",
            vfs::File::Special {
                read: |_, offset, writer| {
                    let mut written = 0;
                    let ringbuffer = kernel::logger::KERNEL_LOGGER.lock_ringbuffer();
                    for byte in ringbuffer.iter().skip(offset) {
                        written += writer.write(&[*byte])?;
                    }

                    Ok(written)
                },
                write: |_, _, _| panic!("Not implemented!"),
            },
        );
        fs.create_file(
            "/proc/list",
            vfs::File::special(
                |_self, offset, writer| {
                    if offset != 0 {
                        return Err(crate::io::WriteError::InvalidOffset);
                    }

                    let mut written = 0;

                    for process in process::PROCESSES
                        .try_read()
                        .expect("Failed to lock processes")
                        .iter()
                    {
                        let line = alloc::format!(
                            "pid {} state {:?} rip {:#x}\n",
                            process.pid(),
                            process.state,
                            process.rip
                        );

                        writer.write(line.as_bytes())?;
                        written += line.len();
                    }

                    Ok(written)
                },
                |_, _, _| panic!("Not implemented!"),
            ),
        );
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

    log::info!("kernel initialization complete, starting scheduler");

    process::try_schedule().unwrap_or_else(|| {
        panic!("No process ready to run!");
    });
}

extern "C" {
    pub fn do_iret(cs: u64, ds: u64, flags: u64, rip: u64, regs: *const idt::GPRegisters) -> !;
}
