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

const Snake = struct {
    positions: [32]struct { u32, u32 } = .{ .{ 10, 10 }, .{ 10, 11 }, .{ 10, 12 }, .{ 10, 13 } } ++ .{undefined} ** 28,
    len: usize = 4,
    direction: enum { up, down, left, right } = .right,

    pub fn update(self: *Snake) void {
        // update the snake's position based on its direction
        const head = self.positions[0];
        const newHead = switch (self.direction) {
            .up => .{ head[0], head[1] - 1 },
            .down => .{ head[0], head[1] + 1 },
            .left => .{ head[0] - 1, head[1] },
            .right => .{ head[0] + 1, head[1] },
        };

        std.mem.copyBackwards(@TypeOf(self.positions[0]), self.positions[1..self.len], self.positions[0 .. self.len - 1]);
        self.positions[0] = newHead;

        if (newHead[0] < 0 or newHead[0] >= 20 or newHead[1] < 0 or newHead[1] >= 20) {
            std.log.err("{s}Game Over: Hit the wall!\n", .{ANSI_FG_RED});
            soos.exit(1);
        }
    }

    pub fn draw(self: Snake) void {
        // clear the screen
        soos.print("\x1b[2J\x1b[H", .{});

        // draw walls
        for (0..20) |i| {
            soos.print("\x1b[{};{}H{s}██", .{ (i * 2) + 1, 1, ANSI_FG_WHITE });
            soos.print("\x1b[{};{}H{s}██", .{ (i * 2) + 1, 21, ANSI_FG_WHITE });
        }
        for (0..21) |i| {
            soos.print("\x1b[{};{}H{s}██", .{ 1, i + 1, ANSI_FG_WHITE });
            soos.print("\x1b[{};{}H{s}██", .{ 41, i + 1, ANSI_FG_WHITE });
        }

        // print points
        soos.print("\x1b[1;1H{s}points: {}", .{ ANSI_FG_YELLOW, self.len - 3 });

        // draw the snake
        for (self.positions[1..self.len]) |pos| {
            soos.print("\x1b[{};{}H{s}██", .{ (pos[0] * 2) + 1, pos[1] + 1, ANSI_FG_BLUE });
        }
        // draw the head
        soos.print("\x1b[{};{}H{s}██", .{ (self.positions[0][0] * 2) + 1, self.positions[0][1] + 1, ANSI_FG_GREEN });
    }
};

fn main(_: []const []const u8) !void {
    var snake = Snake{};

    while (true) {
        var inputBuffer: [64]u8 = undefined;
        const count = try soos.read(0, &inputBuffer, false);
        for (inputBuffer[0..count]) |c| {
            snake.direction = switch (c) {
                'w' => .up,
                's' => .down,
                'a' => .left,
                'd' => .right,
                else => continue,
            };
        }

        snake.update();
        snake.draw();

        soos.sleep(500);
    }
}
