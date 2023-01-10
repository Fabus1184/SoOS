#pragma once

#include <stdint.h>
#include <stddef.h>

void memcpy(void *dest, const void *src, size_t n);

void memset(void *dest, uint8_t val, size_t n);