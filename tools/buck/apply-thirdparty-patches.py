#!/usr/bin/env python3
"""Re-apply third-party/BUCK patches that `reindeer buckify` cannot emit.

reindeer regenerates third-party/BUCK from Cargo.lock + fixups, but its fixup
TOML cannot express buck2 `$(location ...)` macros. Some crates that use cargo's
`links` mechanism (build-script DEP_* propagation) therefore need the dependency's
build-script-run out_dir wired in by macro — which we inject here.

Run AFTER every `reindeer buckify`:  reindeer buckify && python3 tools/buck/apply-thirdparty-patches.py
Idempotent: skips a patch whose marker key is already present.
"""
import sys
from pathlib import Path

BUCK = Path(__file__).resolve().parents[2] / "third-party" / "BUCK"

# (target name, marker key, lines to insert after the CARGO_PKG_VERSION_PRE entry)
PATCHES = [
    (
        "aws-lc-rs-1-build-script-run",
        "DEP_AWS_LC_0_41_0_INCLUDE",
        [
            '        # DEP_* propagated from aws-lc-sys (links = "aws_lc_0_41_0") — reindeer',
            '        # cannot emit the $(location) macro, so it is injected post-buckify.',
            '        "DEP_AWS_LC_0_41_0_INCLUDE": "$(location :aws-lc-sys-0.41-build-script-main-run[out_dir])/include",',
            '        "DEP_AWS_LC_0_41_0_LIBCRYPTO": "aws_lc_0_41_0_crypto",',
            '        "CARGO_FEATURE_AWS_LC_SYS": "1",',
        ],
    ),
]


def apply(text: str) -> tuple[str, int]:
    lines = text.splitlines()
    applied = 0
    for target, marker, inject in PATCHES:
        # locate the target rule, then its first CARGO_PKG_VERSION_PRE within ~40 lines
        ti = next((i for i, l in enumerate(lines) if f'name = "{target}"' in l), None)
        if ti is None:
            print(f"WARN: target {target} not found — skipping", file=sys.stderr)
            continue
        window = lines[ti : ti + 60]
        if any(marker in l for l in window):
            print(f"ok: {target} already patched ({marker})")
            continue
        anchor = next((ti + j for j, l in enumerate(window) if "CARGO_PKG_VERSION_PRE" in l), None)
        if anchor is None:
            print(f"WARN: anchor not found in {target} — skipping", file=sys.stderr)
            continue
        lines[anchor + 1 : anchor + 1] = inject
        applied += 1
        print(f"patched: {target} (+{len(inject)} lines)")
    return "\n".join(lines) + "\n", applied


def main() -> int:
    if not BUCK.exists():
        print(f"ERROR: {BUCK} not found", file=sys.stderr)
        return 2
    new, n = apply(BUCK.read_text())
    if n:
        BUCK.write_text(new)
    print(f"apply-thirdparty-patches: {n} patch(es) applied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
