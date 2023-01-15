%define ALIGN (1 << 0)
%define MEMINFO (1 << 1)
%define FLAGS (ALIGN | MEMINFO)
%define MAGIC 0x1BADB002
%define CHECKSUM -(MAGIC + FLAGS)

[bits 32]

; Multiboot header
section .multiboot
align 4
header_start:
	dd      MAGIC
	dd      FLAGS
	dd      CHECKSUM
header_end:

section .text
align 4

; gdt functions for gdt.c
global gdt_flush
[extern gp] 
gdt_flush:
    xor ax, ax
    mov ds, ax
    lgdt [gp]
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    jmp 0x08:gdt_flush_ret
gdt_flush_ret:
    ret            
