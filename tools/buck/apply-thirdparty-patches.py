#!/usr/bin/env python3
"""Apply the semantic post-buckify overlay that Reindeer cannot express.

reindeer regenerates third-party/BUCK from Cargo.lock + fixups, but its fixup
TOML cannot express buck2 `$(location ...)` macros. Some crates that use cargo's
`links` mechanism (build-script DEP_* propagation) therefore need the dependency's
build-script-run out_dir wired in by macro — which we inject here.

Run only through scripts/ci/regen-third-party.sh. Every patch is anchored to a
unique generated rule and fails closed when the expected Reindeer output changes.
"""
import argparse
from pathlib import Path
import sys

BUCK = Path(__file__).resolve().parents[2] / "third-party" / "BUCK"

# (target name, owned keys, exact lines inserted after CARGO_PKG_VERSION_PRE)
ENV_PATCHES = [
    (
        "aws-lc-rs-1-build-script-run",
        [
            "DEP_AWS_LC_0_41_0_INCLUDE",
            "DEP_AWS_LC_0_41_0_LIBCRYPTO",
            "CARGO_FEATURE_AWS_LC_SYS",
        ],
        [
            '        # DEP_* propagated from aws-lc-sys (links = "aws_lc_0_41_0") — reindeer',
            '        # cannot emit the $(location) macro, so it is injected post-buckify.',
            '        "DEP_AWS_LC_0_41_0_INCLUDE": "$(location :aws-lc-sys-0.41-build-script-main-run[out_dir])/include",',
            '        "DEP_AWS_LC_0_41_0_LIBCRYPTO": "aws_lc_0_41_0_crypto",',
            '        "CARGO_FEATURE_AWS_LC_SYS": "1",',
        ],
    ),
]

PSM_TARGET = "psm-0.1-psm_asm"
PSM_MARKER = '"prelude//os:linux": ['
PSM_BASELINE = [
    '    preprocessor_flags = [',
    '        "-DCFG_TARGET_OS_darwin",',
    '        "-DCFG_TARGET_ARCH_aarch64",',
    '        "-DCFG_TARGET_ENV_",',
    '    ],',
]
PSM_OVERLAY = [
    '    preprocessor_flags = select({',
    '        "prelude//os:linux": [',
    '            "-DCFG_TARGET_OS_linux",',
    '            "-DCFG_TARGET_ARCH_aarch64",',
    '            "-DCFG_TARGET_ENV_",',
    '        ],',
    '        "DEFAULT": [',
    '            "-DCFG_TARGET_OS_darwin",',
    '            "-DCFG_TARGET_ARCH_aarch64",',
    '            "-DCFG_TARGET_ENV_",',
    '        ],',
    '    }),',
]


def unique_rule_start(lines: list[str], target: str) -> int:
    matches = [i for i, line in enumerate(lines) if f'name = "{target}"' in line]
    if len(matches) != 1:
        raise ValueError(f"expected exactly one generated rule {target}, found {len(matches)}")
    return matches[0]


def rule_end(lines: list[str], start: int) -> int:
    for index in range(start, len(lines)):
        if lines[index] == ")":
            return index
    raise ValueError("unterminated generated rule")


def apply(text: str) -> tuple[str, int]:
    lines = text.splitlines()
    applied = 0
    for target, owned_keys, inject in ENV_PATCHES:
        ti = unique_rule_start(lines, target)
        end = rule_end(lines, ti)
        window = lines[ti : end + 1]
        anchor = next((ti + j for j, line in enumerate(window) if "CARGO_PKG_VERSION_PRE" in line), None)
        if anchor is None:
            raise ValueError(f"missing CARGO_PKG_VERSION_PRE anchor in {target}")
        exact = lines[anchor + 1 : anchor + 1 + len(inject)] == inject
        owned_occurrences = {
            key: sum(key in line for line in window)
            for key in owned_keys
        }
        if exact and all(count == 1 for count in owned_occurrences.values()):
            print(f"ok: {target} already patched (exact DEP fragment)")
            continue
        if any(owned_occurrences.values()):
            raise ValueError(
                f"incomplete or corrupt DEP overlay in {target}: {owned_occurrences}"
            )
        lines[anchor + 1 : anchor + 1] = inject
        applied += 1
        print(f"patched: {target} (+{len(inject)} lines)")

    ti = unique_rule_start(lines, PSM_TARGET)
    end = rule_end(lines, ti)
    window = lines[ti : end + 1]
    overlay_starts = [
        ti + offset
        for offset, line in enumerate(window)
        if line == PSM_OVERLAY[0]
    ]
    if overlay_starts:
        start = overlay_starts[0]
        marker_count = sum(PSM_MARKER in line for line in window)
        if (
            len(overlay_starts) != 1
            or marker_count != 1
            or lines[start : start + len(PSM_OVERLAY)] != PSM_OVERLAY
        ):
            raise ValueError(f"incomplete or corrupt platform overlay in {PSM_TARGET}")
        print(f"ok: {PSM_TARGET} already patched (exact platform select)")
    else:
        if any(PSM_MARKER in line for line in window):
            raise ValueError(f"incomplete or corrupt platform overlay in {PSM_TARGET}")
        start = next((ti + j for j, line in enumerate(window) if line == PSM_BASELINE[0]), None)
        if start is None or lines[start : start + len(PSM_BASELINE)] != PSM_BASELINE:
            raise ValueError(f"unexpected generated preprocessor flags in {PSM_TARGET}")
        lines[start : start + len(PSM_BASELINE)] = PSM_OVERLAY
        applied += 1
        print(f"patched: {PSM_TARGET} (platform select)")
    return "\n".join(lines) + "\n", applied


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Apply the exact semantic overlay to Reindeer Buck output."
    )
    parser.add_argument(
        "--buck-file",
        type=Path,
        default=BUCK,
        help="Reindeer output to patch (default: third-party/BUCK)",
    )
    return parser.parse_args()


def main() -> int:
    buck_file = parse_args().buck_file
    if not buck_file.exists():
        print(f"ERROR: {buck_file} not found", file=sys.stderr)
        return 2
    try:
        new, n = apply(buck_file.read_text())
    except ValueError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2
    if n:
        buck_file.write_text(new)
    print(f"apply-thirdparty-patches: {n} patch(es) applied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
