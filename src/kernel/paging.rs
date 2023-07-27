use x86_64::{
    registers::control::Cr3,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

use crate::stuff::memmap::SoosMemmap;

#[derive(Debug)]
pub struct SoosPaging {
    pub mapper: OffsetPageTable<'static>,
    pub frame_allocator: SoosFrameAllocator,
}

impl SoosPaging {
    pub fn new(phys_memory_offset: u64, frame_allocator: SoosFrameAllocator) -> Self {
        let (level_4_table_frame, _flags) = Cr3::read();
        let level_4_table =
            unsafe { &mut *(level_4_table_frame.start_address().as_u64() as *mut PageTable) };

        Self {
            mapper: unsafe {
                OffsetPageTable::new(level_4_table, VirtAddr::new(phys_memory_offset))
            },
            frame_allocator,
        }
    }
}

#[derive(Debug)]
pub struct SoosFrameAllocator {
    memmap: SoosMemmap,
    next: usize,
}

impl SoosFrameAllocator {
    pub fn new(memmap: SoosMemmap) -> Self {
        Self { memmap, next: 0 }
    }
}

unsafe impl FrameAllocator<Size4KiB> for SoosFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.memmap
            .iter_usable_addresses()
            .step_by(4096)
            .nth({
                let n = self.next;
                self.next += 1;
                n
            })
            .map(|x| PhysFrame::containing_address(PhysAddr::new(x)))
    }
}
