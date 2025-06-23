pub const raw = @cImport({
    @cInclude("limine/limine.h");
});

const LIMINE_COMMON_MAGIC: [2]u64 = .{ 0xc7b1dd30df4c8b88, 0x0a82e883a194f07b };

const LIMINE_REQUESTS_START_MARKER: [4]u64 linksection(".limine_requests_start") = .{ 0xf6b8f4b39de7d1ae, 0xfab91a6940fcb9cf, 0x785c6ed015d3e316, 0x181e920a7852b9d9 };
const LIMINE_REQUESTS_END_MARKER: [2]u64 linksection(".limine_requests_end") = .{ 0xadc0e0531bb10d03, 0x9572709f31764c62 };

pub var LIMINE_FRAMEBUFFER_REQUEST: raw.limine_framebuffer_request linksection(".limine_requests") = .{
    .id = LIMINE_COMMON_MAGIC ++ .{ 0x9d5827dcd881dd75, 0xa3148604f6fab11b },
    .revision = raw.LIMINE_API_REVISION,
};
pub var LIMINE_PAGING_MODE_REQUEST: raw.limine_paging_mode_request linksection(".limine_requests") = .{
    .id = LIMINE_COMMON_MAGIC ++ .{ 0x95c1a0edab0944cb, 0xa4e5cb3842f7488a },
    .revision = raw.LIMINE_API_REVISION,
    .mode = raw.LIMINE_PAGING_MODE_X86_64_4LVL,
};
pub var LIMINE_HHDM_REQUEST: raw.limine_hhdm_request linksection(".limine_requests") = .{
    .id = LIMINE_COMMON_MAGIC ++ .{ 0x48dcf1cb8ad2b852, 0x63984e959a98244b },
    .revision = raw.LIMINE_API_REVISION,
};
pub var LIMINE_MEMMAP_REQUEST: raw.limine_memmap_request linksection(".limine_requests") = .{
    .id = LIMINE_COMMON_MAGIC ++ .{ 0x67cf3d9d378a806f, 0xe304acdfc50c3c62 },
    .revision = raw.LIMINE_API_REVISION,
};
