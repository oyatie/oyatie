#!/usr/bin/env python3
"""Fail closed when active CI/CD/build/script lanes regress from Buck2 to Cargo.

The scanner is intentionally policy-file driven so P0.0 additions must be
mapped into the automated chain instead of relying on operator memory.
Historical ADR prose is handled by explicit amendment markers; active lanes
are scanned for executable Cargo commands and legacy required contexts.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(
    __import__("os").environ.get("OYA_REPO_ROOT", Path(__file__).resolve().parents[2])
).resolve()


def load_policy(path: Path) -> dict:
    with path.open() as fh:
        return json.load(fh)


def rel(path: str) -> Path:
    return REPO_ROOT / path


def iter_lines(path: Path):
    try:
        text = path.read_text(errors="replace")
    except FileNotFoundError:
        yield 0, "<missing>"
        return
    for index, line in enumerate(text.splitlines(), 1):
        yield index, line


def expand_policy_paths(policy: dict, file_key: str, glob_key: str) -> list[str]:
    paths: list[str] = []
    seen: set[str] = set()
    for file_name in policy.get(file_key, []):
        if file_name not in seen:
            paths.append(file_name)
            seen.add(file_name)
    for pattern in policy.get(glob_key, []):
        matches = sorted(
            path.as_posix()
            for path in REPO_ROOT.glob(pattern)
            if path.is_file()
        )
        if not matches:
            paths.append(f"<missing-glob:{pattern}>")
            continue
        for file_name in matches:
            if file_name not in seen:
                paths.append(file_name)
                seen.add(file_name)
    return paths


def forbidden_cargo_regex(subcommands: list[str]) -> re.Pattern[str]:
    joined = "|".join(re.escape(cmd) for cmd in subcommands)
    return re.compile(
        rf"(^|[;&|(`]|\s)cargo\s+(\+[^\s]+\s+)?({joined})(\s|$)",
        re.IGNORECASE,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--policy",
        default="specs/buck2-authority-policy.json",
        help="Path to the Buck2 authority policy JSON, relative to repo root by default.",
    )
    args = parser.parse_args()

    policy_path = Path(args.policy)
    if not policy_path.is_absolute():
        policy_path = REPO_ROOT / policy_path
    policy = load_policy(policy_path)

    failures: list[str] = []
    cargo_re = forbidden_cargo_regex(policy["forbidden_cargo_subcommands"])

    command_scan_files = expand_policy_paths(
        policy, "command_scan_files", "command_scan_globs"
    )
    for file_name in command_scan_files:
        if file_name.startswith("<missing-glob:"):
            failures.append(f"command-scan glob matched no files: {file_name[14:-1]}")
            continue
        file_path = rel(file_name)
        if not file_path.is_file():
            failures.append(f"missing command-scan file: {file_name}")
            continue
        for line_no, line in iter_lines(file_path):
            if cargo_re.search(line):
                failures.append(
                    f"{file_name}:{line_no}: forbidden Cargo executable lane: {line.strip()}"
                )

    forbidden_contexts = set(policy["forbidden_status_contexts"])
    for file_name in policy["status_context_scan_files"]:
        file_path = rel(file_name)
        if not file_path.is_file():
            failures.append(f"missing status-context-scan file: {file_name}")
            continue
        text = file_path.read_text(errors="replace")
        for context in sorted(forbidden_contexts):
            if context in text:
                failures.append(
                    f"{file_name}: forbidden legacy status context {context!r}; use oya-ci-required"
                )

    for file_name, anchors in policy["required_anchors"].items():
        file_path = rel(file_name)
        if not file_path.is_file():
            failures.append(f"missing required-anchor file: {file_name}")
            continue
        text = file_path.read_text(errors="replace")
        for anchor in anchors:
            if anchor not in text:
                failures.append(f"{file_name}: missing required Buck2 authority anchor {anchor!r}")

    for group in policy.get("required_glob_anchors", []):
        pattern = group["glob"]
        anchors = group["anchors"]
        matches = sorted(path for path in REPO_ROOT.glob(pattern) if path.is_file())
        if not matches:
            failures.append(f"required-anchor glob matched no files: {pattern}")
            continue
        for file_path in matches:
            file_name = file_path.relative_to(REPO_ROOT).as_posix()
            text = file_path.read_text(errors="replace")
            for anchor in anchors:
                if anchor not in text:
                    failures.append(
                        f"{file_name}: missing required Buck2 authority anchor {anchor!r}"
                    )

    amendment = policy["required_adr_amendment_text"]
    for file_name in policy["adr_amendment_files"]:
        file_path = rel(file_name)
        if not file_path.is_file():
            failures.append(f"missing ADR amendment file: {file_name}")
            continue
        text = file_path.read_text(errors="replace")
        if amendment not in text or "specs/buck2-authority-policy.json" not in text:
            failures.append(
                f"{file_name}: missing {amendment!r} and policy cross-reference"
            )

    release_exception_ids = {
        item["id"] for item in policy.get("allowed_cargo_exceptions", [])
    }
    if "production-release-image-binary-optimization" not in release_exception_ids:
        failures.append("policy lacks production release image/binary Cargo exception")
    if "buck2-graph-metadata-only" not in release_exception_ids:
        failures.append("policy lacks metadata-only Buck2 graph exception")

    if failures:
        print("buck2-authority-policy: RED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(
        json.dumps(
            {
                "verdict": "PASS",
                "policy": str(policy_path.relative_to(REPO_ROOT)),
                "command_scan_files": len(command_scan_files),
                "command_scan_globs": len(policy.get("command_scan_globs", [])),
                "status_context_scan_files": len(policy["status_context_scan_files"]),
                "adr_amendment_files": len(policy["adr_amendment_files"]),
                "authority_context": policy["target_authority"]["required_context"],
                "claim_boundary": policy["claim_boundary"],
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
