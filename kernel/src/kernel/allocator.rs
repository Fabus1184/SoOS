use linked_list_allocator::LockedHeap;

pub fn init_kernel_heap(heap_start: *mut u8, length: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start.cast::<u8>(), length);
    }
}

#[global_allocator]
pub static mut ALLOCATOR: LockedHeap = LockedHeap::empty();
