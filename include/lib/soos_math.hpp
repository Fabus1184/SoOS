#pragma once

#include <cstdint>

# define M_E        2.7182818284590452354    /* e */
# define M_LOG2E    1.4426950408889634074    /* log_2 e */
# define M_LOG10E    0.43429448190325182765    /* log_10 e */
# define M_LN2        0.69314718055994530942    /* log_e 2 */
# define M_LN10        2.30258509299404568402    /* log_e 10 */
# define M_PI        3.14159265358979323846    /* pi */
# define M_PI_2        1.57079632679489661923    /* pi/2 */
# define M_PI_4        0.78539816339744830962    /* pi/4 */
# define M_1_PI        0.31830988618379067154    /* 1/pi */
# define M_2_PI        0.63661977236758134308    /* 2/pi */
# define M_2_SQRTPI    1.12837916709551257390    /* 2/sqrt(pi) */
# define M_SQRT2    1.41421356237309504880    /* sqrt(2) */
# define M_SQRT1_2    0.70710678118654752440    /* 1/sqrt(2) */

uint32_t pow(uint32_t a, uint32_t b);

uint16_t log10(uint32_t n);

uint32_t sqrt(uint32_t n);

bool isPrime(uint32_t n);

uint32_t fib(uint16_t n);

float sqrt(float n);

uint8_t signum(int32_t n);

float min(float a, float b);

float max(float a, float b);

float sin(float x);

float cos(float x);

float fabs(float x);

float fmod(float a, float b);

float truncate(float x);

float round(float x);

class Vector3
{
public:
	const float x, y, z;

	Vector3(float x, float y, float z);

	Vector3 operator+(const Vector3 &v1) const;

	Vector3 operator-(const Vector3 &v1) const;

	float operator*(const Vector3 &v1) const;

	Vector3 operator*(const float d) const;

	Vector3 operator/(const float d) const;

	Vector3 operator^(const Vector3 &b) const;

	float length() const;

	Vector3 unit() const;

	Vector3 transform(
		float m11, float m12, float m13,
		float m21, float m22, float m23,
		float m31, float m32, float m33
	) const;

	Vector3 rotate(float x, float y, float z, Vector3 origin) const;
};