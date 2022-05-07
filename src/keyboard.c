#include "keyboard.h"

static const char *scancodes[] = {
	"ERROR", "ESC", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "+", "Backspace", "Tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "[", "]", "ENTER", "LCtrl",
	"A", "S", "D", "F", "G", "H", "J", "K", "L", ";", "'", "`", "LShift", "\\", "Z", "X", "C", "V", "B", "N", "M", ",", ".", "/", "Rshift", "Keypad *", "LAlt", "Spc"
};

static void keyboard_callback(registers_t *regs)
{
	(void) (regs);
	uint8_t scancode = io_in(0x60);

	if (scancode <= 57) {
		print(scancodes[scancode]);
	}
}

void init_keyboard()
{
	register_interrupt_handler(IRQ1, keyboard_callback);
}


/* 'keuyp' event corresponds to the 'keydown' + 0x80
 * it may still be a scancode we haven't implemented yet, or
 * maybe a control/escape sequence */
/*if (scancode <= 0x7f) {
	return ("Unknown key down");
} else if (scancode <= 0x39 + 0x80) {
	return ("key up ");
} else return ("Unknown key up");*/
