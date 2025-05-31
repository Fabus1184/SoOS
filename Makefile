LIMINE=limine-9.3.3
LIMINE_BIN=limine-9.3.3/bin/limine
LIMINE_FILES = $(patsubst %, $(LIMINE)/bin/%, limine-bios.sys limine-bios-cd.bin limine-uefi-cd.bin)

clean:
	rm SoOS.iso || true

run: $(LIMINE_FILES) $(LIMINE_BIN)
	make -C userspace

	@echo "Limine files: $(LIMINE_FILES)"

	cargo build
	
	rm -rf iso_root || true
	rm -f SoOS.iso || true

	mkdir -p iso_root

	cp -v target/x86_64-unknown-none/debug/soos iso_root/kernel.elf
	cp -v limine.conf $(LIMINE_FILES) iso_root/

	mkdir -p iso_root/EFI/BOOT
	cp -v $(LIMINE)/bin/BOOT*.EFI iso_root/EFI/BOOT/

	xorriso -as mkisofs -b limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o SoOS.iso

	$(LIMINE_BIN) bios-install SoOS.iso

	qemu-system-x86_64 -cpu qemu64,+la57 -cdrom SoOS.iso -d guest_errors -m 8G -d cpu_reset -M smm=off -s -no-reboot

$(LIMINE_FILES) $(LIMINE_BIN): $(LIMINE)
	cd $(LIMINE) && ./configure --enable-bios --enable-bios-cd --enable-uefi-x86-64 --enable-uefi-cd && make -j$(nproc)
