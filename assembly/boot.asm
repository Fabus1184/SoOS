[org 0x7c00]

mov bx, MSG_BOOT_INIT
call print

mov [BOOT_DRIVE], dl

mov bp, STACK_ADDR
mov sp, bp

call load_kernel

call switch_pm

jmp $

print:
	pusha
	mov ah, 0x0e

print_loop:
	mov al, [bx]
	cmp al, 0
	je print_end

	int 0x10
	add bx, 1

	jmp print_loop

print_end:
	popa
	ret

load_kernel:
	mov bx, MSG_LOAD_KERNEL
	call print

	mov bx, KERNEL_OFFSET
	mov dh, NUM_SECTORS
	mov dl, [BOOT_DRIVE]
	call disk_load

	mov bx, MSG_LOAD_KERNEL_FIN
	call print

	ret

disk_load:
	pusha

	push dx

	mov ah, 0x02
	mov al, dh
	mov cl, 0x02
	mov ch, 0x00
	mov dh, 0x00

	int 0x13

	jc disk_error

	pop dx

	cmp al, dh
	jne sectors_error

	popa
	ret

disk_error:
	mov bx, MSG_DISK_ERROR
	call print

	jmp $

sectors_error:
	mov bx, MSG_SECTORS_ERROR
	call print

	jmp $

switch_pm:
	cli
	lgdt [gdt_descriptor]

	mov eax, cr0
	or eax, 0x1
	mov cr0, eax
	jmp CODE_SEG:init_pm

gdt_start:
	dd 0x0
	dd 0x0

gdt_code:
	dw 0xffff
	dw 0x0
	db 0x0
	db 10011010b
	db 11001111b
	db 0x0

gdt_data:
	dw 0xffff
	dw 0x0
	db 0x0
	db 10010010b
	db 11001111b
	db 0x0

gdt_end:

gdt_descriptor:
	dw gdt_end - gdt_start - 1
	dd gdt_start

[bits 32]
init_pm:
	mov ax, DATA_SEG
	mov ds, ax
    mov ss, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    mov ebp, 0x90000
    mov esp, ebp

    call BEGIN_PM

BEGIN_PM:
	mov ebx, MSG_PROT_MODE
	call print_pm

	call KERNEL_OFFSET

	mov ebx, MSG_KERNEL_RET
	call print_pm

	jmp $

print_pm:
	pusha

print_pm_loop:
	mov edx, [VIDEO_MEM]
	mov ah, WHITE_ON_BLACK
	mov al, [ebx]

	cmp al, 0
	je print_pm_end

	mov [edx], ax
	add ebx, 1
	add edx, 2

	mov [VIDEO_MEM], edx

	jmp print_pm_loop

print_pm_end:
	popa
	ret

BOOT_DRIVE: db 0
VIDEO_MEM: dd 0xb8000
MSG_BOOT_INIT: db "Booting SoOS! ...", 13, 10, 0
MSG_LOAD_KERNEL: db "Loading kernel from disk ...", 13, 10, 0
MSG_LOAD_KERNEL_FIN: db "Loading kernel finished!", 13, 10, 0
MSG_DISK_ERROR: db "Error: reading from disk failed!", 13, 10, 0
MSG_SECTORS_ERROR: db "Error: couldn't read all sectors!", 13, 10, 0

MSG_PROT_MODE: db "Entered 32-bit protected mode!"
times (MSG_PROT_MODE + 80 - $) db 0x20
db 0

MSG_KERNEL_RET: db "Error: kernel returned!"
; doesnt fit into 512 bytes
;times (MSG_KERNEL_RET + 80 - $) db 0x20
;db 0

KERNEL_OFFSET equ 0x7e00
WHITE_ON_BLACK equ 0x01
CODE_SEG equ gdt_code - gdt_start
DATA_SEG equ gdt_data - gdt_start
NUM_SECTORS equ 71
STACK_ADDR equ 0x3000

times 510 - ($-$$) db 0
dw 0xaa55
