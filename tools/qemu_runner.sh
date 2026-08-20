#!/bin/sh
# QEMU launcher for BenOS kernel images, invoked through `bazel run`.
#
# The first three args are supplied by the `kernel_image` macro and are always
# non-empty — Bazel's `args` attribute is shell-tokenized, so an empty string
# would silently vanish and shift everything after it. Board-specific QEMU
# flags follow as a pre-assembled list, and anything passed after `--` on the
# bazel run command line lands at the end and is forwarded verbatim.
set -eu

qemu="$1"; display="$2"; kernel="$3"
shift 3

set -- -serial mon:stdio -display "${QEMU_DISPLAY:-$display}" \
       -kernel "$kernel" "$@"

if [ "${QEMU_GDB:-0}" = "1" ]; then
    set -- -s -S "$@"
fi

export QEMU_AUDIO_DRV=none
exec "$qemu" "$@"
