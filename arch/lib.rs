#![no_std]

#[cfg(target_arch = "arm")]
#[path = "arm/lib.rs"]
mod arm;
#[cfg(target_arch = "arm")]
#[allow(unused_imports)]
pub use arm::*;

#[cfg(target_arch = "aarch64")]
#[path = "arm64/lib.rs"]
mod arm64;
#[cfg(target_arch = "aarch64")]
#[allow(unused_imports)]
pub use arm64::*;

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
compile_error!("BenOS only supports the arm and aarch64 architectures");
