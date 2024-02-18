#include <stdarg.h>
#include <stdint.h>

enum SYSCALL {
    SYSCALL_PRINT = 0,
};

int syscall(enum SYSCALL syscall, ...) {
    va_list args;
    va_start(args, syscall);

    int ret = -1;
    switch (syscall) {
    case SYSCALL_PRINT: {
        char *str = va_arg(args, char *);
        uint64_t len = va_arg(args, uint64_t);

        asm volatile("mov $0, %%rax\n"
                     "mov %[str], %%rbx\n"
                     "mov %[len], %%rcx\n"
                     "int $0x80\n"
                     : "=a"(ret)
                     : [str] "m"(str), [len] "m"(len)
                     : "rbx", "rcx");

        break;
    }
    default:
        break;
    }

    va_end(args);

    return ret;
}

void itoa(int value, char *str) {
    int i = 0;
    int is_negative = 0;

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

    for (int j = 0; j < i / 2; j++) {
        char tmp = str[j];
        str[j] = str[i - j - 1];
        str[i - j - 1] = tmp;
    }
}

int strlen(const char *str) {
    int len = 0;
    while (str[len] != '\0') {
        len++;
    }
    return len;
}

void print(const char *str) { syscall(SYSCALL_PRINT, str, strlen(str)); }

void _start() {
    print("Hello from userspace!\n");

    int a = 0, b = 1;
    for (int i = 0; i < 20; i++) {
        char str[32];
        itoa(a, str);
        print(str);
        print("\n");

        int tmp = a;
        a = b;
        b = tmp + b;
    }

    while (1) {
        asm volatile("nop");
    }
}
