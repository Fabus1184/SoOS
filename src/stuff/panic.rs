use core::arch::asm;
use core::fmt::Write;

use alloc::string::String;

use crate::term::TERM;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { asm!("cli") }

    unsafe {
        TERM.fg = 0xFFF87060;
        TERM.println("\nAllahkaputtputt!!");

        let mut panic_buf = String::new();
        write!(&mut panic_buf, "{}", info).unwrap();
        TERM.println(&panic_buf);
    };

    loop {
        unsafe {
            asm!("cli");
            asm!("hlt");
        }
    }
}
