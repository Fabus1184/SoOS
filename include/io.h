#pragma once

#include <inttypes.h>

uint8_t io_in(uint16_t addr);

void io_out(uint8_t data, uint16_t addr);
