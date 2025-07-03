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

    const userspace = b.addSystemCommand(&.{ "cargo", "build" });
    userspace.setCwd(b.path("userspace"));

    const kernel = b.dependency("kernel", .{ .target = target }).artifact("kernel.elf");
    kernel.step.dependOn(&userspace.step);

    const install = b.addInstallArtifact(
        kernel,
        .{
            .dest_dir = .{ .override = .{ .custom = "../build/iso-root" } },
        },
    );

    b.default_step.dependOn(&install.step);

    const xorriso = b.addSystemCommand(&.{
        "xorriso",
        "-as",
        "mkisofs",
        "-b",
        "limine-bios-cd.bin",
        "-no-emul-boot",
        "-boot-load-size",
        "4",
        "-boot-info-table",
        "--efi-boot",
        "limine-uefi-cd.bin",
        "-efi-boot-part",
        "--efi-boot-image",
        "--protective-msdos-label",
        "./build/iso-root",
        "-o",
        "./build/SoOS.iso",
    });
    b.default_step.dependOn(&xorriso.step);

    const limine_install = b.addSystemCommand(&.{
        "./limine/limine",
        "bios-install",
        "./build/SoOS.iso",
    });
    b.default_step.dependOn(&limine_install.step);

    const qemu = b.addSystemCommand(&.{
        "qemu-system-x86_64",
        "-cpu",
        "max",
        "-d",
        "guest_errors",
        "-m",
        "8G",
        "-s",
        "-no-shutdown",
        "-no-reboot",
        "-drive",
        "file=build/SoOS.iso,media=cdrom",
        "-boot",
        "d",
        "-drive",
        "id=disk,file=testfs,if=none",
        "-device",
        "ahci,id=ahci",
        "-device",
        "ide-hd,drive=disk,bus=ahci.0",
    });
    qemu.step.dependOn(b.default_step);

    const run_step = b.step("run", "boot the image in qemu");
    run_step.dependOn(&qemu.step);
}
