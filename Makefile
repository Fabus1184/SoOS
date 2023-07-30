clean:
	rm SoOS.iso || true

run:
	cd userspace && make clean && make && cd ..
	cargo build
	./iso.sh
	qemu-system-x86_64 -cpu qemu64,+la57 -cdrom SoOS.iso -d guest_errors -m 8G -d cpu_reset -M smm=off -s 

limine:
	git clone https://github.com/limine-bootloader/limine.git --branch=v5.x-branch-binary --depth=1
	make -C limine
