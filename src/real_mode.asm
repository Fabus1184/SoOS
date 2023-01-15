[bits 16]
section .lowmem
align 4

global protected_mode16
protected_mode16:
    ; disable interrupts
    cli

    ; set up segment registers
    extern GDT16_DATA
    mov ax, GDT16_DATA
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
 
    ; disable protected mode
    mov eax, cr0
    and eax, ~0x1
    mov cr0, eax

    ; far jump to 16 bit real mode
    jmp 0x0:real_mode16

; 16 bit real mode ivt
isr:
    iret

ivt16:
    times 256 dd isr

; 16 bit real mode idt
idt16:
    dw 0x3ff
    dd ivt16

real_mode16:
    ; set up segment registers
    mov ax, 0x0
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; setup stack
    mov sp, stack16_end

    ; load real mode idt
    lidt [idt16]
   
    ; enable interrupts
    sti

    ; setup video mode
    extern setup_video
    call setup_video

    ; execute neccessary tasks in real mode

    ; print "Hello, world!"
    mov byte ch, 0x0a ; green
    mov dword eax, hello_world
    extern print_string
    call print_string

    ; detect memory
    extern detect_memory
    call detect_memory

    ; switch back to protected mode
    
    ; disable interrupts
    cli

    ; load protected mode gdt
    extern gdt32_descriptor
    lgdt [gdt32_descriptor]

    ; enable protected mode
    mov eax, cr0
    or eax, 0x1
    mov cr0, eax

    ; far jump to 32 bit protected mode
    extern GDT32_CODE
    jmp GDT32_CODE:protected_mode32_

stack16:
    times 0x1000 db 0x0
stack16_end:

hello_world:
    db "Hello, world! from 16 bit real mode", 0x0

[bits 32]
; has to be in lowmem because of the far jump
protected_mode32_:
    extern protected_mode32
    jmp protected_mode32
