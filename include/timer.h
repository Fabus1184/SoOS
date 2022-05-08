#pragma once

#include <inttypes.h>
#include "io.h"
#include "isr.h"
#include "print.h"
#include "shell.h"

void init_timer(uint32_t f);

uint32_t get_timer();

void wait(uint16_t ms);
