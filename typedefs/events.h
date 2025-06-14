#include <stdint.h>

#include "types.h"

struct __attribute__((packed)) mouse_event_t {
    int8_t x_movement;
    int8_t y_movement;
    uint8_t left_button_pressed;
    uint8_t right_button_pressed;
};