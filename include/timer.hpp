#pragma once

#include <inttypes.h>
#include "io.hpp"
#include "isr.hpp"
#include "print.hpp"
#include "shell.hpp"

struct TimerCallback {
	uint16_t modulus;
	void (*func) (const uint32_t timer);
} timer_callbacks[255];

void init_timer(uint32_t f);

void register_timer_callback(struct TimerCallback tc);

uint32_t get_timer();

void wait(uint16_t ms);

