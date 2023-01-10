#include "memory.h"

void memcpy(void *dest, const void *src, size_t n) {
    for (size_t i = 0; i < n; i++) {
        ((uint8_t *) dest)[i] = ((uint8_t *) src)[i];
    }
}

void memset(void *dest, uint8_t val, size_t n) {
    for (size_t i = 0; i < n; i++) {
        ((uint8_t *) dest)[i] = val;
    }
}
