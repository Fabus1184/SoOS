use core::alloc::GlobalAlloc;

use limine::MemmapResponse;

static mut MEMMAP: [Option<MemmapEntry>; 64] = [Some(MemmapEntry {
    base: 0x1000,
    len: 0x10000,
    typ: MemmapEntryType::Usable,
}); 64];
static mut ALLOCATED: Option<*mut u8> = None;

pub struct SoosAllocator {}

impl SoosAllocator {
    pub fn load_limine_memmap(&mut self, memmap: &MemmapResponse) {
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

#[derive(Copy, Clone, PartialEq)]
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
        let ret = ALLOCATED.unwrap_or_else(|| {
            MEMMAP
                .iter()
                .flatten()
                .filter(|x| x.typ == MemmapEntryType::Usable)
                .reduce(|a, b| if a.len > b.len { a } else { b })
                .map(|x| x.base)
                .expect("No usable memory found!") as *mut u8
        });
        ALLOCATED = Some(ret.add(layout.size()));
        ret
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
pub static mut ALLOCATOR: SoosAllocator = SoosAllocator {};
