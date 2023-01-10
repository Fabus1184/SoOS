#include "stdlib.h"

size_t strlen(const char *str) {
    size_t len = 0;
    while (str[len] != '\0') {
        ++len;
    }
    return len;
}

int32_t itoa(int32_t value, char *str) {
    int32_t i = 0;
    int32_t sign = 0;

    if (value < 0) {
        sign = 1;
        value = -value;
    }

    do {
        str[i++] = value % 10 + '0';
    } while ((value /= 10) > 0);

    if (sign) {
        str[i++] = '-';
    }

    str[i] = '\0';

    reverse(str, sizeof(char), i);

    return i;
}

void reverse(void *data, size_t size, size_t count) {
    char *start = data;
    char *end = start + size * (count - 1);
    char tmp;

    while (start < end) {
        tmp = *start;
        *start = *end;
        *end = tmp;

        start += size;
        end -= size;
    }
}