pub struct Font {
    png: minipng::ImageData<'static>,
    pub tile_size: (u32, u32),
    columns: u32,
}

pub static FONT: spin::Lazy<Font> = spin::Lazy::new(|| {
    static mut BUFFER: [u8; 1 << 19] = [0; 1 << 19];

    let bytes = include_bytes!("../../font.png");

    let mut png =
        minipng::decode_png(bytes, unsafe { &mut BUFFER }).expect("Failed to decode font PNG");

    png.convert_to_rgba8bpc()
        .expect("Failed to convert font PNG to RGBA8");

    Font {
        png,
        tile_size: (11, 20),
        columns: 20,
    }
});

impl Font {
    #[inline]
    pub fn get_pixel(&self, char: char, x: u32, y: u32) -> Option<u8> {
        if char < ' ' || char > '~' {
            return None; // Only handle printable ASCII characters
        }

        let char_index = char as u32 - ' ' as u32;
        let char_x = char_index % self.columns;
        let char_y = char_index / self.columns;

        let pixel_x = char_x * self.tile_size.0 + x;
        let pixel_y = char_y * self.tile_size.1 + y;

        let width = self.png.width();
        let height = self.png.height();
        if pixel_x >= width || pixel_y >= height {
            return None; // Out of bounds
        }

        let index = (pixel_y * width + pixel_x) as usize * 4; // 4 bytes per pixel (RGBA)
        if index + 3 >= self.png.pixels().len() {
            return None; // Out of bounds
        }

        // Return the alpha channel value (0-255)
        Some(self.png.pixels()[index + 3])
    }
}
