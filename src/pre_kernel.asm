%define GDT16_CODE 0x8
%define GDT16_DATA 0x10
%define GDT32_CODE 0x8
%define GDT32_DATA 0x10

section .pre_kernel_section
align 4
global pre_kernel

[bits 32]
global low_memory
low_memory:
    dw 0
global high_memory
high_memory:
    times 256 dq 0 0
global high_memory_size
high_memory_size:
    dw 0

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

gdt32:
    dq 0
gdt32_code:
    ; 2 byte limit, 2 byte base
    dw 0xFFFF
    dw 0
    ; 1 byte base
    db 0
    ; present, 2 bit privilege, descriptor type, executable flag, direction flag, read/write flag, accessed flag
    db 0b10011110
    ; granularity, size flag, long mode flag, reserved, 1 byte limit
    db 0b01001111
    ; 1 byte base
    db 0
gdt32_data:
    ; 2 byte limit, 2 byte base
    dw 0xFFFF
    dw 0
    ; 1 byte base
    db 0
    ; present, 2 bit privilege, descriptor type, executable flag, direction flag, read/write flag, accessed flag
    db 0b10010010
    ; granularity, size flag, long mode flag, reserved, 1 byte limit
    db 0b01001111
    ; 1 byte base
    db 0
gdt32_end:
gdt32_descriptor:
    dw gdt32_end - gdt32 - 1
    dd gdt32

stack:
    times 0x2000 db 0
stack_end:

idt16:
    dw 0x3FF
    dd 0

[bits 16]
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
    mov ax, 0
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
    clc
    xor ebx, ebx
    mov dword edx, 0x534D4150
    mov dword eax, 0xE820
    mov word di, high_memory
    mov dword ecx, 0x24

high_memory_loop:
    int 0x15

    ; check if we got a valid memory map
    jc error_high_memory
    cmp eax, 0x534D4150
    jne error_high_memory

    ; setup next call
    mov byte ch, 0
    add word di, 24
    mov dword eax, 0xE820
    mov dword ecx, 24

    ; check if we got the last entry
    cmp ebx, 0
    je high_memory_end
    cmp cl, 0
    je high_memory_end

    jmp high_memory_loop

high_memory_end:
    sub word di, high_memory
    mov word [high_memory_size], di

switch_back_to_protected_mode:
    ; disable interrupts
    cli

    ; load 32 bit gdt
    xor eax, eax
    mov ds, ax
    lgdt [gdt32_descriptor]

    ; enable protected mode
    mov eax, cr0
    or eax, 0x1
    mov cr0, eax

    ; far jump to 32 bit protected mode
    jmp GDT32_CODE:mode_32

[bits 32]
[extern boot_kernel]
mode_32:
    ; set up segment registers
    mov eax, GDT32_DATA
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax
    mov ss, eax

    ; jump to boot.asm kernel
    jmp boot_kernel

[bits 16]
error_video:
    mov eax, error_video_msg
    mov byte ch, 0x0C
    call print_string
    hlt
    jmp $

error_low_memory:
    mov eax, error_low_memory_msg
    mov byte ch, 0x0C
    call print_string
    hlt
    jmp $

error_high_memory:
    mov eax, error_high_memory_msg
    mov byte ch, 0x0C
    call print_string
    hlt
    jmp $

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
