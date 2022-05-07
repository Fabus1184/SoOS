#pragma once

#include <stdint.h>

#include "print.h"
#include "idt.h"

/* Struct which aggregates many registers */
typedef struct
{
	uint32_t ds; /* Data segment selector */
	uint32_t edi, esi, ebp, esp, ebx, edx, ecx, eax; /* Pushed by pusha. */
	uint32_t int_no, err_code; /* Interrupt number and error code (if applicable) */
	uint32_t eip, cs, eflags, useresp, ss; /* Pushed by the processor automatically */
} registers_t;

void isr_install();

__attribute__((unused)) void isr_handler(registers_t r);

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