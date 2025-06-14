const std = @import("std");

const soos = @import("soos");

const zigimg = @import("zigimg");
const zclay = @import("zclay");

const terminal = @import("terminal.zig");
const font = @import("font.zig");
const draw = @import("draw.zig");
const window = @import("window.zig");

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
    std.log.err("{s}sogui panic: {s}", .{ ANSI_FG_RED, message });
    if (stackTrace) |trace| {
        std.log.err("stack trace:", .{});
        for (0.., trace.instruction_addresses) |i, addr| {
            if (addr == 0) break; // Stop at the first zero address

            std.log.err("- {}: {x}", .{ i, addr });
        }
    }
    soos.exit(1);
}

fn drawCursor(position: zclay.Vector2, framebuffer: soos.Framebuffer) void {
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

            const x = @as(usize, @intFromFloat(position.x)) + dx;
            const y = @as(usize, @intFromFloat(position.y)) + dy;
            if (x < framebuffer.width and y < framebuffer.height) {
                if (bit != 0) {
                    framebuffer.ptr[y * framebuffer.width + x] = 0xFF_FFFFFF;
                }
            }
        }
    }
}

export fn _start() callconv(.naked) void {
    asm volatile (
        \\
        \\ call _main
    );
}

export fn _main() noreturn {
    main() catch |err| {
        std.log.err("error: {}\n", .{err});
        soos.exit(1);
    };
    soos.exit(0);
}

fn main() !void {
    var heap = std.heap.ArenaAllocator.init(soos.pageAllocator());
    const allocator = heap.allocator();

    const mainfont = try font.Font.load(allocator);
    defer mainfont.deinit(allocator);

    var windows = std.ArrayList(window.Window).init(allocator);
    defer windows.deinit();
    defer {
        for (windows.items) |*w| {
            w.allocator.free(w.drawBuffer.ptr);
        }
    }

    const framebuffer = soos.mapFramebuffer();

    var drawBuffer = soos.Framebuffer{
        .ptr = try allocator.alignedAlloc(u32, 128 * 4, framebuffer.width * framebuffer.height),
        .width = framebuffer.width,
        .height = framebuffer.height,
    };
    const clayBackend = draw.ClayBufferBackend.init(drawBuffer);

    const size = zclay.minMemorySize();
    const clayBuffer1 = try allocator.alloc(u8, size);
    const clayBuffer2 = try allocator.alloc(u8, size);
    defer allocator.free(clayBuffer1);
    defer allocator.free(clayBuffer2);

    const clayArena1 = zclay.createArenaWithCapacityAndMemory(clayBuffer1);
    const clayArena2 = zclay.createArenaWithCapacityAndMemory(clayBuffer2);

    const clayMainContext = zclay.initialize(clayArena1, zclay.Dimensions{
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
    const clayWindowContext = zclay.initialize(clayArena2, zclay.Dimensions{
        .w = @floatFromInt(500),
        .h = @floatFromInt(400),
    }, zclay.ErrorHandler{
        .error_handler_function = struct {
            fn err(errorData: zclay.ErrorData) callconv(.C) void {
                std.log.err("clay window context error: {s}", .{errorData.error_text.chars[0..@intCast(errorData.error_text.length)]});
            }
        }.err,
        .user_data = null,
    });
    std.log.debug("clay initialized", .{});

    zclay.setCurrentContext(clayMainContext);

    var term = try terminal.Terminal.init(allocator, 800 / 20, 600 / 20, &mainfont);

    const window1 = try window.Window.new(
        allocator,
        "Test Window",
        800,
        600,
        &term,
        clayWindowContext,
        &terminal.Terminal.draw,
    );
    try windows.append(window1);

    const measureTextFn = struct {
        fn f(text: []const u8, _: *zclay.TextElementConfig, _: void) zclay.Dimensions {
            return zclay.Dimensions{
                .w = 20.0 * @as(f32, @floatFromInt(text.len)),
                .h = 20.0,
            };
        }
    }.f;
    zclay.setCurrentContext(clayWindowContext);
    zclay.setMeasureTextFunction(void, {}, measureTextFn);
    zclay.setCurrentContext(clayMainContext);
    zclay.setMeasureTextFunction(void, {}, measureTextFn);

    var mousePosition = zclay.Vector2{ .x = @floatFromInt(framebuffer.width / 2), .y = @floatFromInt(framebuffer.height / 2) };
    var mouseState: struct {
        left_button: bool = false,
        right_button: bool = false,
    } = .{};

    const Pane = enum {
        main,
        log,
    };
    var pane: Pane = .main;
    var logBuffer = try std.RingBuffer.init(allocator, 512);
    defer logBuffer.deinit(allocator);
    const logFd = try soos.open("/var/log");

    const fd = try soos.open("/dev/mouse");
    while (true) {
        var logReadBuffer: [512]u8 = undefined;
        const len = try soos.read(logFd, &logReadBuffer, false);
        logBuffer.writeSliceAssumeCapacity(logReadBuffer[0..len]);

        loop: while (true) {
            var removed = false;
            for (windows.items, 0..) |*w, i| {
                if (w.state == .closed) {
                    _ = windows.orderedRemove(i);
                    removed = true;
                    continue :loop;
                }
            }
            if (!removed) break :loop;
        }

        drawBuffer.clear(0xFF_333333);

        for (windows.items) |*w| {
            w.drawContent();
        }

        zclay.setPointerState(mousePosition, mouseState.left_button);
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

                inline for (&.{ Pane.main, Pane.log }) |p| {
                    zclay.UI()(.{
                        .layout = .{
                            .sizing = .{ .w = .fit, .h = .fit },
                            .padding = .all(8),
                            .child_alignment = .{ .x = .center, .y = .center },
                        },
                        .background_color = if (pane == p) [_]f32{ 70, 70, 70, 255 } else [_]f32{ 60, 60, 60, 255 },
                        .corner_radius = .all(16.0),
                    })({
                        zclay.onHover(*@TypeOf(pane), &pane, struct {
                            fn f(_: zclay.ElementId, pointer: zclay.PointerData, ptr: *@TypeOf(pane)) void {
                                if (pointer.state == .pressed_this_frame) {
                                    ptr.* = p;
                                }
                            }
                        }.f);

                        zclay.text(@tagName(p), .{
                            .color = [_]f32{ 255, 255, 255, 255 },
                        });
                    });
                }
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
                switch (pane) {
                    .main => {
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
                            const str = try std.fmt.bufPrint(&buf, "mouse position: ({d:4}, {d:4})", .{ mousePosition.x, mousePosition.y });
                            zclay.text(str, .{
                                .font_id = 0,
                                .font_size = 16,
                                .color = [_]f32{ 255, 0, 255, 255 },
                            });
                        });

                        for (windows.items) |*w| {
                            w.draw();
                        }
                    },
                    .log => {
                        zclay.text(logBuffer.data, .{
                            .color = [_]f32{ 255, 0, 255, 255 },
                        });
                    },
                }
            });
        });

        const commands = zclay.endLayout();
        clayBackend.draw(&mainfont, commands);

        drawCursor(mousePosition, drawBuffer);

        // Copy the buffer to the framebuffer
        framebuffer.copy(drawBuffer.ptr, 0, 0, drawBuffer.width, drawBuffer.height);

        var mouseBuffer: [16]soos.events.mouse_event_t = undefined;
        const bytesRead = try soos.read(fd, @ptrCast(&mouseBuffer), true);

        const events = @as(usize, bytesRead) / @sizeOf(soos.events.mouse_event_t);
        for (mouseBuffer[0..events]) |event| {
            mousePosition.x = @max(mousePosition.x + @as(f32, @floatFromInt(event.x_movement)), 0);
            mousePosition.y = @max(mousePosition.y + @as(f32, @floatFromInt(event.y_movement)), 0);
            mouseState.left_button = event.left_button_pressed == 1;
            mouseState.right_button = event.right_button_pressed == 1;
        }
        mousePosition.x = @min(mousePosition.x, @as(f32, @floatFromInt(framebuffer.width - 1)));
        mousePosition.y = @min(mousePosition.y, @as(f32, @floatFromInt(framebuffer.height - 1)));
    }
}
