const std = @import("std");

const soos = @import("soos");

const banner =
    ANSI_FG_CYAN ++
    \\
    \\  /$$$$$$             /$$$$$$  /$$   /$$
    \\ /$$__  $$           /$$__  $$| $$  | $$
    \\| $$  \__/  /$$$$$$ | $$  \__/| $$  | $$
    \\|  $$$$$$  /$$__  $$|  $$$$$$ | $$$$$$$$
    \\ \____  $$| $$  \ $$ \____  $$| $$__  $$
    \\ /$$  \ $$| $$  | $$ /$$  \ $$| $$  | $$
    \\|  $$$$$$/|  $$$$$$/|  $$$$$$/| $$  | $$
    \\ \______/  \______/  \______/ |__/  |__/
    \\
    \\          - the SoOS shell -
    ++ ANSI_RESET;

const Command = struct {
    name: []const u8,
    run: *const fn (argc: u32, argv: []const []const u8) anyerror!void,
};

const ANSI_RESET = "\x1b[0m";
const ANSI_FG_RED = "\x1b[31m";
const ANSI_FG_GREEN = "\x1b[32m";
const ANSI_FG_YELLOW = "\x1b[33m";
const ANSI_FG_BLUE = "\x1b[34m";
const ANSI_FG_MAGENTA = "\x1b[35m";
const ANSI_FG_CYAN = "\x1b[36m";
const ANSI_FG_WHITE = "\x1b[37m";

const prompt = ANSI_FG_GREEN ++ "sosh> " ++ ANSI_RESET;

fn print(comptime message: []const u8, args: anytype) void {
    var buffer: [4096]u8 = undefined;
    const str = std.fmt.bufPrint(&buffer, message, args) catch @panic("Failed to format message");
    const len = soos.write(1, str) catch @panic("Failed to write message");
    if (len != str.len) {
        std.log.err("Failed to write all bytes: expected {d}, got {d}\n", .{ str.len, len });
        @panic("Failed to write all bytes");
    }
}

const commands: []const Command = &[_]Command{
    .{ .name = "help", .run = struct {
        fn help(_: u32, _: []const []const u8) !void {
            print("available commands:\n", .{});
            for (commands) |cmd| {
                print("* {s}{s}{s}\n", .{ ANSI_FG_MAGENTA, cmd.name, ANSI_RESET });
            }
        }
    }.help },
    .{ .name = "clear", .run = struct {
        fn clear(_: u32, _: []const []const u8) !void {
            reset();
        }
    }.clear },
    .{ .name = "exit", .run = struct {
        fn exit(_: u32, _: []const []const u8) !void {
            soos.exit(0);
        }
    }.exit },
    .{
        .name = "ls",
        .run = struct {
            fn ls(argc: u32, argv: []const []const u8) !void {
                if (argc != 2) {
                    print("usage: ls <directory>\n", .{});
                    return;
                }
                var listDir = soos.listdir(argv[1]) catch |err| {
                    print("Error: Failed to list directory '{s}': {}\n", .{ argv[1], err });
                    return;
                };
                while (listDir.next()) |entry| {
                    switch (entry.type) {
                        .file => print("{s}{s}\n", .{ ANSI_FG_BLUE, entry.name }),
                        .directory => print("{s}{s}/\n", .{ ANSI_FG_CYAN, entry.name }),
                    }
                }
            }
        }.ls,
    },
    .{
        .name = "fork",
        .run = struct {
            fn fork(_: u32, _: []const []const u8) !void {
                const pid = soos.fork();
                if (pid == 0) {
                    print("Hello from the child process!\n", .{});
                    soos.exit(0);
                } else {
                    print("Forked child process with PID: {d}\n", .{pid});
                }
            }
        }.fork,
    },
    .{
        .name = "test",
        .run = struct {
            fn test_(_: u32, _: []const []const u8) !void {
                const pageAllocator = soos.pageAllocator();
                var heap = std.heap.ArenaAllocator.init(pageAllocator);
                defer heap.deinit();
                var allocator = heap.allocator();
                for (0..100) |i| {
                    print("test iteration {d}, allocating {d} bytes\n", .{ i, 100 + i * 100 });
                    const testPtr = allocator.alloc(u32, 100 + i * 100) catch @panic("Failed to allocate memory");
                    defer allocator.free(testPtr);
                    // fill the allocated memory with some values
                    for (0..100 + i * 100) |j| {
                        testPtr[j] = @intCast(j);
                    }
                    // check if the values are set correctly
                    for (0..100 + i * 100) |j| {
                        if (testPtr[j] != j) {
                            print("{s}Value mismatch at index {d}, expected {d}, got {d}\n", .{ ANSI_FG_RED, j, j, testPtr[j] });
                            soos.exit(1);
                        }
                    }
                    print("{s}test iteration {d} passed\n", .{ ANSI_FG_GREEN, i });
                }
            }
        }.test_,
    },
    .{
        .name = "gui",
        .run = struct {
            fn exec(argc: u32, _: []const []const u8) !void {
                if (argc != 1) {
                    print("usage: gui\n", .{});
                    return;
                }
                const framebuffer = soos.mapFramebuffer();
                // Clear framebuffer
                for (0..framebuffer.height) |y| {
                    for (0..framebuffer.width) |x| {
                        framebuffer.ptr[y * framebuffer.width + x] = 0xFF_FFFFFF; // White background
                    }
                }
                var mousePosition: struct { x: u32, y: u32 } = .{ .x = 0, .y = 0 };
                const cursorColor: u32 = 0xFF_000000; // Blue cursor
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
                const fd = try soos.open("/dev/mouse");
                while (true) {
                    var mouseBuffer: [16]soos.events.mouse_event_t = undefined;
                    const bytesRead = try soos.read(fd, @ptrCast(&mouseBuffer), true);
                    // clear previous mouse cursor
                    for (0..19) |dy| {
                        for (0..16) |dx| {
                            const x = mousePosition.x + dx;
                            const y = mousePosition.y + dy;
                            if (x < framebuffer.width and y < framebuffer.height) {
                                framebuffer.ptr[y * framebuffer.width + x] = 0xFF_FFFFFF; // Restore background color
                            }
                        }
                    }
                    const events = @as(usize, bytesRead) / @sizeOf(soos.events.mouse_event_t);
                    for (mouseBuffer[0..events]) |event| {
                        mousePosition.x = @intCast(@max(@as(i33, mousePosition.x) + event.x_movement, 0));
                        mousePosition.y = @intCast(@max(@as(i33, mousePosition.y) + event.y_movement, 0));
                    }
                    // draw mouse cursor
                    for (0..19) |dy| {
                        for (0..16) |dx| {
                            const x = mousePosition.x + dx;
                            const y = mousePosition.y + dy;
                            if (x < framebuffer.width and y < framebuffer.height) {
                                framebuffer.ptr[y * framebuffer.width + x] = if (cursorBitmap[dy] & (@as(u16, 1) << @intCast(15 - dx)) != 0) cursorColor else 0xFF_FFFFFF; // Draw cursor pixel or restore background
                            }
                        }
                    }
                }
            }
        }.exec,
    },
};

fn reset() void {
    print("\x1b[2J\x1b[H{s}\n\n", .{banner});
}

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

            print("{s}[{s}] ", .{ colors, @tagName(message_level) });
            print(format, args);
            print("{s}\n", .{ANSI_RESET});
        }
    }.f,
};

const DummyMutex = struct {
    pub fn lock(_: *@This()) void {}
    pub fn unlock(_: *@This()) void {}
};

pub fn panic(message: []const u8, _: ?*std.builtin.StackTrace, _: ?usize) noreturn {
    std.log.err("{s}sosh panic: {s}\n", .{ ANSI_FG_RED, message });
    soos.exit(1);
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
    var commandBuffer: [1024]u8 = undefined;
    var commandLength: u64 = 0;

    reset();
    print("{s}_", .{prompt});

    while (true) {
        var inputBuffer: [64]u8 = undefined;
        const inputLength = try soos.read(0, &inputBuffer, true);
        if (inputLength == 0) {
            @panic("EOF reached, exiting shell");
        }

        print("\x08 \x08", .{});

        for (0..inputLength) |i| {
            switch (inputBuffer[i]) {
                '\x08' => {
                    if (commandLength > 0) {
                        commandLength -= 1;
                        print("\x08 \x08", .{});
                    }
                },
                '\n' => {
                    print("\n", .{});

                    var argc: u32 = 0;
                    var argv: [32][]const u8 = undefined;
                    var it = std.mem.splitScalar(u8, commandBuffer[0..commandLength], ' ');
                    while (it.next()) |word| {
                        argv[argc] = word;
                        argc += 1;
                    }

                    for (commands) |cmd| {
                        if (std.mem.eql(u8, cmd.name, argv[0])) {
                            cmd.run(argc, argv[0..argc]) catch |err| {
                                print("error: command '{s}' failed: {}\n", .{ cmd.name, err });
                            };
                            break;
                        }
                    } else {
                        var binList = try soos.listdir("/bin");
                        while (binList.next()) |entry| {
                            if (entry.type != .file) continue;
                            if (std.mem.eql(u8, entry.name, argv[0])) {
                                const pid = soos.fork();
                                if (pid == 0) {
                                    var filename: [256]u8 = undefined;
                                    const str = try std.fmt.bufPrint(&filename, "/bin/{s}", .{entry.name});
                                    soos.execve(str, argv[0..argc]) catch |err| {
                                        print("error: failed to execute '{s}': {}\n", .{ entry.name, err });
                                    };
                                } else {
                                    _ = try soos.waitpid(pid);
                                }
                                break;
                            }
                        } else {
                            print("unknown command: '{s}'", .{argv[0]});
                        }
                    }

                    print("\n{s}", .{prompt});

                    commandLength = 0;
                    break;
                },
                else => {
                    commandBuffer[commandLength] = inputBuffer[i];
                    commandLength += 1;

                    print("{s}", .{inputBuffer[i .. i + 1]});
                },
            }
        }

        print("_", .{});
    }
}
