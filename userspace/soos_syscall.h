#ifndef SOOS_SYSCALL_H
#define SOOS_SYSCALL_H

#include <stdint.h>

void print(const char *str);

void sleep(uint64_t ms);

#endif // SOOS_SYSCALL_H