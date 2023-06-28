use core::arch::asm;
use core::fmt::Write;

use alloc::string::String;

use crate::FRAMEBUFFER_REQUEST;

static mut PANIC_STRING: String = String::new();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { PANIC_STRING = String::from_raw_parts(0x10000 as *mut u8, 0, 2048) };

    let fb = &FRAMEBUFFER_REQUEST
        .get_response()
        .get()
        .unwrap()
        .framebuffers()[0];

    let mut term = crate::display::Term::new(fb);
    term.bg = 0xFF0000FF;
    term.clear();

    term.println("Allahkaputtputt!!");

    write!(unsafe { &mut PANIC_STRING }, "{}", info).unwrap();
    term.println(unsafe { &PANIC_STRING });

    loop {
        unsafe {
            asm!("cli");
            asm!("hlt");
        }
    }
}
