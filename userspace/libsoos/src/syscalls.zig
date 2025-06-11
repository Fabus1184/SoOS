const syscall = enum(u64) {
    print = 0,
    sleep = 1,
    exit = 2,
    listdir = 3,
    read = 4,
    fork = 5,
    open = 6,
    close = 7,
    mmap = 8,
    munmap = 9,
    execve = 10,
};

pub fn print(str: []const u8) void {
    asm volatile ("int $0x80"
        : // no output operands
        : [i] "{rax}" (@intFromEnum(syscall.print)),
          [str] "{rbx}" (str.ptr),
          [len] "{rcx}" (str.len),
        : "rax", "rbx", "rcx"
    );
}

pub fn sleep(ms: u64) void {
    asm volatile ("int $0x80"
        : // no output operands
        : [i] "{rax}" (@intFromEnum(syscall.sleep)),
          [ms] "{rbx}" (ms),
        : "rax", "rbx"
    );
}

pub fn exit(code: u64) noreturn {
    asm volatile ("int $0x80"
        : // no output operands
        : [i] "{rax}" (@intFromEnum(syscall.exit)),
          [code] "{rbx}" (code),
        : "rax", "rbx"
    );
    unreachable; // should not return
}

pub fn listdir(path: []const u8, index: u64, buffer: []u8) u64 {
    var result: u64 = 0;

    asm volatile ("int $0x80"
        : [result] "={rax}" (result),
        : [i] "{rax}" (@intFromEnum(syscall.listdir)),
          [path] "{rbx}" (path.ptr),
          [path_len] "{rcx}" (path.len),
          [index] "{rdx}" (index),
          [buffer] "{r8}" (buffer.ptr),
        : "rax", "rbx", "rcx", "rdx", "r8"
    );

    return result;
}

pub fn read(fd: u64, buffer: []u8) ?u64 {
    var result: i64 = 0;

    asm volatile ("int $0x80"
        : [result] "={rax}" (result),
        : [i] "{rax}" (@intFromEnum(syscall.read)),
          [fd] "{rbx}" (fd),
          [buffer] "{rcx}" (buffer.ptr),
          [buffer_len] "{rdx}" (buffer.len),
        : "rax", "rbx", "rcx", "rdx"
    );

    if (result >= 0) {
        return @intCast(result);
    } else {
        return null;
    }
}

pub fn fork() u32 {
    var result: u32 = 0;

    asm volatile ("int $0x80"
        : [result] "={rax}" (result),
        : [i] "{rax}" (@intFromEnum(syscall.fork)),
        : "rax"
    );

    return result;
}

pub fn open(path: []const u8) ?u64 {
    var result: i64 = 0;

    asm volatile ("int $0x80"
        : [result] "={rax}" (result),
        : [i] "{rax}" (@intFromEnum(syscall.open)),
          [path] "{rbx}" (path.ptr),
          [path_len] "{rcx}" (path.len),
        : "rax", "rbx", "rcx"
    );

    if (result >= 0) {
        return @intCast(result);
    } else {
        return null;
    }
}

pub fn close(fd: u64) u64 {
    var result: u64 = 0;

    asm volatile ("int $0x80"
        : [result] "={rax}" (result),
        : [i] "{rax}" (@intFromEnum(syscall.close)),
          [fd] "{rbx}" (fd),
        : "rax", "rbx"
    );

    return result;
}

pub fn mmap() ?struct { ptr: [*]u8, size: u64 } {
    var ptr: u64 = 0;
    var size: u64 = 0;

    asm volatile ("int $0x80"
        : [ptr] "={rax}" (ptr),
          [size] "={rbx}" (size),
        : [i] "{rax}" (@intFromEnum(syscall.mmap)),
        : "rax", "rbx"
    );

    if (ptr != 0) {
        return .{ .ptr = @ptrFromInt(ptr), .size = size };
    } else {
        return null;
    }
}

pub fn munmap(ptr: *anyopaque) void {
    asm volatile ("int $0x80"
        : // no output operands
        : [i] "{rax}" (@intFromEnum(syscall.munmap)),
          [ptr] "{rbx}" (ptr),
        : "rax", "rbx"
    );
}

pub fn execve(path: []const u8, argv: []const []const u8) !noreturn {
    var result: u64 = 0;

    if (argv.len > 64) {
        return error.TooManyArguments;
    }

    // Prepare the argv array as length-prefixed strings
    const Arg = struct {
        len: u64,
        ptr: [*]const u8,
    };

    var argv_array: [64]Arg = undefined;
    for (argv, 0..) |arg, i| {
        argv_array[i] = Arg{
            .ptr = arg.ptr,
            .len = @intCast(arg.len),
        };
    }

    asm volatile ("int $0x80"
        : [result] "={rax}" (result),
        : [i] "{rax}" (@intFromEnum(syscall.execve)),
          [path] "{rbx}" (path.ptr),
          [path_len] "{rcx}" (path.len),
          [argv_len] "{rdx}" (argv.len),
          [argv_array] "{r8}" (@as(*const anyopaque, &argv_array)),
        : "rax", "rbx", "rcx", "rdx", "r8"
    );

    if (result != 0) {
        return error.ExecveFailed;
    } else {
        unreachable; // should not return
    }
}
