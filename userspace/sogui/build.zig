const std = @import("std");

pub fn build(b: *std.Build) void {
    var disabled = std.Target.Cpu.Feature.Set.empty;
    disabled.addFeature(@intFromEnum(std.Target.x86.Feature.mmx));
    disabled.addFeature(@intFromEnum(std.Target.x86.Feature.sse));
    disabled.addFeature(@intFromEnum(std.Target.x86.Feature.sse2));
    disabled.addFeature(@intFromEnum(std.Target.x86.Feature.avx));
    disabled.addFeature(@intFromEnum(std.Target.x86.Feature.avx2));

    var enabled = std.Target.Cpu.Feature.Set.empty;
    enabled.addFeature(@intFromEnum(std.Target.x86.Feature.soft_float));

    const target = b.standardTargetOptions(.{
        .default_target = .{
            .cpu_arch = .x86_64,
            .os_tag = .freestanding,
            .abi = .gnu,
            .ofmt = .elf,
            .cpu_model = .baseline,
            .cpu_features_add = enabled,
            .cpu_features_sub = disabled,
        },
    });

    const optimize = b.standardOptimizeOption(.{ .preferred_optimize_mode = .ReleaseSmall });

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
