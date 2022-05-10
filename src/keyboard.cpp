#include "keyboard.hpp"

static const char scancodes[] = {
	'\0',
	'\0',
	 '1',
	 '2',
	 '3',
	 '4',
	 '5',
	 '6',
	 '7',
	 '8',
	 '9',
	 '0',
	 '?',
	 '`',
	'\b',
	'\t',
	 'q',
	 'w',
	 'e',
	 'r',
	 't',
	 'z',
	 'u',
	 'i',
	 'o',
	 'p',
	 'u',
	 '+',
	'\n',
	'\0',
	 'a',
	 's',
	 'd',
	 'f',
	 'g',
	 'h',
	 'j',
	 'k',
	 'l',
	 'o',
	 'a',
	'\\',
	'\0',
	 '#',
	 'y',
	 'x',
	 'c',
	 'v',
	 'b',
	 'n',
	 'm',
	 ',',
	 '.',
	 '-',
	'\0',
	'\0',
	'\0',
	 ' '
};

static void keyboard_callback(registers_t *regs)
{
	(void) (regs);
	uint8_t scancode = io_in(0x60);

	if (scancode <= 57) {
		input(scancodes[scancode]);
	}

	for(uint16_t i = 0; i < n_callbacks; i++) {
		if (callbacks[i].predicate(scancodes[scancode])) {
			callbacks[i].func(scancodes[scancode]);
		}
	}
}

void init_keyboard()
{
	register_interrupt_handler(IRQ1, keyboard_callback);
}

bool return_false(const char c)
{
	(void)(c);
	return false;
}

void nothing(const char c)
{
	(void)(c);
}

struct KeyboardCallback null_callback = {
	return_false, nothing
};

void register_callback(struct KeyboardCallback kc)
{
	callbacks[n_callbacks++] = kc;	
}

// TODO: fix this huge botch

void drop_callback(struct KeyboardCallback kc)
{
	for(uint16_t i = 0; i < n_callbacks; i++)
	{
		if(callbacks[i].predicate == kc.predicate && callbacks[i].func == kc.func) {
			callbacks[i] = null_callback;
		}
	}
}

/* 'keuyp' event corresponds to the 'keydown' + 0x80
 * it may still be a scancode we haven't implemented yet, or
 * maybe a control/escape sequence */
/*if (scancode <= 0x7f) {
	return ('Unknown key down');
} else if (scancode <= 0x39 + 0x80) {
	return ('key up ');
} else return ('Unknown key up');*/