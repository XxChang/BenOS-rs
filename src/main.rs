#![no_std]
#![no_main]

#[cfg(target_board = "versatilepb")]
mod lcd;
mod uart;

use core::arch::global_asm;
use panic_halt as _;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    let uart = unsafe { uart::Uart::new() };
    uart.init();
    uart.puts("Welcome BenOS!\n");

    #[cfg(target_board = "versatilepb")]
    {
        let lcd = unsafe { lcd::Lcd::new() };
        lcd.init();
        lcd::init_printing(lcd, lcd::Rgb565::WHITE, lcd::Rgb565::BLACK);
        lcd_println!("BenOS PL110 LCD");
    }

    loop {
        let c = uart.getc();
        uart.putc(c);
    }
}

#[cfg(target_board = "raspi4b")]
global_asm!(
    "
    .section \".text.boot\"

    .global _start, memzero
    _start:
        mrs x0, mpidr_el1
        // 检查处理器ID
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
"
);

#[cfg(target_board = "versatilepb")]
global_asm!(
    "
    .section \".text.boot\"

    .global _start
    _start:
        // VersatilePB只有一个处理器，直接跳到master
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
"
);

global_asm!(
    "
    .global memzero, proc_hang
    proc_hang:
        b proc_hang
"
);
