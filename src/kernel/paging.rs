use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::FrameError, FrameAllocator, FrameDeallocator, OffsetPageTable, PageTable,
        PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use crate::stuff::memmap::SoosMemmap;

pub fn current_page_table() -> *mut PageTable {
    let (level_4_table_frame, _flags) = Cr3::read();
    let level_4_table = level_4_table_frame.start_address().as_u64() as *mut PageTable;
    level_4_table
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
}

const MAX_USABLE_FRAMES: usize = 1 << 24;

static mut FRAME_ALLOCATION_BITMAP: [bool; MAX_USABLE_FRAMES] = [false; MAX_USABLE_FRAMES];

static mut SOOS_FRAME_ALLOCATOR: Option<SoosFrameAllocator> = None;

#[derive(Debug)]
pub struct SoosFrameAllocator {
    memmap: SoosMemmap,
    skip: usize,
}

impl SoosFrameAllocator {
    pub fn get_or_init_with_current_pagetable(memmap: SoosMemmap) -> &'static mut Self {
        unsafe {
            SOOS_FRAME_ALLOCATOR.get_or_insert_with(|| {
                let allocator = Self { memmap, skip: 0 };

                let page_table = &*current_page_table();
                page_table.iter().for_each(|entry| match entry.frame() {
                    Ok(frame) => {
                        let i = frame.start_address().as_u64() as usize / 4096;
                        FRAME_ALLOCATION_BITMAP[i] = true;
                    }
                    Err(FrameError::HugeFrame) => {
                        panic!("Huge frame!");
                    }
                    Err(FrameError::FrameNotPresent) => {}
                });

                allocator
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
            .filter(|&f| {
                let i = f.start_address().as_u64() as usize / 4096;
                unsafe { FRAME_ALLOCATION_BITMAP[i] == false }
            })
            .next();

        if f.is_some() {
            let i = f.unwrap().start_address().as_u64() as usize / 4096;
            unsafe { FRAME_ALLOCATION_BITMAP[i] = true };
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
        FRAME_ALLOCATION_BITMAP[i] = false;
    }
}
