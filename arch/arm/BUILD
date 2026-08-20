load("@rules_rust//rust:defs.bzl", "rust_library")

package(default_visibility = ["//kernel:__pkg__"])

# Boot code for armv7a-none-eabi.
#
# `crate_name` is deliberately "arch" rather than "arm": every arch package
# exports the same crate name, so kernel/boot.rs can pull it in with a single
# `use arch as _;` regardless of which one //tools:kernel.bzl selected.
rust_library(
    name = "arm",
    srcs = ["lib.rs"],
    compile_data = ["head.S"],
    crate_name = "arch",
)
