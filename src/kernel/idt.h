#ifndef HOME_FABIAN_GIT_SOOS_SRC_KERNEL_IDT_H
#define HOME_FABIAN_GIT_SOOS_SRC_KERNEL_IDT_H

#include <stddef.h>
#include <stdint.h>

void set_idt_gate(uint32_t n, uint64_t handler);

void set_idt(void);
#endif
