const std = @import("std");

const zclay = @import("zclay");

const soos = @import("soos");
const backend = @import("draw.zig");
const font = @import("font.zig");

pub const Terminal = struct {
    width: usize,
    height: usize,
    contents: []u8,

    cursor: struct {
        x: usize,
        y: usize,
    },

    mainfont: *const font.Font,

    childStdin: i32,
    childStdout: i32,

    keyboardFd: i32,

    pub fn init(allocator: std.mem.Allocator, width: usize, height: usize, mainfont: *const font.Font) !Terminal {
        const contents = try allocator.alloc(u8, width * height);
        for (contents) |*c| c.* = ' ';

        const str = "Welcome to SoOS!\n";
        @memcpy(contents[0..str.len], str);

        const child = soos.fork();
        if (child == 0) {
            soos.execve("/bin/sosh", &.{}) catch |err| {
                std.log.err("failed to exec sosh: {}", .{err});
                @panic("failed to exec sosh");
            };
        }

        var buf: [64]u8 = undefined;
        const childStdinPath = try std.fmt.bufPrint(&buf, "/proc/{d}/stdin", .{child});
        const childStdin = try soos.open(childStdinPath);

        const childStdoutPath = try std.fmt.bufPrint(&buf, "/proc/{d}/stdout", .{child});
        const childStdout = try soos.open(childStdoutPath);

        const keyboardFd = try soos.open("/dev/keyboard");

        return Terminal{
            .width = width,
            .height = height,
            .contents = contents,
            .cursor = .{ .x = 0, .y = 0 },
            .mainfont = mainfont,
            .childStdin = childStdin,
            .childStdout = childStdout,
            .keyboardFd = keyboardFd,
        };
    }

    pub fn draw(self_: *anyopaque, drawBuffer: soos.Framebuffer) void {
        const self: *Terminal = @alignCast(@ptrCast(self_));

        var readBuffer: [512]u8 = undefined;
        const len = soos.read(self.childStdout, &readBuffer, false) catch |err| {
            std.log.err("failed to read from child stdout: {}", .{err});
            return;
        };
        if (len > 0) {
            std.mem.copyForwards(u8, self.contents[0 .. self.contents.len - len], self.contents[len..]);
            @memcpy(self.contents[self.contents.len - len ..], readBuffer[0..len]);
        }

        var keyboardInput: [128]u8 = undefined;
        const keyboardLen = soos.read(self.keyboardFd, &keyboardInput, false) catch |err| {
            std.log.err("failed to read from keyboard: {}", .{err});
            return;
        };
        if (keyboardLen > 0) {
            _ = soos.write(self.childStdin, keyboardInput[0..keyboardLen]) catch |err| {
                std.log.err("failed to write to child stdin: {}", .{err});
                return;
            };
        }

        zclay.beginLayout();

        zclay.UI()(zclay.ElementDeclaration{
            .layout = .{ .direction = .top_to_bottom },
        })({
            for (0..self.height) |y| {
                const line = self.contents[(y * self.width)..((y + 1) * self.width)];
                zclay.text(line, .{
                    .color = .{ 0, 255, 0, 255 },
                });
            }
        });

        const b = backend.ClayBufferBackend.init(drawBuffer);
        const commands = zclay.endLayout();
        b.draw(self.mainfont, commands);
    }
};
