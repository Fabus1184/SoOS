use core::arch::asm;

use log::trace;
use x86_64::{
    structures::{
        gdt::SegmentSelector,
        idt::InterruptStackFrameValue,
        paging::{FrameAllocator, Mapper, Page, PageTable, PageTableFlags, Size4KiB},
    },
    VirtAddr,
};

use crate::{elf, kernel::paging::SoosPaging};

#[derive(Debug, Clone, Copy)]
pub enum ProcessState {
    Running,
    Waiting,
    Ready,
}

pub struct Process<'a> {
    paging: SoosPaging<'a>,
    stack: InterruptStackFrameValue,
    state: ProcessState,
}

impl<'a> Process<'a> {
    pub fn from_elf_bytes(
        elf_bytes: &[u8],
        physical_memory_offset: u64,
        kernel_page_table: *const PageTable,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
        user_code_segment: SegmentSelector,
        user_data_segment: SegmentSelector,
        stack_address: VirtAddr,
        code_address: VirtAddr,
        stack_size_pages: u64,
    ) -> Self {
        let user_page_table = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame!")
            .start_address()
            .as_u64() as *mut PageTable;
        unsafe { (&mut *user_page_table).zero() };
        unsafe { core::ptr::copy::<PageTable>(kernel_page_table, user_page_table, 1) };

        let mut paging =
            SoosPaging::offset_page_table(physical_memory_offset, unsafe { &mut *user_page_table });

        unsafe { paging.load() };

        trace!("Loading ELF...");
        let entry_point = elf::load(&mut paging, frame_allocator, elf_bytes, code_address);
        trace!("ELF loaded!");

        for i in 0..stack_size_pages {
            trace!(
                "Mapping stack page: {:?} at {:?}",
                i,
                stack_address + (i * 4096)
            );
            let stack_page = Page::containing_address(stack_address + (i * 4096));
            let frame = frame_allocator
                .allocate_frame()
                .expect("Failed to allocate frame!");
            unsafe {
                paging
                    .offset_page_table
                    .map_to(
                        stack_page,
                        frame,
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::USER_ACCESSIBLE
                            | PageTableFlags::NO_EXECUTE,
                        frame_allocator,
                    )
                    .expect("Failed to map page!")
                    .flush()
            };
        }

        Self {
            paging,
            stack: InterruptStackFrameValue {
                instruction_pointer: entry_point,
                code_segment: ((user_code_segment.index() * 8) | 3) as u64,
                cpu_flags: 0x202,
                stack_pointer: stack_address + (stack_size_pages * 4096),
                stack_segment: ((user_data_segment.index() * 8) | 3) as u64,
            },
            state: ProcessState::Ready,
        }
    }

    pub unsafe fn run(&mut self) -> ! {
        self.paging.load();

        asm!(
            "cli",
            "push {uds:r}",
            "push {stack:r}",
            "push {rflags:r}",
            "push {ucs:r}",
            "push {userland_function:r}",
            "mov ax, {uds:x}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "iretq",
            uds = in(reg) self.stack.stack_segment,
            ucs = in(reg) self.stack.code_segment,
            // out("ax") _,
            stack = in(reg) self.stack.stack_pointer.as_u64(),
            rflags = in(reg) 0x202,
            userland_function = in(reg) self.stack.instruction_pointer.as_u64(),
            options(noreturn),
        );
    }
}
