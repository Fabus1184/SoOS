const std = @import("std");

pub const types = @cImport({
    @cInclude("typedefs/syscalls.h");
});

const Syscall = struct {
    name: []const u8,
    number: c_int,
    arg_type: type,
    return_type: type,
};

const SYSCALLS: []const Syscall = &[_]Syscall{
    Syscall{ .name = "print", .number = types.SYSCALL_PRINT, .arg_type = types.syscall_print_t, .return_type = void },
    Syscall{ .name = "sleep", .number = types.SYSCALL_SLEEP, .arg_type = types.syscall_sleep_t, .return_type = void },
    Syscall{ .name = "exit", .number = types.SYSCALL_EXIT, .arg_type = types.syscall_exit_t, .return_type = void },
    Syscall{ .name = "listdir", .number = types.SYSCALL_LISTDIR, .arg_type = types.syscall_listdir_t, .return_type = types.syscall_listdir_return_t },
    Syscall{ .name = "read", .number = types.SYSCALL_READ, .arg_type = types.syscall_read_t, .return_type = types.syscall_read_return_t },
    Syscall{ .name = "fork", .number = types.SYSCALL_FORK, .arg_type = types.syscall_fork_t, .return_type = types.syscall_fork_return_t },
    Syscall{ .name = "open", .number = types.SYSCALL_OPEN, .arg_type = types.syscall_open_t, .return_type = types.syscall_open_return_t },
    Syscall{ .name = "close", .number = types.SYSCALL_CLOSE, .arg_type = types.syscall_close_t, .return_type = types.syscall_close_return_t },
    Syscall{ .name = "mmap", .number = types.SYSCALL_MMAP, .arg_type = types.syscall_mmap_t, .return_type = types.syscall_mmap_return_t },
    Syscall{ .name = "munmap", .number = types.SYSCALL_MUNMAP, .arg_type = types.syscall_munmap_t, .return_type = types.syscall_munmap_return_t },
    Syscall{ .name = "execve", .number = types.SYSCALL_EXECVE, .arg_type = types.syscall_execve_t, .return_type = types.syscall_execve_return_t },
    Syscall{ .name = "map_framebuffer", .number = types.SYSCALL_MAP_FRAMEBUFFER, .arg_type = types.syscall_map_framebuffer_t, .return_type = types.syscall_map_framebuffer_return_t },
};

fn call(comptime syscall: Syscall, arg: *syscall.arg_type) syscall.return_type {
    asm volatile (
        \\int $0x80
        :
        : [number] "{rax}" (syscall.number),
          [arg] "{rbx}" (arg),
        : "rax", "rbx", "memory"
    );

    if (syscall.return_type == void) {
        return;
    } else {
        return arg.*.return_value;
    }
}

pub fn print(arg: *types.syscall_print_t) void {
    call(SYSCALLS[0], arg);
}
pub fn sleep(arg: *types.syscall_sleep_t) void {
    call(SYSCALLS[1], arg);
}
pub fn exit(arg: *types.syscall_exit_t) void {
    call(SYSCALLS[2], arg);
}
pub fn listdir(arg: *types.syscall_listdir_t) types.syscall_listdir_return_t {
    return call(SYSCALLS[3], arg);
}
pub fn read(arg: *types.syscall_read_t) types.syscall_read_return_t {
    return call(SYSCALLS[4], arg);
}
pub fn fork(arg: *types.syscall_fork_t) types.syscall_fork_return_t {
    return call(SYSCALLS[5], arg);
}
pub fn open(arg: *types.syscall_open_t) types.syscall_open_return_t {
    return call(SYSCALLS[6], arg);
}
pub fn close(arg: *types.syscall_close_t) types.syscall_close_return_t {
    return call(SYSCALLS[7], arg);
}
pub fn mmap(arg: *types.syscall_mmap_t) types.syscall_mmap_return_t {
    return call(SYSCALLS[8], arg);
}
pub fn munmap(arg: *types.syscall_munmap_t) types.syscall_munmap_return_t {
    return call(SYSCALLS[9], arg);
}
pub fn execve(arg: *types.syscall_execve_t) types.syscall_execve_return_t {
    return call(SYSCALLS[10], arg);
}
pub fn mapFramebuffer(arg: *types.syscall_map_framebuffer_t) types.syscall_map_framebuffer_return_t {
    return call(SYSCALLS[11], arg);
}
