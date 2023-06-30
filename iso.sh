#!/bin/bash

set -e
set -x

rm -rf iso_root || true
rm -f SoOS.iso || true

make -C limine
mkdir -p iso_root

cp -v target/x86_64-unknown-none/release/SoOS iso_root/kernel.elf
cp -v limine.cfg limine/limine-bios.sys limine/limine-bios-cd.bin limine/limine-uefi-cd.bin iso_root/

mkdir -p iso_root/EFI/BOOT
cp -v limine/BOOT*.EFI iso_root/EFI/BOOT/

xorriso -as mkisofs -b limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    iso_root -o SoOS.iso

./limine/limine bios-install SoOS.iso
