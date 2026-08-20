#![no_std]
#![no_main]

mod uart;

// Pulls in `_start` and friends from the arch package //tools:kernel.bzl picked
// for this board. Every arch package is named `arch`, so this line is the same
// for all of them.
use arch as _;
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
