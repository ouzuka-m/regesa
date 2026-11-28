default rel
global _start

section .text
    _start:
        syscall
        jmp loop

    loop:
        jmp loop
