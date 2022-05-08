#pragma once

#include <inttypes.h>
#include <stddef.h>

void memcpy(uint8_t *source, uint8_t *dest, uint32_t nbytes);

void memset(uint8_t *dest, uint8_t val, uint32_t nbytes);
