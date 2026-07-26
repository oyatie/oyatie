#!/usr/bin/env python3
"""
Generate first-party BUCK rules for all workspace crates.

Usage:
    python3 scripts/gen_first_party_buck.py [--dry-run] [--subsystem SUBSYSTEM]

Rules:
  - rust_library for crates with a lib target
  - rust_binary for crates with a bin target (additional entry in same BUCK)
  - proc_macro=True for proc-macro crate types
  - path deps -> //<relative>:<name>
  - registry deps -> third-party//:<name>-<major>  (best-match from BUCK)
  - Skips crates that already have a BUCK file (unless --force)
  - Skips third-party/, buck-out/
"""

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).parent.parent.resolve()
SKIP_DIRS = {"third-party", "buck-out", ".git", "target"}

# Crates with build.rs that need special treatment
# Key = crate name, value = dict of extra attrs
BUILDSCRIPT_OVERRIDES = {
    # These two use protoc-bin-vendored; build.rs runs prost/tonic codegen
    "oya-shared-backbone-grpc-generated-adapter": {
        "has_buildscript": True,
        "proto": True,
    },
    "oya-identity-workload-rest": {
        "has_buildscript": True,
        "proto": True,
    },
    # Placeholder build.rs — no codegen, just emit rerun-if-changed
    "oya-intelligence-supervisor-app": {
        "has_buildscript": True,
        "proto": False,
    },
}

# Package-specific native build-graph requirements that Cargo metadata cannot express.
# These are generator inputs, never hand-maintained BUCK output.
RUST_RULE_OVERRIDES = {
    "oya-application-shell-frontend": {
        "rustc_flags": ["--cfg", 'feature="ssr"'],
        "integration_tests": [
            {
                "name": "oya-application-shell-frontend-live-server-integration",
                "src": "tests/live_server.rs",
                "extra_srcs": ["src/app.rs", "src/lib.rs"],
                "crate": "oya_application_shell_frontend_live_server",
                "deps": [
                    ":oya-application-shell-frontend",
                    "third-party//:tokio",
                ],
            },
        ],
    },
}


def load_third_party_names(repo_root: Path) -> tuple[list[str], set[str]]:
    """Return (all_target_names, public_alias_names) from third-party/BUCK."""
    buck_path = repo_root / "third-party" / "BUCK"
    content = buck_path.read_text()
    names = re.findall(r'name = "([^"]+)"', content)
    all_names = sorted(set(
        n for n in names
        if not n.endswith(".crate") and "build-script" not in n
    ))
    # Public aliases: alias() with visibility=["PUBLIC"]
    alias_pattern = re.compile(
        r'alias\([^)]*name\s*=\s*"([^"]+)"[^)]*visibility\s*=\s*\["PUBLIC"\][^)]*\)',
        re.DOTALL,
    )
    public_aliases = set(m.group(1) for m in alias_pattern.finditer(content))
    return all_names, public_aliases


def build_tp_resolver(tp_names: list[str], public_aliases: set[str]):
    """
    Build a function that maps (crate_name, version_req) -> third-party target name.

    Strategy (PUBLIC aliases preferred — versioned targets have visibility=[]):
      1. Public alias exact match: normalized crate name (e.g. serde_json -> serde-json... wait, serde_json alias uses underscore)
      2. Public alias with underscore name
      3. Versioned public alias fallback
      4. Versioned non-public (last resort — will be a visibility error but lets build proceed)
    """
    tp_set = set(tp_names)

    def _major_from_req(req: str) -> str | None:
        """Extract major version from semver req like ^1.2.3, >=1, =1.2 etc."""
        m = re.search(r"(\d+)", req)
        return m.group(1) if m else None

    def resolve(crate_name: str, version_req: str) -> str | None:
        # Normalize crate name: Cargo uses _ but Buck uses - for target names
        normalized = crate_name.replace("_", "-")
        major = _major_from_req(version_req)

        # 1. Public alias with normalized name (dash form)
        if normalized in public_aliases:
            return normalized
        # 2. Public alias with original name (underscore form, e.g. clap_complete)
        if crate_name in public_aliases:
            return crate_name

        # 3. versioned match (may not be publicly visible, but try)
        if major:
            candidate = f"{normalized}-{major}"
            if candidate in tp_set:
                return candidate
            # Also try with underscore name kept
            candidate2 = f"{crate_name}-{major}"
            if candidate2 in tp_set:
                return candidate2

        # 4. exact name (short alias without version)
        if normalized in tp_set:
            return normalized
        if crate_name in tp_set:
            return crate_name

        # 3. prefix scan — pick longest match
        matches = [n for n in tp_names if n == normalized or n.startswith(normalized + "-")]
        if not matches:
            matches = [n for n in tp_names if n == crate_name or n.startswith(crate_name + "-")]
        if matches:
            # Prefer versioned (contains a digit suffix)
            versioned = [m for m in matches if re.search(r"-\d", m)]
            return versioned[0] if versioned else matches[0]

        return None

    return resolve


def load_workspace_metadata(repo_root: Path) -> dict:
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        capture_output=True, text=True, cwd=repo_root
    )
    if result.returncode != 0:
        print(f"ERROR: cargo metadata failed:\n{result.stderr}", file=sys.stderr)
        sys.exit(1)
    return json.loads(result.stdout)


def pkg_rel_path(pkg_manifest: str, repo_root: Path) -> Path:
    """Return path of crate dir relative to repo root."""
    return Path(os.path.relpath(os.path.dirname(pkg_manifest), repo_root))


def crate_name_to_ident(name: str) -> str:
    """Convert crate name to Rust ident (replace - with _)."""
    return name.replace("-", "_")


def should_skip(rel_path: Path) -> bool:
    parts = rel_path.parts
    for skip in SKIP_DIRS:
        if skip in parts or str(rel_path).startswith(skip):
            return True
    return False


def generate_buck_content(
    pkg: dict,
    rel_path: Path,
    repo_root: Path,
    pkg_by_id: dict,
    tp_resolve,
    force: bool,
) -> str | None:
    """
    Generate BUCK file content for a package.
    Returns None if the file should be skipped.
    """
    pkg_dir = repo_root / rel_path
    buck_path = pkg_dir / "BUCK"

    if not force and buck_path.exists():
        return None  # already has BUCK

    name = pkg["name"]
    edition = pkg.get("edition", "2024")

    # Classify targets
    targets = pkg["targets"]
    lib_target = None
    bin_targets = []
    is_proc_macro = False

    for t in targets:
        kinds = t.get("kind", [])
        ctypes = t.get("crate_types", [])
        if "lib" in kinds or "rlib" in kinds:
            lib_target = t
        elif "proc-macro" in kinds or "proc-macro" in ctypes:
            lib_target = t
            is_proc_macro = True
        elif "bin" in kinds:
            bin_targets.append(t)
        # skip: test, bench, example, custom-build

    # Resolve deps
    deps_lines = []
    dep_ids = {d["name"]: d for d in pkg.get("dependencies", [])}

    # We need the resolved package list to map dep name -> path
    # pkg_by_id maps id -> pkg; we need name -> pkg for path deps
    # Build name->rel_path for all first-party pkgs
    # (passed in as pkg_by_name)

    for dep in pkg.get("dependencies", []):
        dep_name = dep["name"]
        dep_req = dep.get("req", "*")
        dep_path = dep.get("path")  # set for path deps

        if dep_path:
            # First-party path dep
            dep_rel = Path(os.path.relpath(dep_path, repo_root))
            # Target label: //<rel>:<name>
            # The target name is the package name at that path
            # We'll use dep_name as the target name (matches BUCK convention)
            label = f"//{dep_rel}:{dep_name}"
            deps_lines.append(f'        "{label}",')
        else:
            # Registry dep
            tp_name = tp_resolve(dep_name, dep_req)
            if tp_name:
                deps_lines.append(f'        "third-party//:{tp_name}",')
            else:
                # Unknown — emit a comment so it's visible
                deps_lines.append(f'        # UNRESOLVED: {dep_name} {dep_req}')

    deps_lines.sort()

    # Build output
    lines = []

    buildscript_info = BUILDSCRIPT_OVERRIDES.get(name, {})
    rule_override = RUST_RULE_OVERRIDES.get(name, {})
    has_buildscript = buildscript_info.get("has_buildscript", False)
    is_proto = buildscript_info.get("proto", False)

    # For proto crates, we need buildscript_run
    if has_buildscript and is_proto:
        lines.append('load("@prelude//rust:cargo_buildscript.bzl", "buildscript_run")')
        lines.append("")

    def _render_rule(rule: str, rule_name: str, crate_root: str, extra_attrs: list[str] = None, crate_ident: str = None) -> list[str]:
        out = []
        out.append(f"{rule}(")
        out.append(f'    name = "{rule_name}",')
        out.append(f'    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),')
        if rule in ("rust_library", "rust_test"):
            ci = crate_ident or crate_name_to_ident(rule_name)
            out.append(f'    crate = "{ci}",')
        out.append(f'    crate_root = "{crate_root}",')
        if edition != "2024":  # 2024 is the workspace default; only emit for 2021
            out.append(f'    edition = "{edition}",')
        if is_proc_macro and rule == "rust_library":
            out.append(f'    proc_macro = True,')
        if extra_attrs:
            for attr in extra_attrs:
                out.append(f'    {attr}')
        if rule_override.get("rustc_flags"):
            out.append(f'    rustc_flags = {json.dumps(rule_override["rustc_flags"])},')
        out.append(f'    visibility = ["PUBLIC"],')
        if deps_lines:
            out.append(f'    deps = [')
            for d in deps_lines:
                out.append(d)
            out.append(f'    ],')
        out.append(")")
        return out

    # Proto/buildscript crates: add buildscript_run target
    if has_buildscript and is_proto:
        edition_line = f'    edition = "{edition}",' if edition != "2024" else ""
        lines.extend([
            f'rust_binary(',
            f'    name = "{name}-build-script",',
            f'    srcs = ["build.rs"],',
            f'    crate = "build_script_build",',
            f'    crate_root = "build.rs",',
            *(([edition_line]) if edition_line else []),
            f'    visibility = [],',
            f'    deps = [',
            f'        "third-party//:protoc-bin-vendored-3",',
            f'        "third-party//:tonic-prost-build-0.14",',
            f'    ],',
            f')',
            f'',
            f'buildscript_run(',
            f'    name = "{name}-build-script-run",',
            f'    script = ":{name}-build-script",',
            f')',
            f'',
        ])

    # Lib target
    if lib_target:
        # Determine crate_root
        src_file = lib_target.get("src_path", "")
        if src_file:
            crate_root = os.path.relpath(src_file, str(pkg_dir))
        else:
            crate_root = "src/lib.rs"

        extra = []
        if has_buildscript and is_proto:
            extra.append(f'env = {{"OUT_DIR": "$(location :{name}-build-script-run[out_dir])"}},')

        lines.extend(_render_rule("rust_library", name, crate_root, extra))

        # Unit-test target (#84): compile the lib in --test mode and run its #[test]s.
        # Mirrors the lib's srcs/crate/crate_root/deps — deps already include dev-deps
        # (cargo metadata lumps all kinds), so tests using dev-deps compile. Skip
        # proc-macro crates (test-mode proc-macro compilation needs special handling).
        if not is_proc_macro:
            lines.append("")
            lines.extend(_render_rule(
                "rust_test", f"{name}-unittest", crate_root, extra,
                crate_ident=crate_name_to_ident(name),
            ))

    # Bin targets
    for bt in bin_targets:
        bin_name = bt["name"]
        # Avoid name collision with the lib target (lib uses pkg name with dashes,
        # bin target may also use pkg name). Append -bin suffix when they clash.
        if lib_target and bin_name == name:
            buck_bin_name = f"{bin_name}-bin"
        else:
            buck_bin_name = bin_name

        src_file = bt.get("src_path", "")
        if src_file:
            bin_crate_root = os.path.relpath(src_file, str(pkg_dir))
        else:
            bin_crate_root = f"src/main.rs"

        if lines:  # separate rules with blank line
            lines.append("")

        lines.append(f"rust_binary(")
        lines.append(f'    name = "{buck_bin_name}",')
        lines.append(f'    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),')
        lines.append(f'    crate_root = "{bin_crate_root}",')
        if edition != "2024":
            lines.append(f'    edition = "{edition}",')
        if rule_override.get("rustc_flags"):
            lines.append(f'    rustc_flags = {json.dumps(rule_override["rustc_flags"])},')
        lines.append(f'    visibility = ["PUBLIC"],')
        # For binaries in lib+bin crates, always emit deps so the lib is included
        bin_deps = []
        if lib_target:
            bin_deps.append(f'        "//{rel_path}:{name}",')
        bin_deps.extend(deps_lines)
        if bin_deps:
            lines.append(f'    deps = [')
            for d in bin_deps:
                lines.append(d)
            lines.append(f'    ],')
        lines.append(")")

    for integration_test in rule_override.get("integration_tests", []):
        if lines:
            lines.append("")
        lines.append("rust_test(")
        lines.append(f'    name = "{integration_test["name"]}",')
        test_srcs = [integration_test["src"], *integration_test.get("extra_srcs", [])]
        lines.append(f'    srcs = {json.dumps(test_srcs)},')
        lines.append(f'    crate = "{integration_test["crate"]}",')
        lines.append(f'    crate_root = "{integration_test["src"]}",')
        if edition != "2024":
            lines.append(f'    edition = "{edition}",')
        if rule_override.get("rustc_flags"):
            lines.append(f'    rustc_flags = {json.dumps(rule_override["rustc_flags"])},')
        lines.append('    visibility = ["PUBLIC"],')
        lines.append("    deps = [")
        for dependency in integration_test["deps"]:
            lines.append(f'        "{dependency}",')
        lines.append("    ],")
        lines.append(")")

    if not lib_target and not bin_targets:
        # Custom-build only or empty — skip
        return None

    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description="Generate first-party BUCK files")
    parser.add_argument("--dry-run", action="store_true", help="Print what would be written without writing")
    parser.add_argument("--force", action="store_true", help="Overwrite existing BUCK files")
    parser.add_argument("--subsystem", help="Only generate for a specific subsystem (e.g. libs, cloud, oya, tools)")
    args = parser.parse_args()

    repo_root = REPO_ROOT
    print(f"Repository root: {repo_root}", file=sys.stderr)

    tp_names, public_aliases = load_third_party_names(repo_root)
    tp_resolve = build_tp_resolver(tp_names, public_aliases)
    print(f"Loaded {len(tp_names)} third-party targets ({len(public_aliases)} public aliases)", file=sys.stderr)

    metadata = load_workspace_metadata(repo_root)
    pkgs = metadata["packages"]
    print(f"Loaded {len(pkgs)} workspace packages", file=sys.stderr)

    # Build id -> pkg map
    pkg_by_id = {p["id"]: p for p in pkgs}

    generated = 0
    skipped_existing = 0
    skipped_no_targets = 0
    errors = []

    for pkg in sorted(pkgs, key=lambda p: p["name"]):
        rel_path = pkg_rel_path(pkg["manifest_path"], repo_root)

        if should_skip(rel_path):
            continue

        if args.subsystem and not str(rel_path).startswith(args.subsystem):
            continue

        try:
            content = generate_buck_content(
                pkg, rel_path, repo_root, pkg_by_id, tp_resolve,
                force=args.force
            )
        except Exception as e:
            errors.append((pkg["name"], str(e)))
            print(f"ERROR generating {pkg['name']}: {e}", file=sys.stderr)
            continue

        if content is None:
            buck_path = repo_root / rel_path / "BUCK"
            if buck_path.exists():
                skipped_existing += 1
            else:
                skipped_no_targets += 1
            continue

        buck_path = repo_root / rel_path / "BUCK"
        if args.dry_run:
            print(f"[DRY-RUN] Would write {buck_path}")
            print(content)
            print("---")
        else:
            buck_path.write_text(content)
            print(f"  wrote {rel_path}/BUCK")
            generated += 1

    print(f"\nSummary:", file=sys.stderr)
    print(f"  Generated: {generated}", file=sys.stderr)
    print(f"  Skipped (existing): {skipped_existing}", file=sys.stderr)
    print(f"  Skipped (no targets): {skipped_no_targets}", file=sys.stderr)
    print(f"  Errors: {len(errors)}", file=sys.stderr)
    if errors:
        for name, err in errors:
            print(f"    {name}: {err}", file=sys.stderr)


if __name__ == "__main__":
    main()
