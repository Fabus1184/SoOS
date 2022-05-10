#include "print.hpp"

uint16_t get_cursor()
{
	// high byte
	io_out(14, SCREEN_CTRL_ADDR);
	uint16_t offset = io_in(SCREEN_DATA_ADDR) << 8;

	// low byte
	io_out(15, SCREEN_CTRL_ADDR);
	offset += io_in(SCREEN_DATA_ADDR);

	return offset;
}

void scroll()
{
	memcpy((uint8_t *) VIDEO_MEM + (COLS * 2), (uint8_t *) VIDEO_MEM, (COLS * ROWS * 2));
	memset((uint8_t *) VIDEO_MEM + (COLS * (ROWS - 1) * 2), 0x0, COLS * 2);
}

void set_cursor(uint16_t cursor)
{
	// high byte
	io_out(14, SCREEN_CTRL_ADDR);
	io_out((uint8_t) (cursor >> 8), SCREEN_DATA_ADDR);

	// low byte
	io_out(15, SCREEN_CTRL_ADDR);
	io_out((uint8_t) (cursor & 0xFF), SCREEN_DATA_ADDR);
}

void print_char(const char c)
{
	uint16_t cursor = get_cursor();

	switch (c) {
		case '\n':
			println("");
			break;

		case '\b':
			set_cursor(cursor - 1);
			print(" ");
			set_cursor(cursor - 1);
			break;

		case '\t':
			print("    ");
			break;
			
		default:

			if (cursor >= ROWS * COLS) {
				scroll();
				set_cursor(cursor - COLS);
			}

			cursor = get_cursor();
			VIDEO_MEM[cursor] = (COLOR << 8) + c;
			set_cursor(cursor + 1);
			break;
	}
}

void print(const char *c)
{
	for (uint16_t i = 0; c[i] != 0; i++) {
		print_char(c[i]);
	}
}

void println(const char *c)
{
	if (c[0] == 0) {
		print(" ");
	} else {
		print(c);
	}

	if (get_cursor() % COLS != 0) {
		set_cursor(COLS + get_cursor() - (get_cursor() % COLS));
	}
}

void clear_screen()
{
	uint32_t size = COLS * ROWS;
	for (uint32_t i = 0; i < size; i++) {
		VIDEO_MEM[i] = 0x000a;
	}
	set_cursor(0);
}
