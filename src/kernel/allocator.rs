use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use super::paging::{SoosFrameAllocator, SoosPaging};

const HEAP_START: u64 = 0xFFFF_ACDC_ABBA_0000;
const HEAP_SIZE: u64 = 100 * 1024 * 1024; // 100 MiB

pub fn init_kernel_heap(
    paging: &mut SoosPaging,
    frame_allocator: &mut SoosFrameAllocator,
) -> Result<(), MapToError<Size4KiB>> {
    let heap_start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(HEAP_START));
    let heap_end_page =
        Page::<Size4KiB>::containing_address(VirtAddr::new(HEAP_START + HEAP_SIZE - 1));

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            paging
                .offset_page_table
                .map_to(page, frame, flags, frame_allocator)
        }?
        .flush();
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START as *mut u8, HEAP_SIZE as usize);
    }

    Ok(())
}

#[global_allocator]
pub static mut ALLOCATOR: LockedHeap = LockedHeap::empty();
