#!/usr/bin/env python3
"""Fail when active Rust pins drift from rust-toolchain.toml."""

from __future__ import annotations

import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path


EXCLUDED_PREFIXES = (
    "cloud/cloud-kernel/",
    "docs/audit/",
    "docs/research/",
    "evidence/",
)


ACTIVE_TEXT_PATHS = (
    "docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md",
    "docs/architecture/",
    "docs/automation/",
    "docs/decisions/ADR-0392-buck2-canonical-build-graph.md",
    "docs/plans/",
    "docs/standards/",
    "specs/oss-stewardship-registry.json",
)


def excluded(path: str) -> bool:
    return any(path.startswith(prefix) for prefix in EXCLUDED_PREFIXES)


def active_text_path(path: str) -> bool:
    return any(path == prefix or path.startswith(prefix) for prefix in ACTIVE_TEXT_PATHS)


def tracked_files(root: Path) -> list[str]:
    output = subprocess.check_output(["git", "-C", str(root), "ls-files"], text=True)
    return [
        line
        for line in output.splitlines()
        if line and not line.endswith(".generated.json") and not excluded(line)
    ]


def read_toolchain(root: Path) -> str:
    return tomllib.loads((root / "rust-toolchain.toml").read_text())["toolchain"][
        "channel"
    ]


def check_cargo(root: Path, files: list[str], want: str, problems: list[str]) -> None:
    workspace = tomllib.loads((root / "Cargo.toml").read_text())["workspace"][
        "package"
    ]["rust-version"]
    if workspace != want:
        problems.append(f"Cargo.toml workspace rust-version is {workspace}, want {want}")

    for rel in files:
        if not rel.endswith("Cargo.toml"):
            continue
        data = tomllib.loads((root / rel).read_text())
        package = data.get("package")
        if not package:
            continue
        rust_version = package.get("rust-version")
        if isinstance(rust_version, str) and rust_version != want:
            problems.append(f"{rel}: rust-version is {rust_version}, want {want}")


def check_json(root: Path, files: list[str], want: str, problems: list[str]) -> None:
    for rel in files:
        if not (rel.endswith("manifest.json") or rel.endswith("supported-oses.json")):
            continue
        data = json.loads((root / rel).read_text())
        rust = data.get("toolchain", {}).get("rust")
        if rust and rust != want:
            problems.append(f"{rel}: toolchain.rust is {rust}, want {want}")
        lts_pins = data.get("lts_pins")
        lts_rust = lts_pins.get("rust") if isinstance(lts_pins, dict) else None
        if lts_rust and lts_rust != want:
            problems.append(f"{rel}: lts_pins.rust is {lts_rust}, want {want}")
        rust_toolchain = data.get("rust_toolchain")
        if rust_toolchain and rust_toolchain != f"{want}-stable":
            problems.append(
                f"{rel}: rust_toolchain is {rust_toolchain}, want {want}-stable"
            )


def check_docker(root: Path, files: list[str], want: str, problems: list[str]) -> None:
    from_rust = re.compile(r"^FROM rust:([^\s]+)", re.MULTILINE)
    rust_arg = re.compile(r"^ARG RUST_VERSION=([^\s]+)", re.MULTILINE)
    version_tag = re.compile(r"^(1\.\d+(?:\.\d+)?)(?:-|$)")

    for rel in files:
        if not Path(rel).name.startswith("Dockerfile"):
            continue
        text = (root / rel).read_text()
        for value in rust_arg.findall(text):
            if value != want:
                problems.append(f"{rel}: ARG RUST_VERSION={value}, want {want}")
        for tag in from_rust.findall(text):
            match = version_tag.match(tag)
            if match and match.group(1) != want:
                problems.append(f"{rel}: FROM rust:{tag}, want {want}")


def check_ci_text(root: Path, files: list[str], want: str, problems: list[str]) -> None:
    stale_toolchain_path = re.compile(r"\.rustup/toolchains/(1\.\d+\.\d+)-")
    rustup_install = re.compile(r"rustup toolchain install (1\.\d+\.\d+)")
    explicit_version = re.compile(r"1\.\d+\.\d+")

    for rel in files:
        if not (rel.startswith(".github/workflows/") or rel.startswith("toolchains/")):
            continue
        text = (root / rel).read_text()
        if "toolchain: stable" in text:
            problems.append(f"{rel}: uses floating Rust stable")
        for version in stale_toolchain_path.findall(text):
            if version != want:
                problems.append(f"{rel}: cache path pins {version}, want {want}")
        for version in rustup_install.findall(text):
            if version != want:
                problems.append(f"{rel}: rustup install pins {version}, want {want}")
        if rel.startswith("toolchains/"):
            for version in explicit_version.findall(text):
                if version != want:
                    problems.append(f"{rel}: text pins {version}, want {want}")


def check_active_text(root: Path, files: list[str], want: str, problems: list[str]) -> None:
    old_pin = re.compile(r"1\.95\.0|rust:1-bookworm|rust:1\.95|RUST_VERSION=1\.82")
    rust_image_version = re.compile(r"rust:(1\.\d+(?:\.\d+)?)(?=[-\s`\"']|$)")

    for rel in files:
        if not active_text_path(rel):
            continue
        text = (root / rel).read_text(errors="ignore")
        if old_pin.search(text):
            problems.append(f"{rel}: active text still contains a stale Rust pin")
        for match in rust_image_version.finditer(text):
            if match.group(1) != want:
                problems.append(
                    f"{rel}: Rust image {match.group(0)} is not patch-pinned to {want}"
                )


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    want = read_toolchain(root)
    files = tracked_files(root)
    problems: list[str] = []

    check_cargo(root, files, want, problems)
    check_json(root, files, want, problems)
    check_docker(root, files, want, problems)
    check_ci_text(root, files, want, problems)
    check_active_text(root, files, want, problems)

    if problems:
        print("Rust toolchain drift:", file=sys.stderr)
        for problem in problems:
            print(f"- {problem}", file=sys.stderr)
        return 1
    print(f"Rust toolchain pins aligned to {want}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
