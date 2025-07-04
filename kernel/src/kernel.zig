const std = @import("std");

const pic = @import("pic.zig");
const Term = @import("term/term.zig").Term;
const idt = @import("idt.zig");
const GDT = @import("gdt.zig").GDT;
const TSS = @import("tss.zig").TSS;
const paging = @import("paging.zig");
const limine = @import("limine.zig");
const cpuid = @import("cpuid.zig");
const types = @import("types.zig");
const serial = @import("serial.zig");
const process = @import("process/process.zig");
const Scheduler = @import("process/scheduler.zig").Scheduler;

extern const _START_KERNEL_MEMORY: *anyopaque;
extern const _END_KERNEL_MEMORY: *anyopaque;

const _KERNEL_STACK_SIZE: usize = 0x1_000_000;
var _KERNEL_STACK: [_KERNEL_STACK_SIZE]u8 align(16) = undefined;

const _INTERRUPT_STACK_SIZE: usize = 0x1_000_000;
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
    std.log.debug("interrupt stack is at 0x{x}..0x{x}", .{
        @intFromPtr(&_INTERRUPT_STACK[0]),
        @intFromPtr(&_INTERRUPT_STACK[0]) + _INTERRUPT_STACK_SIZE,
    });
    std.log.debug("kernel heap is at 0x{x}..0x{x}", .{
        @intFromPtr(&_KERNEL_HEAP[0]),
        @intFromPtr(&_KERNEL_HEAP[0]) + _KERNEL_HEAP_SIZE,
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

    const xsaveAreaSize = cpuid.cpuid(0xD, 0x0).ecx;
    std.log.debug("xsave area size: {d}", .{xsaveAreaSize});

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
    IDT.setExceptionHandler(.generalProtectionFault, .ring0, generalProtectionFaultHandler, kernelCodeSegment);
    IDT.setExceptionHandler(.pageFault, .ring0, pageFaultHandler, kernelCodeSegment);
    IDT.setInterruptHandler(.invalidOpcode, .ring0, invalidOpcodeHandler, kernelCodeSegment);

    IDT.setIrqHandler(0x20 + 0, .ring0, irqHandler(0), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 1, .ring0, irqHandler(1), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 2, .ring0, irqHandler(2), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 3, .ring0, irqHandler(3), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 4, .ring0, irqHandler(4), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 5, .ring0, irqHandler(5), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 6, .ring0, irqHandler(6), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 7, .ring0, irqHandler(7), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 8, .ring0, irqHandler(8), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 9, .ring0, irqHandler(9), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 10, .ring0, irqHandler(10), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 11, .ring0, irqHandler(11), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 12, .ring0, irqHandler(12), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 13, .ring0, irqHandler(13), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 14, .ring0, irqHandler(14), kernelCodeSegment);
    IDT.setIrqHandler(0x20 + 15, .ring0, irqHandler(15), kernelCodeSegment);

    IDT.setIrqHandler(0x80, .ring3, syscallHandler, kernelCodeSegment);

    IDT.load();

    pic.init();

    @breakpoint();

    std.log.debug("returned after breakpoint handler", .{});

    KERNEL_PAGING = paging.OffsetPageTable.fromCurrent(limine.LIMINE_HHDM_REQUEST.response.*.offset);
    KERNEL_PAGING.load();

    const frameAllocator = paging.FRAME_ALLOCATOR.init(memmap, &KERNEL_PAGING);

    KERNEL_PAGING.keep(&.{
        .{ limine.LIMINE_HHDM_REQUEST.response.*.offset, limine.LIMINE_HHDM_REQUEST.response.*.offset + 0x800000000 },
        .{ @intFromPtr(&_START_KERNEL_MEMORY), @intFromPtr(&_END_KERNEL_MEMORY) },
        .{
            @intFromPtr(framebuffer.*.address.?),
            @intFromPtr(framebuffer.*.address.?) + framebuffer.*.width * framebuffer.*.height * 4,
        },
    }, frameAllocator);
    KERNEL_PAGING.load();
    std.log.debug("paging initialized", .{});

    const serial0 = serial.SerialPort.serial0();
    try serial0.init();
    try std.fmt.format(serial0, "serial port intialized!\n", .{});

    SCHEDULER.init(KERNEL_ALLOCATOR.allocator());
    try SCHEDULER.add(try process.Process.initKernel(
        KERNEL_ALLOCATOR.allocator(),
        &KERNEL_PAGING,
        kernelCodeSegment,
        kernelDataSegment,
        @intFromPtr(&_KERNEL_STACK[0]) + _KERNEL_STACK_SIZE & ~@as(u64, 0xF),
        kernelProcess,
    ));
    try SCHEDULER.add(try process.Process.initUser(
        KERNEL_ALLOCATOR.allocator(),
        @embedFile("userspace-build/x86_64-unknown-soos/debug/sosh"),
        &KERNEL_PAGING,
        frameAllocator,
        userCodeSegment,
        userDataSegment,
        "sosh",
    ));

    try SCHEDULER.schedule();
}

pub var KERNEL_PAGING: paging.OffsetPageTable = undefined;

const KERNEL_PROCESS_STACK_SIZE: usize = 0x100_000;
var _KERNEL_PROCESS_STACK: [KERNEL_PROCESS_STACK_SIZE]u8 align(16) = undefined;
fn kernelProcess() callconv(.naked) noreturn {
    asm volatile (
        \\ call kernelWorker
    );
}
export fn kernelWorker() noreturn {
    while (true) {
        for (SCHEDULER.processes.items) |*p| {
            if (p.state == .syscall) {
                p.handleSyscall() catch |err| {
                    std.log.err("process {s} failed to handle syscall: {}", .{ p.name, err });
                    @panic("process failed to handle syscall");
                };
            }
        }

        // yield to the scheduler
        asm volatile (
            \\ int $0x80
            :
            : [_] "{rdi}" (types.SYSCALL_YIELD),
        );
    }
}

var KERNEL_ALLOCATOR = std.heap.FixedBufferAllocator.init(&_KERNEL_HEAP);
var SCHEDULER: Scheduler = undefined;

fn breakpointHandler(
    stackFrame: *idt.InterruptStackFrame,
) callconv(.{ .x86_64_interrupt = .{} }) void {
    std.log.debug("breakpoint hit at rip 0x{x}, rsp 0x{x}", .{ stackFrame.rip, stackFrame.rsp });
}

fn generalProtectionFaultHandler(
    stackFrame: *idt.InterruptStackFrame,
    errorCode: u64,
) callconv(.{ .x86_64_interrupt = .{} }) noreturn {
    std.log.err("general protection fault at rip 0x{x}, rsp 0x{x}, error code: 0x{x}", .{
        stackFrame.rip,
        stackFrame.rsp,
        errorCode,
    });
    @panic("general protection fault");
}

fn pageFaultHandler(
    stackFrame: *idt.InterruptStackFrame,
    errorCode: u64,
) callconv(.{ .x86_64_interrupt = .{} }) noreturn {
    KERNEL_PAGING.load();

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
        stackFrame.rip,
        stackFrame.rsp,
        @as(Error, @bitCast(errorCode)),
        address,
    });

    SCHEDULER.abort() catch |err| {
        std.log.err("failed to abort scheduler: {}", .{err});
        @panic("failed to abort scheduler");
    };

    SCHEDULER.schedule() catch |err| {
        std.log.err("scheduler returned with an error: {}", .{err});
        @panic("scheduler returned with an error");
    };
}

fn invalidOpcodeHandler(
    stackFrame: *idt.InterruptStackFrame,
) callconv(.{ .x86_64_interrupt = .{} }) noreturn {
    std.log.err("invalid opcode at rip 0x{x}, rsp 0x{x}", .{
        stackFrame.rip,
        stackFrame.rsp,
    });
    @panic("invalid opcode");
}

fn syscallHandler(
    stackFrame: *idt.InterruptStackFrame,
    state: *idt.State,
) callconv(.c) noreturn {
    KERNEL_PAGING.load();

    if (state.registers.rdi == types.SYSCALL_YIELD) {
        SCHEDULER.storeState(state, stackFrame, .ready) catch |err| {
            std.log.err("failed to store state: {}", .{err});
            @panic("failed to store state");
        };
    } else {
        SCHEDULER.storeState(state, stackFrame, .syscall) catch |err| {
            std.log.err("failed to store state: {}", .{err});
            @panic("failed to store state");
        };
    }

    SCHEDULER.schedule() catch |err| {
        std.log.err("scheduler returned with an error: {}", .{err});
        @panic("scheduler returned with an error");
    };
}

pub fn irqHandler(comptime irq: u8) *const fn (
    stackFrame: *idt.InterruptStackFrame,
    state: *idt.State,
) callconv(.c) noreturn {
    return struct {
        fn f(
            stackFrame: *idt.InterruptStackFrame,
            state: *idt.State,
        ) callconv(.c) noreturn {
            // Acknowledge the interrupt
            pic.endOfInterrupt(irq);

            switch (irq) {
                // IRQ 4 (COM1)
                4 => {
                    const serial0 = serial.SerialPort.serial0();
                    var buffer: [8]u8 = undefined;
                    const count = serial0.readNonBlocking(&buffer);

                    var i: usize = 0;
                    while (i < count) {
                        const len = std.unicode.utf8ByteSequenceLength(buffer[i]) catch |err| {
                            std.log.err("invalid UTF-8 sequence at index {}: {}", .{ i, err });
                            i += 1; // Skip invalid byte
                            continue;
                        };

                        if (len == 1) {
                            switch (buffer[i]) {
                                '\r' => serial0.writeAll("\r\n") catch unreachable,
                                '\x08' => serial0.writeAll("\x08 \x08") catch unreachable,
                                else => serial0.write(buffer[i]),
                            }
                        } else {
                            serial0.writeAll(buffer[i .. i + len]) catch unreachable;
                        }

                        i += len;
                    }

                    for (SCHEDULER.processes.items) |*p| {
                        if (p.stdin) |*stdin| {
                            stdin.writeSlice(buffer[0..count]) catch |err| {
                                std.log.err("failed to write to stdin: {}", .{err});
                                @panic("failed to write to stdin");
                            };
                        }
                    }
                },
                else => {},
            }

            process.iret(stackFrame, state);
        }
    }.f;
}
