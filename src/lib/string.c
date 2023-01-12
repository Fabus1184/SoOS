#include "string.h"

size_t strlen(const char *str) {
    size_t len = 0;
    while (0 != str[len]) {
        ++len;
    }
    return len;
}

char *itoa(size_t n, char *s, uint8_t base) {
    const char *digits = "0123456789ABCDEF";
    size_t i = 0;
    do {
        s[i++] = digits[n % base];
        n /= base;
    } while (n > 0);
    s[i] = '\0';
    reverse((uint8_t *) s, i);
    return s;
}
