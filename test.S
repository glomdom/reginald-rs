.global _start

_start:

loop:
    mov $0x69420, %eax
    jmp loop