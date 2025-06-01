#include <stdarg.h>
#include <stdint.h>

void itoa(int32_t value, char *str);
uint64_t strlen(const char *str);
void memset(void *ptr, uint8_t value, uint64_t num);
int32_t strcmp(const char *str1, const char *str2);

void print(const char *str);
void sleep(uint64_t ms);
void exit(uint64_t status);
uint64_t listdir(const char *path, uint64_t index, char *buffer);
uint64_t read(uint64_t fd, void *buffer, uint64_t size);
uint32_t fork(void);
int64_t open(const char *path);
int64_t close(uint64_t fd);