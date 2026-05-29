#!/usr/bin/env bash
# Reject deployable release placeholders that can silently pass reviews.
#
# Usage:
#   scripts/reject-placeholder-digests.sh [repo-root]
#
# The gate intentionally allows historical evidence and test fixtures, but
# fails active source/IaC surfaces carrying:
#   * low-entropy sha256 placeholders such as 64 repeated hex characters
#   * dogfood placeholder tags/annotations that look deployable
set -euo pipefail

root="${1:-.}"

python3 - "$root" <<'PY'
from __future__ import annotations

import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1]).resolve()

repeated_digest = re.compile(r"(?<![A-Za-z0-9_-])sha256:([0-9a-f])\1{63}(?![0-9a-f])")
placeholder_terms = [
    "release-" + "digest-required",
    "release-signed-image-" + "digest-required",
]

ignored_names = {".git", "target", "node_modules", ".next", "dist", "build"}
ignored_prefixes = (
    "docs/decisions/",
    "docs/raw/",
    "docs/archive/",
    "evidence/",
    "crates/oya-llm-gateway",
    "microservices/llm-gateway/",
)
ignored_suffixes = (
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".ico",
    ".pdf",
    ".zip",
    ".gz",
    ".tar",
    ".tgz",
)


def relpath(path: pathlib.Path) -> str:
    return path.relative_to(root).as_posix()


def ignored(path: pathlib.Path) -> bool:
    rel = relpath(path)
    if any(part in ignored_names for part in path.parts):
        return True
    if rel.endswith(ignored_suffixes):
        return True
    if any(rel.startswith(prefix) for prefix in ignored_prefixes):
        return True
    if "/tests/" in rel or rel.startswith("tests/") or rel.endswith("_test.rs"):
        return True
    if rel == "scripts/reject-placeholder-digests.sh":
        return True
    return False


failures: list[str] = []
for path in sorted(root.rglob("*")):
    if not path.is_file() or ignored(path):
        continue
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        continue

    matches: list[str] = []
    if repeated_digest.search(text):
        matches.append("low-entropy repeated sha256 digest")
    for term in placeholder_terms:
        if term in text:
            matches.append(term)
    if matches:
        failures.append(f"{relpath(path)}: {', '.join(matches)}")

if failures:
    print("release placeholder digest gate failed:", file=sys.stderr)
    for failure in failures:
        print(f"  - {failure}", file=sys.stderr)
    sys.exit(1)

print("release placeholder digest gate passed")
PY
