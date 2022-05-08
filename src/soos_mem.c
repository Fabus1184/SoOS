#include "soos_mem.h"

uint32_t mem_ptr = 0x10000;

void memcpy(uint8_t *source, uint8_t *dest, uint32_t nbytes)
{
	for (uint32_t n = 0; n < nbytes; n++) {
		dest[n] = source[n];
	}
}

void memset(uint8_t *dest, uint8_t val, uint32_t nbytes)
{
	for (uint32_t n = 0; n < nbytes; ++n) {
		dest[n] = val;
	}
}
