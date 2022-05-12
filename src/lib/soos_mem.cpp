#include "soos_mem.hpp"

void memcpy(const uint8_t *source, uint8_t *dest, const uint32_t nbytes)
{
	for (uint32_t n = 0; n < nbytes; n++) {
		dest[n] = source[n];
	}
}

void memset(uint8_t *dest, const uint8_t val, const uint32_t nbytes)
{
	for (uint32_t n = 0; n < nbytes; ++n) {
		dest[n] = val;
	}
}
