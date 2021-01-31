; use 64-bit mode
bits 64

; export entry point
global _start

; code section
section .text

_start:
  mov rax, 1        ; system call 4: write
  mov rdi, 1        ; file descriptor 1: stdout
  mov rsi, msg      ; string to write
  mov rdx, msglen   ; length of string
  syscall           ; execute system call

  mov rax, 60       ; system call 60: exit
  mov rdi, 0        ;
  syscall

; data section
section .rodata

  msg: db "Hello, world!", 10
  msglen: equ $ - msg
