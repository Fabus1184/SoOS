TARGET = all

CC = x86-i686-cross/bin/i686-linux-gcc
LD = x86-i686-cross/bin/i686-linux-ld

SRCS = ${wildcard src/*.c}
HEADS = ${wildcard include/*.h}
OBJS = ${SRCS:.c=.o interrupt.o}

CFLAGS = -ffreestanding -fno-pie -Iinclude -nostdlib -fno-builtin -fno-stack-protector -nostartfiles -nodefaultlibs -Wall -Wextra -Werror

all: os-image.bin

os-image.bin: boot.bin kernel.bin 
	cat boot.bin kernel.bin > os-image.bin

boot.bin: boot.asm
	nasm -f bin -o $@ $^

kernel.bin: kernel_entry.o kernel.o $(OBJS)
	$(LD) --allow-multiple-definition -m elf_i386 -o $@ -Ttext 0x1000 $^ --oformat binary

.c.o:
	${CC} ${CFLAGS} -c $< -o $@

kernel_entry.o: kernel_entry.asm
	nasm $< -f elf -o $@

interrupt.o: interrupt.asm
	nasm $< -f elf -o $@

run: os-image.bin
	qemu-system-i386 -drive format=raw,file=os-image.bin -d guest_errors,int

clean:
	find . -name "*.bin" -delete
	find . -name "*.o" -delete
	
