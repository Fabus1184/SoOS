#include "isr.h"

void isr_install(void) {
    set_idt_gate(0, &isr0);
    set_idt_gate(1, &isr1);
    set_idt_gate(2, &isr2);
    set_idt_gate(3, &isr3);
    set_idt_gate(4, &isr4);
    set_idt_gate(5, &isr5);
    set_idt_gate(6, &isr6);
    set_idt_gate(7, &isr7);
    set_idt_gate(8, &isr8);
    set_idt_gate(9, &isr9);
    set_idt_gate(10, &isr10);
    set_idt_gate(11, &isr11);
    set_idt_gate(12, &isr12);
    set_idt_gate(13, &isr13);
    set_idt_gate(14, &isr14);
    set_idt_gate(15, &isr15);
    set_idt_gate(16, &isr16);
    set_idt_gate(17, &isr17);
    set_idt_gate(18, &isr18);
    set_idt_gate(19, &isr19);
    set_idt_gate(20, &isr20);
    set_idt_gate(21, &isr21);
    set_idt_gate(22, &isr22);
    set_idt_gate(23, &isr23);
    set_idt_gate(24, &isr24);
    set_idt_gate(25, &isr25);
    set_idt_gate(26, &isr26);
    set_idt_gate(27, &isr27);
    set_idt_gate(28, &isr28);
    set_idt_gate(29, &isr29);
    set_idt_gate(30, &isr30);
    set_idt_gate(31, &isr31);

    /* Remap the PIC */
    io_out(0x11, 0x20);
    io_out(0x11, 0xA0);
    io_out(0x20, 0x21);
    io_out(0x28, 0xA1);
    io_out(0x04, 0x21);
    io_out(0x02, 0xA1);
    io_out(0x01, 0x21);
    io_out(0x01, 0xA1);
    io_out(0x0, 0x21);
    io_out(0x0, 0xA1);

    /* Install the IRQs */
    set_idt_gate(32, &irq0);
    set_idt_gate(33, &irq1);
    set_idt_gate(34, &irq2);
    set_idt_gate(35, &irq3);
    set_idt_gate(36, &irq4);
    set_idt_gate(37, &irq5);
    set_idt_gate(38, &irq6);
    set_idt_gate(39, &irq7);
    set_idt_gate(40, &irq8);
    set_idt_gate(41, &irq9);
    set_idt_gate(42, &irq10);
    set_idt_gate(43, &irq11);
    set_idt_gate(44, &irq12);
    set_idt_gate(45, &irq13);
    set_idt_gate(46, &irq14);
    set_idt_gate(47, &irq15);

    set_idt(); /* Load with ASM */
}

void irq_install(void) {
    init_timer(50);
    init_keyboard();
    asm volatile("sti");
}

/* To print the message which defines every exception */
char *exception_messages[32] = {
        "Division By Zero",
        "Debug",
        "Non Maskable Interrupt",
        "Breakpoint",
        "Into Detected Overflow",
        "Out of Bounds",
        "Invalid Opcode",
        "No Coprocessor",

        "Double Fault",
        "Coprocessor Segment Overrun",
        "Bad TSS",
        "Segment Not Present",
        "Stack Fault",
        "General Protection Fault",
        "Page Fault",
        "Unknown Interrupt",

        "Coprocessor Fault",
        "Alignment Check",
        "Machine Check",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",

        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved",
        "Reserved"
};

void isr_handler(const registers_t *r) {
    print("received interrupt: ");
    char s[3];
    itoa((int32_t) r->int_no, s);
    print(s);
    print(" (");
    print(exception_messages[r->int_no]);
    println(")");
    asm volatile("hlt");
}

void register_interrupt_handler(uint8_t n, isr_t handler) {
    interrupt_handlers[n] = handler;
}

void irq_handler(registers_t *r) {
    if (r->int_no >= 40) {
        io_out(0x20, 0xA0);
    }
    io_out(0x20, 0x20);

    if (interrupt_handlers[r->int_no] != NULL) {
        isr_t handler = interrupt_handlers[r->int_no];
        handler(r);
    }
}
