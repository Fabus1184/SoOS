#ifndef SOOS_INTEL8254X_H
#define SOOS_INTEL8254X_H

#include <kernel/drivers/pci.h>
#include <kernel/drivers/vga_text.h>
#include <lib/io.h>
#include <stddef.h>
#include <stdint.h>

uint32_t init_intel8254x_devices(struct pci_device *devices, uint32_t num_devices);

#endif  // SOOS_INTEL8254X_H