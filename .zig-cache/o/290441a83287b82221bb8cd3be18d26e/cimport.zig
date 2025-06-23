pub const __builtin_bswap16 = @import("std").zig.c_builtins.__builtin_bswap16;
pub const __builtin_bswap32 = @import("std").zig.c_builtins.__builtin_bswap32;
pub const __builtin_bswap64 = @import("std").zig.c_builtins.__builtin_bswap64;
pub const __builtin_signbit = @import("std").zig.c_builtins.__builtin_signbit;
pub const __builtin_signbitf = @import("std").zig.c_builtins.__builtin_signbitf;
pub const __builtin_popcount = @import("std").zig.c_builtins.__builtin_popcount;
pub const __builtin_ctz = @import("std").zig.c_builtins.__builtin_ctz;
pub const __builtin_clz = @import("std").zig.c_builtins.__builtin_clz;
pub const __builtin_sqrt = @import("std").zig.c_builtins.__builtin_sqrt;
pub const __builtin_sqrtf = @import("std").zig.c_builtins.__builtin_sqrtf;
pub const __builtin_sin = @import("std").zig.c_builtins.__builtin_sin;
pub const __builtin_sinf = @import("std").zig.c_builtins.__builtin_sinf;
pub const __builtin_cos = @import("std").zig.c_builtins.__builtin_cos;
pub const __builtin_cosf = @import("std").zig.c_builtins.__builtin_cosf;
pub const __builtin_exp = @import("std").zig.c_builtins.__builtin_exp;
pub const __builtin_expf = @import("std").zig.c_builtins.__builtin_expf;
pub const __builtin_exp2 = @import("std").zig.c_builtins.__builtin_exp2;
pub const __builtin_exp2f = @import("std").zig.c_builtins.__builtin_exp2f;
pub const __builtin_log = @import("std").zig.c_builtins.__builtin_log;
pub const __builtin_logf = @import("std").zig.c_builtins.__builtin_logf;
pub const __builtin_log2 = @import("std").zig.c_builtins.__builtin_log2;
pub const __builtin_log2f = @import("std").zig.c_builtins.__builtin_log2f;
pub const __builtin_log10 = @import("std").zig.c_builtins.__builtin_log10;
pub const __builtin_log10f = @import("std").zig.c_builtins.__builtin_log10f;
pub const __builtin_abs = @import("std").zig.c_builtins.__builtin_abs;
pub const __builtin_labs = @import("std").zig.c_builtins.__builtin_labs;
pub const __builtin_llabs = @import("std").zig.c_builtins.__builtin_llabs;
pub const __builtin_fabs = @import("std").zig.c_builtins.__builtin_fabs;
pub const __builtin_fabsf = @import("std").zig.c_builtins.__builtin_fabsf;
pub const __builtin_floor = @import("std").zig.c_builtins.__builtin_floor;
pub const __builtin_floorf = @import("std").zig.c_builtins.__builtin_floorf;
pub const __builtin_ceil = @import("std").zig.c_builtins.__builtin_ceil;
pub const __builtin_ceilf = @import("std").zig.c_builtins.__builtin_ceilf;
pub const __builtin_trunc = @import("std").zig.c_builtins.__builtin_trunc;
pub const __builtin_truncf = @import("std").zig.c_builtins.__builtin_truncf;
pub const __builtin_round = @import("std").zig.c_builtins.__builtin_round;
pub const __builtin_roundf = @import("std").zig.c_builtins.__builtin_roundf;
pub const __builtin_strlen = @import("std").zig.c_builtins.__builtin_strlen;
pub const __builtin_strcmp = @import("std").zig.c_builtins.__builtin_strcmp;
pub const __builtin_object_size = @import("std").zig.c_builtins.__builtin_object_size;
pub const __builtin___memset_chk = @import("std").zig.c_builtins.__builtin___memset_chk;
pub const __builtin_memset = @import("std").zig.c_builtins.__builtin_memset;
pub const __builtin___memcpy_chk = @import("std").zig.c_builtins.__builtin___memcpy_chk;
pub const __builtin_memcpy = @import("std").zig.c_builtins.__builtin_memcpy;
pub const __builtin_expect = @import("std").zig.c_builtins.__builtin_expect;
pub const __builtin_nanf = @import("std").zig.c_builtins.__builtin_nanf;
pub const __builtin_huge_valf = @import("std").zig.c_builtins.__builtin_huge_valf;
pub const __builtin_inff = @import("std").zig.c_builtins.__builtin_inff;
pub const __builtin_isnan = @import("std").zig.c_builtins.__builtin_isnan;
pub const __builtin_isinf = @import("std").zig.c_builtins.__builtin_isinf;
pub const __builtin_isinf_sign = @import("std").zig.c_builtins.__builtin_isinf_sign;
pub const __has_builtin = @import("std").zig.c_builtins.__has_builtin;
pub const __builtin_assume = @import("std").zig.c_builtins.__builtin_assume;
pub const __builtin_unreachable = @import("std").zig.c_builtins.__builtin_unreachable;
pub const __builtin_constant_p = @import("std").zig.c_builtins.__builtin_constant_p;
pub const __builtin_mul_overflow = @import("std").zig.c_builtins.__builtin_mul_overflow;
pub const int_least64_t = i64;
pub const uint_least64_t = u64;
pub const int_fast64_t = i64;
pub const uint_fast64_t = u64;
pub const int_least32_t = i32;
pub const uint_least32_t = u32;
pub const int_fast32_t = i32;
pub const uint_fast32_t = u32;
pub const int_least16_t = i16;
pub const uint_least16_t = u16;
pub const int_fast16_t = i16;
pub const uint_fast16_t = u16;
pub const int_least8_t = i8;
pub const uint_least8_t = u8;
pub const int_fast8_t = i8;
pub const uint_fast8_t = u8;
pub const intmax_t = c_long;
pub const uintmax_t = c_ulong;
pub const struct_limine_uuid = extern struct {
    a: u32 = @import("std").mem.zeroes(u32),
    b: u16 = @import("std").mem.zeroes(u16),
    c: u16 = @import("std").mem.zeroes(u16),
    d: [8]u8 = @import("std").mem.zeroes([8]u8),
};
pub const struct_limine_file = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    address: ?*anyopaque = @import("std").mem.zeroes(?*anyopaque),
    size: u64 = @import("std").mem.zeroes(u64),
    path: [*c]u8 = @import("std").mem.zeroes([*c]u8),
    cmdline: [*c]u8 = @import("std").mem.zeroes([*c]u8),
    media_type: u32 = @import("std").mem.zeroes(u32),
    unused: u32 = @import("std").mem.zeroes(u32),
    tftp_ip: u32 = @import("std").mem.zeroes(u32),
    tftp_port: u32 = @import("std").mem.zeroes(u32),
    partition_index: u32 = @import("std").mem.zeroes(u32),
    mbr_disk_id: u32 = @import("std").mem.zeroes(u32),
    gpt_disk_uuid: struct_limine_uuid = @import("std").mem.zeroes(struct_limine_uuid),
    gpt_part_uuid: struct_limine_uuid = @import("std").mem.zeroes(struct_limine_uuid),
    part_uuid: struct_limine_uuid = @import("std").mem.zeroes(struct_limine_uuid),
};
pub const struct_limine_bootloader_info_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    name: [*c]u8 = @import("std").mem.zeroes([*c]u8),
    version: [*c]u8 = @import("std").mem.zeroes([*c]u8),
};
pub const struct_limine_bootloader_info_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_bootloader_info_response = @import("std").mem.zeroes([*c]struct_limine_bootloader_info_response),
};
pub const struct_limine_executable_cmdline_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    cmdline: [*c]u8 = @import("std").mem.zeroes([*c]u8),
};
pub const struct_limine_executable_cmdline_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_executable_cmdline_response = @import("std").mem.zeroes([*c]struct_limine_executable_cmdline_response),
};
pub const struct_limine_firmware_type_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    firmware_type: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_firmware_type_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_firmware_type_response = @import("std").mem.zeroes([*c]struct_limine_firmware_type_response),
};
pub const struct_limine_stack_size_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_stack_size_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_stack_size_response = @import("std").mem.zeroes([*c]struct_limine_stack_size_response),
    stack_size: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_hhdm_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    offset: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_hhdm_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_hhdm_response = @import("std").mem.zeroes([*c]struct_limine_hhdm_response),
};
pub const struct_limine_video_mode = extern struct {
    pitch: u64 = @import("std").mem.zeroes(u64),
    width: u64 = @import("std").mem.zeroes(u64),
    height: u64 = @import("std").mem.zeroes(u64),
    bpp: u16 = @import("std").mem.zeroes(u16),
    memory_model: u8 = @import("std").mem.zeroes(u8),
    red_mask_size: u8 = @import("std").mem.zeroes(u8),
    red_mask_shift: u8 = @import("std").mem.zeroes(u8),
    green_mask_size: u8 = @import("std").mem.zeroes(u8),
    green_mask_shift: u8 = @import("std").mem.zeroes(u8),
    blue_mask_size: u8 = @import("std").mem.zeroes(u8),
    blue_mask_shift: u8 = @import("std").mem.zeroes(u8),
};
pub const struct_limine_framebuffer = extern struct {
    address: ?*anyopaque = @import("std").mem.zeroes(?*anyopaque),
    width: u64 = @import("std").mem.zeroes(u64),
    height: u64 = @import("std").mem.zeroes(u64),
    pitch: u64 = @import("std").mem.zeroes(u64),
    bpp: u16 = @import("std").mem.zeroes(u16),
    memory_model: u8 = @import("std").mem.zeroes(u8),
    red_mask_size: u8 = @import("std").mem.zeroes(u8),
    red_mask_shift: u8 = @import("std").mem.zeroes(u8),
    green_mask_size: u8 = @import("std").mem.zeroes(u8),
    green_mask_shift: u8 = @import("std").mem.zeroes(u8),
    blue_mask_size: u8 = @import("std").mem.zeroes(u8),
    blue_mask_shift: u8 = @import("std").mem.zeroes(u8),
    unused: [7]u8 = @import("std").mem.zeroes([7]u8),
    edid_size: u64 = @import("std").mem.zeroes(u64),
    edid: ?*anyopaque = @import("std").mem.zeroes(?*anyopaque),
    mode_count: u64 = @import("std").mem.zeroes(u64),
    modes: [*c][*c]struct_limine_video_mode = @import("std").mem.zeroes([*c][*c]struct_limine_video_mode),
};
pub const struct_limine_framebuffer_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    framebuffer_count: u64 = @import("std").mem.zeroes(u64),
    framebuffers: [*c][*c]struct_limine_framebuffer = @import("std").mem.zeroes([*c][*c]struct_limine_framebuffer),
};
pub const struct_limine_framebuffer_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_framebuffer_response = @import("std").mem.zeroes([*c]struct_limine_framebuffer_response),
};
pub const struct_limine_terminal = extern struct {
    columns: u64 = @import("std").mem.zeroes(u64),
    rows: u64 = @import("std").mem.zeroes(u64),
    framebuffer: [*c]struct_limine_framebuffer = @import("std").mem.zeroes([*c]struct_limine_framebuffer),
};
pub const limine_terminal_write = ?*const fn ([*c]struct_limine_terminal, [*c]const u8, u64) callconv(.c) void;
pub const limine_terminal_callback = ?*const fn ([*c]struct_limine_terminal, u64, u64, u64, u64) callconv(.c) void;
pub const struct_limine_terminal_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    terminal_count: u64 = @import("std").mem.zeroes(u64),
    terminals: [*c][*c]struct_limine_terminal = @import("std").mem.zeroes([*c][*c]struct_limine_terminal),
    write: limine_terminal_write = @import("std").mem.zeroes(limine_terminal_write),
};
pub const struct_limine_terminal_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_terminal_response = @import("std").mem.zeroes([*c]struct_limine_terminal_response),
    callback: limine_terminal_callback = @import("std").mem.zeroes(limine_terminal_callback),
};
pub const struct_limine_paging_mode_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    mode: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_paging_mode_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_paging_mode_response = @import("std").mem.zeroes([*c]struct_limine_paging_mode_response),
    mode: u64 = @import("std").mem.zeroes(u64),
    max_mode: u64 = @import("std").mem.zeroes(u64),
    min_mode: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_5_level_paging_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_5_level_paging_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_5_level_paging_response = @import("std").mem.zeroes([*c]struct_limine_5_level_paging_response),
};
pub const limine_goto_address = ?*const fn ([*c]struct_limine_smp_info) callconv(.c) void;
pub const struct_limine_smp_info = extern struct {
    processor_id: u32 = @import("std").mem.zeroes(u32),
    lapic_id: u32 = @import("std").mem.zeroes(u32),
    reserved: u64 = @import("std").mem.zeroes(u64),
    goto_address: limine_goto_address = @import("std").mem.zeroes(limine_goto_address),
    extra_argument: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_smp_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    flags: u32 = @import("std").mem.zeroes(u32),
    bsp_lapic_id: u32 = @import("std").mem.zeroes(u32),
    cpu_count: u64 = @import("std").mem.zeroes(u64),
    cpus: [*c][*c]struct_limine_smp_info = @import("std").mem.zeroes([*c][*c]struct_limine_smp_info),
};
pub const struct_limine_smp_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_smp_response = @import("std").mem.zeroes([*c]struct_limine_smp_response),
    flags: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_memmap_entry = extern struct {
    base: u64 = @import("std").mem.zeroes(u64),
    length: u64 = @import("std").mem.zeroes(u64),
    type: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_memmap_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    entry_count: u64 = @import("std").mem.zeroes(u64),
    entries: [*c][*c]struct_limine_memmap_entry = @import("std").mem.zeroes([*c][*c]struct_limine_memmap_entry),
};
pub const struct_limine_memmap_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_memmap_response = @import("std").mem.zeroes([*c]struct_limine_memmap_response),
};
pub const limine_entry_point = ?*const fn () callconv(.c) void;
pub const struct_limine_entry_point_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_entry_point_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_entry_point_response = @import("std").mem.zeroes([*c]struct_limine_entry_point_response),
    entry: limine_entry_point = @import("std").mem.zeroes(limine_entry_point),
};
pub const struct_limine_kernel_file_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    kernel_file: [*c]struct_limine_file = @import("std").mem.zeroes([*c]struct_limine_file),
};
pub const struct_limine_kernel_file_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_kernel_file_response = @import("std").mem.zeroes([*c]struct_limine_kernel_file_response),
};
pub const struct_limine_internal_module = extern struct {
    path: [*c]const u8 = @import("std").mem.zeroes([*c]const u8),
    cmdline: [*c]const u8 = @import("std").mem.zeroes([*c]const u8),
    flags: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_module_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    module_count: u64 = @import("std").mem.zeroes(u64),
    modules: [*c][*c]struct_limine_file = @import("std").mem.zeroes([*c][*c]struct_limine_file),
};
pub const struct_limine_module_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_module_response = @import("std").mem.zeroes([*c]struct_limine_module_response),
    internal_module_count: u64 = @import("std").mem.zeroes(u64),
    internal_modules: [*c][*c]struct_limine_internal_module = @import("std").mem.zeroes([*c][*c]struct_limine_internal_module),
};
pub const struct_limine_rsdp_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    address: ?*anyopaque = @import("std").mem.zeroes(?*anyopaque),
};
pub const struct_limine_rsdp_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_rsdp_response = @import("std").mem.zeroes([*c]struct_limine_rsdp_response),
};
pub const struct_limine_smbios_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    entry_32: ?*anyopaque = @import("std").mem.zeroes(?*anyopaque),
    entry_64: ?*anyopaque = @import("std").mem.zeroes(?*anyopaque),
};
pub const struct_limine_smbios_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_smbios_response = @import("std").mem.zeroes([*c]struct_limine_smbios_response),
};
pub const struct_limine_efi_system_table_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    address: ?*anyopaque = @import("std").mem.zeroes(?*anyopaque),
};
pub const struct_limine_efi_system_table_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_efi_system_table_response = @import("std").mem.zeroes([*c]struct_limine_efi_system_table_response),
};
pub const struct_limine_efi_memmap_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    memmap: ?*anyopaque = @import("std").mem.zeroes(?*anyopaque),
    memmap_size: u64 = @import("std").mem.zeroes(u64),
    desc_size: u64 = @import("std").mem.zeroes(u64),
    desc_version: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_efi_memmap_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_efi_memmap_response = @import("std").mem.zeroes([*c]struct_limine_efi_memmap_response),
};
pub const struct_limine_boot_time_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    boot_time: i64 = @import("std").mem.zeroes(i64),
};
pub const struct_limine_boot_time_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_boot_time_response = @import("std").mem.zeroes([*c]struct_limine_boot_time_response),
};
pub const struct_limine_kernel_address_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    physical_base: u64 = @import("std").mem.zeroes(u64),
    virtual_base: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_kernel_address_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_kernel_address_response = @import("std").mem.zeroes([*c]struct_limine_kernel_address_response),
};
pub const struct_limine_dtb_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    dtb_ptr: ?*anyopaque = @import("std").mem.zeroes(?*anyopaque),
};
pub const struct_limine_dtb_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_dtb_response = @import("std").mem.zeroes([*c]struct_limine_dtb_response),
};
pub const struct_limine_riscv_bsp_hartid_response = extern struct {
    revision: u64 = @import("std").mem.zeroes(u64),
    bsp_hartid: u64 = @import("std").mem.zeroes(u64),
};
pub const struct_limine_riscv_bsp_hartid_request = extern struct {
    id: [4]u64 = @import("std").mem.zeroes([4]u64),
    revision: u64 = @import("std").mem.zeroes(u64),
    response: [*c]struct_limine_riscv_bsp_hartid_response = @import("std").mem.zeroes([*c]struct_limine_riscv_bsp_hartid_response),
};
pub const __llvm__ = @as(c_int, 1);
pub const __clang__ = @as(c_int, 1);
pub const __clang_major__ = @as(c_int, 19);
pub const __clang_minor__ = @as(c_int, 1);
pub const __clang_patchlevel__ = @as(c_int, 7);
pub const __clang_version__ = "19.1.7 (https://github.com/ziglang/zig-bootstrap 1c3c59435891bc9caf8cd1d3783773369d191c5f)";
pub const __GNUC__ = @as(c_int, 4);
pub const __GNUC_MINOR__ = @as(c_int, 2);
pub const __GNUC_PATCHLEVEL__ = @as(c_int, 1);
pub const __GXX_ABI_VERSION = @as(c_int, 1002);
pub const __ATOMIC_RELAXED = @as(c_int, 0);
pub const __ATOMIC_CONSUME = @as(c_int, 1);
pub const __ATOMIC_ACQUIRE = @as(c_int, 2);
pub const __ATOMIC_RELEASE = @as(c_int, 3);
pub const __ATOMIC_ACQ_REL = @as(c_int, 4);
pub const __ATOMIC_SEQ_CST = @as(c_int, 5);
pub const __MEMORY_SCOPE_SYSTEM = @as(c_int, 0);
pub const __MEMORY_SCOPE_DEVICE = @as(c_int, 1);
pub const __MEMORY_SCOPE_WRKGRP = @as(c_int, 2);
pub const __MEMORY_SCOPE_WVFRNT = @as(c_int, 3);
pub const __MEMORY_SCOPE_SINGLE = @as(c_int, 4);
pub const __OPENCL_MEMORY_SCOPE_WORK_ITEM = @as(c_int, 0);
pub const __OPENCL_MEMORY_SCOPE_WORK_GROUP = @as(c_int, 1);
pub const __OPENCL_MEMORY_SCOPE_DEVICE = @as(c_int, 2);
pub const __OPENCL_MEMORY_SCOPE_ALL_SVM_DEVICES = @as(c_int, 3);
pub const __OPENCL_MEMORY_SCOPE_SUB_GROUP = @as(c_int, 4);
pub const __FPCLASS_SNAN = @as(c_int, 0x0001);
pub const __FPCLASS_QNAN = @as(c_int, 0x0002);
pub const __FPCLASS_NEGINF = @as(c_int, 0x0004);
pub const __FPCLASS_NEGNORMAL = @as(c_int, 0x0008);
pub const __FPCLASS_NEGSUBNORMAL = @as(c_int, 0x0010);
pub const __FPCLASS_NEGZERO = @as(c_int, 0x0020);
pub const __FPCLASS_POSZERO = @as(c_int, 0x0040);
pub const __FPCLASS_POSSUBNORMAL = @as(c_int, 0x0080);
pub const __FPCLASS_POSNORMAL = @as(c_int, 0x0100);
pub const __FPCLASS_POSINF = @as(c_int, 0x0200);
pub const __PRAGMA_REDEFINE_EXTNAME = @as(c_int, 1);
pub const __VERSION__ = "Clang 19.1.7 (https://github.com/ziglang/zig-bootstrap 1c3c59435891bc9caf8cd1d3783773369d191c5f)";
pub const __OBJC_BOOL_IS_BOOL = @as(c_int, 0);
pub const __CONSTANT_CFSTRINGS__ = @as(c_int, 1);
pub const __clang_literal_encoding__ = "UTF-8";
pub const __clang_wide_literal_encoding__ = "UTF-32";
pub const __ORDER_LITTLE_ENDIAN__ = @as(c_int, 1234);
pub const __ORDER_BIG_ENDIAN__ = @as(c_int, 4321);
pub const __ORDER_PDP_ENDIAN__ = @as(c_int, 3412);
pub const __BYTE_ORDER__ = __ORDER_LITTLE_ENDIAN__;
pub const __LITTLE_ENDIAN__ = @as(c_int, 1);
pub const _LP64 = @as(c_int, 1);
pub const __LP64__ = @as(c_int, 1);
pub const __CHAR_BIT__ = @as(c_int, 8);
pub const __BOOL_WIDTH__ = @as(c_int, 8);
pub const __SHRT_WIDTH__ = @as(c_int, 16);
pub const __INT_WIDTH__ = @as(c_int, 32);
pub const __LONG_WIDTH__ = @as(c_int, 64);
pub const __LLONG_WIDTH__ = @as(c_int, 64);
pub const __BITINT_MAXWIDTH__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 8388608, .decimal);
pub const __SCHAR_MAX__ = @as(c_int, 127);
pub const __SHRT_MAX__ = @as(c_int, 32767);
pub const __INT_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 2147483647, .decimal);
pub const __LONG_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_long, 9223372036854775807, .decimal);
pub const __LONG_LONG_MAX__ = @as(c_longlong, 9223372036854775807);
pub const __WCHAR_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 2147483647, .decimal);
pub const __WCHAR_WIDTH__ = @as(c_int, 32);
pub const __WINT_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 2147483647, .decimal);
pub const __WINT_WIDTH__ = @as(c_int, 32);
pub const __INTMAX_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_long, 9223372036854775807, .decimal);
pub const __INTMAX_WIDTH__ = @as(c_int, 64);
pub const __SIZE_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_ulong, 18446744073709551615, .decimal);
pub const __SIZE_WIDTH__ = @as(c_int, 64);
pub const __UINTMAX_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_ulong, 18446744073709551615, .decimal);
pub const __UINTMAX_WIDTH__ = @as(c_int, 64);
pub const __PTRDIFF_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_long, 9223372036854775807, .decimal);
pub const __PTRDIFF_WIDTH__ = @as(c_int, 64);
pub const __INTPTR_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_long, 9223372036854775807, .decimal);
pub const __INTPTR_WIDTH__ = @as(c_int, 64);
pub const __UINTPTR_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_ulong, 18446744073709551615, .decimal);
pub const __UINTPTR_WIDTH__ = @as(c_int, 64);
pub const __SIZEOF_DOUBLE__ = @as(c_int, 8);
pub const __SIZEOF_FLOAT__ = @as(c_int, 4);
pub const __SIZEOF_INT__ = @as(c_int, 4);
pub const __SIZEOF_LONG__ = @as(c_int, 8);
pub const __SIZEOF_LONG_DOUBLE__ = @as(c_int, 16);
pub const __SIZEOF_LONG_LONG__ = @as(c_int, 8);
pub const __SIZEOF_POINTER__ = @as(c_int, 8);
pub const __SIZEOF_SHORT__ = @as(c_int, 2);
pub const __SIZEOF_PTRDIFF_T__ = @as(c_int, 8);
pub const __SIZEOF_SIZE_T__ = @as(c_int, 8);
pub const __SIZEOF_WCHAR_T__ = @as(c_int, 4);
pub const __SIZEOF_WINT_T__ = @as(c_int, 4);
pub const __SIZEOF_INT128__ = @as(c_int, 16);
pub const __INTMAX_TYPE__ = c_long;
pub const __INTMAX_FMTd__ = "ld";
pub const __INTMAX_FMTi__ = "li";
pub const __INTMAX_C_SUFFIX__ = @compileError("unable to translate macro: undefined identifier `L`");
// (no file):95:9
pub const __UINTMAX_TYPE__ = c_ulong;
pub const __UINTMAX_FMTo__ = "lo";
pub const __UINTMAX_FMTu__ = "lu";
pub const __UINTMAX_FMTx__ = "lx";
pub const __UINTMAX_FMTX__ = "lX";
pub const __UINTMAX_C_SUFFIX__ = @compileError("unable to translate macro: undefined identifier `UL`");
// (no file):101:9
pub const __PTRDIFF_TYPE__ = c_long;
pub const __PTRDIFF_FMTd__ = "ld";
pub const __PTRDIFF_FMTi__ = "li";
pub const __INTPTR_TYPE__ = c_long;
pub const __INTPTR_FMTd__ = "ld";
pub const __INTPTR_FMTi__ = "li";
pub const __SIZE_TYPE__ = c_ulong;
pub const __SIZE_FMTo__ = "lo";
pub const __SIZE_FMTu__ = "lu";
pub const __SIZE_FMTx__ = "lx";
pub const __SIZE_FMTX__ = "lX";
pub const __WCHAR_TYPE__ = c_int;
pub const __WINT_TYPE__ = c_int;
pub const __SIG_ATOMIC_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 2147483647, .decimal);
pub const __SIG_ATOMIC_WIDTH__ = @as(c_int, 32);
pub const __CHAR16_TYPE__ = c_ushort;
pub const __CHAR32_TYPE__ = c_uint;
pub const __UINTPTR_TYPE__ = c_ulong;
pub const __UINTPTR_FMTo__ = "lo";
pub const __UINTPTR_FMTu__ = "lu";
pub const __UINTPTR_FMTx__ = "lx";
pub const __UINTPTR_FMTX__ = "lX";
pub const __FLT16_DENORM_MIN__ = @as(f16, 5.9604644775390625e-8);
pub const __FLT16_NORM_MAX__ = @as(f16, 6.5504e+4);
pub const __FLT16_HAS_DENORM__ = @as(c_int, 1);
pub const __FLT16_DIG__ = @as(c_int, 3);
pub const __FLT16_DECIMAL_DIG__ = @as(c_int, 5);
pub const __FLT16_EPSILON__ = @as(f16, 9.765625e-4);
pub const __FLT16_HAS_INFINITY__ = @as(c_int, 1);
pub const __FLT16_HAS_QUIET_NAN__ = @as(c_int, 1);
pub const __FLT16_MANT_DIG__ = @as(c_int, 11);
pub const __FLT16_MAX_10_EXP__ = @as(c_int, 4);
pub const __FLT16_MAX_EXP__ = @as(c_int, 16);
pub const __FLT16_MAX__ = @as(f16, 6.5504e+4);
pub const __FLT16_MIN_10_EXP__ = -@as(c_int, 4);
pub const __FLT16_MIN_EXP__ = -@as(c_int, 13);
pub const __FLT16_MIN__ = @as(f16, 6.103515625e-5);
pub const __FLT_DENORM_MIN__ = @as(f32, 1.40129846e-45);
pub const __FLT_NORM_MAX__ = @as(f32, 3.40282347e+38);
pub const __FLT_HAS_DENORM__ = @as(c_int, 1);
pub const __FLT_DIG__ = @as(c_int, 6);
pub const __FLT_DECIMAL_DIG__ = @as(c_int, 9);
pub const __FLT_EPSILON__ = @as(f32, 1.19209290e-7);
pub const __FLT_HAS_INFINITY__ = @as(c_int, 1);
pub const __FLT_HAS_QUIET_NAN__ = @as(c_int, 1);
pub const __FLT_MANT_DIG__ = @as(c_int, 24);
pub const __FLT_MAX_10_EXP__ = @as(c_int, 38);
pub const __FLT_MAX_EXP__ = @as(c_int, 128);
pub const __FLT_MAX__ = @as(f32, 3.40282347e+38);
pub const __FLT_MIN_10_EXP__ = -@as(c_int, 37);
pub const __FLT_MIN_EXP__ = -@as(c_int, 125);
pub const __FLT_MIN__ = @as(f32, 1.17549435e-38);
pub const __DBL_DENORM_MIN__ = @as(f64, 4.9406564584124654e-324);
pub const __DBL_NORM_MAX__ = @as(f64, 1.7976931348623157e+308);
pub const __DBL_HAS_DENORM__ = @as(c_int, 1);
pub const __DBL_DIG__ = @as(c_int, 15);
pub const __DBL_DECIMAL_DIG__ = @as(c_int, 17);
pub const __DBL_EPSILON__ = @as(f64, 2.2204460492503131e-16);
pub const __DBL_HAS_INFINITY__ = @as(c_int, 1);
pub const __DBL_HAS_QUIET_NAN__ = @as(c_int, 1);
pub const __DBL_MANT_DIG__ = @as(c_int, 53);
pub const __DBL_MAX_10_EXP__ = @as(c_int, 308);
pub const __DBL_MAX_EXP__ = @as(c_int, 1024);
pub const __DBL_MAX__ = @as(f64, 1.7976931348623157e+308);
pub const __DBL_MIN_10_EXP__ = -@as(c_int, 307);
pub const __DBL_MIN_EXP__ = -@as(c_int, 1021);
pub const __DBL_MIN__ = @as(f64, 2.2250738585072014e-308);
pub const __LDBL_DENORM_MIN__ = @as(c_longdouble, 3.64519953188247460253e-4951);
pub const __LDBL_NORM_MAX__ = @as(c_longdouble, 1.18973149535723176502e+4932);
pub const __LDBL_HAS_DENORM__ = @as(c_int, 1);
pub const __LDBL_DIG__ = @as(c_int, 18);
pub const __LDBL_DECIMAL_DIG__ = @as(c_int, 21);
pub const __LDBL_EPSILON__ = @as(c_longdouble, 1.08420217248550443401e-19);
pub const __LDBL_HAS_INFINITY__ = @as(c_int, 1);
pub const __LDBL_HAS_QUIET_NAN__ = @as(c_int, 1);
pub const __LDBL_MANT_DIG__ = @as(c_int, 64);
pub const __LDBL_MAX_10_EXP__ = @as(c_int, 4932);
pub const __LDBL_MAX_EXP__ = @as(c_int, 16384);
pub const __LDBL_MAX__ = @as(c_longdouble, 1.18973149535723176502e+4932);
pub const __LDBL_MIN_10_EXP__ = -@as(c_int, 4931);
pub const __LDBL_MIN_EXP__ = -@as(c_int, 16381);
pub const __LDBL_MIN__ = @as(c_longdouble, 3.36210314311209350626e-4932);
pub const __POINTER_WIDTH__ = @as(c_int, 64);
pub const __BIGGEST_ALIGNMENT__ = @as(c_int, 16);
pub const __INT8_TYPE__ = i8;
pub const __INT8_FMTd__ = "hhd";
pub const __INT8_FMTi__ = "hhi";
pub const __INT8_C_SUFFIX__ = "";
pub const __INT16_TYPE__ = c_short;
pub const __INT16_FMTd__ = "hd";
pub const __INT16_FMTi__ = "hi";
pub const __INT16_C_SUFFIX__ = "";
pub const __INT32_TYPE__ = c_int;
pub const __INT32_FMTd__ = "d";
pub const __INT32_FMTi__ = "i";
pub const __INT32_C_SUFFIX__ = "";
pub const __INT64_TYPE__ = c_long;
pub const __INT64_FMTd__ = "ld";
pub const __INT64_FMTi__ = "li";
pub const __INT64_C_SUFFIX__ = @compileError("unable to translate macro: undefined identifier `L`");
// (no file):201:9
pub const __UINT8_TYPE__ = u8;
pub const __UINT8_FMTo__ = "hho";
pub const __UINT8_FMTu__ = "hhu";
pub const __UINT8_FMTx__ = "hhx";
pub const __UINT8_FMTX__ = "hhX";
pub const __UINT8_C_SUFFIX__ = "";
pub const __UINT8_MAX__ = @as(c_int, 255);
pub const __INT8_MAX__ = @as(c_int, 127);
pub const __UINT16_TYPE__ = c_ushort;
pub const __UINT16_FMTo__ = "ho";
pub const __UINT16_FMTu__ = "hu";
pub const __UINT16_FMTx__ = "hx";
pub const __UINT16_FMTX__ = "hX";
pub const __UINT16_C_SUFFIX__ = "";
pub const __UINT16_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 65535, .decimal);
pub const __INT16_MAX__ = @as(c_int, 32767);
pub const __UINT32_TYPE__ = c_uint;
pub const __UINT32_FMTo__ = "o";
pub const __UINT32_FMTu__ = "u";
pub const __UINT32_FMTx__ = "x";
pub const __UINT32_FMTX__ = "X";
pub const __UINT32_C_SUFFIX__ = @compileError("unable to translate macro: undefined identifier `U`");
// (no file):223:9
pub const __UINT32_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_uint, 4294967295, .decimal);
pub const __INT32_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 2147483647, .decimal);
pub const __UINT64_TYPE__ = c_ulong;
pub const __UINT64_FMTo__ = "lo";
pub const __UINT64_FMTu__ = "lu";
pub const __UINT64_FMTx__ = "lx";
pub const __UINT64_FMTX__ = "lX";
pub const __UINT64_C_SUFFIX__ = @compileError("unable to translate macro: undefined identifier `UL`");
// (no file):231:9
pub const __UINT64_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_ulong, 18446744073709551615, .decimal);
pub const __INT64_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_long, 9223372036854775807, .decimal);
pub const __INT_LEAST8_TYPE__ = i8;
pub const __INT_LEAST8_MAX__ = @as(c_int, 127);
pub const __INT_LEAST8_WIDTH__ = @as(c_int, 8);
pub const __INT_LEAST8_FMTd__ = "hhd";
pub const __INT_LEAST8_FMTi__ = "hhi";
pub const __UINT_LEAST8_TYPE__ = u8;
pub const __UINT_LEAST8_MAX__ = @as(c_int, 255);
pub const __UINT_LEAST8_FMTo__ = "hho";
pub const __UINT_LEAST8_FMTu__ = "hhu";
pub const __UINT_LEAST8_FMTx__ = "hhx";
pub const __UINT_LEAST8_FMTX__ = "hhX";
pub const __INT_LEAST16_TYPE__ = c_short;
pub const __INT_LEAST16_MAX__ = @as(c_int, 32767);
pub const __INT_LEAST16_WIDTH__ = @as(c_int, 16);
pub const __INT_LEAST16_FMTd__ = "hd";
pub const __INT_LEAST16_FMTi__ = "hi";
pub const __UINT_LEAST16_TYPE__ = c_ushort;
pub const __UINT_LEAST16_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 65535, .decimal);
pub const __UINT_LEAST16_FMTo__ = "ho";
pub const __UINT_LEAST16_FMTu__ = "hu";
pub const __UINT_LEAST16_FMTx__ = "hx";
pub const __UINT_LEAST16_FMTX__ = "hX";
pub const __INT_LEAST32_TYPE__ = c_int;
pub const __INT_LEAST32_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 2147483647, .decimal);
pub const __INT_LEAST32_WIDTH__ = @as(c_int, 32);
pub const __INT_LEAST32_FMTd__ = "d";
pub const __INT_LEAST32_FMTi__ = "i";
pub const __UINT_LEAST32_TYPE__ = c_uint;
pub const __UINT_LEAST32_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_uint, 4294967295, .decimal);
pub const __UINT_LEAST32_FMTo__ = "o";
pub const __UINT_LEAST32_FMTu__ = "u";
pub const __UINT_LEAST32_FMTx__ = "x";
pub const __UINT_LEAST32_FMTX__ = "X";
pub const __INT_LEAST64_TYPE__ = c_long;
pub const __INT_LEAST64_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_long, 9223372036854775807, .decimal);
pub const __INT_LEAST64_WIDTH__ = @as(c_int, 64);
pub const __INT_LEAST64_FMTd__ = "ld";
pub const __INT_LEAST64_FMTi__ = "li";
pub const __UINT_LEAST64_TYPE__ = c_ulong;
pub const __UINT_LEAST64_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_ulong, 18446744073709551615, .decimal);
pub const __UINT_LEAST64_FMTo__ = "lo";
pub const __UINT_LEAST64_FMTu__ = "lu";
pub const __UINT_LEAST64_FMTx__ = "lx";
pub const __UINT_LEAST64_FMTX__ = "lX";
pub const __INT_FAST8_TYPE__ = i8;
pub const __INT_FAST8_MAX__ = @as(c_int, 127);
pub const __INT_FAST8_WIDTH__ = @as(c_int, 8);
pub const __INT_FAST8_FMTd__ = "hhd";
pub const __INT_FAST8_FMTi__ = "hhi";
pub const __UINT_FAST8_TYPE__ = u8;
pub const __UINT_FAST8_MAX__ = @as(c_int, 255);
pub const __UINT_FAST8_FMTo__ = "hho";
pub const __UINT_FAST8_FMTu__ = "hhu";
pub const __UINT_FAST8_FMTx__ = "hhx";
pub const __UINT_FAST8_FMTX__ = "hhX";
pub const __INT_FAST16_TYPE__ = c_short;
pub const __INT_FAST16_MAX__ = @as(c_int, 32767);
pub const __INT_FAST16_WIDTH__ = @as(c_int, 16);
pub const __INT_FAST16_FMTd__ = "hd";
pub const __INT_FAST16_FMTi__ = "hi";
pub const __UINT_FAST16_TYPE__ = c_ushort;
pub const __UINT_FAST16_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 65535, .decimal);
pub const __UINT_FAST16_FMTo__ = "ho";
pub const __UINT_FAST16_FMTu__ = "hu";
pub const __UINT_FAST16_FMTx__ = "hx";
pub const __UINT_FAST16_FMTX__ = "hX";
pub const __INT_FAST32_TYPE__ = c_int;
pub const __INT_FAST32_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 2147483647, .decimal);
pub const __INT_FAST32_WIDTH__ = @as(c_int, 32);
pub const __INT_FAST32_FMTd__ = "d";
pub const __INT_FAST32_FMTi__ = "i";
pub const __UINT_FAST32_TYPE__ = c_uint;
pub const __UINT_FAST32_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_uint, 4294967295, .decimal);
pub const __UINT_FAST32_FMTo__ = "o";
pub const __UINT_FAST32_FMTu__ = "u";
pub const __UINT_FAST32_FMTx__ = "x";
pub const __UINT_FAST32_FMTX__ = "X";
pub const __INT_FAST64_TYPE__ = c_long;
pub const __INT_FAST64_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_long, 9223372036854775807, .decimal);
pub const __INT_FAST64_WIDTH__ = @as(c_int, 64);
pub const __INT_FAST64_FMTd__ = "ld";
pub const __INT_FAST64_FMTi__ = "li";
pub const __UINT_FAST64_TYPE__ = c_ulong;
pub const __UINT_FAST64_MAX__ = @import("std").zig.c_translation.promoteIntLiteral(c_ulong, 18446744073709551615, .decimal);
pub const __UINT_FAST64_FMTo__ = "lo";
pub const __UINT_FAST64_FMTu__ = "lu";
pub const __UINT_FAST64_FMTx__ = "lx";
pub const __UINT_FAST64_FMTX__ = "lX";
pub const __USER_LABEL_PREFIX__ = "";
pub const __FINITE_MATH_ONLY__ = @as(c_int, 0);
pub const __GNUC_STDC_INLINE__ = @as(c_int, 1);
pub const __GCC_ATOMIC_TEST_AND_SET_TRUEVAL = @as(c_int, 1);
pub const __GCC_DESTRUCTIVE_SIZE = @as(c_int, 64);
pub const __GCC_CONSTRUCTIVE_SIZE = @as(c_int, 64);
pub const __CLANG_ATOMIC_BOOL_LOCK_FREE = @as(c_int, 2);
pub const __CLANG_ATOMIC_CHAR_LOCK_FREE = @as(c_int, 2);
pub const __CLANG_ATOMIC_CHAR16_T_LOCK_FREE = @as(c_int, 2);
pub const __CLANG_ATOMIC_CHAR32_T_LOCK_FREE = @as(c_int, 2);
pub const __CLANG_ATOMIC_WCHAR_T_LOCK_FREE = @as(c_int, 2);
pub const __CLANG_ATOMIC_SHORT_LOCK_FREE = @as(c_int, 2);
pub const __CLANG_ATOMIC_INT_LOCK_FREE = @as(c_int, 2);
pub const __CLANG_ATOMIC_LONG_LOCK_FREE = @as(c_int, 2);
pub const __CLANG_ATOMIC_LLONG_LOCK_FREE = @as(c_int, 2);
pub const __CLANG_ATOMIC_POINTER_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_BOOL_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_CHAR_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_CHAR16_T_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_CHAR32_T_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_WCHAR_T_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_SHORT_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_INT_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_LONG_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_LLONG_LOCK_FREE = @as(c_int, 2);
pub const __GCC_ATOMIC_POINTER_LOCK_FREE = @as(c_int, 2);
pub const __NO_INLINE__ = @as(c_int, 1);
pub const __FLT_RADIX__ = @as(c_int, 2);
pub const __DECIMAL_DIG__ = __LDBL_DECIMAL_DIG__;
pub const __ELF__ = @as(c_int, 1);
pub const __GCC_ASM_FLAG_OUTPUTS__ = @as(c_int, 1);
pub const __code_model_kernel__ = @as(c_int, 1);
pub const __amd64__ = @as(c_int, 1);
pub const __amd64 = @as(c_int, 1);
pub const __x86_64 = @as(c_int, 1);
pub const __x86_64__ = @as(c_int, 1);
pub const __SEG_GS = @as(c_int, 1);
pub const __SEG_FS = @as(c_int, 1);
pub const __seg_gs = @compileError("unable to translate macro: undefined identifier `address_space`");
// (no file):362:9
pub const __seg_fs = @compileError("unable to translate macro: undefined identifier `address_space`");
// (no file):363:9
pub const __znver3 = @as(c_int, 1);
pub const __znver3__ = @as(c_int, 1);
pub const __tune_znver3__ = @as(c_int, 1);
pub const __REGISTER_PREFIX__ = "";
pub const __NO_MATH_INLINES = @as(c_int, 1);
pub const __AES__ = @as(c_int, 1);
pub const __VAES__ = @as(c_int, 1);
pub const __PCLMUL__ = @as(c_int, 1);
pub const __VPCLMULQDQ__ = @as(c_int, 1);
pub const __LAHF_SAHF__ = @as(c_int, 1);
pub const __LZCNT__ = @as(c_int, 1);
pub const __RDRND__ = @as(c_int, 1);
pub const __FSGSBASE__ = @as(c_int, 1);
pub const __BMI__ = @as(c_int, 1);
pub const __BMI2__ = @as(c_int, 1);
pub const __POPCNT__ = @as(c_int, 1);
pub const __PRFCHW__ = @as(c_int, 1);
pub const __RDSEED__ = @as(c_int, 1);
pub const __ADX__ = @as(c_int, 1);
pub const __MWAITX__ = @as(c_int, 1);
pub const __MOVBE__ = @as(c_int, 1);
pub const __SSE4A__ = @as(c_int, 1);
pub const __FMA__ = @as(c_int, 1);
pub const __F16C__ = @as(c_int, 1);
pub const __SHA__ = @as(c_int, 1);
pub const __FXSR__ = @as(c_int, 1);
pub const __XSAVE__ = @as(c_int, 1);
pub const __XSAVEOPT__ = @as(c_int, 1);
pub const __XSAVEC__ = @as(c_int, 1);
pub const __XSAVES__ = @as(c_int, 1);
pub const __PKU__ = @as(c_int, 1);
pub const __CLFLUSHOPT__ = @as(c_int, 1);
pub const __CLWB__ = @as(c_int, 1);
pub const __WBNOINVD__ = @as(c_int, 1);
pub const __SHSTK__ = @as(c_int, 1);
pub const __CLZERO__ = @as(c_int, 1);
pub const __RDPID__ = @as(c_int, 1);
pub const __RDPRU__ = @as(c_int, 1);
pub const __INVPCID__ = @as(c_int, 1);
pub const __CRC32__ = @as(c_int, 1);
pub const __AVX2__ = @as(c_int, 1);
pub const __AVX__ = @as(c_int, 1);
pub const __SSE4_2__ = @as(c_int, 1);
pub const __SSE4_1__ = @as(c_int, 1);
pub const __SSSE3__ = @as(c_int, 1);
pub const __SSE3__ = @as(c_int, 1);
pub const __SSE2__ = @as(c_int, 1);
pub const __SSE2_MATH__ = @as(c_int, 1);
pub const __SSE__ = @as(c_int, 1);
pub const __SSE_MATH__ = @as(c_int, 1);
pub const __MMX__ = @as(c_int, 1);
pub const __GCC_HAVE_SYNC_COMPARE_AND_SWAP_1 = @as(c_int, 1);
pub const __GCC_HAVE_SYNC_COMPARE_AND_SWAP_2 = @as(c_int, 1);
pub const __GCC_HAVE_SYNC_COMPARE_AND_SWAP_4 = @as(c_int, 1);
pub const __GCC_HAVE_SYNC_COMPARE_AND_SWAP_8 = @as(c_int, 1);
pub const __GCC_HAVE_SYNC_COMPARE_AND_SWAP_16 = @as(c_int, 1);
pub const unix = @as(c_int, 1);
pub const __unix = @as(c_int, 1);
pub const __unix__ = @as(c_int, 1);
pub const __GNU__ = @as(c_int, 1);
pub const __gnu_hurd__ = @as(c_int, 1);
pub const __MACH__ = @as(c_int, 1);
pub const __GLIBC__ = @as(c_int, 1);
pub const __STDC__ = @as(c_int, 1);
pub const __STDC_HOSTED__ = @as(c_int, 0);
pub const __STDC_VERSION__ = @as(c_long, 201710);
pub const __STDC_UTF_16__ = @as(c_int, 1);
pub const __STDC_UTF_32__ = @as(c_int, 1);
pub const __STDC_EMBED_NOT_FOUND__ = @as(c_int, 0);
pub const __STDC_EMBED_FOUND__ = @as(c_int, 1);
pub const __STDC_EMBED_EMPTY__ = @as(c_int, 2);
pub const _DEBUG = @as(c_int, 1);
pub const __GCC_HAVE_DWARF2_CFI_ASM = @as(c_int, 1);
pub const LIMINE_H = @as(c_int, 1);
pub const __CLANG_STDINT_H = "";
pub const __int_least64_t = i64;
pub const __uint_least64_t = u64;
pub const __int_least32_t = i64;
pub const __uint_least32_t = u64;
pub const __int_least16_t = i64;
pub const __uint_least16_t = u64;
pub const __int_least8_t = i64;
pub const __uint_least8_t = u64;
pub const __uint32_t_defined = "";
pub const __int8_t_defined = "";
pub const __stdint_join3 = @compileError("unable to translate C expr: unexpected token '##'");
// /home/fabian/.local/lib/include/stdint.h:291:9
pub const __intptr_t_defined = "";
pub const _INTPTR_T = "";
pub const _UINTPTR_T = "";
pub const __int_c_join = @compileError("unable to translate C expr: unexpected token '##'");
// /home/fabian/.local/lib/include/stdint.h:328:9
pub inline fn __int_c(v: anytype, suffix: anytype) @TypeOf(__int_c_join(v, suffix)) {
    _ = &v;
    _ = &suffix;
    return __int_c_join(v, suffix);
}
pub const __uint_c = @compileError("unable to translate macro: undefined identifier `U`");
// /home/fabian/.local/lib/include/stdint.h:330:9
pub const __int64_c_suffix = __INT64_C_SUFFIX__;
pub const __int32_c_suffix = __INT64_C_SUFFIX__;
pub const __int16_c_suffix = __INT64_C_SUFFIX__;
pub const __int8_c_suffix = __INT64_C_SUFFIX__;
pub inline fn INT64_C(v: anytype) @TypeOf(__int_c(v, __int64_c_suffix)) {
    _ = &v;
    return __int_c(v, __int64_c_suffix);
}
pub inline fn UINT64_C(v: anytype) @TypeOf(__uint_c(v, __int64_c_suffix)) {
    _ = &v;
    return __uint_c(v, __int64_c_suffix);
}
pub inline fn INT32_C(v: anytype) @TypeOf(__int_c(v, __int32_c_suffix)) {
    _ = &v;
    return __int_c(v, __int32_c_suffix);
}
pub inline fn UINT32_C(v: anytype) @TypeOf(__uint_c(v, __int32_c_suffix)) {
    _ = &v;
    return __uint_c(v, __int32_c_suffix);
}
pub inline fn INT16_C(v: anytype) @TypeOf(__int_c(v, __int16_c_suffix)) {
    _ = &v;
    return __int_c(v, __int16_c_suffix);
}
pub inline fn UINT16_C(v: anytype) @TypeOf(__uint_c(v, __int16_c_suffix)) {
    _ = &v;
    return __uint_c(v, __int16_c_suffix);
}
pub inline fn INT8_C(v: anytype) @TypeOf(__int_c(v, __int8_c_suffix)) {
    _ = &v;
    return __int_c(v, __int8_c_suffix);
}
pub inline fn UINT8_C(v: anytype) @TypeOf(__uint_c(v, __int8_c_suffix)) {
    _ = &v;
    return __uint_c(v, __int8_c_suffix);
}
pub const INT64_MAX = INT64_C(@import("std").zig.c_translation.promoteIntLiteral(c_int, 9223372036854775807, .decimal));
pub const INT64_MIN = -INT64_C(@import("std").zig.c_translation.promoteIntLiteral(c_int, 9223372036854775807, .decimal)) - @as(c_int, 1);
pub const UINT64_MAX = UINT64_C(@import("std").zig.c_translation.promoteIntLiteral(c_int, 18446744073709551615, .decimal));
pub const __INT_LEAST64_MIN = INT64_MIN;
pub const __INT_LEAST64_MAX = INT64_MAX;
pub const __UINT_LEAST64_MAX = UINT64_MAX;
pub const __INT_LEAST32_MIN = INT64_MIN;
pub const __INT_LEAST32_MAX = INT64_MAX;
pub const __UINT_LEAST32_MAX = UINT64_MAX;
pub const __INT_LEAST16_MIN = INT64_MIN;
pub const __INT_LEAST16_MAX = INT64_MAX;
pub const __UINT_LEAST16_MAX = UINT64_MAX;
pub const __INT_LEAST8_MIN = INT64_MIN;
pub const __INT_LEAST8_MAX = INT64_MAX;
pub const __UINT_LEAST8_MAX = UINT64_MAX;
pub const INT_LEAST64_MIN = __INT_LEAST64_MIN;
pub const INT_LEAST64_MAX = __INT_LEAST64_MAX;
pub const UINT_LEAST64_MAX = __UINT_LEAST64_MAX;
pub const INT_FAST64_MIN = __INT_LEAST64_MIN;
pub const INT_FAST64_MAX = __INT_LEAST64_MAX;
pub const UINT_FAST64_MAX = __UINT_LEAST64_MAX;
pub const INT32_MAX = INT32_C(@import("std").zig.c_translation.promoteIntLiteral(c_int, 2147483647, .decimal));
pub const INT32_MIN = -INT32_C(@import("std").zig.c_translation.promoteIntLiteral(c_int, 2147483647, .decimal)) - @as(c_int, 1);
pub const UINT32_MAX = UINT32_C(@import("std").zig.c_translation.promoteIntLiteral(c_int, 4294967295, .decimal));
pub const INT_LEAST32_MIN = __INT_LEAST32_MIN;
pub const INT_LEAST32_MAX = __INT_LEAST32_MAX;
pub const UINT_LEAST32_MAX = __UINT_LEAST32_MAX;
pub const INT_FAST32_MIN = __INT_LEAST32_MIN;
pub const INT_FAST32_MAX = __INT_LEAST32_MAX;
pub const UINT_FAST32_MAX = __UINT_LEAST32_MAX;
pub const INT16_MAX = INT16_C(@as(c_int, 32767));
pub const INT16_MIN = -INT16_C(@as(c_int, 32767)) - @as(c_int, 1);
pub const UINT16_MAX = UINT16_C(@import("std").zig.c_translation.promoteIntLiteral(c_int, 65535, .decimal));
pub const INT_LEAST16_MIN = __INT_LEAST16_MIN;
pub const INT_LEAST16_MAX = __INT_LEAST16_MAX;
pub const UINT_LEAST16_MAX = __UINT_LEAST16_MAX;
pub const INT_FAST16_MIN = __INT_LEAST16_MIN;
pub const INT_FAST16_MAX = __INT_LEAST16_MAX;
pub const UINT_FAST16_MAX = __UINT_LEAST16_MAX;
pub const INT8_MAX = INT8_C(@as(c_int, 127));
pub const INT8_MIN = -INT8_C(@as(c_int, 127)) - @as(c_int, 1);
pub const UINT8_MAX = UINT8_C(@as(c_int, 255));
pub const INT_LEAST8_MIN = __INT_LEAST8_MIN;
pub const INT_LEAST8_MAX = __INT_LEAST8_MAX;
pub const UINT_LEAST8_MAX = __UINT_LEAST8_MAX;
pub const INT_FAST8_MIN = __INT_LEAST8_MIN;
pub const INT_FAST8_MAX = __INT_LEAST8_MAX;
pub const UINT_FAST8_MAX = __UINT_LEAST8_MAX;
pub const __INTN_MIN = @compileError("unable to translate macro: undefined identifier `INT`");
// /home/fabian/.local/lib/include/stdint.h:875:10
pub const __INTN_MAX = @compileError("unable to translate macro: undefined identifier `INT`");
// /home/fabian/.local/lib/include/stdint.h:876:10
pub const __UINTN_MAX = @compileError("unable to translate macro: undefined identifier `UINT`");
// /home/fabian/.local/lib/include/stdint.h:877:9
pub const __INTN_C = @compileError("unable to translate macro: undefined identifier `INT`");
// /home/fabian/.local/lib/include/stdint.h:878:10
pub const __UINTN_C = @compileError("unable to translate macro: undefined identifier `UINT`");
// /home/fabian/.local/lib/include/stdint.h:879:9
pub const INTPTR_MIN = -__INTPTR_MAX__ - @as(c_int, 1);
pub const INTPTR_MAX = __INTPTR_MAX__;
pub const UINTPTR_MAX = __UINTPTR_MAX__;
pub const PTRDIFF_MIN = -__PTRDIFF_MAX__ - @as(c_int, 1);
pub const PTRDIFF_MAX = __PTRDIFF_MAX__;
pub const SIZE_MAX = __SIZE_MAX__;
pub const INTMAX_MIN = -__INTMAX_MAX__ - @as(c_int, 1);
pub const INTMAX_MAX = __INTMAX_MAX__;
pub const UINTMAX_MAX = __UINTMAX_MAX__;
pub const SIG_ATOMIC_MIN = __INTN_MIN(__SIG_ATOMIC_WIDTH__);
pub const SIG_ATOMIC_MAX = __INTN_MAX(__SIG_ATOMIC_WIDTH__);
pub const WINT_MIN = __INTN_MIN(__WINT_WIDTH__);
pub const WINT_MAX = __INTN_MAX(__WINT_WIDTH__);
pub const WCHAR_MAX = __WCHAR_MAX__;
pub const WCHAR_MIN = __INTN_MIN(__WCHAR_WIDTH__);
pub inline fn INTMAX_C(v: anytype) @TypeOf(__int_c(v, __INTMAX_C_SUFFIX__)) {
    _ = &v;
    return __int_c(v, __INTMAX_C_SUFFIX__);
}
pub inline fn UINTMAX_C(v: anytype) @TypeOf(__int_c(v, __UINTMAX_C_SUFFIX__)) {
    _ = &v;
    return __int_c(v, __UINTMAX_C_SUFFIX__);
}
pub inline fn LIMINE_PTR(TYPE: anytype) @TypeOf(TYPE) {
    _ = &TYPE;
    return TYPE;
}
pub const LIMINE_API_REVISION = @as(c_int, 0);
pub const LIMINE_DEPRECATED = @compileError("unable to translate macro: undefined identifier `__deprecated__`");
// /home/fabian/git/SoOS-zig/limine/limine.h:43:11
pub const LIMINE_DEPRECATED_IGNORE_START = @compileError("unable to translate macro: undefined identifier `_Pragma`");
// /home/fabian/git/SoOS-zig/limine/limine.h:44:11
pub const LIMINE_DEPRECATED_IGNORE_END = @compileError("unable to translate macro: undefined identifier `_Pragma`");
// /home/fabian/git/SoOS-zig/limine/limine.h:47:11
pub const LIMINE_REQUESTS_START_MARKER = @compileError("unable to translate macro: undefined identifier `limine_requests_start_marker`");
// /home/fabian/git/SoOS-zig/limine/limine.h:55:9
pub const LIMINE_REQUESTS_END_MARKER = @compileError("unable to translate macro: undefined identifier `limine_requests_end_marker`");
// /home/fabian/git/SoOS-zig/limine/limine.h:58:9
pub const LIMINE_REQUESTS_DELIMITER = LIMINE_REQUESTS_END_MARKER;
pub const LIMINE_BASE_REVISION = @compileError("unable to translate macro: undefined identifier `limine_base_revision`");
// /home/fabian/git/SoOS-zig/limine/limine.h:63:9
pub const LIMINE_BASE_REVISION_SUPPORTED = @compileError("unable to translate macro: undefined identifier `limine_base_revision`");
// /home/fabian/git/SoOS-zig/limine/limine.h:66:9
pub const LIMINE_LOADED_BASE_REV_VALID = @compileError("unable to translate macro: undefined identifier `limine_base_revision`");
// /home/fabian/git/SoOS-zig/limine/limine.h:68:9
pub const LIMINE_LOADED_BASE_REVISION = @compileError("unable to translate macro: undefined identifier `limine_base_revision`");
// /home/fabian/git/SoOS-zig/limine/limine.h:69:9
pub const LIMINE_COMMON_MAGIC = blk: {
    _ = @import("std").zig.c_translation.promoteIntLiteral(c_int, 0xc7b1dd30df4c8b88, .hex);
    break :blk @import("std").zig.c_translation.promoteIntLiteral(c_int, 0x0a82e883a194f07b, .hex);
};
pub const LIMINE_MEDIA_TYPE_GENERIC = @as(c_int, 0);
pub const LIMINE_MEDIA_TYPE_OPTICAL = @as(c_int, 1);
pub const LIMINE_MEDIA_TYPE_TFTP = @as(c_int, 2);
pub const LIMINE_BOOTLOADER_INFO_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:107:9
pub const LIMINE_EXECUTABLE_CMDLINE_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:123:9
pub const LIMINE_FIRMWARE_TYPE_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:138:9
pub const LIMINE_FIRMWARE_TYPE_X86BIOS = @as(c_int, 0);
pub const LIMINE_FIRMWARE_TYPE_UEFI32 = @as(c_int, 1);
pub const LIMINE_FIRMWARE_TYPE_UEFI64 = @as(c_int, 2);
pub const LIMINE_FIRMWARE_TYPE_SBI = @as(c_int, 3);
pub const LIMINE_STACK_SIZE_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:158:9
pub const LIMINE_HHDM_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:173:9
pub const LIMINE_FRAMEBUFFER_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:188:9
pub const LIMINE_FRAMEBUFFER_RGB = @as(c_int, 1);
pub const LIMINE_TERMINAL_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:241:9
pub const LIMINE_TERMINAL_CB_DEC = @as(c_int, 10);
pub const LIMINE_TERMINAL_CB_BELL = @as(c_int, 20);
pub const LIMINE_TERMINAL_CB_PRIVATE_ID = @as(c_int, 30);
pub const LIMINE_TERMINAL_CB_STATUS_REPORT = @as(c_int, 40);
pub const LIMINE_TERMINAL_CB_POS_REPORT = @as(c_int, 50);
pub const LIMINE_TERMINAL_CB_KBD_LEDS = @as(c_int, 60);
pub const LIMINE_TERMINAL_CB_MODE = @as(c_int, 70);
pub const LIMINE_TERMINAL_CB_LINUX = @as(c_int, 80);
pub const LIMINE_TERMINAL_CTX_SIZE = @import("std").zig.c_translation.cast(u64, -@as(c_int, 1));
pub const LIMINE_TERMINAL_CTX_SAVE = @import("std").zig.c_translation.cast(u64, -@as(c_int, 2));
pub const LIMINE_TERMINAL_CTX_RESTORE = @import("std").zig.c_translation.cast(u64, -@as(c_int, 3));
pub const LIMINE_TERMINAL_FULL_REFRESH = @import("std").zig.c_translation.cast(u64, -@as(c_int, 4));
pub const LIMINE_TERMINAL_OOB_OUTPUT_GET = @import("std").zig.c_translation.cast(u64, -@as(c_int, 10));
pub const LIMINE_TERMINAL_OOB_OUTPUT_SET = @import("std").zig.c_translation.cast(u64, -@as(c_int, 11));
pub const LIMINE_TERMINAL_OOB_OUTPUT_OCRNL = @as(c_int, 1) << @as(c_int, 0);
pub const LIMINE_TERMINAL_OOB_OUTPUT_OFDEL = @as(c_int, 1) << @as(c_int, 1);
pub const LIMINE_TERMINAL_OOB_OUTPUT_OFILL = @as(c_int, 1) << @as(c_int, 2);
pub const LIMINE_TERMINAL_OOB_OUTPUT_OLCUC = @as(c_int, 1) << @as(c_int, 3);
pub const LIMINE_TERMINAL_OOB_OUTPUT_ONLCR = @as(c_int, 1) << @as(c_int, 4);
pub const LIMINE_TERMINAL_OOB_OUTPUT_ONLRET = @as(c_int, 1) << @as(c_int, 5);
pub const LIMINE_TERMINAL_OOB_OUTPUT_ONOCR = @as(c_int, 1) << @as(c_int, 6);
pub const LIMINE_TERMINAL_OOB_OUTPUT_OPOST = @as(c_int, 1) << @as(c_int, 7);
pub const LIMINE_PAGING_MODE_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:301:9
pub const LIMINE_PAGING_MODE_X86_64_4LVL = @as(c_int, 0);
pub const LIMINE_PAGING_MODE_X86_64_5LVL = @as(c_int, 1);
pub const LIMINE_PAGING_MODE_MIN = LIMINE_PAGING_MODE_X86_64_4LVL;
pub const LIMINE_PAGING_MODE_DEFAULT = LIMINE_PAGING_MODE_X86_64_4LVL;
pub const LIMINE_5_LEVEL_PAGING_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:343:9
pub const LIMINE_SMP_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:365:11
pub const LIMINE_MP = @compileError("unable to translate macro: undefined identifier `limine_smp_`");
// /home/fabian/git/SoOS-zig/limine/limine.h:366:11
pub const LIMINE_SMP_X2APIC = @as(c_int, 1) << @as(c_int, 0);
pub const LIMINE_MEMMAP_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:458:9
pub const LIMINE_MEMMAP_USABLE = @as(c_int, 0);
pub const LIMINE_MEMMAP_RESERVED = @as(c_int, 1);
pub const LIMINE_MEMMAP_ACPI_RECLAIMABLE = @as(c_int, 2);
pub const LIMINE_MEMMAP_ACPI_NVS = @as(c_int, 3);
pub const LIMINE_MEMMAP_BAD_MEMORY = @as(c_int, 4);
pub const LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE = @as(c_int, 5);
pub const LIMINE_MEMMAP_KERNEL_AND_MODULES = @as(c_int, 6);
pub const LIMINE_MEMMAP_FRAMEBUFFER = @as(c_int, 7);
pub const LIMINE_ENTRY_POINT_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:493:9
pub const LIMINE_KERNEL_FILE_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:513:11
pub const LIMINE_MODULE_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:545:9
pub const LIMINE_INTERNAL_MODULE_REQUIRED = @as(c_int, 1) << @as(c_int, 0);
pub const LIMINE_INTERNAL_MODULE_COMPRESSED = @as(c_int, 1) << @as(c_int, 1);
pub const LIMINE_RSDP_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:578:9
pub const LIMINE_SMBIOS_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:597:9
pub const LIMINE_EFI_SYSTEM_TABLE_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:618:9
pub const LIMINE_EFI_MEMMAP_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:637:9
pub const LIMINE_BOOT_TIME_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:658:11
pub const LIMINE_KERNEL_ADDRESS_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:693:11
pub const LIMINE_DTB_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:722:9
pub const LIMINE_RISCV_BSP_HARTID_REQUEST = @compileError("unable to translate C expr: unexpected token '{'");
// /home/fabian/git/SoOS-zig/limine/limine.h:737:9
pub const limine_uuid = struct_limine_uuid;
pub const limine_file = struct_limine_file;
pub const limine_bootloader_info_response = struct_limine_bootloader_info_response;
pub const limine_bootloader_info_request = struct_limine_bootloader_info_request;
pub const limine_executable_cmdline_response = struct_limine_executable_cmdline_response;
pub const limine_executable_cmdline_request = struct_limine_executable_cmdline_request;
pub const limine_firmware_type_response = struct_limine_firmware_type_response;
pub const limine_firmware_type_request = struct_limine_firmware_type_request;
pub const limine_stack_size_response = struct_limine_stack_size_response;
pub const limine_stack_size_request = struct_limine_stack_size_request;
pub const limine_hhdm_response = struct_limine_hhdm_response;
pub const limine_hhdm_request = struct_limine_hhdm_request;
pub const limine_video_mode = struct_limine_video_mode;
pub const limine_framebuffer = struct_limine_framebuffer;
pub const limine_framebuffer_response = struct_limine_framebuffer_response;
pub const limine_framebuffer_request = struct_limine_framebuffer_request;
pub const limine_terminal = struct_limine_terminal;
pub const limine_terminal_response = struct_limine_terminal_response;
pub const limine_terminal_request = struct_limine_terminal_request;
pub const limine_paging_mode_response = struct_limine_paging_mode_response;
pub const limine_paging_mode_request = struct_limine_paging_mode_request;
pub const limine_5_level_paging_response = struct_limine_5_level_paging_response;
pub const limine_5_level_paging_request = struct_limine_5_level_paging_request;
pub const limine_smp_info = struct_limine_smp_info;
pub const limine_smp_response = struct_limine_smp_response;
pub const limine_smp_request = struct_limine_smp_request;
pub const limine_memmap_entry = struct_limine_memmap_entry;
pub const limine_memmap_response = struct_limine_memmap_response;
pub const limine_memmap_request = struct_limine_memmap_request;
pub const limine_entry_point_response = struct_limine_entry_point_response;
pub const limine_entry_point_request = struct_limine_entry_point_request;
pub const limine_kernel_file_response = struct_limine_kernel_file_response;
pub const limine_kernel_file_request = struct_limine_kernel_file_request;
pub const limine_internal_module = struct_limine_internal_module;
pub const limine_module_response = struct_limine_module_response;
pub const limine_module_request = struct_limine_module_request;
pub const limine_rsdp_response = struct_limine_rsdp_response;
pub const limine_rsdp_request = struct_limine_rsdp_request;
pub const limine_smbios_response = struct_limine_smbios_response;
pub const limine_smbios_request = struct_limine_smbios_request;
pub const limine_efi_system_table_response = struct_limine_efi_system_table_response;
pub const limine_efi_system_table_request = struct_limine_efi_system_table_request;
pub const limine_efi_memmap_response = struct_limine_efi_memmap_response;
pub const limine_efi_memmap_request = struct_limine_efi_memmap_request;
pub const limine_boot_time_response = struct_limine_boot_time_response;
pub const limine_boot_time_request = struct_limine_boot_time_request;
pub const limine_kernel_address_response = struct_limine_kernel_address_response;
pub const limine_kernel_address_request = struct_limine_kernel_address_request;
pub const limine_dtb_response = struct_limine_dtb_response;
pub const limine_dtb_request = struct_limine_dtb_request;
pub const limine_riscv_bsp_hartid_response = struct_limine_riscv_bsp_hartid_response;
pub const limine_riscv_bsp_hartid_request = struct_limine_riscv_bsp_hartid_request;
