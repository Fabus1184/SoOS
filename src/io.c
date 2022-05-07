#include "io.h"

uint8_t io_in(uint16_t addr)
{
	uint8_t res;

	asm volatile(
		"in %%dx, %%al" : "=a" (res) : "d" (addr)
		);
	return res;
}

void io_out(uint8_t data, uint16_t addr)
{
	asm volatile(
		"out %%al, %%dx" : : "a" (data), "d" (addr)
		);
}

int32_t io_sys_clock()
{
	asm volatile (
		"mov $0, %ax\n"
		"int $0x1a"
		);

	return 0;
}