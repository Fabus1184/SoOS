#pragma once

#include <cinttypes>

#include "soos_string.hpp"
#include "soos_mem.hpp"
#include "print.hpp"
#include "timer.hpp"
#include "commands.hpp"

#define PROMPT "$ SoOS CLI -> "
#define N_BUF 60

[[noreturn]] void init_shell();
