use core::arch::asm;
use core::fmt::Write;

use alloc::string::String;

use crate::term::TERM;

static mut BUF: [u8; 8192] = [0; 8192];

struct Writer<'a>(&'a mut [u8], usize);
impl Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            self.0[self.1] = b;
            self.1 += 1;
        }
        Ok(())
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { asm!("cli") }

    unsafe {
        TERM.fg = 0xFFF87060;
        TERM.println("\nAllahkaputtputt!!");
        TERM.print("Kernel Panic: ");

        let mut writer = Writer(BUF.as_mut(), 0);
        write!(writer, "{}", info).expect("Failed to write panic info!");
        let str = core::str::from_utf8(BUF[..writer.1].as_ref())
            .expect("Failed to convert panic info to string!");
        TERM.println(str);
    };

    unsafe {
        asm!("cli");
        loop {
            asm!("nop");
        }
    }
}
