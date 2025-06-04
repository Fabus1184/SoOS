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
        paging::FrameAllocator,
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

static mut KERNEL_PAGING: *mut SoosPaging = core::ptr::null_mut();

static FILE_SYSTEM: spin::Lazy<spin::Mutex<vfs::Directory>> =
    spin::Lazy::new(|| spin::Mutex::new(vfs::Directory::new(&["home", "bin"])));

static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];
const KERNEL_STACK_SIZE: usize = 4192 * 100;
const KERNEL_STACK_POINTER: fn() -> u64 =
    || unsafe { KERNEL_STACK.as_mut_ptr() as u64 + KERNEL_STACK.len() as u64 - 1 };

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

    tables::load_tss(tss);

    let paging = PAGING_MODE_REQUEST
        .get_response()
        .expect("Failed to get paging mode!");
    if paging.mode() != limine::paging::Mode::FOUR_LEVEL {
        panic!("Failed to enable 4-level paging!");
    }

    let hhdm = HHDM_REQUEST.get_response().expect("Failed to get HHDM!");

    let limine_memmap = MEMMAP_REQUEST
        .get_response()
        .expect("Failed to get memmap!");

    let memmap = SoosMemmap::from(limine_memmap);

    let kernel_heap = SoosFrameAllocator::init_with_current_pagetable(memmap, 0x1000);

    let kernel_page_table = paging::current_page_table();
    let mut _kernel_paging = SoosPaging::offset_page_table(hhdm.offset(), &mut *kernel_page_table);
    KERNEL_PAGING = &mut _kernel_paging;
    (*KERNEL_PAGING).load();

    // no allocation before this point!
    kernel::allocator::init_kernel_heap(kernel_heap, 0x1000 * 4096)
        .expect("Failed to init kernel heap!");

    log::info!("SoOS version {}", env!("CARGO_PKG_VERSION"));

    log::info!("Memory map:");
    for entry in memmap.iter() {
        log::info!(
            "  {:#x} {:#} - {:?}",
            entry.base,
            byte_unit::Byte::from_u64(entry.len),
            entry.type_
        );
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

    {
        process::PROCESSES
            .lock()
            .push_back(process::Process::user_from_elf(
                hhdm.offset(),
                ucs,
                uds,
                0x202,
                include_bytes_aligned::include_bytes_aligned!(32, "../userspace/build/sosh"),
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

    process::try_schedule().unwrap_or_else(|| {
        panic!("No process ready to run!");
    });
}

extern "C" {
    pub fn do_iret(cs: u64, ds: u64, flags: u64, rip: u64, regs: *const idt::GPRegisters) -> !;
}
