#!/usr/bin/env python3
"""Validate product PRD and regional-pack index traceability.

This is a claim-control gate: it proves index/link consistency only. It does
not assert product runtime readiness, pack completeness, or hyperscaler parity.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
PRODUCT_INDEX = REPO_ROOT / "docs" / "products" / "README.md"
PRODUCTS_DIR = REPO_ROOT / "docs" / "products"
LOCALIZATION_INDEX = REPO_ROOT / "docs" / "localization-packs" / "INDEX.md"
PACKS_DIR = REPO_ROOT / "packs"

LINK_RE = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
PATH_TOKEN_RE = re.compile(r"`([^`]+/PRD\.md)`")


def fail(message: str) -> NoReturn:
    print(f"product-region index traceability check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing {path.relative_to(REPO_ROOT)}")


def section(text: str, heading: str, next_heading_prefix: str = "### ") -> str:
    marker = f"### {heading}"
    start = text.find(marker)
    require(start >= 0, f"missing section {marker!r}")
    rest = text[start + len(marker):]
    next_match = re.search(rf"\n{re.escape(next_heading_prefix)}", rest)
    if next_match:
        return rest[: next_match.start()]
    return rest


def existing_product_prds() -> set[str]:
    return {
        path.relative_to(PRODUCTS_DIR).as_posix()
        for path in PRODUCTS_DIR.glob("*/PRD.md")
        if path.is_file()
    }


def authored_links(authored_section: str) -> set[str]:
    return {
        target
        for _label, target in LINK_RE.findall(authored_section)
        if target.endswith("/PRD.md")
    }


def planned_slots(planned_section: str) -> set[str]:
    return {
        token.removeprefix("docs/products/")
        for token in PATH_TOKEN_RE.findall(planned_section)
    }


def validate() -> None:
    text = read(PRODUCT_INDEX)
    authored = section(text, "Authored product PRDs")
    planned = section(text, "Planned product PRD slots (not yet authored)")
    utilities = section(text, "Cross-product utilities")

    actual_prds = existing_product_prds()
    linked_prds = authored_links(authored)
    missing_from_index = sorted(actual_prds - linked_prds)
    stale_authored_links = sorted(linked_prds - actual_prds)
    require(not missing_from_index, f"authored PRD(s) missing from index: {missing_from_index}")
    require(not stale_authored_links, f"authored PRD link(s) do not exist: {stale_authored_links}")

    planned_paths = planned_slots(planned)
    planned_existing = sorted(path for path in planned_paths if (PRODUCTS_DIR / path).exists())
    require(not planned_existing, f"planned slot(s) now exist and must move to authored table: {planned_existing}")

    planned_markdown_links = [target for _label, target in LINK_RE.findall(planned) if target.endswith("/PRD.md")]
    require(not planned_markdown_links, f"planned slots must remain non-link code paths: {planned_markdown_links}")

    require("../localization-packs/INDEX.md" in utilities, "regional packs row must link docs/localization-packs/INDEX.md")
    require("../../packs/" in utilities, "regional packs row must link repo-root packs/")
    require(LOCALIZATION_INDEX.exists(), "docs/localization-packs/INDEX.md must exist")
    require(PACKS_DIR.is_dir(), "repo-root packs/ directory must exist")
    require("traceability only" in utilities.lower(), "regional-pack row must keep traceability-only claim ceiling")
    require("does not claim pack runtime readiness" in utilities.lower(), "regional-pack row must deny pack runtime-readiness claim")

    forbidden_runtime_claims = [
        "runtime ready",
        "production ready",
        "hyperscaler ready",
        "pack completeness",
    ]
    lower_text = text.lower()
    offenders = [claim for claim in forbidden_runtime_claims if claim in lower_text]
    require(not offenders, f"index contains forbidden readiness/completeness wording: {offenders}")


def main() -> None:
    validate()
    print("product-region index traceability check passed")


if __name__ == "__main__":
    main()
