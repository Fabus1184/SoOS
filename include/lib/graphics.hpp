#pragma once

#include <cstdint>

#include "vga.hpp"
#include "soos_math.hpp"

class Rectangle
{
public:
	const Vector3 a, b, c, d;

	Rectangle(Vector3 a, Vector3 b, Vector3 c, Vector3 d);

	void draw(uint8_t color) const;

	Rectangle rotate(float xx, float yy, float zz, Vector3 origin) const;

};

void draw_line(Vector3 v1, Vector3 v2, uint8_t color);