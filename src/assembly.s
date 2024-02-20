
[bits 64]
extern syscall_handler
global syscall_handler_asm_stub
; stack frame layout
; - 8 bytes: instruction_pointer
; - 8 bytes: code_segment
; - 8 bytes: cpu_flags
; - 8 bytes: stack_pointer
; - 8 bytes: stack_segment
syscall_handler_asm_stub:
    ; push complete state of the CPU
    push rbp
    push rdi
    push rsi
    push rdx
    push rcx
    push rbx
    push rax
    
    ; zero all register arguments
    mov rdi, 0
    mov rsi, 0
    mov rdx, 0
    mov rcx, 0
    mov r8, 0
    mov r9, 0

    ; InterruptStackFrame by value on the stack
    call syscall_handler

    ; make big zappzarapp if the syscall_handler returns
    jmp 0x0

extern irq_handler
; stack frame layout
; - 8 bytes: instruction_pointer
; - 8 bytes: code_segment
; - 8 bytes: cpu_flags
; - 8 bytes: stack_pointer
; - 8 bytes: stack_segment
irq_common:
    ; push complete state of the CPU
    push rbp
    push rdi
    push rsi
    push rdx
    push rcx
    push rbx
    push rax

    ; zero all register arguments
    mov rdi, 0
    mov rsi, 0
    mov rdx, 0
    mov rcx, 0
    mov r8, 0
    mov r9, 0

    ; InterruptStackFrame by value on the stack
    call irq_handler

    ; make big zappzarapp if the irq_handler returns
    jmp 0x0

global irq0
irq0:
    push byte 0
    jmp irq_common

global irq1
irq1:
    push byte 1
    jmp irq_common

global irq2
irq2:
    push byte 2
    jmp irq_common

global irq3
irq3:
    push byte 3
    jmp irq_common

global irq4
irq4:
    push byte 4
    jmp irq_common

global irq5
irq5:
    push byte 5
    jmp irq_common

global irq6
irq6:
    push byte 6
    jmp irq_common

global irq7
irq7:
    push byte 7
    jmp irq_common

global irq8
irq8:
    push byte 8
    jmp irq_common

global irq9
irq9:
    push byte 9
    jmp irq_common

global irq10
irq10:
    push byte 10
    jmp irq_common

global irq11
irq11:
    push byte 11
    jmp irq_common

global irq12
irq12:
    push byte 12
    jmp irq_common

global irq13
irq13:
    push byte 13
    jmp irq_common

global irq14
irq14:
    push byte 14
    jmp irq_common 

global irq15
irq15:
    push byte 15
    jmp irq_common

