use core::arch::asm;
use core::fmt::Write;

use alloc::string::String;

use crate::term::TERM;

static mut PANIC_STRING: String = String::new();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { asm!("cli") }

    unsafe {
        PANIC_STRING = String::from_raw_parts(0x10000 as *mut u8, 0, 2048);

        TERM.bg = 0xFF32292F;
        TERM.fg = 0xFFF87060;
        TERM.clear();
        TERM.println("Allahkaputtputt!!");
        write!(&mut PANIC_STRING, "{}", info).unwrap();
        TERM.println(&PANIC_STRING);
    };

    loop {
        unsafe {
            asm!("cli");
            asm!("hlt");
        }
    }
}
