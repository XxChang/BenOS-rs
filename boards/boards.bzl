"""Single source of truth for the target triples and boards BenOS supports.

Adding a board is one entry in `BOARDS` plus its linker script. Everything
downstream — the `platform()` targets, the `config_setting()`s crate_universe
keys its `select()`s on, the arch package the kernel links against, the kernel
ELF, the QEMU launcher — is generated from these two tables.
"""

def _triple(constraints, arch = None):
    """Describes one rust target triple.

    Args:
        constraints: the constraint_values that identify this triple, used for
            both its `platform()` and its `config_setting()`.
        arch: label of the arch package supplying boot code (`head.S`) for this
            triple, or `None` for triples no board targets.

    Returns:
        A struct for the `TRIPLES` table.
    """
    return struct(constraints = constraints, arch = arch)

# Triple -> definition.
#
# This grows with *architectures*, not with board count: a hundred boards
# realistically share a handful of triples.
#
# Keep in sync with `rust.repository_set` and `supported_platform_triples` in
# MODULE.bazel. That duplication is unavoidable — MODULE.bazel is restricted
# Starlark and cannot `load()` this file.
TRIPLES = {
    "armv7a-none-eabi": _triple(
        constraints = [
            "@platforms//cpu:armv7",
            "@platforms//os:none",
        ],
        arch = "//arch/arm",
    ),
    "aarch64-unknown-none": _triple(
        constraints = [
            "@platforms//cpu:aarch64",
            "@platforms//os:none",
        ],
        arch = "//arch/arm64",
    ),
    # The host. Present so crate_universe can resolve build scripts and proc
    # macros; no board targets it, so it needs no arch package.
    "aarch64-apple-darwin": _triple(
        constraints = [
            "@platforms//cpu:aarch64",
            "@platforms//os:macos",
        ],
    ),
}

def _qemu(binary, machine, cpu, memory = "", display = "none"):
    """Describes how to boot a board under QEMU.

    Args:
        binary: QEMU executable to launch.
        machine: QEMU `-M` value.
        cpu: QEMU `-cpu` value.
        memory: QEMU `-m` value; empty when the machine has fixed RAM (raspi*).
        display: QEMU `-display` value; overridable at run time via
            `QEMU_DISPLAY`.

    Returns:
        A struct for the `qemu` argument of `_board`.
    """
    return struct(
        binary = binary,
        machine = machine,
        cpu = cpu,
        memory = memory,
        display = display,
    )

def _board(triple, linker_script, qemu = None):
    """Describes one board.

    Args:
        triple: key into `TRIPLES`; selects the toolchain, the `platform()` and
            the arch package.
        linker_script: label of the board's linker script.
        qemu: a `_qemu(...)` struct, or `None` for boards that only cross
            compile. Build-only boards still get a `//kernel:<name>` ELF; they
            just get no `//kernel:<name>.run` launcher.

    Returns:
        A struct consumed by the `kernel_image` macro.
    """
    if triple not in TRIPLES:
        fail("unknown triple '{}' — add it to TRIPLES first".format(triple))
    if TRIPLES[triple].arch == None:
        fail("triple '{}' has no arch package, so it cannot boot a board".format(triple))
    return struct(
        triple = triple,
        linker_script = linker_script,
        qemu = qemu,
    )

# Board name -> definition. The key doubles as the `target_board` cfg value and
# as the target name under //kernel.
#
# A board without a `qemu = ...` argument is build-only: real hardware, or a
# machine QEMU does not model. It builds like any other board and is simply not
# runnable from Bazel.
BOARDS = {
    "versatilepb": _board(
        triple = "armv7a-none-eabi",
        linker_script = "//boards:versatilepb.ld",
        qemu = _qemu(
            binary = "qemu-system-arm",
            machine = "versatilepb",
            cpu = "cortex-a8",
            memory = "128M",
        ),
    ),
    # raspi4b: fixed 2G of RAM, so no `memory` — QEMU rejects -m on this machine.
    "raspi4b": _board(
        triple = "aarch64-unknown-none",
        linker_script = "//boards:raspi4b.ld",
        qemu = _qemu(
            binary = "qemu-system-aarch64",
            machine = "raspi4b",
            cpu = "cortex-a72",
        ),
    ),
}
