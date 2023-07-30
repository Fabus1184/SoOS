#include <stdint.h>

#include "soos_syscall.h"

char MESSAGE[] = "Hello, world!\n";

void test() {
    while (1) {
        asm("nop");
    }
}

void _start() {
    while (1) {
        print(MESSAGE);

        MESSAGE[0] += 1;

        sleep(100);
    }
}