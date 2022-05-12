#include "shell.hpp"

char buf[N_BUF];
char *buf_ptr;

void redraw()
{
	set_cursor(get_cursor() - (get_cursor() % TEXT_COLS));
	print(PROMPT);
	print(buf);
	for (uint32_t i = 0; i < TEXT_COLS - strlen(PROMPT) - strlen(buf) - 1; ++i) print(" ");
}

void evaluate()
{
	println("");
	void (*cmd)(const char *) = nullptr;
	for (auto &c: commands) {
		if (strcmp(c.cmd, buf) != 0) {
			cmd = c.func;
			break;
		}
	}

	if (cmd == nullptr) {
		print("Command '");
		print(buf);
		println("' not found!");
	} else {
		cmd(buf);
	}

	memset((uint8_t *) (buf), 0, N_BUF);
	buf_ptr = buf;

	redraw();
}

[[noreturn]] void init_shell()
{
	buf_ptr = buf;
	print(PROMPT);

	while (true) {
		char c = get_char();

		if (buf_ptr - buf == N_BUF) continue;

		switch (c) {
			case '\n':
				evaluate();
				break;

			case '\b':
				if (buf_ptr != buf) {
					*(--buf_ptr) = '\0';
					redraw();
				}
				break;

			case '\0':
				redraw();
				break;

			default:
				*(buf_ptr++) = c;
				redraw();
				break;
		}
	}
}
