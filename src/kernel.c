#include "print.h"
#include "soos_string.h"
#include "isr.h"
#include "vga.h"

__attribute__((unused)) _Noreturn void kmain()
{

	isr_install();
	irq_install();

	clear_screen();

	char welcome[] = "Welcome to SoOS!";

	print("+");
	for (uint16_t i = 2; i < COLS; i++) print("-");
	println("+");

	print("|");
	for (uint16_t i = 1; i < (COLS - strlen(welcome)) / 2; i++) print(" ");
	print(welcome);
	for (uint16_t i = 1; i < (COLS - strlen(welcome)) / 2; i++) print(" ");
	println("|");

	print("+");
	for (uint16_t i = 2; i < COLS; i++) print("-");
	println("+");

	println("");

	char buf[15];

	println("Powers of two:");
	for (uint32_t i = 0; i < 22; ++i) {
		print(itoa(pow(2, i), buf));
		print(", ");
	}

	println("");
	println("");

	println("Primes:");
	for (uint32_t i = 2; i < 200; ++i) {
		if (isPrime(i)) {
			print(itoa(i, buf));
			print(", ");
		}
	}

	println("");
	println("");

	println("Fibonacci numbers:");
	for (uint16_t i = 0; i < 20; ++i) {
		print(itoa(fib(i), buf));
		print(", ");
	}

	println("");
	println("");

	init_shell();

	// this should never return
	asm volatile("hlt");
	while (1) asm volatile("nop");
}
