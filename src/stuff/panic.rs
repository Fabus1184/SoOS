use core::arch::asm;

use crate::term::TERM;

use core::fmt::Write as _;

struct Cursor<'a> {
    index: usize,
    buffer: &'a mut [u8],
}

impl<'a> Cursor<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        Cursor { index: 0, buffer }
    }

    fn finish(self) -> &'a mut [u8] {
        &mut self.buffer[..self.index]
    }
}

impl core::fmt::Write for Cursor<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.index + s.len() > self.buffer.len() {
            return Err(core::fmt::Error);
        }

        self.buffer[self.index..self.index + s.len()].copy_from_slice(s.as_bytes());

        self.index += s.len();

        Ok(())
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { asm!("cli") }

    TERM.set_color(0xFFF87060, 0xFF000000);
    TERM.println("\nAllahkaputtputt!!");
    TERM.print("Kernel Panic: ");

    let mut buffer = [0u8; 8192];
    let mut writer = Cursor::new(&mut buffer);

    write!(writer, "{}", info).expect("Failed to write panic info!");

    let bytes = writer.finish();
    let str = core::str::from_utf8(bytes).expect("Failed to convert panic info to string!");
    TERM.println(str);

    unsafe {
        asm!("cli");
        loop {
            asm!("nop");
        }
    }
}
