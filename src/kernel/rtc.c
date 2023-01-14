#include "rtc.h"

#define CMOS_ADDRESS 0x70
#define CMOS_DATA 0x71
#define CENTURY 2000

bool update_in_progress(void) {
    io_write8(0x0A, CMOS_ADDRESS);
    return io_read8(CMOS_DATA) & 0x80;
}

uint8_t read_rtc_register(uint8_t reg) {
    io_write8(reg, CMOS_ADDRESS);
    return io_read8(CMOS_DATA);
}

struct rtc_time get_rtc_time(void) {
    while (update_in_progress()) { /* */
    }
    struct rtc_time time = (struct rtc_time){
        .seconds = read_rtc_register(0x00),
        .minutes = read_rtc_register(0x02),
        .hours = read_rtc_register(0x04),
        .day_of_month = read_rtc_register(0x07),
        .month = read_rtc_register(0x08),
        .year = read_rtc_register(0x09),
    };

    if (0 == (read_rtc_register(0x0B) & 0x04)) {
        time.seconds = (time.seconds & 0x0F) + ((time.seconds / 16) * 10);
        time.minutes = (time.minutes & 0x0F) + ((time.minutes / 16) * 10);
        time.hours = ((time.hours & 0x0F) + (((time.hours & 0x70) / 16) * 10)) | (time.hours & 0x80);
        time.day_of_month = (time.day_of_month & 0x0F) + ((time.day_of_month / 16) * 10);
        time.month = (time.month & 0x0F) + ((time.month / 16) * 10);
        time.year = (time.year & 0x0F) + ((time.year / 16) * 10);
    }

    if (time.hours & (1 << 7)) {
        time.hours &= ~(1 << 7);
        time.hours += 12;
    }
    time.hours %= 24;

    time.year += CENTURY;

    return time;
}