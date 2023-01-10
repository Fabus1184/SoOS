#include "print.h"

uint16_t get_cursor() {
    // high byte
    io_out(14, SCREEN_CTRL_ADDRESS);
    uint16_t offset = io_in(SCREEN_DATA_ADDRESS) << 8;

    // low byte
    io_out(15, SCREEN_CTRL_ADDRESS);
    offset += io_in(SCREEN_DATA_ADDRESS);

    return offset;
}

void scroll() {
    memcpy((uint8_t *) VIDEO_MEM + (TEXT_COLS * 2), (uint8_t *) VIDEO_MEM, (TEXT_COLS * TEXT_ROWS * 2));
    memset((uint8_t *) VIDEO_MEM + (TEXT_COLS * (TEXT_ROWS - 1) * 2), 0x0, TEXT_COLS * 2);
}

void set_cursor(uint16_t cursor) {
    // high byte
    io_out(14, SCREEN_CTRL_ADDRESS);
    io_out((uint8_t) (cursor >> 8), SCREEN_DATA_ADDRESS);

    // low byte
    io_out(15, SCREEN_CTRL_ADDRESS);
    io_out((uint8_t) (cursor & 0xFF), SCREEN_DATA_ADDRESS);
}

void print_char(const char c) {
    uint16_t cursor = get_cursor();

    switch (c) {
        case '\n':
            println("");
            break;

        case '\b':
            set_cursor(cursor - 1);
            print(" ");
            set_cursor(cursor - 1);
            break;

        case '\t':
            print("    ");
            break;

        default:

            if (cursor >= TEXT_ROWS * TEXT_COLS) {
                scroll();
                set_cursor(cursor - TEXT_COLS);
            }

            cursor = get_cursor();
            VIDEO_MEM[cursor] = (COLOR << 8) + c;
            set_cursor(cursor + 1);
            break;
    }
}

void print(const char *c) {
    for (uint16_t i = 0; c[i] != 0; i++) {
        print_char(c[i]);
    }
}

void println(const char *c) {
    if (c[0] == 0) {
        print(" ");
    } else {
        print(c);
    }

    if (get_cursor() % TEXT_COLS != 0) {
        set_cursor(TEXT_COLS + get_cursor() - (get_cursor() % TEXT_COLS));
    }
}

void clear_screen() {
    uint32_t size = TEXT_COLS * TEXT_ROWS;
    for (uint32_t i = 0; i < size; i++) {
        VIDEO_MEM[i] = 0x000a;
    }
    set_cursor(0);
}
