#include "timer.h"

uint32_t tick = 0;

static void timer_callback(registers_t *regs)
{
	(void) (regs);

	tick++;
	char buf[10];
	println(itoa(tick, buf));
}

void init_timer(uint32_t freq)
{
	register_interrupt_handler(IRQ0, timer_callback);

	uint32_t div = 1193180 / freq;
	uint8_t low = div & 0xFF;
	uint8_t high = (div >> 8) & 0xFF;

	io_out(0x36, 0x43);
	io_out(low, 0x40);
	io_out(high, 0x40);
}