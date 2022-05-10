#pragma once

#include <inttypes.h>

#include "io.hpp"

#define CURRENT_YEAR 2022
#define CMOS_ADDRESS 0x70
#define CMOS_DATA 0x71

uint32_t century_register = 0x00;

struct DateTime {

	uint8_t second;
	uint8_t minute;
	uint8_t hour;
	uint8_t day;
	uint8_t month;
	uint32_t year;
};

struct DateTime get_rtc();
