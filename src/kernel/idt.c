#include "idt.h"

#define IDT_ENTRIES 256

struct __attribute__((packed)) idt_gate {
    uint16_t low_offset;
    uint16_t sel;
    uint8_t zero;
    uint8_t flags;
    uint16_t high_offset;
};

struct __attribute__((packed)) idt_register {
    uint16_t size;
    uint32_t offset;
};

struct idt_gate idt[IDT_ENTRIES];
struct idt_register idt_reg;

void set_idt_gate(uint32_t n, uint64_t handler) {
    idt[n] = (struct idt_gate){
        .low_offset = handler & 0xFFFF,
        .sel = (0x1 << 3) | 0b000,
        .zero = 0,
        .flags = 0x8E,
        .high_offset = (handler >> 16) & 0xFFFF,
    };
}

void set_idt(void) {
    idt_reg.offset = (uint64_t) &idt;
    idt_reg.size = (IDT_ENTRIES * sizeof(struct idt_gate)) - 1;
    // asm volatile("lidtl (%0)" : : "r"(&idt_reg));
}