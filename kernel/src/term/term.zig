const std = @import("std");

const Font = @import("font.zig").Font;

pub const Term = struct {
    pub const Color = packed struct(u32) {
        b: u8,
        g: u8,
        r: u8,
        _: u8 = 0,

        pub fn rgb(r: u8, g: u8, b: u8) Color {
            return .{ .b = b, .g = g, .r = r };
        }

        pub const RED: Color = .rgb(0xFF, 0x00, 0x00);
        pub const GREEN: Color = .rgb(0x00, 0xFF, 0x00);
        pub const BLUE: Color = .rgb(0x00, 0x00, 0xFF);
        pub const WHITE: Color = .rgb(0xFF, 0xFF, 0xFF);
        pub const BLACK: Color = .rgb(0x00, 0x00, 0x00);
        pub const CYAN: Color = .rgb(0x00, 0xFF, 0xFF);
        pub const MAGENTA: Color = .rgb(0xFF, 0x00, 0xFF);
        pub const YELLOW: Color = .rgb(0xFF, 0xFF, 0x00);

        pub inline fn lerp(self: Color, other: Color, t: u8) Color {
            return Color{
                .b = @as(u8, @intCast((@as(u32, @intCast(self.b)) * (255 - t) + @as(u32, @intCast(other.b)) * t) / 255)),
                .g = @as(u8, @intCast((@as(u32, @intCast(self.g)) * (255 - t) + @as(u32, @intCast(other.g)) * t) / 255)),
                .r = @as(u8, @intCast((@as(u32, @intCast(self.r)) * (255 - t) + @as(u32, @intCast(other.r)) * t) / 255)),
            };
        }
    };

    ptr: [*]u32,
    width: u64,
    height: u64,

    font: Font,

    fg: Color = Term.Color.WHITE,
    bg: Color = Term.Color.BLACK,
    cursor: struct {
        x: u64 = 0,
        y: u64 = 0,
    } = .{ .x = 0, .y = 0 },

    pub fn init(ptr: *anyopaque, width: u64, height: u64) !Term {
        return Term{
            .ptr = @alignCast(@ptrCast(ptr)),
            .width = width,
            .height = height,
            .font = try Font.init(),
        };
    }

    pub fn clear(self: Term, bg: Color) void {
        const pixels = self.width * self.height;
        const chunkSize = 64;
        for (0..pixels / chunkSize) |i| {
            const chunkPtr: *@Vector(chunkSize, u32) = @alignCast(@ptrCast(self.ptr + i * chunkSize));
            chunkPtr.* = @splat(@bitCast(bg));
        }
    }

    pub fn scroll(self: Term) void {
        const lineHeight = @TypeOf(self.font).CHAR_HEIGHT;
        std.mem.copyForwards(
            u32,
            self.ptr[0 .. (self.height - lineHeight) * self.width],
            self.ptr[lineHeight * self.width .. self.height * self.width],
        );

        @memset(
            self.ptr[(self.height - lineHeight) * self.width .. self.height * self.width],
            @bitCast(self.bg),
        );
    }

    fn putChar(self: *Term, c: u8) void {
        const x = self.cursor.x;
        const y = self.cursor.y;

        switch (c) {
            '\n' => {
                self.cursor.y += 1;
                self.cursor.x = 0;
            },
            else => {
                const char = self.font.getChar(c) orelse return;

                for (0..@TypeOf(self.font).CHAR_HEIGHT) |char_y| {
                    for (0..@TypeOf(self.font).CHAR_WIDTH) |char_x| {
                        const pixel_x = x * @TypeOf(self.font).CHAR_WIDTH + char_x;
                        const pixel_y = y * @TypeOf(self.font).CHAR_HEIGHT + char_y;
                        if (pixel_x < self.width and pixel_y < self.height) {
                            const pixel_index = pixel_y * self.width + pixel_x;

                            self.ptr[pixel_index] = @bitCast(self.fg.lerp(self.bg, 255 - char[char_y][char_x]));
                        }
                    }
                }

                self.cursor.x += 1;
                if (self.cursor.x >= self.width / @TypeOf(self.font).CHAR_WIDTH) {
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                }
            },
        }

        if (self.cursor.y >= self.height / @TypeOf(self.font).CHAR_HEIGHT) {
            for (self.height / @TypeOf(self.font).CHAR_HEIGHT..self.cursor.y + 1) |_| {
                self.scroll();
                self.cursor.y -= 1;
            }
        }
    }

    pub fn ansiParser(self: *Term) TermWriter {
        return TermWriter{
            .term = self,
        };
    }
};

const TermWriter = struct {
    term: *Term,
    parser: Parser = .new(),

    const GenericWriter = std.io.GenericWriter(*TermWriter, error{}, TermWriter.write);

    pub fn writer(self: *TermWriter) GenericWriter {
        return GenericWriter{ .context = self };
    }

    fn write(self: *TermWriter, bytes: []const u8) !usize {
        for (bytes) |byte| {
            if (self.parser.feed(byte)) |token| {
                switch (token) {
                    .ansiSequence => |seq| {
                        if (std.mem.startsWith(u8, seq, "\x1b[")) {
                            const command = seq[2..];
                            if (std.mem.endsWith(u8, command, "m")) {
                                // Color change sequence
                                const colorCode = std.fmt.parseInt(u8, command[0 .. command.len - 1], 10) catch |err| {
                                    std.log.err("failed to parse color code: {c}, error: {}", .{ command, err });
                                    @panic("failed to parse color code");
                                };
                                switch (colorCode) {
                                    30 => self.term.fg = Term.Color.BLACK,
                                    31 => self.term.fg = Term.Color.RED,
                                    32 => self.term.fg = Term.Color.GREEN,
                                    33 => self.term.fg = Term.Color.YELLOW,
                                    34 => self.term.fg = Term.Color.BLUE,
                                    35 => self.term.fg = Term.Color.MAGENTA,
                                    36 => self.term.fg = Term.Color.CYAN,
                                    37 => self.term.fg = Term.Color.WHITE,
                                    40 => self.term.bg = Term.Color.BLACK,
                                    41 => self.term.bg = Term.Color.RED,
                                    42 => self.term.bg = Term.Color.GREEN,
                                    43 => self.term.bg = Term.Color.YELLOW,
                                    44 => self.term.bg = Term.Color.BLUE,
                                    45 => self.term.bg = Term.Color.MAGENTA,
                                    46 => self.term.bg = Term.Color.CYAN,
                                    47 => self.term.bg = Term.Color.WHITE,
                                    0 => {
                                        self.term.fg = Term.Color.WHITE;
                                        self.term.bg = Term.Color.BLACK;
                                    },
                                    else => {
                                        std.log.err("unknown color code: {d}", .{colorCode});
                                        @panic("unknown color code");
                                    },
                                }
                            } else if (std.mem.endsWith(u8, command, "K")) {
                                // Clear line sequence
                                self.term.clear(self.term.bg);
                            } else if (std.mem.endsWith(u8, command, "J")) {
                                // Clear screen sequence
                                self.term.clear(self.term.bg);
                            } else {
                                std.log.err("unknown ansi sequence: {c}", .{seq});
                                @panic("unknown ansi sequence");
                            }
                        } else {
                            std.log.err("invalid ansi sequence: {s}", .{seq});
                            @panic("invalid ansi sequence");
                        }
                    },
                    .char => {
                        self.term.putChar(token.char);
                    },
                }
            }
        }
        return bytes.len;
    }
};

const Token = union(enum) {
    ansiSequence: []const u8,
    char: u8,
};

const Parser = struct {
    state: enum { normal, escape, csi } = .normal,
    buf: [32]u8 = undefined,
    buf_len: usize = 0,

    pub fn new() Parser {
        return Parser{
            .state = .normal,
            .buf_len = 0,
        };
    }

    pub fn feed(self: *Parser, byte: u8) ?Token {
        switch (self.state) {
            .normal => {
                if (byte == 0x1B) { // ESC
                    self.state = .escape;
                    self.buf[0] = byte;
                    self.buf_len = 1;
                    return null;
                } else {
                    return Token{ .char = byte };
                }
            },
            .escape => {
                if (byte == '[') {
                    self.buf[self.buf_len] = byte;
                    self.buf_len += 1;
                    self.state = .csi;
                    return null;
                } else {
                    // Invalid sequence
                    std.log.err("invalid escape sequence: {}", .{byte});
                    @panic("Invalid escape sequence");
                }
            },
            .csi => {
                self.buf[self.buf_len] = byte;
                self.buf_len += 1;

                // Check for command terminator (a-z or A-Z)
                if ((byte >= 'a' and byte <= 'z') or (byte >= 'A' and byte <= 'Z')) {
                    self.state = .normal;
                    const result = self.buf[0..self.buf_len];
                    self.buf_len = 0;
                    return Token{ .ansiSequence = result };
                } else if (self.buf_len >= self.buf.len) {
                    // Overflow — reset
                    self.state = .normal;
                    self.buf_len = 0;
                    return null;
                } else {
                    return null;
                }
            },
        }
    }
};
