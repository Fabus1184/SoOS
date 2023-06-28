#![no_std]
#![no_main]

extern crate alloc;

mod allocator;
mod display;
mod font;
mod panic;

use core::{arch::asm, fmt::Write};

use alloc::string::String;

use crate::allocator::ALLOCATOR;

pub static FRAMEBUFFER_REQUEST: limine::LimineFramebufferRequest =
    limine::LimineFramebufferRequest::new(0);

static MEMMAP_REQUEST: limine::LimineMemmapRequest = limine::LimineMemmapRequest::new(0);

#[no_mangle]
extern "C" fn _start() -> ! {
    let response = FRAMEBUFFER_REQUEST.get_response().get().unwrap();

    if response.framebuffer_count == 0 {
        panic!("No framebuffer found!");
    }

    let fb = &response.framebuffers()[0];
    let mut term = display::Term::new(fb);
    term.fg = 0xFF00FF00;
    term.println("Hello, world!");

    let memmap = MEMMAP_REQUEST.get_response().get().unwrap();
    unsafe { ALLOCATOR.load_limine_memmap(memmap) };

    let mut s = String::new();
    write!(s, "Memory map: {:#?}", memmap).unwrap();
    term.println(&s);

    panic!("HIer wallah PANIK wallah");

    loop {
        unsafe { asm!("nop") };
    }

    panic!("Kernel returned from main!");
}
