#include "vga.h"

regs16_t regs;

// int32 test
void switch_gm()
{	
	// switch to 320x200x256 graphics mode
	regs.ax = 0x0013;
	//regs.ax = 0x0012;
	int32(0x10, &regs);
}

void switch_tm()
{
	// switch to 80x25x16 text mode
	regs.ax = 0x0003;
	int32(0x10, &regs);

	input(0);
}

void putpixel(uint16_t x, uint16_t y, uint8_t col)
{
	uint8_t *location = (uint8_t*) VGA_ADDR + (VGA_WIDTH * y) + x;
	*location = col; 	
}
