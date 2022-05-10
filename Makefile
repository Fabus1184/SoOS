TARGET = all

CC = x86-i686-cross/bin/i686-linux-gcc
LD = x86-i686-cross/bin/i686-linux-gcc

SRCS = ${wildcard src/*.cpp}
HEADS = ${wildcard include/*.hpp}
OBJS = ${SRCS:src/%.cpp=build/%.o build/interrupt.o build/int32.o}

CFLAGS := -Os -ffreestanding -fno-exceptions -fno-rtti -fno-pie -Iinclude -nostdlib -fno-builtin -fno-stack-protector -nostartfiles -nodefaultlibs -Wall -Wextra -Werror

all: os-image.bin
	find . -name "*.o" -delete
	find . -name "*.bin" -not -name "os-image.bin" -delete

os-image.bin: boot.bin kernel.bin 
	cat boot.bin kernel.bin > os-image.bin

boot.bin: assembly/boot.nasm
	nasm -f bin -o $@ $^

kernel.bin: build/kernel_entry.o build/kernel.o $(OBJS)
	$(LD) -Wl,--allow-multiple-definition -Wl,-Ttext=0x7e00 -Wl,--oformat=binary $^ $(CFLAGS) -o $@ -lgcc

build/%.o: src/%.cpp
	$(CC) $(CFLAGS) -c $< -o $@

build/%.o: assembly/%.nasm
	nasm $< -f elf -o $@

run: os-image.bin
	qemu-system-i386 -drive format=raw,file=os-image.bin -d guest_errors -soundhw pcspk

clean:
	find . -name "*.bin" -delete
	find . -name "*.o" -delete
	
