const std = @import("std");
const syscalls = @import("syscalls.zig");
pub const events = @cImport({
    @cInclude("typedefs/events.h");
});

const PageAllocator = struct {
    fn alloc(_: *anyopaque, len: usize, _: std.mem.Alignment, _: usize) ?[*]u8 {
        const firstPage: []u8 = @ptrCast(mmap() catch @panic("Failed to allocate memory"));
        // allocate additional pages if the requested length exceeds one page
        for (0..len / 4096) |i| {
            const page = mmap() catch @panic("Failed to allocate additional memory");
            if (page.ptr != firstPage.ptr + 4096 * (i + 1)) {
                // if the page is not contiguous, we return null
                @panic("pages are not contiguous");
            }
        }
        return @as([*]u8, @ptrCast(firstPage.ptr));
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

        munmap(memory.ptr) catch @panic("Failed to unmap memory");

        if (memory.len > 4096) {
            for (0..memory.len / 4096) |i| {
                munmap(@alignCast(@ptrCast(memory.ptr + 4096 * (i + 1)))) catch |err| {
                    std.log.err("Failed to unmap additional memory: {}", .{err});
                    @panic("Failed to unmap additional memory");
                };
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
    const str = std.fmt.bufPrint(&buffer, fmt, args) catch |err| {
        std.log.err("buffer too small for format string: {}", .{err});
        @panic("buffer too small for format string");
    };

    var arg = syscalls.types.syscall_print_t{
        .message = syscalls.types.string_const_t{
            .ptr = str.ptr,
            .len = @intCast(str.len),
        },
    };

    syscalls.print(&arg);
}

pub fn sleep(milliseconds: u32) void {
    var arg = syscalls.types.syscall_sleep_t{
        .milliseconds = milliseconds,
    };
    syscalls.sleep(&arg);
}

pub fn exit(status: u32) noreturn {
    var arg = syscalls.types.syscall_exit_t{
        .status = status,
    };
    syscalls.exit(&arg);

    @panic("exit syscall returned, which should not happen");
}

const ListDir = struct {
    _buffer: [64][64]u8,
    entries: [64]syscalls.types.syscall_listdir_entry_t,
    entries_len: usize,

    index: usize = 0,
    const Item = struct {
        name: []const u8,
        type: enum { file, directory },
    };

    pub fn next(self: *ListDir) ?Item {
        if (self.index >= self.entries_len) {
            return null;
        }

        const entry = self.entries[self.index];
        self.index += 1;

        return Item{
            .name = self._buffer[self.index - 1][0..entry.name.len],
            .type = switch (entry.type) {
                syscalls.types.SYSCALL_LISTDIR_ENTRY_TYPE_FILE => .file,
                syscalls.types.SYSCALL_LISTDIR_ENTRY_TYPE_DIR => .directory,
                else => @panic("unknown entry type"),
            },
        };
    }
};

pub fn listdir(path: []const u8) !ListDir {
    var listDir = ListDir{
        ._buffer = undefined,
        .entries = undefined,
        .entries_len = 0,
    };

    for (0..listDir.entries.len) |i| {
        listDir.entries[i] = syscalls.types.syscall_listdir_entry_t{
            .name = syscalls.types.string_t{
                .ptr = &listDir._buffer[i],
                .len = 64,
            },
        };
    }

    var arg = syscalls.types.syscall_listdir_t{
        .entries = &listDir.entries,
        .entries_len = @intCast(listDir.entries.len),
        .path = syscalls.types.string_const_t{
            .ptr = path.ptr,
            .len = @intCast(path.len),
        },
    };

    const ret = syscalls.listdir(&arg);

    if (ret.@"error" != syscalls.types.SYSCALL_LISTDIR_ERROR_NONE) {
        return switch (ret.@"error") {
            syscalls.types.SYSCALL_LISTDIR_ERROR_NOT_FOUND => error.NotFound,
            syscalls.types.SYSCALL_LISTDIR_ERROR_BUFFER_TOO_SMALL => error.BufferTooSmall,
            else => @panic("listdir unexpected error"),
        };
    }

    listDir.entries_len = @intCast(ret.entries_count);
    return listDir;
}

pub fn read(fd: i32, buffer: []u8) !usize {
    var arg = syscalls.types.syscall_read_t{
        .fd = fd,
        .buf = buffer.ptr,
        .len = @intCast(buffer.len),
    };

    const ret = syscalls.read(&arg);

    if (ret.@"error" != syscalls.types.SYSCALL_READ_ERROR_NONE) {
        return switch (ret.@"error") {
            syscalls.types.SYSCALL_READ_ERROR_INVALID_FD => error.InvalidFd,
            else => @panic("read unexpected error"),
        };
    }

    return @intCast(ret.bytes_read);
}

pub fn fork() u32 {
    var arg = syscalls.types.syscall_fork_t{ .return_value = .{ .child_pid = 0 } };

    const ret = syscalls.fork(&arg);

    return @intCast(ret.child_pid);
}

pub fn open(path: []const u8) !i32 {
    var arg = syscalls.types.syscall_open_t{
        .path = syscalls.types.string_const_t{
            .ptr = path.ptr,
            .len = @intCast(path.len),
        },
    };

    const ret = syscalls.open(&arg);

    if (ret.@"error" != syscalls.types.SYSCALL_OPEN_ERROR_NONE) {
        return switch (ret.@"error") {
            syscalls.types.SYSCALL_OPEN_ERROR_NOT_FOUND => error.NotFound,
            else => @panic("open unexpected error"),
        };
    }

    return ret.fd;
}

pub fn close(fd: i32) !void {
    var arg = syscalls.types.syscall_close_t{
        .fd = fd,
    };

    const ret = syscalls.close(&arg);

    if (ret.@"error" != syscalls.types.SYSCALL_CLOSE_ERROR_NONE) {
        return switch (ret.@"error") {
            syscalls.types.SYSCALL_CLOSE_ERROR_INVALID_FD => error.InvalidFd,
            else => @panic("close unexpected error"),
        };
    }
}

pub fn mmap() ![]u8 {
    var arg = syscalls.types.syscall_mmap_t{
        .size = 4096, // Allocate one page (4KB)
    };

    const ret = syscalls.mmap(&arg);

    if (ret.@"error" != syscalls.types.SYSCALL_MMAP_ERROR_NONE) {
        return switch (ret.@"error") {
            else => @panic("mmap unexpected error"),
        };
    }

    return @as([*]u8, @ptrCast(ret.addr))[0..4096];
}

pub fn munmap(ptr: *anyopaque) !void {
    var arg = syscalls.types.syscall_munmap_t{
        .addr = @ptrCast(ptr),
        .size = 4096,
    };

    const ret = syscalls.munmap(&arg);

    if (ret.@"error" != syscalls.types.SYSCALL_MUNMAP_ERROR_NONE) {
        return switch (ret.@"error") {
            syscalls.types.SYSCALL_MUNMAP_ERROR_INVALID_ADDR => error.InvalidAddress,
            else => @panic("munmap unexpected error"),
        };
    }
}

pub fn execve(program: []const u8, args: []const []const u8) !noreturn {
    var argv: [64]syscalls.types.string_const_t = undefined;
    for (0..args.len) |i| {
        argv[i] = syscalls.types.string_const_t{
            .ptr = args[i].ptr,
            .len = @intCast(args[i].len),
        };
    }

    var arg = syscalls.types.syscall_execve_t{
        .path = syscalls.types.string_const_t{
            .ptr = program.ptr,
            .len = @intCast(program.len),
        },
        .argv = &argv,
        .argv_len = @intCast(args.len),
        .envp = null,
        .envp_len = 0,
    };

    const ret = syscalls.execve(&arg);

    if (ret.@"error" != syscalls.types.SYSCALL_EXECVE_ERROR_NONE) {
        return switch (ret.@"error") {
            syscalls.types.SYSCALL_EXECVE_ERROR_NOT_FOUND => error.NotFound,
            else => @panic("execve unexpected error"),
        };
    } else {
        @panic("execve should not return, it replaces the current process");
    }
}

pub const Framebuffer = struct {
    ptr: [*]u32,
    width: u32,
    height: u32,

    pub inline fn blit(self: Framebuffer, x: usize, y: usize, color: u32) void {
        if (x < self.width and y < self.height) {
            self.ptr[y * self.width + x] = color;
        }
    }

    pub fn clear(self: Framebuffer, color: u32) void {
        for (0..self.height) |y| {
            for (0..self.width) |x| {
                self.ptr[y * self.width + x] = color;
            }
        }
    }
};

pub fn mapFramebuffer() Framebuffer {
    var arg = syscalls.types.syscall_map_framebuffer_t{};

    const ret = syscalls.mapFramebuffer(&arg);

    return .{
        .ptr = @as([*]u32, @alignCast(@ptrCast(ret.addr))),
        .width = ret.width,
        .height = ret.height,
    };
}
