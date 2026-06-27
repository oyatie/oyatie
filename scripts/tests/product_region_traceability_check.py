#!/usr/bin/env python3
"""Validate product PRD and regional-pack traceability indexes.

This is an index/spec traceability gate only. It does not assert product runtime,
regional-pack activation, tenant readiness, or production/hyperscaler maturity.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
PRODUCT_INDEX = REPO_ROOT / "docs" / "products" / "README.md"
LOCALIZATION_INDEX = REPO_ROOT / "docs" / "localization-packs" / "INDEX.md"
PRODUCT_ROOT = REPO_ROOT / "docs" / "products"
PACK_ROOT = REPO_ROOT / "packs"

FORBIDDEN_POSITIVE_PATTERNS = [
    re.compile(pattern)
    for pattern in [
        r"\bproduction\s+ready\b",
        r"\bruntime\s+ready\b",
        r"\btenant\s+ready\b",
        r"\bactive\s+tenants?\b",
        r"\bregional\s+pack\s+active\b",
        r"\bproduct\s+runtime\s+readiness\b.{0,30}\b(achieved|complete|green|passed)\b",
    ]
]


def fail(message: str) -> NoReturn:
    print(f"product/region traceability check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def markdown_links(text: str) -> list[str]:
    return [target.strip() for _label, target in re.findall(r"\[([^\]]+)\]\(([^)]+)\)", text)]


def existing_relative_link(base: Path, target: str) -> bool:
    if target.startswith(("http://", "https://", "#", "mailto:")) or target == "TBD":
        return True
    clean = target.split("#", 1)[0]
    if not clean:
        return True
    return (base / clean).resolve().exists()


def require_links_resolve(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    missing = [target for target in markdown_links(text) if not existing_relative_link(path.parent, target)]
    require(not missing, f"{path.relative_to(REPO_ROOT)} has unresolved markdown links: {missing}")


def contains_forbidden_positive_claim(text: str) -> bool:
    normalized = re.sub(r"\s+", " ", text.lower())
    return any(pattern.search(normalized) for pattern in FORBIDDEN_POSITIVE_PATTERNS)


def main() -> None:
    require(PRODUCT_INDEX.exists(), "missing docs/products/README.md")
    require(LOCALIZATION_INDEX.exists(), "missing docs/localization-packs/INDEX.md")
    require_links_resolve(PRODUCT_INDEX)
    require_links_resolve(LOCALIZATION_INDEX)

    product_text = PRODUCT_INDEX.read_text(encoding="utf-8")
    localization_text = LOCALIZATION_INDEX.read_text(encoding="utf-8")

    require("traceability surface, not a runtime-readiness claim" in product_text, "product index must keep non-runtime claim ceiling")
    require("Spec-backed surfaces without product PRD files yet" in product_text, "product index must document non-PRD spec-backed surfaces")
    require("Repository-local pack directories" in localization_text, "localization index must list repository-local pack directories")
    require("traceability inventory only" in localization_text, "localization index must keep pack non-runtime claim ceiling")
    require(not contains_forbidden_positive_claim(product_text), "product index contains forbidden runtime/readiness claim wording")
    require(not contains_forbidden_positive_claim(localization_text), "localization index contains forbidden runtime/readiness claim wording")

    prds = sorted(path.relative_to(PRODUCT_ROOT).as_posix() for path in PRODUCT_ROOT.glob("*/PRD.md"))
    missing_prds = [prd for prd in prds if f"]({prd})" not in product_text]
    require(not missing_prds, f"product index omits existing PRD files: {missing_prds}")

    unresolved_prd_links = [target for target in markdown_links(product_text) if target.endswith("/PRD.md") and not (PRODUCT_INDEX.parent / target).resolve().exists()]
    require(not unresolved_prd_links, f"product index links missing PRD files: {unresolved_prd_links}")

    pack_dirs = sorted(path.name for path in PACK_ROOT.iterdir() if path.is_dir())
    missing_pack_rows = [code for code in pack_dirs if f"`{code}`" not in localization_text or f"](../../packs/{code}/)" not in localization_text]
    require(not missing_pack_rows, f"localization index omits pack directories: {missing_pack_rows}")

    overview_docs = sorted(path.stem for path in LOCALIZATION_INDEX.parent.glob("*.md") if path.name != "INDEX.md")
    for code in overview_docs:
        require(f"{code}.md" in localization_text, f"localization index omits overview doc {code}.md")

    print("product/region traceability check passed: docs/products/README.md and docs/localization-packs/INDEX.md")


if __name__ == "__main__":
    main()
