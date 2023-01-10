#ifndef __i386__
#error "This needs to be compiled with a ix86-elf compiler"
#endif

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include <kernel/drivers/vbe.h>

/* Hardware text mode color constants. */
enum vga_color {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_BLUE = 1,
    VGA_COLOR_GREEN = 2,
    VGA_COLOR_CYAN = 3,
    VGA_COLOR_RED = 4,
    VGA_COLOR_MAGENTA = 5,
    VGA_COLOR_BROWN = 6,
    VGA_COLOR_LIGHT_GREY = 7,
    VGA_COLOR_DARK_GREY = 8,
    VGA_COLOR_LIGHT_BLUE = 9,
    VGA_COLOR_LIGHT_GREEN = 10,
    VGA_COLOR_LIGHT_CYAN = 11,
    VGA_COLOR_LIGHT_RED = 12,
    VGA_COLOR_LIGHT_MAGENTA = 13,
    VGA_COLOR_LIGHT_BROWN = 14,
    VGA_COLOR_WHITE = 15,
};

static inline uint8_t vga_entry_color(enum vga_color fg, enum vga_color bg) {
    return fg | bg << 4;
}

static inline uint16_t vga_entry(unsigned char uc, uint8_t color) {
    return (uint16_t) uc | (uint16_t) color << 8;
}

static const size_t VGA_WIDTH = 80;
static const size_t VGA_HEIGHT = 25;

size_t terminal_row;
size_t terminal_column;
uint8_t terminal_color;
uint16_t *terminal_buffer;

void terminal_initialize(void) {
    terminal_row = 0;
    terminal_column = 0;
    terminal_color = vga_entry_color(VGA_COLOR_LIGHT_GREY, VGA_COLOR_BLACK);
    terminal_buffer = (uint16_t *) 0xB8000;
    for (size_t y = 0; y < VGA_HEIGHT; y++) {
        for (size_t x = 0; x < VGA_WIDTH; x++) {
            const size_t index = y * VGA_WIDTH + x;
            terminal_buffer[index] = vga_entry(' ', terminal_color);
        }
    }
}

void terminal_setcolor(uint8_t color) {
    terminal_color = color;
}

void terminal_putentryat(char c, uint8_t color, size_t x, size_t y) {
    const size_t index = y * VGA_WIDTH + x;
    terminal_buffer[index] = vga_entry(c, color);
}

void terminal_putchar(char c) {
    terminal_putentryat(c, terminal_color, terminal_column, terminal_row);
    if (++terminal_column == VGA_WIDTH) {
        terminal_column = 0;
        if (++terminal_row == VGA_HEIGHT)
            terminal_row = 0;
    }
}

void terminal_write(const char *data, size_t size) {
    for (size_t i = 0; i < size; i++)
        terminal_putchar(data[i]);
}

void terminal_writestring(const char *data) {
    terminal_write(data, strlen(data));
}

void reverse(char *str, size_t len) {
    int32_t i = 0, j = len - 1, temp;
    while (i < j) {
        temp = str[i];
        str[i] = str[j];
        str[j] = temp;
        ++i;
        --j;
    }
}

char *itoa(size_t n, char *s) {
    size_t i = 0;
    do {
        s[i++] = '0' + (int8_t) (n % 10);
        n /= 10;
    } while (n > 0);
    s[i] = '\0';
    reverse(s, i);
    return s;
}

void print_pci_device(const struct pci_device *device) {
    char buffer[100];
    terminal_writestring("PCI: Bus ");
    terminal_writestring(itoa(device->bus, buffer));
    terminal_writestring(",Dev ");
    terminal_writestring(itoa(device->slot, buffer));
    terminal_writestring(",Func ");
    terminal_writestring(itoa(device->function, buffer));
    terminal_writestring(",vID ");
    terminal_writestring(itoa(device->vendor_id, buffer));
    terminal_writestring(",dID ");
    terminal_writestring(itoa(device->device_id, buffer));
    terminal_writestring(",CC ");
    terminal_writestring(itoa(device->class_code, buffer));
    terminal_writestring(",SC ");
    terminal_writestring(itoa(device->subclass, buffer));
    terminal_writestring(",PI ");
    terminal_writestring(itoa(device->prog_if, buffer));
    terminal_writestring(",HT ");
    terminal_writestring(itoa(device->header_type, buffer));
    terminal_writestring(",LT ");
    terminal_writestring(itoa(device->latency_timer, buffer));
    terminal_writestring(",CLS ");
    terminal_writestring(itoa(device->cache_line_size, buffer));

    terminal_column = 0;
    ++terminal_row;

    char a[100], b[100], c[100];
    pci_get_description(device, a, b, c);
    terminal_writestring(a);
    terminal_writestring(" - ");
    terminal_writestring(b);
    terminal_writestring(" - ");
    terminal_writestring(c);

    terminal_column = 0;
    ++terminal_row;
}

void kernel_main(void) {
    struct pci_device devices[64];

    /* Initialize terminal interface */
    terminal_initialize();

    /* Newline support is left as an exercise. */
    terminal_write("Hello, kernel World!", 20);
    ++terminal_row;
    terminal_column = 0;

    uint32_t device_count = pci_enumerate_devices(devices, 64);

    char buffer[20];
    memset(buffer, 0, 10);
    itoa(device_count, buffer);
    terminal_writestring(buffer);
    ++terminal_row;
    terminal_column = 0;

    for (uint32_t d = 0; d < device_count; ++d) {
        print_pci_device(devices + d);
    }

    while (1) {
    }

    vbe_init(800, 600, 24);

    for (uint16_t y = 0; y < 800; ++y) {
        for (uint16_t x = 0; x < 600; ++x) {
            vbe_pixel(x, y, 0xFF00FF);
        }
    }
}
