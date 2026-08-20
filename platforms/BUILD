# Generated from //boards:boards.bzl — do not add platforms by hand.
#
# One platform per *triple*, not per board: a Bazel platform describes what the
# code is compiled for, and board identity is carried by `--cfg target_board`
# instead (see //tools:kernel.bzl).
load("//boards:boards.bzl", "TRIPLES")

package(default_visibility = ["//visibility:public"])

[
    platform(
        name = triple,
        constraint_values = defn.constraints,
    )
    for triple, defn in TRIPLES.items()
]
