const std = @import("std");

const Term = @import("term/term.zig").Term;
const IDT = @import("idt.zig").IDT;
const GDT = @import("gdt.zig").GDT;
const TSS = @import("tss.zig").TSS;
const paging = @import("paging.zig");
const limine = @import("limine.zig");

const _KERNEL_STACK_SIZE: usize = 0x1_000_000;
var _KERNEL_STACK: [_KERNEL_STACK_SIZE]u8 align(16) = undefined;

const _INTERRUPT_STACK_SIZE: usize = 0x1_000;
var _INTERRUPT_STACK: [_INTERRUPT_STACK_SIZE]u8 align(16) = undefined;

export fn _start() callconv(.naked) noreturn {
    asm volatile (
        \\ call _init
        :
        : [stackPointer] "{rsp}" (@intFromPtr(&_KERNEL_STACK[0]) + _KERNEL_STACK_SIZE & ~@as(u64, 0xF)),
        : "memory"
    );
}

export fn _init() noreturn {
    // enable SSE, AVX, and x87 instructions
    asm volatile (
        \\
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

    main() catch {
        @trap();
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
    const fb = limine.LIMINE_FRAMEBUFFER_REQUEST.response.*.framebuffers[0];

    var term = Term.init(fb.*.address.?, fb.*.width, fb.*.height) catch |err| earlyPanic(err);
    term.clear(Term.Color.BLACK);
    _LOG_TERM = &term;

    std.log.info("SoOS version 0.1.0", .{});
    std.log.debug("framebuffer: {}x{}@{}bpp", .{ fb.*.width, fb.*.height, fb.*.bpp });
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

    var idt align(16) = IDT.init();

    idt.setInterruptHandler(.breakpoint, .ring0, @as(*const anyopaque, breakpointHandler), kernelCodeSegment);
    idt.setExceptionHandler(.generalProtectionFault, .ring0, @as(*const anyopaque, generalProtectionFault), kernelCodeSegment);
    idt.setExceptionHandler(.pageFault, .ring0, @as(*const anyopaque, pageFault), kernelCodeSegment);

    idt.load();

    @breakpoint();

    std.log.debug("returned after breakpoint handler", .{});

    var p = paging.OffsetPageTable.fromCurrent(limine.LIMINE_HHDM_REQUEST.response.*.offset);
    var it = p.iterator();
    while (it.next()) |entry| {
        std.log.debug("page table entry: 0x{x} ({s}) => 0x{x}", .{
            entry.virtualAddress,
            @tagName(entry.size),
            entry.physicalAddress,
        });
    }

    // translate framebuffer pointer
    {
        const t = p.translate(@intFromPtr(fb.*.address.?));
        std.log.debug("address 0x{x} translates to {?}", .{
            @intFromPtr(fb.*.address.?),
            t,
        });
    }

    const frameAllocator = paging.FRAME_ALLOCATOR.init(memmap, &p);
    _ = frameAllocator;

    std.log.info("nothing more to do, bye!", .{});

    while (true) {
        asm volatile ("hlt");
    }
}

const InterruptStackFrame = packed struct(u320) {
    instructionPointer: u64,
    codeSegment: u16,
    _0: u48,
    flags: u64,
    stackPointer: u64,
    stackSegment: u16,
    _1: u48,
};

fn breakpointHandler(
    stackFrame: *InterruptStackFrame,
) callconv(.{ .x86_64_interrupt = .{} }) void {
    std.log.debug("breakpoint hit at rip 0x{x}, rsp 0x{x}", .{ stackFrame.instructionPointer, stackFrame.stackPointer });
}

fn generalProtectionFault(
    stackFrame: *InterruptStackFrame,
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
    stackFrame: *InterruptStackFrame,
    errorCode: u64,
) callconv(.{ .x86_64_interrupt = .{} }) noreturn {
    std.log.err("page fault at rip 0x{x}, rsp 0x{x}, error code: 0x{x}", .{
        stackFrame.instructionPointer,
        stackFrame.stackPointer,
        errorCode,
    });
    @panic("page fault");
}
