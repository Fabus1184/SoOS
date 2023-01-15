#include "gdt.h"

#define GDT_ENTRIES 4

struct __attribute__((packed)) gdt_entry {
    uint16_t limit_low;
    uint16_t base_low;
    uint8_t base_middle;
    uint8_t access;
    uint8_t granularity;
    uint8_t base_high;
} gdt[GDT_ENTRIES];

struct __attribute__((packed)) gdt_ptr {
    uint16_t limit;
    uint32_t base;
} gp;

extern void gdt_flush(void);

void gdt_set_gate(int32_t num, uint64_t base, uint64_t limit, uint8_t access, uint8_t gran) {
    gdt[num] = (struct gdt_entry){
        .base_low = (base & 0xFFFF),
        .base_middle = (base >> 16) & 0xFF,
        .base_high = (base >> 24) & 0xFF,
        .limit_low = (limit & 0xFFFF),
        .granularity = ((limit >> 16) & 0x0F) | (gran & 0xF0),
        .access = access,
    };
}

void gdt_install(void) {
    gp.limit = (sizeof(struct gdt_entry) * GDT_ENTRIES) - 1;
    gp.base = (uint64_t) &gdt;

    gdt_set_gate(0, 0, 0, 0, 0);
    gdt_set_gate(1, 0, 0xFFFFFFFF, 0b10011010, 0xCF);
    gdt_set_gate(2, 0, 0xFFFFFFFF, 0x92, 0xCF);
    gdt_set_gate(3, 0, 0xFFFFFFFF, 0xFA, 0xCF);

    gdt_flush();
}