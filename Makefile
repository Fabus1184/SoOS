clean:
	rm SoOS.iso || true

run:
	cargo build --release
	./iso.sh
	qemu-system-x86_64 -cdrom SoOS.iso -d guest_errors -m 8G -d cpu_reset
