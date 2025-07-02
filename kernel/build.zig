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

    const exe = b.addExecutable(.{
        .name = "kernel.elf",
        .root_source_file = b.path("src/kernel.zig"),
        .target = target,
        .optimize = optimize,
        .code_model = .kernel,
    });
    exe.linker_script = b.path("link.ld");
    exe.root_module.stack_check = false;

    exe.addIncludePath(b.path("../"));

    const zigimg_dependency = b.dependency("zigimg", .{
        .target = target,
        .optimize = optimize,
    });
    exe.root_module.addImport("zigimg", zigimg_dependency.module("zigimg"));

    exe.addIncludePath(b.path(".."));

    b.installArtifact(exe);
}
