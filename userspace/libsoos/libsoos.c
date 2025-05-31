#include <stdarg.h>
#include <stdint.h>

void itoa(int32_t value, char *str) {
    int32_t i = 0;
    int32_t is_negative = 0;

    if (value < 0) {
        is_negative = 1;
        value = -value;
    }

    do {
        str[i++] = value % 10 + '0';
        value /= 10;
    } while (value > 0);

    if (is_negative) {
        str[i++] = '-';
    }

    str[i] = '\0';

    for (int32_t j = 0; j < i / 2; j++) {
        char tmp = str[j];
        str[j] = str[i - j - 1];
        str[i - j - 1] = tmp;
    }
}

uint64_t strlen(const char *str) {
    int32_t len = 0;
    while (str[len] != '\0') {
        len++;
    }
    return len;
}

enum SYSCALL {
    SYSCALL_PRINT = 0,
    SYSCALL_SLEEP = 1,
    SYSCALL_EXIT = 2,
};

int32_t syscall(enum SYSCALL _syscall, ...) {
    va_list args;
    va_start(args, _syscall);

    uint64_t syscall = (uint64_t)_syscall;

    int32_t ret = -1;
    switch (syscall) {
    case SYSCALL_PRINT: {
        char *str = va_arg(args, char *);
        uint64_t len = va_arg(args, uint64_t);

        asm volatile("mov %[syscall], %%rax\n"
                     "mov %[str], %%rbx\n"
                     "mov %[len], %%rcx\n"
                     "int $0x80\n"
                     : "=a"(ret)
                     : [syscall] "m"(syscall), [str] "m"(str), [len] "m"(len)
                     : "rbx", "rcx");

        break;
    }
    case SYSCALL_SLEEP: {
        uint64_t ms = va_arg(args, uint64_t);

        asm volatile("mov %[syscall], %%rax\n"
                     "mov %[ms], %%rbx\n"
                     "int $0x80\n"
                     : "=a"(ret)
                     : [syscall] "m"(syscall), [ms] "m"(ms)
                     : "rbx");

        break;
    }
    case SYSCALL_EXIT: {
        uint64_t status = va_arg(args, uint64_t);

        asm volatile("mov %[syscall], %%rax\n"
                     "mov %[status], %%rbx\n"
                     "int $0x80\n"
                     : "=a"(ret)
                     : [syscall] "m"(syscall), [status] "m"(status)
                     : "rbx");

        break;
    }
    default:
        break;
    }

    va_end(args);

    return ret;
}

void print(const char *str) { syscall(SYSCALL_PRINT, str, strlen(str)); }
void sleep(uint64_t ms) { syscall(SYSCALL_SLEEP, ms); }
void exit(uint64_t status) { syscall(SYSCALL_EXIT, status); }