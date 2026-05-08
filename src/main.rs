#![no_std]
#![no_main]

mod uart;

use core::arch::global_asm;
use panic_halt as _;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    let uart = unsafe { uart::Uart::new() };
    uart.init();
    uart.puts("Welcome BenOS!\n");
    loop {
        let c = uart.getc();
        uart.putc(c);
    }
}

global_asm!("
    .section \".text.boot\"

    .global _start, memzero
    _start:
        mrs x0, mpidr_el1
        // 检查处理器ID
        and x0, x0, #0xff
        cbz x0, master
        b proc_hang
    
    proc_hang:
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
");