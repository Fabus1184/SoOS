const std = @import("std");

pub fn build(b: *std.Build) void {
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
            .cpu_model = .baseline,
            .cpu_features_add = features,
        },
    });

    const optimize = b.standardOptimizeOption(.{ .preferred_optimize_mode = .ReleaseFast });

    const exe = b.addExecutable(.{
        .name = "sogui",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    exe.root_module.stack_check = false;

    const libsoos = b.dependency("libsoos", .{});
    exe.root_module.addImport("soos", libsoos.module("libsoos"));

    const zigimg_dependency = b.dependency("zigimg", .{
        .target = target,
        .optimize = optimize,
    });
    const zigimg_module = zigimg_dependency.module("zigimg");
    zigimg_module.stack_check = false;
    exe.root_module.addImport("zigimg", zigimg_module);

    const zclay_dep = b.dependency("zclay", .{
        .target = target,
        .optimize = optimize,
    });
    exe.root_module.addImport("zclay", zclay_dep.module("zclay"));

    b.installArtifact(exe);
}
