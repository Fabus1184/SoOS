TARGET = all

CC = gcc
LD = ld

SRCS = ${wildcard src/*/*.c src/*/*/*.c}
HEADS = ${wildcard src/*/*.h src/*/*/*.h}
OBJS = ${SRCS:.c=.o src/assembly/interrupt.o src/assembly/int32.o}

CFLAGS := -m32 -Os -ffreestanding -fno-exceptions -fno-pie -nostdlib -fno-builtin -fno-stack-protector -nostartfiles \
	-nodefaultlibs -Wall -Wextra -Isrc -std=gnu11

all: clean
	@echo "Sources: " $(SRCS)
	@echo "Headers: " $(HEADS)
	@$(MAKE) -f $(lastword $(MAKEFILE_LIST)) os-image.bin

os-image.bin: boot.bin kernel.bin 
	cat boot.bin kernel.bin > os-image.bin

boot.bin: src/assembly/boot.asm
	nasm -f bin -o $@ $^

kernel.bin: src/assembly/kernel_entry.o src/kernel/kernel.o $(OBJS)
	$(LD) --allow-multiple-definition -melf_i386 -T Linker.ld -o $@ $^ --oformat=binary

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

src/assembly/%.o: src/assembly/%.asm
	nasm $< -f elf -o $@

run: all
	./run.sh

clean:
	find . -name "*.bin" -delete
	find . -name "*.o" -delete
