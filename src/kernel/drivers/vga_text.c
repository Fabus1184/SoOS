#include "vga_text.h"

uint8_t line = 0;
uint8_t col = 0;
uint16_t *fb = NULL;
uint8_t cols = 0;
uint8_t rows = 0;
uint8_t fg = 0xF;
uint8_t bg = 0x0;

void set_color(uint8_t _fg, uint8_t _bg) {
    fg = _fg;
    bg = _bg;
}

void init_text_mode(void) {
    write_regs(g_90x30_text);
    set_cursor(0);
    line = 0;
    col = 0;
    cols = 90;
    rows = 30;
    fb = (uint16_t *) (get_fb_seg() * 16);
    fg = 0xF;
    bg = 0x0;

    for (uint32_t i = 0; i < cols * rows; ++i) {
        fb[i] = (bg << 12) | (fg << 8) | ' ';
    }
}

void kputchar(char c) {
    if (col == cols) {
        col = 0;
        ++line;
    }

    if (line == rows) {
        memmove(fb, fb + cols, cols * (rows - 1) * 2);
        line = rows - 1;
        for (uint32_t i = 0; i < cols; ++i) {
            fb[line * cols + i] = (bg << 12) | (fg << 8) | ' ';
        }
    }

    if ('\n' == c) {
        ++line;
        col = 0;
    } else {
        fb[line * cols + col] = (bg << 12) | (fg << 8) | c;
        ++col;
    }

    set_cursor(col + (line * cols));
}

void kprintf(const char *fmt, ...) {
    va_list args1, args2;
    va_start(args1, fmt);
    va_copy(args2, args1);

    uint32_t size = stbsp_vsnprintf(NULL, 0, fmt, args1);
    va_end(args1);

    char buf[size + 1];
    stbsp_vsprintf(buf, fmt, args2);
    va_end(args2);

    for (uint32_t i = 0; i < size; ++i) {
        kputchar(buf[i]);
    }
}

void kprintf_color(uint8_t fg, uint8_t bg, const char *fmt, ...) {
    uint8_t old_fg = fg;
    uint8_t old_bg = bg;

    set_color(fg, bg);

    va_list args;
    va_start(args, fmt);

    uint32_t size = stbsp_vsnprintf(NULL, 0, fmt, args);
    char buf[size + 1];
    stbsp_vsprintf(buf, fmt, args);

    for (uint32_t i = 0; i < size; ++i) {
        kputchar(buf[i]);
    }

    va_end(args);

    set_color(old_fg, old_bg);
}