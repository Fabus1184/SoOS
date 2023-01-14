#ifndef SOOS_VGA_H
#define SOOS_VGA_H

#include <lib/io.h>
#include <lib/memory.h>
#include <stddef.h>
#include <stdint.h>

extern uint8_t g_40x25_text[];
extern uint8_t g_40x50_text[];
extern uint8_t g_80x25_text[];
extern uint8_t g_80x50_text[];
extern uint8_t g_90x30_text[];
extern uint8_t g_90x60_text[];
extern uint8_t g_640x480x2[];
extern uint8_t g_320x200x4[];
extern uint8_t g_640x480x16[];
extern uint8_t g_720x480x16[];
extern uint8_t g_320x200x256[];
extern uint8_t g_320x200x256_modex[];
extern uint8_t g_8x8_font[2048];
extern uint8_t g_8x16_font[4096];

void read_regs(uint8_t *regs);

void write_regs(uint8_t *regs);

void set_plane(uint32_t p);

uint32_t get_fb_seg(void);

void write_font(uint8_t *buf, uint32_t font_height);

void set_cursor(uint32_t offset);

#endif  // SOOS_VGA_H