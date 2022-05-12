#include "keyboard.hpp"

char last_keys[N_LAST_KEYS];
char *last_keys_ptr;

/**
 * @attention german keyboard layout
 * @todo implement different keyboard layouts
 */
static const char SCANCODES[] = {
	'\0', '\0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '?', '`', '\b', '\t', 'q', 'w',
	'e', 'r', 't', 'z', 'u', 'i', 'o', 'p', 'u', '+', '\n', '\0', 'a', 's', 'd', 'f', 'g',
	'h', 'j', 'k', 'l', 'o', 'a', '\\', '\0', '#', 'y', 'x', 'c', 'v', 'b', 'n', 'm', ',',
	'.', '-', '\0', '\0', '\0', ' '
};

/**
 * @param regs unused
 * @attention this is called by an interrupt
 * @todo handle modifier keys
 */
static void keyboard_callback(registers_t *regs)
{
	(void) (regs);
	uint8_t key = io_in(0x60);

	if (key <= 57) {
		*(++last_keys_ptr) = SCANCODES[key];
	}

	if (last_keys_ptr == last_keys + N_LAST_KEYS) {
		last_keys_ptr = last_keys;
	}
}

/**
 * @brief initializes the keyboard IRQ
 */
void init_keyboard()
{
	register_interrupt_handler(IRQ1, keyboard_callback);
	last_keys_ptr = last_keys;
}

char get_char()
{
	while (last_keys_ptr == last_keys) {
		asm volatile("nop");
	}

	return *(last_keys_ptr--);
}