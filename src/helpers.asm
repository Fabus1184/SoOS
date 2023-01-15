[bits 16]
section .lowmem
align 4

video_mem:
    dd 0xB8000

global setup_video
setup_video:
    ; set text mode
    mov ah, 0x00
    mov al, 0x26
    clc
    int 0x10
    jc setup_video_error

    ret

setup_video_error:
    mov eax, setup_video_error_msg
    mov ch, 0x0C ; red on black
    call error16 

setup_video_error_msg:
    db "Error setting video mode", 0

; error16: print a string to the screen and halt, string is passed in eax, color is set in ch
global error16
error16:
    call print_string

    ; halt
    hlt
    jmp $

global print_string
print_string:
    mov dword ebx, [video_mem]

print_string_loop:
    ; read first byte of string
    mov byte cl, [eax]
    cmp byte cl, 0
    je print_string_end
    
    mov word [ebx], cx
    inc dword eax
    add dword ebx, 2
    jmp print_string_loop

print_string_end:
    ; advance to next line
    mov dword ebx, [video_mem]
    add dword ebx, 160
    mov dword [video_mem], ebx

    ret
