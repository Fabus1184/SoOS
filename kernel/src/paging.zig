const std = @import("std");

const limine = @import("limine.zig");

pub var FRAME_ALLOCATOR: FrameAllocator = undefined;

pub const FrameAllocator = struct {
    const Area = struct {
        start: u64,
        size: u64,
        bitmapOffset: usize,
    };

    const MAX_FRAMES = 0x1_000_000; // 1 million frames, 4GB of memory
    /// bitmap for 4k physical frames
    globalBitmap: [MAX_FRAMES]bool,
    globalBitmapLen: usize = 0,

    areas: [32]Area,
    areasLen: usize = 0,

    pub fn init(self: *FrameAllocator, memmap: []limine.raw.limine_memmap_entry, paging: *const OffsetPageTable) *FrameAllocator {
        // Initialize areas based on memory map
        for (memmap) |entry| {
            if (entry.type == limine.raw.LIMINE_MEMMAP_USABLE or entry.type == limine.raw.LIMINE_MEMMAP_KERNEL_AND_MODULES) {
                const areaSize = entry.length;
                const areaStart = entry.base;

                if (areaSize < 0x1000) {
                    std.log.warn("skipping too small memory area: 0x{x} bytes at 0x{x}", .{ areaSize, areaStart });
                    continue; // skip too small areas
                }

                self.areas[self.areasLen] = Area{
                    .start = areaStart,
                    .size = areaSize,
                    .bitmapOffset = self.globalBitmapLen,
                };
                self.areasLen += 1;
                self.globalBitmapLen += @intCast(areaSize / 0x1000); // number of frames in this area
            }
        }

        var it = paging.iterator();
        while (it.next()) |mapping| {
            const physAddr = mapping.physicalAddress;

            const area = for (self.areas[0..self.areasLen]) |*a| {
                if (physAddr >= a.start and physAddr < a.start + a.size) {
                    break a;
                }
            } else {
                // mapping is not in a usable area
                continue;
            };

            if (mapping.size.size() > area.size) {
                std.log.warn("FrameAllocator: mapped size 0x{x} exceeds area size 0x{x} at 0x{x}", .{
                    mapping.size.size(),
                    area.size,
                    area.start,
                });
                continue; // skip too large allocations
            }

            for (0..mapping.size.size() / 0x1000) |i| {
                const frameIndex = (physAddr - area.start) / 0x1000 + i;
                const bit = &self.globalBitmap[area.bitmapOffset + frameIndex];
                std.debug.assert(!bit.*);
                bit.* = true; // mark as allocated
            }
        }

        var used: u64 = 0;
        for (self.globalBitmap[0..self.globalBitmapLen]) |bit| {
            if (bit) used += 1;
        }
        std.log.debug("frame allocator initialized: used {}/{} frames ({}/{} MiB)", .{
            used,
            self.globalBitmapLen,
            used * 0x1000 / 0x100000,
            self.globalBitmapLen * 0x1000 / 0x100000,
        });

        return self;
    }

    pub fn allocateFrame(self: *FrameAllocator, size: FrameSize) ?u64 {
        const areas = self.areas[0..self.areasLen];

        const physFrameCount = size.size() / 0x1000; // number of physical frames needed

        // find the first area that has enough space
        for (areas) |area| {
            const bitmapSize = area.size / 0x1000;
            const bitmap = area.bitmap[0..bitmapSize];

            var freeCount: usize = 0;
            var startIndex: usize = 0;

            for (bitmap, 0..) |bit, index| {
                if (!bit) {
                    if (freeCount == 0) {
                        startIndex = index;
                    }
                    freeCount += 1;
                } else {
                    freeCount = 0;
                }

                if (freeCount == physFrameCount) {
                    // found enough free frames
                    for (startIndex..startIndex + freeCount) |i| {
                        bitmap[i] = true; // mark as allocated
                    }
                    return area.start + 0x1000 * startIndex;
                }
            }
        }

        return null;
    }

    pub fn deallocateFrame(self: *FrameAllocator, physAddr: u64, size: FrameSize) void {
        const areas = self.areas[0..self.areasLen];

        std.debug.assert(physAddr % 0x1000 == 0);

        const physFrameCount = size.size() / 0x1000; // number of physical frames needed

        for (areas) |area| {
            if (physAddr >= area.start and physAddr < area.start + area.size) {
                const offset = (physAddr - area.start) / 0x1000;

                const bitmapSize = area.size / 0x1000;
                const bitmap = self.globalBitmap[area.bitmapOffset .. area.bitmapOffset + bitmapSize];

                for (0..physFrameCount) |i| {
                    std.debug.assert(bitmap[offset + i]);

                    bitmap[offset + i] = false; // mark as free
                }
                return;
            }
        }
    }
};

pub const FrameSize = enum(u64) {
    @"4KiB",
    @"2MiB",
    @"1GiB",

    pub fn size(self: FrameSize) u64 {
        return switch (self) {
            .@"4KiB" => 0x1000,
            .@"2MiB" => 0x200000,
            .@"1GiB" => 0x40000000,
        };
    }
};

pub const PageTableEntry = packed struct(u64) {
    present: bool,
    writable: bool,
    userAccessible: bool,
    writeThrough: bool,
    cacheDisabled: bool,
    accessed: bool,
    dirty: bool,
    hugePage: bool,
    global: bool,
    customBit1: bool,
    customBit2: bool,
    customBit3: bool,
    address: u40,
    customBits4: bool,
    customBits5: bool,
    customBits6: bool,
    customBits7: bool,
    customBits8: bool,
    customBits9: bool,
    customBits10: bool,
    customBits11: bool,
    customBits12: bool,
    customBits13: bool,
    customBits14: bool,
    noExecute: bool,
};

/// physical addresses of frames are mapped at
/// the specified offset into the virtual address space
pub const OffsetPageTable = struct {
    offset: u64,

    pageTable: [512]PageTableEntry align(0x1000) = undefined,

    pub fn fromCurrent(offset: u64) OffsetPageTable {
        var self = OffsetPageTable{
            .offset = offset,
            .pageTable = undefined,
        };

        var current: u64 = undefined;
        asm volatile (
            \\ movq %%cr3, %[current]
            : [current] "=r" (current),
            :
            : "memory"
        );

        const ptr: *align(0x1000) const [512]PageTableEntry = @ptrFromInt(self.frameVirtualAddress(current));

        @memcpy(&self.pageTable, ptr);

        return self;
    }

    pub fn load(self: *const OffsetPageTable) void {
        const translation = self.translate(@intFromPtr(&self.pageTable));
        std.debug.assert(translation != null);
        std.debug.assert(translation.?.size == .@"4KiB");
        std.debug.assert(translation.?.physicalAddress % 0x1000 == 0);

        asm volatile (
            \\ movq %[page_table], %%cr3
            :
            : [page_table] "r" (translation.?.physicalAddress),
            : "memory"
        );
    }

    fn frameVirtualAddress(self: OffsetPageTable, frameAddress: u64) u64 {
        return frameAddress + self.offset;
    }

    pub fn translate(self: *const OffsetPageTable, virtualAddress: u64) ?Mapping {
        const l4Index = (virtualAddress >> 39) & 0x1FF;
        const l4Entry = self.pageTable[l4Index];
        if (!l4Entry.present) {
            return null; // not present
        }

        const l3Table: *align(0x1000) [512]PageTableEntry = @ptrFromInt(self.frameVirtualAddress(l4Entry.address << 12));
        const l3Index = (virtualAddress >> 30) & 0x1FF;
        const l3Entry = &l3Table[l3Index];
        if (!l3Entry.present) {
            return null; // not present
        }

        if (l3Entry.hugePage) {
            return Mapping{
                .virtualAddress = makeCanonical((l4Index << 39) | (l3Index << 30)),
                .physicalAddress = l3Entry.address << 12,
                .size = .@"1GiB",
                .entry = l3Entry,
            };
        }

        const l2Table: *align(0x1000) [512]PageTableEntry = @ptrFromInt(self.frameVirtualAddress(l3Entry.address << 12));
        const l2Index = (virtualAddress >> 21) & 0x1FF;
        const l2Entry = &l2Table[l2Index];
        if (!l2Entry.present) {
            return null; // not present
        }

        if (l2Entry.hugePage) {
            return Mapping{
                .virtualAddress = makeCanonical((l4Index << 39) | (l3Index << 30) | (l2Index << 21)),
                .physicalAddress = l2Entry.address << 12,
                .size = .@"2MiB",
                .entry = l2Entry,
            };
        }

        const l1Table: *align(0x1000) [512]PageTableEntry = @ptrFromInt(self.frameVirtualAddress(l2Entry.address << 12));
        const l1Index = (virtualAddress >> 12) & 0x1FF;
        const l1Entry = &l1Table[l1Index];
        if (!l1Entry.present) {
            return null; // not present
        }

        return Mapping{
            .virtualAddress = makeCanonical((l4Index << 39) | (l3Index << 30) | (l2Index << 21) | (l1Index << 12)),
            .physicalAddress = l1Entry.address << 12,
            .size = .@"4KiB",
            .entry = l1Entry,
        };
    }

    pub fn keep(self: *OffsetPageTable, rangesInclusive: []const struct { u64, u64 }, frameAllocator: *FrameAllocator) void {
        var it = self.iterator();
        while (it.next()) |mapping| {
            for (rangesInclusive) |range| {
                if (mapping.virtualAddress >= range[0] and mapping.virtualAddress <= range[1]) {
                    // this mapping is within the range, keep it
                    break;
                }
            } else {
                // this mapping is not within any range, deallocate it
                mapping.entry.present = false;
                frameAllocator.deallocateFrame(mapping.physicalAddress, mapping.size);
                invalidatePage(mapping.virtualAddress);
            }
        }
    }

    pub fn iterator(self: *const OffsetPageTable) EntryIterator {
        return EntryIterator{ .pageTable = self, .l4Index = 0, .l3Index = 0, .l2Index = 0, .l1Index = 0 };
    }

    const EntryIterator = struct {
        pageTable: *const OffsetPageTable,
        l4Index: usize = 0,
        l3Index: usize = 0,
        l2Index: usize = 0,
        l1Index: usize = 0,

        pub fn next(self: *EntryIterator) ?Mapping {
            for (self.l4Index..512) |l4| {
                const l4Table: *align(0x1000) const [512]PageTableEntry = &self.pageTable.pageTable;
                const l4Entry = l4Table[l4];
                if (!l4Entry.present) {
                    continue; // skip non-present entries
                }

                for (self.l3Index..512) |l3| {
                    const l3Table: *align(0x1000) [512]PageTableEntry = @ptrFromInt(self.pageTable.frameVirtualAddress(l4Entry.address << 12));
                    const l3Entry = &l3Table[l3];
                    if (!l3Entry.present) {
                        continue; // skip non-present entries
                    }

                    if (l3Entry.hugePage) {
                        self.l4Index = l4 + 1; // move to next l4 entry
                        self.l3Index = 0; // reset l3 index for next l4 entry
                        self.l2Index = 0; // reset l2 index for next l3 entry
                        self.l1Index = 0; // reset l1 index for next l2 entry

                        return Mapping{
                            .virtualAddress = makeCanonical((l4 << 39) | (l3 << 30)),
                            .physicalAddress = l3Entry.address << 12,
                            .size = .@"1GiB",
                            .entry = l3Entry,
                        };
                    }

                    for (self.l2Index..512) |l2| {
                        const l2Table: *align(0x1000) [512]PageTableEntry = @ptrFromInt(self.pageTable.frameVirtualAddress(l3Entry.address << 12));
                        const l2Entry = &l2Table[l2];
                        if (!l2Entry.present) {
                            continue; // skip non-present entries
                        }

                        if (l2Entry.hugePage) {
                            self.l3Index = l3 + 1; // move to next l3 entry
                            self.l2Index = 0; // reset l2 index for next l3 entry
                            self.l1Index = 0; // reset l1 index for next l2 entry

                            return Mapping{
                                .virtualAddress = makeCanonical((l4 << 39) | (l3 << 30) | (l2 << 21)),
                                .physicalAddress = l2Entry.address << 12,
                                .size = .@"2MiB",
                                .entry = l2Entry,
                            };
                        }

                        for (self.l1Index..512) |l1| {
                            const l1Table: *align(0x1000) [512]PageTableEntry = @ptrFromInt(self.pageTable.frameVirtualAddress(l2Entry.address << 12));
                            const l1Entry = &l1Table[l1];
                            if (!l1Entry.present) {
                                continue; // skip non-present entries
                            }

                            self.l1Index = l1 + 1; // move to next l1 entry

                            return Mapping{
                                .virtualAddress = makeCanonical((l4 << 39) | (l3 << 30) | (l2 << 21) | (l1 << 12)),
                                .physicalAddress = l1Entry.address << 12,
                                .size = .@"4KiB",
                                .entry = l1Entry,
                            };
                        }
                    }
                }
            }

            return null; // no more entries
        }
    };
};

fn makeCanonical(value: u64) u64 {
    const signed: i64 = @bitCast(value);
    return @bitCast((signed << 16) >> 16);
}

pub const Mapping = struct {
    virtualAddress: u64,
    physicalAddress: u64,
    size: FrameSize,
    entry: *PageTableEntry,
};

fn invalidatePage(virtualAddress: u64) void {
    asm volatile (
        \\ invlpg %[address]
        :
        : [address] "m" (virtualAddress),
        : "memory"
    );
}
