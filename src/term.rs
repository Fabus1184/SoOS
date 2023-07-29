use crate::font::{self, FONT_HEIGHT, FONT_WIDTH};

pub static FRAMEBUFFER_REQUEST: limine::FramebufferRequest = limine::FramebufferRequest::new(0);

pub static mut TERM: once_cell::unsync::Lazy<Term> = once_cell::unsync::Lazy::new(|| {
    let fbr = FRAMEBUFFER_REQUEST
        .get_response()
        .get()
        .expect("Failed to get framebuffer!");
    let fb = fbr.framebuffers().iter().next().expect("No framebuffers!");
    Term::new(fb)
});

pub struct Term {
    x: usize,
    y: usize,
    pub fg: u32,
    pub bg: u32,
    framebuffer: &'static limine::Framebuffer,
}

const FONT_SCALE: u64 = 2;

unsafe impl Sync for Term {}

impl Term {
    pub fn new(fb: &'static limine::Framebuffer) -> Term {
        Term {
            x: 0,
            y: 0,
            fg: 0xFFFFFFFF,
            bg: 0x00000000,
            framebuffer: fb,
        }
    }

    pub fn clear(&mut self) {
        for x in 0..self.framebuffer.width {
            for y in 0..self.framebuffer.height {
                self.blit(x, y, self.bg);
            }
        }

        self.x = 0;
        self.y = 0;
    }

    pub fn blit(&self, x: u64, y: u64, color: u32) {
        if x >= self.framebuffer.width as u64 || y >= self.framebuffer.height as u64 {}

        let fb_ptr = self.framebuffer.address.as_ptr().unwrap() as *mut u32;

        unsafe {
            fb_ptr
                .wrapping_offset((y * self.framebuffer.width + x) as isize)
                .write_volatile(color);
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
        if c == '\0' {
            return;
        }

        if c.is_ascii_graphic() {
            let x_off = (self.x * FONT_WIDTH) as u64 / FONT_SCALE;
            let y_off = (self.y * FONT_HEIGHT) as u64 / FONT_SCALE;

            for x in 0..FONT_WIDTH {
                for y in 0..FONT_HEIGHT {
                    let byte = font::FONT[c as usize - 32][FONT_WIDTH * (y / 8) + x];
                    let bit = (byte >> (y % 8)) & 1;
                    let color = if bit == 1 { self.fg } else { self.bg };
                    self.blit(
                        x_off + x as u64 / FONT_SCALE,
                        y_off + y as u64 / FONT_SCALE,
                        color,
                    );
                }
            }
        }

        if c == '\n' {
            self.x = 0;
            self.y += 1;
        } else {
            self.x += 1;
        }

        if self.x >= self.framebuffer.width as usize / FONT_WIDTH * FONT_SCALE as usize {
            self.x = 0;
            self.y += 1;
        }

        if self.y >= self.framebuffer.height as usize / FONT_HEIGHT * FONT_SCALE as usize {
            self.scroll();
            self.y -= 1;
        }
    }

    pub fn scroll(&mut self) {
        let fb_ptr = self.framebuffer.address.as_ptr().unwrap() as *mut u32;
        unsafe {
            core::ptr::copy(
                fb_ptr.wrapping_offset((FONT_HEIGHT as u64 * self.framebuffer.width / 2) as isize),
                fb_ptr,
                (self.framebuffer.width * self.framebuffer.height) as usize,
            );
        }
    }
}
