#pragma once

#include <inttypes.h>

#include "soos_string.h"
#include "soos_mem.h"
#include "print.h"
#include "timer.h"
#include "commands.h"

#define PROMPT "SooSHELL $>"
#define CURSOR "_"

void input(char c);

void init_shell();
