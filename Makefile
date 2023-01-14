CC = gcc
CFLAGS = -m32 -no-pie -fno-stack-protector -g -ffreestanding -O2 -nostdlib -Wall -Wextra -march=i386 -std=gnu99 -Ilib -Isrc -I/usr/include
SOURCES = $(shell find src -name '*.c')
ASM_SOURCES = $(shell find src -name '*.asm')
OBJS = $(patsubst src/%,build/%,$(SOURCES:.c=.o)) 
ASMS = $(patsubst src/%,build/%,$(ASM_SOURCES:.asm=.o))

run: SoOS.iso
	qemu-system-i386 -cdrom $^ -d guest_errors -m 16 -d cpu_reset # -S -gdb tcp::1234

SoOS.iso: build/soos.bin iso/boot/grub/grub.cfg
	grub-file --is-x86-multiboot build/soos.bin
	cp build/soos.bin iso/boot/
	grub-mkrescue -o $@ iso

build/soos.bin: $(ASMS) $(OBJS)
	$(CC) -T linker.ld -Wl,--fatal-warnings -o $@ $(CFLAGS) $^ -lgcc
	objcopy --only-keep-debug $@ $@.debug
	objcopy --strip-debug $@

$(OBJS): $(SOURCES)
	@mkdir -p $(@D)
	$(foreach src,$(SOURCES),$(CC) -c $(src) -o $(patsubst src/%,build/%,$(src:.c=.o)) $(CFLAGS) || exit;)

build/%.o: src/%.asm
	nasm -f elf32 $^ -o $@

clean:
	rm -r build/*
	rm SoOS.iso 2>/dev/null || true
