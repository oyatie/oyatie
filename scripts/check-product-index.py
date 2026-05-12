#!/usr/bin/env python3
"""Validate product PRD index and machine-readable product catalog stay in sync."""

import json
import pathlib
import re
import sys

root = pathlib.Path(__file__).resolve().parents[1]
readme = (root / "docs/products/README.md").read_text()
try:
    axis_section = readme.split("### Axis products (7)", 1)[1].split("### Vertical products", 1)[0]
except IndexError:
    sys.exit("product README is missing the axis/vertical section markers")
axis_rows = [
    line
    for line in axis_section.splitlines()
    if line.startswith("| ") and not line.startswith("| Product") and not line.startswith("|---")
]
if len(axis_rows) != 7:
    sys.exit(f"expected 7 axis rows, found {len(axis_rows)}")
required_axis_rows = {
    "saas-platform/PRD.md": "SaaS Platform",
    "workspace/PRD.md": "Workspace",
    "foundry/PRD.md": "Foundry",
    "cloud/PRD.md": "Cloud Provider",
    "search/PRD.md": "Search",
    "ads-analytics/PRD.md": "Ads + Analytics",
    "Vertical Industry Cloud": "Vertical Industry Cloud",
}
for needle, label in required_axis_rows.items():
    if not any(needle in row for row in axis_rows):
        sys.exit(f"missing axis product row for {label}")
if sum("foundry/PRD.md" in row for row in axis_rows) != 1:
    sys.exit("Foundry appears more than once in the axis product table")

catalog = json.loads((root / "docs/machine-readable/catalog.json").read_text())
for product_id in ["saas-platform", "workspace", "foundry", "cloud", "search", "ads-analytics"]:
    if product_id not in catalog["products"]:
        sys.exit(f"machine-readable catalog missing product {product_id}")

missing_paths = []
for product_id, record in catalog["products"].items():
    path = record.get("prd_path")
    if path and not (root / path).exists():
        missing_paths.append(f"{product_id}:{path}")
if missing_paths:
    sys.exit("machine-readable catalog references missing PRDs: " + ", ".join(missing_paths))

print("product index and catalog mirror check passed")
