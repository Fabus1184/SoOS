#ifndef SOOS_STRING_H
#define SOOS_STRING_H

#include <stdint.h>
#include <stddef.h>

#include <lib/memory.h>

size_t strlen(const char *str);

char *itoa(size_t n, char *s, uint8_t base);

#endif //SOOS_STRING_H
