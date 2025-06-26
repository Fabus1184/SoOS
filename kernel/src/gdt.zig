const std = @import("std");

const TSS = @import("tss.zig").TSS;

pub const SegmentSelector = packed struct(u16) {
    rpl: Ring,
    ti: u1 = 0, // Table Indicator, 0 for GDT, 1 for LDT
    index: u13,
};

const Gdtr = packed struct(u80) {
    size: u16,
    offset: u64,
};

pub const GDT = struct {
    entries: [16]u64 align(16) = .{0} ** 16,
    entries_count: u16 = 1,

    pub fn init() GDT {
        return GDT{};
    }

    pub fn addSegment(self: *GDT, ring: Ring, executable: bool) SegmentSelector {
        self.entries[self.entries_count] = @bitCast(SegmentDescriptor{
            .access = .{
                .read_write = 1,
                .direction_conforming = 0,
                .executable = executable,
                .s = 1,
                .ring = ring,
                .present = 1,
            },
            .flags = .{},
        });
        self.entries_count += 1;

        return SegmentSelector{
            .rpl = ring,
            .ti = 0, // GDT
            .index = @intCast(self.entries_count - 1),
        };
    }

    pub fn addSystemSegment(self: *GDT, tss: *const TSS) SegmentSelector {
        const ptr: u64 = @intFromPtr(tss);

        const descriptor = SystemSegmentDescriptor{
            .limit_low = @intCast(((@bitSizeOf(TSS) / 8) - 1) & 0xFFFF),
            .base_low0 = @intCast(ptr & 0xFFFF),
            .base_low1 = @intCast((ptr >> 16) & 0xFF),
            .access = .{
                .type_ = .tssAvailable,
                .dpl = Ring.ring0,
            },
            .flags = .{
                .longMode = 0,
                .sizeFlag = 0,
                .granularity = 0,
            },
            .limit_high = @intCast((((@bitSizeOf(TSS) / 8) - 1) >> 16) & 0xF),
            .base_mid = @intCast((ptr >> 24) & 0xFF),
            .base_high = @intCast(ptr >> 32),
        };

        const value = @as(u128, @bitCast(descriptor));
        self.entries[self.entries_count] = @intCast(value & 0xFFFFFFFFFFFFFFFF);
        self.entries[self.entries_count + 1] = @intCast(value >> 64);

        self.entries_count += 2;

        return SegmentSelector{
            .rpl = Ring.ring0,
            .ti = 0, // GDT
            .index = @intCast(self.entries_count - 2),
        };
    }

    pub fn load(self: *GDT) void {
        const gdtr = Gdtr{
            .size = @sizeOf(@TypeOf(self.entries)) - 1,
            .offset = @intFromPtr(&self.entries),
        };

        asm volatile (
            \\ lgdt (%[gdt_ptr])
            :
            : [gdt_ptr] "r" (&gdtr),
            : "memory"
        );
    }

    pub fn reloadSegments(codeSegment: SegmentSelector, dataSegment: SegmentSelector) void {
        std.log.debug("reloading code segment: {x}", .{@as(u16, @bitCast(codeSegment))});

        // reload code segment register using a far return
        asm volatile (
        // 6 byte padding to ensure the stack is still aligned
            \\ sub $6, %%rsp
            // push 2 byte segment selector
            \\ pushw %[code_segment]
            \\ lea .finish(%%rip), %%rax
            // push 8 byte return address
            \\ push %%rax
            \\ lretq
            \\ .finish:
            :
            : [code_segment] "r" (codeSegment),
            : "rax", "memory"
        );

        // reload data segment registers
        asm volatile (
            \\ movw %%ax, %%ds
            \\ movw %%ax, %%es
            \\ movw %%ax, %%fs
            \\ movw %%ax, %%gs
            \\ movw %%ax, %%ss
            :
            : [data_segment] "{ax}" (dataSegment),
            : "ax", "memory"
        );

        std.log.debug("reloaded data segment: {x}", .{@as(u16, @bitCast(dataSegment))});
    }
};

const Ring = enum(u2) {
    ring0 = 0b00,
    ring1 = 0b01,
    ring2 = 0b10,
    ring3 = 0b11,
};

const SegmentDescriptor = packed struct(u64) {
    limit_low: u16 = 0xFFFF,
    base_low: u16 = 0,
    base_middle: u8 = 0,
    access: packed struct(u8) {
        accessed: u1 = 1,
        read_write: u1,
        direction_conforming: u1,
        executable: bool,
        s: u1, // 0 for system segment, 1 for code/data segment
        ring: Ring,
        present: u1,
    },
    limit_high: u4 = 0b1111,
    flags: Flags,
    base_high: u8 = 0,
};

const Flags = packed struct(u4) {
    reserved: u1 = 0, // Reserved bits
    longMode: u1 = 1, // Long mode (64-bit)
    sizeFlag: u1 = 0, // 0 for 16-bit segment, 1 for 32-bit segment
    granularity: u1 = 1, // Granularity
};

const SystemSegmentDescriptor = packed struct(u128) {
    limit_low: u16,
    base_low0: u16,
    base_low1: u8,
    access: packed struct(u8) {
        type_: enum(u4) {
            ldt = 0x2, // Local Descriptor Table
            tssAvailable = 0x9, // 64-bit TSS available
            tssBusy = 0xB, // 64-bit TSS busy
        },
        s: u1 = 0, // 0 for system segment, 1 for code/data segment
        dpl: Ring,
        present: u1 = 1, // Segment present
    },
    limit_high: u4,
    flags: Flags,
    base_mid: u8,
    base_high: u32,
    _0: u32 = 0, // Reserved bits
};
