#pragma once

#include <inttypes.h>

#include "print.hpp"
#include "rtc.hpp"
#include "soos_mem.hpp"
#include "pcspkr.hpp"
#include "vga.hpp"

#define N_CMDS 6

void clear(const char *args);
void help(const char *args);
void time(const char *args);
void beep(const char *args);
void sleep(const char *args);
void draw(const char *args);

struct Command {
	const char *cmd;
	void (*func) (const char *args);
};

struct Command commands[N_CMDS] = {
	{"clear", clear},
	{"help", help},
	{"time", time},
	{"beep", beep},
	{"sleep", sleep},
	{"draw", draw}
};