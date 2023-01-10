#ifndef SOOS_PCI_H
#define SOOS_PCI_H

#include <stdint.h>
#include <stddef.h>

#include <lib/io.h>
#include <lib/memory.h>

struct pci_header_0 {
    uint32_t bar0;
    uint32_t bar1;
    uint32_t bar2;
    uint32_t bar3;
    uint32_t bar4;
    uint32_t bar5;
    uint32_t cardbus_cis_pointer;
    uint16_t subsystem_vendor_id;
    uint16_t subsystem_id;
    uint32_t expansion_rom_base_address;
    uint8_t capabilities_pointer;
    uint8_t reserved_low;
    uint16_t reserved_high;
    uint32_t reserved;
    uint8_t interrupt_line;
    uint8_t interrupt_pin;
    uint8_t min_grant;
    uint8_t max_latency;
};

struct pci_header_1 {
    uint32_t bar0;
    uint32_t bar1;
    uint32_t primary_bus_number;
    uint32_t secondary_bus_number;
    uint32_t subordinate_bus_number;
    uint32_t secondary_latency_timer;
    uint32_t io_base;
    uint32_t io_limit;
    uint16_t secondary_status;
    uint16_t memory_base;
    uint16_t memory_limit;
    uint16_t prefetchable_memory_base;
    uint16_t prefetchable_memory_limit;
    uint32_t prefetchable_base_upper_32;
    uint32_t prefetchable_limit_upper_32;
    uint16_t io_base_upper_16;
    uint16_t io_limit_upper_16;
    uint32_t reserved;
    uint32_t expansion_rom_base_address;
    uint8_t interrupt_line;
    uint8_t interrupt_pin;
    uint16_t bridge_control;
};

struct pci_header_2 {
    uint32_t cardbus_socket_register_base_address;
    uint8_t capabilities_pointer;
    uint8_t reserved_low;
    uint16_t secondary_status;
    uint8_t pci_bus_number;
    uint8_t cardbus_bus_number;
    uint8_t subordinate_bus_number;
    uint8_t cardbus_latency_timer;
    uint32_t memory_base_0;
    uint32_t memory_limit_0;
    uint32_t memory_base_1;
    uint32_t memory_limit_1;
    uint32_t io_base_0;
    uint32_t io_limit_0;
    uint32_t io_base_1;
    uint32_t io_limit_1;
    uint8_t interrupt_line;
    uint8_t interrupt_pin;
    uint16_t bridge_control;
    uint16_t subsystem_vendor_id;
    uint16_t subsystem_id;
    uint32_t legacy_base_address;
};

struct pci_device {
    uint32_t bus;
    uint32_t slot;
    uint32_t function;

    uint16_t vendor_id;
    uint16_t device_id;
    uint16_t command;
    uint16_t status;
    uint8_t revision_id;
    uint8_t prog_if;
    uint8_t subclass;
    uint8_t class_code;
    uint8_t cache_line_size;
    uint8_t latency_timer;
    enum {
        GENERAL_DEVICE, PCI_TO_PCI_BRIDGE, CARDBUS_BRIDGE
    } header_type;
    uint8_t bist;

    union {
        struct pci_header_0 header_0;
        struct pci_header_1 header_1;
        struct pci_header_2 header_2;
    } header;
};

uint32_t pci_enumerate_devices(struct pci_device *devices, uint32_t max_devices);

void pci_get_description(const struct pci_device *device, char *class_name, char *subclass_name, char *prog_if_name);

#endif //SOOS_PCI_H
