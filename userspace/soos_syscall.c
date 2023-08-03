#include <stdint.h>

void print(const char *str) {
    asm volatile("mov $0, %%rax\n"
                 "mov %[str], %%rbx\n"
                 "int $0x80\n"
                 :
                 : [str] "r"(str)
                 : "rax", "rbx");
}

void sleep(uint64_t ms) {
    asm volatile("mov $1, %%rax\n"
                 "mov %[ms], %%rbx\n"
                 "int $0x80\n"
                 :
                 : [ms] "r"(ms)
                 : "rax", "rbx");
}

uint64_t getpid(void) {
    uint64_t pid;
    asm volatile("mov $2, %%rax\n"
                 "mov %[pid], %%rbx\n"
                 "int $0x80\n"

                 : "=r"(pid)
                 : [pid] "r"(&pid)
                 : "rax", "rbx");

    return pid;
}
