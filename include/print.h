#pragma once

#include "io.h"
#include "soos_string.h"
#include "soos_mem.h"

#define ROWS 25
#define COLS 80

#define SCREEN_CTRL_ADDR 0x3d4
#define SCREEN_DATA_ADDR 0x3d5

#define COLOR 0x0a

static uint16_t *const VIDEO_MEM = (uint16_t *) 0xb8000;

uint16_t get_cursor();

void set_cursor(uint16_t cursor);

void print_char(char c);

void print(const char *c);

void println(const char *c);

void clear_screen();
