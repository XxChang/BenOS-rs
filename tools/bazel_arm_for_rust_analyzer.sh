#!/bin/sh

# `bazel query` does not accept configuration options. Flycheck only uses it
# to map a saved source file to a label, so that lookup is configuration-free.
for arg do
    if [ "$arg" = "query" ]; then
        exec bazel "$@"
    fi
done

# Keep rust-analyzer's on-save Bazel diagnostics on the same target as its
# discovered crate graph. Bazel build/info accept command options at the end.
exec bazel "$@" --platforms=//platforms:armv7a-none-eabi
