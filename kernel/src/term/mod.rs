use core::cell::Cell;

use limine::request::FramebufferRequest;

mod font;
mod vte;

pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub static TERM: spin::Lazy<Term> = spin::Lazy::new(|| {
    let fbr = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("Failed to get framebuffer!");
    let fb = fbr.framebuffers().next().expect("No framebuffers!");

    let term = Term::new(&fb);
    term.clear();

    term
});

pub struct Term {
    pub ptr_pixels: *mut Color,
    pub width_pixels: usize,
    pub height_pixels: usize,
    x: Cell<usize>,
    y: Cell<usize>,
    font: &'static font::Font,
    fg: Cell<Color>,
    bg: Cell<Color>,
}
unsafe impl Send for Term {}
unsafe impl Sync for Term {}

impl Term {
    pub fn new(framebuffer: &limine::framebuffer::Framebuffer<'static>) -> Term {
        assert!(framebuffer.bpp() == 32, "framebuffer must be 32 bpp");
        assert!(
            framebuffer.width() * framebuffer.height() % 64 == 0,
            "framebuffer size must be a multiple of 64"
        );

        Term {
            ptr_pixels: framebuffer.addr().cast::<Color>(),
            width_pixels: framebuffer.width() as usize,
            height_pixels: framebuffer.height() as usize,
            x: Cell::new(0),
            y: Cell::new(0),
            font: &font::FONT_1,
            fg: Cell::new(Color::WHITE),
            bg: Cell::new(Color::BLACK),
        }
    }

    pub fn writer(&self) -> impl core::fmt::Write + '_ {
        vte::Writer::new(self)
    }
}

// pixel based interface
impl Term {
    fn putpixel(&self, x: usize, y: usize, color: Color) {
        let ptr = self.ptr_pixels.wrapping_add(y * self.width_pixels + x);
        debug_assert!(
            ptr < unsafe { self.ptr_pixels.add(self.width_pixels * self.height_pixels) },
            "framebuffer {:#0x} out of bounds at ({x}, {y})",
            ptr as usize
        );

        unsafe {
            ptr.write_volatile(color);
        }
    }

    fn clear(&self) {
        // clear by in chunks of 64 pixels
        let chunks = self.width_pixels * self.height_pixels / 64;
        for i in 0..chunks {
            let mask = core::simd::Mask::from_bitmask(u64::MAX);
            unsafe {
                core::simd::u32x64::splat(self.bg.get().as_u32())
                    .store_select_ptr(self.ptr_pixels.cast::<u32>().add(i * 64), mask);
            }
        }
    }
}

// char based interface
impl Term {
    fn columns(&self) -> usize {
        self.width_pixels / self.font.width_pixels()
    }
    fn rows(&self) -> usize {
        self.height_pixels / self.font.height_pixels()
    }
    fn current_pixel_position(&self) -> (usize, usize) {
        (
            self.x.get() * self.font.width_pixels(),
            self.y.get() * self.font.height_pixels(),
        )
    }
    fn pixel_ptr(&self, column: usize, row: usize) -> *mut Color {
        let x = column * self.font.width_pixels();
        let y = row * self.font.height_pixels();
        unsafe { self.ptr_pixels.add(y * self.width_pixels + x) }
    }

    fn putchar(&self, c: char) {
        // if no columns remain, go to the next line
        if self.x.get() >= self.columns() {
            self.x.set(0);
            self.y.update(|y| y + 1);
        }

        // if no rows remain, scroll the screen
        if self.y.get() >= self.rows() {
            for _ in 0..=(self.y.get() - self.rows()) {
                self.scroll();
                self.y.update(|y| y - 1);
            }
        }

        let position = self.current_pixel_position();
        match c {
            ' ' => {
                for y in 0..self.font.height_pixels() {
                    unsafe {
                        self.font.simd_width(self.bg.get()).store_select_ptr(
                            self.ptr_pixels
                                .add((position.1 + y) * self.width_pixels + position.0)
                                .cast::<u32>(),
                            core::simd::Mask::from_bitmask(u64::MAX),
                        );
                    }
                }
            }
            '█' => {
                for y in 0..self.font.height_pixels() {
                    unsafe {
                        self.font.simd_width(self.fg.get()).store_select_ptr(
                            self.ptr_pixels
                                .add((position.1 + y) * self.width_pixels + position.0)
                                .cast::<u32>(),
                            core::simd::Mask::from_bitmask(u64::MAX),
                        );
                    }
                }
            }
            c if c.is_ascii_graphic() => {
                let ptr = unsafe {
                    self.ptr_pixels
                        .add(position.1 * self.width_pixels + position.0)
                };

                self.font
                    .blit_char(c, ptr, self.width_pixels, self.fg.get(), self.bg.get());
            }
            c => panic!("unsupported character: {:x}", u32::from(c)),
        }

        self.x.update(|x| x + 1);
    }

    fn scroll(&self) {
        // copy in chunks of 64 pixels
        let chunks = self.width_pixels * (self.height_pixels - self.font.height_pixels()) / 64;
        for i in 0..chunks {
            let mask = core::simd::Mask::from_bitmask(u64::MAX);

            unsafe {
                core::simd::u32x64::load_select_ptr(
                    self.ptr_pixels
                        .cast::<u32>()
                        .add(i * 64 + self.width_pixels * self.font.height_pixels()),
                    mask,
                    core::simd::u32x64::splat(0x00),
                )
                .store_select_ptr(self.ptr_pixels.cast::<u32>().add(i * 64), mask);
            }
        }

        // clear the last line
        let ptr = self.pixel_ptr(self.rows(), self.rows() - 1).cast::<Color>();
        for i in 0..(self.width_pixels * self.font.height_pixels()) {
            unsafe {
                ptr.add(i).write_volatile(self.bg.get());
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(align(4))]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { b, g, r }
    }

    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 255, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);
    pub const CYAN: Color = Color::rgb(0, 255, 255);
    pub const MAGENTA: Color = Color::rgb(255, 0, 255);
    pub const YELLOW: Color = Color::rgb(255, 255, 0);

    pub const fn as_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn blend(self, other: Color, alpha: u8) -> Color {
        let alpha = alpha as u32;
        let inv_alpha = 255 - alpha;

        Color {
            r: ((other.r as u32 * inv_alpha + self.r as u32 * alpha) / 255) as u8,
            g: ((other.g as u32 * inv_alpha + self.g as u32 * alpha) / 255) as u8,
            b: ((other.b as u32 * inv_alpha + self.b as u32 * alpha) / 255) as u8,
        }
    }
}
