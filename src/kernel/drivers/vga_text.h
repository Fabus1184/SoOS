#ifndef HOME_FABIAN_GIT_SOOS_SRC_KERNEL_DRIVERS_VGA_TEXT_H
#define HOME_FABIAN_GIT_SOOS_SRC_KERNEL_DRIVERS_VGA_TEXT_H

#include <kernel/drivers/vga.h>
#include <stb/stb_sprintf.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

enum { VGA_CRTC_INDEX = 0x3D4, VGA_CRTC_DATA = 0x3D5 };

enum {
    COLOR16_BLACK = 0x00,
    COLOR16_BLUE = 0x01,
    COLOR16_GREEN = 0x02,
    COLOR16_CYAN = 0x03,
    COLOR16_RED = 0x04,
    COLOR16_MAGENTA = 0x05,
    COLOR16_BROWN = 0x06,
    COLOR16_LIGHT_GREY = 0x07,
    COLOR16_DARK_GREY = 0x08,
    COLOR16_LIGHT_BLUE = 0x09,
    COLOR16_LIGHT_GREEN = 0x0A,
    COLOR16_LIGHT_CYAN = 0x0B,
    COLOR16_LIGHT_RED = 0x0C,
    COLOR16_LIGHT_MAGENTA = 0x0D,
    COLOR16_LIGHT_BROWN = 0x0E,
    COLOR16_WHITE = 0x0F
};

void kprintf(const char *fmt, ...);

void kprintf_color(uint8_t fg, uint8_t bg, const char *fmt, ...);

void kputchar(char c);

void set_color(uint8_t fg, uint8_t bg);

void init_text_mode(void);

#endif  // HOME_FABIAN_GIT_SOOS_SRC_KERNEL_DRIVERS_VGA_TEXT_H