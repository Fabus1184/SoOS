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
