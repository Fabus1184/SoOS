const std = @import("std");

const elf = @import("elf.zig");
const paging = @import("../paging.zig");
const idt = @import("../idt.zig");
const gdt = @import("../gdt.zig");
const types = @import("../types.zig");
const kernel = @import("../kernel.zig");

var NEXT_PID: u32 = 1;

pub const State = enum {
    ready,
    syscall,
    aborted,
};

pub const Process = struct {
    pid: u32,
    name: []const u8,

    paging: paging.OffsetPageTable,
    mappedPages: std.ArrayList(elf.MappedPage),

    state: State = .ready,

    savedState: struct { idt.State, idt.InterruptStackFrame },

    stdin: ?std.ArrayList(u8),

    pub fn initUser(
        allocator: std.mem.Allocator,
        elfBytes: []const u8,
        kernelPaging: *paging.OffsetPageTable,
        frameAllocator: *paging.FrameAllocator,
        codeSegment: gdt.SegmentSelector,
        dataSegment: gdt.SegmentSelector,
        name: []const u8,
    ) !Process {
        var processPaging = kernelPaging.clone();

        var mappedPages = std.ArrayList(elf.MappedPage).init(allocator);

        const program = try elf.Elf.load(elfBytes, &processPaging, frameAllocator, &mappedPages);

        const userStackAddress = 0x8000_0000_0000 - 0x40000; // 2MiB stack at the top of the address space
        const frame = try frameAllocator.allocateFrame(.@"2MiB");
        try processPaging.map(
            frameAllocator,
            userStackAddress,
            frame,
            .@"2MiB",
            .{ .writable = true, .userAccessible = true },
        );
        try mappedPages.append(elf.MappedPage{ .virtualAddress = userStackAddress, .size = .@"2MiB" });

        return Process{
            .pid = @atomicRmw(u32, &NEXT_PID, .Add, 1, .monotonic),
            .name = name,
            .paging = processPaging,
            .mappedPages = mappedPages,
            .stdin = std.ArrayList(u8).init(allocator),
            .savedState = .{
                idt.State{},
                idt.InterruptStackFrame{
                    .codeSegment = @bitCast(codeSegment),
                    .stackSegment = @bitCast(dataSegment),
                    .rip = program.entry,
                    .rsp = userStackAddress + 0x40000,
                    .flags = 0x202,
                },
            },
        };
    }

    pub fn initKernel(
        allocator: std.mem.Allocator,
        kernelPaging: *paging.OffsetPageTable,
        codeSegment: gdt.SegmentSelector,
        dataSegment: gdt.SegmentSelector,
        stackPtr: u64,
        entry: *const fn () callconv(.naked) noreturn,
    ) !Process {
        return Process{
            .pid = @atomicRmw(u32, &NEXT_PID, .Add, 1, .monotonic),
            .name = "kernel",
            .paging = kernelPaging.clone(),
            .mappedPages = std.ArrayList(elf.MappedPage).init(allocator),
            .stdin = null,
            .savedState = .{
                idt.State{},
                idt.InterruptStackFrame{
                    .codeSegment = @bitCast(codeSegment),
                    .stackSegment = @bitCast(dataSegment),
                    .rip = @intFromPtr(entry),
                    .rsp = stackPtr,
                    .flags = 0x202,
                },
            },
        };
    }

    pub fn deinit(self: Process, frameAllocator: *paging.FrameAllocator) void {
        for (self.mappedPages.items) |_| {
            @panic("todo");
        }

        _ = frameAllocator;

        self.mappedPages.deinit();
        self.stdin.deinit();
    }

    pub fn run(self: *Process) noreturn {
        asm volatile ("cli");

        self.paging.load();

        iret(&self.savedState[1], &self.savedState[0]);
    }

    pub fn ready(self: *Process) bool {
        return self.state == .ready;
    }

    pub fn copyFromProcess(self: *Process, comptime T: type, src: *const T) T {
        asm volatile ("cli");
        self.paging.load();
        const value = @as(*const volatile T, src).*;
        kernel.KERNEL_PAGING.load();
        asm volatile ("sti");
        return value;
    }
    pub fn copyFromProcessSlice(self: *Process, comptime T: type, src: []const T, dest: []T) void {
        asm volatile ("cli");
        self.paging.load();
        std.mem.copyForwards(T, dest, src);
        kernel.KERNEL_PAGING.load();
        asm volatile ("sti");
    }
    pub fn writeToProcess(self: *Process, comptime T: type, dest: *T, value: T) void {
        asm volatile ("cli");
        self.paging.load();
        @as(*volatile T, dest).* = value;
        kernel.KERNEL_PAGING.load();
        asm volatile ("sti");
    }

    pub fn handleSyscall(self: *Process) void {
        std.log.debug("process {s} handling syscall: {}", .{ self.name, self.savedState[0].registers.rdi });

        switch (self.savedState[0].registers.rdi) {
            types.SYSCALL_WRITE => {
                const argsPtr: *types.syscall_write_t = @ptrFromInt(self.savedState[0].registers.rsi);
                const args = self.copyFromProcess(types.syscall_write_t, argsPtr);

                var buf: [4096]u8 = undefined;
                if (args.len > buf.len) {
                    std.log.err("write syscall called with too large buffer: {}", .{args.len});
                    @panic("write syscall called with too large buffer");
                }

                const src: [*]const u8 = @ptrCast(args.buf);
                self.copyFromProcessSlice(u8, src[0..args.len], buf[0..args.len]);

                switch (args.fd) {
                    1 => {
                        std.log.info("write to stdout: {s}", .{buf[0..args.len]});

                        self.writeToProcess(types.syscall_write_return_t, &argsPtr.return_value, .{
                            .bytes_written = @intCast(args.len),
                            .@"error" = types.SYSCALL_WRITE_ERROR_NONE,
                        });
                    },
                    else => {
                        std.log.err("write syscall called with unknown fd: {}", .{args.fd});
                        @panic("write syscall called with unknown fd");
                    },
                }
            },
            else => {
                std.log.err("unknown syscall called: {}", .{self.savedState[0].registers.rdi});
                @panic("unknown syscall");
            },
        }

        self.state = .ready;
    }
};

pub fn iret(
    stackFrame: *idt.InterruptStackFrame,
    state: *idt.State,
) noreturn {
    asm volatile (
        \\ cli
        // push stack frame for iret
        // ss
        \\ push 0x20(%[stackFrame])
        // rsp
        \\ push 0x18(%[stackFrame])
        // flags
        \\ push 0x10(%[stackFrame])
        // cs
        \\ push 0x8(%[stackFrame])
        // rip
        \\ push 0x0(%[stackFrame])
        \\
        // load ss in all other segment registers
        \\ mov 0x20(%[stackFrame]), %ds
        \\ mov 0x20(%[stackFrame]), %es
        \\ mov 0x20(%[stackFrame]), %fs
        \\ mov 0x20(%[stackFrame]), %gs
        // xrstor xsave area
        \\ mov $~0, %eax
        \\ mov $~0, %edx
        \\ xrstor 0(%[state])
        // restore registers
        \\ mov (0x1000 + ( 0 * 0x8))(%rbx), %rax
        // skip rbx for now
        // mov (0x1000 + ( 1 * 0x8))(%rbx), %rbx
        \\ mov (0x1000 + ( 2 * 0x8))(%rbx), %rcx
        \\ mov (0x1000 + ( 3 * 0x8))(%rbx), %rdx
        \\ mov (0x1000 + ( 4 * 0x8))(%rbx), %rsi
        \\ mov (0x1000 + ( 5 * 0x8))(%rbx), %rdi
        \\ mov (0x1000 + ( 6 * 0x8))(%rbx), %rbp
        \\ mov (0x1000 + ( 7 * 0x8))(%rbx), %r8
        \\ mov (0x1000 + ( 8 * 0x8))(%rbx), %r9
        \\ mov (0x1000 + ( 9 * 0x8))(%rbx), %r10
        \\ mov (0x1000 + (10 * 0x8))(%rbx), %r11
        \\ mov (0x1000 + (11 * 0x8))(%rbx), %r12
        \\ mov (0x1000 + (12 * 0x8))(%rbx), %r13
        \\ mov (0x1000 + (13 * 0x8))(%rbx), %r14
        \\ mov (0x1000 + (14 * 0x8))(%rbx), %r15
        // restore rbx
        \\ mov (0x1000 + ( 1 * 0x8))(%rbx), %rbx
        \\
        \\ iretq
        :
        : [stackFrame] "{rax}" (stackFrame),
          [state] "{rbx}" (state),
        : "memory", "noreturn"
    );
    unreachable;
}
