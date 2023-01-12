%define ALIGN (1 << 0)
%define MEMINFO (1 << 1)
%define FLAGS (ALIGN | MEMINFO)
%define MAGIC 0x1BADB002
%define CHECKSUM -(MAGIC + FLAGS)

section .multiboot
align 4
header_start:
	dd MAGIC
	dd FLAGS
	dd CHECKSUM
header_end:

section .bss
align 16
kernel_stack:
	resb 32768
kernel_stack_end:

section .text
bits 32
global _start
extern kernel_main

_start:
	mov esp, kernel_stack_end

	call kernel_main
 
	cli
_1:
	hlt
	jmp _1
