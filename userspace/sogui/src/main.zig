const std = @import("std");

const soos = @import("soos");

const zigimg = @import("zigimg");
const zclay = @import("zclay");

const terminal = @import("terminal.zig");

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

pub fn panic(message: []const u8, stackTrace: ?*std.builtin.StackTrace, _: ?usize) noreturn {
    std.log.err("{s}panic: {s}", .{ ANSI_FG_RED, message });
    if (stackTrace) |trace| {
        std.log.err("stack trace:", .{});
        for (0.., trace.instruction_addresses) |i, addr| {
            if (addr == 0) break; // Stop at the first zero address

            std.log.err("- {}: {x}", .{ i, addr });
        }
    }
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
            const bit = cursorBitmap[dy] & (@as(u16, 1) << @intCast(15 - dx));

            const x = position.x + dx;
            const y = position.y + dy;
            if (x < framebuffer.width and y < framebuffer.height) {
                if (bit != 0) {
                    framebuffer.ptr[y * framebuffer.width + x] = 0xFF_FFFFFF;
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
    state: enum {
        normal,
        minimized,
        maximized,
        closed,
    } = .normal,

    dragStartOffset: ?zclay.Vector2 = null,

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
            .position = Position{ .x = 100, .y = 100 },
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

    fn draw(self: *Window) void {
        zclay.UI()(zclay.ElementDeclaration{
            .layout = zclay.LayoutConfig{
                .direction = .top_to_bottom,
                .sizing = .{ .w = .fixed(@floatFromInt(self.width)), .h = .fixed(@floatFromInt(self.height)) },
                .child_alignment = .{ .x = .left, .y = .top },
            },
            .floating = .{
                .attach_to = .to_root,
                .offset = .{
                    .x = @floatFromInt(self.position.x),
                    .y = @floatFromInt(self.position.y),
                },
            },
            .background_color = [_]f32{ 255, 0, 255, 255 }, // Dark gray background
        })({
            // draw titlebar
            zclay.UI()(.{
                .layout = .{
                    .direction = .left_to_right,
                    .sizing = .{ .w = .grow, .h = .fixed(40.0) },
                    .padding = .all(4.0),
                    .child_alignment = .{ .x = .left, .y = .center },
                    .child_gap = 0,
                },
                .background_color = [_]f32{ 0.0, 0.0, 50, 255 }, // Blue titlebar
            })({
                zclay.onHover(*Window, self, struct {
                    fn f(
                        _: zclay.ElementId,
                        pointer_data: zclay.PointerData,
                        self_: *Window,
                    ) void {
                        if (pointer_data.state == .pressed) {
                            if (self_.dragStartOffset) |offset| {
                                // Continue dragging
                                self_.position.x = @as(u32, @intFromFloat(pointer_data.position.x - offset.x));
                                self_.position.y = @as(u32, @intFromFloat(pointer_data.position.y - offset.y));
                            } else {
                                // Start dragging
                                self_.dragStartOffset = .{
                                    .x = pointer_data.position.x - @as(f32, @floatFromInt(self_.position.x)),
                                    .y = pointer_data.position.y - @as(f32, @floatFromInt(self_.position.y)),
                                };
                            }
                        } else {
                            // Stop dragging
                            self_.dragStartOffset = null;
                        }
                    }
                }.f);

                zclay.UI()(.{
                    .layout = .{
                        .direction = .left_to_right,
                        .sizing = .{ .w = .grow, .h = .fit },
                    },
                })({
                    zclay.text(self.title, .{
                        .font_id = 0,
                        .font_size = 16,
                        .color = [_]f32{ 255, 255, 255, 255 }, // White text
                    });
                });

                // buttons at end
                zclay.UI()(.{
                    .layout = .{
                        .direction = .left_to_right,
                        .sizing = .{ .w = .grow, .h = .fit },
                        .child_alignment = .{ .x = .right, .y = .center },
                        .child_gap = 8,
                    },
                })({
                    zclay.text("[_]", .{
                        .font_id = 0,
                        .font_size = 16,
                        .color = [_]f32{ 0, 255, 0, 255 }, // Green minimize button
                    });
                    zclay.text("[+]", .{
                        .font_id = 0,
                        .font_size = 16,
                        .color = [_]f32{ 0, 0, 255, 255 }, // Blue maximize button
                    });
                    zclay.UI()(.{})({
                        zclay.onHover(*Window, self, struct {
                            fn f(
                                _: zclay.ElementId,
                                pointer_data: zclay.PointerData,
                                self_: *Window,
                            ) void {
                                if (pointer_data.state == .pressed_this_frame) {
                                    self_.state = .closed;
                                }
                            }
                        }.f);
                        zclay.text("[X]", .{
                            .font_id = 0,
                            .font_size = 16,
                            .color = [_]f32{ 255, 0, 0, 255 }, // Red close button
                        });
                    });
                });
            });

            zclay.UI()(.{
                .layout = .{
                    .direction = .top_to_bottom,
                    .sizing = .{ .w = .grow, .h = .grow },
                    .padding = .all(8),
                    .child_alignment = .{ .x = .left, .y = .top },
                    .child_gap = 8,
                },
                .custom = zclay.CustomElementConfig{
                    .custom_data = self,
                },
            })({});
        });
    }
};

export fn _start() callconv(.naked) void {
    asm volatile (
        \\
        \\ call _main
    );
}

export fn _main() noreturn {
    main() catch |err| {
        std.log.err("Error: {}\n", .{err});
        soos.exit(1);
    };
    soos.exit(0);
}

fn main() !void {
    var heap = std.heap.ArenaAllocator.init(soos.pageAllocator());
    const allocator = heap.allocator();

    const font = try Font.load(allocator);
    defer font.deinit(allocator);

    var windows = std.ArrayList(Window).init(allocator);
    defer windows.deinit();
    defer {
        for (windows.items) |*window| {
            window.allocator.free(window.contents);
        }
    }

    const framebuffer = soos.mapFramebuffer();

    const clayBuffer = try allocator.alloc(u8, zclay.minMemorySize());
    defer allocator.free(clayBuffer);
    const clayArena = zclay.createArenaWithCapacityAndMemory(clayBuffer);

    const context = zclay.initialize(clayArena, zclay.Dimensions{
        .w = @floatFromInt(framebuffer.width),
        .h = @floatFromInt(framebuffer.height),
    }, zclay.ErrorHandler{
        .error_handler_function = struct {
            fn err(errorData: zclay.ErrorData) callconv(.C) void {
                std.log.err("clay error: {s}", .{errorData.error_text.chars[0..@intCast(errorData.error_text.length)]});
            }
        }.err,
        .user_data = null,
    });
    std.log.debug("clay initialized", .{});
    _ = .{context};

    zclay.setMeasureTextFunction(void, {}, struct {
        fn f(text: []const u8, _: *zclay.TextElementConfig, _: void) zclay.Dimensions {
            return zclay.Dimensions{
                .w = 20.0 * @as(f32, @floatFromInt(text.len)),
                .h = 20.0,
            };
        }
    }.f);

    const window1 = try Window.new(allocator, "Test Window", 500, 400);
    try windows.append(window1);

    const drawBuffer = soos.Framebuffer{
        .ptr = try allocator.alignedAlloc(u32, 128 * 4, framebuffer.width * framebuffer.height),
        .width = framebuffer.width,
        .height = framebuffer.height,
    };

    var mousePosition = Position{ .x = framebuffer.width / 2, .y = framebuffer.height / 2 };
    var mouseState: struct {
        left_button: bool = false,
        right_button: bool = false,
    } = .{};

    const fd = soos.open("/dev/mouse") catch unreachable;
    while (true) {
        loop: while (true) {
            var removed = false;
            for (windows.items, 0..) |*window, i| {
                if (window.state == .closed) {
                    _ = windows.orderedRemove(i);
                    removed = true;
                    continue :loop;
                }
            }
            if (!removed) break :loop;
        }

        drawBuffer.clear(0xFF_333333);

        {
            zclay.setPointerState(.{
                .x = @floatFromInt(mousePosition.x),
                .y = @floatFromInt(mousePosition.y),
            }, mouseState.left_button);

            zclay.beginLayout();
            zclay.UI()(.{
                .layout = .{
                    .direction = .top_to_bottom,
                    .sizing = .{ .w = .grow, .h = .grow },
                    .child_alignment = .{ .x = .left, .y = .top },
                    .child_gap = 16,
                },
                .background_color = [_]f32{ 20, 20, 20, 255 },
            })({
                zclay.UI()(.{
                    .layout = .{
                        .direction = .left_to_right,
                        .sizing = .{ .w = .grow, .h = .fixed(50) },
                        .padding = .all(8),
                        .child_alignment = .{ .x = .left, .y = .center },
                        .child_gap = 8,
                    },
                    .background_color = [_]f32{ 0, 10, 0, 255 },
                })({
                    zclay.text("SoOS", .{
                        .font_id = 0,
                        .font_size = 24,
                        .color = [_]f32{ 255, 255, 255, 255 },
                    });
                });

                zclay.UI()(.{
                    .layout = .{
                        .direction = .top_to_bottom,
                        .sizing = .{ .w = .grow, .h = .grow },
                        .padding = .all(16),
                        .child_alignment = .{ .x = .left, .y = .top },
                        .child_gap = 16,
                    },
                })({
                    zclay.UI()(.{
                        .layout = .{
                            .direction = .left_to_right,
                            .sizing = .{ .w = .fit, .h = .fit },
                            .padding = .all(8),
                            .child_alignment = .{ .x = .center, .y = .center },
                            .child_gap = 8,
                        },
                        .background_color = if (zclay.hovered()) [_]f32{ 70, 70, 70, 255 } else [_]f32{ 60, 60, 60, 255 },
                        .corner_radius = .all(16.0),
                    })({
                        var buf: [64]u8 = undefined;
                        const str = try std.fmt.bufPrint(&buf, "mouse position: ({:4}, {:4})", .{ mousePosition.x, mousePosition.y });
                        zclay.text(str, .{
                            .font_id = 0,
                            .font_size = 16,
                            .color = [_]f32{ 255, 0, 255, 255 },
                        });
                    });

                    for (windows.items) |*window| {
                        window.draw();
                    }
                });
            });

            const commands = zclay.endLayout();
            for (0..commands.length) |i| {
                const command = commands.internal_array[i];
                switch (command.command_type) {
                    .none => {},
                    .text => {
                        const color = @as(u32, @intFromFloat(command.render_data.text.text_color[3])) << 24 |
                            @as(u32, @intFromFloat(command.render_data.text.text_color[0])) << 16 |
                            @as(u32, @intFromFloat(command.render_data.text.text_color[1])) << 8 |
                            @as(u32, @intFromFloat(command.render_data.text.text_color[2]));
                        const str = command.render_data.text.string_contents.chars[0..@intCast(command.render_data.text.string_contents.length)];
                        font.blitText(color, str, @intFromFloat(command.bounding_box.x), @intFromFloat(command.bounding_box.y), 2, drawBuffer);
                    },
                    .rectangle => {
                        drawBuffer.fillRect(@intFromFloat(command.bounding_box.x), @intFromFloat(command.bounding_box.y), @intFromFloat(command.bounding_box.width), @intFromFloat(command.bounding_box.height), @as(u32, @intFromFloat(command.render_data.rectangle.background_color[3])) << 24 |
                            @as(u32, @intFromFloat(command.render_data.rectangle.background_color[0])) << 16 |
                            @as(u32, @intFromFloat(command.render_data.rectangle.background_color[1])) << 8 |
                            @as(u32, @intFromFloat(command.render_data.rectangle.background_color[2])), [_]usize{
                            @intFromFloat(command.render_data.rectangle.corner_radius.top_left),
                            @intFromFloat(command.render_data.rectangle.corner_radius.top_right),
                            @intFromFloat(command.render_data.rectangle.corner_radius.bottom_left),
                            @intFromFloat(command.render_data.rectangle.corner_radius.bottom_right),
                        });
                    },
                    .custom => {
                        if (command.render_data.custom.custom_data) |windowPtr| {
                            const window: *Window = @alignCast(@ptrCast(windowPtr));
                            // copy window contents
                            const height: usize = @intFromFloat(command.bounding_box.height);
                            const width: usize = @intFromFloat(command.bounding_box.width);
                            const x: usize = @intFromFloat(command.bounding_box.x);
                            const y: usize = @intFromFloat(command.bounding_box.y);
                            for (0..height) |y_| {
                                @memcpy(
                                    drawBuffer.ptr[(y + y_) * drawBuffer.width + x .. (y + y_) * drawBuffer.width + x + width],
                                    window.contents[(y_ * window.width)..(y_ * window.width + width)],
                                );
                            }
                        } else {
                            std.log.err("custom command with no custom data", .{});
                        }
                    },
                    else => |cmd| {
                        std.log.err("unhandled command type: {s}", .{@tagName(cmd)});
                        @panic("unhandled command type");
                    },
                }
            }
        }

        drawCursor(mousePosition, drawBuffer);

        // Copy the buffer to the framebuffer
        framebuffer.copy(drawBuffer.ptr, 0, 0, drawBuffer.width, drawBuffer.height);

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
    }
}

const Font = struct {
    const FONT: []const u8 = @embedFile("font.png");

    chars: [][40 * 40]u32 = undefined,

    fn load(allocator: std.mem.Allocator) !Font {
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

    fn deinit(self: Font, allocator: std.mem.Allocator) void {
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

    fn blitText(
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
