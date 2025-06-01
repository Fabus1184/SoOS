use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{mapper::MapToError, Size4KiB};

pub fn init_kernel_heap(heap_start: u64, length: usize) -> Result<(), MapToError<Size4KiB>> {
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, length);
    }

    Ok(())
}

#[global_allocator]
pub static mut ALLOCATOR: LockedHeap = LockedHeap::empty();
