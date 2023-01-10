#include "kernel.h"

static char welcome[17][TEXT_COLS + 1] = {
        R"(+------------------------------------------------------------------------------+)",
        R"(|                _    _      _                            _                    |)",
        R"(|               | |  | |    | |                          | |                   |)",
        R"(|               | |  | | ___| | ___ ___  _ __ ___   ___  | |_ ___              |)",
        R"(|               | |/\| |/ _ \ |/ __/ _ \| '_ ` _ \ / _ \ | __/ _ \             |)",
        R"(|               \  /\  /  __/ | (_| (_) | | | | | |  __/ | || (_) |            |)",
        R"(|                \/  \/ \___|_|\___\___/|_| |_| |_|\___|  \__\___/             |)",
        R"(|                                                                              |)",
        R"(|                                                                              |)",
        R"(|                          _____       _____ _____                             |)",
        R"(|                         /  ___|     |  _  /  ___|                            |)",
        R"(|                         \ `--.  ___ | | | \ `--.                             |)",
        R"(|                          `--. \/ _ \| | | |`--. \                            |)",
        R"(|                         /\__/ / (_) \ \_/ /\__/ /                            |)",
        R"(|                         \____/ \___/ \___/\____/                             |)",
        R"(|                                                                              |)",
        R"(+------------------------------------------------------------------------------+)"
};

char *kernel_log = NULL;
size_t kernel_log_size = 0;
size_t kernel_log_capacity = 0;

_Noreturn void kmain(void) {
    isr_install();
    irq_install();

    kernel_log = kmalloc(4096);
    kernel_log_capacity = 4096;

    KERNEL_INFO("Welcome to SoOS\n");

    switch_vga_mode();

    for (uint16_t y = 0; y < 100; ++y) {
        for (uint16_t x = 0; x < 100; ++x) {
            put_pixel(x, y, (uint8_t) (x * y));
        }
    }

    /*clear_screen();

    for (size_t i = 0; i < (sizeof(welcome) / sizeof(*welcome)); ++i) {
        println(welcome[i]);
    }
    println("");*/

    while (1) {
        asm volatile("nop");
    }
}

void kmemcpy(void *dest, void *src, size_t n) {
    for (size_t i = 0; i < n; ++i) {
        ((char *) dest)[i] = ((char *) src)[i];
    }
}

size_t kprintf(const char *str, ...) {
    va_list args;
    va_start(args, str);
    // int32_t ret = stbsp_vsprintf(NULL, str, args);

    /* if (ret <= 0) {
        return ret;
    } */

    size_t length = strlen(str) + 1;

    if (length > (kernel_log_capacity - kernel_log_size)) {
        kernel_log = krealloc(kernel_log, kernel_log_capacity + length);
        kernel_log_capacity += length;
    }

    memcpy(kernel_log + kernel_log_size, str, length);
    kernel_log_size += length;

    va_end(args);

    return length;
}

void kfree(__attribute__((unused)) void *ptr) {
}

void *krealloc(void *ptr, size_t size) {
    void *new_ptr = kmalloc(size);
    kmemcpy(new_ptr, ptr, size);
    kfree(ptr);
    return new_ptr;
}

void *kmalloc(size_t size) {
    static uint32_t address = 0x00100000;
    void *ptr = (void *) address;
    address += size;
    return ptr;
}
