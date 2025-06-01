use alloc::borrow::ToOwned as _;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::FrameError, FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page,
        PageTable, PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use crate::stuff::memmap::SoosMemmap;

pub fn current_page_table() -> *mut PageTable {
    let (level_4_table_frame, _flags) = Cr3::read();
    level_4_table_frame.start_address().as_u64() as *mut PageTable
}

#[derive(Debug)]
pub struct SoosPaging<'a> {
    pub offset_page_table: OffsetPageTable<'a>,
}

impl<'a> SoosPaging<'a> {
    pub fn offset_page_table(phys_memory_offset: u64, page_table: &'a mut PageTable) -> Self {
        let offset_page_table =
            unsafe { OffsetPageTable::new(page_table, VirtAddr::new(phys_memory_offset)) };

        Self { offset_page_table }
    }

    pub fn load(&mut self) {
        let flags = Cr3::read().1;
        unsafe {
            Cr3::write(
                PhysFrame::containing_address(PhysAddr::new(
                    self.offset_page_table.level_4_table() as *const _ as u64,
                )),
                flags,
            )
        };
    }

    pub fn fork(&self, pages: &[(Page, PageTableFlags)]) -> Self {
        (unsafe { &mut *crate::KERNEL_PAGING }).load();

        let frame_allocator = unsafe {
            SOOS_FRAME_ALLOCATOR
                .as_mut()
                .expect("Frame allocator not initialized!")
        };

        let new_pagetable = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!")
            .start_address()
            .as_u64() as *mut x86_64::structures::paging::PageTable;

        (unsafe { &*crate::KERNEL_PAGING })
            .offset_page_table
            .level_4_table()
            .clone_into(unsafe { &mut *new_pagetable });

        let mut offset_page_table = unsafe {
            OffsetPageTable::new(&mut *new_pagetable, self.offset_page_table.phys_offset())
        };

        for &(page, flags) in pages {
            let new_frame = frame_allocator
                .allocate_frame()
                .expect("Failed to allocate frame for cloned page table");

            log::debug!(
                "Cloning page {:#x} to new frame",
                page.start_address().as_u64(),
            );

            unsafe {
                offset_page_table
                    .map_to(page, new_frame, flags, frame_allocator)
                    .expect("Failed to map frame in cloned page table")
                    .flush();
            };

            // find free address to map the old and new frames
            let virt_addr = VirtAddr::new(0x77CC_BBBB_C000);

            let old_frame = self
                .offset_page_table
                .translate_page(page)
                .expect("Failed to translate page")
                .start_address();

            // map old and new frames to temporary pages
            unsafe {
                (*crate::KERNEL_PAGING)
                    .offset_page_table
                    .map_to(
                        Page::<Size4KiB>::containing_address(virt_addr),
                        PhysFrame::containing_address(old_frame),
                        PageTableFlags::PRESENT,
                        frame_allocator,
                    )
                    .expect("Failed to map frame in cloned page table")
                    .flush();

                (*crate::KERNEL_PAGING)
                    .offset_page_table
                    .map_to(
                        Page::containing_address(virt_addr + 0x1000),
                        new_frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        frame_allocator,
                    )
                    .expect("Failed to map frame in cloned page table")
                    .flush();
            };

            // copy the data
            for i in 0..0x1000 {
                let old_byte = unsafe { (virt_addr + i).as_ptr::<u8>().read_volatile() };
                unsafe {
                    (virt_addr + 0x1000 + i)
                        .as_mut_ptr::<u8>()
                        .write_volatile(old_byte)
                };
            }

            // unmap the temporary pages
            unsafe {
                let (frame, flush) = (*crate::KERNEL_PAGING)
                    .offset_page_table
                    .unmap(Page::<Size4KiB>::containing_address(virt_addr))
                    .expect("Failed to unmap old frame in cloned page table");
                frame_allocator.deallocate_frame(frame);
                flush.flush();

                let (frame, flush) = (*crate::KERNEL_PAGING)
                    .offset_page_table
                    .unmap(Page::<Size4KiB>::containing_address(virt_addr + 0x1000))
                    .expect("Failed to unmap new frame in cloned page table");
                frame_allocator.deallocate_frame(frame);
                flush.flush();
            }
        }

        Self { offset_page_table }
    }
}

const MAX_USABLE_FRAMES: usize = 1 << 24;

pub static mut SOOS_FRAME_ALLOCATOR: Option<SoosFrameAllocator> = None;

#[derive(Debug)]
pub struct SoosFrameAllocator {
    bitmap: &'static mut [bool; MAX_USABLE_FRAMES],
    memmap: SoosMemmap,
    skip: usize,
}

impl SoosFrameAllocator {
    pub fn init_with_current_pagetable(memmap: SoosMemmap) -> &'static mut Self {
        static mut BITMAP: [bool; MAX_USABLE_FRAMES] = [false; MAX_USABLE_FRAMES];

        unsafe {
            SOOS_FRAME_ALLOCATOR.get_or_insert_with(|| {
                let mut self_ = Self {
                    memmap,
                    skip: 0,
                    bitmap: &mut BITMAP,
                };

                let page_table = &*current_page_table();
                page_table.iter().for_each(|entry| match entry.frame() {
                    Ok(frame) => {
                        let i = frame.start_address().as_u64() as usize / 4096;
                        self_.bitmap[i] = true;
                    }
                    Err(FrameError::HugeFrame) => {
                        panic!("Huge frame!");
                    }
                    Err(FrameError::FrameNotPresent) => {}
                });

                self_
            })
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for SoosFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let f = self
            .memmap
            .iter_usable_addresses()
            .step_by(4096)
            .skip(self.skip)
            .map(|a| PhysFrame::containing_address(PhysAddr::new(a)))
            .find(|&f| {
                let i = f.start_address().as_u64() as usize / 4096;
                !self.bitmap[i]
            });

        if f.is_some() {
            let i = f.unwrap().start_address().as_u64() as usize / 4096;
            self.bitmap[i] = true;
            self.skip = i;
            f
        } else {
            self.skip = 0;
            self.allocate_frame()
        }
    }
}

impl FrameDeallocator<Size4KiB> for SoosFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let i = frame.start_address().as_u64() as usize / 4096;
        self.bitmap[i] = false;
    }
}
