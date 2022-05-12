#pragma once

#include <cstdint>

#include "idt.hpp"
#include "keyboard.hpp"
#include "timer.hpp"

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

extern "C" void isr_handler(registers_t *r);

extern "C" void irq_handler(registers_t *r);

typedef void (*isr_t)(registers_t *);

isr_t interrupt_handlers[256];

void register_interrupt_handler(uint8_t n, isr_t handler);

// cpu exceptions
extern "C" void isr0();

extern "C" void isr1();

extern "C" void isr2();

extern "C" void isr3();

extern "C" void isr4();

extern "C" void isr5();

extern "C" void isr6();

extern "C" void isr7();

extern "C" void isr8();

extern "C" void isr9();

extern "C" void isr10();

extern "C" void isr11();

extern "C" void isr12();

extern "C" void isr13();

extern "C" void isr14();

extern "C" void isr15();

extern "C" void isr16();

extern "C" void isr17();

extern "C" void isr18();

extern "C" void isr19();

extern "C" void isr20();

extern "C" void isr21();

extern "C" void isr22();

extern "C" void isr23();

extern "C" void isr24();

extern "C" void isr25();

extern "C" void isr26();

extern "C" void isr27();

extern "C" void isr28();

extern "C" void isr29();

extern "C" void isr30();

extern "C" void isr31();

/* IRQ definitions */
extern "C" void irq0();

extern "C" void irq1();

extern "C" void irq2();

extern "C" void irq3();

extern "C" void irq4();

extern "C" void irq5();

extern "C" void irq6();

extern "C" void irq7();

extern "C" void irq8();

extern "C" void irq9();

extern "C" void irq10();

extern "C" void irq11();

extern "C" void irq12();

extern "C" void irq13();

extern "C" void irq14();

extern "C" void irq15();

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
