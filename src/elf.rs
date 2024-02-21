use elf_rs::{Elf, ElfFile, ProgramHeaderFlags, ProgramType};
use log::{info, warn};
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageSize, PageTableFlags, Size4KiB},
    VirtAddr,
};

use crate::kernel::paging::SoosPaging;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Elf64Rela {
    offset: u64,
    info: u64,
    addend: i64,
}

impl Elf64Rela {
    pub unsafe fn from_bytes(bytes: [u8; core::mem::size_of::<Elf64Rela>()]) -> Self {
        core::ptr::read(bytes.as_ptr() as *const Self)
    }
}

pub fn load(
    process_paging: &mut SoosPaging,
    kernel_paging: &mut SoosPaging,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    bytes: &[u8],
    code_address: VirtAddr,
) -> (VirtAddr, VirtAddr) {
    unsafe { process_paging.load() };

    let elf = Elf::from_bytes(bytes).expect("Failed to parse ELF!");

    elf.program_header_iter().for_each(|ph| match ph.ph_type() {
        ProgramType::LOAD => {
            let start_page = Page::containing_address(code_address + ph.vaddr());
            let end_page = Page::containing_address(code_address + ph.vaddr() + ph.memsz());

            info!(
                "Mapping pages [{:#0x} - {:#0x}] to [{:#0x}]",
                start_page.start_address(),
                end_page.start_address(),
                code_address + ph.vaddr()
            );

            let flags = PageTableFlags::PRESENT
                | PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::WRITABLE
                | PageTableFlags::NO_EXECUTE;

            for page in Page::range_inclusive(start_page, end_page) {
                let frame = frame_allocator
                    .allocate_frame()
                    .expect("Failed to allocate frame!");

                unsafe {
                    process_paging
                        .offset_page_table
                        .map_to(page, frame, flags, frame_allocator)
                        .expect("Failed to map page!")
                        .flush()
                };
            }

            unsafe {
                core::ptr::copy::<u8>(
                    bytes.as_ptr().add(ph.offset() as usize),
                    (code_address + ph.vaddr()).as_mut_ptr(),
                    ph.filesz() as usize,
                )
            };
        }
        s => warn!("Skipping program header: {:?}", s),
    });

    elf.section_header_iter().for_each(|sh| match sh.sh_type() {
        elf_rs::SectionType::SHT_REL => {
            todo!()
        }
        elf_rs::SectionType::SHT_RELA => {
            let entries = sh.size() / sh.entsize();

            for i in 0..entries {
                let entry = sh
                    .content()
                    .expect("Failed to get section content!")
                    .chunks(sh.entsize() as usize)
                    .nth(i as usize)
                    .expect("Failed to get entry!");
                let entry = unsafe {
                    Elf64Rela::from_bytes(entry.try_into().expect("Failed to get entry!"))
                };

                unsafe {
                    *((code_address + entry.offset).as_mut_ptr::<u64>()) += code_address.as_u64()
                };
            }
        }
        _ => {}
    });

    elf.program_header_iter().for_each(|ph| {
        if let ProgramType::LOAD = ph.ph_type() {
            let start_page = Page::<Size4KiB>::containing_address(code_address + ph.vaddr());
            let end_page = Page::containing_address(code_address + ph.vaddr() + ph.memsz());

            let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
            if ph.flags().contains(ProgramHeaderFlags::WRITE) {
                flags |= PageTableFlags::WRITABLE;
            }
            if !ph.flags().contains(ProgramHeaderFlags::EXECUTE) {
                flags |= PageTableFlags::NO_EXECUTE;
            }

            for page in Page::range_inclusive(start_page, end_page) {
                unsafe {
                    process_paging
                        .offset_page_table
                        .update_flags(page, flags)
                        .expect("Failed to map page!")
                        .flush()
                };
            }
        }
    });

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
            process_paging
                .offset_page_table
                .map_to(page, frame, flags, frame_allocator)
                .expect("Failed to map page!")
                .flush()
        };
    }
    let stack_pointer = stack_address + stack_size_pages * Size4KiB::SIZE - 0x100_u64;

    unsafe { kernel_paging.load() };

    (code_address + elf.entry_point(), stack_pointer)
}
