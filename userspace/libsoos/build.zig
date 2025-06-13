const std = @import("std");

pub fn build(b: *std.Build) !void {
    var features = std.Target.Cpu.Feature.Set.empty;
    features.addFeature(@intFromEnum(std.Target.x86.Feature.x87));
    features.addFeature(@intFromEnum(std.Target.x86.Feature.sse));
    features.addFeature(@intFromEnum(std.Target.x86.Feature.sse2));
    features.addFeature(@intFromEnum(std.Target.x86.Feature.sse3));
    features.addFeature(@intFromEnum(std.Target.x86.Feature.avx));

    const target = b.standardTargetOptions(.{
        .default_target = .{
            .cpu_arch = .x86_64,
            .os_tag = .freestanding,
            .abi = .gnu,
            .ofmt = .elf,
            .cpu_model = .native,
            .cpu_features_add = features,
        },
    });

    const optimize = b.standardOptimizeOption(.{ .preferred_optimize_mode = .Debug });

    const lib = b.addModule("libsoos", .{
        .root_source_file = b.path("src/libsoos.zig"),
        .target = target,
        .optimize = optimize,
    });
    lib.stack_check = false;

    lib.addIncludePath(b.path("../../"));
}
