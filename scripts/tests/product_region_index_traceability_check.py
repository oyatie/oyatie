#!/usr/bin/env python3
"""Fail closed on stale local references in the product/region index.

Scope: docs/products/README.md only. This is a narrow traceability gate for the
C1-6 lane; it does not validate product runtime readiness or pack completeness.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
INDEX = REPO_ROOT / "docs" / "products" / "README.md"
LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def is_local_reference(target: str) -> bool:
    return not (
        target.startswith("http://")
        or target.startswith("https://")
        or target.startswith("mailto:")
        or target.startswith("#")
    )


def strip_anchor(target: str) -> str:
    return target.split("#", 1)[0]


def main() -> int:
    text = INDEX.read_text(encoding="utf-8")
    failures: list[str] = []
    for match in LINK_RE.finditer(text):
        raw_target = match.group(1).strip()
        if not is_local_reference(raw_target):
            continue
        target = strip_anchor(raw_target)
        if not target:
            continue
        resolved = (INDEX.parent / target).resolve()
        try:
            resolved.relative_to(REPO_ROOT)
        except ValueError:
            failures.append(f"escapes repository root: {raw_target}")
            continue
        if not resolved.exists():
            failures.append(f"missing local target: {raw_target}")
    if failures:
        print("Product/region index traceability check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("PASS: docs/products/README.md local links resolve; planned PRD slots are non-links.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
