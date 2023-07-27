use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use super::paging::SoosPaging;

const HEAP_START: u64 = 0x_1234_5678_0000;
const HEAP_SIZE: u64 = 100 * 1024 * 1024; // 100 MiB

pub fn init_kernel_heap(paging: &mut SoosPaging) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::<Size4KiB>::containing_address(heap_start);
        let heap_end_page = Page::<Size4KiB>::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = paging
            .frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            paging
                .mapper
                .map_to(page, frame, flags, &mut paging.frame_allocator)
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
