build:
	gcc -fstack-protector -O0 -fPIC -fPIE -ffreestanding -nostdlib -o main.elf main.c soos.c soos_syscall.c

clean:
	rm main.elf || true