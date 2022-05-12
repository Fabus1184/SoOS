TARGET = all

CC = x86-i686-cross/bin/i686-linux-gcc
LD = x86-i686-cross/bin/i686-linux-ld

SRCS = ${wildcard src/*/*.cpp}
HEADS = ${wildcard include/*/*.hpp}
OBJS = ${SRCS:src/%.cpp=build/%.o build/assembly/interrupt.o build/assembly/int32.o}

CFLAGS := -Os -ffreestanding -fno-exceptions -fno-rtti -fno-pie -nostdlib -fno-builtin -fno-stack-protector \
-nostartfiles -nodefaultlibs -Wall -Wextra -Werror \
-Iinclude/drivers -Iinclude/interrupts -Iinclude/kernel -Iinclude/lib

all: clean
	@$(MAKE) -f $(lastword $(MAKEFILE_LIST)) os-image.bin

os-image.bin: boot.bin kernel.bin 
	cat boot.bin kernel.bin > os-image.bin

boot.bin: src/assembly/boot.asm
	nasm -f bin -o $@ $^

kernel.bin: build/assembly/kernel_entry.o build/kernel/kernel.o $(OBJS)
	$(LD) -T Linker.ld --allow-multiple-definition -o $@ $^ --oformat=binary

build/%.o: src/%.cpp
	$(CC) $(CFLAGS) -c $< -o $@

build/%.o: src/%.asm
	nasm $< -f elf -o $@

run: all
	./run.sh

clean:
	find . -name "*.bin" -delete
	find . -name "*.o" -delete
