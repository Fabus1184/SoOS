use alloc::borrow::ToOwned as _;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, FrameDeallocator, MappedPageTable, Mapper, OffsetPageTable, Page,
        PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate,
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
    pub offset_page_table: MappedPageTable<'a>,
}

impl<'a> SoosPaging<'a> {
    pub fn offset_page_table(phys_memory_offset: u64, page_table: &'a mut PageTable) -> Self {
        let offset_page_table =
            unsafe { OffsetPageTable::new(page_table, VirtAddr::new(phys_memory_offset)) };

        Self { offset_page_table }
    }

    pub fn load(&mut self) {
        let flags = Cr3::read().1;
        let addr = core::ptr::from_ref(self.offset_page_table.level_4_table()) as u64;
        let physical_address = self
            .offset_page_table
            .translate_addr(VirtAddr::new(addr))
            .expect("Failed to translate address");

        log::debug!(
            "Loading page table at {:#x} with flags",
            physical_address.as_u64(),
        );

        unsafe {
            Cr3::write(PhysFrame::containing_address(physical_address), flags);
        }
    }

    pub fn fork(&self, pages: &[(Page, PageTableFlags)]) -> Self {
        let mut kernel_paging = crate::KERNEL_PAGING.lock();
        kernel_paging.load();

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

        kernel_paging
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
                kernel_paging
                    .offset_page_table
                    .map_to(
                        Page::<Size4KiB>::containing_address(virt_addr),
                        PhysFrame::containing_address(old_frame),
                        PageTableFlags::PRESENT,
                        frame_allocator,
                    )
                    .expect("Failed to map frame in cloned page table")
                    .flush();

                kernel_paging
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
                        .write_volatile(old_byte);
                }
            }

            // unmap the temporary pages
            unsafe {
                let (frame, flush) = kernel_paging
                    .offset_page_table
                    .unmap(Page::<Size4KiB>::containing_address(virt_addr))
                    .expect("Failed to unmap old frame in cloned page table");
                frame_allocator.deallocate_frame(frame);
                flush.flush();

                let (frame, flush) = kernel_paging
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

const MAX_USABLE_FRAMES: usize = 1 << 10;

pub static mut SOOS_FRAME_ALLOCATOR: Option<SoosFrameAllocator> = None;

#[derive(Debug)]
pub struct SoosFrameAllocator {
    bitmap: &'static mut [bool; MAX_USABLE_FRAMES],
    memmap: SoosMemmap,
}

impl SoosFrameAllocator {
    pub fn init_empty(memmap: &SoosMemmap) {
        static mut BITMAP: [bool; MAX_USABLE_FRAMES] = [false; MAX_USABLE_FRAMES];

        unsafe {
            SOOS_FRAME_ALLOCATOR.get_or_insert(Self {
                memmap: *memmap,
                bitmap: &mut BITMAP,
            })
        };
    }
}

unsafe impl FrameAllocator<Size4KiB> for SoosFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let address = self
            .memmap
            .iter_usable_addresses()
            .step_by(4096)
            .take_while(|&addr| addr < 0x1_000_000)
            .find(|&addr| (!self.bitmap[addr as usize / 4096]))
            .expect("No free frames available");

        self.bitmap[address as usize / 4096] = true;

        log::debug!("allocated frame at {address:#0x}");

        Some(PhysFrame::containing_address(PhysAddr::new(address)))
    }
}

impl FrameDeallocator<Size4KiB> for SoosFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let i = frame.start_address().as_u64() as usize / 4096;
        if i < self.bitmap.len() {
            self.bitmap[i] = false;
        }
    }
}
