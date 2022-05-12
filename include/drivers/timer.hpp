#pragma once

#include <cinttypes>
#include "io.hpp"
#include "isr.hpp"
#include "print.hpp"
#include "shell.hpp"

[[maybe_unused]] static uint32_t tick;
[[maybe_unused]] static uint32_t freq;

void init_timer(uint32_t freq);

void wait(uint16_t ms);

