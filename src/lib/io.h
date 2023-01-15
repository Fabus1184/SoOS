#ifndef HOME_FABIAN_GIT_SOOS_SRC_LIB_IO_H
#define HOME_FABIAN_GIT_SOOS_SRC_LIB_IO_H

#include <stddef.h>
#include <stdint.h>

void io_write8(uint8_t value, uint32_t port);

void io_write16(uint16_t value, uint32_t port);

void io_write32(uint32_t value, uint32_t port);

uint8_t io_read8(uint32_t port);

uint16_t io_read16(uint32_t port);

uint32_t io_read32(uint32_t port);

#endif  // HOME_FABIAN_GIT_SOOS_SRC_LIB_IO_H
