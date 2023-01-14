/*****************************************************************************
https://files.osdev.org/mirrors/geezer/osd/graphics/modes.c
Sets VGA-compatible video modes without using the BIOS
Chris Giese <geezer@execpc.com>	http://my.execpc.com/~geezer
Release date: ?

This code is public domain (no copyright).
You can do whatever you want with it.

*****************************************************************************/

#include "vga.h"

#include "vga_modes.c.inc"

#define VGA_AC_INDEX 0x3C0
#define VGA_AC_WRITE 0x3C0
#define VGA_AC_READ 0x3C1
#define VGA_MISC_WRITE 0x3C2
#define VGA_SEQ_INDEX 0x3C4
#define VGA_SEQ_DATA 0x3C5
#define VGA_DAC_READ_INDEX 0x3C7
#define VGA_DAC_WRITE_INDEX 0x3C8
#define VGA_DAC_DATA 0x3C9
#define VGA_MISC_READ 0x3CC
#define VGA_GC_INDEX 0x3CE
#define VGA_GC_DATA 0x3CF
/*			COLOR emulation		MONO emulation */
#define VGA_CRTC_INDEX 0x3D4 /* 0x3B4 */
#define VGA_CRTC_DATA 0x3D5  /* 0x3B5 */
#define VGA_INSTAT_READ 0x3DA

#define VGA_NUM_SEQ_REGS 5
#define VGA_NUM_CRTC_REGS 25
#define VGA_NUM_GC_REGS 9
#define VGA_NUM_AC_REGS 21
#define VGA_NUM_REGS (1 + VGA_NUM_SEQ_REGS + VGA_NUM_CRTC_REGS + VGA_NUM_GC_REGS + VGA_NUM_AC_REGS)

uint8_t VGA_X_SIZE;
uint8_t VGA_Y_SIZE;

/*****************************************************************************
*****************************************************************************/
void read_regs(uint8_t *regs) {
    uint32_t i;

    /* read MISCELLANEOUS reg */
    *regs = io_read8(VGA_MISC_READ);
    regs++;
    /* read SEQUENCER regs */
    for (i = 0; i < VGA_NUM_SEQ_REGS; i++) {
        io_write8(i, VGA_SEQ_INDEX);
        *regs = io_read8(VGA_SEQ_DATA);
        regs++;
    }
    /* read CRTC regs */
    for (i = 0; i < VGA_NUM_CRTC_REGS; i++) {
        io_write8(i, VGA_CRTC_INDEX);
        *regs = io_read8(VGA_CRTC_DATA);
        regs++;
    }
    /* read GRAPHICS CONTROLLER regs */
    for (i = 0; i < VGA_NUM_GC_REGS; i++) {
        io_write8(i, VGA_GC_INDEX);
        *regs = io_read8(VGA_GC_DATA);
        regs++;
    }
    /* read ATTRIBUTE CONTROLLER regs */
    for (i = 0; i < VGA_NUM_AC_REGS; i++) {
        (void) io_read8(VGA_INSTAT_READ);
        io_write8(i, VGA_AC_INDEX);
        *regs = io_read8(VGA_AC_READ);
        regs++;
    }
    /* lock 16-color palette and unblank display */
    (void) io_read8(VGA_INSTAT_READ);
    io_write8(0x20, VGA_AC_INDEX);
}
/*****************************************************************************
*****************************************************************************/
void write_regs(uint8_t *regs) {
    uint32_t i;

    /* write MISCELLANEOUS reg */
    io_write8(*regs, VGA_MISC_WRITE);
    regs++;
    /* write SEQUENCER regs */
    for (i = 0; i < VGA_NUM_SEQ_REGS; i++) {
        io_write8(i, VGA_SEQ_INDEX);
        io_write8(*regs, VGA_SEQ_DATA);
        regs++;
    }
    /* unlock CRTC registers */
    io_write8(0x03, VGA_CRTC_INDEX);
    io_write8(io_read8(VGA_CRTC_DATA) | 0x80, VGA_CRTC_DATA);
    io_write8(0x11, VGA_CRTC_INDEX);
    io_write8(io_read8(VGA_CRTC_DATA) & ~0x80, VGA_CRTC_DATA);

    regs[0x03] |= 0x80;
    regs[0x11] &= ~0x80;
    /* write CRTC regs */
    for (i = 0; i < VGA_NUM_CRTC_REGS; i++) {
        io_write8(i, VGA_CRTC_INDEX);
        io_write8(*regs, VGA_CRTC_DATA);
        regs++;
    }
    /* write GRAPHICS CONTROLLER regs */
    for (i = 0; i < VGA_NUM_GC_REGS; i++) {
        io_write8(i, VGA_GC_INDEX);
        io_write8(*regs, VGA_GC_DATA);
        regs++;
    }
    /* write ATTRIBUTE CONTROLLER regs */
    for (i = 0; i < VGA_NUM_AC_REGS; i++) {
        (void) io_read8(VGA_INSTAT_READ);
        io_write8(i, VGA_AC_INDEX);
        io_write8(*regs, VGA_AC_WRITE);
        regs++;
    }
    /* lock 16-color palette and unblank display */
    (void) io_read8(VGA_INSTAT_READ);
    io_write8(0x20, VGA_AC_INDEX);
}
/*****************************************************************************
*****************************************************************************/
void set_plane(uint32_t p) {
    uint8_t pmask;

    p &= 3;
    pmask = 1 << p;
    /* set read plane */
    io_write8(4, VGA_GC_INDEX);
    io_write8(p, VGA_GC_DATA);
    /* set write plane */
    io_write8(2, VGA_SEQ_INDEX);
    io_write8(pmask, VGA_SEQ_DATA);
}
/*****************************************************************************
VGA framebuffer is at A000:0000, B000:0000, or B800:0000
depending on bits in GC 6
*****************************************************************************/
uint32_t get_fb_seg(void) {
    uint32_t seg;

    io_write8(6, VGA_GC_INDEX);
    seg = io_read8(VGA_GC_DATA);
    seg >>= 2;
    seg &= 3;
    switch (seg) {
        case 0:
        case 1:
            seg = 0xA000;
            break;
        case 2:
            seg = 0xB000;
            break;
        case 3:
            seg = 0xB800;
            break;
    }
    return seg;
}

/*****************************************************************************
write font to plane P4 (assuming planes are named P1, P2, P4, P8)
*****************************************************************************/
void write_font(uint8_t *buf, uint32_t font_height) {
    uint8_t seq2, seq4, gc4, gc5, gc6;
    uint32_t i;

    /* save registers
    set_plane() modifies GC 4 and SEQ 2, so save them as well */
    io_write8(2, VGA_SEQ_INDEX);
    seq2 = io_read8(VGA_SEQ_DATA);

    io_write8(4, VGA_SEQ_INDEX);
    seq4 = io_read8(VGA_SEQ_DATA);
    /* turn off even-odd addressing (set flat addressing)
    assume: chain-4 addressing already off */
    io_write8(seq4 | 0x04, VGA_SEQ_DATA);

    io_write8(4, VGA_GC_INDEX);
    gc4 = io_read8(VGA_GC_DATA);

    io_write8(5, VGA_GC_INDEX);
    gc5 = io_read8(VGA_GC_DATA);
    /* turn off even-odd addressing */
    io_write8(gc5 & ~0x10, VGA_GC_DATA);

    io_write8(6, VGA_GC_INDEX);
    gc6 = io_read8(VGA_GC_DATA);
    /* turn off even-odd addressing */
    io_write8(gc6 & ~0x02, VGA_GC_DATA);
    /* write font to plane P4 */
    set_plane(2);
    /* write font 0 */
    for (i = 0; i < 256; i++) {
        memcpy((uint8_t *) (get_fb_seg() * 16) + (16384u * 0 + i * 32), buf, font_height);
        buf += font_height;
    }

    /* restore registers */
    io_write8(2, VGA_SEQ_INDEX);
    io_write8(seq2, VGA_SEQ_DATA);
    io_write8(4, VGA_SEQ_INDEX);
    io_write8(seq4, VGA_SEQ_DATA);
    io_write8(4, VGA_GC_INDEX);
    io_write8(gc4, VGA_GC_DATA);
    io_write8(5, VGA_GC_INDEX);
    io_write8(gc5, VGA_GC_DATA);
    io_write8(6, VGA_GC_INDEX);
    io_write8(gc6, VGA_GC_DATA);
}

void set_cursor(uint32_t offset) {
    /* cursor LOW port to vga INDEX register */
    io_write8(0x0F, VGA_CRTC_INDEX);
    io_write8(offset & 0xFF, VGA_CRTC_DATA);
    /* cursor HIGH port to vga INDEX register */
    io_write8(0x0E, VGA_CRTC_INDEX);
    io_write8(offset >> 8, VGA_CRTC_DATA);
}