use crate::font::{FONT_HEIGHT, FONT_WIDTH};

pub struct Term<'a> {
    x: usize,
    y: usize,
    pub fg: u32,
    pub bg: u32,
    framebuffer: &'a limine::LimineFramebuffer,
}
impl<'a> Term<'a> {
    pub fn new(fb: &'a limine::LimineFramebuffer) -> Term<'a> {
        Term {
            x: 0,
            y: 0,
            fg: 0xFFFFFFFF,
            bg: 0x00000000,
            framebuffer: fb,
        }
    }

    pub fn clear(&self) {
        for x in 0..self.framebuffer.width {
            for y in 0..self.framebuffer.height {
                unsafe {
                    (self.framebuffer.address.as_ptr().unwrap() as *mut u32)
                        .wrapping_offset((y * self.framebuffer.width + x) as isize)
                        .write(self.bg);
                };
            }
        }
    }

    pub fn print(&mut self, s: &str) {
        for c in s.chars() {
            self.print_char(c);
        }
    }

    pub fn println(&mut self, s: &str) {
        self.print(s);
        self.print_char('\n');
    }

    pub fn print_char(&mut self, c: char) {
        let fb_ptr = self.framebuffer.address.as_ptr().unwrap() as *mut u32;

        if c == '\0' {
            return;
        }

        if c.is_ascii_graphic() {
            let x_off = (self.x * FONT_WIDTH) as u64;
            let y_off = (self.y * FONT_HEIGHT) as u64;

            for x in 0..FONT_WIDTH {
                for y in 0..FONT_HEIGHT {
                    let byte = crate::font::FONT[c as usize - 32][FONT_WIDTH * (y / 8) + x];
                    let bit = (byte >> (y % 8)) & 1;
                    let color = if bit == 1 { self.fg } else { self.bg };
                    unsafe {
                        *fb_ptr.wrapping_offset(
                            ((y_off + y as u64) * self.framebuffer.width + x as u64 + x_off)
                                as isize,
                        ) = color;
                    }
                }
            }
        }

        if c == '\n' {
            self.x = 0;
            self.y += 1;
        } else {
            self.x += 1;
        }

        if self.x >= self.framebuffer.width as usize / FONT_WIDTH {
            self.x = 0;
            self.y += 1;
        }
    }
}
