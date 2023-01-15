#pragma once

#include <stdint.h>

void set_idt_gate(uint32_t n, uint64_t handler);

void set_idt(void);