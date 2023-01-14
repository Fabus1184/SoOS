#ifndef SOOS_STRING_H
#define SOOS_STRING_H

#include <lib/memory.h>
#include <stddef.h>
#include <stdint.h>

size_t strlen(const char *str);

char *itoa(size_t n, char *s, uint8_t base);

uint64_t prefix_decimal(uint64_t n, char **prefix);

#endif  // SOOS_STRING_H
