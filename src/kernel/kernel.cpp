#include "print.hpp"
#include "soos_string.hpp"
#include "isr.hpp"

static char welcome[17][TEXT_COLS + 1] = {
	R"(+------------------------------------------------------------------------------+)",
	R"(|                _    _      _                            _                    |)",
	R"(|               | |  | |    | |                          | |                   |)",
	R"(|               | |  | | ___| | ___ ___  _ __ ___   ___  | |_ ___              |)",
	R"(|               | |/\| |/ _ \ |/ __/ _ \| '_ ` _ \ / _ \ | __/ _ \             |)",
	R"(|               \  /\  /  __/ | (_| (_) | | | | | |  __/ | || (_) |            |)",
	R"(|                \/  \/ \___|_|\___\___/|_| |_| |_|\___|  \__\___/             |)",
	R"(|                                                                              |)",
	R"(|                                                                              |)",
	R"(|                          _____       _____ _____                             |)",
	R"(|                         /  ___|     |  _  /  ___|                            |)",
	R"(|                         \ `--.  ___ | | | \ `--.                             |)",
	R"(|                          `--. \/ _ \| | | |`--. \                            |)",
	R"(|                         /\__/ / (_) \ \_/ /\__/ /                            |)",
	R"(|                         \____/ \___/ \___/\____/                             |)",
	R"(|                                                                              |)",
	R"(+------------------------------------------------------------------------------+)"
};

extern "C" __attribute__((unused, noreturn)) void kmain()
{
	isr_install();
	irq_install();

	clear_screen();

	for (auto str: welcome) println(str);
	println("");

	init_shell();

	// this should never return
	asm volatile("hlt");
	while (true) asm volatile("nop");
}