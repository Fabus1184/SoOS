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

    width: u32 = 40,
    height: u32 = 20,

    random: std.Random,
    apple: struct { u32, u32 } = .{ 20, 10 },

    pub fn processInput(self: *Snake, input: u8) void {
        const newDirection: @TypeOf(self.direction) = switch (input) {
            'w' => .up,
            's' => .down,
            'a' => .left,
            'd' => .right,
            else => return,
        };

        if ((self.direction == .up and newDirection != .down) or
            (self.direction == .down and newDirection != .up) or
            (self.direction == .left and newDirection != .right) or
            (self.direction == .right and newDirection != .left))
        {
            self.direction = newDirection;
        }
    }

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

        if (newHead[0] == 0 or newHead[0] == self.width or newHead[1] == 0 or newHead[1] == self.height + 1) {
            // clear the screen
            print("\x1b[2J\x1b[H", .{});
            print("\x1b[{};1H{s}Game Over! You hit the wall!\n", .{ 12, ANSI_FG_RED });
            print("\x1b[{};1H{s}Final Score: {d}{s}\n", .{ 13, ANSI_FG_CYAN, self.len - 4, ANSI_RESET });
            soos.exit(1);
        }

        // check for self-collision
        for (self.positions[1..self.len]) |pos| {
            if (std.meta.eql(pos, newHead)) {
                // clear the screen
                print("\x1b[2J\x1b[H", .{});
                print("\x1b[{};1H{s}Game Over! You ran into yourself!\n", .{ 12, ANSI_FG_RED });
                print("\x1b[{};1H{s}Final Score: {d}{s}\n", .{ 13, ANSI_FG_CYAN, self.len - 4, ANSI_RESET });
                soos.exit(1);
            }
        }

        // check if the snake has eaten the apple
        if (std.meta.eql(self.positions[0], self.apple)) {
            self.positions[self.len] = self.positions[self.len - 1];
            self.len += 1;

            findApple: while (true) {
                const newApple = .{
                    self.random.intRangeAtMost(u32, 1, self.width - 1),
                    self.random.intRangeAtMost(u32, 1, self.height),
                };

                // check if the new apple position collides with the snake
                for (self.positions[0..self.len]) |pos| {
                    if (std.meta.eql(pos, newApple)) {
                        continue; // apple collides with snake, try again
                    } else {
                        self.apple = newApple;
                        break :findApple; // found a valid apple position
                    }
                }
            }

            if (self.len >= self.positions.len) {
                // snake is too long, reset the game
                print("\x1b[2J\x1b[H", .{});
                print("\x1b[{};1H{s}You win! The snake is too long!\n", .{ 12, ANSI_FG_GREEN });
                print("\x1b[{};1H{s}Final Score: {d}{s}\n", .{ 13, ANSI_FG_CYAN, self.len - 4, ANSI_RESET });
                soos.exit(1);
            }
        }
    }

    pub fn initDraw(self: Snake) void {
        // clear the screen
        print("\x1b[2J\x1b[H", .{});

        // draw walls
        print("{s}", .{ANSI_FG_MAGENTA});
        for (0..self.width) |x| {
            print("\x1b[{};{}H==", .{ 1, (x * 2) + 1 });
            print("\x1b[{};{}H==", .{ self.height + 2, (x * 2) + 1 });
        }
        for (0..self.height + 2) |y| {
            print("\x1b[{};1H||", .{y + 1});
            print("\x1b[{};{}H||", .{ y + 1, (self.width * 2) + 1 });
        }
        print("{s}", .{ANSI_RESET});
    }

    pub fn clearDirty(self: Snake) void {
        // clear the last snake position
        const pos = self.positions[self.len - 1];
        print("\x1b[{};{}H  ", .{ pos[1] + 1, (pos[0] * 2) + 1 });

        // clear the apple position
        print("\x1b[{};{}H  ", .{ self.apple[1] + 1, (self.apple[0] * 2) + 1 });
    }

    pub fn redrawDirty(self: Snake) void {
        // draw the snake
        for (self.positions[1..self.len]) |pos| {
            print("\x1b[{};{}H{s}[]", .{ pos[1] + 1, (pos[0] * 2) + 1, ANSI_FG_BLUE });
        }

        // draw the head
        print("\x1b[{};{}H{s}$${s}", .{ self.positions[0][1] + 1, (self.positions[0][0] * 2) + 1, ANSI_FG_GREEN, ANSI_RESET });

        // draw the apple
        print("\x1b[{};{}H{s}(){s}", .{ self.apple[1] + 1, (self.apple[0] * 2) + 1, ANSI_FG_RED, ANSI_RESET });

        // print points
        print("\x1b[{};1H{s}Points: {d}{s}", .{ self.height + 3, ANSI_FG_CYAN, self.len - 4, ANSI_RESET });
    }
};

fn main(_: []const []const u8) !void {
    var prng = std.Random.DefaultPrng.init(0);

    var snake = Snake{ .random = prng.random() };

    snake.initDraw();

    while (true) {
        var inputBuffer: [64]u8 = undefined;
        const count = try soos.read(0, &inputBuffer, false);
        for (inputBuffer[0..count]) |c| {
            snake.processInput(c);
        }

        snake.clearDirty();
        snake.update();
        snake.redrawDirty();

        soos.sleep(500);
    }
}

fn print(comptime message: []const u8, args: anytype) void {
    var buffer: [4096]u8 = undefined;
    const str = std.fmt.bufPrint(&buffer, message, args) catch @panic("Failed to format message");
    const len = soos.write(1, str) catch @panic("Failed to write message");
    if (len != str.len) {
        std.log.err("Failed to write all bytes: expected {d}, got {d}\n", .{ str.len, len });
        @panic("Failed to write all bytes");
    }
}
