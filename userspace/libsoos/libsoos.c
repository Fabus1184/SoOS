#include <stdarg.h>
#include <stdint.h>

#include "libsoos.h"

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

int32_t strcmp(const char *str1, const char *str2) {
    while (*str1 && (*str1 == *str2)) {
        str1++;
        str2++;
    }
    return *(uint8_t *)str1 - *(uint8_t *)str2;
}

void memset(void *ptr, uint8_t value, uint64_t num) {
    uint8_t *byte_ptr = (uint8_t *)ptr;
    for (uint64_t i = 0; i < num; i++) {
        byte_ptr[i] = value;
    }
}

enum SYSCALL {
    SYSCALL_PRINT = 0,
    SYSCALL_SLEEP = 1,
    SYSCALL_EXIT = 2,
    SYSCALL_LISTDIR = 3,
    SYSCALL_READ = 4,
    SYSCALL_FORK = 5,
};

void print(const char *str) {
    uint64_t len = strlen(str);

    asm volatile("mov %[syscall], %%rax\n"
                 "mov %[str], %%rbx\n"
                 "mov %[len], %%rcx\n"
                 "int $0x80\n"
                 :
                 : [syscall] "i"(SYSCALL_PRINT), [str] "m"(str), [len] "m"(len)
                 : "rax", "rbx", "rcx");
}

void sleep(uint64_t ms) {
    asm volatile("mov %[syscall], %%rax\n"
                 "mov %[ms], %%rbx\n"
                 "int $0x80\n"
                 :
                 : [syscall] "i"(SYSCALL_SLEEP), [ms] "m"(ms)
                 : "rax", "rbx");
}

void exit(uint64_t status) {
    asm volatile("mov %[syscall], %%rax\n"
                 "mov %[status], %%rbx\n"
                 "int $0x80\n"
                 :
                 : [syscall] "i"(SYSCALL_EXIT), [status] "m"(status)
                 : "rax", "rbx");
}

uint64_t listdir(const char *path, uint64_t index, char *buffer) {
    uint64_t len = strlen(path);

    uint64_t ret;

    // list directory (path pointer in rbx, path length in rcx, index in rdx, return name in r8)
    // copy name to pointer in r8, return name length in rax

    asm volatile("mov %[syscall], %%rax\n"
                 "mov %[path], %%rbx\n"
                 "mov %[len], %%rcx\n"
                 "mov %[index], %%rdx\n"
                 "mov %[buffer], %%r8\n"
                 "int $0x80\n"
                 : "=rax"(ret)
                 : [syscall] "i"(SYSCALL_LISTDIR), [path] "m"(path), [len] "m"(len), [index] "m"(index), [buffer] "m"(buffer)
                 : "rbx", "rcx", "rdx", "r8");

    return ret;
}

uint64_t read(uint64_t fd, void *buffer, uint64_t size) {
    uint64_t ret;

    // read from file descriptor (fd in rbx, buffer pointer in rcx, size in rdx)
    // return number of bytes read in rax

    asm volatile("mov %[syscall], %%rax\n"
                 "mov %[fd], %%rbx\n"
                 "mov %[buffer], %%rcx\n"
                 "mov %[size], %%rdx\n"
                 "int $0x80\n"
                 : "=rax"(ret)
                 : [syscall] "i"(SYSCALL_READ), [fd] "m"(fd), [buffer] "m"(buffer), [size] "m"(size)
                 : "rbx", "rcx", "rdx");

    return ret;
}

uint32_t fork(void) {
    uint32_t ret;

    // fork process, return child PID in rax

    asm volatile("mov %[syscall], %%rax\n"
                 "int $0x80\n"
                 : "=rax"(ret)
                 : [syscall] "i"(SYSCALL_FORK)
                 :);

    return ret;
}