#include "vbe.h"

void vbe_write(uint16_t index, uint16_t value) {
    io_write16(index, VBE_DISPI_IOPORT_INDEX);
    io_write16(value, VBE_DISPI_IOPORT_DATA);
}

static uint8_t bpp = 0;
static uint16_t width = 0;

void vbe_init(uint16_t _width, uint16_t height, uint8_t _bpp) {
    // disable vbe extensions
    vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_DISABLED);

    // set resolution
    vbe_write(VBE_DISPI_INDEX_XRES, _width);
    vbe_write(VBE_DISPI_INDEX_YRES, height);

    // set bpp
    vbe_write(VBE_DISPI_INDEX_BPP, _bpp);

    // enable vbe extensions
    vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED);

    bpp = _bpp;
    width = _width;
}

void vbe_pixel(uint16_t x, uint16_t y, uint32_t color) {
    void *fb = (uint32_t *) VBE_DISPI_BANK_ADDRESS;
    switch (bpp) {
        case 8:
            ((uint8_t *) fb)[(y * width) + x] = color;
            break;
        case 15:
        case 16:
            ((uint16_t *) fb)[(y * width) + x] = color;
            break;
        case 24:
        case 32:
            ((uint32_t *) fb)[(y * width) + x] = color;
            break;
        default:
            break;
    }

}