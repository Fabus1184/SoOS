use elf_rs::{Elf, ElfFile, ProgramHeaderFlags, ProgramType};
use log::info;
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageSize, PageTableFlags, Size4KiB},
    VirtAddr,
};

use crate::{
    kernel::paging::{KernelPaging, UserspacePaging},
    process::MappedPage,
};

pub fn load<T: AsRef<str>>(
    process_paging: &mut UserspacePaging,
    kernel_paging: &mut KernelPaging,
    bytes: &[u8],
    args: &[T],
) -> (VirtAddr, VirtAddr, alloc::vec::Vec<MappedPage>) {
    let elf = Elf::from_bytes(bytes).expect("Failed to parse ELF!");
    match elf.elf_header().elftype() {
        elf_rs::ElfType::ET_EXEC => {}
        e => panic!("Unsupported ELF type: {:?}", e),
    }

    let mut pages = alloc::vec::Vec::with_capacity(10);

    // map pages
    for ph in elf
        .program_header_iter()
        .filter(|ph| ph.ph_type() == ProgramType::LOAD)
    {
        let start_page = Page::containing_address(VirtAddr::new(ph.vaddr()));
        let end_page = Page::containing_address(VirtAddr::new(ph.vaddr() + ph.memsz()));

        let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
        if !ph.flags().contains(ProgramHeaderFlags::EXECUTE) {
            flags |= PageTableFlags::NO_EXECUTE;
        }
        if ph.flags().contains(ProgramHeaderFlags::WRITE) {
            flags |= PageTableFlags::WRITABLE;
        }

        info!(
            "mapping code pages [{:#0x} - {:#0x}] with flags {:?}",
            start_page.start_address(),
            end_page.start_address(),
            flags
        );

        // map page for copying program
        for page in Page::range_inclusive(start_page, end_page) {
            pages.push(MappedPage {
                name: "elf",
                page,
                flags,
            });

            let frame = kernel_paging
                .allocate_frame()
                .expect("Failed to allocate frame!");

            unsafe {
                process_paging
                    .page_table
                    .map_to(
                        page,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        &mut *kernel_paging,
                    )
                    .unwrap_or_else(|e| panic!("Failed to map page {:#?}: {:?}", page, e))
                    .flush();
            }
        }

        unsafe {
            process_paging.load();

            // copy data
            core::ptr::copy::<u8>(
                bytes.as_ptr().add(ph.offset() as usize),
                ph.vaddr() as *mut u8,
                ph.filesz() as usize,
            );

            // zero out the rest of the page
            let remaining_size = ph.memsz() as usize - ph.filesz() as usize;
            core::ptr::write_bytes((ph.vaddr() + ph.filesz()) as *mut u8, 0, remaining_size);

            kernel_paging.load();
        }

        // remap the page to the correct flags
        for page in Page::range_inclusive(start_page, end_page) {
            unsafe {
                let (frame, flush) = process_paging
                    .page_table
                    .unmap(page)
                    .expect("Failed to unmap page!");
                flush.flush();

                process_paging
                    .page_table
                    .map_to(page, frame, flags, &mut *kernel_paging)
                    .expect("Failed to remap page!")
                    .flush();
            }
        }
    }

    // create stack
    let stack_size_pages = 100;
    let stack_address = VirtAddr::new(0x0000_1000_0000_0000);
    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE
        | PageTableFlags::NO_EXECUTE;
    info!(
        "mapping stack pages [{:#0x} - {:#0x}] with flags {:?}",
        stack_address,
        stack_address + stack_size_pages * Size4KiB::SIZE,
        flags
    );
    for i in 0..stack_size_pages {
        let page = Page::containing_address(stack_address + i * Size4KiB::SIZE);
        let frame = kernel_paging
            .allocate_frame()
            .expect("Failed to allocate frame!");

        unsafe {
            process_paging
                .page_table
                .map_to(page, frame, flags, &mut *kernel_paging)
                .expect("Failed to map page!")
                .flush();
        }

        pages.push(MappedPage {
            name: "stack",
            page,
            flags,
        });
    }

    let stack_top = stack_address + stack_size_pages * Size4KiB::SIZE;

    // prepare arguments on the stack

    let stack_top_frame = process_paging
        .page_table
        .translate_page(Page::<Size4KiB>::containing_address(stack_top - 1))
        .expect("Failed to translate stack top page");

    log::debug!("preparing arguments on the stack at {stack_top:#x} (frame: {stack_top_frame:x?})");

    {
        let l4_index = Page::<Size4KiB>::containing_address(stack_top - 1)
            .start_address()
            .p4_index();
        let kernel_l4_ptr = &kernel_paging.page_table().level_4_table()[l4_index];
        let process_l4_ptr = &process_paging.page_table.level_4_table()[l4_index];
        log::log!(
            log::Level::Debug,
            "l4_index: {l4_index:?}, kernel L4 entry: {:#x} ({:x?}), process L4 entry: {:#x} ({:x?})",
            kernel_l4_ptr as *const _ as u64,
            kernel_l4_ptr,
            process_l4_ptr as *const _ as u64,
            process_l4_ptr
        );
    }

    unsafe {
        kernel_paging
            .map_to(
                Page::<Size4KiB>::containing_address(stack_top - 1),
                stack_top_frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                &mut crate::kernel::paging::UseKernelFrameAllocator,
            )
            .expect("failed to map stack top page")
            .flush();
    }

    // copy all arg string contents to the stack
    let str_area = stack_top - args.iter().map(|s| s.as_ref().len()).sum::<usize>() as u64;
    log::debug!("arg string area starts at {str_area:#x}");
    let mut offset = 0;
    for arg in args {
        let ptr = unsafe { str_area.as_mut_ptr::<u8>().add(offset) };
        unsafe { core::ptr::copy_nonoverlapping(arg.as_ref().as_ptr(), ptr, arg.as_ref().len()) };

        offset += arg.as_ref().len();
    }

    // create string structs for each argument
    let arg_str_ptr = (str_area - (args.len() * size_of::<crate::types::string_const_t>()) as u64)
        .align_down(align_of::<crate::types::string_const_t>() as u64);
    log::debug!("arg string structs starts at {arg_str_ptr:#x}");
    let mut offset = 0;
    for (i, arg) in args.iter().enumerate() {
        let arg_ptr = unsafe {
            arg_str_ptr
                .as_mut_ptr::<crate::types::string_const_t>()
                .add(i)
        };
        unsafe {
            *arg_ptr = crate::types::string_const_t {
                ptr: str_area.as_ptr::<i8>().add(offset),
                len: arg.as_ref().len() as u32,
            };
        }
        offset += arg.as_ref().len();
    }

    // create entry struct
    let entry_struct_ptr = (arg_str_ptr - size_of::<crate::types::entry_t>() as u64)
        .align_down(align_of::<crate::types::entry_t>() as u64);
    log::debug!("entry pointer starts at {entry_struct_ptr:#x}");
    let entry_args_ptr = entry_struct_ptr.as_mut_ptr::<crate::types::entry_t>();

    unsafe {
        *entry_args_ptr = crate::types::entry_t {
            argc: args.len() as u32,
            argv: arg_str_ptr.as_mut_ptr::<crate::types::string_const_t>(),
        };
    }

    // create pointer to the entry struct
    let entry_ptr = entry_struct_ptr - size_of::<u64>() as u64;
    log::debug!("entry pointer is at {entry_ptr:#x}");
    unsafe {
        *(entry_ptr.as_mut_ptr::<u64>()) = entry_args_ptr as u64;
    }

    kernel_paging
        .unmap(Page::<Size4KiB>::containing_address(stack_top - 1))
        .expect("Failed to unmap stack top page")
        .1
        .flush();

    (VirtAddr::new(elf.entry_point()), entry_ptr, pages)
}
