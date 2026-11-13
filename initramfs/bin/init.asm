default rel

section .rodata
    ; Test Read-Only segment (PF_R)
    magic_seed  dq 0x19450817CAFEBABE
    xor_key     dq 0x0F0F0F0F55555555

section .data
    ; Test Read-Write segment (PF_R | PF_W)
    counter     dq 0
    accumulator dq 0x1122334455667788

section .bss
    ; Test Uninitialized Memory segment (p_memsz > p_filesz)
    ; Must all be 0 when the program starts!
    buffer      resq 64     ; 512 bytes (64 qwords)

section .text
global _start

_start:
    ; Save callee-saved registers according to System V AMD64 ABI
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    push r15

    ; ==========================================================
    ; TEST 1: BSS Zeroing Verification
    ; If the loader fails to zero out BSS, the program immediately
    ; returns with error code 0xDEAD0001
    ; ==========================================================
    lea rdi, [buffer]
    mov rcx, 64
.verify_bss:
    mov rax, [rdi]
    test rax, rax
    jnz .bss_corrupted
    add rdi, 8
    dec rcx
    jnz .verify_bss

    ; ==========================================================
    ; TEST 2: Fill Buffer & Manipulate .data + .rodata Segments
    ; ==========================================================
    mov r12, [magic_seed]       ; Read from .rodata
    mov r13, [xor_key]          ; Read from .rodata
    lea r14, [buffer]           ; Pointer to .bss
    xor r15, r15                ; Loop index (0..63)

.fill_loop:
    ; Calculate pseudo-random value: (seed ^ index) + key
    mov rax, r12
    xor rax, r15
    add rax, r13
    rol rax, 7                  ; Bitwise rotate left

    ; Write to .bss buffer
    mov [r14 + r15 * 8], rax

    ; Update global variables in .data
    add [accumulator], rax
    inc qword [counter]

    inc r15
    cmp r15, 64
    jl .fill_loop

    ; ==========================================================
    ; TEST 3: Recursion & Stack Stress Test
    ; Tests chained push/pop operations during timer interrupt preemption
    ; ==========================================================
    mov rdi, 12                 ; Compute the 12th Fibonacci number recursively
    call fib_recursive
    mov rbx, rax                ; Save result fib(12) = 144 (0x90) in rbx

    ; ==========================================================
    ; TEST 4: CPU Register Churn & Fold Checksum
    ; Ensures no registers leak or are swapped across context switches
    ; ==========================================================
    lea rsi, [buffer]
    mov rcx, 64
    mov r8, [accumulator]
    mov r9, [counter]

.checksum_loop:
    mov r10, [rsi]
    xor r8, r10
    ror r8, 3
    add rsi, 8
    dec rcx
    jnz .checksum_loop

    ; Combine checksum (.bss), counter (.data), and fib(12) (stack)
    add r8, r9
    add r8, rbx

    ; Place final result in RAX before returning to kernel exit::exit
    mov rax, r8
    jmp .cleanup

.bss_corrupted:
    mov rax, 0xDEAD0001

.cleanup:
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

; ----------------------------------------------------------
; Recursive Function: uint64_t fib_recursive(uint64_t n)
; Input : RDI = n
; Output: RAX = fib(n)
; ----------------------------------------------------------
fib_recursive:
    cmp rdi, 1
    jle .base_case

    push rbx
    push r12

    mov r12, rdi        ; Save n

    dec rdi             ; n - 1
    call fib_recursive
    mov rbx, rax        ; Save fib(n - 1)

    lea rdi, [r12 - 2]  ; n - 2
    call fib_recursive
    add rax, rbx        ; fib(n - 1) + fib(n - 2)

    pop r12
    pop rbx
    ret

.base_case:
    mov rax, rdi
    ret
