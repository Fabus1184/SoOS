; start of boot sector
[org 0x7c00]

; ------------------------------
; main entry point for BIOS
; ------------------------------
mov bx, MSG_BOOT_INIT
call print

; BIOS puts boot drive in dl
mov [BOOT_DRIVE], dl

; setup stack
mov bp, STACK_ADDR
mov sp, bp

; load kernel from disk and return
call load_kernel

; switch to protected mode, this will never return
call switch_pm

; ------------------------------



; ------------------------------
; print contents of bx
; resets ax
; ------------------------------
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

; ------------------------------



; ------------------------------
; load NUM_SECTORS to KERNEL_OFFSET
; resets bx, dx
; ------------------------------
load_kernel:
	mov bx, MSG_LOAD_KERNEL
	call print

    ; setup registers for disk_load
	mov bx, KERNEL_OFFSET
	mov dh, NUM_SECTORS
	mov dl, [BOOT_DRIVE]

	; setup interrupt, raise and return
	call disk_load

	mov bx, MSG_LOAD_KERNEL_FIN
	call print

	ret

; ------------------------------



; ------------------------------
; setup
; preservers registers
; ------------------------------
disk_load:
    ; push registers onto stack
	pusha

	push dx

    ; setup BIOS interrupt
	mov ah, 0x02
	mov al, dh
	mov cl, 0x02
	mov ch, 0x00
	mov dh, 0x00

	int 0x13

    ; carry flag is set on error
	jc disk_error

	pop dx

    ; number of sectors read is stored in dh
	cmp al, dh
	jne sectors_error

    ; restore registers
	popa
	ret

disk_error:
	mov bx, MSG_DISK_ERROR
	call print

	hlt

sectors_error:
	mov bx, MSG_SECTORS_ERROR
	call print

	hlt

; ------------------------------



; ------------------------------
; switch to 32-bit protected mode
; resets all registers
; ------------------------------
switch_pm:
    ; disable interrupts
	cli

	; load empty global descriptor table
	lgdt [gdt_descriptor]

    ; set 32-bit mode
	mov eax, cr0
	or eax, 0x1
	mov cr0, eax

    ; transfer execution to code in 32-bit mode
	jmp CODE_SEG:init_pm

; ------------------------------



; -----------------------------
; global descriptor table
; -----------------------------
; GDT Entry:
;
;   Bit | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
;
;   7     -   Present bit: has to be 1
;   6,5   -   Descriptor privilege level: 0 (highest) - 3 (lowest)
;   4     -   Descriptor type bit: if 0 it's a system segment, if 1 it's a code or data segment
;   3     -   Executable bit: if 0 it's data, if 1 it's code which can be executed
;   2     -   Direction bit: for data selectors: 0 (up), 1 (down)
;                          | for code if (1) it can be executed by equal or lower privilege level
;   1     -   Readable / Writable bit: {write, read} access is never allowed for {code, data} segments
;                                    | if (1) {read, write} access is allowed for {code, data} segments
;   0     -   Accessed bit: CPU will set to 1 if accessed
; ------------------------------
; ------------------------------
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

; ------------------------------



; ------------------------------
; initialize protected mode registers
; resets all registers
; ------------------------------
[bits 32]
init_pm:
    ; setup 32-bit register
	mov ax, DATA_SEG
	mov ds, ax
    mov ss, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; setup new stack
    mov ebp, STACK32_ADDR
    mov esp, ebp

    ; this will never return
    call BEGIN_PM

; ------------------------------



; ------------------------------
; call kernel in 32-bit mode
; resets ebx
; ------------------------------
BEGIN_PM:
	mov ebx, MSG_PROT_MODE
	call print_pm

    ; call the kernel (no jump so it could return)
	call KERNEL_OFFSET

    ; if the kernel ever returns it's probably an error
	mov ebx, MSG_KERNEL_RET
	call print_pm

	hlt

; ------------------------------



; ------------------------------
; print in protected mode
; preserves all registers
; ------------------------------
print_pm:
    ; push registers onto stack
	pusha

print_pm_loop:
    ; load address, color and char
	mov edx, [VIDEO_MEM]
	mov ah, WHITE_ON_BLACK
	mov al, [ebx]

	cmp al, 0
	je print_pm_end

    ; mov color and char to address
	mov [edx], ax

	; increment string pointer
	add ebx, 1

	; double increment VIDEO_MEM pointer
	add edx, 2

	mov [VIDEO_MEM], edx

	jmp print_pm_loop

print_pm_end:
	popa
	ret

; ------------------------------



; ------------------------------
; Variables (pointers)
; ------------------------------
BOOT_DRIVE:             db 0
VIDEO_MEM:              dd 0xb8000
MSG_BOOT_INIT:          db "Booting SoOS! ...", 13, 10, 0
MSG_LOAD_KERNEL:        db "Loading kernel from disk ...", 13, 10, 0
MSG_LOAD_KERNEL_FIN:    db "Loading kernel finished!", 13, 10, 0
MSG_DISK_ERROR:         db "Error: reading from disk failed!", 13, 10, 0
MSG_SECTORS_ERROR:      db "Error: couldn't read all sectors!", 13, 10, 0

MSG_PROT_MODE:          db "Entered 32-bit protected mode!"
; move cursor to next line in text mode
times (MSG_PROT_MODE + 80 - $) db 0x20
db 0

MSG_KERNEL_RET:         db "Error: kernel returned!", 0
; move cursor to next line in text mode
; doesnt fit into 512 bytes, so no newline after this message
; times (MSG_KERNEL_RET + 80 - $) db 0x20
; db 0

; ------------------------------



; ------------------------------
; Constants (values)
; ------------------------------
CODE_SEG        equ gdt_code - gdt_start
DATA_SEG        equ gdt_data - gdt_start

; has to be same as when linking in Makefile
KERNEL_OFFSET   equ 0x7e00
WHITE_ON_BLACK  equ 0x01

; this must be adjusted to the size of the kernel, each sector MAY OR MAY NOT BE 512 Bytes in size
NUM_SECTORS     equ 71

; this may have to be adjusted to accommodate detected RAM and / or special memory mappings
STACK_ADDR      equ 0x3000
STACK32_ADDR    equ 0x90000

; ------------------------------


; ------------------------------
; padding for exactly 512 Bytes
; ------------------------------
times 510 - ($-$$) db 0
dw 0xaa55

; ------------------------------
