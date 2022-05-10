#pragma once

#include <inttypes.h>

#include "soos_mem.hpp"
#include "shell.hpp"
#include "isr.hpp"

#define VGA_WIDTH 320
#define VGA_HEIGHT 200
#define VGA_ADDR 0xa0000

typedef struct __attribute__ ((packed)) {
	uint16_t di, si, bp, sp, bx, dx, cx, ax;
	uint16_t gs, fs, es, ds, eflags;
} regs16_t;

extern "C" void int32(uint8_t intnum, regs16_t *regs);

void switch_tm();

void switch_gm();

void putpixel(uint16_t x, uint16_t y, uint8_t col);
