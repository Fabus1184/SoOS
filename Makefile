#CC = gcc
CC = x86-i686-cross/bin/i686-linux-gcc
LD = x86-i686-cross/bin/i686-linux-ld

#CFLAGS = -g -m32 -nostdlib -nostdinc -fno-builtin -fno-stack-protector -nostartfiles -nodefaultlibs -Wall -Wextra -Werror
CFLAGS = -ffreestanding -fno-pie

all: os-image.bin

os-image.bin: boot.bin kernel.bin 
	cat $^ > $@

boot.bin: boot.asm
	nasm -f bin -o $@ $^

kernel.bin: kernel_entry.o kernel.o
	$(LD) -m elf_i386 -o $@ -Ttext 0x1000 $^ --oformat binary

%.o: %.c ${HEADERS}
	${CC} ${CFLAGS} -c $< -o $@

%.o: %.asm
	nasm $< -f elf -o $@

run: os-image.bin
	qemu-system-i386 -drive format=raw,file=os-image.bin -d guest_errors,int

clean:
	find . -name "*.bin" -delete
	find . -name "*.o" -delete
	
