#pragma once

#include <stdint.h>
#include <stddef.h>
#include <stddef.h>
#include <stdarg.h>

#include <stb/stb_sprintf.h>

#include "../interrupts/isr.h"
#include "drivers/vga.h"

#define KERNEL_INFO(msg, ...) kprintf("[INFO] " msg, ##__VA_ARGS__)
#define KERNEL_WARN(msg, ...) kprintf("[WARN] " msg, ##__VA_ARGS__)
#define KERNEL_ERROR(msg, ...) kprintf("[ERROR] " msg, ##__VA_ARGS__)

_Noreturn void kmain(void);

void kmemcpy(void *dest, void *src, size_t n);

size_t kprintf(const char *str, ...);

void *kmalloc(size_t size);

void *krealloc(void *ptr, size_t size);

void kfree(void *ptr);