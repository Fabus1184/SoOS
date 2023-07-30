#include <stdint.h>

void print(const char *str) {
    asm("push %[str]\n"
        "push $0\n"
        "int $0x80\n"
        "pop %%rax\n"
        "pop %%rax\n"

        ::[str] "r"(str)
        : "rax");
}

void sleep(int64_t ms) {
    asm("push %[ms]\n"
        "push $1\n"
        "int $0x80\n"
        "pop %%rax\n"
        "pop %%rax\n"

        ::[ms] "r"(ms)
        : "rax");
}
