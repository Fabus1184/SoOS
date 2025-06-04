const std = @import("std");
pub const syscalls = @import("syscalls.zig");

pub fn print(comptime fmt: []const u8, args: anytype) void {
    var buffer: [1024]u8 = undefined;
    const str = std.fmt.bufPrint(&buffer, fmt, args) catch @panic("Failed to format string");

    syscalls.print(str);
}
