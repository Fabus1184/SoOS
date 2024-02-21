clean:
	rm SoOS.iso || true
	rm userspace/build/* || true

run: userspace/build/test
	cargo build -vv
	
	rm -rf iso_root || true
	rm -f SoOS.iso || true

	make -C limine
	mkdir -p iso_root

	cp -v target/x86_64-unknown-none/debug/soos iso_root/kernel.elf
	cp -v limine.cfg limine/limine-bios.sys limine/limine-bios-cd.bin limine/limine-uefi-cd.bin iso_root/

	mkdir -p iso_root/EFI/BOOT
	cp -v limine/BOOT*.EFI iso_root/EFI/BOOT/

	xorriso -as mkisofs -b limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o SoOS.iso

	./limine/limine bios-install SoOS.iso

	qemu-system-x86_64 -cpu qemu64,+la57 -cdrom SoOS.iso -d guest_errors -m 8G -d cpu_reset -M smm=off -s

limine:
	git clone https://github.com/limine-bootloader/limine.git --branch=v5.x-branch-binary --depth=1
	make -C limine

userspace/build/%: userspace/src/%.c
	gcc -ffreestanding -nostdlib -nodefaultlibs -nostartfiles -fno-stack-protector -o $@ $<