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
	char input_wo_args[buf_size];
	char args[buf_size];

	for(uint16_t i = 0; i <= buf_size; i++) {
		input_wo_args[i] = input[i];

		if (input[i] == ' ') {
			input_wo_args[i] = '\0';
			memcpy(((uint8_t*) input) + i, (uint8_t*) args, buf_size - i);
			break;
		}
	}

	uint16_t i = 0;
	for(; i < N_CMDS; i++) {
		if (strcmp(input_wo_args, commands[i].cmd)) {
			((void(*)()) commands[i].func)(args);
			println("");
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
