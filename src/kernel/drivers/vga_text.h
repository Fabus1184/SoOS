#ifndef SOOS_VGA_TEXT_H
#define SOOS_VGA_TEXT_H

#include <kernel/drivers/vga.h>
#include <stb/stb_sprintf.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

#define VGA_CRTC_INDEX 0x3D4
#define VGA_CRTC_DATA 0x3D5

#define COLOR16_BLACK 0x00
#define COLOR16_BLUE 0x01
#define COLOR16_GREEN 0x02
#define COLOR16_CYAN 0x03
#define COLOR16_RED 0x04
#define COLOR16_MAGENTA 0x05
#define COLOR16_BROWN 0x06
#define COLOR16_LIGHT_GREY 0x07
#define COLOR16_DARK_GREY 0x08
#define COLOR16_LIGHT_BLUE 0x09
#define COLOR16_LIGHT_GREEN 0x0A
#define COLOR16_LIGHT_CYAN 0x0B
#define COLOR16_LIGHT_RED 0x0C
#define COLOR16_LIGHT_MAGENTA 0x0D
#define COLOR16_LIGHT_BROWN 0x0E
#define COLOR16_WHITE 0x0F

void kprintf(const char *fmt, ...);

void kprintf_color(uint8_t fg, uint8_t bg, const char *fmt, ...);

void kputchar(char c);

void set_color(uint8_t fg, uint8_t bg);

void init_text_mode(void);

#endif  // SOOS_VGA_TEXT_H