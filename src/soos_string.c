#include "soos_string.h"

char *itoa(uint32_t n, char *buf)
{
	uint16_t i = 0;
	for (; i <= log10(n); i++) {
		buf[i] = (char) ('0' + (n / pow(10, i)) % 10);
	}

	reverse(buf, log10(n) + 1);

	buf[i] = 0;
	return buf;
}

char *reverse(char *buf, uint16_t n)
{
	for (uint16_t i = 0; i < n / 2; ++i) {
		char tmp = buf[i];
		buf[i] = buf[n - i - 1];
		buf[n - i - 1] = tmp;
	}
	return buf;
}

bool strcmp(const char *s1, const char *s2)
{
	for (uint16_t i = 0;; ++i) {
		if ((s1[i] == 0) ^ (s2[i] == 0)) return false;
		if (s1[i] != s2[i]) return false;
		if (s1[i] == 0 && s2[i] == 0) return true;
	}
}
