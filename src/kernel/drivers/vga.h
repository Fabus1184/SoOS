#pragma once

#include <stdint.h>
#include <stddef.h>

#include "../../interrupts/isr.h"

#define VGA_WIDTH 320
#define VGA_HEIGHT 200
#define VGA_MEMORY_ADDRESS 0xa0000

typedef struct __attribute__ ((packed)) {
    uint16_t di, si, bp, sp, bx, dx, cx, ax;
    uint16_t gs, fs, es, ds, eflags;
} regs16_t;

void switch_text_mode();

void switch_vga_mode();

void put_pixel(uint16_t x, uint16_t y, uint8_t col);
