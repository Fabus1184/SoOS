#pragma once

#include <stdint.h>
#include <stddef.h>

int32_t itoa(int32_t value, char *str);

size_t strlen(const char *str);

void reverse(void *data, size_t size, size_t count);
