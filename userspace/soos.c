#include <stdint.h>

uint8_t char_to_int(char c) {
    if (c >= '0' && c <= '9') {
        return c - '0';
    } else {
        return -1;
    }
}

char int_to_char(uint8_t i) {
    if (i >= 0 && i <= 9) {
        return i + '0';
    } else {
        return '\0';
    }
}