const std = @import("std");

pub fn build(b: *std.Build) !void {
    const target = b.standardTargetOptions(.{
        .default_target = .{
            .cpu_arch = .x86_64,
            .os_tag = .freestanding,
            .abi = .gnu,
            .ofmt = .elf,
            .cpu_model = .baseline,
        },
    });

    const optimize = b.standardOptimizeOption(.{ .preferred_optimize_mode = .ReleaseSmall });

    _ = b.addModule("libsoos", .{
        .root_source_file = b.path("src/libsoos.zig"),
        .target = target,
        .optimize = optimize,
    });
}
