# Generated from //boards:boards.bzl — do not add settings by hand.
#
# The `select()` branches in crate_universe's generated BUILD files resolve
# here: target names must equal the rust target triple exactly, because
# `crate.render_config(platforms_template = ...)` in MODULE.bazel points at
# `@@//platforms/settings:{triple}`.
#
# These are separate from the `platform()` targets in //platforms because a
# platform and a config_setting cannot share a name within one package.
load("//boards:boards.bzl", "TRIPLES")

package(default_visibility = ["//visibility:public"])

[
    config_setting(
        name = triple,
        constraint_values = defn.constraints,
    )
    for triple, defn in TRIPLES.items()
]
