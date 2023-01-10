#include "idt.h"

void set_idt_gate(uint32_t n, void *handler) {
    idt[n].low_offset = (uint64_t) handler & 0xFFFF;
    idt[n].sel = KERNEL_CS;
    idt[n].always0 = 0;
    idt[n].flags = 0x8E;
    idt[n].high_offset = ((uint64_t) handler >> 16) & 0xFFFF;
}

void set_idt(void) {
    idt_reg.base = (uint32_t) &idt;
    idt_reg.limit = (IDT_ENTRIES * sizeof(idt_gate_t)) - 1;
    /* Don't make the mistake of loading &idt -- always load &idt_reg */
    asm volatile("lidtl (%0)" : : "r" (&idt_reg));
}
