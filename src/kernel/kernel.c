#include <kernel/drivers/intel8254x.h>
#include <kernel/drivers/pci.h>
#include <kernel/drivers/vga_text.h>
#include <kernel/gdt.h>
#include <kernel/isr.h>
#include <kernel/rtc.h>
#include <stb/stb_sprintf.h>
#include <stddef.h>
#include <stdint.h>

void print_pci_device(const struct pci_device *dev) {
    const char *a, *b;
    pci_get_description(dev, &a, &b, NULL);
    kprintf("%02x:%02x.%x: %s - %s (%04x)\n", dev->bus, dev->device_id, dev->function, a, b, dev->vendor_id);
}

struct __attribute__((packed)) memory_region {
    uint64_t base;
    uint64_t length;
    /* TODO: very bad style: the size of enum type is defined by it's maximum value because of attribute packed */
    enum { AVAILABLE = 1, RESERVED = 2, ACPI_RECLAIMABLE = 3, ACPI_NVS = 4, BAD_MEMORY = 5, _MAX = (1 << 31) } type;
    uint32_t acpi_3_0_extended_attributes;
};

const char *memory_type_names[6] = {
    "?", "Available", "Reserved", "ACPI reclaimable", "ACPI NVS", "Bad memory",
};

extern uint16_t low_memory;
extern struct memory_region high_memory[];
extern uint16_t high_memory_size;

void print_memory_map(void) {
    kprintf("Detected %u KB of low memory\n", low_memory);
    uint8_t region_count = high_memory_size / 24;
    kprintf("Detected %d regions of high memory\n", region_count);
    for (uint8_t i = 0; i < region_count; ++i) {
        char *prefix = "";
        uint64_t length = prefix_decimal(high_memory[i].length, &prefix);
        kprintf("Region %2u: base: %#15llx, length: %3lld%s, type: %s\n", i, high_memory[i].base, length, prefix,
                memory_type_names[high_memory[i].type]);
    }
}

void __attribute__((noreturn)) kernel_main(void) {
    gdt_install();

    init_text_mode();

    struct rtc_time time = get_rtc_time();
    kprintf("Hello, world! @ %02u.%02u.%04u %02u:%02u:%02u\n", time.day_of_month, time.month, time.year, time.hours, time.minutes, time.seconds);

    print_memory_map();

    // isr_install();
    // enable_interrupts();

    struct pci_device devices[64];
    uint32_t device_count = pci_enumerate_devices(devices, 64);

    kprintf("Found %u PCI devices\n", device_count);
    const struct pci_device *vga_controller = NULL;
    for (uint32_t i = 0; i < device_count; ++i) {
        print_pci_device(devices + i);
        if ((0x1234 == devices[i].vendor_id) && (0x1111 == devices[i].device_id)) {
            vga_controller = devices + i;
        }
    }

    init_intel8254x_devices(devices, device_count);

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
