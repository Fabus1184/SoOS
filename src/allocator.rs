use core::alloc::GlobalAlloc;

use limine::LimineMemmapResponse;

static mut MEMMAP: [Option<MemmapEntry>; 64] = [None; 64];
static mut ALLOCATED: Option<*mut u8> = None;

pub struct SoosAllocator {}

impl SoosAllocator {
    pub fn load_limine_memmap(&mut self, memmap: &LimineMemmapResponse) {
        unsafe {
            for (e, i) in MEMMAP.iter_mut().zip(0..memmap.entry_count) {
                let ptr = memmap.entries.as_ptr().wrapping_offset(i as isize).read();
                *e = Some(MemmapEntry {
                    base: ptr.base,
                    len: ptr.len,
                    typ: core::mem::transmute(ptr.typ),
                });
            }
        };
    }
}

#[derive(Copy, Clone)]
struct AllocationEntry {
    base: u64,
    len: u64,
}

#[derive(Copy, Clone)]
struct MemmapEntry {
    base: u64,
    len: u64,
    typ: MemmapEntryType,
}

#[derive(Copy, Clone)]
#[repr(u32)]
enum MemmapEntryType {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    BadMemory = 4,
    BootloaderReclaimable = 5,
    KernelAndModules = 6,
    Framebuffer = 7,
}

unsafe impl GlobalAlloc for SoosAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if let Some(allocated) = ALLOCATED {
            ALLOCATED = Some(allocated.add(layout.size()));
            allocated
        } else {
            ALLOCATED = Some((MEMMAP[0].unwrap().base as usize + layout.size()) as *mut u8);
            MEMMAP[0].unwrap().base as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
pub static mut ALLOCATOR: SoosAllocator = SoosAllocator {};
