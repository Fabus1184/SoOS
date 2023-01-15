section .bss
align 4

stack32:
    resb 0x2000
stack32_end:

stack64:
    resb 0x2000
stack64_end:

[bits 32]
section .text
align 4

; entry point where the bootloader jumps to
global _start
_start:
    ; disable interrupts
    cli

    ; load 16 bit gdt
    extern gdt16_descriptor
    lgdt [gdt16_descriptor]
    
    ; far jump to 16 bit protected mode
    extern GDT16_CODE
    extern protected_mode16
    jmp GDT16_CODE:protected_mode16

; resume in 32 bit protected mode at text section
global protected_mode32
protected_mode32:
    ; set up stack
    mov esp, stack32_end

    ; set up segment registers
    extern GDT32_DATA
    mov eax, GDT32_DATA
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
    ; set up stack
    mov rsp, stack64_end

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
