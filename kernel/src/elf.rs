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

pub fn load(
    process_paging: &mut UserspacePaging,
    kernel_paging: &mut KernelPaging,
    bytes: &[u8],
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
                .frame_allocator
                .allocate_frame()
                .expect("Failed to allocate frame!");

            unsafe {
                process_paging
                    .page_table
                    .map_to(
                        page,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        &mut kernel_paging.frame_allocator,
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
                    .map_to(page, frame, flags, &mut kernel_paging.frame_allocator)
                    .expect("Failed to remap page!")
                    .flush();
            }
        }
    }

    // create stack
    let stack_size_pages = 10;
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
            .frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!");

        log::debug!("Mapping stack page {page:#?} to frame {frame:?}");

        unsafe {
            process_paging
                .page_table
                .map_to(page, frame, flags, &mut kernel_paging.frame_allocator)
                .expect("Failed to map page!")
                .flush();
        }

        pages.push(MappedPage {
            name: "stack",
            page,
            flags,
        });
    }

    let stack_pointer = stack_address + stack_size_pages * Size4KiB::SIZE;

    (VirtAddr::new(elf.entry_point()), stack_pointer, pages)
}
