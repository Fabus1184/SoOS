#include <stdarg.h>
#include <stdint.h>

void itoa(int32_t value, char *str);
uint64_t strlen(const char *str);

void print(const char *str);
void sleep(uint64_t ms);
void exit(uint64_t status);