const std = @import("std");

const gdt = @import("gdt.zig");

const Idtr = packed struct(u80) {
    size: u16,
    offset: u64,
};

pub const IDT = struct {
    entries: [256]GateDescriptor align(16),

    /// regular interrupts without error code
    pub const Interrupt = enum(u8) {
        divideByZero = 0,
        debug = 1,
        nonMaskableInterrupt = 2,
        breakpoint = 3,
        overflow = 4,
        boundRangeExceeded = 5,
        invalidOpcode = 6,
        deviceNotAvailable = 7,
        coprocessorSegmentOverrun = 9,
        intelReserved = 15,
        x87FloatingPointException = 16,
        machineCheck = 18,
        simdFloatingPointException = 19,
        virtualizationException = 20,
    };

    /// exceptions with error code
    pub const Exception = enum(u8) {
        doubleFault = 8,
        invalidTss = 10,
        segmentNotPresent = 11,
        stackSegmentFault = 12,
        generalProtectionFault = 13,
        pageFault = 14,
        x87FloatingPointError = 16,
        controlProtectionException = 21,
    };

    pub fn init() IDT {
        return IDT{
            .entries = .{GateDescriptor.empty()} ** 256,
        };
    }

    pub fn load(self: *IDT) void {
        const idtr = Idtr{
            .size = @sizeOf(@TypeOf(self.entries)) - 1,
            .offset = @intFromPtr(&self.entries),
        };

        asm volatile (
            \\ lidt (%[idt_ptr])
            :
            : [idt_ptr] "r" (&idtr),
            : "memory"
        );
    }

    pub fn setInterruptHandler(
        self: *IDT,
        interrupt: Interrupt,
        ring: Ring,
        handler: ?*const fn (*InterruptStackFrame) callconv(.{ .x86_64_interrupt = .{} }) void,
        segmentSelector: gdt.SegmentSelector,
    ) void {
        const offset: u64 = @intFromPtr(handler);
        self.entries[@intFromEnum(interrupt)] = GateDescriptor{
            .offsetLow = @intCast(offset & 0xFFFF),
            .segmentSelector = segmentSelector,
            .ist = 1,
            .gateType = .interrupt,
            .dpl = ring,
            .present = if (handler) |_| 1 else 0,
            .offsetMiddle = @intCast((offset >> 16) & 0xFFFF),
            .offsetHigh = @intCast(offset >> 32),
        };
    }

    pub fn setExceptionHandler(
        self: *IDT,
        exception: Exception,
        ring: Ring,
        handler: ?*const fn (*InterruptStackFrame, u64) callconv(.{ .x86_64_interrupt = .{} }) void,
        segmentSelector: gdt.SegmentSelector,
    ) void {
        self.entries[@intFromEnum(exception)] = GateDescriptor{
            .offsetLow = @intCast(@intFromPtr(handler) & 0xFFFF),
            .segmentSelector = segmentSelector,
            .ist = 1,
            .gateType = .trap,
            .dpl = ring,
            .present = if (handler) |_| 1 else 0,
            .offsetMiddle = @intCast((@intFromPtr(handler) >> 16) & 0xFFFF),
            .offsetHigh = @intCast(@intFromPtr(handler) >> 32),
        };
    }

    pub fn setIrqHandler(
        self: *IDT,
        irq: u8,
        ring: Ring,
        comptime handler: ?*const fn (*InterruptStackFrame, *State) callconv(.c) void,
        segmentSelector: gdt.SegmentSelector,
    ) void {
        if (irq < 0x20 or irq > 0x100) {
            @panic("IRQ must be in the range 0x20 to 0x100");
        }

        const f = &struct {
            fn f() callconv(.naked) void {
                // currently on the stack is the interrupt stack frame
                asm volatile (
                    \\
                    // align State
                    \\ subq $0x30, %rsp
                    \\ pushq %r15
                    \\ pushq %r14
                    \\ pushq %r13
                    \\ pushq %r12
                    \\ pushq %r11
                    \\ pushq %r10
                    \\ pushq %r9
                    \\ pushq %r8
                    \\ pushq %rbp
                    \\ pushq %rdi
                    \\ pushq %rsi
                    \\ pushq %rdx
                    \\ pushq %rcx
                    \\ pushq %rbx
                    \\ pushq %rax
                    ::: "memory");

                asm volatile (
                    \\
                    // push 4096 byte state
                    \\ subq $0x1000, %rsp
                    \\ mov $~0, %eax
                    \\ mov $~0, %edx
                    \\ xsave 0(%rsp)
                    ::: "memory");

                asm volatile (
                    \\
                    // set up arguments for the handler
                    \\ mov %rsp, %rdi
                    // adjust stack pointer to account for the pushed State
                    \\ addq $0x10A8, %rdi
                    \\ mov %rsp, %rsi
                    \\ call *%[handler]
                    :
                    : [handler] "r" (handler),
                );

                comptime {
                    // assert no padding in State
                    std.debug.assert(@sizeOf(State) == @bitSizeOf(State) / 8);
                }
            }
        }.f;

        const offset: u64 = @intFromPtr(f);
        self.entries[irq] = GateDescriptor{
            .offsetLow = @intCast(offset & 0xFFFF),
            .segmentSelector = segmentSelector,
            .ist = 1,
            .gateType = .interrupt,
            .dpl = ring,
            .present = if (handler) |_| 1 else 0,
            .offsetMiddle = @intCast((offset >> 16) & 0xFFFF),
            .offsetHigh = @intCast(offset >> 32),
        };
    }
};

const Ring = enum(u2) {
    ring0 = 0b00,
    ring1 = 0b01,
    ring2 = 0b10,
    ring3 = 0b11,
};

const GateDescriptor = packed struct(u128) {
    offsetLow: u16,
    segmentSelector: gdt.SegmentSelector,
    ist: u3 = 0,
    _1: u5 = 0,
    gateType: enum(u4) {
        interrupt = 0b1110,
        trap = 0b1111,
    },
    zero: u1 = 0,
    dpl: Ring = .ring0,
    present: u1,
    offsetMiddle: u16,
    offsetHigh: u32,
    _2: u32 = 0,

    fn empty() GateDescriptor {
        return GateDescriptor{
            .offsetLow = 0,
            .segmentSelector = .{
                .rpl = .ring0,
                .index = 0,
            },
            .gateType = .interrupt,
            .present = 0,
            .offsetMiddle = 0,
            .offsetHigh = 0,
        };
    }
};

pub const InterruptStackFrame = packed struct(u320) {
    rip: u64,
    codeSegment: u16,
    _0: u48 = 0,
    flags: u64,
    rsp: u64,
    stackSegment: u16,
    _1: u48 = 0,

    pub fn format(self: InterruptStackFrame, comptime _: []const u8, _: std.fmt.FormatOptions, writer: anytype) !void {
        try std.fmt.format(writer, "InterruptStackFrame {{ ", .{});
        inline for (std.meta.fields(@TypeOf(self))) |field| {
            try std.fmt.format(writer, ".{s} = 0x{x}, ", .{ field.name, @field(self, field.name) });
        }
        try std.fmt.format(writer, "}}", .{});
    }
};

pub const State = extern struct {
    xsave: [4096]u8 align(64) = .{0} ** 4096,

    registers: extern struct {
        rax: u64 = 0xFAAF_6969_FEEF_6969,
        rbx: u64 = 0xFAAF_6969_FEEF_6969,
        rcx: u64 = 0xFAAF_6969_FEEF_6969,
        rdx: u64 = 0xFAAF_6969_FEEF_6969,
        rsi: u64 = 0xFAAF_6969_FEEF_6969,
        rdi: u64 = 0xFAAF_6969_FEEF_6969,
        rbp: u64 = 0xFAAF_6969_FEEF_6969,
        r8: u64 = 0xFAAF_6969_FEEF_6969,
        r9: u64 = 0xFAAF_6969_FEEF_6969,
        r10: u64 = 0xFAAF_6969_FEEF_6969,
        r11: u64 = 0xFAAF_6969_FEEF_6969,
        r12: u64 = 0xFAAF_6969_FEEF_6969,
        r13: u64 = 0xFAAF_6969_FEEF_6969,
        r14: u64 = 0xFAAF_6969_FEEF_6969,
        r15: u64 = 0xFAAF_6969_FEEF_6969,
    } = .{},

    pub fn format(self: State, comptime _: []const u8, _: std.fmt.FormatOptions, writer: anytype) !void {
        try std.fmt.format(writer, "State {{ .registers = {{ ", .{});
        inline for (std.meta.fields(@TypeOf(self.registers))) |field| {
            try std.fmt.format(writer, ".{s} = 0x{x}, ", .{ field.name, @field(self.registers, field.name) });
        }
        try std.fmt.format(writer, "}} }}", .{});
    }
};
