#include "soos_math.h"

uint32_t pow(uint32_t a, uint32_t b)
{
	uint32_t ret = 1;
	for (uint32_t i = 0; i < b; i++) {
		ret *= a;
	}
	return ret;
}

uint16_t log10(uint32_t n)
{
	for (uint16_t ret = 0;; ret++) {
		if (pow(10, ret) > n) return ret - 1;
	}
}

uint32_t sqrt(uint32_t n)
{
	for (uint32_t i = 0;; ++i) {
		if (i * i > n) return n - 1;
	}
}

bool isPrime(uint32_t n)
{
	for (uint32_t i = 2; i < sqrt(n); ++i) {
		if (n % i == 0) return false;
	}
	return true;
}

uint32_t fib(uint16_t n)
{
	if (n == 0 || n == 1) return 1;
	else return fib(n - 2) + fib(n - 1);
}

