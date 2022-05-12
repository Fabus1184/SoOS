#pragma once

#include <cinttypes>

#include "print.hpp"
#include "rtc.hpp"
#include "soos_mem.hpp"
#include "pcspkr.hpp"
#include "vga.hpp"
#include "music.hpp"
#include "graphics.hpp"

#define N_CMDS 7

void clear(const char *args);

void help(const char *args);

void time(const char *args);

void sleep(const char *args);

void draw(const char *args);

void test(const char *args);

extern void imperial_march(const char *args);

struct Command
{
	const char *cmd;

	void (*func)(const char *args);
};

struct Command commands[N_CMDS] = {
	{"clear", clear},
	{"help",  help},
	{"time",  time},
	{"im",    imperial_march},
	{"sleep", sleep},
	{"draw",  draw},
	{"test",  test}
};
