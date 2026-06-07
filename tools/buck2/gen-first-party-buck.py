#!/usr/bin/env python3
"""
Generate BUCK files for all first-party crates in oya-dev-cli's closure.

Usage:
    python3 tools/buck2/gen-first-party-buck.py [--dry-run]

Run from repo root. Requires `cargo metadata` (allowed by no-cargo hook).
Writes one BUCK file per package directory in the closure.
"""

import json
import sys
import os
import collections
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DRY_RUN = "--dry-run" in sys.argv

# ── Load reindeer-generated third-party target names ──────────────────────────
TP_BUCK = os.path.join(ROOT, "third-party", "BUCK")
import re as _re
_tp_names = _re.findall(r'name = "([^"]+)"', open(TP_BUCK).read())
EXISTING_TP = set(n for n in _tp_names if ".crate" not in n)


def tp_target(dep_name: str, dep_ver: str) -> str:
    """Map a third-party crate name+version to its reindeer Buck2 target."""
    parts = dep_ver.split(".")
    maj = parts[0]
    majmin = f"{parts[0]}.{parts[1]}"
    for cand in [f"{dep_name}-{maj}", f"{dep_name}-{majmin}", dep_name]:
        if cand in EXISTING_TP:
            return f"third-party//:{cand}"
    # Not yet in third-party/BUCK — note it but don't fail generator
    return f"third-party//:{dep_name}-{maj}  # MISSING-{dep_ver}"


def fp_target(pkg: dict) -> str:
    """Return Buck2 target label for a first-party package."""
    pkg_dir = os.path.dirname(pkg["manifest_path"])
    rel = os.path.relpath(pkg_dir, ROOT)
    return f"//{rel}:{pkg['name']}"


def is_normal_dep(dep: dict) -> bool:
    """True if this resolve dep is a normal (not dev/build) dependency."""
    kinds = [k.get("kind") for k in dep.get("dep_kinds", [])]
    return None in kinds  # null kind = normal runtime dep


# ── Run cargo metadata ────────────────────────────────────────────────────────
print("Running cargo metadata...", file=sys.stderr)
result = subprocess.run(
    ["cargo", "metadata", "--format-version", "1",
     "--manifest-path", os.path.join(ROOT, "Cargo.toml")],
    capture_output=True, text=True
)
if result.returncode != 0:
    print(f"cargo metadata failed: {result.stderr[:500]}", file=sys.stderr)
    sys.exit(1)

data = json.loads(result.stdout)
pkgs_by_id = {p["id"]: p for p in data["packages"]}
resolve_by_id = {n["id"]: n for n in data["resolve"]["nodes"]}

# ── Compute oya-dev-cli closure ───────────────────────────────────────────────
cli_pkg = next(p for p in data["packages"] if p["name"] == "oya-dev-cli")
cli_id = cli_pkg["id"]

visited: set[str] = set()
queue: collections.deque[str] = collections.deque([cli_id])
while queue:
    cur = queue.popleft()
    if cur in visited:
        continue
    visited.add(cur)
    node = resolve_by_id.get(cur)
    if node:
        for dep in node["deps"]:
            queue.append(dep["pkg"])

first_party = [pkgs_by_id[pid] for pid in visited
               if pkgs_by_id[pid]["source"] is None]
fp_by_name = {p["name"]: p for p in first_party}

print(f"Closure: {len(visited)} total, {len(first_party)} first-party", file=sys.stderr)

# ── Generate BUCK files ───────────────────────────────────────────────────────
# Test fixture bins to skip (they're test infrastructure, not the main binary)
SKIP_BINS = {"fake-cargo", "fake-verify-command"}

written = 0
skipped = 0

for pkg in first_party:
    pkg_dir = os.path.dirname(pkg["manifest_path"])
    rel_dir = os.path.relpath(pkg_dir, ROOT)
    buck_path = os.path.join(pkg_dir, "BUCK")

    node = resolve_by_id.get(pkg["id"])

    # Collect normal deps
    deps: list[str] = []
    if node:
        for dep in node["deps"]:
            if not is_normal_dep(dep):
                continue
            dpkg = pkgs_by_id[dep["pkg"]]
            if dpkg["source"] is None:
                deps.append(fp_target(dpkg))
            else:
                deps.append(tp_target(dpkg["name"], dpkg["version"]))
    deps = sorted(set(deps))

    lib_targets = [t for t in pkg["targets"]
                   if "lib" in t.get("kind", []) or "proc-macro" in t.get("kind", [])]
    bin_targets = [t for t in pkg["targets"]
                   if "bin" in t.get("kind", []) and t["name"] not in SKIP_BINS]

    rules: list[str] = []

    for lt in lib_targets:
        is_pm = "proc-macro" in lt.get("crate_types", [])
        crate_name = lt["name"].replace("-", "_")
        src_root = os.path.relpath(lt["src_path"], pkg_dir)

        lines = [f'rust_library(']
        lines.append(f'    name = "{pkg["name"]}",')
        lines.append(f'    srcs = glob(["src/**/*.rs"]),')
        lines.append(f'    crate = "{crate_name}",')
        lines.append(f'    crate_root = "{src_root}",')
        if is_pm:
            lines.append(f'    proc_macro = True,')
        lines.append(f'    visibility = ["PUBLIC"],')
        if deps:
            lines.append(f'    deps = [')
            for d in deps:
                lines.append(f'        "{d}",')
            lines.append(f'    ],')
        lines.append(f')')
        rules.append("\n".join(lines))

    for bt in bin_targets:
        src_root = os.path.relpath(bt["src_path"], pkg_dir)
        lines = [f'rust_binary(']
        lines.append(f'    name = "{bt["name"]}",')
        lines.append(f'    srcs = glob(["src/**/*.rs"]),')
        lines.append(f'    crate_root = "{src_root}",')
        lines.append(f'    visibility = ["PUBLIC"],')
        if deps:
            lines.append(f'    deps = [')
            for d in deps:
                lines.append(f'        "{d}",')
            lines.append(f'    ],')
        lines.append(f')')
        rules.append("\n".join(lines))

    if not rules:
        skipped += 1
        continue

    content = "\n\n".join(rules) + "\n"

    if DRY_RUN:
        print(f"[DRY-RUN] Would write {rel_dir}/BUCK ({len(rules)} rules)")
    else:
        with open(buck_path, "w") as f:
            f.write(content)
        written += 1

print(f"Done: {written} BUCK files written, {skipped} skipped (no rules)", file=sys.stderr)
