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

    /// allocate a frame of the specified size that is aligned to the same size
    pub fn allocateFrame(self: *FrameAllocator, size: FrameSize) !u64 {
        const areas = self.areas[0..self.areasLen];
        const frameCount = size.size() / 0x1000; // number of physical frames needed

        for (areas) |area| {
            if (area.size < frameCount * 0x1000) {
                continue; // not enough space in this area
            }

            const bitmapSize = area.size / 0x1000;
            const bitmap = self.globalBitmap[area.bitmapOffset .. area.bitmapOffset + bitmapSize];

            // search with alignment constraint
            var i: usize = 0;
            while (i + frameCount <= bitmapSize) {
                const physAddr = area.start + i * 0x1000;
                if ((physAddr & (size.size() - 1)) != 0) {
                    // not aligned to size, skip to next aligned frame
                    const nextAligned = ((physAddr + size.size() - 1) & ~(size.size() - 1));
                    i = (nextAligned - area.start) / 0x1000;
                    continue;
                }

                // check if frames [i .. i + frameCount) are all free
                var allFree = true;
                for (0..frameCount) |j| {
                    if (bitmap[i + j]) {
                        allFree = false;
                        break;
                    }
                }

                if (allFree) {
                    for (0..frameCount) |j| {
                        bitmap[i + j] = true;
                    }
                    return area.start + i * 0x1000;
                } else {
                    i += 1;
                }
            }
        }

        return error.OutOfMemory;
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
    present: bool = false,
    writable: bool = false,
    userAccessible: bool = false,
    writeThrough: bool = false,
    cacheDisabled: bool = false,
    accessed: bool = false,
    dirty: bool = false,
    hugePage: bool = false,
    global: bool = false,
    customBit1: bool = false,
    customBit2: bool = false,
    customBit3: bool = false,
    address: u40,
    reserved: u11 = 0,
    noExecute: bool = false,

    pub fn format(self: PageTableEntry, comptime fmt: []const u8, options: std.fmt.FormatOptions, writer: anytype) !void {
        _ = .{ fmt, options };

        try std.fmt.format(writer,
            \\
            \\|P|W|U|WT|CD|A|D|HP|G|CB1|CB2|CB3|         Addr.|Res.|NX|
            \\|{b}|{b}|{b}|{b:2}|{b:2}|{b}|{b}|{b:2}|{b}|{b:3}|{b:3}|{b:3}|0x{x:12}|{b:4}|{b:2}|
            \\
        , .{
            @intFromBool(self.present),      @intFromBool(self.writable),      @intFromBool(self.userAccessible),
            @intFromBool(self.writeThrough), @intFromBool(self.cacheDisabled), @intFromBool(self.accessed),
            @intFromBool(self.dirty),        @intFromBool(self.hugePage),      @intFromBool(self.global),
            @intFromBool(self.customBit1),   @intFromBool(self.customBit2),    @intFromBool(self.customBit3),
            @as(u64, self.address) << 12,    self.reserved,                    @intFromBool(self.noExecute),
        });
    }
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

        const ptr: *const [512]PageTableEntry = @ptrCast(self.frameVirtualAddress(current));

        @memcpy(&self.pageTable, ptr);

        return self;
    }

    pub fn clone(self: *const OffsetPageTable) OffsetPageTable {
        var newTable = OffsetPageTable{
            .offset = self.offset,
            .pageTable = undefined,
        };
        @memcpy(&newTable.pageTable, &self.pageTable);
        return newTable;
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

    fn frameVirtualAddress(self: OffsetPageTable, frameAddress: u64) *align(0x1000) [0x1000]u8 {
        if (frameAddress % 0x1000 != 0) {
            std.log.err("frame address 0x{x} is not aligned to 4KiB", .{frameAddress});
            @panic("frame address must be aligned to 4KiB");
        }

        return @ptrFromInt(frameAddress + self.offset);
    }

    pub fn translate(self: *const OffsetPageTable, virtualAddress: u64) ?Mapping {
        const l4Index = (virtualAddress >> 39) & 0x1FF;
        const l4Entry = &self.pageTable[l4Index];
        if (!l4Entry.present) {
            return null; // not present
        }

        const l3Table: *align(0x1000) [512]PageTableEntry = @ptrCast(self.frameVirtualAddress(l4Entry.address << 12));
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
                .entries = .{ l4Entry, l3Entry, null, null },
            };
        }

        const l2Table: *align(0x1000) [512]PageTableEntry = @ptrCast(self.frameVirtualAddress(l3Entry.address << 12));
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
                .entries = .{ l4Entry, l3Entry, l2Entry, null },
            };
        }

        const l1Table: *align(0x1000) [512]PageTableEntry = @ptrCast(self.frameVirtualAddress(l2Entry.address << 12));
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
            .entries = .{ l4Entry, l3Entry, l2Entry, l1Entry },
        };
    }

    pub fn map(
        self: *OffsetPageTable,
        frameAllocator: *FrameAllocator,
        virtualAddress: u64,
        physicalAddress: u64,
        size: FrameSize,
        options: struct {
            writable: bool = true,
            userAccessible: bool = false,
            writeThrough: bool = false,
            cacheDisabled: bool = false,
            global: bool = false,
            noExecute: bool = false,
        },
    ) !void {
        std.debug.assert(virtualAddress % 0x1000 == 0);

        switch (size) {
            .@"4KiB" => {
                std.debug.assert(physicalAddress % 0x1000 == 0);
            },
            .@"2MiB" => {
                std.debug.assert(physicalAddress % 0x200000 == 0);
            },
            .@"1GiB" => {
                std.debug.assert(physicalAddress % 0x40000000 == 0);
            },
        }

        const l4Index = (virtualAddress >> 39) & 0x1FF;
        const l3Index = (virtualAddress >> 30) & 0x1FF;
        const l2Index = (virtualAddress >> 21) & 0x1FF;
        const l1Index = (virtualAddress >> 12) & 0x1FF;

        const l4Entry = &self.pageTable[l4Index];
        if (!l4Entry.present) {
            // allocate a new frame
            const frame = try frameAllocator.allocateFrame(.@"4KiB");
            @memset(self.frameVirtualAddress(frame), 0);

            l4Entry.* = PageTableEntry{
                .present = true,
                .writable = true,
                .userAccessible = true,
                .address = @intCast(frame >> 12),
            };
            invalidatePage(makeCanonical(l4Index << 39));
        }

        // ensure parent flags do not override child flags
        l4Entry.writable = true;
        l4Entry.userAccessible = true;
        l4Entry.noExecute = false;
        invalidatePage(makeCanonical(l4Index << 39));

        const l3Table: *align(0x1000) [512]PageTableEntry = @ptrCast(self.frameVirtualAddress(l4Entry.address << 12));
        const l3Entry = &l3Table[l3Index];
        if (size == .@"1GiB") {
            if (l3Entry.present) {
                return error.MappingAlreadyExists;
            }

            l3Entry.* = PageTableEntry{
                .present = true,
                .writable = options.writable,
                .userAccessible = options.userAccessible,
                .writeThrough = options.writeThrough,
                .cacheDisabled = options.cacheDisabled,
                .hugePage = true,
                .global = options.global,
                //.noExecute = options.noExecute,
                .address = @intCast(physicalAddress >> 12),
            };
            invalidatePage(makeCanonical((l4Index << 39) | (l3Index << 30)));
            return;
        }

        if (!l3Entry.present) {
            // allocate a new frame for the l3 table
            const frame = try frameAllocator.allocateFrame(.@"4KiB");
            @memset(self.frameVirtualAddress(frame), 0);

            l3Entry.* = PageTableEntry{
                .present = true,
                .writable = true,
                .userAccessible = true,
                .address = @intCast(frame >> 12),
            };
            invalidatePage(makeCanonical((l4Index << 39) | (l3Index << 30)));
        }

        // ensure parent flags do not override child flags
        l3Entry.writable = true;
        l3Entry.userAccessible = true;
        l3Entry.noExecute = false;
        invalidatePage(makeCanonical((l4Index << 39) | (l3Index << 30)));

        const l2Table: *align(0x1000) [512]PageTableEntry = @ptrCast(self.frameVirtualAddress(l3Entry.address << 12));
        const l2Entry = &l2Table[l2Index];
        if (size == .@"2MiB") {
            if (l2Entry.present) {
                return error.MappingAlreadyExists;
            }

            l2Entry.* = PageTableEntry{
                .present = true,
                .writable = options.writable,
                .userAccessible = options.userAccessible,
                .writeThrough = options.writeThrough,
                .cacheDisabled = options.cacheDisabled,
                .hugePage = true,
                .global = options.global,
                //.noExecute = options.noExecute,
                .address = @intCast(physicalAddress >> 12),
            };
            invalidatePage(makeCanonical((l4Index << 39) | (l3Index << 30) | (l2Index << 21)));
            return;
        }

        if (!l2Entry.present) {
            // allocate a new frame for the l2 table
            const frame = try frameAllocator.allocateFrame(.@"4KiB");
            @memset(self.frameVirtualAddress(frame), 0);

            l2Entry.* = PageTableEntry{
                .present = true,
                .writable = true,
                .userAccessible = true,
                .writeThrough = false,
                .cacheDisabled = false,
                .hugePage = false,
                .global = false,
                .noExecute = false,
                .address = @intCast(frame >> 12),
            };
            invalidatePage(makeCanonical((l4Index << 39) | (l3Index << 30) | (l2Index << 21)));
        }

        // ensure parent flags do not override child flags
        l2Entry.writable = true;
        l2Entry.userAccessible = true;
        l2Entry.noExecute = false;
        invalidatePage(makeCanonical((l4Index << 39) | (l3Index << 30) | (l2Index << 21)));

        const l1Table: *align(0x1000) [512]PageTableEntry = @ptrCast(self.frameVirtualAddress(l2Entry.address << 12));
        const l1Entry = &l1Table[l1Index];
        if (l1Entry.present) {
            return error.MappingAlreadyExists; // already mapped
        }

        l1Entry.* = PageTableEntry{
            .present = true,
            .writable = options.writable,
            .userAccessible = options.userAccessible,
            .writeThrough = options.writeThrough,
            .cacheDisabled = options.cacheDisabled,
            .hugePage = false,
            .global = options.global,
            //.noExecute = options.noExecute,
            .address = @intCast(physicalAddress >> 12),
        };
        invalidatePage(makeCanonical((l4Index << 39) | (l3Index << 30) | (l2Index << 21) | (l1Index << 12)));

        return;
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
                    const l3Table: *align(0x1000) [512]PageTableEntry = @ptrCast(self.pageTable.frameVirtualAddress(l4Entry.address << 12));
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
                        const l2Table: *align(0x1000) [512]PageTableEntry = @ptrCast(self.pageTable.frameVirtualAddress(l3Entry.address << 12));
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
                            const l1Table: *align(0x1000) [512]PageTableEntry = @ptrCast(self.pageTable.frameVirtualAddress(l2Entry.address << 12));
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
    entries: [4]?*const PageTableEntry = .{null} ** 4,
};

fn invalidatePage(virtualAddress: u64) void {
    asm volatile (
        \\ invlpg %[address]
        :
        : [address] "m" (virtualAddress),
        : "memory"
    );
}
