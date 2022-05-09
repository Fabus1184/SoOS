TARGET = all

CC = x86-i686-cross/bin/i686-linux-gcc
LD = x86-i686-cross/bin/i686-linux-ld

SRCS = ${wildcard src/*.c printf/*.c}
HEADS = ${wildcard include/*.h printf/*.h}
OBJS = ${SRCS:.c=.o assembly/interrupt.o assembly/vga.o}

CFLAGS = -Os -ffreestanding -fno-pie -Iinclude -nostdlib -fno-builtin -fno-stack-protector -nostartfiles -nodefaultlibs -Wall -Wextra -Werror

all: os-image.bin
	find . -name "*.o" -delete
	find . -name "*.bin" -not -name "os-image.bin" -delete

os-image.bin: boot.bin kernel.bin 
	cat boot.bin kernel.bin > os-image.bin

boot.bin: assembly/boot.asm
	nasm -f bin -o $@ $^

kernel.bin: assembly/kernel_entry.o src/kernel.o $(OBJS)
	$(LD) --allow-multiple-definition -m elf_i386 -o $@ -Ttext 0x7e00 $^ --oformat binary

.c.o:
	${CC} ${CFLAGS} -c $< -o $@

assembly/%.o: assembly/%.asm
	nasm $< -f elf -o $@

run: os-image.bin
	qemu-system-i386 -drive format=raw,file=os-image.bin -d guest_errors -soundhw pcspk

clean:
	find . -name "*.bin" -delete
	find . -name "*.o" -delete
	
