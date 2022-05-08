#include "soos_string.h"

char *itoa(uint32_t n, char *buf)
{
	if (n == 0) {
		buf = "0";
		return buf;
	}
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

uint32_t strlen(const char *str)
{
	uint32_t i = 0;
	while(str[i] != 0) i++;
	return i;
}

char *pad(uint16_t padding, char filler, char *buf)
{
	uint16_t len = strlen(buf);

	if (len < padding) {
		memcpy((uint8_t*) buf, (uint8_t*) buf + (padding - len), len);
		memset((uint8_t*) buf, filler, padding - len);
	}

	return buf;
}
