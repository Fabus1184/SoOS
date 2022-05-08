#include "shell.h"

char buf[255];
uint16_t buf_size = 0;

bool blink_state = false;

void init_shell()
{
	print(PROMPT);
	print(CURSOR);
}

void interpret(const char *input)
{
	uint16_t i = 0;
	for(; i < N_CMDS; i++) {
		if (strcmp(input, commands[i].cmd)) {
			((void(*)()) commands[i].func)();
			return;
		}
	}

	print("error, command '");
	print(input);
	println("' not found!");
}

void evaluate()
{
	char in[buf_size + 1];
	memcpy((uint8_t*) buf, (uint8_t*) in, buf_size);
	in[buf_size] = 0;

	println("");

	interpret(in);

	input(0);
	input(0);
}

void input(char c)
{
	switch (c) {
		case '\0':
			break;
	
		case '\n':
			evaluate();
			buf_size = 0;
			break;

		case '\b':
			if (buf_size == 0) break;		
			buf_size--;
			break;

		default:	
			buf[buf_size++] = c;
			break;
	}

	set_cursor(get_cursor() - (get_cursor() % COLS) + strlen(PROMPT));
	for(uint16_t i = 0; i < COLS - (get_cursor() % COLS) - strlen(PROMPT); i++) print(" ");
	set_cursor(get_cursor() - (get_cursor() % COLS));
	print(PROMPT);
	for(uint16_t i = 0; i < buf_size; i++) print_char(buf[i]);

	if (buf_size == COLS - strlen(PROMPT) - 1) {
		buf_size--;
	}
}
