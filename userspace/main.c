#include <stdint.h>

#include "soos_syscall.h"

const char *MESSAGE = "Hello, world!\n";

void test() {
    while (1) {
        asm("nop");
    }
}

void _start() {
    while (1) {
        print(MESSAGE);

        for (uint64_t i = 0; i < 100 * 1000 * 1000; ++i) {
            asm("nop");
        }
    }
}