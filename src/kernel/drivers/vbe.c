#include "vbe.h"
#include "pci.h"
#include "../../lib/io.h"

void vbe_write(uint16_t index, uint16_t value) {
    io_write16(index, VBE_DISPI_IOPORT_INDEX);
    io_write16(value, VBE_DISPI_IOPORT_DATA);
}

uint16_t vbe_read(uint16_t index) {
    io_write16(index, VBE_DISPI_IOPORT_INDEX);
    return io_read16(VBE_DISPI_IOPORT_DATA);
}

uint8_t bpp = 0;
uint16_t width = 0;
uint16_t height;
void *framebuffer = NULL;

int32_t vbe_init(uint16_t _width, uint16_t _height, uint8_t _bpp, const struct pci_device *vga_controller) {
    if (NULL == vga_controller) {
        return -1;
    }

    if (VBE_DISPI_ID0 > vbe_read(VBE_DISPI_INDEX_ID)) {
        return -2;
    }

    vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_DISABLED);

    vbe_write(VBE_DISPI_INDEX_XRES, _width);
    vbe_write(VBE_DISPI_INDEX_YRES, _height);
    vbe_write(VBE_DISPI_INDEX_BPP, _bpp);

    vbe_write(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED);

    union BAR bar = pci_read_bar(vga_controller, 0);
    framebuffer = (void *) bar.address;
    if (NULL == framebuffer) {
        bpp = 0;
        width = 0;
        height = 0;
        return -3;
    } else {
        bpp = _bpp;
        width = _width;
        height = _height;
    }
}

void vbe_pixel(uint16_t x, uint16_t y, uint32_t color) {
    if ((x >= width) || (y >= height) || (NULL == framebuffer)) {
        return;
    }

    switch (bpp) {
        case 8:
            ((uint8_t *) framebuffer)[(y * width) + x] = color;
            break;
        case 15:
        case 16:
            ((uint16_t *) framebuffer)[(y * width) + x] = color;
            break;
        case 24:
            ((uint8_t *) framebuffer)[(y * width * 3) + (x * 3)] = color >> 16;
            ((uint8_t *) framebuffer)[(y * width * 3) + (x * 3) + 1] = color;
            ((uint8_t *) framebuffer)[(y * width * 3) + (x * 3) + 2] = color >> 8;
            break;
        case 32:
            ((uint32_t *) framebuffer)[(y * width) + x] = color;
            break;
        default:
            break;
    }

}