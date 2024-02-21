use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{mapper::MapToError, Size4KiB};

const HEAP_SIZE: u64 = 100 * 1024 * 1024; // 100 MiB

pub fn init_kernel_heap(heap_start: u64) -> Result<(), MapToError<Size4KiB>> {
    unsafe {
        ALLOCATOR
            .lock()
            .init(heap_start as *mut u8, HEAP_SIZE as usize);
    }

    Ok(())
}

#[global_allocator]
pub static mut ALLOCATOR: LockedHeap = LockedHeap::empty();
