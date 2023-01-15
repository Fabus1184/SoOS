#ifndef HOME_FABIAN_GIT_SOOS_SRC_KERNEL_ISR_H
#define HOME_FABIAN_GIT_SOOS_SRC_KERNEL_ISR_H

#include <kernel/drivers/vga_text.h>
#include <kernel/idt.h>
#include <lib/memory.h>
#include <stddef.h>
#include <stdint.h>

enum {
    IRQ0 = 32,
    IRQ1 = 33,
    IRQ2 = 34,
    IRQ3 = 35,
    IRQ4 = 36,
    IRQ5 = 37,
    IRQ6 = 38,
    IRQ7 = 39,
    IRQ8 = 40,
    IRQ9 = 41,
    IRQ10 = 42,
    IRQ11 = 43,
    IRQ12 = 44,
    IRQ13 = 45,
    IRQ14 = 46,
    IRQ15 = 47
};

struct __attribute__((packed)) registers_t {
    uint32_t ds;
    uint32_t edi, esi, ebp, esp, ebx, edx, ecx, eax;
    uint32_t int_no, err_code;
    uint32_t eip, cs, eflags, useresp, ss;
} __attribute__((aligned(64)));

typedef void (*isr_t)(struct registers_t *);

void enable_interrupts(void);

void isr_install(void);

void irq_install(void);

void isr_handler(struct registers_t *r);

void irq_handler(struct registers_t *r);

extern void isr0(void);

extern void isr1(void);

extern void isr2(void);

extern void isr3(void);

extern void isr4(void);

extern void isr5(void);

extern void isr6(void);

extern void isr7(void);

extern void isr8(void);

extern void isr9(void);

extern void isr10(void);

extern void isr11(void);

extern void isr12(void);

extern void isr13(void);

extern void isr14(void);

extern void isr15(void);

extern void isr16(void);

extern void isr17(void);

extern void isr18(void);

extern void isr19(void);

extern void isr20(void);

extern void isr21(void);

extern void isr22(void);

extern void isr23(void);

extern void isr24(void);

extern void isr25(void);

extern void isr26(void);

extern void isr27(void);

extern void isr28(void);

extern void isr29(void);

extern void isr30(void);

extern void isr31(void);

extern void irq0(void);

extern void irq1(void);

extern void irq2(void);

extern void irq3(void);

extern void irq4(void);

extern void irq5(void);

extern void irq6(void);

extern void irq7(void);

extern void irq8(void);

extern void irq9(void);

extern void irq10(void);

extern void irq11(void);

extern void irq12(void);

extern void irq13(void);

extern void irq14(void);

extern void irq15(void);
#endif
