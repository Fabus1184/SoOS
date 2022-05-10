#pragma once

#include <inttypes.h>

#include "soos_string.hpp"
#include "soos_mem.hpp"
#include "print.hpp"
#include "timer.hpp"
#include "commands.hpp"

#define PROMPT "$ SoOS CLI -> "
#define CURSOR "_"

void input(char c);

void init_shell();
