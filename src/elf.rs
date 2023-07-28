use elf_rs::{Elf, ElfFile, ProgramType};
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
    VirtAddr,
};

use crate::kernel::paging::SoosPaging;

const USERLAND_STACK_ADDR: u64 = 0xFFFF_BBBB_0000_0000;
const USERLAND_CODE_ADDR: u64 = 0xFFFF_FFAA_0000_0000;

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
    kernel_paging: &mut SoosPaging,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> (u64, u64) {
    const STACK_PAGES: u64 = 100;

    for i in 0..STACK_PAGES {
        let stack_page = Page::containing_address(VirtAddr::new(USERLAND_STACK_ADDR + (i * 4096)));
        let frame = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!");
        unsafe {
            kernel_paging
                .offset_page_table
                .map_to(
                    stack_page,
                    frame,
                    PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE
                        | PageTableFlags::NO_EXECUTE,
                    frame_allocator,
                )
                .expect("Failed to map page!")
                .flush()
        };
    }

    const ELF_BYTES: &'static [u8] = include_bytes!("../userspace/main.elf");

    let elf = Elf::from_bytes(ELF_BYTES).expect("Failed to parse ELF!");

    elf.program_header_iter().for_each(|ph| match ph.ph_type() {
        ProgramType::LOAD => {
            let start_page =
                Page::containing_address(VirtAddr::new(USERLAND_CODE_ADDR + ph.vaddr()));
            let end_page = Page::containing_address(VirtAddr::new(
                USERLAND_CODE_ADDR + ph.vaddr() + ph.memsz(),
            ));

            for page in Page::range_inclusive(start_page, end_page) {
                let frame = frame_allocator
                    .allocate_frame()
                    .expect("Failed to allocate frame!");

                unsafe {
                    kernel_paging
                        .offset_page_table
                        .map_to(
                            page,
                            frame,
                            PageTableFlags::PRESENT
                                | PageTableFlags::WRITABLE
                                | PageTableFlags::USER_ACCESSIBLE,
                            frame_allocator,
                        )
                        .expect("Failed to map page!")
                        .flush()
                };
            }

            unsafe {
                core::ptr::copy(
                    ELF_BYTES.as_ptr().add(ph.offset() as usize),
                    (USERLAND_CODE_ADDR + ph.vaddr()) as *mut u8,
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

                unsafe { *((USERLAND_CODE_ADDR + entry.offset) as *mut u64) += USERLAND_CODE_ADDR };
            }
        }
        _ => {
            /* printk!(
                "Ignoring section header: {:?}\n",
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
            let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(
                USERLAND_CODE_ADDR + ph.vaddr(),
            ));
            let end_page = Page::containing_address(VirtAddr::new(
                USERLAND_CODE_ADDR + ph.vaddr() + ph.memsz(),
            ));

            for page in Page::range_inclusive(start_page, end_page) {
                unsafe {
                    kernel_paging
                        .offset_page_table
                        .update_flags(
                            page,
                            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
                        )
                        .expect("Failed to map page!")
                        .flush()
                };
            }
        }
        _ => {}
    });

    (
        USERLAND_STACK_ADDR + (STACK_PAGES * 4096),
        USERLAND_CODE_ADDR + elf.entry_point(),
    )
}
