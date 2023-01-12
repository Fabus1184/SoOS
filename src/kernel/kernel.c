#include <kernel/drivers/pci.h>
#include <kernel/drivers/vga.h>
#include <stb/stb_sprintf.h>
#include <stddef.h>
#include <stdint.h>

uint8_t line = 0;
uint8_t col = 0;
uint16_t *fb = NULL;

void kputchar(char c) {
    if (90 == col) {
        col = 0;
        ++line;

        if (60 == line) {
            line = 59;
            memmove(fb, fb + 90, (90 * 59 * 2));
            memset(fb + (90 * 59), 0, 90 * 2);
        }
    }

    if ('\n' == c) {
        ++line;
        col = 0;
    } else {
        fb[line * 90 + col] = (0x0F << 8) | c;
        ++col;
    }

    set_cursor_pos(col, line);
}

void kprintf(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);

    uint32_t size = stbsp_vsnprintf(NULL, 0, fmt, args);
    char buf[size + 1];
    stbsp_vsprintf(buf, fmt, args);

    for (uint32_t i = 0; i < size; ++i) {
        kputchar(buf[i]);
    }

    va_end(args);
}

void print_pci_device(const struct pci_device *dev) {
    char *a, *b, *c;
    pci_get_description(dev, &a, &b, NULL);
    kprintf("%02x:%02x.%x: %s - %s (%04x)\n", dev->bus, dev->device_id, dev->function, a, b, dev->vendor_id);
}

_Noreturn void kernel_main(void) {
    write_regs(g_90x30_text);
    write_font(g_8x16_font, 16);
    fb = get_fb_seg() * 16;

    struct pci_device devices[64];
    uint32_t device_count = pci_enumerate_devices(devices, 64);

    const struct pci_device *vga_controller = NULL;
    for (uint32_t i = 0; i < device_count; ++i) {
        print_pci_device(devices + i);
        if ((0x1234 == devices[i].vendor_id) && (0x1111 == devices[i].device_id)) {
            vga_controller = devices + i;
            break;
        }
    }

    if (NULL == vga_controller) {
        kprintf("ERROR: No VGA controller found!\n");
        asm volatile("hlt");
        while (1) { /* */
        }
    }

    asm volatile("hlt");
    while (1) { /* */
    }
}
