load("@rules_rust//rust:defs.bzl", "rust_library")

package(default_visibility = ["//kernel:__pkg__"])

# Boot code for aarch64-unknown-none. head.S is still empty — nothing depends
# on this target until a board in //boards:boards.bzl claims the triple.
#
# `crate_name` is deliberately "arch" rather than "arm64": every arch package
# exports the same crate name, so kernel/boot.rs can pull it in with a single
# `use arch as _;` regardless of which one //tools:kernel.bzl selected.
rust_library(
    name = "arm64",
    srcs = ["lib.rs"],
    compile_data = ["head.S"],
    crate_name = "arch",
)
