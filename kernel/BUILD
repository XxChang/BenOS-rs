load("//boards:boards.bzl", "BOARDS")
load("//tools:kernel.bzl", "kernel_image")

package(default_visibility = ["//visibility:public"])

# One ELF (`//kernel:<board>`) per board, plus a QEMU launcher
# (`//kernel:<board>.run`) for the boards that can be simulated.
# `bazel build //kernel/...` cross-compiles every board at once.
[
    kernel_image(
        name = board,
        srcs = [
            "boot.rs",
            "uart.rs",
        ],
        crate_root = "boot.rs",
        deps = [
            "@crates//:panic-halt",
            "@crates//:tock-registers",
        ],
    )
    for board in BOARDS
]
