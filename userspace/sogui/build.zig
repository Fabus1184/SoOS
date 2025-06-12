const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{
        .default_target = .{
            .cpu_arch = .x86_64,
            .os_tag = .freestanding,
            .abi = .gnu,
            .ofmt = .elf,
            .cpu_model = .native,
        },
    });

    const optimize = b.standardOptimizeOption(.{ .preferred_optimize_mode = .ReleaseFast });

    const exe = b.addExecutable(.{
        .name = "sogui",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    const libsoos = b.dependency("libsoos", .{ .target = target });
    exe.root_module.addImport("soos", libsoos.module("libsoos"));

    b.installArtifact(exe);
}
