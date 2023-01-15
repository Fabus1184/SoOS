[bits 32]
section .lowmem
align 4

global gdt16_descriptor
global gdt32_descriptor

global GDT16_CODE
global GDT16_DATA

global GDT32_CODE
global GDT32_DATA

GDT16_CODE equ gdt16_code - gdt16
GDT16_DATA equ gdt16_data - gdt16

GDT32_CODE equ gdt32_code - gdt32
GDT32_DATA equ gdt32_data - gdt32

; gdt with 16 bit code and data segments
gdt16:
    dq 0
gdt16_code:
    ; 2 byte limit = 0xFFFF, 2 byte base = 0
    dw 0xFFFF
    dw 0
    ; 1 byte base = 0
    db 0
    ; present = 1, 2 bit privilege = 0, descriptor type = 1, executable flag = 1, direction flag = 0, read/write flag = 1, accessed flag = 0
    db 0b10011110
    ; granularity = 1, size flag = 0, long mode flag = 0, reserved = 0, 1 byte limit = 0xFF
    db 0b00001111
    ; 1 byte base = 0
    db 0
gdt16_data:
    ; 2 byte limit = 0xFFFF, 2 byte base = 0
    dw 0xFFFF
    dw 0
    ; 1 byte base = 0
    db 0
    ; present = 1, 2 bit privilege = 0, descriptor type = 1, executable flag = 0, direction flag = 0, read/write flag = 1, accessed flag = 0
    db 0b10010010
    ; granularity = 1, size flag = 0, long mode flag = 0, reserved = 0, 1 byte limit = 0xFF
    db 0b00001111
    ; 1 byte base = 0
    db 0
gdt16_end:
gdt16_descriptor:
    dw gdt16_end - gdt16 - 1
    dd gdt16

; gdt with 32 bit code and data segments
gdt32:
    dq 0
gdt32_code:
    ; 2 byte limit = 0xFFFF, 2 byte base = 0
    dw 0xFFFF
    dw 0
    ; 1 byte base = 0
    db 0
    ; present = 1, 2 bit privilege = 0, descriptor type = 1, executable flag = 1, direction flag = 0, read/write flag = 1, accessed flag = 0
    db 0b10011010
    ; granularity = 1, size flag = 1, long mode flag = 0, reserved = 0, 1 byte limit = 0xFF
    db 0b11001111
    ; 1 byte base = 0
    db 0
gdt32_data:
    ; 2 byte limit = 0xFFFF, 2 byte base = 0
    dw 0xFFFF
    dw 0
    ; 1 byte base = 0
    db 0
    ; present = 1, 2 bit privilege = 0, descriptor type = 1, executable flag = 0, direction flag = 0, read/write flag = 1, accessed flag = 0
    db 0b10010010
    ; granularity = 1, size flag = 1, long mode flag = 0, reserved = 0, 1 byte limit = 0xFF
    db 0b11001111
    ; 1 byte base = 0
    db 0
gdt32_end:
gdt32_descriptor:
    dw gdt32_end - gdt32 - 1
    dd gdt32

[bits 64]
section .rodata
align 8

global GDT64_CODE
global gdt64_descriptor

GDT64_CODE equ gdt64_code - gdt64

gdt64:
    ; null descriptor
    dq 0
gdt64_code:
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53)
gdt64_end:
gdt64_descriptor:
    dw gdt64_end - gdt64 - 1
    dq gdt64
