#include "gdt.h"

#define GDT_ENTRIES 3

struct gdt_entry {
    uint32_t base;
    uint32_t limit;
    bool present;
    enum { PRIV_0, PRIV_1, PRIV_2, PRIV_3 } privilege_level;
    enum { TASK_STATE_SEGMENT = 0, CODE_OR_DATA_SEGMENT = 1 } descriptor_type;
    enum { DATA_SEGMENT = 0, CODE_SEGMENT = 1 } executable;
    enum { DATA_DIRECTION_UP = 0, DATA_DIRECTION_DOWN = 1, CODE_CONFORMING_EQ = 0, CODE_CONFORMING_LEQ = 1 } direction_or_conforming;
    enum { DATA_NOT_WRITEABLE = 0, DATA_WRITEABLE = 1, CODE_NOT_READABLE = 0, CODE_READABLE = 1 } read_write;
    bool accessed;
    enum { GRANULARITY_BYTE, GRANULARITY_4KIB } granularity;
    enum { SIZE_16_BIT, SIZE_32_BIT } size;
    bool long_mode;
};

uint64_t gdt[GDT_ENTRIES];

struct __attribute__((packed)) gdt_ptr {
    uint16_t limit;
    uint32_t base;
} gp;

extern void gdt_flush(void);

void gdt_set_gate(int32_t num, struct gdt_entry entry) {
    uint8_t base2 = (entry.base >> 24) & 0xFF;
    uint8_t flags_limit1 = (entry.granularity << 7) | (entry.size << 6) | (entry.long_mode << 5) | (0 << 4) | ((entry.limit >> 16) & 0xFF);
    uint8_t access = (entry.accessed << 7) | (entry.read_write << 6) | (entry.direction_or_conforming << 5) | (entry.executable << 4) |
                     (entry.descriptor_type << 3) | (entry.privilege_level << 1) | (entry.present << 0);
    uint8_t base1 = (entry.base >> 16) & 0xFF;
    uint8_t base0 = (entry.base >> 0) & 0xFF;
    uint8_t limit0 = (entry.limit >> 0) & 0xFF;
    gdt[num] = ((uint64_t) base2 << 56) | ((uint64_t) flags_limit1 << 48) | ((uint64_t) access << 40) | ((uint64_t) base1 << 32) |
               ((uint64_t) base0 << 24) | ((uint64_t) limit0 << 16);
}

void gdt_install(void) {
    gp.limit = (sizeof(uint64_t) * GDT_ENTRIES) - 1;
    gp.base = (uint32_t) &gdt;

    gdt_set_gate(0, (struct gdt_entry){
                        .base = 0,
                        .limit = 0,
                        .present = true,
                        .privilege_level = PRIV_0,
                        .descriptor_type = CODE_OR_DATA_SEGMENT,
                        .executable = CODE_SEGMENT,
                        .direction_or_conforming = CODE_CONFORMING_LEQ,
                        .read_write = CODE_NOT_READABLE,
                        .accessed = false,
                        .granularity = GRANULARITY_BYTE,
                        .size = SIZE_32_BIT,
                        .long_mode = false,
                    });
    gdt_set_gate(1, (struct gdt_entry){
                        .base = 0,
                        .limit = 0xFFFFFFFF,
                        .present = true,
                        .privilege_level = PRIV_0,
                        .descriptor_type = CODE_OR_DATA_SEGMENT,
                        .executable = CODE_SEGMENT,
                        .direction_or_conforming = CODE_CONFORMING_LEQ,
                        .read_write = CODE_READABLE,
                        .accessed = false,
                        .granularity = GRANULARITY_4KIB,
                        .size = SIZE_32_BIT,
                        .long_mode = false,
                    });
    gdt_set_gate(2, (struct gdt_entry){
                        .base = 0,
                        .limit = 0xFFFFFFFF,
                        .present = true,
                        .privilege_level = PRIV_0,
                        .descriptor_type = CODE_OR_DATA_SEGMENT,
                        .executable = DATA_SEGMENT,
                        .direction_or_conforming = DATA_DIRECTION_DOWN,
                        .read_write = DATA_WRITEABLE,
                        .accessed = false,
                        .granularity = GRANULARITY_4KIB,
                        .size = SIZE_32_BIT,
                        .long_mode = false,
                    });
    gdt_flush();
}