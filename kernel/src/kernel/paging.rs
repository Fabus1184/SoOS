use alloc::{borrow::ToOwned as _, string::ParseError};
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{
        mapper::{MappedFrame, TranslateResult},
        FrameAllocator, FrameDeallocator, Mapper as _, OffsetPageTable, Page, PageSize, PageTable,
        PageTableFlags, PhysFrame, Size1GiB, Size2MiB, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};

use crate::{
    process::MappedPage,
    stuff::memmap::{MemmapEntryType, SoosMemmap},
};

pub fn current_page_table() -> *mut PageTable {
    let (level_4_table_frame, _flags) = Cr3::read();
    level_4_table_frame.start_address().as_u64() as *mut PageTable
}

/// The address where the kernel frame mapping starts.
/// This is used to map all frames into virtual memory
pub const KERNEL_FRAME_MAPPING_ADDRESS: u64 = 0x7776_0000_0000;

pub struct KernelPaging {
    pub page_table: OffsetPageTable<'static>,
    pub frame_allocator: KernelFrameAllocator,
}

pub struct KernelFrameAllocator {
    frame_map: &'static mut [(Option<u64>, [bool; 1 << 20]); 8],
    pub memmap: SoosMemmap,
    skip: usize,
    allocated: usize,
}

impl KernelFrameAllocator {
    pub fn init(memmap: &SoosMemmap) -> Self {
        static mut FRAME_MAP: [(Option<u64>, [bool; 1 << 20]); 8] = [(None, [false; 1 << 20]); 8];
        let frame_map = unsafe { &mut FRAME_MAP };

        let mut i = 0;
        for entry in memmap.iter() {
            if entry.type_ != MemmapEntryType::Usable
                && entry.type_ != MemmapEntryType::KernelAndModules
            {
                continue;
            }

            let start_address = entry.base;
            let end_address = entry.base + entry.len;

            // Calculate the number of frames in this range
            let num_frames = ((end_address - start_address) / 4096) as usize;

            if num_frames == 0 {
                continue;
            }

            frame_map[i].0 = Some(start_address);
            i += 1;
        }

        Self {
            frame_map,
            memmap: *memmap,
            skip: 0,
            allocated: 0,
        }
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let used = self
            .frame_map
            .iter()
            .map(|(_, bitmap)| bitmap.iter().filter(|&&b| b).count())
            .sum();
        let total = self
            .frame_map
            .iter()
            .map(|(_, bitmap)| bitmap.len())
            .sum::<usize>();
        (self.allocated, used, total)
    }

    fn mark_frame<P: PageSize>(&mut self, frame: PhysFrame<P>, used: bool) {
        for (start_address, bitmap) in &mut *self.frame_map {
            let Some(start_address) = start_address else {
                continue;
            };

            if frame.start_address().as_u64() >= *start_address
                && frame.start_address().as_u64() < *start_address + (bitmap.len() as u64 * 4096)
            {
                for i in 0..frame.size() / 4096 {
                    let index = (frame.start_address().as_u64() - *start_address) as usize / 4096
                        + i as usize;
                    assert!(
                        index < bitmap.len(),
                        "frame {:#x} is out of bounds of the bitmap",
                        frame.start_address().as_u64()
                    );

                    bitmap[index] = used;
                }

                return;
            }
        }
    }

    fn is_used(&self, frame: PhysFrame<Size4KiB>) -> bool {
        for (start_address, bitmap) in &*self.frame_map {
            let Some(start_address) = start_address else {
                continue;
            };

            if frame.start_address().as_u64() >= *start_address
                && frame.start_address().as_u64() < *start_address + (bitmap.len() as u64 * 4096)
            {
                let index = (frame.start_address().as_u64() - *start_address) as usize / 4096;
                assert!(
                    index < bitmap.len(),
                    "frame {:#x} is out of bounds of the bitmap",
                    frame.start_address().as_u64()
                );
                return bitmap[index];
            }
        }

        for (start_address, bitmap) in &*self.frame_map {
            if start_address.is_none() {
                continue;
            }

            log::warn!(
                "frame {:#x} is not in {:#x}, size {:#x}",
                frame.start_address().as_u64(),
                start_address.unwrap(),
                bitmap.len() * 4096
            );
        }

        panic!(
            "frame {:#x} is not in the frame map",
            frame.start_address().as_u64()
        );
    }

    pub fn init_with_page_table(&mut self, table: &OffsetPageTable) {
        // walk old page table and mark all frames as used
        for l4_index in 0..512 {
            let l4_entry = &table.level_4_table()[l4_index];
            if l4_entry.is_unused() {
                continue;
            }

            let l3_table = unsafe {
                &*((l4_entry.addr().as_u64() + table.phys_offset().as_u64()) as *const PageTable)
            };

            for l3_index in 0..512 {
                let l3_entry = &l3_table[l3_index];
                if l3_entry.is_unused() {
                    continue;
                }

                if l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                    // 1GiB page
                    for frame_index in 0..512 * 512 {
                        let frame_address = l3_entry.addr().as_u64() + (frame_index * 0x1000);
                        let frame =
                            PhysFrame::<Size1GiB>::containing_address(PhysAddr::new(frame_address));
                        if frame.start_address().as_u64() as usize / 4096 < self.frame_map.len() {
                            self.mark_frame(frame, true);
                        }
                    }
                    continue;
                }

                let l2_table = unsafe {
                    &*((l3_entry.addr().as_u64() + table.phys_offset().as_u64())
                        as *const PageTable)
                };
                for l2_index in 0..512 {
                    let l2_entry = &l2_table[l2_index];
                    if l2_entry.is_unused() {
                        continue;
                    }

                    if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                        // 2MiB page
                        for frame_index in 0..512 {
                            let frame_address = l2_entry.addr().as_u64() + (frame_index * 0x1000);
                            let frame = PhysFrame::<Size2MiB>::containing_address(PhysAddr::new(
                                frame_address,
                            ));
                            if frame.start_address().as_u64() as usize / 4096 < self.frame_map.len()
                            {
                                self.mark_frame(frame, true);
                            }
                        }
                    } else {
                        // 4KiB page
                        let l1_table = unsafe {
                            &*((l2_entry.addr().as_u64() + table.phys_offset().as_u64())
                                as *const PageTable)
                        };
                        for l1_index in 0..512 {
                            let l1_entry = &l1_table[l1_index];
                            if l1_entry.is_unused() {
                                continue;
                            }

                            let frame = PhysFrame::<Size4KiB>::containing_address(l1_entry.addr());
                            self.mark_frame(frame, true);
                        }
                    }
                }
            }
        }

        let (allocated, used, total) = self.stats();
        log::debug!(
            "initialized frame allocator with {allocated} allocated frames, {used} used, {total} total"
        );
    }
}

unsafe impl FrameAllocator<Size4KiB> for KernelFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let Some((skip, frame)) = self
            .memmap
            .iter_usable_frames()
            .skip(self.skip)
            .take_while(|frame| frame.start_address().as_u64() < 0x4_0000_0000)
            .enumerate()
            .find(|&(_, frame)| !self.is_used(frame))
        else {
            self.skip = 0;
            log::warn!("No free frame found, resetting skip to 0");
            return self.allocate_frame();
        };

        self.mark_frame(frame, true);
        self.skip = skip + 1;

        self.allocated += 1;

        Some(frame)
    }
}

impl FrameDeallocator<Size4KiB> for KernelFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.mark_frame(frame, false);

        self.allocated -= 1;
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

        let mut frame_allocator = KernelFrameAllocator::init(memmap);

        // initialize the frame allocator with the old page table
        frame_allocator.init_with_page_table(old_page_table);

        let mut kernel_page_table =
            unsafe { OffsetPageTable::new(&mut PAGE_TABLE, old_page_table.phys_offset()) };

        // map all frames in the new page table
        for entry in memmap.iter() {
            if entry.type_ != MemmapEntryType::Usable
                && entry.type_ != MemmapEntryType::KernelAndModules
            {
                continue;
            }

            log::debug!(
                "mapping memory entry: base {:#x}, len {:#x}, type {:?}",
                entry.base,
                entry.len,
                entry.type_
            );

            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            let mut pages = (0, 0, 0);

            let mut address = entry.base;

            while address != entry.base + entry.len {
                let remaining_len = entry.base + entry.len - address;

                if address % Size1GiB::SIZE == 0 && remaining_len >= Size1GiB::SIZE {
                    let frame = PhysFrame::<Size1GiB>::containing_address(PhysAddr::new(
                        entry.base + (entry.len - remaining_len),
                    ));

                    unsafe {
                        // map the frame to the old page table
                        kernel_page_table
                            .map_to(
                                Page::<Size1GiB>::containing_address(VirtAddr::new(
                                    KERNEL_FRAME_MAPPING_ADDRESS + frame.start_address().as_u64(),
                                )),
                                frame,
                                flags,
                                &mut frame_allocator,
                            )
                            .expect("failed to map frame in kernel page table")
                            .flush();
                    }

                    address += Size1GiB::SIZE; // 1GiB
                    pages.0 += 1;
                } else if address % Size2MiB::SIZE == 0 && remaining_len >= Size2MiB::SIZE {
                    // map 1GiB page
                    let frame = PhysFrame::<Size2MiB>::containing_address(PhysAddr::new(
                        entry.base + (entry.len - remaining_len),
                    ));

                    unsafe {
                        // map the frame to the old page table
                        kernel_page_table
                            .map_to(
                                Page::<Size2MiB>::containing_address(VirtAddr::new(
                                    KERNEL_FRAME_MAPPING_ADDRESS + frame.start_address().as_u64(),
                                )),
                                frame,
                                flags,
                                &mut frame_allocator,
                            )
                            .expect("failed to map frame in kernel page table")
                            .flush();
                    }

                    address += Size2MiB::SIZE; // 2MiB
                    pages.1 += 1;
                } else if address % Size4KiB::SIZE == 0 && remaining_len >= Size4KiB::SIZE {
                    // map 4KiB page
                    let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(
                        entry.base + (entry.len - remaining_len),
                    ));

                    unsafe {
                        // map the frame to the old page table
                        kernel_page_table
                            .map_to(
                                Page::<Size4KiB>::containing_address(VirtAddr::new(
                                    KERNEL_FRAME_MAPPING_ADDRESS + frame.start_address().as_u64(),
                                )),
                                frame,
                                flags,
                                &mut frame_allocator,
                            )
                            .expect("failed to map frame in kernel page table")
                            .flush();
                    }

                    address += 0x1000; // 4KiB
                    pages.2 += 1;
                } else {
                    panic!(
                        "Unexpected memory entry size: base {:#x}, len {:#x}, type {:?}",
                        entry.base, entry.len, entry.type_
                    );
                }
            }

            log::debug!(
                "mapped memory entry using {} 1GiB, {} 2MiB, {} 4KiB pages",
                pages.0,
                pages.1,
                pages.2
            );
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

        log::debug!("kernel paging initialized, switching to new page table");

        self_.load();
        self_.page_table = unsafe {
            OffsetPageTable::new(&mut PAGE_TABLE, VirtAddr::new(KERNEL_FRAME_MAPPING_ADDRESS))
        };

        log::debug!("kernel paging loaded");

        // re-initialize frame allocator with the new page table
        self_
            .frame_allocator
            .init_with_page_table(&self_.page_table);

        log::debug!("kernel paging initialized");

        self_
    }

    pub fn load(&mut self) {
        let addr = core::ptr::from_ref(self.page_table.level_4_table()) as u64;
        let physical_address =
            PhysFrame::containing_address(match self.page_table.translate(VirtAddr::new(addr)) {
                TranslateResult::Mapped { frame, offset, .. } => {
                    assert!(offset == 0, "Offset must be zero for kernel page table");

                    match frame {
                        MappedFrame::Size4KiB(phys_frame) => phys_frame.start_address(),
                        e => panic!("unexpected frame type of kernel page table: {:?}", e),
                    }
                }
                e => panic!("unexpected translation result: {:?}", e),
            });

        unsafe {
            Cr3::write(physical_address, Cr3Flags::empty());
        }
    }

    pub fn make_userspace_paging(&mut self) -> UserspacePaging<'static> {
        let new_pagetable = self
            .frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!");

        log::debug!(
            "creating new userspace page table at {:#x}",
            new_pagetable.start_address().as_u64()
        );

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

impl Drop for UserspacePaging<'_> {
    fn drop(&mut self) {
        let mut kernel_paging = crate::kernel_paging();

        unsafe {
            kernel_paging
                .frame_allocator
                .deallocate_frame(PhysFrame::containing_address(PhysAddr::new(
                    core::ptr::from_ref::<PageTable>(self.page_table.level_4_table()) as u64
                        - self.page_table.phys_offset().as_u64(),
                )));
        }
    }
}

impl UserspacePaging<'_> {
    pub fn load(&self) {
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
        pages: &[MappedPage],
    ) -> UserspacePaging<'static> {
        kernel_paging.load();

        let new_pagetable = kernel_paging
            .frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!");

        let new_pagetable_ptr = (self.page_table.phys_offset().as_u64()
            + new_pagetable.start_address().as_u64())
            as *mut PageTable;

        let mut new_page_table =
            unsafe { OffsetPageTable::new(&mut *new_pagetable_ptr, self.page_table.phys_offset()) };

        kernel_paging
            .page_table
            .level_4_table()
            .clone_into(new_page_table.level_4_table_mut());

        for &MappedPage { page, flags, .. } in pages {
            let new_frame = kernel_paging
                .frame_allocator
                .allocate_frame()
                .expect("Failed to allocate frame for cloned page table");

            log::trace!(
                "cloning page {:#x} to new frame {:#x}",
                page.start_address().as_u64(),
                new_frame.start_address().as_u64()
            );

            unsafe {
                new_page_table
                    .map_to(page, new_frame, flags, &mut kernel_paging.frame_allocator)
                    .expect("Failed to map frame in cloned page table")
                    .flush();
            };

            // find free address to map the old and new frames
            let temp_addr_src = VirtAddr::new_truncate(0x6666_0000_0000);
            let temp_addr_dst = temp_addr_src + 0x1000;

            let old_frame = self
                .page_table
                .translate_page(page)
                .expect("Failed to translate page")
                .start_address();

            // map old and new frames to temporary addresses for copying
            unsafe {
                kernel_paging
                    .page_table
                    .map_to(
                        Page::<Size4KiB>::containing_address(temp_addr_src),
                        PhysFrame::containing_address(old_frame),
                        PageTableFlags::PRESENT,
                        &mut kernel_paging.frame_allocator,
                    )
                    .expect("Failed to map frame in cloned page table")
                    .flush();

                kernel_paging
                    .page_table
                    .map_to(
                        Page::containing_address(temp_addr_dst),
                        new_frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        &mut kernel_paging.frame_allocator,
                    )
                    .expect("Failed to map frame in cloned page table")
                    .flush();
            };

            unsafe {
                for i in 0..0x1000 {
                    let byte = temp_addr_src.as_mut_ptr::<u8>().add(i).read_volatile();
                    temp_addr_dst.as_mut_ptr::<u8>().add(i).write_volatile(byte);
                }
            }

            // unmap the temporary pages
            // dont deallocate the frames, the old one belongs to the parent process, the new one to the child
            let (_old_frame, flush) = kernel_paging
                .page_table
                .unmap(Page::<Size4KiB>::containing_address(temp_addr_src))
                .expect("Failed to unmap old frame in cloned page table");
            flush.flush();

            let (_frame, flush) = kernel_paging
                .page_table
                .unmap(Page::<Size4KiB>::containing_address(temp_addr_dst))
                .expect("Failed to unmap new frame in cloned page table");
            flush.flush();
        }

        UserspacePaging {
            page_table: new_page_table,
        }
    }
}
