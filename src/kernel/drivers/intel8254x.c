#include "intel8254x.h"

#define CTRL_RESET 0x4000000

static uint16_t vendor_ids[] = {0x8086};
static uint16_t device_ids[] = {0x1019, 0x101A, 0x1019, 0x1010, 0x1012, 0x101D, 0x1079, 0x107A, 0x107B, 0x100F, 0x1011, 0x1026, 0x1027,
                                0x1028, 0x1107, 0x1112, 0x1013, 0x1018, 0x1076, 0x1077, 0x1078, 0x1017, 0x1016, 0x100E, 0x1015};

void init_device(struct pci_device *device) {
    /* enable bus mastering, memory and IO accesses */
    pci_config_write16(device->bus, device->device_id, device->function, 0x04, device->command | 0b111);

    /* reset */
    io_write32(device->header.header_0.bar[0] + 0x0000, CTRL_RESET);
}

uint32_t init_intel8254x_devices(struct pci_device *devices, uint32_t num_devices) {
    uint32_t n = 0;
    for (uint32_t i = 0; i < num_devices; ++i) {
        if (find(vendor_ids, sizeof(vendor_ids) / sizeof(vendor_ids[0]), sizeof(vendor_ids[0]), &devices[i].vendor_id) &&
            find(device_ids, sizeof(device_ids) / sizeof(device_ids[0]), sizeof(device_ids[0]), &devices[i].device_id)) {
            kprintf("Found Intel 8254x device at %02x:%02x.%x (vendor id: %04x, device id: %04x)\n", devices[i].bus, devices[i].device_id,
                    devices[i].function, devices[i].vendor_id, devices[i].device_id);
            ++n;
            init_device(devices + i);
        }
    }
    return n;
}
