"""Macros for building bootable kernel images and their QEMU launchers."""

load("@rules_shell//shell:sh_binary.bzl", "sh_binary")
load("@rules_rust//rust:defs.bzl", "rust_binary")
load("//boards:boards.bzl", "BOARDS", "TRIPLES")

# Every legal `target_board` value, so rustc flags typos in
# `#[cfg(target_board = "...")]` instead of silently compiling the branch out.
_CHECK_CFG = "--check-cfg=cfg(target_board, values({}))".format(
    ", ".join(['"{}"'.format(board) for board in sorted(BOARDS)]),
)

def kernel_image(name, srcs, crate_root, deps = [], rustc_flags = []):
    """Builds a bare-metal kernel ELF, plus a `<name>.run` QEMU launcher.

    All board-specific settings come from the `BOARDS` entry named `name`, so a
    board is described in exactly one place. Boards whose entry has no `qemu`
    config are build-only: they produce the ELF and no launcher.

    The generated `rust_binary` pins its target platform through the `platform`
    attribute, which means `bazel build //kernel:<board>` needs no
    `--platforms` flag, and `target_board` can be baked in directly rather than
    routed through `select()`.

    Args:
        name: board name; must be a key of `BOARDS`.
        srcs: kernel sources.
        crate_root: entry point among `srcs`; required because rules_rust only
            infers it for single-source crates.
        deps: crate dependencies.
        rustc_flags: extra flags, appended after the board's own.
    """
    if name not in BOARDS:
        fail("no board '{}' in //boards:boards.bzl".format(name))
    board = BOARDS[name]

    rust_binary(
        name = name,
        srcs = srcs,
        crate_root = crate_root,
        linker_script = board.linker_script,
        platform = "//platforms:" + board.triple,
        rustc_flags = [
            '--cfg=target_board="{}"'.format(name),
            _CHECK_CFG,
        ] + rustc_flags,
        # The arch package carries head.S. Which one is a plain table lookup —
        # the board's triple is known here at macro expansion, so this needs no
        # select() and no arch constraint.
        deps = deps + [TRIPLES[board.triple].arch],
    )

    if board.qemu == None:
        return

    # Boards with fixed RAM (raspi*) leave `memory` empty; QEMU rejects -m on
    # them anyway. Building the flags as a list keeps empty values out of the
    # argv entirely — see the note in qemu_runner.sh.
    qemu_flags = ["-M", board.qemu.machine, "-cpu", board.qemu.cpu]
    if board.qemu.memory:
        qemu_flags += ["-m", board.qemu.memory]

    sh_binary(
        name = name + ".run",
        srcs = ["//tools:qemu_runner.sh"],
        args = [
            board.qemu.binary,
            board.qemu.display,
            "$(rootpath :{})".format(name),
        ] + qemu_flags,
        data = [":" + name],
    )
