#pragma once

#include <cinttypes>

#include "io.hpp"
#include "isr.hpp"

#define N_LAST_KEYS 100

void init_keyboard();

char get_char();