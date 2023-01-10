#ifndef SOOS_MEMORY_H
#define SOOS_MEMORY_H

#include <stdint.h>
#include <stddef.h>

void memcpy(void *dest, const void *src, size_t size);

void memset(void *dest, uint8_t byte, size_t size);

size_t strlen(const char *str);

#endif //SOOS_MEMORY_H
