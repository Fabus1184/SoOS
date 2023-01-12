#ifndef SOOS_MEMORY_H
#define SOOS_MEMORY_H

#include <stddef.h>
#include <stdint.h>

void memcpy(void *dest, const void *src, size_t size);

void memmove(void *dest, const void *src, size_t size);

void memset(void *dest, uint8_t byte, size_t size);

void reverse(uint8_t *data, size_t len);

#endif  // SOOS_MEMORY_H
