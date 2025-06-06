const std = @import("std");
pub const syscalls = @import("syscalls.zig");

const PageAllocator = struct {
    fn alloc(_: *anyopaque, len: usize, _: std.mem.Alignment, _: usize) ?[*]u8 {
        const firstPage: ?[*]u8 = (syscalls.mmap() orelse @panic("Failed to allocate memory")).ptr;
        // allocate additional pages if the requested length exceeds one page
        for (0..len / 4096) |i| {
            const page = syscalls.mmap() orelse @panic("Failed to allocate additional memory");
            if (page.ptr != firstPage.? + 4096 * (i + 1)) {
                // if the page is not contiguous, we return null
                @panic("pages are not contiguous");
            }
        }
        return firstPage;
    }

    fn resize(_: *anyopaque, _: []u8, _: std.mem.Alignment, _: usize, _: usize) bool {
        return false;
    }

    fn remap(_: *anyopaque, _: []u8, _: std.mem.Alignment, _: usize, _: usize) ?[*]u8 {
        return null;
    }

    fn free(_: *anyopaque, memory: []u8, _: std.mem.Alignment, _: usize) void {
        if (@intFromPtr(memory.ptr) % 4096 != 0) {
            @panic("memory is not page aligned");
        }

        syscalls.munmap(memory.ptr);

        if (memory.len > 4096) {
            for (0..memory.len / 4096) |i| {
                syscalls.munmap(memory.ptr + 4096 * (i + 1));
            }
        }
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
    var buffer: [8192]u8 = undefined;
    const str = std.fmt.bufPrint(&buffer, fmt, args) catch @panic("Failed to format string");

    syscalls.print(str);
}
