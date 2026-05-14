#!/usr/bin/env python3
"""Static Stage 0 Application-shell prerequisite check.

This intentionally does not deploy anything. It proves the local repository still
has the prerequisite source surfaces that the later M02 Stage 0 deployment gate
will need before scripts/check.sh proceeds to expensive workspace gates.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
REQUIRED_PATHS = [
    "Cargo.toml",
    "crates/oya-application-app/Cargo.toml",
    "crates/oya-application-app/src/lib.rs",
    "docs/decisions/ADR-0061-application-b2b-unified-shell.md",
]


def fail(message: str) -> int:
    print(f"stage0-application-shell-prereqs: {message}", file=sys.stderr)
    return 1


def read_workspace_members() -> list[str]:
    cargo_toml = (ROOT / "Cargo.toml").read_text()
    members: list[str] = []
    in_members = False
    for raw_line in cargo_toml.splitlines():
        line = raw_line.strip()
        if line.startswith("members") and "[" in line:
            in_members = True
            continue
        if in_members and line.startswith("]"):
            break
        if in_members and line.startswith('"'):
            members.append(line.split('"', 2)[1])
    return members


def run_cargo_metadata() -> dict:
    proc = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or "cargo metadata failed")
    return json.loads(proc.stdout)


def check_repo() -> list[str]:
    errors: list[str] = []
    for rel_path in REQUIRED_PATHS:
        if not (ROOT / rel_path).exists():
            errors.append(f"missing required path: {rel_path}")

    members = read_workspace_members()
    if "crates/oya-application-app" not in members:
        errors.append("workspace members does not include crates/oya-application-app")

    try:
        metadata = run_cargo_metadata()
    except Exception as exc:  # pragma: no cover - command failure path
        errors.append(str(exc))
    else:
        packages = {package["name"]: package for package in metadata.get("packages", [])}
        app = packages.get("oya-application-app")
        if app is None:
            errors.append("cargo metadata does not include oya-application-app")
        else:
            if app.get("edition") != "2024":
                errors.append(f"oya-application-app edition is {app.get('edition')}, expected 2024")
            if app.get("rust_version") != "1.95.0":
                errors.append(
                    "oya-application-app rust-version is "
                    f"{app.get('rust_version')}, expected 1.95.0"
                )
    return errors


def self_test() -> int:
    members = read_workspace_members()
    if "crates/oya-application-app" not in members:
        return fail("self-test failed: application app workspace member missing")
    errors = check_repo()
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return fail("self-test failed")
    print("stage0 application shell prereqs self-test passed")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true", help="run repository preflight checks")
    args = parser.parse_args(argv)
    if args.self_test:
        return self_test()
    errors = check_repo()
    if errors:
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return fail("prerequisite check failed")
    print("stage0 application shell prereqs passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
