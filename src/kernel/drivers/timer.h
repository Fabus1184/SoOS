#pragma once

#include <stdint.h>
#include "../../lib/io.h"
#include "../../interrupts/isr.h"
#include "../../lib/print.h"

static uint32_t tick;
static uint32_t freq;

void init_timer(uint32_t freq);

void wait(uint16_t ms);

