#include "isr.h"

#define PIC1 0x20 /* IO base address for master PIC */
#define PIC2 0xA0 /* IO base address for slave PIC */
#define PIC1_DATA (PIC1 + 1)
#define PIC2_DATA (PIC2 + 1)
#define PIC_EOI 0x20 /* End-of-interrupt command code */

#define ICW1_ICW4 0x01      /* ICW4 (not) needed */
#define ICW1_SINGLE 0x02    /* Single (cascade) mode */
#define ICW1_INTERVAL4 0x04 /* Call address interval 4 (8) */
#define ICW1_LEVEL 0x08     /* Level triggered (edge) mode */
#define ICW1_INIT 0x10      /* Initialization - required! */

#define ICW4_8086 0x01       /* 8086/88 (MCS-80/85) mode */
#define ICW4_AUTO 0x02       /* Auto (normal) EOI */
#define ICW4_BUF_SLAVE 0x08  /* Buffered mode/slave */
#define ICW4_BUF_MASTER 0x0C /* Buffered mode/master */
#define ICW4_SFNM 0x10       /* Special fully nested (not) */

/* clang-format off */
const void *isrs[] = {
    isr0,  "Divide by zero",
    isr1,  "Debug",
    isr2,  "Non maskable interrupt",
    isr3,  "Breakpoint",
    isr4,  "Into detected overflow",
    isr5,  "Out of bounds",
    isr6,  "Invalid opcode",
    isr7,  "No coprocessor",
    isr8,  "Double fault",
    isr9,  "Coprocessor segment overrun",
    isr10, "Bad TSS",
    isr11, "Segment not present",
    isr12, "Stack fault",
    isr13, "General protection fault",
    isr14, "Page fault",
    isr15, "Unknown interrupt",
    isr16, "Coprocessor fault",
    isr17, "Alignment check",
    isr18, "Machine check",
    isr19, "Reserved",
    isr20, "Reserved",
    isr21, "Reserved",
    isr22, "Reserved",
    isr23, "Reserved",
    isr24, "Reserved",
    isr25, "Reserved",
    isr26, "Reserved",
    isr27, "Reserved",
    isr28, "Reserved",
    isr29, "Reserved",
    isr30, "Reserved",
    isr31, "Reserved",
    irq0,  "System Timer",
    irq1,  "Keyboard Controller",
    irq2,  "Cascade (used internally by the two PICs. never raised)",
    irq3,  "Serial Port 1",
    irq4,  "Serial Port 2",
    irq5,  "Parallel Port 2 & 3 / Sound Card",
    irq6,  "Floppy Disk",
    irq7,  "Parallel Port 1 / Printer / Secondary Sound Card",
    irq8,  "Real Time Clock",
    irq9,  "Intel ACPI / Other",
    irq10, "Any PCI / SCSI / NIC",
    irq11, "Any PCI / SCSI / NIC",
    irq12, "PS/2 Mouse",
    irq13, "FPU / Coprocessor / Inter-processor",
    irq14, "Primary ATA",
    irq15, "Secondary ATA",
};
/* clang-format on */

isr_t interrupt_handlers[256];

void enable_interrupts(void) { asm volatile("sti"); }

#define ISR(n) isr##n

void isr_install() {
    for (uint8_t i = 0; i < 32; ++i) {
        set_idt_gate(i, (uint32_t) isrs[i * 2]);
    }

    uint8_t mask1 = io_read8(PIC1_DATA);
    uint8_t mask2 = io_read8(PIC2_DATA);

    io_write8(ICW1_INIT | ICW1_ICW4, PIC1);
    io_write8(ICW1_INIT | ICW1_ICW4, PIC2);
    io_write8(0x20, PIC1_DATA);
    io_write8(0x28, PIC2_DATA);
    io_write8(4, PIC1_DATA);
    io_write8(2, PIC2_DATA);
    io_write8(ICW4_8086, PIC1_DATA);
    io_write8(ICW4_8086, PIC2_DATA);

    io_write8(mask1, PIC1_DATA);
    io_write8(mask2, PIC2_DATA);

    set_idt_gate(32, (uint32_t) irq0);
    set_idt_gate(33, (uint32_t) irq1);
    set_idt_gate(34, (uint32_t) irq2);
    set_idt_gate(35, (uint32_t) irq3);
    set_idt_gate(36, (uint32_t) irq4);
    set_idt_gate(37, (uint32_t) irq5);
    set_idt_gate(38, (uint32_t) irq6);
    set_idt_gate(39, (uint32_t) irq7);
    set_idt_gate(40, (uint32_t) irq8);
    set_idt_gate(41, (uint32_t) irq9);
    set_idt_gate(42, (uint32_t) irq10);
    set_idt_gate(43, (uint32_t) irq11);
    set_idt_gate(44, (uint32_t) irq12);
    set_idt_gate(45, (uint32_t) irq13);
    set_idt_gate(46, (uint32_t) irq14);
    set_idt_gate(47, (uint32_t) irq15);

    set_idt();

    memset(&interrupt_handlers, 0, sizeof(isr_t) * 256);
}

void irq_install() {}

void isr_handler(struct registers_t *r) {
    init_text_mode();
    kprintf_color(COLOR16_RED, COLOR16_BLACK, "REALLY BIG OOPSIE: Received interrupt: %#04x (%s)\n", r->int_no, isrs[r->int_no]);
    kprintf_color(COLOR16_RED, COLOR16_BLACK, "error code: %d\n", r->err_code);
    asm volatile("hlt");
}

void register_interrupt_handler(uint8_t n, isr_t handler) { interrupt_handlers[n] = handler; }

void irq_handler(struct registers_t *r) {
    /* kprintf("Received IRQ%d: %s\n", r->int_no - IRQ0, isrs[(r->int_no * 2) + 1]); */

    if (interrupt_handlers[r->int_no] != NULL) {
        isr_t handler = interrupt_handlers[r->int_no];
        handler(r);
    }

    if (r->int_no >= 40) {
        io_write8(PIC_EOI, PIC2);
    }
    io_write8(PIC_EOI, PIC1);
}