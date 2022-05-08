#pragma once

#include <inttypes.h>

#include "print.h"
#include "rtc.h"
#include "soos_mem.h"
#include "pcspkr.h"

#define N_CMDS 4

void clear();
void help();
void time();
void beep();

struct Command {
	const char *cmd;
	const void *func;
};

struct Command commands[N_CMDS] = {
	{"clear", clear},
	{"help", help},
	{"time", time},
	{"beep", beep}
};
