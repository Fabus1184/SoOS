#pragma once

#include <stdint.h>

#include "../../lib/io.h"
#include "../../interrupts/isr.h"

#define N_LAST_KEYS 100

void init_keyboard(void);

char get_char(void);