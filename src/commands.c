#include "commands.h"

void help()
{
	print("Available commands: ");
	
	for(uint16_t i = 0; i < N_CMDS; i++) {
		print(commands[i].cmd);
		print(", ");
	}

	println("");
}

void clear()
{
	clear_screen();
}

void time()
{
	struct DateTime t = get_rtc();

	char buf[20];

	print(pad(2, '0', itoa(t.hour, buf)));
	print(":");
	print(pad(2, '0', itoa(t.minute, buf)));

	print(" ");

	print(pad(2, '0', itoa(t.day, buf)));
	print(".");
	print(pad(2, '0', itoa(t.month, buf)));
	print(".");
	print(itoa(t.year, buf));
	
	
	println("");
}
