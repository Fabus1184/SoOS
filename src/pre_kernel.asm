section .pre_kernel_section
align 4
global pre_kernel

[bits 32]
pre_kernel:
switch_to_real_mode:
    ; disable interrupts
    cli

    ; TODO: turn off paging if enabled
    
    ; load 16 bit gdt
    xor ax, ax
    mov ds, ax
    lgdt [gdt16_descriptor]

    ; far jump to real mode
    jmp 0x8:real_mode_start

gdt16:
    dq 0
gdt16_code:
    ; 2 byte limit, 2 byte base
    dw 0xFFFFF
    dw 0
    ; 1 byte base
    db 0
    ; granularity, size flag, long mode flag, reserved, 1 byte limit
    db 0b00000000
    ; present, 2 bit privilege, descriptor type, executable flag, direction flag, read/write flag, accessed flag
    db 0b10011010
    ; 1 byte base
    db 0
gdt16_data:
    dw 0xFFFFF
    dw 0
    db 0
    db 0b00000000
    db 0b10010010
    db 0
gdt16_end:
gdt16_descriptor:
    dw gdt16_end - gdt16 - 1
    dd gdt16

global low_memory
low_memory:
    dw 0

[bits 16]
real_mode_start:
    hlt

    ; set up segment registers
    mov ax, 0x8
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; set up stack
    mov sp, 0x7C00
    mov bp, 0x7C00

    ; disable protected mode
    mov eax, cr0
    and eax, 0xFFFFFFFE
    mov cr0, eax

    ; set video mode
    mov ah, 0
    mov al, 0
    int 0x10

    ; clear carry flag
    clc

    ; detect low memory
    int 0x12

    ; check for error
    jc low_memory_error

    mov [low_memory], ax

    ; amount of memory in kb in ax

low_memory_error:
    hlt



    