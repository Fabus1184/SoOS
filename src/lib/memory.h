#ifndef SOOS_MEMORY_H
#define SOOS_MEMORY_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

void memcpy(void *dest, const void *src, size_t size);

void memmove(void *dest, const void *src, size_t size);

void memset(void *dest, uint8_t byte, size_t size);

void reverse(uint8_t *data, size_t len);

uint32_t memcmp(const void *a, const void *b, size_t size);

bool find(void *array, uint32_t size, uint32_t element_size, void *element);

#endif  // SOOS_MEMORY_H
