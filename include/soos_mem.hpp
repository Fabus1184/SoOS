#pragma once

#include <inttypes.h>
#include <stddef.h>

void memcpy(const uint8_t *source, uint8_t *dest, const uint32_t nbytes);

void memset(uint8_t *dest, const uint8_t val, const uint32_t nbytes);
