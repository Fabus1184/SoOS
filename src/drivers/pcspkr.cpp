#include "pcspkr.hpp"

void play_sound(uint32_t freq)
{
	uint32_t div;
	uint8_t tmp;

	div = 1193180 / freq;

	io_out(0xb6, 0x43);
	io_out((uint8_t) div, 0x42);
	io_out((uint8_t) (div >> 8), 0x42);

	tmp = io_in(0x61);

	if (tmp != (tmp | 3)) {
		io_out(tmp | 3, 0x61);
	}
}

void nosound()
{
	uint8_t tmp = io_in(0x61) & 0xFC;
	io_out(tmp, 0x61);
}

void beep(const char *args)
{
	(void)(args);
	play_sound(1000);
	wait(10);
	nosound();
}
