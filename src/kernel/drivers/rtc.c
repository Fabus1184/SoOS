#include "rtc.h"

/**
 * @return 1 if rtc update is in progress
 */
uint32_t get_update_in_progress_flag() {
    io_out(0x0A, CMOS_ADDRESS);
    return io_in(CMOS_DATA) & 0x80;
}

/**
 * @param reg
 * @return read data from RTC
 */
uint8_t get_RTC_register(uint32_t reg) {
    io_out(reg, CMOS_ADDRESS);
    return io_in(CMOS_DATA);
}

/**
 * @brief get RTC time from CMOS
 * @return struct DateTime, the RTC may not have a century register and century will be a hardcoded value from rtc.h
 */
struct DateTime get_rtc() {
    while (get_update_in_progress_flag());

    uint8_t second = get_RTC_register(0x00);
    uint8_t minute = get_RTC_register(0x02);
    uint8_t hour = get_RTC_register(0x04);
    uint8_t day = get_RTC_register(0x07);
    uint8_t month = get_RTC_register(0x08);
    uint32_t year = get_RTC_register(0x09);
    uint8_t century;

    uint8_t last_second;
    uint8_t last_minute;
    uint8_t last_hour;
    uint8_t last_day;
    uint8_t last_month;
    uint8_t last_year;
    uint8_t last_century;

    if (century_register != 0) {
        century = get_RTC_register(century_register);
    } else {
        century = 0;
    }

    do {
        last_second = second;
        last_minute = minute;
        last_hour = hour;
        last_day = day;
        last_month = month;
        last_year = year;
        last_century = century;

        while (get_update_in_progress_flag());

        second = get_RTC_register(0x00);
        minute = get_RTC_register(0x02);
        hour = get_RTC_register(0x04);
        day = get_RTC_register(0x07);
        month = get_RTC_register(0x08);
        year = get_RTC_register(0x09);

        if (century_register != 0) {
            century = get_RTC_register(century_register);
        } else {
            century = 0;
        }

    } while (
            (last_second != second) || (last_minute != minute) || (last_hour != hour) || (last_day != day) ||
            (last_month != month)
            || (last_year != year) || (last_century != century)
            );

    uint8_t registerB = get_RTC_register(0x0B);

    if (!(registerB & 0x04)) {
        second = (second & 0x0F) + ((second / 16) * 10);
        minute = (minute & 0x0F) + ((minute / 16) * 10);
        hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10)) | (hour & 0x80);
        day = (day & 0x0F) + ((day / 16) * 10);
        month = (month & 0x0F) + ((month / 16) * 10);
        year = (year & 0x0F) + ((year / 16) * 10);
        if (century_register != 0) {
            century = (century & 0x0F) + ((century / 16) * 10);
        }
    }

    if (!(registerB & 0x02) && (hour & 0x80)) {
        hour = ((hour & 0x7F) + 12) % 24;
    }

    if (century_register != 0) {
        year += century * 1000;
    } else {
        year += (CURRENT_YEAR / 100) * 100;
        if (year < CURRENT_YEAR) year += 100;
    }

    struct DateTime ret = {second, minute, hour, day, month, year};
    return ret;
}
