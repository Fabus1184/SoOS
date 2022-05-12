#include "graphics.hpp"

Rectangle::Rectangle(Vector3 a, Vector3 b, Vector3 c, Vector3 d) : a(a), b(b), c(c), d(d)
{
}

void Rectangle::draw(uint8_t color) const
{
	draw_line(a, b, color);
	draw_line(b, c, color);
	draw_line(c, d, color);
	draw_line(d, a, color);
}

Rectangle Rectangle::rotate(float xx, float yy, float zz, Vector3 origin) const
{
	return {
		a.rotate(xx, yy, zz, origin),
		b.rotate(xx, yy, zz, origin),
		c.rotate(xx, yy, zz, origin),
		d.rotate(xx, yy, zz, origin),
	};
}

void draw_line(Vector3 v1, Vector3 v2, uint8_t color)
{
	(void) (v2);
	put_pixel((uint16_t) v1.x, (uint16_t) v1.y, color);

	uint16_t i = 0, x, y;
	do {
		Vector3 v = {v1 + ((v2 - v1).unit() * i++)};
		x = (uint16_t) round(v.x);
		y = (uint16_t) round(v.y);

		if (v.x < 0 || v.y < 0) {
			continue;
		} else {
			put_pixel(x, y, color);
		}
	} while (i <= (uint16_t) round((v2 - v1).length() / (v2 - v1).unit().length()));
}
