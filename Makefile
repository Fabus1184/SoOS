clean:
	rm SoOS.iso || true

run:
	cargo build
	./iso.sh
	qemu-system-x86_64 -cdrom SoOS.iso -d guest_errors -m 4G -d cpu_reset
