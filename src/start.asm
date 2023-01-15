[bits 32]

section .bss
align 4
stack32:
    resb 0x10000
stack32_end:

section .text
align 4

; entry point where the bootloader jumps to
global _start
_start:
    ; disable interrupts
    cli
    
    ; set up stack
    mov esp, stack32_end
    mov ebp, esp
    
    ; load gdt
    xor ax, ax
    mov ds, ax
    extern gdt_descriptor
    lgdt [gdt_descriptor]

    ; far jump to 16 bit protected mode
    extern GDT_CODE16
    jmp GDT_CODE16:protected_mode16

[bits 16]
section .lowmem
align 4

stack16:
    times 0x2000 db 0x0
stack16_end:

protected_mode16:
    ; set up stack
    mov sp, stack16_end
    mov bp, sp

    ; set up segment registers
    extern GDT_DATA16
    mov eax, GDT_DATA16
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
    jmp 0x0:real_mode16

idt16:
    dw 0x3FF
    dd 0x0

real_mode16:
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

    ; execute neccessary tasks in real mode
    extern detect_memory
    call detect_memory

    ; switch back to protected mode
    
    ; disable interrupts
    cli

    ; enable protected mode
    mov eax, cr0
    or eax, 0x1
    mov cr0, eax

    ; far jump to 32 bit protected mode
    extern GDT_CODE32
    jmp GDT_CODE32:protected_mode32

[bits 32]

section .lowmem
align 4

; has to be in lowmem because of the far jump
protected_mode32:
    jmp protected_mode32_start

section .text
align 4

; resume in 32 bit protected mode at text section
protected_mode32_start:
    ; set up stack
    mov esp, stack32_end
    mov ebp, esp

    ; set up segment registers
    extern GDT_DATA32
    mov eax, GDT_DATA32
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax
    mov ss, eax

    ; setup & enable paging, then switch to long mode
    extern paging_and_long_mode
    call paging_and_long_mode

    ; load 64 bit gdt
    extern gdt64_descriptor
    lgdt [gdt64_descriptor]

    ; far jump to 64 bit long mode
    extern GDT64_CODE
    jmp GDT64_CODE:long_mode_start

[bits 64]
long_mode_start:
    ; setup segment registers
    mov ax, 0x0
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; transfer control to kernel
    extern kernel_main
    call kernel_main

    ; the kernel should never return, but if it does, halt the system
    hlt
    jmp $
