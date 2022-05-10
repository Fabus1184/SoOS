#pragma once

#include "soos_math.hpp"
#include "soos_mem.hpp"

#include <stdint.h>

char *itoa(uint32_t n, char *buf);

char *reverse(char *buf, uint16_t n);

bool strcmp(const char *s1, const char *s2);

uint32_t strlen(const char *str);

char *pad(uint16_t padding, char filler, char *buf);
