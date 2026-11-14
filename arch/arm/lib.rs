use aarch32_cpu::*;

// The pre-`kernel_main` boot code. Kept in a real `.S` file so it reads like
// assembly instead of a Rust string literal; `compile_data` on //arch:arch is
// what makes it visible to `include_str!`.
core::arch::global_asm!(include_str!("head.S"));

#[inline]
pub fn interrupt_disable() {
  aarch32_cpu::interrupt::disable();
}

#[inline]
pub fn interrupt_enable() {
  unsafe {
    aarch32_cpu::interrupt::enable();
  }
}