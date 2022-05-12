#include "vga.hpp"

static regs16_t regs;

extern "C" void int32(uint8_t int_num, regs16_t *regs);

/**
 * @brief switch to vga graphics mode, resolution and color depth is determined by ax from
 * 00 	text 40*25 16 color (mono)
 * 01 	text 40*25 16 color
 * 02 	text 80*25 16 color (mono)
 * 03 	text 80*25 16 color
 * 04 	CGA 320*200 4 color
 * 05 	CGA 320*200 4 color (m)
 * 06 	CGA 640*200 2 color
 * 07 	MDA monochrome text 80*25
 * 08 	PCjr
 * 09 	PCjr
 * 0A 	PCjr
 * 0B 	reserved
 * 0C 	reserved
 * 0D 	EGA 320*200 16 color
 * 0E 	EGA 640*200 16 color
 * 0F 	EGA 640*350 mono
 * 10 	EGA 640*350 16 color
 * 11 	VGA 640*480 mono
 * 12 	VGA 640*480 16 color
 * 13 	VGA 320*200 256 color
 */
void switch_vga_mode()
{
	regs.ax = 0x0013;
	int32(0x10, &regs);
}

/**
 * @brief switch back to text mode
 */
void switch_text_mode()
{
	regs.ax = 0x0003;
	int32(0x10, &regs);
}

/**
 * @brief set color of pixel at (x, y) to col
 * @warning only works in vga graphics mode!
 * @param x
 * @param y
 * @param col
 */
void put_pixel(uint16_t x, uint16_t y, uint8_t col)
{
	if (x > VGA_WIDTH) return;
	if (y > VGA_HEIGHT) return;
	*((uint8_t *) VGA_MEMORY_ADDRESS + (VGA_WIDTH * y) + x) = col;
}
