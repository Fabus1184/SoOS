; ------------------------------
; define ISRs (Interrupt Service Routines) & IRQs (Interrupt ReQuests)
; provides ISR & IRQ-Handlers
; ------------------------------

; these are defined in isr.c
[extern isr_handler]
[extern irq_handler]

; ------------------------------
; ISR handler
; preserves all registers
; pushes pointer to registers_t struct onto stack
; ------------------------------
isr_common_stub:
    ; push registers onto stack
	pusha

	mov ax, ds
	push eax
	mov ax, 0x10
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax

	; registers_t *r parameter in isr_handler.c
	push esp

    ; sysV ABI requires df (direction flag) to be clear on function entry
    cld

    ; call C function
	call isr_handler

    ; restore registers from stack
	pop eax
    pop eax
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax
    popa

	; cleans up the pushed error code and pushed ISR number
	add esp, 8

	; pops cs, eip, eflags, ss, and esp
	iret
; ------------------------------



; ------------------------------
; IRQ handler
; ------------------------------
irq_common_stub:
    ; push registers onto stack
    pusha
    mov ax, ds
    push eax
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; registers_t *r parameter in isr_handler.c
    push esp

    ; sysV ABI requires df (direction flag) to be clear on function entry
    cld

    ; call C function
    call irq_handler
    pop ebx
    pop ebx
    mov ds, bx
    mov es, bx
    mov fs, bx
    mov gs, bx
    popa

    ; cleans up the pushed IRQ number
    add esp, 8

    ; pops cs, eip, eflags, ss, and esp
    iret

; ------------------------------


; isrN & irqN are accessed by isr.c
global isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7, isr8, isr9, isr10, isr11, isr12, isr13, isr14, isr15, isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23, isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31
global irq0, irq1, irq2, irq3, irq4, irq5, irq6, irq7, irq8, irq9, irq10, irq11, irq12, irq13, irq14, irq15

; 0: Divide By Zero Exception
isr0:
    push byte 0
    push byte 0
    jmp isr_common_stub

; 1: Debug Exception
isr1:
    push byte 0
    push byte 1
    jmp isr_common_stub

; 2: Non Maskable Interrupt Exception
isr2:
    push byte 0
    push byte 2
    jmp isr_common_stub

; 3: Int 3 Exception
isr3:
    push byte 0
    push byte 3
    jmp isr_common_stub

; 4: INTO Exception
isr4:
    push byte 0
    push byte 4
    jmp isr_common_stub

; 5: Out of Bounds Exception
isr5:
    push byte 0
    push byte 5
    jmp isr_common_stub

; 6: Invalid Opcode Exception
isr6:
    push byte 0
    push byte 6
    jmp isr_common_stub

; 7: Coprocessor Not Available Exception
isr7:
    push byte 0
    push byte 7
    jmp isr_common_stub

; 8: Double Fault Exception (With Error Code!)
isr8:
    push byte 8
    jmp isr_common_stub

; 9: Coprocessor Segment Overrun Exception
isr9:
    push byte 0
    push byte 9
    jmp isr_common_stub

; 10: Bad TSS Exception (With Error Code!)
isr10:
    push byte 10
    jmp isr_common_stub

; 11: Segment Not Present Exception (With Error Code!)
isr11:
    push byte 11
    jmp isr_common_stub

; 12: Stack Fault Exception (With Error Code!)
isr12:
    push byte 12
    jmp isr_common_stub

; 13: General Protection Fault Exception (With Error Code!)
isr13:
    push byte 13
    jmp isr_common_stub

; 14: Page Fault Exception (With Error Code!)
isr14:
    push byte 14
    jmp isr_common_stub

; 15: Reserved Exception
isr15:
    push byte 0
    push byte 15
    jmp isr_common_stub

; 16: Floating Point Exception
isr16:
    push byte 0
    push byte 16
    jmp isr_common_stub

; 17: Alignment Check Exception
isr17:
    push byte 0
    push byte 17
    jmp isr_common_stub

; 18: Machine Check Exception
isr18:
    push byte 0
    push byte 18
    jmp isr_common_stub

; 19: Reserved
isr19:
    push byte 0
    push byte 19
    jmp isr_common_stub

; 20: Reserved
isr20:
    push byte 0
    push byte 20
    jmp isr_common_stub

; 21: Reserved
isr21:
    push byte 0
    push byte 21
    jmp isr_common_stub

; 22: Reserved
isr22:
    push byte 0
    push byte 22
    jmp isr_common_stub

; 23: Reserved
isr23:
    push byte 0
    push byte 23
    jmp isr_common_stub

; 24: Reserved
isr24:
    push byte 0
    push byte 24
    jmp isr_common_stub

; 25: Reserved
isr25:
    push byte 0
    push byte 25
    jmp isr_common_stub

; 26: Reserved
isr26:
    push byte 0
    push byte 26
    jmp isr_common_stub

; 27: Reserved
isr27:
    push byte 0
    push byte 27
    jmp isr_common_stub

; 28: Reserved
isr28:
    push byte 0
    push byte 28
    jmp isr_common_stub

; 29: Reserved
isr29:
    push byte 0
    push byte 29
    jmp isr_common_stub

; 30: Reserved
isr30:
    push byte 0
    push byte 30
    jmp isr_common_stub

; 31: Reserved
isr31:
    push byte 0
    push byte 31
    jmp isr_common_stub

; ------------------------------

; 0: system timer
irq0:
    push byte 0
	push byte 32
	jmp irq_common_stub

; 1: keyboard controller
irq1:
    push byte 1
	push byte 33
	jmp irq_common_stub

; 2: Internal
irq2:
    push byte 2
	push byte 34
	jmp irq_common_stub

; 3: Serial Port 1
irq3:
    push byte 3
	push byte 35
	jmp irq_common_stub

; 4: Serial Port 2
irq4:
    push byte 4
	push byte 36
	jmp irq_common_stub

; 5: Parallel Port 2 & 3 or Soundcard
irq5:
    push byte 5
	push byte 37
	jmp irq_common_stub

; 6: Floppy Disk
irq6:
    push byte 6
	push byte 38
	jmp irq_common_stub

; 7: Parallel Port 1, Printer or secondary Soundcard
irq7:
    push byte 7
	push byte 39
	jmp irq_common_stub

; 8: RTC
irq8:
    push byte 8
	push byte 40
	jmp irq_common_stub

; 9: on Intel ACPI, else anything
irq9:
    push byte 9
	push byte 41
	jmp irq_common_stub

; 10: Any PCI / SCSI / NIC
irq10:
    push byte 10
	push byte 42
	jmp irq_common_stub

; 11: Any PCI / SCSI / NIC
irq11:
    push byte 11
	push byte 43
	jmp irq_common_stub

; 12: PS2 Mouse, may be emulated by USB Mouse
irq12:
    push byte 12
	push byte 44
	jmp irq_common_stub

; 13: FPU
irq13:
    push byte 13
	push byte 45
	jmp irq_common_stub

; 14: Primary ATA
irq14:
    push byte 14
	push byte 46
	jmp irq_common_stub

; 15: Secondary ATA
irq15:
    push byte 15
	push byte 47
	jmp irq_common_stub
