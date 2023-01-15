CC = gcc-cross/bin/x86_64-elf-gcc
CFLAGS = -m64 -g -ffreestanding -O2 -nostdlib -Wall -Wextra -march=x86-64 -std=gnu99 -Ilib -Isrc -I/usr/include
SOURCES = $(shell find src -name '*.c')
ASM_SOURCES = $(shell find src -name '*.asm')
OBJS = $(patsubst src/%,build/%,$(SOURCES:.c=.o)) 
ASMS = $(patsubst src/%,build/%,$(ASM_SOURCES:.asm=.o))

run: SoOS.iso
	qemu-system-x86_64 -cdrom $^ -d guest_errors -m 8G -d cpu_reset -gdb tcp::1234

SoOS.iso: build/soos.bin iso/boot/grub/grub.cfg
	grub-file --is-x86-multiboot build/soos.bin
	cp build/soos.bin iso/boot/
	grub-mkrescue -o $@ iso

build/soos.bin: $(ASMS) $(OBJS)
	$(CC) -T linker.ld --verbose -o $@ $(CFLAGS) $^ -lgcc
	objcopy --only-keep-debug $@ $@.debug
	objcopy --strip-debug $@

$(OBJS): $(SOURCES)
	$(foreach src,$(SOURCES), mkdir -p `dirname $(patsubst src/%,build/%,$(src:.c=.o))`)
	$(foreach src,$(SOURCES), $(CC) -c $(src) -o $(patsubst src/%,build/%,$(src:.c=.o)) $(CFLAGS) || exit;)

build/%.o: src/%.asm
	nasm -f elf64 $^ -o $@

clean:
	rm -r build/*
	rm SoOS.iso 2>/dev/null || true
