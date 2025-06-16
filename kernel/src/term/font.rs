use crate::term::Color;

pub struct Font {
    chars: [[[u8; Self::CHAR_WIDTH]; Self::CHAR_HEIGHT]; 128],
}

impl Font {
    const CHAR_WIDTH: usize = 11;
    const CHAR_HEIGHT: usize = 20;

    pub fn width_pixels(&self) -> usize {
        Self::CHAR_WIDTH
    }
    pub fn height_pixels(&self) -> usize {
        Self::CHAR_HEIGHT
    }

    fn new() -> Self {
        let buffer: &mut [u8] = {
            static mut BUFFER: [u8; 1 << 19] = [0; 1 << 19];
            unsafe { &mut BUFFER }
        };

        let bytes = include_bytes!("../../../font.png");

        let mut png = minipng::decode_png(bytes, buffer).expect("Failed to decode font PNG");
        let columns = 20;

        png.convert_to_rgba8bpc()
            .expect("Failed to convert font PNG to RGBA8");

        let mut chars = [[[0; Self::CHAR_WIDTH]; Self::CHAR_HEIGHT]; 128];

        for c in ' '..'~' {
            let char_index = (c as u32 - ' ' as u32) as usize;
            let tile_position = (
                (char_index % columns) * Self::CHAR_WIDTH,
                (char_index / columns) * Self::CHAR_HEIGHT,
            );

            for y in 0..Self::CHAR_HEIGHT {
                for x in 0..Self::CHAR_WIDTH {
                    let pixel_index = (tile_position.1 + y) * (columns * Self::CHAR_WIDTH)
                        + (tile_position.0 + x);
                    let alpha = png.pixels()[pixel_index as usize * 4 + 3];

                    chars[char_index][y][x] = alpha;
                }
            }
        }

        Self { chars }
    }

    pub fn blit_char(&self, c: char, ptr: *mut Color, row_pitch: usize, fg: Color, bg: Color) {
        let bitmap = self
            .chars
            .get(c as usize - ' ' as usize)
            .expect("Character not found in font");

        for row in 0..Self::CHAR_HEIGHT {
            for col in 0..Self::CHAR_WIDTH {
                let ptr = unsafe { ptr.add(row_pitch * row + col) };

                let alpha = bitmap[row][col];

                if alpha > 0 {
                    unsafe {
                        ptr.write_volatile(fg.blend(bg, alpha));
                    }
                }
            }
        }
    }
}

pub static FONT_1: spin::Lazy<Font> = spin::Lazy::new(Font::new);
