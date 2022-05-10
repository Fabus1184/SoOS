#pragma once

#include <inttypes.h>

#include "io.hpp"
#include "isr.hpp"
#include "print.hpp"
#include "shell.hpp"

#define MAX_KCBS 255

uint16_t n_callbacks = 0;

struct KeyboardCallback {
	bool (*predicate) (const char c);
	void (*func) (const char c);
};

struct KeyboardCallback callbacks[MAX_KCBS];

void init_keyboard();

void register_callback(struct KeyboardCallback);

void drop_callback(struct KeyboardCallback);