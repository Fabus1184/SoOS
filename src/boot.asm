%define ALIGN (1 << 0)
%define MEMINFO (1 << 1)
%define FLAGS (ALIGN | MEMINFO)
%define MAGIC 0x1BADB002
%define CHECKSUM -(MAGIC + FLAGS)

[BITS 32]
[extern kernel_main]
[extern pre_kernel]

section .multiboot
align 4
header_start:
	dd      MAGIC
	dd      FLAGS
	dd      CHECKSUM
header_end:

section .bss
align 16
kernel_stack:
    resb	32768
kernel_stack_end:

section .text
align 4

global _start
_start:
    call    pre_kernel

global boot_kernel
boot_kernel:
	mov     esp, kernel_stack_end
    call    kernel_main
    hlt
_1:
    jmp     _1

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
