
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
    push r15
    push r14
    push r13
    push r12
    push r11
    push r10
    push r9
    push r8
    push rbp
    push rsp
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
    push r15
    push r14
    push r13
    push r12
    push r11
    push r10
    push r9
    push r8
    push rbp
    push rsp
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

; #[repr(C)]
; #[derive(Debug, Copy, Clone, Default)]
; pub struct GPRegisters {
;     pub rax: u64,
;     pub rbx: u64,
;     pub rcx: u64,
;     pub rdx: u64,
;     pub rsi: u64,
;     pub rdi: u64,
;     pub rsp: u64,
;     pub rbp: u64,
;     pub r8: u64,
;     pub r9: u64,
;     pub r10: u64,
;     pub r11: u64,
;     pub r12: u64,
;     pub r13: u64,
;     pub r14: u64,
;     pub r15: u64,
; }
; fn do_iret(cs: u64, ds: u64, flags: u64, rip: u64, regs: *const GPRegisters) -> !;
;                rdi    , rsi    , rdx       , rcx     , r8
global do_iret
do_iret:
    ; setup iretq

    ; push data segment
    push rsi
    ; push rsp from regs
    push qword [r8 + (8 * 6)]
    ; push cpu flags
    push rdx
    ; push code segment
    push rdi
    ; push instruction pointer
    push rcx
    ; setup stack segment
    mov ax, si
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; restore registers
    mov rax, [r8 + (8 * 0)]
    mov rbx, [r8 + (8 * 1)]
    mov rcx, [r8 + (8 * 2)]
    mov rdx, [r8 + (8 * 3)]
    mov rsi, [r8 + (8 * 4)]
    mov rdi, [r8 + (8 * 5)]
    ; rsp will be restored by iretq
    ; mov rsp, [r8 + (8 * 6)]
    mov rbp, [r8 + (8 * 7)]
    ; omit r8
    mov r9, [r8 + (8 * 9)]
    mov r10, [r8 + (8 * 10)]
    mov r11, [r8 + (8 * 11)]
    mov r12, [r8 + (8 * 12)]
    mov r13, [r8 + (8 * 13)]
    mov r14, [r8 + (8 * 14)]
    mov r15, [r8 + (8 * 15)]
    ; r8 here
    mov r8, [r8 + (8 * 8)]

    iretq

