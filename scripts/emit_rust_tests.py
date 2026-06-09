#!/usr/bin/env python3
"""Append a mirrored rust_test target to each first-party crate BUCK that has a
rust_library but no rust_test.

APPEND-ONLY by construction: it copies the crate's existing `rust_library(...)`
block verbatim, renames the rule to `rust_test` and the target to
`<name>-unittest`, and appends it. Everything else (srcs/crate/crate_root/deps/
env, including hand-tuned proto/build-script wiring) is mirrored exactly — so it
is safe on hand-edited BUCK files that `gen_first_party_buck.py --force` would
clobber. The test compiles the SAME crate in --test mode and runs its #[test]s.

Skips: proc-macro libraries (proc_macro = True), and BUCK files that already
contain a rust_test target (idempotent).

Usage:
    python3 scripts/emit_rust_tests.py [--subsystem DIR] [--dry-run]
"""

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).parent.parent.resolve()
# Never touch generated/vendored/upstream trees.
SKIP_PREFIXES = ("third-party/", "buck-out/", "prelude/", ".")

# Crates whose -unittest target currently fails under buck2 — emit is deferred
# until each is fixed (see #85). Re-running the emitter must NOT re-introduce a
# known-broken test target (it would fail the gate on the next change to that crate).
#   oya-check-dependency-seam              : test-binary link failure (rustc link [pic])
#   oya-shared-postgres-command-adapter-sqlx: test-binary link failure (native crypto/frameworks)
#   oya-shared-backbone-rest-runtime-adapter: route-count assertion 22 != 23
KNOWN_FAILING = {
    "libs/oya-check-dependency-seam",
    "libs/oya-shared-postgres-command-adapter-sqlx",
    "libs/oya-shared-backbone-rest-runtime-adapter",
}


def iter_buck_files(subsystem: str | None):
    base = REPO / subsystem if subsystem else REPO
    for p in sorted(base.rglob("BUCK")):
        rel = p.relative_to(REPO).as_posix()
        if any(rel == s.rstrip("/") or rel.startswith(s) for s in SKIP_PREFIXES):
            continue
        if p.parent.relative_to(REPO).as_posix() in KNOWN_FAILING:
            continue  # deferred until fixed (see #85)
        yield p, rel


def extract_blocks(text: str, rule: str):
    """Yield (start, end, block_text) for each top-level `<rule>(...)` call.
    block_text includes the rule name and the matching closing paren."""
    for m in re.finditer(rf'(?m)^{re.escape(rule)}\(', text):
        i = m.end()
        depth = 1
        while i < len(text) and depth:
            c = text[i]
            if c == '(':
                depth += 1
            elif c == ')':
                depth -= 1
            i += 1
        yield m.start(), i, text[m.start():i]


def lib_name(block: str) -> str | None:
    m = re.search(r'(?m)^\s*name\s*=\s*"([^"]+)"\s*,', block)
    return m.group(1) if m else None


def make_test_block(lib_block: str, name: str) -> str:
    """Turn a rust_library block into a rust_test block: rename rule + target."""
    out = lib_block.replace("rust_library(", "rust_test(", 1)
    # Replace only the target name attr (first `name = "<name>",`).
    out = re.sub(
        r'(?m)^(\s*name\s*=\s*")' + re.escape(name) + r'("\s*,)',
        r'\g<1>' + name + r'-unittest\g<2>',
        out,
        count=1,
    )
    return out


def process(text: str):
    """Return (new_text, n_added). Appends a rust_test per eligible rust_library."""
    if list(extract_blocks(text, "rust_test")):
        return text, 0  # already has test target(s) — idempotent skip
    additions = []
    for _s, _e, block in extract_blocks(text, "rust_library"):
        if re.search(r'(?m)^\s*proc_macro\s*=\s*True\s*,', block):
            continue  # proc-macro: test-mode compile is special; skip
        name = lib_name(block)
        if not name:
            continue
        additions.append(make_test_block(block, name))
    if not additions:
        return text, 0
    suffix = "\n" + "\n\n".join(additions) + "\n"
    if not text.endswith("\n"):
        suffix = "\n" + suffix
    return text + suffix, len(additions)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--subsystem", help="restrict to a path prefix (e.g. libs, oya, cloud)")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    changed = added = skipped = 0
    for p, rel in iter_buck_files(args.subsystem):
        text = p.read_text()
        new_text, n = process(text)
        if n == 0:
            skipped += 1
            continue
        changed += 1
        added += n
        if args.dry_run:
            print(f"[DRY-RUN] +{n} rust_test -> {rel}")
        else:
            p.write_text(new_text)
            print(f"  +{n} rust_test -> {rel}")

    print(f"\nSummary: {changed} BUCK files updated, {added} rust_test added, "
          f"{skipped} skipped (no lib / already has test).", file=sys.stderr)


if __name__ == "__main__":
    main()
