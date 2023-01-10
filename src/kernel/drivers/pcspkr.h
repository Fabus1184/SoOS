#pragma once

#include <stdint.h>

#include "../../lib/io.h"
#include "timer.h"

void nosound();

void beep(int16_t frequency, int16_t ms_duration);
