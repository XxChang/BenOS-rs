.section .text.boot

.global _start
_start:
    b master

master:
    ldr r0, =bss_begin
    ldr r1, =bss_end
    sub r1, r1, r0
    bl memzero
    ldr r0, =_stack_top
    mov sp, r0
    bl kernel_main
    b proc_hang


memzero:
    cmp r1, #0
    beq memzero_done
    mov r2, #0
memzero_loop:
    str r2, [r0], #4
    subs r1, r1, #4
    bgt memzero_loop
memzero_done:
    bx lr

.global memzero, proc_hang
proc_hang:
    b proc_hang
