const std = @import("std");
const zigimg = @import("zigimg");

const PNG = @embedFile("font.png");

pub const Font = struct {
    pub const CHAR_WIDTH: usize = 11;
    pub const CHAR_HEIGHT: usize = 20;
    pub const CHAR_COUNT: usize = 128;

    chars: [CHAR_COUNT][CHAR_HEIGHT][CHAR_WIDTH]u8,

    pub fn init() !Font {
        var buffer: [0x100_000]u8 = undefined;
        var allocator = std.heap.FixedBufferAllocator.init(&buffer);

        var image = try zigimg.Image.fromMemory(allocator.allocator(), PNG);
        defer image.deinit();

        try image.convert(.grayscale8Alpha);

        var font = Font{ .chars = undefined };

        for (' '..'~') |c| {
            const char_index = @as(usize, @intCast(c)) - @as(usize, @intCast(' '));
            const char_x = (c - ' ') % 20 * CHAR_WIDTH;
            const char_y = (c - ' ') / 20 * CHAR_HEIGHT;

            for (0..CHAR_HEIGHT) |y| {
                for (0..CHAR_WIDTH) |x| {
                    font.chars[char_index][y][x] = image.pixels.grayscale8Alpha.ptr[(char_y + y) * image.width + (char_x + x)].alpha;
                }
            }
        }

        return font;
    }

    pub fn getChar(self: Font, c: u8) ?[CHAR_HEIGHT][CHAR_WIDTH]u8 {
        if (c < ' ' or c >= '~') {
            return null;
        }
        const char_index = @as(usize, @intCast(c)) - @as(usize, @intCast(' '));
        return self.chars[char_index];
    }
};
