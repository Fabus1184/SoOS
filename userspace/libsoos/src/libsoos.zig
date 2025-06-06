const std = @import("std");
pub const syscalls = @import("syscalls.zig");

const PageAllocator = struct {
    fn alloc(_: *anyopaque, len: usize, _: std.mem.Alignment, _: usize) ?[*]u8 {
        if (len > 4096) return null; // Limit allocation to 4KB pages

        if (syscalls.mmap()) |page| {
            std.log.debug("allocated page at: {*} size: {d}", .{ page.ptr, 4096 });
            return page.ptr;
        } else return null;
    }

    fn resize(_: *anyopaque, _: []u8, _: std.mem.Alignment, _: usize, _: usize) bool {
        return false;
    }

    fn remap(_: *anyopaque, _: []u8, _: std.mem.Alignment, _: usize, _: usize) ?[*]u8 {
        return null;
    }

    fn free(_: *anyopaque, memory: []u8, _: std.mem.Alignment, _: usize) void {
        syscalls.munmap(memory.ptr);
    }
};

pub fn pageAllocator() std.mem.Allocator {
    return std.mem.Allocator{
        .ptr = undefined,
        .vtable = &std.mem.Allocator.VTable{
            .alloc = PageAllocator.alloc,
            .remap = PageAllocator.remap,
            .free = PageAllocator.free,
            .resize = PageAllocator.resize,
        },
    };
}

pub fn print(comptime fmt: []const u8, args: anytype) void {
    var buffer: [1024]u8 = undefined;
    const str = std.fmt.bufPrint(&buffer, fmt, args) catch @panic("Failed to format string");

    syscalls.print(str);
}
