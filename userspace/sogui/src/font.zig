const std = @import("std");

const zigimg = @import("zigimg");
const soos = @import("soos");

pub const Font = struct {
    const FONT: []const u8 = @embedFile("font.png");

    chars: [][40 * 40]u32 = undefined,

    pub fn load(allocator: std.mem.Allocator) !Font {
        var image = try zigimg.Image.fromMemory(allocator, FONT);
        defer image.deinit();

        std.log.debug("image pixel format {}", .{image.pixelFormat()});

        var chars = try allocator.alloc([40 * 40]u32, '~' - ' ' + 1);

        for (' '..'~') |char| {
            const charIndex = char - ' ';
            const charRow = charIndex / 20;
            const charCol = charIndex % 20;

            const charWidth = 40;
            const charHeight = 40;

            for (0..charHeight) |i| {
                for (0..charWidth) |j| {
                    const alpha = image.pixels.rgba32[
                        (charRow * charHeight + i) * image.width + (charCol * charWidth + j)
                    ].a;

                    chars[charIndex][i * charWidth + j] = if (alpha >= 127) 0xFF_FFFFFF else 0x00_000000;
                }
            }
        }

        return Font{ .chars = chars };
    }

    pub fn deinit(self: Font, allocator: std.mem.Allocator) void {
        allocator.free(self.chars);
    }

    fn blitChar(self: Font, color: u32, char: u8, x: usize, y: usize, downscale: usize, framebuffer: soos.Framebuffer) void {
        if (char < ' ' or char > '~') {
            return;
        }
        const charIndex = char - ' ';

        for (0..40) |i| {
            for (0..40) |j| {
                const pixel = self.chars[charIndex][i * 40 + j];
                if ((pixel >> 24) & 0xFF != 0) {
                    framebuffer.blit(x + j / downscale, y + i / downscale, color);
                }
            }
        }
    }

    pub fn blitText(
        self: Font,
        color: u32,
        text: []const u8,
        x: usize,
        y: usize,
        downscale: usize,
        framebuffer: soos.Framebuffer,
    ) void {
        var currentX = x;
        for (text) |char| {
            if (char == ' ') {
                currentX += 40 / downscale;
                continue;
            }

            self.blitChar(color, char, currentX, y, downscale, framebuffer);
            currentX += 40 / downscale;
        }
    }
};
