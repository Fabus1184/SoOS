CC = gcc
CFLAGS = -ffreestanding -nostdlib -nodefaultlibs -nostartfiles -fno-stack-protector

all: build/libsoos.a build/fib

build/%: examples/%.c build/libsoos.a
	$(CC) $(CFLAGS) -o $@ $< -Lbuild -lsoos

build/libsoos.a: libsoos/libsoos.c
	$(CC) $(CFLAGS) -c -o $@.o $<
	ar rcs $@ $@.o

clean:
	rm -f build/*