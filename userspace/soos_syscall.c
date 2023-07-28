#include <stdint.h>

void print(const char *str) {
    asm("push %0\n"
        "push $0\n"
        "int $0x80\n"
        "pop %%rax\n"
        "pop %%rax\n"

        ::"r"(str)
        : "rax");
}