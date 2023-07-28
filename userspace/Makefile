SRCS = $(wildcard *.c)

build:
	gcc -fPIC -fPIE -ffreestanding -nostdlib -o main.elf $(SRCS)

clean:
	rm main.elf || true