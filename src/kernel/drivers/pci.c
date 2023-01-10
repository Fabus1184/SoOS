#include "pci.h"
#include "../../lib/io.h"
#include "../../lib/memory.h"

uint32_t pci_config_read32(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset) {
    uint32_t address =
            ((uint32_t) bus << 16) | ((uint32_t) slot << 11) | ((uint32_t) func << 8) | (offset & 0xFC) | 0x80000000U;
    io_write32(address, 0xCF8);
    return io_read32(0xCFC);
}

uint16_t pci_config_read16(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset) {
    return (uint16_t) (pci_config_read32(bus, slot, func, offset) >> ((offset & 2U) * 8U));
}

uint8_t pci_config_read8(uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset) {
    return (uint8_t) (pci_config_read16(bus, slot, func, offset) >> ((offset & 1U) * 8U));
}

union BAR pci_read_bar(const struct pci_device *device, uint8_t bar_index) {
    if ((0 == device->header_type) && (bar_index > 5)) {
        return (union BAR) {.address = 0};
    }

    if ((1 == device->header_type) && (bar_index > 1)) {
        return (union BAR) {.address = 0};
    }

    if ((device->header_type != 0) && (device->header_type != 1)) {
        return (union BAR) {.address = 0};
    }

    union BAR ret;
    const uint32_t *bar = NULL;
    switch (device->header_type) {
        case 0:
            bar = device->header.header_0.bar;
            break;
        case 1:
            bar = device->header.header_1.bar;
            break;
    }

    ret.is_mem = (0 == (bar[0] & 0x1U));

    switch (ret.is_mem) {
        case true:
            /* Memory space */
            ret.BAR_MEM_SPACE.type = (uint8_t) (bar[0] & 0x6U) >> 1U;
            ret.BAR_MEM_SPACE.prefetchable = (0 != (bar[0] & 0x8U));
            switch (ret.BAR_MEM_SPACE.type) {
                case 0:
                    /* 32-bit */
                    ret.address = bar[0] & 0xFFFFFFF0U;
                    break;
                case 1:
                    /* 16-bit */
                    ret.address = bar[0] & 0xFFF0U;
                    break;
                case 2:
                    /* 64-bit */
                    ret.address = ((uint64_t) bar[1] << 32U) + (bar[0] & 0xFFFFFFF0U);
                    break;
            }
            break;
        case false:
            /* IO space */
            ret.BAR_IO_SPACE.reserved = (0 != (bar[0] & 0x2U));
            ret.address = bar[0] & 0xFFFFFFFCU;
            break;
    }
    return ret;
}

void pci_append_device(uint8_t bus, uint8_t slot, uint8_t function, struct pci_device *devices, uint32_t *device_count,
                       uint32_t max_devices) {
    uint16_t vendor_id = pci_config_read16(bus, slot, function, 0x00);

    if (0xFFFF == vendor_id) {
        return;
    }

    if (*device_count >= max_devices) {
        return;
    }

    struct pci_device *device = devices + *device_count;
    device->bus = bus;
    device->slot = slot;
    device->function = function;

    for (uint32_t i = 0; i < 4; ++i) {
        *(3 + i + (uint32_t *) device) = pci_config_read32(bus, slot, function, i * 4);
    }

    switch (device->header_type & 0b11) {
        case 0:
            for (uint32_t i = 0; i < 12; ++i) {
                ((uint32_t *) &device->header.header_0)[i] = pci_config_read32(bus, slot, function, i * 4);
            }
            break;
        case 1:
            for (uint32_t i = 0; i < 12; ++i) {
                ((uint32_t *) &device->header.header_1)[i] = pci_config_read32(bus, slot, function, i * 4);
            }
            break;
        case 2:
            for (uint32_t i = 0; i < 14; ++i) {
                ((uint32_t *) &device->header.header_2)[i] = pci_config_read32(bus, slot, function, i * 4);
            }
            break;
        default:
            break;
    }

    ++(*device_count);

    if (0 != (device->header_type & 0x80U)) {
        for (uint8_t ffunction = 1; ffunction < 8; ++ffunction) {
            pci_append_device(bus, slot, ffunction, devices, device_count, max_devices);
        }
    }
}

uint32_t pci_enumerate_devices(struct pci_device *devices, uint32_t max_devices) {
    uint32_t device_count = 0;
    for (uint8_t bus = 0; bus < 0xFF; ++bus) {
        for (uint8_t slot = 0; slot < 32; ++slot) {
            pci_append_device(bus, slot, 0, devices, &device_count, max_devices);
        }
    }
    return device_count;
}

struct class_code {
    uint8_t class_code;
    const char *class_code_name;
    uint8_t subclass_size;
    struct subclass {
        uint8_t subclass;
        const char *subclass_name;
        uint8_t prog_if_size;
        struct prog_if {
            uint8_t prog_if;
            const char *prog_if_name;
        } prog_ifs[0xFF];
    } subclasses[0XFF];
};

const uint8_t class_code_size = 22;
const struct class_code class_codes[22] = {
        {0x00, "Unclassified",                       2,  {
                                                                 {0x00, "Non-VGA unclassified device",                 0},
                                                                 {0x01, "VGA compatible unclassified device",  0, {}}
                                                         }},
        {0x01, "Mass Storage Controller",            2,  {
                                                                 {0x00, "SCSI Bus Controller",                         0, {}},
                                                                 {0x01, "IDE Controller",                      8,
                                                                                                                  {
                                                                                                                          {0x00, "ISA Compatibility mode-only controller"},
                                                                                                                          {0x05, "PCI native mode-only controller"},
                                                                                                                          {0x0A, "ISA Compatibility mode controller, supports both channels switched to PCI native mode"},
                                                                                                                          {0x0F, "PCI native mode controller, supports both channels switched to ISA compatibility mode"},
                                                                                                                          {0x80, "ISA Compatibility mode-only controller, supports bus mastering"},
                                                                                                                          {0x85, "PCI native mode-only controller, supports bus mastering"},
                                                                                                                          {0x8A, "ISA Compatibility mode controller, supports both channels switched to PCI native mode, supports bus mastering"},
                                                                                                                          {0x8F, "PCI native mode controller, supports both channels switched to ISA compatibility mode, supports bus mastering"}
                                                                                                                  }},
                                                                 {0x02, "Floppy Disk Controller",             0, {}},
                                                                 {0x03, "IPI Bus Controller",           0, {}},
                                                                 {0x04, "RAID Controller",                0, {}},
                                                                 {0x05, "ATA Controller",                2, {
                                                                                                                    {0x20, "Single DMA"},
                                                                                                                    {0x30, "Chained DMA"}
                                                                                                            }},
                                                                 {0x06, "Serial ATA Controller",         3, {
                                                                                                                    {0x0, "Vendor Specific Interface"},
                                                                                                                    {0x01, "AHCI 1.0"},
                                                                                                                    {0x02, "Serial Storage Bus"}
                                                                                                            }},
                                                                 {0x07, "Serial Attached SCSI Controller", 2, {
                                                                                                                      {0x0, "SAS"},
                                                                                                                      {0x1, "Serial Storage Bus"}
                                                                                                              }},
                                                                 {0x08, "Non-Volatile Memory Controller",        2, {
                                                                                                                            {0x1, "NVMHCI"},
                                                                                                                            {0x2, "NVM Express"}
                                                                                                                    }},
                                                                 {0x80, "Other",             0, {}}
                                                         }},
        {0x02, "Network Controller",                 10, {
                                                                 {0x0,  "Ethernet Controller",                         0, {}},
                                                                 {0x1,  "Token Ring Controller",               0, {}},
                                                                 {0x2,  "FDDI Controller",                    0, {}},
                                                                 {0x3,  "ATM Controller",               0, {}},
                                                                 {0x4,  "ISDN Controller",                0, {}},
                                                                 {0x5,  "WorldFlp Controller",           0, {}},
                                                                 {0x6,  "PICMG 2.14 Multi Computing",    0, {}},
                                                                 {0x7,  "Infiniband Controller",           0, {}},
                                                                 {0x8,  "Fabric Controller",                     0, {}},
                                                                 {0x80, "Other",             0, {}}
                                                         }},
        {0x03, "Display Controller",                 4,  {
                                                                 {0x00, "VGA Compatible Controller",                   2, {
                                                                                                                                  {0x0, "VGA Controller"},
                                                                                                                                  {0x1,  "8514-Compatible Controller"}
                                                                                                                          }},
                                                                 {0x01, "XGA Compatible Controller",           0, {}},
                                                                 {0x02, "3D Controller (Non-VGA Compatible)", 0, {}},
                                                                 {0x80, "Other",                        0, {}}
                                                         }},
        {0x4,  "Multimedia Controller",              5,  {
                                                                 {0x0,  "Multimedia Video Contoller",                  0, {}},
                                                                 {0x1,  "Multimedia Audio Controller",         0, {}},
                                                                 {0x2,  "Computer Telephony Device",          0, {}},
                                                                 {0x3,  "Audio Device",                 0, {}},
                                                                 {0x80, "Other",                          0, {}}
                                                         }},
        {0x5,  "Memory Controller",                  3,  {
                                                                 {0x0,  "RAM Controller",                              0, {}},
                                                                 {0x1,  "Flash Controller",                    0, {}},
                                                                 {0x80, "Other",                              0, {}}
                                                         }},
        {0x6,  "Bridge",                             12, {
                                                                 {0x0,  "Host Bridge",                                 0, {}},
                                                                 {0x1,  "ISA Bridge",                          0, {}},
                                                                 {0x2,  "EISA Bridge",                        0, {}},
                                                                 {0x3,  "MCA Bridge",                   0, {}},
                                                                 {0x4,  "PCI-to-PCI Bridge",              2, {
                                                                                                                     {0x0, "Normal Decode"},
                                                                                                                     {0x1, "Subtractive Decode"}
                                                                                                             }},
                                                                 {0x5,  "PCMCIA Bridge",                 0, {}},
                                                                 {0x6,  "NuBus Bridge",                  0, {}},
                                                                 {0x7,  "CardBus Bridge",                  0, {}},
                                                                 {0x8,  "RACEway Bridge",                        2, {
                                                                                                                            {0x0, "Transparent Mode"},
                                                                                                                            {0x1, "Endpoint Mode"}
                                                                                                                    }},
                                                                 {0x9,  "PCI-to-PCI Bridge", 2, {
                                                                                                        {0x40, "Semi-Transparent, Primary bus towards host CPU"},
                                                                                                        {0x80, "Semi-Transparent, Secondary bus towards host CPU"}
                                                                                                }},
                                                                 {0xA,  "InfiniBand to PCI Host Bridge", 0, {}},
                                                                 {0x80, "Other", 0, {}}
                                                         }
        },
        {0x7,  "Simple Communication Controller",    7,  {
                                                                 {0x0,  "Serial Controller",                           7, {
                                                                                                                                  {0x0, "8250-Compatible (Generic XT)"},
                                                                                                                                  {0x1,  "16450-Compatible"},
                                                                                                                                  {0x2, "16550-Compatible"},
                                                                                                                                  {0x3,  "16650-Compatible"},
                                                                                                                                  {0x4,  "16750-Compatible"},
                                                                                                                                  {0x5, "16850-Compatible"},
                                                                                                                                  {0x6, "16950-Compatible"}
                                                                                                                          }},
                                                                 {0x1,  "Parallel Controller",                 5, {
                                                                                                                          {0x0,  "Standard Parallel Port"},
                                                                                                                          {0x1,  "Bidirectional Parallel Port"},
                                                                                                                          {0x2,  "ECP 1.X Compliant Parallel Port"},
                                                                                                                          {0x3,  "IEEE 1284 Controller"},
                                                                                                                          {0xFE, "IEEE 1284 Target Device"}
                                                                                                                  }},
                                                                 {0x2,  "Multiport Serial Controller",        0, {}},
                                                                 {0x3,  "Modem",                        5, {
                                                                                                                   {0x0, "Generic Modem"},
                                                                                                                   {0x1,  "Hayes 16450-Compatible Interface"},
                                                                                                                   {0x2,  "Hayes 16550-Compatible Interface"},
                                                                                                                   {0x3,  "Hayes 16650-Compatible Interface"},
                                                                                                                   {0x4,  "Hayes 16750-Compatible Interface"}
                                                                                                           }},
                                                                 {0x4,  "GPIB (IEEE 488.1/2) Controller", 0, {}},
                                                                 {0x5,  "Smart Card",                    0, {}},
                                                                 {0x80, "Other",                         0, {}}
                                                         }},
        {0x8,  "Base System Peripheral",             8,  {
                                                                 {0x0,  "PIC",                                         5, {
                                                                                                                                  {0x0, "Generic 8259"},
                                                                                                                                  {0x1,  "ISA PIC"},
                                                                                                                                  {0x2, "EISA PIC"},
                                                                                                                                  {0x10, "I/O APIC Interrupt Controller"},
                                                                                                                                  {0x20, "I/O(x) APIC Interrupt Controller"}
                                                                                                                          }},
                                                                 {0x1,  "DMA Controller",                      3, {
                                                                                                                          {0x0,  "Generic 8237"},
                                                                                                                          {0x1,  "ISA DMA Controller"},
                                                                                                                          {0x2,  "EISA DMA Controller"}
                                                                                                                  }},
                                                                 {0x2,  "System Timer",                       4, {
                                                                                                                         {0x0, "Generic 8254"},
                                                                                                                         {0x1, "ISA System Timer"},
                                                                                                                         {0x2, "EISA System Timer"},
                                                                                                                         {0x3, "HPET"}
                                                                                                                 }},
                                                                 {0x3,  "RTC Controller",               2, {
                                                                                                                   {0x0, "Generic RTC"},
                                                                                                                   {0x1,  "ISA-Compatible"}
                                                                                                           }},
                                                                 {0x4,  "PCI Hot-Plug Controller",        0, {}},
                                                                 {0x5,  "SD Host Controller",            0, {}},
                                                                 {0x6,  "IOMMU",                         0, {}},
                                                                 {0x80, "Other",                           0, {}}
                                                         }},
        {0x9,  "Input Device Controller",            5,  {
                                                                 {0x0,  "Keyboard Controller",                         0, {}},
                                                                 {0x1,  "Digitizer Pen",                       0, {}},
                                                                 {0x2,  "Mouse Controller",                   0, {}},
                                                                 {0x3,  "Scanner Controller",           0, {}},
                                                                 {0x4,  "Gameport Controller",            2, {
                                                                                                                     {0x0, "Generic"},
                                                                                                                     {0x1, "Extended"}
                                                                                                             }},
                                                                 {0x80, "Other",                         0, {}}
                                                         }},
        {0xA,  "Docking Station",                    2,  {
                                                                 {0x0,  "Generic",                                     0, {}},
                                                                 {0x80, "Other",                               0, {}}
                                                         }},
        {0xB,  "Processor",                          7,  {
                                                                 {0x0,  "386",                                         0, {}},
                                                                 {0x1,  "486",                                 0, {}},
                                                                 {0x2,  "Pentium",                            0, {}},
                                                                 {0x10, "Alpha",                        0, {}},
                                                                 {0x20, "PowerPC",                        0, {}},
                                                                 {0x30, "MIPS",                          0, {}},
                                                                 {0x40, "Co-Processor",                  0, {}}
                                                         }},
        {0xC,  "Serial Bus Controller",              10, {
                                                                 {0x0,  "FireWire (IEEE 1394)",                        2, {
                                                                                                                                  {0x0, "Generic"},
                                                                                                                                  {0x10, "OHCI"}
                                                                                                                          }},
                                                                 {0x1,  "ACCESS Bus",                          0, {}},
                                                                 {0x2,  "SSA",                                0, {}},
                                                                 {0x3,  "USB Controller",               6, {
                                                                                                                   {0x0, "UHCI"},
                                                                                                                   {0x10, "OHCI"},
                                                                                                                   {0x20, "EHCI"},
                                                                                                                   {0x30, "XHCI"},
                                                                                                                   {0x80, "Unspecified"},
                                                                                                                   {0xFE, "USB Device (Not a host controller)"}
                                                                                                           }},
                                                                 {0x4,  "Fibre Channel",                  0, {}},
                                                                 {0x5,  "SMBus",                         0, {}},
                                                                 {0x6,  "InfiniBand",                    0, {}},
                                                                 {0x7,  "IPMI Interface",                  3, {
                                                                                                                      {0x0, "SMIC"},
                                                                                                                      {0x1, "Keyboard Controller Style"},
                                                                                                                      {0x2, "Block Transfer"}
                                                                                                              }},
                                                                 {0x8,  "SERCOS Interface Standard (IEC 61491)", 0, {}},
                                                                 {0x9,  "CANbus Controller", 0, {}},
                                                                 {0x80, "Other",                         0, {}}
                                                         }},
        {0xD,  "Wireless Controller",                8,  {
                                                                 {0x0,  "iRDA Compatible Controller",                  0, {}},
                                                                 {0x1,  "Consumer IR Controller",              0, {}},
                                                                 {0x10, "RF Controller",                      0, {}},
                                                                 {0x11, "Bluetooth Controller",         0, {}},
                                                                 {0x12, "Broadband Controller",           0, {}},
                                                                 {0x20, "Ethernet Controller (802.11a)", 0, {}},
                                                                 {0x21, "Ethernet Controller (802.11b)", 0, {}},
                                                                 {0x80, "Other",                           0, {}}
                                                         }},
        {0xE,  "Intelligent Controller",             1,  {
                                                                 {0x0,  "I20",                                         0, {}}
                                                         }},
        {0xF,  "Satellite Communication Controller", 4,  {
                                                                 {0x1,  "Satellite TV Controller",                     0, {}},
                                                                 {0x2,  "Satellite Audio Controller",          0, {}},
                                                                 {0x3,  "Satellite Voice Controller",         0, {}},
                                                                 {0x4,  "Satellite Data Controller",    0, {}}
                                                         }},
        {0x10, "Encryption Controller",              3,  {
                                                                 {0x0,  "Network and Computing Encryption/Decryption", 0, {}},
                                                                 {0x10, "Entertainment Encryption/Decryption", 0, {}},
                                                                 {0x80, "Other Encryption/Decryption",        0, {}}
                                                         }},
        {0x11, "Signal Processing Controller",       5,  {
                                                                 {0x0,  "DPIO Modules",                                0, {}},
                                                                 {0x1,  "Performance Counters",                0, {}},
                                                                 {0x10, "Communication Synchronizer",         0, {}},
                                                                 {0x20, "Signal Processing Management", 0, {}},
                                                                 {0x80, "Other",                          0, {}}
                                                         }},
        {0x12, "Processing Accelerator",             3,  {
                                                                 {0x0,  "Processing Accelerator",                      0, {}},
                                                                 {0x1,  "AI Inference Accelerator",            0, {}},
                                                                 {0x10, "Signal Processing Accelerator",      0, {}}
                                                         }},
        {0x13, "Non-Essential Instrumentation",      5,  {
                                                                 {0x0,  "DPIO Modules",                                0, {}},
                                                                 {0x1,  "Performance Counters",                0, {}},
                                                                 {0x10, "Communication Synchronizer",         0, {}},
                                                                 {0x20, "Signal Processing Management", 0, {}},
                                                                 {0x80, "Other",                          0, {}}
                                                         }},
        {0x40, "Coprocessor",                        5,  {
                                                                 {0x0,  "IEEE 754 Floating Point",                     0, {}},
                                                                 {0x1,  "IEEE 754 Arithmetic",                 0, {}},
                                                                 {0x10, "Reserved",                           0, {}},
                                                                 {0x20, "Reserved",                     0, {}},
                                                                 {0x80, "Other",                          0, {}}
                                                         }},
        {0xFF, "Unassigned Class",                   0,  {}}
};

void pci_get_description(const struct pci_device *device, char *class_name, char *subclass_name, char *prog_if_name) {
    memcpy(class_name, "Unknown", 8);
    memcpy(subclass_name, "Unknown", 8);
    memcpy(prog_if_name, "Unknown", 8);

    uint8_t class_index = 0;
    for (; class_index < class_code_size; ++class_index) {
        if (device->class_code == class_codes[class_index].class_code) {
            memcpy(class_name, class_codes[class_index].class_code_name,
                   strlen(class_codes[class_index].class_code_name) + 1);
            break;
        }
    }

    if (class_index == class_code_size) {
        memcpy(class_name, "Unknown", 8);
    }

    uint8_t subclass_index = 0;
    for (; subclass_index < class_codes[class_index].subclass_size; ++subclass_index) {
        if (device->subclass == class_codes[class_index].subclasses[subclass_index].subclass) {
            memcpy(subclass_name, class_codes[class_index].subclasses[subclass_index].subclass_name,
                   strlen(class_codes[class_index].subclasses[subclass_index].subclass_name) + 1);
            break;
        }
    }

    if (subclass_index == class_codes[class_index].subclass_size) {
        memcpy(subclass_name, "Unknown", 8);
    }

    uint8_t prog_if_index = 0;
    for (; prog_if_index < class_codes[class_index].subclasses[subclass_index].prog_if_size; ++prog_if_index) {
        if (device->prog_if == class_codes[class_index].subclasses[subclass_index].prog_ifs[prog_if_index].prog_if) {
            memcpy(prog_if_name,
                   class_codes[class_index].subclasses[subclass_index].prog_ifs[prog_if_index].prog_if_name,
                   strlen(class_codes[class_index].subclasses[subclass_index].prog_ifs[prog_if_index].prog_if_name) +
                   1);
            break;
        }
    }

    if (prog_if_index == class_codes[class_index].subclasses[subclass_index].prog_if_size) {
        memcpy(prog_if_name, "Unknown", 8);
    }
}