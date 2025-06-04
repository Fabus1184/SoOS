const std = @import("std");

const soos = @import("soos");

const prompt = "sosh> ";

const Command = struct {
    name: []const u8,
    run: *const fn (argc: u32, argv: []const []const u8) void,
};

const commands: []const Command = &[_]Command{
    .{ .name = "help", .run = struct {
        fn help(_: u32, _: []const []const u8) void {
            soos.print("Available commands:\n", .{});
            for (commands) |cmd| {
                soos.print("  {s}\n", .{cmd.name});
            }
        }
    }.help },
};

export fn _start() void {
    soos.print("\x1b[2J\x1b[H{s}_", .{prompt});

    var commandBuffer: [1024]u8 = undefined;
    var commandLength: u64 = 0;

    while (true) {
        var inputBuffer: [64]u8 = undefined;
        const inputLength = soos.syscalls.read(0, &inputBuffer);
        if (inputLength == 0) {
            soos.print("Error: Failed to read input\n", .{});
            soos.syscalls.exit(1);
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
                        soos.print("Unknown command: {s}", .{argv[0]});
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
