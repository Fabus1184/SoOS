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

uint32_t memcmp(const void *a, const void *b, size_t size) {
    for (size_t i = 0; i < size; ++i) {
        if (((const uint8_t *) a)[i] != ((const uint8_t *) b)[i]) {
            return ((const uint8_t *) a)[i] - ((const uint8_t *) b)[i];
        }
    }
    return 0;
}

bool find(void *array, uint32_t size, uint32_t element_size, void *element) {
    for (uint32_t i = 0; i < size; ++i) {
        if (memcmp(array + i * element_size, element, element_size) == 0) {
            return true;
        }
    }
    return false;
}
