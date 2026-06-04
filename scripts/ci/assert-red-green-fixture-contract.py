#!/usr/bin/env python3
"""Validate Phase-0 RED/GREEN fixture coverage registry.

AC-0.14 is local/static registry evidence only. It verifies that checked-in
Phase-0 gate targets have explicit GOOD and BAD fixture/probe markers, that the
markers remain present, and that the contract keeps live readiness claims false.
It never runs live CI, posts statuses, mutates branch protection, or proves
Phase-0 completion.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

DEFAULT_SPEC = Path("specs/red-green-fixture-contract.json")
FALSE_CLAIMS = (
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)
# Policy anchor: forbidden_true_or_missing_claim_p0_0_green
REQUIRED_ENTRY_FIELDS = (
    "id",
    "buck2_target",
    "test_paths",
    "green_markers",
    "red_markers",
    "non_claim_markers",
)
TARGET_RE = re.compile(r"name\s*=\s*\"([^\"]+)\"")


def load_json(path: Path) -> dict[str, Any]:
    with path.open() as fh:
        data = json.load(fh)
    if not isinstance(data, dict):
        raise TypeError(f"{path}: expected JSON object")
    return data


def read(path: Path) -> str:
    return path.read_text(errors="replace")


def rel(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def display_path(path: Path, root: Path) -> str:
    try:
        return rel(path, root)
    except ValueError:
        return str(path)


def buck_targets(root: Path) -> set[str]:
    return set(TARGET_RE.findall(read(root / "BUCK")))


def matrix_rows(root: Path) -> dict[str, dict[str, Any]]:
    matrix_path = root / "specs/phase0-automation-matrix.json"
    if not matrix_path.is_file():
        return {}
    matrix = load_json(matrix_path)
    rows = matrix.get("seed_rows", [])
    return {row.get("id"): row for row in rows if isinstance(row, dict) and isinstance(row.get("id"), str)}


def marker_tokens(marker: dict[str, Any]) -> list[str]:
    contains = marker.get("contains")
    if isinstance(contains, str):
        return [contains]
    if isinstance(contains, list):
        return [item for item in contains if isinstance(item, str)]
    return []


def validate_marker(root: Path, marker: dict[str, Any], prefix: str, failures: list[str]) -> None:
    marker_path = marker.get("path")
    if not isinstance(marker_path, str) or not marker_path:
        failures.append(f"{prefix}:marker_missing_path")
        return
    path = root / marker_path
    if not path.is_file():
        failures.append(f"{prefix}:marker_path_missing:{marker_path}")
        return
    tokens = marker_tokens(marker)
    if not tokens:
        failures.append(f"{prefix}:marker_missing_contains:{marker_path}")
        return
    text = read(path)
    for token in tokens:
        if token not in text:
            failures.append(f"{prefix}:marker_text_missing:{marker_path}:{token}")


def validate_spec(root: Path, spec: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    boundary = spec.get("claim_boundary") if isinstance(spec.get("claim_boundary"), dict) else {}
    if boundary.get("red_green_fixture_contract_measured") is not True:
        failures.append("red_green_fixture_contract_not_measured")
    for claim in FALSE_CLAIMS:
        if boundary.get(claim) is not False:
            failures.append(f"forbidden_true_or_missing_claim_{claim}")
    automated_chain = "\n".join(str(item) for item in spec.get("automated_chain", []))
    if "//:phase0-red-green-fixture-contract-check" not in automated_chain:
        failures.append("missing_buck2_target_in_automated_chain")
    if "scripts/ci/assert-red-green-fixture-contract.py" not in automated_chain:
        failures.append("missing_checker_in_automated_chain")
    return failures


def validate_entry(root: Path, entry: dict[str, Any], targets: set[str], rows: dict[str, dict[str, Any]], buck_text: str) -> tuple[dict[str, Any], list[str]]:
    failures: list[str] = []
    entry_id = str(entry.get("id") or "<missing-id>")
    for field in REQUIRED_ENTRY_FIELDS:
        if field not in entry:
            failures.append(f"{entry_id}:missing_required_field:{field}")
    target = entry.get("buck2_target")
    if not isinstance(target, str) or not target.startswith("//:"):
        failures.append(f"{entry_id}:invalid_buck2_target")
        target_name = ""
    else:
        target_name = target.removeprefix("//:")
        if target_name not in targets:
            failures.append(f"{entry_id}:buck2_target_missing:{target}")
    test_paths = entry.get("test_paths") if isinstance(entry.get("test_paths"), list) else []
    if not test_paths:
        failures.append(f"{entry_id}:missing_test_paths")
    for path_value in test_paths:
        if not isinstance(path_value, str):
            failures.append(f"{entry_id}:test_path_not_string")
            continue
        if not (root / path_value).is_file():
            failures.append(f"{entry_id}:test_path_missing:{path_value}")
        if path_value not in buck_text:
            failures.append(f"{entry_id}:test_path_not_wired_in_buck:{path_value}")

    row_id = entry.get("automation_matrix_row_id")
    if isinstance(row_id, str):
        row = rows.get(row_id)
        if not row:
            failures.append(f"{entry_id}:automation_matrix_row_missing:{row_id}")
        else:
            row_blob = json.dumps(row, sort_keys=True)
            if isinstance(target, str) and target not in row_blob:
                failures.append(f"{entry_id}:automation_matrix_row_missing_target:{target}")

    for marker_kind in ("green_markers", "red_markers", "non_claim_markers"):
        markers = entry.get(marker_kind)
        if not isinstance(markers, list) or not markers:
            failures.append(f"{entry_id}:missing_{marker_kind}")
            continue
        for marker in markers:
            if not isinstance(marker, dict):
                failures.append(f"{entry_id}:{marker_kind}:marker_not_object")
                continue
            validate_marker(root, marker, f"{entry_id}:{marker_kind}", failures)

    result = {
        "id": entry_id,
        "buck2_target": target,
        "test_path_count": len(test_paths),
        "green_marker_count": len(entry.get("green_markers", [])) if isinstance(entry.get("green_markers"), list) else 0,
        "red_marker_count": len(entry.get("red_markers", [])) if isinstance(entry.get("red_markers"), list) else 0,
        "non_claim_marker_count": len(entry.get("non_claim_markers", [])) if isinstance(entry.get("non_claim_markers"), list) else 0,
        "automation_matrix_row_id": row_id,
        "failures": failures,
    }
    return result, failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--spec", default=str(DEFAULT_SPEC))
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    spec_path = Path(args.spec)
    if not spec_path.is_absolute():
        spec_path = root / spec_path

    failures: list[str] = []
    if not spec_path.is_file():
        failures.append("missing_contract_spec")
        spec: dict[str, Any] = {}
    else:
        spec = load_json(spec_path)
        failures.extend(validate_spec(root, spec))

    targets = buck_targets(root) if (root / "BUCK").is_file() else set()
    buck_text = read(root / "BUCK") if (root / "BUCK").is_file() else ""
    rows = matrix_rows(root)
    entries = spec.get("fixture_contract_entries", []) if isinstance(spec.get("fixture_contract_entries"), list) else []
    if not entries:
        failures.append("missing_fixture_contract_entries")

    entry_results: list[dict[str, Any]] = []
    for entry in entries:
        if not isinstance(entry, dict):
            failures.append("fixture_contract_entry_not_object")
            continue
        result, entry_failures = validate_entry(root, entry, targets, rows, buck_text)
        entry_results.append(result)
        failures.extend(entry_failures)

    required_minimum = spec.get("minimum_entry_count")
    if isinstance(required_minimum, int) and len(entries) < required_minimum:
        failures.append("entry_count_below_minimum")

    result = {
        "authority_boundary": "AC-0.14 local/static RED/GREEN fixture registry evidence only; no status mutation, live required-context authority, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven",
        "red_green_fixture_contract_measured": not failures,
        "status_mutation_performed": False,
        "protected_branch_authority_proven": False,
        "live_required_context_execution_proven": False,
        "p0_0_green": False,
        "phase0_complete": False,
        "production_ready": False,
        "hyperscaler_grade": False,
        "contract_spec": display_path(spec_path, root) if spec_path.exists() else str(spec_path),
        "entry_count": len(entries),
        "buck2_target_count": len({entry.get("buck2_target") for entry in entries if isinstance(entry, dict)}),
        "green_marker_count": sum(item.get("green_marker_count", 0) for item in entry_results),
        "red_marker_count": sum(item.get("red_marker_count", 0) for item in entry_results),
        "non_claim_marker_count": sum(item.get("non_claim_marker_count", 0) for item in entry_results),
        "entries": entry_results,
        "verdict": "PASS" if not failures else "FAIL",
        "failures": sorted(set(failures)),
    }
    rendered = json.dumps(result, sort_keys=True)
    if args.json or result["verdict"] == "PASS":
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if result["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
