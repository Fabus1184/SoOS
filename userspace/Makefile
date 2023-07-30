build:
	gcc -fPIC -fPIE -ffreestanding -nostdlib -o main.elf main.c soos_syscall.c
	gcc -fPIC -fPIE -ffreestanding -nostdlib -o loop.elf loop.c

clean:
	rm main.elf || true