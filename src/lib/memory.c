#include "memory.h"

void memcpy(void *dest, const void *src, size_t size) {
    for (size_t i = 0; i < size; ++i) {
        ((uint8_t *) dest)[i] = ((const uint8_t *) src)[i];
    }
}

void memmove(void *dest, const void *src, size_t size) {
    if (dest < src) {
        memcpy(dest, src, size);
    } else {
        for (size_t i = size; i > 0; --i) {
            ((uint8_t *) dest)[i - 1] = ((const uint8_t *) src)[i - 1];
        }
    }
}

void memset(void *dest, uint8_t byte, size_t size) {
    for (size_t i = 0; i < size; ++i) {
        ((uint8_t *) dest)[i] = byte;
    }
}

void reverse(uint8_t *data, size_t len) {
    size_t i = 0;
    size_t j = len - 1;
    while (i < j) {
        uint8_t temp = data[i];
        data[i] = data[j];
        data[j] = temp;
        ++i;
        --j;
    }
}