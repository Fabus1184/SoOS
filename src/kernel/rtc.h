#ifndef HOME_FABIAN_GIT_SOOS_SRC_KERNEL_RTC_H
#define HOME_FABIAN_GIT_SOOS_SRC_KERNEL_RTC_H

#include <lib/io.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

struct rtc_time {
    uint8_t seconds;
    uint8_t minutes;
    uint8_t hours;
    uint8_t day_of_month;
    uint8_t month;
    uint16_t year;
} __attribute__((aligned(8)));

struct rtc_time get_rtc_time(void);

#endif  // HOME_FABIAN_GIT_SOOS_SRC_KERNEL_RTC_H
