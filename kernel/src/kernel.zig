const std = @import("std");

const Term = @import("term/term.zig").Term;
const idt = @import("idt.zig");
const GDT = @import("gdt.zig").GDT;
const TSS = @import("tss.zig").TSS;
const paging = @import("paging.zig");
const limine = @import("limine.zig");
const elf = @import("elf.zig");

extern const _START_KERNEL_MEMORY: *anyopaque;
extern const _END_KERNEL_MEMORY: *anyopaque;

const _KERNEL_STACK_SIZE: usize = 0x1_000_000;
var _KERNEL_STACK: [_KERNEL_STACK_SIZE]u8 align(16) = undefined;

const _INTERRUPT_STACK_SIZE: usize = 0x100_000;
var _INTERRUPT_STACK: [_INTERRUPT_STACK_SIZE]u8 align(16) = undefined;

const _KERNEL_HEAP_SIZE: usize = 0x10_000_000;
var _KERNEL_HEAP: [_KERNEL_HEAP_SIZE]u8 align(16) = undefined;

export fn _start() callconv(.naked) noreturn {
    asm volatile (
        \\ call _init
        :
        : [stackPointer] "{rsp}" (@intFromPtr(&_KERNEL_STACK[0]) + _KERNEL_STACK_SIZE & ~@as(u64, 0xF)),
        : "memory"
    );
}

export fn _init() noreturn {
    asm volatile (
        \\
        // enable SSE, AVX, and x87 instructions
        // CR0 remove EMULATE_COPROCESSOR flag, set MONITOR_COPROCESSOR flag
        \\ mov %cr0, %rax
        \\ and $~(1 << 2), %rax
        \\ or $(1 << 1), %rax
        \\ mov %rax, %cr0
        // CR4 set OSFXSR, OSXMMEXCPT, and OSXSAVE flags
        \\ mov %cr4, %rax
        \\ or $(1 << 9 | 1 << 10 | 1 << 18), %rax
        \\ mov %rax, %cr4
        // XCR0 set SSE, AVX, and x87 flags
        \\ xor %ecx, %ecx
        \\ xgetbv
        \\ or $0x7, %eax
        \\ xsetbv
        \\
        ::: "rax", "rcx", "rdx", "memory");

    main() catch |err| {
        std.log.err("kernel main returned with an error: {}", .{err});
        @panic("kernel main returned with an error");
    };

    @panic("kernel main returned unexpectedly");
}

fn earlyPanic(err: anyerror) noreturn {
    var debugThis: [1024]u8 = undefined;
    _ = std.fmt.bufPrint(&debugThis, "error: {}", .{err}) catch unreachable;
    @trap();
}

var _LOG_TERM: ?*Term = null;
pub const std_options = std.Options{
    .log_level = .debug,
    .logFn = struct {
        pub fn f(
            comptime message_level: std.log.Level,
            comptime _: @TypeOf(.enum_literal),
            comptime format: []const u8,
            args: anytype,
        ) void {
            const colors = std.EnumArray(std.log.Level, []const u8).init(.{
                .debug = "\x1b[34m",
                .info = "\x1b[32m",
                .warn = "\x1b[33m",
                .err = "\x1b[31m",
            });
            const reset = "\x1b[0m";

            const t = _LOG_TERM orelse earlyPanic(error.termNotInitialized);
            var parser = t.ansiParser();
            std.fmt.format(parser.writer(), "{s}[{s}]: ", .{ colors.get(message_level), @tagName(message_level) }) catch |err| earlyPanic(err);
            std.fmt.format(parser.writer(), format, args) catch |err| earlyPanic(err);
            std.fmt.format(parser.writer(), "{s}\n", .{reset}) catch |err| earlyPanic(err);
        }
    }.f,
};
pub const panic = std.debug.FullPanic(struct {
    fn panic(message: []const u8, ptr: ?usize) noreturn {
        asm volatile ("cli");

        std.log.err("kernel panic: {s}, at {?x}", .{ message, ptr });

        while (true) {
            asm volatile ("hlt");
        }
    }
}.panic);

fn main() !void {
    const framebuffer = limine.LIMINE_FRAMEBUFFER_REQUEST.response.*.framebuffers[0];

    var term = Term.init(framebuffer.*.address.?, framebuffer.*.width, framebuffer.*.height) catch |err| earlyPanic(err);
    term.clear(Term.Color.BLACK);
    _LOG_TERM = &term;

    std.log.info("SoOS version 0.1.0", .{});
    std.log.debug("kernel memory: 0x{x}..0x{x}", .{ @intFromPtr(&_START_KERNEL_MEMORY), @intFromPtr(&_END_KERNEL_MEMORY) });
    std.log.debug("framebuffer: {}x{}@{}bpp", .{ framebuffer.*.width, framebuffer.*.height, framebuffer.*.bpp });
    std.log.debug("stack is at 0x{x}..0x{x}", .{
        @intFromPtr(&_KERNEL_STACK[0]),
        @intFromPtr(&_KERNEL_STACK[0]) + _KERNEL_STACK_SIZE,
    });
    std.log.debug("paging mode: {}", .{limine.LIMINE_PAGING_MODE_REQUEST.response.*.mode});
    std.debug.assert(limine.LIMINE_PAGING_MODE_REQUEST.response.*.mode == limine.raw.LIMINE_PAGING_MODE_X86_64_4LVL);
    std.log.debug("hhdm at 0x{x}", .{limine.LIMINE_HHDM_REQUEST.response.*.offset});
    const memmap = limine.LIMINE_MEMMAP_REQUEST.response.*.entries.*[0..limine.LIMINE_MEMMAP_REQUEST.response.*.entry_count];
    for (memmap, 0..) |entry, i| {
        std.log.debug("memmap entry {d:2}: type: {d:2}, base: 0x{x:16}, length: 0x{x}", .{ i, entry.type, entry.base, entry.length });
    }

    var tss align(16) = TSS.zero();
    tss.rsp0 = @intFromPtr(&_INTERRUPT_STACK[0]) + _INTERRUPT_STACK_SIZE & ~@as(u64, 0xF);
    tss.ist1 = tss.rsp0; // Use the same stack for all interrupts for simplicity

    var gdt = GDT.init();
    const kernelCodeSegment = gdt.addSegment(.ring0, true);
    const kernelDataSegment = gdt.addSegment(.ring0, false);
    const userCodeSegment = gdt.addSegment(.ring3, true);
    const userDataSegment = gdt.addSegment(.ring3, false);
    const tssSelector = gdt.addSystemSegment(&tss);
    for (0..gdt.entries_count) |i| {
        std.log.debug("gdt {d}: 0x{x}", .{ i, @as(u64, @bitCast(gdt.entries[i])) });
    }
    std.log.debug("kcs: {x}, kds: {x}, ucs: {x}, uds: {x}, tss: {x}", .{
        @as(u16, @bitCast(kernelCodeSegment)),
        @as(u16, @bitCast(kernelDataSegment)),
        @as(u16, @bitCast(userCodeSegment)),
        @as(u16, @bitCast(userDataSegment)),
        @as(u16, @bitCast(tssSelector)),
    });
    gdt.load();
    GDT.reloadSegments(kernelCodeSegment, kernelDataSegment);

    TSS.load(tssSelector);

    var currentCodeSegment: u16 = 0x00;
    asm volatile (
        \\ mov %cs, %[cs]
        : [cs] "=r" (currentCodeSegment),
        :
        : "memory"
    );
    std.log.debug("current code segment: {x}", .{currentCodeSegment});

    var IDT align(16) = idt.IDT.init();

    IDT.setInterruptHandler(.breakpoint, .ring0, breakpointHandler, kernelCodeSegment);
    IDT.setExceptionHandler(.generalProtectionFault, .ring0, generalProtectionFault, kernelCodeSegment);
    IDT.setExceptionHandler(.pageFault, .ring0, pageFault, kernelCodeSegment);

    IDT.load();

    @breakpoint();

    std.log.debug("returned after breakpoint handler", .{});

    var p = paging.OffsetPageTable.fromCurrent(limine.LIMINE_HHDM_REQUEST.response.*.offset);
    p.load();

    const frameAllocator = paging.FRAME_ALLOCATOR.init(memmap, &p);

    p.keep(&.{
        .{ limine.LIMINE_HHDM_REQUEST.response.*.offset, limine.LIMINE_HHDM_REQUEST.response.*.offset + 0x800000000 },
        .{ @intFromPtr(&_START_KERNEL_MEMORY), @intFromPtr(&_END_KERNEL_MEMORY) },
        .{
            @intFromPtr(framebuffer.*.address.?),
            @intFromPtr(framebuffer.*.address.?) + framebuffer.*.width * framebuffer.*.height * 4,
        },
    }, frameAllocator);
    std.log.debug("paging initialized", .{});
    p.load();

    //var kernelAllocator = std.heap.FixedBufferAllocator.init(&_KERNEL_HEAP);

    var userspacePaging = p.clone();
    const e = try elf.Elf.load(@embedFile("userspace-build/x86_64-unknown-none/debug/sosh"), &userspacePaging, frameAllocator);

    const userStackAddress = 0x8000_0000_0000 - 0x40000; // 2MiB stack at the top of the address space
    const frame = try frameAllocator.allocateFrame(.@"2MiB");
    std.log.debug("user stack in frame 0x{x}", .{frame});
    try userspacePaging.map(
        frameAllocator,
        userStackAddress,
        frame,
        .@"2MiB",
        .{ .writable = true, .userAccessible = true },
    );

    std.log.debug("jumping to userspace (0x{x})", .{e.entry});

    iretToUserspace(
        @bitCast(userDataSegment),
        @bitCast(userCodeSegment),
        e.entry,
        userStackAddress + 0x40000,
        0x202,
    );

    std.log.info("nothing more to do, bye!", .{});

    while (true) {
        asm volatile ("hlt");
    }
}

fn iretToUserspace(
    dataSegment: u16,
    codeSegment: u16,
    instructionPointer: u64,
    stackPointer: u64,
    flags: u64,
) noreturn {
    asm volatile (
        \\ mov %[dataSegment], %ds
        \\ mov %[dataSegment], %es
        \\ mov %[dataSegment], %fs
        \\ mov %[dataSegment], %gs
        \\ push %[dataSegment]
        \\ push %[stackPointer]
        \\ push %[flags]
        \\ push %[codeSegment]
        \\ push %[instructionPointer]
        \\ iretq
        :
        : [instructionPointer] "r" (instructionPointer),
          [codeSegment] "r" (@as(u64, @intCast(codeSegment))),
          [flags] "r" (flags),
          [stackPointer] "r" (stackPointer),
          [dataSegment] "r" (@as(u64, @intCast(dataSegment))),
        : "memory"
    );
    @panic("iret should not return");
}

fn breakpointHandler(
    stackFrame: *idt.InterruptStackFrame,
) callconv(.{ .x86_64_interrupt = .{} }) void {
    std.log.debug("breakpoint hit at rip 0x{x}, rsp 0x{x}", .{ stackFrame.instructionPointer, stackFrame.stackPointer });
}

fn generalProtectionFault(
    stackFrame: *idt.InterruptStackFrame,
    errorCode: u64,
) callconv(.{ .x86_64_interrupt = .{} }) noreturn {
    std.log.err("general protection fault at rip 0x{x}, rsp 0x{x}, error code: 0x{x}", .{
        stackFrame.instructionPointer,
        stackFrame.stackPointer,
        errorCode,
    });
    @panic("general protection fault");
}

fn pageFault(
    stackFrame: *idt.InterruptStackFrame,
    errorCode: u64,
) callconv(.{ .x86_64_interrupt = .{} }) noreturn {
    const Error = packed struct(u64) {
        pageNotPresent: bool,
        write: bool,
        userMode: bool,
        reserved: bool,
        instructionFetch: bool,
        _: u59,
    };

    var address: u64 = 0;
    asm volatile (
        \\ mov %cr2, %[addr]
        : [addr] "=r" (address),
        :
        : "memory"
    );

    std.log.err("page fault at rip 0x{x}, rsp 0x{x}, error code: {}, address: 0x{x}", .{
        stackFrame.instructionPointer,
        stackFrame.stackPointer,
        @as(Error, @bitCast(errorCode)),
        address,
    });
    @panic("page fault");
}
