use elf_rs::{Elf, ElfFile, ProgramHeaderFlags, ProgramType};
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
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
    paging: &mut SoosPaging,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    bytes: &[u8],
    code_address: VirtAddr,
) -> VirtAddr {
    let elf = Elf::from_bytes(bytes).expect("Failed to parse ELF!");

    elf.program_header_iter().for_each(|ph| match ph.ph_type() {
        ProgramType::LOAD => {
            let start_page = Page::containing_address(code_address + ph.vaddr());
            let end_page = Page::containing_address(code_address + ph.vaddr() + ph.memsz());

            let mut flags = PageTableFlags::PRESENT
                | PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::WRITABLE;
            if !ph.flags().contains(ProgramHeaderFlags::EXECUTE) {
                flags |= PageTableFlags::NO_EXECUTE;
            }

            for page in Page::range_inclusive(start_page, end_page) {
                let frame = frame_allocator
                    .allocate_frame()
                    .expect("Failed to allocate frame!");

                unsafe {
                    paging
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
        _ => {}
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
        _ => {
            /* printk!(
                "Ignoring section header: {:?}",
                sh.section_name()
                    .expect("Failed to get name!")
                    .into_iter()
                    .map(|&c| c as char)
                    .collect::<alloc::string::String>()
            ); */
        }
    });

    elf.program_header_iter().for_each(|ph| match ph.ph_type() {
        ProgramType::LOAD => {
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
                    paging
                        .offset_page_table
                        .update_flags(page, flags)
                        .expect("Failed to map page!")
                        .flush()
                };
            }
        }
        _ => {}
    });

    code_address + elf.entry_point()
}
