[bits 16]
section .lowmem
align 4

; place to store the information for later access from the kernel
global low_memory
low_memory:
    dw 0
global high_memory
high_memory:
    times 256 dq 0 0
global high_memory_size
high_memory_size:
    dw 0

detect_memory_msg:
    db "Detecting memory...", 0
detect_memory_success_msg:
    db "Successfully detected memory!", 0

global detect_memory
detect_memory:
    ; print message
    mov byte ch, 0x0E ; light gray
    mov eax, detect_memory_msg
    extern print_string
    call print_string

    ; detect low memory
    clc
    int 0x12
    jc error_low_memory
    mov word [low_memory], ax

    ; detect high memory
    xor ebx, ebx
    mov dword edx, 0x534D4150
    mov dword eax, 0xE820
    mov word di, high_memory
    mov dword ecx, 0x24

high_memory_loop:
    clc
    int 0x15

    ; check if we got a valid memory map
    jc error_high_memory

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

    ; print success message
    mov byte ch, 0x0A ; light green
    mov eax, detect_memory_success_msg
    call print_string

    ret

error_low_memory:
    mov eax, error_low_memory_msg
    mov byte ch, 0x0C
    extern error16
    jmp error16

error_high_memory:
    mov eax, error_high_memory_msg
    mov byte ch, 0x0C
    extern error16
    jmp error16

; error messages
error_low_memory_msg:
    db "Error detecting low memory!", 0
error_high_memory_msg:
    db "Error detecting high memory!", 0
