#!/usr/bin/env python3
"""Apply durable third-party/BUCK hand-edits that reindeer cannot express.

The main patch captures most hand edits. This idempotent post-processor keeps
platform-selectable $(location ...) sysroot env behind the musl platform branch
so default GitHub affected builds do not materialize optional external archives.
"""

from __future__ import annotations

import re
from pathlib import Path

BUCK = Path("third-party/BUCK")
START = '    env = '
ANCHOR = '"CARGO_MANIFEST_LINKS": "aws_lc_0_41_0"'
END = '    features = ["prebuilt-nasm"],'

BASE_PAIRS = [
    ("CARGO_MANIFEST_LINKS", "aws_lc_0_41_0"),
    ("CARGO_PKG_VERSION_MAJOR", "0"),
    ("CARGO_PKG_VERSION_MINOR", "41"),
    ("CARGO_PKG_VERSION_PATCH", "0"),
    ("CARGO_PKG_VERSION_PRE", ""),
    ("DEBUG", "false"),
    ("LDFLAGS", "-nostartfiles"),
    ("OPT_LEVEL", "3"),
    ("PROFILE", "release"),
]

COMMENT = '''        # aws-lc-sys's cc_builder compiler feature-test (memcmp_invalid_stripped_check)
        # LINKS an executable via $CC. Under buck2 the build-script $CC is the prelude
        # cc-shim `clang --ld-path=__ld_shim.sh`, and __ld_shim re-invokes clang as the
        # link driver — which RE-ADDS the C-runtime startfiles (Scrt1.o/crti.o/crtbeginS.o)
        # on top of the complete ld line the outer clang already passed → `ld.lld: error:
        # duplicate symbol: _start/_init/...` on aarch64-linux (darwin ld64 tolerates it).
        # aws-lc-sys explicitly respects LDFLAGS for this probe (cc_builder.rs ~745, "brings
        # us back to parity with CMake" for custom-linker setups), so -nostartfiles makes the
        # OUTER clang omit its CRT; the inner __ld_shim clang adds exactly one set → links
        # clean and the probe still runs. Scoped to this build script only (compile steps are
        # -c, where -nostartfiles is ignored). Verified on the rust-ci image (clang 19.1.7).
        # The durable class-wide fix is a prelude patch (-nostartfiles on the __ld_shim clang);
        # this targeted fixup unblocks aws-lc-sys (and thus all aws-lc-rs Linux binaries) now.
'''

MUSL_COMMENT = '''        # musl-static (#83 musl lane): compile bcm.c against MUSL headers. -nostdlibinc
        # drops glibc /usr/include (whose stdlib.h redirects strtol->__isoc23_strtol, a
        # glibc-2.38 symbol undefined when linking musl). Keep the $(location) sysroot
        # hidden dep behind the musl platform select so default GitHub affected builds
        # do not materialize optional external toolchain proof archives.
'''

MUSL_PAIRS = [
    ("CC_aarch64_unknown_linux_musl", "clang"),
    (
        "CFLAGS_aarch64_unknown_linux_musl",
        "--target=aarch64-unknown-linux-musl -nostdlibinc -isystem $(location toolchains//cxx/clang_hermetic:aarch64-musl-sysroot)/aarch64-linux-musl/include -isystem $(location toolchains//cxx/clang_hermetic:aarch64-musl-sysroot)/aarch64-linux-musl/include/linux",
    ),
]


def indent_block(text: str, spaces: int) -> str:
    prefix = " " * spaces
    return "".join(prefix + line.lstrip() if line.strip() else line for line in text.splitlines(True))


def pair_lines(pairs: list[tuple[str, str]], spaces: int = 12) -> str:
    prefix = " " * spaces
    return "".join(f'{prefix}"{key}": "{value}",\n' for key, value in pairs)


def selected_env() -> str:
    base = pair_lines(BASE_PAIRS)
    musl = pair_lines(MUSL_PAIRS)
    comment = indent_block(COMMENT, 12)
    musl_comment = indent_block(MUSL_COMMENT, 12)
    return (
        '    env = select({\n'
        '        "root//platforms:libc_musl": {\n'
        + comment
        + base
        + musl_comment
        + musl
        + '        },\n'
        + '        "DEFAULT": {\n'
        + comment
        + base
        + '        },\n'
        + '    }),\n'
    )


def main() -> int:
    text = BUCK.read_text()
    anchor = text.find(ANCHOR)
    if anchor == -1:
        raise SystemExit("aws-lc-sys buildscript env anchor not found")
    start = text.rfind(START, 0, anchor)
    if start == -1:
        raise SystemExit("aws-lc-sys env start not found")
    end = text.find(END, anchor)
    if end == -1:
        raise SystemExit("aws-lc-sys features anchor not found")

    replacement = selected_env()
    updated = text[:start] + replacement + text[end:]
    if '"DEFAULT": {\n' not in replacement or 'root//platforms:libc_musl' not in replacement:
        raise SystemExit("internal error: selected env missing required branches")
    if re.search(r'env = \{[^}]*CFLAGS_aarch64_unknown_linux_musl', replacement, re.S):
        raise SystemExit("internal error: musl env is not select-gated")
    if updated != text:
        BUCK.write_text(updated)
        print("updated third-party/BUCK aws-lc-sys musl env select")
    else:
        print("third-party/BUCK aws-lc-sys musl env select already current")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
