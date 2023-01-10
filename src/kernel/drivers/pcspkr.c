#include "pcspkr.h"

void nosound() {
    uint8_t tmp = io_in(0x61) & 0xFC;
    io_out(tmp, 0x61);
}

void play_sound(uint32_t frequency) {
    nosound();
    uint32_t div;
    uint8_t tmp;

    div = 1193180 / frequency;

    io_out(0xb6, 0x43);
    io_out((uint8_t) div, 0x42);
    io_out((uint8_t) (div >> 8), 0x42);

    tmp = io_in(0x61);

    if (tmp != (tmp | 3)) {
        io_out(tmp | 3, 0x61);
    }
}

void beep(const int16_t frequency, const int16_t ms_duration) {
    play_sound(frequency);
    wait(ms_duration);
    nosound();
}
