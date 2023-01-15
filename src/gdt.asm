[bits 16]
section .lowmem
align 4

global GDT_CODE16
global GDT_DATA16
global GDT_CODE32
global GDT_DATA32
global GDT_CODE64
global GDT_DATA64
global gdt_descriptor

GDT_CODE16 equ gdt_code16 - gdt
GDT_DATA16 equ gdt_data16 - gdt
GDT_CODE32 equ gdt_code32 - gdt
GDT_DATA32 equ gdt_data32 - gdt
GDT_CODE64 equ gdt_code64 - gdt
GDT_DATA64 equ gdt_data64 - gdt

global setup_gdt
setup_gdt:
    lgdt [gdt_descriptor]
    ret

; gdt with 16 and 32 bit code and data segments
gdt:
    dq 0
gdt_code16:
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
gdt_data16:
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
gdt_code32:
    ; 2 byte limit = 0xFFFF, 2 byte base = 0
    dw 0xFFFF
    dw 0
    ; 1 byte base = 0
    db 0
    ; present = 1, 2 bit privilege = 0, descriptor type = 1, executable flag = 1, direction flag = 0, read/write flag = 1, accessed flag = 0
    db 0b10011110
    ; granularity = 1, size flag = 1, long mode flag = 0, reserved = 0, 1 byte limit = 0xFF
    db 0b01001111
    ; 1 byte base
    db 0
gdt_data32:
    ; 2 byte limit = 0xFFFF, 2 byte base = 0
    dw 0xFFFF
    dw 0
    ; 1 byte base = 0
    db 0
    ; present = 1, 2 bit privilege = 0, descriptor type = 1, executable flag = 0, direction flag = 0, read/write flag = 1, accessed flag = 0
    db 0b10010010
    ; granularity = 1, size flag = 1, long mode flag = 0, reserved = 0, 1 byte limit = 0xFF
    db 0b01001111
    ; 1 byte base = 0
    db 0
gdt_code64:
    ; 2 byte limit = 0xFFFF, 2 byte base = 0
    dw 0xFFFF
    dw 0
    ; 1 byte base = 0
    db 0
    ; present = 1, 2 bit privilege = 0, descriptor type = 1, executable flag = 1, direction flag = 0, read/write flag = 1, accessed flag = 0
    db 0b10011010
    ; granularity = 1, size flag = 1, long mode flag = 1, reserved = 0, 1 byte limit = 0xFF
    db 0b01001111
    ; 1 byte base
    db 0
gdt_data64:
    ; 2 byte limit = 0xFFFF, 2 byte base = 0
    dw 0xFFFF
    dw 0
    ; 1 byte base = 0
    db 0
    ; present = 1, 2 bit privilege = 0, descriptor type = 1, executable flag = 0, direction flag = 0, read/write flag = 1, accessed flag = 0
    db 0b10010010
    ; granularity = 1, size flag = 1, long mode flag = 1, reserved = 0, 1 byte limit = 0
    db 0b01001111
    ; 1 byte base = 0
    db 0
gdt_end:
gdt_descriptor:
    dw gdt_end - gdt - 1
    dd gdt
