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
    run: *const fn (argc: u32, argv: []const []const u8) void,
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

const commands: []const Command = &[_]Command{
    .{ .name = "help", .run = struct {
        fn help(_: u32, _: []const []const u8) void {
            soos.print("available commands:\n", .{});
            for (commands) |cmd| {
                soos.print("* {s}{s}{s}\n", .{ ANSI_FG_MAGENTA, cmd.name, ANSI_RESET });
            }
        }
    }.help },
    .{ .name = "clear", .run = struct {
        fn clear(_: u32, _: []const []const u8) void {
            reset();
        }
    }.clear },
    .{ .name = "exit", .run = struct {
        fn exit(_: u32, _: []const []const u8) void {
            soos.syscalls.exit(0);
        }
    }.exit },
    .{
        .name = "ls",
        .run = struct {
            fn ls(argc: u32, argv: []const []const u8) void {
                if (argc != 2) {
                    soos.print("usage: ls <directory>\n", .{});
                    return;
                }
                var i: u64 = 0;
                while (true) {
                    var buffer: [512]u8 = undefined;
                    const n = soos.syscalls.listdir(argv[1], i, &buffer);
                    if (n == 0) {
                        if (i == 0) {
                            soos.print("Error: Failed to list directory '{s}'\n", .{argv[1]});
                        }
                        return;
                    }
                    soos.print("{s}\n", .{buffer[0..n]});
                    i += 1;
                }
            }
        }.ls,
    },
    .{
        .name = "fork",
        .run = struct {
            fn fork(_: u32, _: []const []const u8) void {
                const pid = soos.syscalls.fork();
                if (pid == 0) {
                    soos.print("Hello from the child process!\n", .{});
                    soos.syscalls.exit(0);
                } else {
                    soos.print("Forked child process with PID: {d}\n", .{pid});
                }
            }
        }.fork,
    },
    .{
        .name = "cat",
        .run = struct {
            fn cat(argc: u32, argv: []const []const u8) void {
                if (argc != 2) {
                    soos.print("usage: cat <file>\n", .{});
                    return;
                }
                const fd = soos.syscalls.open(argv[1]) orelse {
                    soos.print("Error: Failed to open file '{s}'\n", .{argv[1]});
                    return;
                };
                var buffer: [4096]u8 = undefined;
                var bytes_read: usize = 0;
                while (true) {
                    bytes_read = soos.syscalls.read(fd, &buffer) orelse {
                        soos.print("Error: Failed to read file '{s}'\n", .{argv[1]});
                        return;
                    };
                    if (bytes_read == 0) break; // EOF
                    soos.print("{s}", .{buffer[0..bytes_read]});
                }
            }
        }.cat,
    },
    .{
        .name = "test",
        .run = struct {
            fn test_(_: u32, _: []const []const u8) void {
                const pageAllocator = soos.pageAllocator();
                var heap = std.heap.ArenaAllocator.init(pageAllocator);
                defer heap.deinit();
                var allocator = heap.allocator();
                for (0..100) |i| {
                    soos.print("test iteration {d}, allocating {d} bytes\n", .{ i, 100 + i * 100 });
                    const testPtr = allocator.alloc(u32, 100 + i * 100) catch @panic("Failed to allocate memory");
                    defer allocator.free(testPtr);
                    // fill the allocated memory with some values
                    for (0..100 + i * 100) |j| {
                        testPtr[j] = @intCast(j);
                    }
                    // check if the values are set correctly
                    for (0..100 + i * 100) |j| {
                        if (testPtr[j] != j) {
                            soos.print("{s}Value mismatch at index {d}, expected {d}, got {d}\n", .{ ANSI_FG_RED, j, j, testPtr[j] });
                            soos.syscalls.exit(1);
                        }
                    }
                    soos.print("{s}test iteration {d} passed\n", .{ ANSI_FG_GREEN, i });
                }
            }
        }.test_,
    },
};

fn reset() void {
    soos.print("\x1b[2J\x1b[H{s}\n\n", .{banner});
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
    soos.syscalls.exit(1);
}

export fn _start() void {
    var commandBuffer: [1024]u8 = undefined;
    var commandLength: u64 = 0;

    reset();
    soos.print("{s}_", .{prompt});

    while (true) {
        var inputBuffer: [64]u8 = undefined;
        const inputLength = soos.syscalls.read(0, &inputBuffer) orelse {
            soos.print("Error: Failed to read input\n", .{});
            soos.syscalls.exit(1);
        };
        if (inputLength == 0) {
            soos.syscalls.exit(0);
        }

        soos.print("\x08 \x08", .{});

        for (0..inputLength) |i| {
            switch (inputBuffer[i]) {
                '\x08' => {
                    if (commandLength > 0) {
                        commandLength -= 1;
                        soos.print("\x08 \x08", .{});
                    }
                },
                '\n' => {
                    soos.print("\n", .{});

                    var argc: u32 = 0;
                    var argv: [32][]const u8 = undefined;
                    var it = std.mem.splitScalar(u8, commandBuffer[0..commandLength], ' ');
                    while (it.next()) |word| {
                        argv[argc] = word;
                        argc += 1;
                    }

                    for (commands) |cmd| {
                        if (std.mem.eql(u8, cmd.name, argv[0])) {
                            cmd.run(argc, argv[0..argc]);
                            break;
                        }
                    } else {
                        soos.print("unknown command: '{s}'", .{argv[0]});
                    }

                    soos.print("\n{s}", .{prompt});

                    commandLength = 0;
                    break;
                },
                else => {
                    commandBuffer[commandLength] = inputBuffer[i];
                    commandLength += 1;

                    soos.print("{s}", .{inputBuffer[i .. i + 1]});
                },
            }
        }

        soos.print("_", .{});
    }
}
