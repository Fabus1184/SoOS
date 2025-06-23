const SegmentSelector = @import("gdt.zig").SegmentSelector;

pub const TSS = packed struct(u832) {
    _0: u32,
    rsp0: u64, // Ring 0 stack pointer
    rsp1: u64, // Ring 1 stack pointer
    rsp2: u64, // Ring 2 stack pointer
    _1: u64,
    ist1: u64, // Interrupt Stack Table 1
    ist2: u64, // Interrupt Stack Table 2
    ist3: u64, // Interrupt Stack Table 3
    ist4: u64, // Interrupt Stack Table 4
    ist5: u64, // Interrupt Stack Table 5
    ist6: u64, // Interrupt Stack Table 6
    ist7: u64, // Interrupt Stack Table 7
    _2: u64,
    _3: u16,
    io_map_base: u16, // I/O map base address

    pub fn zero() TSS {
        return TSS{
            ._0 = 0,
            .rsp0 = 0,
            .rsp1 = 0,
            .rsp2 = 0,
            ._1 = 0,
            .ist1 = 0,
            .ist2 = 0,
            .ist3 = 0,
            .ist4 = 0,
            .ist5 = 0,
            .ist6 = 0,
            .ist7 = 0,
            ._2 = 0,
            ._3 = 0,
            .io_map_base = @sizeOf(TSS),
        };
    }

    pub fn load(systemSegmentSelector: SegmentSelector) void {
        asm volatile (
            \\ ltr %[tss_selector]
            :
            : [tss_selector] "r" (systemSegmentSelector),
            : "memory"
        );
    }
};
