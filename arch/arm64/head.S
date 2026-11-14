.section .text.boot

.global _start
_start:
    mrs x0, mpidr_el1
    and x0, x0, #0xff
    cbz x0, master
    b proc_hang

master:
    adr x0, bss_begin
    adr x1, bss_end
    sub x1, x1, x0
    bl memzero
    ldr x0, =_stack_top
    mov sp, x0
    bl kernel_main
    b proc_hang


memzero:
    str xzr, [x0], #8
    subs x1, x1, #8
    b.gt memzero
    ret

.global memzero, proc_hang
proc_hang:
    b proc_hang

.global my_atomic_write
my_atomic_write:
    adr x6, my_data
1:
    ldxr x2, [x6]
    mov x2, x0
    stxr w3, x2, [x6]
    cbnz w3, 1b
    
    mov x0, x2
    ret

.align 3
.global my_data
my_data:
    .dword 0x0
