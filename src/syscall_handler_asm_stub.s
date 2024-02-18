; /// Represents the interrupt stack frame pushed by the CPU on interrupt or exception entry.
; #[derive(Clone, Copy)]
; #[repr(C)]
; pub struct InterruptStackFrameValue {
;     /// This value points to the instruction that should be executed when the interrupt
;     /// handler returns. For most interrupts, this value points to the instruction immediately
;     /// following the last executed instruction. However, for some exceptions (e.g., page faults),
;     /// this value points to the faulting instruction, so that the instruction is restarted on
;     /// return. See the documentation of the [`InterruptDescriptorTable`] fields for more details.
;     pub instruction_pointer: VirtAddr,
;     /// The code segment selector, padded with zeros.
;     pub code_segment: u64,
;     /// The flags register before the interrupt handler was invoked.
;     pub cpu_flags: u64,
;     /// The stack pointer at the time of the interrupt.
;     pub stack_pointer: VirtAddr,
;     /// The stack segment descriptor at the time of the interrupt (often zero in 64-bit mode).
;     pub stack_segment: u64,
; }

; pub extern "C" fn syscall_handler(rax: u64, rbx: u64, rcx: u64, stack_frame: InterruptStackFrame) {

; stack frame layout
; - 8 bytes: instruction_pointer
; - 8 bytes: code_segment
; - 8 bytes: cpu_flags
; - 8 bytes: stack_pointer
; - 8 bytes: stack_segment

[bits 64]
extern syscall_handler
global syscall_handler_asm_stub
syscall_handler_asm_stub:
    ; first 3 arguments
    mov rdi, rax ; rax
    mov rsi, rbx ; rbx
    mov rdx, rcx ; rcx
    ; InterruptStackFrame by value on the stack
    call syscall_handler 

    ; halt if syscall_handler returns
    cli
    hlt
    jmp $

