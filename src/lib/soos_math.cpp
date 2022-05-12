#include "soos_math.hpp"

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

uint8_t signum(int32_t n)
{
	if (n < 0) return -1;
	if (n > 0) return 1;
	return 0;
}

float min(float a, float b)
{
	return a < b ? a : b;
}

float max(float a, float b)
{
	return a > b ? a : b;
}

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wstrict-aliasing"

float sqrt(float number)
{
	long i;
	float x2, y;
	const float threehalfs = 1.5F;

	x2 = number * 0.5F;
	y = number;
	i = *(long *) &y;                     // floating point bit level hacking [sic]
	i = 0x5f3759df - (i >> 1);             // Newton's approximation
	y = *(float *) &i;
	y = y * (threehalfs - (x2 * y * y)); // 1st iteration
	y = y * (threehalfs - (x2 * y * y)); // 2nd iteration
	y = y * (threehalfs - (x2 * y * y)); // 3rd iteration

	return 1 / y;
}

#pragma GCC diagnostic pop

float fabs(float x)
{
	return x < 0 ? -x : x;
}

float sin(float n)
{
	int16_t steps = 0;
	int16_t max_steps = 100;
	float denominator, sinx;
	float temp = n;
	sinx = n;
	int i = 1;
	do {
		denominator = 2.0f * (float) i * (2.0f * (float) i + 1.0f);
		temp = -temp * n * n / denominator;
		sinx = sinx + temp;
		i = i + 1;
	} while (steps++ < max_steps);
	return sinx;
}

float cos(float n)
{
	return sin((float) (n + M_PI_2));
}

float fmod(float a, float b)
{
	return a - truncate(a / b) * b;
}

float truncate(float x)
{
	return x < 0 ? -(float) ((int32_t) x) : (float) ((int32_t) x);
}

float round(float x)
{
	return truncate(x + 0.5f);
}

Vector3::Vector3(float x, float y, float z) : x(x), y(y), z(z)
{
}

Vector3 Vector3::operator+(const Vector3 &v1) const
{
	return {v1.x + this->x, v1.y + this->y, v1.z + this->z};
}

Vector3 Vector3::operator-(const Vector3 &v1) const
{
	return {this->x - v1.x, this->y - v1.y, this->z - v1.z};
}

float Vector3::operator*(const Vector3 &v1) const
{
	return (v1.x * this->x) + (v1.y * this->y) + (v1.z * this->z);
}

Vector3 Vector3::operator*(const float d) const
{
	return {d * this->x, d * this->y, d * this->z};
}

Vector3 Vector3::operator/(const float d) const
{
	return {this->x / d, this->y / d, this->z / d};
}

Vector3 Vector3::operator^(const Vector3 &b) const
{
	return {
		(this->y * b.z) - (this->z * b.y),
		(this->z * b.x) - (this->x * b.z),
		(this->x * b.y) - (this->y * b.x),
	};
}

float Vector3::length() const
{
	return sqrt((this->x * this->x) + (this->y * this->y) + (this->z * this->z));
}

Vector3 Vector3::unit() const
{
	return (*this) / (this->length());
}

Vector3 Vector3::transform(
	float m11, float m12, float m13,
	float m21, float m22, float m23,
	float m31, float m32, float m33
) const
{
	return {
		m11 * x + m12 * y + m13 * z,
		m21 * x + m22 * y + m23 * z,
		m31 * x + m32 * y + m33 * z
	};
}

Vector3 Vector3::rotate(float xx, float yy, float zz, Vector3 origin) const
{
	xx = fmod(xx, M_PI * 2.0f);
	yy = fmod(xx, M_PI * 2.0f);
	zz = fmod(xx, M_PI * 2.0f);

	return ((*this) - origin).transform(
		cos(yy) * cos(zz), sin(xx) * sin(yy) * cos(zz) - cos(xx) * sin(zz), cos(xx) * sin(yy) * cos(zz) + sin(xx) * sin(zz),
		cos(yy) * sin(zz), sin(xx) * sin(yy) * sin(zz) + cos(xx) * cos(zz), cos(xx) * sin(yy) * sin(zz) - sin(xx) * cos(zz),
		-sin(yy), sin(xx) * cos(yy), cos(xx) * cos(yy)
	) + origin;
}