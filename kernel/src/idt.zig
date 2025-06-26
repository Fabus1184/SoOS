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
    instructionPointer: u64,
    codeSegment: u16,
    _0: u48,
    flags: u64,
    stackPointer: u64,
    stackSegment: u16,
    _1: u48,
};
