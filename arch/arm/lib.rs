#![no_std]

// The pre-`kernel_main` boot code. Kept in a real `.S` file so it reads like
// assembly instead of a Rust string literal; `compile_data` in BUILD is what
// makes it visible to `include_str!`.
core::arch::global_asm!(include_str!("head.S"));
