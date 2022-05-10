; ------------------------------
; kernel entry: the linker will use this as the entry point to our kernel,
; boot.asm will call the address of this
; ------------------------------
[bits 32]

; linker will know where kmain is
; implemented in kernel.c
[extern kmain]

; declare global for gcc to find
global _start

; call kmain
_start:
	call kmain

; -------------------------------------