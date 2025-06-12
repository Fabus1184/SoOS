const std = @import("std");

const soos = @import("soos");

const ANSI_RESET = "\x1b[0m";
const ANSI_FG_RED = "\x1b[31m";
const ANSI_FG_GREEN = "\x1b[32m";
const ANSI_FG_YELLOW = "\x1b[33m";
const ANSI_FG_BLUE = "\x1b[34m";
const ANSI_FG_MAGENTA = "\x1b[35m";
const ANSI_FG_CYAN = "\x1b[36m";
const ANSI_FG_WHITE = "\x1b[37m";

pub const std_options = std.Options{
    .page_size_max = 4096,
    .log_level = .debug,
    .logFn = struct {
        fn f(
            comptime message_level: std.log.Level,
            comptime _: @TypeOf(.enum_literal),
            comptime format: []const u8,
            args: anytype,
        ) void {
            const colors = switch (message_level) {
                .debug => ANSI_FG_CYAN,
                .info => ANSI_FG_GREEN,
                .warn => ANSI_FG_YELLOW,
                .err => ANSI_FG_RED,
            };

            soos.print("{s}[{s}] ", .{ colors, @tagName(message_level) });
            soos.print(format, args);
            soos.print("{s}\n", .{ANSI_RESET});
        }
    }.f,
};

const DummyMutex = struct {
    pub fn lock(_: *@This()) void {}
    pub fn unlock(_: *@This()) void {}
};

pub fn panic(message: []const u8, _: ?*std.builtin.StackTrace, _: ?usize) noreturn {
    std.log.err("{s}panic: {s}\n", .{ ANSI_FG_RED, message });
    soos.exit(1);
}

const Position = struct {
    x: u32,
    y: u32,
};

fn drawCursor(position: Position, framebuffer: soos.Framebuffer) void {
    // Simple 16x16 cursor bitmap
    const cursorBitmap: [19]u16 = [_]u16{
        0b1000000000000000,
        0b1100000000000000,
        0b1010000000000000,
        0b1001000000000000,
        0b1000100000000000,
        0b1000010000000000,
        0b1000001000000000,
        0b1000000100000000,
        0b1000000010000000,
        0b1000000001000000,
        0b1000000000100000,
        0b1000000000010000,
        0b1000000011100000,
        0b1000000100000000,
        0b1000100010000000,
        0b1001010010000000,
        0b1010001001000000,
        0b1100001001000000,
        0b0000000110000000,
    };

    // draw mouse cursor
    for (0..19) |dy| {
        for (0..16) |dx| {
            const x = position.x + dx;
            const y = position.y + dy;
            if (x < framebuffer.width and y < framebuffer.height) {
                if (cursorBitmap[dy] & (@as(u16, 1) << @intCast(15 - dx)) != 0) {
                    framebuffer.ptr[y * framebuffer.width + x] = 0xFF_000000;
                }
            }
        }
    }
}

const Window = struct {
    title: []const u8,
    width: u32,
    height: u32,
    position: Position,
    contents: []u32,
    allocator: std.mem.Allocator,

    fn new(allocator: std.mem.Allocator, title: []const u8, width: u32, height: u32) !Window {
        var contents = try allocator.alloc(u32, width * height);

        // initialize 32x32 checkerboard pattern
        for (0..height) |y| {
            for (0..width) |x| {
                const checker_color: u32 = if ((x / 32 + y / 32) % 2 == 0)
                    0xFF_FFFFFF // White
                else
                    0xFF_000000; // Black
                contents[y * width + x] = checker_color;
            }
        }

        return Window{
            .title = title,
            .width = width,
            .height = height,
            .position = Position{ .x = 0, .y = 0 },
            .contents = contents,
            .allocator = allocator,
        };
    }

    fn resize(self: *Window, new_width: u32, new_height: u32) !void {
        var new_contents = try self.allocator.alloc(u32, new_width * new_height);
        // clear new contents
        @memset(new_contents, 0xFF_FFFFFF); // White background
        // copy old contents to new contents
        const min_width = @min(self.width, new_width);
        const min_height = @min(self.height, new_height);
        for (0..min_height) |y| {
            for (0..min_width) |x| {
                new_contents[y * new_width + x] = self.contents[y * self.width + x];
            }
        }
        // free old contents
        self.allocator.free(self.contents);
        // update window properties
        self.contents = new_contents;
        self.width = new_width;
        self.height = new_height;
    }

    fn cursorOverTitlebar(self: Window, mouse_position: Position) bool {
        return mouse_position.x >= self.position.x and
            mouse_position.x < self.position.x + self.width and
            mouse_position.y >= self.position.y and
            mouse_position.y < self.position.y + 20; // Titlebar height
    }

    fn draw(self: Window, framebuffer: soos.Framebuffer) void {
        // draw titlebar
        const titlebar_height = 20;
        for (0..self.width) |x| {
            for (0..titlebar_height) |y_| {
                const y = self.position.y + y_;
                framebuffer.blit(self.position.x + x, y, 0xFF_0000FF); // Blue titlebar
            }
        }

        // draw title text
        // TODO

        // draw border
        const border_color = 0xFF_000000; // Black border
        for (0..self.width) |x| {
            framebuffer.blit(self.position.x + x, self.position.y + titlebar_height, border_color); // Top border
            framebuffer.blit(self.position.x + x, self.position.y + titlebar_height + self.height - 1, border_color); // Bottom border
        }
        for (0..self.height) |y| {
            framebuffer.blit(self.position.x, self.position.y + y + titlebar_height, border_color); // Left border
            framebuffer.blit(self.position.x + self.width - 1, self.position.y + y + titlebar_height, border_color); // Right border
        }

        // draw window contents
        for (0..self.height) |y| {
            const content_y = self.position.y + y + titlebar_height;
            for (0..self.width) |x| {
                const content_x = self.position.x + x;
                framebuffer.blit(content_x, content_y, self.contents[y * self.width + x]);
            }
        }
    }
};

export fn _start() void {
    main() catch |err| {
        std.log.err("Error: {}\n", .{err});
        soos.exit(1);
    };
}

fn main() !void {
    var heap = std.heap.ArenaAllocator.init(soos.pageAllocator());
    const allocator = heap.allocator();

    var windows = std.ArrayList(Window).init(allocator);
    defer windows.deinit();
    defer {
        for (windows.items) |*window| {
            window.allocator.free(window.contents);
        }
    }

    const window1 = try Window.new(allocator, "Test Window", 125, 125);
    try windows.append(window1);

    const framebuffer = soos.mapFramebuffer();
    framebuffer.clear(0xFF_EEEEEE);

    var mousePosition = Position{ .x = framebuffer.width / 2, .y = framebuffer.height / 2 };
    var mouseState: struct {
        left_button: bool = false,
        right_button: bool = false,
    } = .{};

    const fd = soos.open("/dev/mouse") catch unreachable;
    while (true) {
        var mouseBuffer: [16]soos.events.mouse_event_t = undefined;
        const bytesRead = soos.read(fd, @ptrCast(&mouseBuffer)) catch unreachable;

        const events = @as(usize, bytesRead) / @sizeOf(soos.events.mouse_event_t);
        for (mouseBuffer[0..events]) |event| {
            mousePosition.x = @intCast(@max(@as(i33, mousePosition.x) + event.x_movement, 0));
            mousePosition.y = @intCast(@max(@as(i33, mousePosition.y) + event.y_movement, 0));
            mouseState.left_button = event.left_button_pressed == 1;
            mouseState.right_button = event.right_button_pressed == 1;
        }
        mousePosition.x = @min(mousePosition.x, framebuffer.width - 1);
        mousePosition.y = @min(mousePosition.y, framebuffer.height - 1);

        framebuffer.clear(0xFF_EEEEEE);

        for (windows.items) |*window| {
            if (window.cursorOverTitlebar(mousePosition) and mouseState.left_button) {
                // Dragging the window
                window.position.x = @intCast(@max(mousePosition.x - window.width / 2, 0));
                window.position.y = @intCast(@max(mousePosition.y - 20, 0)); // Titlebar height
            }

            window.draw(framebuffer);
        }

        drawCursor(mousePosition, framebuffer);
    }
}
