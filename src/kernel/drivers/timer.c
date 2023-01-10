#include "timer.h"

/**
 * @warning called by an interrupt
 * @param regs
 */
void timer_callback(registers_t *regs) {
    (void) (regs);
    tick++;
}

/**
 * @brief initialized the timer with the given frequency
 * @param frequency
 */
void init_timer(uint32_t frequency) {
    register_interrupt_handler(IRQ0, timer_callback);

    uint32_t div = 1193180 / frequency;
    uint8_t low = div & 0xFF;
    uint8_t high = (div >> 8) & 0xFF;

    io_out(0x36, 0x43);
    io_out(low, 0x40);
    io_out(high, 0x40);

    freq = frequency;
}

/**
 * @brief waits for ms milliseconds
 * @warning calling this function from within an interrupt will never return!
 * @param ms
 */
void wait(uint16_t ms) {
    uint32_t end = tick + ((ms * freq) / 1000);

    while (tick < end) {
        asm volatile("nop");
    }
}
