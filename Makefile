LIMINE=build/limine-9.3.3
LIMINE_BIN=build/limine-9.3.3/bin/limine
LIMINE_FILES = $(patsubst %, $(LIMINE)/bin/%, limine-bios.sys limine-bios-cd.bin limine-uefi-cd.bin)

RELEASE=1

KERNEL=build/kernel/x86_64-unknown-none/$(if $(RELEASE),release,debug)/soos
KERNEL_SOURCES := $(shell find kernel -type f)

USERSPACE_APPLICATIONS=$(patsubst %, build/userspace/bin/%, sosh sogui)

clean:
	rm SoOS.iso || true
	rm -rf build/* || true

$(LIMINE): limine-9.3.3.tar.gz
	mkdir -p $(LIMINE)
	tar -xzf limine-9.3.3.tar.gz -C build

$(LIMINE_FILES) $(LIMINE_BIN): $(LIMINE)
	cd $(LIMINE) && ./configure --enable-bios --enable-bios-cd --enable-uefi-x86-64 --enable-uefi-cd && make -j$(nproc)

build/userspace/bin:
	mkdir -p build/userspace/bin

build/userspace/bin/%: userspace/% build/userspace/bin
	cd $< && zig build -p ../../build/userspace

$(KERNEL): $(USERSPACE_APPLICATIONS) $(KERNEL_SOURCES)
	cd kernel && cargo build $(if $(RELEASE),--release)

build/iso-root: $(KERNEL) $(LIMINE_FILES)
	mkdir -p build/iso-root
	cp -v $(KERNEL) build/iso-root/kernel.elf
	cp -v limine.conf $(LIMINE_FILES) build/iso-root/
	mkdir -p build/iso-root/EFI/BOOT
	cp -v $(LIMINE)/bin/BOOT*.EFI build/iso-root/EFI/BOOT/

build/SoOS.iso: build/iso-root $(LIMINE_FILES) $(LIMINE_BIN) $(KERNEL)
	xorriso -as mkisofs -b limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		build/iso-root -o build/SoOS.iso

	$(LIMINE_BIN) bios-install build/SoOS.iso

run: build/SoOS.iso
	qemu-system-x86_64 \
		-cpu qemu64,+la57 -cdrom build/SoOS.iso -d guest_errors,cpu_reset -m 8G -M smm=off -s \
		-no-shutdown -no-reboot
