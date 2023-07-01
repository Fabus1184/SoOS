clean:
	rm SoOS.iso || true

run:
	cargo build --release
	./iso.sh
	qemu-system-x86_64 -cpu qemu64,+la57 -cdrom SoOS.iso -d guest_errors -m 8G -d cpu_reset

limine:
	git clone https://github.com/limine-bootloader/limine.git --branch=v5.x-branch-binary --depth=1
	make -C limine
