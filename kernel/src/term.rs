use core::sync::atomic::AtomicUsize;

use limine::request::FramebufferRequest;

use crate::font::{self, FONT_HEIGHT, FONT_WIDTH};

pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub static TERM: spin::Lazy<Term> = spin::Lazy::new(|| {
    let fbr = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("Failed to get framebuffer!");
    let fb = fbr.framebuffers().next().expect("No framebuffers!");

    Term::new(&fb)
});

pub struct Term {
    pub framebuffer: Framebuffer,
    x: AtomicUsize,
    y: AtomicUsize,
}

#[derive(Clone, Copy)]
pub struct Framebuffer {
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub ptr: *mut u32,
}

unsafe impl Sync for Framebuffer {}
unsafe impl Send for Framebuffer {}

struct Performer {
    x: usize,
    y: usize,
    fg: u32,
    bg: u32,
    framebuffer: Framebuffer,
}

impl Term {
    pub fn new(framebuffer: &limine::framebuffer::Framebuffer<'static>) -> Term {
        Term {
            framebuffer: Framebuffer {
                width: framebuffer.width(),
                height: framebuffer.height(),
                pitch: framebuffer.pitch() / 32,
                ptr: framebuffer.addr().cast::<u32>(),
            },
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
                fg: 0xFFFF_FFFF,
                bg: 0xFF00_0000,
                framebuffer: self.framebuffer,
            },
        }
    }
}

struct Writer<'a> {
    term: &'a Term,
    performer: Performer,
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

impl Performer {
    fn blit(&self, x: u64, y: u64, color: u32) {
        let ptr = self
            .framebuffer
            .ptr
            .wrapping_offset((y * self.framebuffer.width + x) as isize);
        assert!(
            (ptr as usize)
                < self.framebuffer.ptr as usize
                    + (self.framebuffer.width * self.framebuffer.height * 4) as usize,
            "framebuffer {:#0x} out of bounds at ({x}, {y})",
            ptr as usize
        );

        unsafe {
            ptr.write_volatile(color);
        }
    }

    fn clear(&self) {
        for x in 0..self.framebuffer.width {
            for y in 0..self.framebuffer.height {
                self.blit(x, y, self.bg);
            }
        }
    }

    fn print_char(&mut self, c: char) {
        let font_scale: u64 = 2;

        if self.x >= self.framebuffer.width as usize / FONT_WIDTH * font_scale as usize {
            self.x = 0;
            self.y += 1;
        }

        if self.y >= self.framebuffer.height as usize / FONT_HEIGHT * font_scale as usize {
            // If no lines remain, scroll the screen
            self.scroll();
            self.y = self.y.saturating_sub(2);
        }

        if c == '\0' {
            return;
        } else if c == ' ' {
            let x_off = (self.x * FONT_WIDTH) as u64 / font_scale;
            let y_off = (self.y * FONT_HEIGHT) as u64 / font_scale;
            for x in 0..FONT_WIDTH {
                for y in 0..FONT_HEIGHT {
                    self.blit(
                        x_off + x as u64 / font_scale,
                        y_off + y as u64 / font_scale,
                        self.bg,
                    );
                }
            }
        } else if c.is_ascii_graphic() {
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
    }

    fn scroll(&self) {
        unsafe {
            core::ptr::copy(
                self.framebuffer
                    .ptr
                    .add(self.framebuffer.width as usize * FONT_HEIGHT),
                self.framebuffer.ptr,
                (self.framebuffer.height as usize - FONT_HEIGHT) * self.framebuffer.width as usize,
            );
        }

        // Clear the last line
        for x in 0..self.framebuffer.width {
            for y in 0..FONT_HEIGHT {
                self.blit(
                    x,
                    self.framebuffer.height - FONT_HEIGHT as u64 + y as u64,
                    self.bg,
                );
            }
        }
    }
}

impl vte::Perform for Performer {
    fn print(&mut self, c: char) {
        self.print_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            // new line
            b'\n' => {
                self.x = 0;
                self.y += 1;
            }
            // carriage return
            b'\r' => {
                self.x = 0;
            }
            // backspace
            b'\x08' => {
                self.x = self.x.saturating_sub(1);
            }
            c => panic!("Unhandled control character: {}", c),
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
                self.fg = 0xFFFF_FFFF; // Reset foreground color to white
                self.bg = 0xFF00_0000; // Reset background color to black
            }
            ('m', &[&[fg]]) if (30..=37).contains(&fg) => {
                self.fg = match fg {
                    30 => 0xFF00_0000, // Black
                    31 => 0xFFFF_0000, // Red
                    32 => 0xFF00_FF00, // Green
                    33 => 0xFFFF_FF00, // Yellow
                    34 => 0xFF00_00FF, // Blue
                    35 => 0xFFFF_00FF, // Magenta
                    36 => 0xFF00_FFFF, // Cyan
                    37 => 0xFFFF_FFFF, // White
                    _ => unreachable!(),
                };
            }
            ('m', &[&[bg]]) if (40..=47).contains(&bg) => {
                self.bg = match bg {
                    40 => 0xFF00_0000, // Black
                    41 => 0xFFFF_0000, // Red
                    42 => 0xFF00_FF00, // Green
                    43 => 0xFFFF_FF00, // Yellow
                    44 => 0xFF00_00FF, // Blue
                    45 => 0xFFFF_00FF, // Magenta
                    46 => 0xFF00_FFFF, // Cyan
                    47 => 0xFFFF_FFFF, // White
                    _ => unreachable!(),
                };
            }
            _ => panic!("Unhandled CSI action: {} with params: {:?}", action, params),
        }
    }
}
