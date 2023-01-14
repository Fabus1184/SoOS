; DISCLAIMER: das ist alles total 

%define GDT16_CODE 0x8
%define GDT16_DATA 0x10

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

    ; far jump to 16 bit protected mode
    jmp GDT16_CODE:protected_mode16_start

gdt16:
    dq 0
gdt16_code:
    ; 2 byte limit, 2 byte base
    dw 0xFFFF
    dw 0
    ; 1 byte base
    db 0
    ; present, 2 bit privilege, descriptor type, executable flag, direction flag, read/write flag, accessed flag
    db 0b10011110
    ; granularity, size flag, long mode flag, reserved, 1 byte limit
    db 0b00001111
    ; 1 byte base
    db 0
gdt16_data:
    ; 2 byte limit, 2 byte base
    dw 0xFFFF
    dw 0
    ; 1 byte base
    db 0
    ; present, 2 bit privilege, descriptor type, executable flag, direction flag, read/write flag, accessed flag
    db 0b10010010
    ; granularity, size flag, long mode flag, reserved, 1 byte limit
    db 0b00001111
    ; 1 byte base
    db 0

gdt16_end:
gdt16_descriptor:
    dw gdt16_end - gdt16 - 1
    dd gdt16

global low_memory
low_memory:
    dw 0

global high_memory
high_memory:
    dw 0

[bits 16]

stack:
    resb 0x1000
stack_end:

idt16:
    dw 0x3FF
    dd 0

protected_mode16_start:
    ; set up segment registers
    mov eax, GDT16_DATA
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax
    mov ss, eax

    ; disable protected mode
    mov eax, cr0
    and eax, ~0x1
    mov cr0, eax
    
    ; far jump to 16 bit real mode
    jmp 0:real_mode_start

real_mode_start:
    ; set up stack
    mov sp, stack_end
    mov bp, sp

    ; set up segment registers
    mov ax, 0x0
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; set up idt
    lidt [idt16]

    ; enable interrupts
    sti

    ; setup video mode
    mov ah, 0x00
    mov al, 0x26
    clc
    int 0x10
    jc error_video

    ; print hello world
    mov eax, hello_world_msg
    mov byte ch, 0x0A
    call print_string

    ; detect low memory
    clc
    int 0x12
    jc error_low_memory
    mov word [low_memory], ax

    ; detect high memory
    mov ax, 0xE801
    clc
    int 0x15
    jc error_high_memory

    hlt

x:
    jmp x

error_video:
    mov eax, error_video_msg
    mov byte ch, 0x0C
    call print_string
    hlt

error_low_memory:
    mov eax, error_low_memory_msg
    mov byte ch, 0x0C
    call print_string
    hlt

error_high_memory:
    mov eax, error_high_memory_msg
    mov byte ch, 0x0C
    call print_string
    hlt

print_string:
    mov dword ebx, 0xB8000
print_string_loop:
    mov byte cl, [eax]
    cmp byte cl, 0
    je print_string_end
    mov word [ebx], cx
    inc dword eax
    add dword ebx, 2
    jmp print_string_loop

print_string_end:
    ret

error_video_msg:
    db "Error setting video mode!", 0
error_low_memory_msg:
    db "Error detecting low memory!", 0
error_high_memory_msg:
    db "Error detecting high memory!", 0
hello_world_msg:
    db "Hello World from 16 bit real mode!", 0
