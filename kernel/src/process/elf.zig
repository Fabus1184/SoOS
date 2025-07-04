const std = @import("std");

const paging = @import("../paging.zig");

const Header = packed struct {
    type: enum(u16) {
        ET_NONE = 0,
        ET_REL = 1,
        ET_EXEC = 2,
        ET_DYN = 3,
        ET_CORE = 4,
        _,
    },
    machine: enum(u16) {
        EM_X86_64 = 62,
        _,
    },
    version: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
};

const ProgramHeader = packed struct(u448) {
    type: enum(u32) {
        PT_NULL = 0,
        PT_LOAD = 1,
        PT_DYNAMIC = 2,
        PT_INTERP = 3,
        PT_NOTE = 4,
        PT_SHLIB = 5,
        PT_PHDR = 6,
        _,
    },
    flags: u32,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    @"align": u64,
};

pub const MappedPage = struct {
    virtualAddress: u64,
    size: paging.FrameSize,
};

pub const Elf = struct {
    bytes: []const u8,
    entry: u64,

    fn ident(self: Elf) []const u8 {
        return self.bytes[0..16];
    }

    fn header(self: Elf) Header {
        return std.mem.bytesToValue(Header, self.bytes[16 .. 16 + @sizeOf(Header)]);
    }

    pub fn load(
        bytes: []const u8,
        pagetable: *paging.OffsetPageTable,
        frameAllocator: *paging.FrameAllocator,
        mappedPages: *std.ArrayList(MappedPage),
    ) !Elf {
        pagetable.load();

        const self = Elf{ .bytes = bytes, .entry = 0 };

        // confirm ELF magic number
        if (!std.mem.startsWith(u8, self.ident(), "\x7fELF")) {
            return error.InvalidElf;
        }

        const h = self.header();
        if (h.type != .ET_EXEC) {
            return error.InvalidElf;
        }
        if (h.machine != .EM_X86_64) {
            return error.InvalidElfMachine;
        }
        if (h.version != 1) {
            return error.InvalidElfVersion;
        }

        std.log.debug("ELF header: type: {d}, machine: {d}, version: {d}, entry: 0x{x}, phoff: 0x{x}, shoff: 0x{x}, flags: {d}, ehsize: {d}, phentsize: {d}, phnum: {d}", .{
            h.type, h.machine, h.version, h.entry, h.phoff, h.shoff, h.flags, h.ehsize, h.phentsize, h.phnum,
        });

        for (0..h.phnum) |i| {
            const phdr = @as(*align(1) const ProgramHeader, @ptrCast(&self.bytes[h.phoff + i * h.phentsize]));

            switch (phdr.type) {
                .PT_LOAD => {
                    const startPage = phdr.vaddr & ~@as(u64, 0xFFF);
                    const endPage = (phdr.vaddr + phdr.memsz) & ~@as(u64, 0xFFF);

                    var page = startPage;
                    while (page <= endPage) : (page += 0x1000) {
                        // skip if the page is already mapped
                        if (pagetable.translate(page) != null) {
                            std.log.debug("skipping already mapped page 0x{x}", .{page});
                            continue;
                        }

                        const frame = try frameAllocator.allocateFrame(.@"4KiB");
                        try pagetable.map(
                            frameAllocator,
                            page,
                            frame,
                            .@"4KiB",
                            .{
                                .userAccessible = true,
                                //.writable = (phdr.flags & 0x2) != 0,
                                .writable = true,
                                .noExecute = (phdr.flags & 0x1) == 0,
                            },
                        );

                        // zero out the page
                        const memory: []u8 = @as([*]u8, @ptrFromInt(page))[0..0x1000];
                        @memset(memory, 0);

                        try mappedPages.append(.{ .virtualAddress = page, .size = .@"4KiB" });
                    }

                    // copy the segment data into the mapped memory
                    const memory: []u8 = @as([*]u8, @ptrFromInt(phdr.vaddr))[0..phdr.memsz];
                    const data: []const u8 = self.bytes[phdr.offset .. phdr.offset + phdr.filesz];
                    std.mem.copyForwards(u8, memory, data);
                },
                else => {
                    std.log.warn("ignoring segment {d}: type: {d}", .{ i, phdr.type });
                },
            }
        }

        return Elf{
            .bytes = bytes,
            .entry = h.entry,
        };
    }
};
