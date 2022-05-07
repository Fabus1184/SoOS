#pragma once

#include <stdint.h>

#include "print.h"
#include "idt.h"
#include "keyboard.h"
#include "timer.h"

/* Struct which aggregates many registers */
typedef struct
{
	uint32_t ds; /* Data segment selector */
	uint32_t edi, esi, ebp, esp, ebx, edx, ecx, eax; /* Pushed by pusha. */
	uint32_t int_no, err_code; /* Interrupt number and error code (if applicable) */
	uint32_t eip, cs, eflags, useresp, ss; /* Pushed by the processor automatically */
} registers_t;

void isr_install();

void irq_install();

void isr_handler(registers_t *r);

typedef void (*isr_t)(registers_t *);

isr_t interrupt_handlers[256];

__attribute__((unused)) void register_interrupt_handler(uint8_t n, isr_t handler);

// cpu exceptions
__attribute__((unused)) extern void isr0();

__attribute__((unused)) extern void isr1();

__attribute__((unused)) extern void isr2();

__attribute__((unused)) extern void isr3();

__attribute__((unused)) extern void isr4();

__attribute__((unused)) extern void isr5();

__attribute__((unused)) extern void isr6();

__attribute__((unused)) extern void isr7();

__attribute__((unused)) extern void isr8();

__attribute__((unused)) extern void isr9();

__attribute__((unused)) extern void isr10();

__attribute__((unused)) extern void isr11();

__attribute__((unused)) extern void isr12();

__attribute__((unused)) extern void isr13();

__attribute__((unused)) extern void isr14();

__attribute__((unused)) extern void isr15();

__attribute__((unused)) extern void isr16();

__attribute__((unused)) extern void isr17();

__attribute__((unused)) extern void isr18();

__attribute__((unused)) extern void isr19();

__attribute__((unused)) extern void isr20();

__attribute__((unused)) extern void isr21();

__attribute__((unused)) extern void isr22();

__attribute__((unused)) extern void isr23();

__attribute__((unused)) extern void isr24();

__attribute__((unused)) extern void isr25();

__attribute__((unused)) extern void isr26();

__attribute__((unused)) extern void isr27();

__attribute__((unused)) extern void isr28();

__attribute__((unused)) extern void isr29();

__attribute__((unused)) extern void isr30();

__attribute__((unused)) extern void isr31();

/* IRQ definitions */
extern void irq0();

extern void irq1();

extern void irq2();

extern void irq3();

extern void irq4();

extern void irq5();

extern void irq6();

extern void irq7();

extern void irq8();

extern void irq9();

extern void irq10();

extern void irq11();

extern void irq12();

extern void irq13();

extern void irq14();

extern void irq15();

#define IRQ0 32
#define IRQ1 33
#define IRQ2 34
#define IRQ3 35
#define IRQ4 36
#define IRQ5 37
#define IRQ6 38
#define IRQ7 39
#define IRQ8 40
#define IRQ9 41
#define IRQ10 42
#define IRQ11 43
#define IRQ12 44
#define IRQ13 45
#define IRQ14 46
#define IRQ15 47
