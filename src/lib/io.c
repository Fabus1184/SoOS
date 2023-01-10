#include "io.h"

void io_write8(uint8_t value, uint32_t port) {
    asm volatile("outb %0, %1" : : "a"(value), "Nd"(port));
}

void io_write16(uint16_t value, uint32_t port) {
    asm volatile("outw %0, %1" : : "a"(value), "Nd"(port));
}

void io_write32(uint32_t value, uint32_t port) {
    asm volatile("outl %0, %1" : : "a"(value), "Nd"(port));
}

uint8_t io_in8(uint32_t port) {
    uint8_t ret;
    asm volatile("inb %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

uint16_t io_read16(uint32_t port) {
    uint16_t ret;
    asm volatile("inw %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}

uint32_t io_read32(uint32_t port) {
    uint32_t ret;
    asm volatile("inl %1, %0" : "=a"(ret) : "Nd"(port));
    return ret;
}
