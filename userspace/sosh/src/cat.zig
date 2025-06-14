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
    std.log.err("{s}cat panic: {s}\n", .{ ANSI_FG_RED, message });
    soos.exit(1);
}

var _entry_ptr: *const soos.types.entry_t = undefined;

export fn _start() callconv(.naked) void {
    // on entry there is a pointer on the stack to the arguments
    asm volatile (
        \\ pop %rdi
        : [entry_pointer] "={rdi}" (_entry_ptr),
        :
        : "rdi", "memory"
    );

    asm volatile (
        \\ call _main
    );
}

export fn _main() noreturn {
    var argBuffer: [512][]const u8 = undefined;
    for (0.._entry_ptr.argc) |i| {
        const entry = _entry_ptr.argv[i];
        argBuffer[i] = entry.ptr[0..entry.len];
    }

    main(argBuffer[0.._entry_ptr.argc]) catch |err| {
        std.log.err("error: {}\n", .{err});
        soos.exit(1);
    };
    soos.exit(0);
}

fn main(args: []const []const u8) !void {
    if (args.len == 1) {
        std.log.err("Usage: cat <file1> <file2> ...", .{});
        return error.InvalidArguments;
    }

    for (args[1..]) |arg| {
        const fd = try soos.open(arg);
        defer {
            soos.close(fd) catch @panic("failed to close file");
        }

        var buffer: [4096]u8 = undefined;
        while (true) {
            const bytes_read = try soos.read(fd, &buffer, true);
            if (bytes_read == 0) break; // EOF

            // Write to stdout
            const bytes_written = try soos.write(1, buffer[0..bytes_read]);
            if (bytes_written != bytes_read) {
                return error.WriteError;
            }
        }
    }
}
