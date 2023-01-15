[bits 16]

section .lowmem
align 4

; error16: print a string to the screen and halt, string is passed in eax, color is set in ch
global error16
error16:
    call print_string
    
    ; halt
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
