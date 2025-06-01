use core::sync::atomic::AtomicUsize;

use limine::request::FramebufferRequest;

use crate::font::{self, FONT_HEIGHT, FONT_WIDTH};

pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub static TERM: spin::Lazy<Term> = spin::Lazy::new(|| {
    let fbr = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("Failed to get framebuffer!");
    let fb = fbr.framebuffers().next().expect("No framebuffers!");
    Term::new(fb)
});

pub struct Term {
    framebuffer: limine::framebuffer::Framebuffer<'static>,
    x: AtomicUsize,
    y: AtomicUsize,
}

struct Performer<'a> {
    x: usize,
    y: usize,
    fg: u32,
    bg: u32,
    framebuffer: &'a limine::framebuffer::Framebuffer<'static>,
}

impl Term {
    pub fn new(framebuffer: limine::framebuffer::Framebuffer<'static>) -> Term {
        Term {
            framebuffer,
            x: AtomicUsize::new(0),
            y: AtomicUsize::new(0),
        }
    }

    pub fn writer(&self) -> impl core::fmt::Write + '_ {
        Writer {
            term: self,
            performer: Performer {
                x: self.x.load(core::sync::atomic::Ordering::Relaxed),
                y: self.y.load(core::sync::atomic::Ordering::Relaxed),
                fg: 0xFFFFFFFF,
                bg: 0xFF000000,
                framebuffer: &self.framebuffer,
            },
        }
    }
}

struct Writer<'a> {
    term: &'a Term,
    performer: Performer<'a>,
}

impl Drop for Writer<'_> {
    fn drop(&mut self) {
        // Update the term's cursor position
        self.term
            .x
            .store(self.performer.x, core::sync::atomic::Ordering::Relaxed);
        self.term
            .y
            .store(self.performer.y, core::sync::atomic::Ordering::Relaxed);
    }
}

impl core::fmt::Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut state_machine = vte::Parser::new();

        state_machine.advance(&mut self.performer, s.as_bytes());

        Ok(())
    }
}

impl Performer<'_> {
    fn blit(&self, x: u64, y: u64, color: u32) {
        unsafe {
            (self.framebuffer.addr() as *mut u32)
                .wrapping_offset((y * self.framebuffer.width() + x) as isize)
                .write_volatile(color);
        }
    }

    fn clear(&self) {
        for x in 0..self.framebuffer.width() {
            for y in 0..self.framebuffer.height() {
                self.blit(x, y, self.bg);
            }
        }
    }

    fn print_char(&mut self, c: char) {
        let font_scale: u64 = 2;

        if c == '\0' {
            return;
        }

        if c == '\x08' {
            if self.x > 0 {
                self.blit(
                    (self.x * FONT_WIDTH) as u64 / font_scale,
                    (self.y * FONT_HEIGHT) as u64 / font_scale,
                    self.bg,
                );
                self.x = self.x.saturating_sub(1);
            }
            return;
        }

        if c.is_ascii_graphic() {
            let x_off = (self.x * FONT_WIDTH) as u64 / font_scale;
            let y_off = (self.y * FONT_HEIGHT) as u64 / font_scale;

            for x in 0..FONT_WIDTH {
                for y in 0..FONT_HEIGHT {
                    let byte = font::FONT[c as usize - 32][FONT_WIDTH * (y / 8) + x];
                    let bit = (byte >> (y % 8)) & 1;
                    let color = if bit == 1 { self.fg } else { self.bg };
                    self.blit(
                        x_off + x as u64 / font_scale,
                        y_off + y as u64 / font_scale,
                        color,
                    );
                }
            }
        }

        self.x += 1;

        if self.x >= self.framebuffer.width() as usize / FONT_WIDTH * font_scale as usize {
            self.x = 0;
            self.y += 1;
        }

        if self.y >= self.framebuffer.height() as usize / FONT_HEIGHT * font_scale as usize {
            self.scroll();
            self.y = self.y.saturating_sub(1);
        }
    }

    fn scroll(&self) {
        let fb_ptr = self.framebuffer.addr() as *mut u32;
        unsafe {
            core::ptr::copy(
                fb_ptr
                    .wrapping_offset((FONT_HEIGHT as u64 * self.framebuffer.width() / 2) as isize),
                fb_ptr,
                (self.framebuffer.width() * self.framebuffer.height()) as usize,
            );
        }
    }
}

impl vte::Perform for Performer<'_> {
    fn print(&mut self, c: char) {
        self.print_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.x = 0;
                self.y += 1;
            }
            b'\r' => {
                self.x = 0;
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let params = params.iter().collect::<heapless::Vec<&[u16], 4>>();

        match (action, params.as_slice()) {
            // Clear screen
            ('J', [[2]]) => {
                self.clear();
                self.x = 0;
                self.y = 0;
            }
            // Move cursor to position
            ('H', [[0]]) => {
                self.x = 0;
                self.y = 0;
            }
            // Text attributes
            ('m', [[0]]) => {
                self.fg = 0xFFFFFFFF; // Reset foreground color to white
                self.bg = 0xFF000000; // Reset background color to black
            }
            ('m', &[&[fg]]) if (30..=37).contains(&fg) => {
                self.fg = match fg {
                    30 => 0xFF000000, // Black
                    31 => 0xFFFF0000, // Red
                    32 => 0xFF00FF00, // Green
                    33 => 0xFFFFFF00, // Yellow
                    34 => 0xFF0000FF, // Blue
                    35 => 0xFFFF00FF, // Magenta
                    36 => 0xFF00FFFF, // Cyan
                    37 => 0xFFFFFFFF, // White
                    _ => unreachable!(),
                };
            }
            ('m', &[&[bg]]) if (40..=47).contains(&bg) => {
                self.bg = match bg {
                    40 => 0xFF000000, // Black
                    41 => 0xFFFF0000, // Red
                    42 => 0xFF00FF00, // Green
                    43 => 0xFFFFFF00, // Yellow
                    44 => 0xFF0000FF, // Blue
                    45 => 0xFFFF00FF, // Magenta
                    46 => 0xFF00FFFF, // Cyan
                    47 => 0xFFFFFFFF, // White
                    _ => unreachable!(),
                };
            }
            _ => panic!("Unhandled CSI action: {} with params: {:?}", action, params),
        }
    }
}
