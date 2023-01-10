#include "memory.h"

void memcpy(void *dest, const void *src, size_t size) {
    for (size_t i = 0; i < size; ++i) {
        ((uint8_t *) dest)[i] = ((const uint8_t *) src)[i];
    }
}

void memset(void *dest, uint8_t byte, size_t size) {
    for (size_t i = 0; i < size; ++i) {
        ((uint8_t *) dest)[i] = byte;
    }
}

size_t strlen(const char *str) {
    size_t len = 0;
    while (0 != str[len]) {
        ++len;
    }
    return len;
}
