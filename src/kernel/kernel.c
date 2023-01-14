#include <kernel/drivers/pci.h>
#include <kernel/drivers/vga_text.h>
#include <kernel/gdt.h>
#include <kernel/isr.h>
#include <stb/stb_sprintf.h>
#include <stddef.h>
#include <stdint.h>

void print_pci_device(const struct pci_device *dev) {
    const char *a, *b;
    pci_get_description(dev, &a, &b, NULL);
    kprintf("%02x:%02x.%x: %s - %s (%04x)\n", dev->bus, dev->device_id, dev->function, a, b, dev->vendor_id);
}

void __attribute__((noreturn)) kernel_main(void) {
    gdt_install();

    init_text_mode();
    kprintf("Hello, world!\n");

    isr_install();
    enable_interrupts();

    struct pci_device devices[64];
    uint32_t device_count = pci_enumerate_devices(devices, 64);

    const struct pci_device *vga_controller = NULL;
    for (uint32_t i = 0; i < device_count; ++i) {
        print_pci_device(devices + i);
        if ((0x1234 == devices[i].vendor_id) && (0x1111 == devices[i].device_id)) {
            vga_controller = devices + i;
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
