#include "print.h"
#include "soos_string.h"
#include "isr.h"

void kmain()
{
	isr_install();

	asm volatile("int $1");
gi
	clear_screen();
	println("Welcome to SoOS!");

	for (uint16_t i = 0; i < COLS; i++) {
		print_char('-');
	}

	println("");

	char buf[15];

	println("Powers of two:");
	for (uint32_t i = 0; i < 22; ++i) {
		print(itoa(pow(2, i), buf));
		print(", ");
	}

	println("");

	println("Primes:");
	for (uint32_t i = 2; i < 200; ++i) {
		if (isPrime(i)) {
			print(itoa(i, buf));
			print(", ");
		}
	}

	println("");
	println("Fibonacci numbers:");
	for (uint16_t i = 0; i < 20; ++i) {
		print(itoa(fib(i), buf));
		print(", ");
	}

	// This should never return
	asm volatile("hlt");
}
