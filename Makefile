CC = ./x86-i686--glibc--stable-2022.08-1/bin/i686-buildroot-linux-gnu-gcc.br_real
AS = ./x86-i686--glibc--stable-2022.08-1/bin/i686-buildroot-linux-gnu-as
CFLAGS = -g -ffreestanding -O2 -nostdlib -Wall -Wextra -march=i386 -std=gnu99 -Ilib -Isrc -I/usr/include
SOURCES = $(shell find src -name '*.c')
OBJS = $(patsubst src/%,build/%,$(SOURCES:.c=.o))

run: SoOS.iso
	qemu-system-i386 -cdrom $^ -d guest_errors -m 16 # -S -gdb tcp::1234

SoOS.iso: build/soos.bin iso/boot/grub/grub.cfg
	grub-file --is-x86-multiboot build/soos.bin
	cp build/soos.bin iso/boot/
	grub-mkrescue -o $@ iso

build/soos.bin: build/boot.o $(OBJS)
	$(CC) -T linker.ld -o $@ $(CFLAGS) $^ -lgcc
	objcopy --only-keep-debug $@ $@.debug
	objcopy --strip-debug $@

$(OBJS): $(SOURCES)
	@mkdir -p $(@D)
	$(foreach src,$(SOURCES),$(CC) -c $(src) -o $(patsubst src/%,build/%,$(src:.c=.o)) $(CFLAGS) || exit;)

build/boot.o: src/boot.asm
	nasm -f elf32 $^ -o $@

clean:
	rm -r build/*
	rm SoOS.iso 2>/dev/null || true
