use alloc::borrow::ToOwned as _;
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{
        mapper::{MappedFrame, TranslateResult},
        FrameAllocator, FrameDeallocator, Mapper as _, OffsetPageTable, Page, PageTable,
        PageTableFlags, PhysFrame, Size2MiB, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};

use crate::stuff::memmap::SoosMemmap;

pub fn current_page_table() -> *mut PageTable {
    let (level_4_table_frame, _flags) = Cr3::read();
    level_4_table_frame.start_address().as_u64() as *mut PageTable
}

const MAX_USABLE_FRAMES: usize = 1 << 20;

/// The address where the kernel frame mapping starts.
/// This is used to map all frames into virtual memory
pub const KERNEL_FRAME_MAPPING_ADDRESS: u64 = 0x7776_0000_0000;

pub struct KernelPaging {
    pub page_table: OffsetPageTable<'static>,
    pub frame_allocator: KernelFrameAllocator,
}

pub struct KernelFrameAllocator {
    bitmap: &'static mut [bool; MAX_USABLE_FRAMES],
    memmap: SoosMemmap,
    skip: usize,
    allocated: usize,
}

unsafe impl FrameAllocator<Size4KiB> for KernelFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let result = self
            .memmap
            .iter_usable_frames()
            .enumerate()
            .skip(self.skip)
            .take_while(|(_, frame)| {
                frame.start_address().as_u64() as usize / 4096 < self.bitmap.len()
            })
            .find(|&(_, frame)| !self.bitmap[frame.start_address().as_u64() as usize / 4096]);

        let Some((index, frame)) = result else {
            self.skip = 0;
            log::warn!("No free frame found, resetting skip to 0");
            return self.allocate_frame();
        };

        self.bitmap[frame.start_address().as_u64() as usize / 4096] = true;
        self.skip = index;

        self.allocated += 1;

        assert!(
            self.allocated < 1000,
            "allocated more than 1000 frames, this is not expected!"
        );

        Some(frame)
    }
}

impl FrameDeallocator<Size4KiB> for KernelFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let i = frame.start_address().as_u64() as usize / 4096;
        if i < self.bitmap.len() {
            self.bitmap[i] = false;
        }
    }
}

impl KernelPaging {
    /// Creates a new [`KernelPaging`] instance, initializing the page table
    /// all frames are mapped at [`KERNEL_FRAME_MAPPING_ADDRESS`]
    pub fn make_kernel_paging(
        memmap: &SoosMemmap,
        old_page_table: &mut OffsetPageTable,
        keep_pages: impl Iterator<Item = Page>,
    ) -> Self {
        static mut PAGE_TABLE: PageTable = PageTable::new();

        let mut kernel_page_table = unsafe {
            OffsetPageTable::new(&mut PAGE_TABLE, VirtAddr::new(KERNEL_FRAME_MAPPING_ADDRESS))
        };

        let mut frame_allocator = KernelFrameAllocator {
            bitmap: {
                static mut BITMAP: [bool; MAX_USABLE_FRAMES] = [false; MAX_USABLE_FRAMES];
                unsafe { &mut BITMAP }
            },
            memmap: *memmap,
            skip: 0,
            allocated: 0,
        };

        // walk old page table and mark all frames as used
        for l4_index in 0..512 {
            let l4_entry = &old_page_table.level_4_table()[l4_index];
            if l4_entry.is_unused() {
                continue;
            }

            let l3_table = unsafe { &*((l4_entry.addr().as_u64()) as *const PageTable) };
            for l3_index in 0..512 {
                let l3_entry = &l3_table[l3_index];
                if l3_entry.is_unused() {
                    continue;
                }

                assert!(
                    !l3_entry.flags().contains(PageTableFlags::HUGE_PAGE),
                    "Huge pages are not supported in kernel paging: {:#x}",
                    l3_entry.addr().as_u64()
                );

                let l2_table = unsafe { &*((l3_entry.addr().as_u64()) as *const PageTable) };
                for l2_index in 0..512 {
                    let l2_entry = &l2_table[l2_index];
                    if l2_entry.is_unused() {
                        continue;
                    }

                    if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                        for frame_index in 0..512 {
                            let frame_address = l2_entry.addr().as_u64() + (frame_index * 0x1000);
                            let frame = PhysFrame::<Size2MiB>::containing_address(PhysAddr::new(
                                frame_address,
                            ));
                            if frame.start_address().as_u64() as usize / 4096
                                < frame_allocator.bitmap.len()
                            {
                                frame_allocator.bitmap
                                    [frame.start_address().as_u64() as usize / 4096] = true;
                            }
                        }
                    } else {
                        // 4KiB page
                        let l1_table =
                            unsafe { &*((l2_entry.addr().as_u64()) as *const PageTable) };
                        for l1_index in 0..512 {
                            let l1_entry = &l1_table[l1_index];
                            if l1_entry.is_unused() {
                                continue;
                            }

                            let frame = PhysFrame::<Size4KiB>::containing_address(l1_entry.addr());
                            frame_allocator.bitmap
                                [frame.start_address().as_u64() as usize / 4096] = true;
                        }
                    }
                }
            }
        }

        let count = memmap
            .iter_usable_frames()
            .take_while(|frame| {
                frame.start_address().as_u64() as usize / 4096 < frame_allocator.bitmap.len()
            })
            .filter(|frame| !frame_allocator.bitmap[frame.start_address().as_u64() as usize / 4096])
            .count();
        log::debug!("found {count} free frames in old page table");

        // map all frames
        let mut mapped = 0;
        for frame in memmap.iter_usable_frames() {
            if frame.start_address().as_u64() as usize / 4096 >= frame_allocator.bitmap.len() {
                break; // out of bounds, can never be allocated
            }

            if frame_allocator.bitmap[frame.start_address().as_u64() as usize / 4096] {
                continue; // already used
            }

            if mapped == 1000 {
                log::warn!("mapped 1000 frames, this is enough");
                break;
            }

            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            log::trace!(
                "mapping frame {mapped}/{count}, {:#x} at {:#x}",
                frame.start_address().as_u64(),
                KERNEL_FRAME_MAPPING_ADDRESS + frame.start_address().as_u64()
            );
            mapped += 1;

            unsafe {
                // map the frame to the old page table
                old_page_table
                    .map_to(
                        Page::<Size4KiB>::containing_address(VirtAddr::new(
                            KERNEL_FRAME_MAPPING_ADDRESS + frame.start_address().as_u64(),
                        )),
                        frame,
                        flags,
                        &mut frame_allocator,
                    )
                    .expect("Failed to map frame in kernel page table")
                    .flush();
            }
        }

        for page in keep_pages {
            match old_page_table.translate(page.start_address()) {
                TranslateResult::Mapped { frame, offset, .. } => {
                    if offset != 0 {
                        continue;
                    }

                    log::trace!(
                        "keeping page {:#x} mapped to frame {:#x} with offset {:#x}",
                        page.start_address().as_u64(),
                        frame.start_address().as_u64(),
                        offset
                    );

                    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

                    match frame {
                        MappedFrame::Size4KiB(phys_frame) => unsafe {
                            kernel_page_table
                                .map_to(page, phys_frame, flags, &mut frame_allocator)
                                .expect("Failed to map frame in kernel page table")
                                .flush();
                        },
                        MappedFrame::Size2MiB(phys_frame) => unsafe {
                            kernel_page_table
                                .map_to(
                                    Page::<Size2MiB>::containing_address(page.start_address()),
                                    phys_frame,
                                    flags,
                                    &mut frame_allocator,
                                )
                                .expect("Failed to map 2MiB frame in kernel page table")
                                .flush();
                        },
                        MappedFrame::Size1GiB(phys_frame) => panic!(
                            "1GiB pages are not supported in kernel paging: {:#x}",
                            phys_frame.start_address().as_u64()
                        ),
                    }
                }
                TranslateResult::NotMapped => {
                    log::trace!(
                        "keeping page {:#x} not mapped",
                        page.start_address().as_u64()
                    );
                    // do nothing, the page is not mapped
                }
                e @ TranslateResult::InvalidFrameAddress(_) => {
                    panic!("unexpected translation result: {:?}", e)
                }
            }
        }

        let mut self_ = Self {
            page_table: kernel_page_table,
            frame_allocator,
        };

        log::debug!(
            "kernel paging initialized with {} frames",
            self_.frame_allocator.bitmap.len()
        );

        self_.load();

        log::debug!("Kernel paging initialized");

        self_
    }

    pub fn load(&mut self) {
        let addr = core::ptr::from_ref(self.page_table.level_4_table()) as u64;
        let physical_address = PhysFrame::containing_address(
            match self.page_table.translate(VirtAddr::new(addr)) {
                TranslateResult::Mapped {
                    frame,
                    offset,
                    flags,
                } => {
                    log::debug!(
                        "translated kernel page table at {:#x} to {:#x} with offset {:#x} and flags {:?}",
                        addr,
                        frame.start_address().as_u64(),
                        offset,
                        flags
                    );

                    assert!(offset == 0, "Offset must be zero for kernel page table");

                    match frame {
                        MappedFrame::Size4KiB(phys_frame) => phys_frame.start_address(),
                        e => panic!("unexpected frame type of kernel page table: {:?}", e),
                    }
                }
                e => panic!("unexpected translation result: {:?}", e),
            },
        );

        log::debug!(
            "loading kernel page table at {:#x}",
            physical_address.start_address().as_u64()
        );

        unsafe {
            Cr3::write(physical_address, Cr3Flags::empty());
        }
    }

    pub fn make_userspace_paging(&mut self) -> UserspacePaging<'static> {
        let new_pagetable = self
            .frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!");

        let new_pagetable_ptr = (self.page_table.phys_offset().as_u64()
            + new_pagetable.start_address().as_u64())
            as *mut PageTable;

        self.page_table
            .level_4_table()
            .clone_into(unsafe { &mut *new_pagetable_ptr });

        let page_table =
            unsafe { OffsetPageTable::new(&mut *new_pagetable_ptr, self.page_table.phys_offset()) };

        UserspacePaging { page_table }
    }
}

pub struct UserspacePaging<'a> {
    pub page_table: OffsetPageTable<'a>,
}

impl UserspacePaging<'_> {
    pub fn load(&mut self) {
        let addr = core::ptr::from_ref(self.page_table.level_4_table()) as u64;
        let physical_address = self
            .page_table
            .translate_addr(VirtAddr::new(addr))
            .expect("Failed to translate address");

        log::trace!(
            "loading userspace page table at {:#x}",
            physical_address.as_u64(),
        );

        unsafe {
            Cr3::write(
                PhysFrame::containing_address(physical_address),
                Cr3Flags::empty(),
            );
        }
    }

    pub fn fork(
        &self,
        kernel_paging: &mut KernelPaging,
        extra_pages: &[(Page, PageTableFlags)],
    ) -> UserspacePaging<'static> {
        let new_pagetable = kernel_paging
            .frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!");

        let new_pagetable_ptr = (self.page_table.phys_offset().as_u64()
            + new_pagetable.start_address().as_u64())
            as *mut PageTable;

        self.page_table
            .level_4_table()
            .clone_into(unsafe { &mut *new_pagetable_ptr });

        let mut new_page_table =
            unsafe { OffsetPageTable::new(&mut *new_pagetable_ptr, self.page_table.phys_offset()) };

        for &(page, flags) in extra_pages {
            let new_frame = kernel_paging
                .frame_allocator
                .allocate_frame()
                .expect("Failed to allocate frame for cloned page table");

            log::debug!(
                "Cloning page {:#x} to new frame",
                page.start_address().as_u64(),
            );

            unsafe {
                new_page_table
                    .map_to(page, new_frame, flags, &mut kernel_paging.frame_allocator)
                    .expect("Failed to map frame in cloned page table")
                    .flush();
            };

            // find free address to map the old and new frames
            let virt_addr = VirtAddr::new(0x8888_0000_0000);

            let old_frame = self
                .page_table
                .translate_page(page)
                .expect("Failed to translate page")
                .start_address();

            // map old and new frames to temporary pages
            unsafe {
                kernel_paging
                    .page_table
                    .map_to(
                        Page::<Size4KiB>::containing_address(virt_addr),
                        PhysFrame::containing_address(old_frame),
                        PageTableFlags::PRESENT,
                        &mut kernel_paging.frame_allocator,
                    )
                    .expect("Failed to map frame in cloned page table")
                    .flush();

                kernel_paging
                    .page_table
                    .map_to(
                        Page::containing_address(virt_addr + 0x1000),
                        new_frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        &mut kernel_paging.frame_allocator,
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
                    .page_table
                    .unmap(Page::<Size4KiB>::containing_address(virt_addr))
                    .expect("Failed to unmap old frame in cloned page table");
                kernel_paging.frame_allocator.deallocate_frame(frame);
                flush.flush();

                let (frame, flush) = kernel_paging
                    .page_table
                    .unmap(Page::<Size4KiB>::containing_address(virt_addr + 0x1000))
                    .expect("Failed to unmap new frame in cloned page table");
                kernel_paging.frame_allocator.deallocate_frame(frame);
                flush.flush();
            }
        }

        UserspacePaging {
            page_table: new_page_table,
        }
    }
}
