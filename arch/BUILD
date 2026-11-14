load("@rules_rust//rust:defs.bzl", "rust_library")

package(default_visibility = ["//kernel:__pkg__"])

# A single architecture crate for every bare-metal target. The target's
# built-in `target_arch` cfg selects the implementation in lib.rs.
rust_library(
    name = "arch",
    srcs = [
        "lib.rs",
        "arm/lib.rs",
        "arm64/lib.rs",
    ],
    compile_data = [
        "arm/head.S",
        "arm64/head.S",
    ],
    crate_name = "arch",
    deps = select({
        "//platforms/settings:armv7a-none-eabi": [
            "@crates//:aarch32-cpu",
        ],
        "//platforms/settings:aarch64-unknown-none": [
            "@crates//:aarch64-cpu",
        ],
        "//platforms/settings:aarch64-apple-darwin": [
            "@crates//:aarch64-cpu",
        ],
        "//conditions:default": [],
    }),
)
