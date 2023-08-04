#include <stdint.h>

#include "soos.h"
#include "soos_syscall.h"

char MESSAGE[] = "Hello, world!\n";

void test() {
    while (1) {
        asm("nop");
    }
}

void _start() {
    uint64_t pid = getpid();
    MESSAGE[0] = int_to_char((uint8_t)pid);

    while (1) {
        print(MESSAGE);
        sleep(100);
    }
}