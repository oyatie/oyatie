#!/usr/bin/env python3
"""Validate AC-0.2 status enum and spec/manifest drift fixtures.

This checker is local/static fixture evidence only. It validates the checked-in
3-axis status enum registry and GOOD/BAD fixtures for invalid status values,
retired REAL live-field tokens, spec/code/manifest mismatches, and status drift.
It never posts statuses, mutates branch protection, proves full manifest/PRD
conformance, or claims P0.0/Phase-0 completion.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

DEFAULT_REGISTRY = Path("specs/status-enum-registry.json")
DEFAULT_FIXTURE_DIR = Path("specs/fixtures/phase0-status-enum-drift")
AXES = ("decision", "maturity", "constraint")
REQUIRED_AXIS_FIELDS = ("decision_status", "maturity_status", "constraint_status")
FALSE_CLAIMS = (
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "full_manifest_prd_conformance_proven",
    "status_drift_live_gate_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)
FIXTURE_FALSE_CLAIMS = (
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)
REAL_TOKEN_RE = re.compile(r"\bREAL\b")


def load_json(path: Path) -> dict[str, Any]:
    with path.open() as fh:
        data = json.load(fh)
    if not isinstance(data, dict):
        raise TypeError(f"{path}: expected JSON object")
    return data


def string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, str)]


def object_list(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, dict)]


def display_path(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return str(path)


def validate_false_claims(mapping: dict[str, Any], failures: list[str], *, claims: tuple[str, ...] = FALSE_CLAIMS, prefix: str = "") -> None:
    for claim in claims:
        if mapping.get(claim) is not False:
            failures.append(f"{prefix}forbidden_true_or_missing_claim_{claim}")


def validate_axis_fields(fields: dict[str, Any], allowed_by_field: dict[str, set[str]], failures: list[str], *, prefix: str = "") -> None:
    for field in REQUIRED_AXIS_FIELDS:
        value = fields.get(field)
        if not isinstance(value, str) or not value:
            failures.append(f"{prefix}missing_status_axis_field:{field}")
            continue
        if value not in allowed_by_field.get(field, set()):
            failures.append(f"{prefix}invalid_status_enum_value:{field}:{value}")
        if REAL_TOKEN_RE.search(value):
            failures.append(f"{prefix}retired_real_token_live_field:{field}")


def validate_registry(root: Path, registry: dict[str, Any]) -> tuple[list[str], dict[str, Any], dict[str, set[str]]]:
    failures: list[str] = []
    boundary = registry.get("claim_boundary") if isinstance(registry.get("claim_boundary"), dict) else {}
    if boundary.get("status_enum_registry_published") is not True:
        failures.append("status_enum_registry_not_published")
    if boundary.get("status_drift_fixture_contract_measured") is not True:
        failures.append("status_drift_fixture_contract_not_measured")
    validate_false_claims(boundary, failures)

    axes = registry.get("axes") if isinstance(registry.get("axes"), dict) else {}
    allowed_by_field: dict[str, set[str]] = {}
    for axis in AXES:
        axis_spec = axes.get(axis) if isinstance(axes.get(axis), dict) else {}
        field = axis_spec.get("field")
        allowed = set(string_list(axis_spec.get("allowed_values")))
        expected_field = f"{axis}_status"
        if field != expected_field:
            failures.append(f"axis_field_mismatch:{axis}")
        if not allowed:
            failures.append(f"axis_allowed_values_missing:{axis}")
        if "REAL" in allowed:
            failures.append(f"retired_real_token_allowed:{axis}")
        allowed_by_field[expected_field] = allowed

    field_contract = registry.get("field_contract") if isinstance(registry.get("field_contract"), dict) else {}
    if tuple(string_list(field_contract.get("required_axis_fields"))) != REQUIRED_AXIS_FIELDS:
        failures.append("required_axis_fields_drift")
    if field_contract.get("full_manifest_prd_conformance_proven") is not False:
        failures.append("forbidden_true_or_missing_field_contract_full_manifest_prd_conformance_proven")
    if "REAL" not in string_list(field_contract.get("retired_live_status_tokens")):
        failures.append("retired_real_token_not_registered")

    surfaces = object_list(registry.get("seed_surface_registry"))
    for surface in surfaces:
        surface_id = surface.get("surface_id") if isinstance(surface.get("surface_id"), str) else "<missing-surface-id>"
        validate_axis_fields(surface, allowed_by_field, failures, prefix=f"{surface_id}:")
        for path_field in ("spec_path", "code_path", "manifest_path"):
            path_value = surface.get(path_field)
            if not isinstance(path_value, str) or not path_value:
                failures.append(f"{surface_id}:missing_surface_path:{path_field}")
                continue
            if not (root / path_value).exists():
                failures.append(f"{surface_id}:surface_path_missing:{path_field}:{path_value}")

    summary = {
        "axis_count": len(axes),
        "allowed_value_count": sum(len(values) for values in allowed_by_field.values()),
        "seed_surface_count": len(surfaces),
    }
    return failures, summary, allowed_by_field


def validate_pair(root: Path, pair: dict[str, Any], allowed_by_field: dict[str, set[str]], observed: list[str]) -> None:
    surface_id = pair.get("surface_id") if isinstance(pair.get("surface_id"), str) else "<missing-surface-id>"
    for path_field in ("spec_path", "code_path", "manifest_path"):
        path_value = pair.get(path_field)
        if not isinstance(path_value, str) or not path_value or not (root / path_value).exists():
            observed.append("spec_code_manifest_mismatch")
            observed.append(f"spec_code_manifest_mismatch:{surface_id}:{path_field}")
    spec_fields = pair.get("spec_status_fields") if isinstance(pair.get("spec_status_fields"), dict) else {}
    manifest_fields = pair.get("manifest_status_fields") if isinstance(pair.get("manifest_status_fields"), dict) else {}
    validate_axis_fields(spec_fields, allowed_by_field, observed, prefix=f"{surface_id}:spec:")
    validate_axis_fields(manifest_fields, allowed_by_field, observed, prefix=f"{surface_id}:manifest:")
    for field in REQUIRED_AXIS_FIELDS:
        if spec_fields.get(field) != manifest_fields.get(field):
            observed.append("status_drift_mismatch")
            observed.append(f"status_drift_mismatch:{surface_id}:{field}")


def validate_fixture(root: Path, fixture: dict[str, Any], allowed_by_field: dict[str, set[str]]) -> dict[str, Any]:
    fixture_id = fixture.get("fixture_id") if isinstance(fixture.get("fixture_id"), str) else "<missing-fixture-id>"
    expected_verdict = fixture.get("expected_verdict")
    if expected_verdict not in {"GREEN", "RED"}:
        expected_verdict = "RED"
    expected_violations = set(string_list(fixture.get("expected_violations")))
    observed: list[str] = []

    boundary = fixture.get("claim_boundary") if isinstance(fixture.get("claim_boundary"), dict) else {}
    validate_false_claims(boundary, observed, claims=FIXTURE_FALSE_CLAIMS)
    status_fields = fixture.get("status_fields") if isinstance(fixture.get("status_fields"), dict) else {}
    validate_axis_fields(status_fields, allowed_by_field, observed)
    for pair in object_list(fixture.get("spec_manifest_pairs")):
        validate_pair(root, pair, allowed_by_field, observed)

    # Keep stable base classes for expected_violations while retaining details.
    base_observed = set(observed)
    for item in list(observed):
        if item.startswith("invalid_status_enum_value:"):
            base_observed.add("invalid_status_enum_value")
        if item.startswith("retired_real_token_live_field:"):
            base_observed.add("retired_real_token_live_field")
        if item.startswith("spec_code_manifest_mismatch:"):
            base_observed.add("spec_code_manifest_mismatch")
        if item.startswith("status_drift_mismatch:"):
            base_observed.add("status_drift_mismatch")
    observed_set = base_observed

    fixture_failures: list[str] = []
    if expected_verdict == "GREEN":
        if observed_set:
            fixture_failures.append(f"{fixture_id}: GREEN status-enum fixture produced violations {sorted(observed_set)}")
        if expected_violations:
            fixture_failures.append(f"{fixture_id}: GREEN fixture must not list expected_violations")
    else:
        if not observed_set:
            fixture_failures.append(f"{fixture_id}: RED status-enum fixture must produce violations")
        missing_expected = sorted(expected_violations - observed_set)
        if missing_expected:
            fixture_failures.append(f"{fixture_id}: expected violations were not observed {missing_expected}")

    return {
        "fixture_id": fixture_id,
        "expected_verdict": expected_verdict,
        "expected_violations": sorted(expected_violations),
        "observed_violations": sorted(observed_set),
        "fixture_passed": not fixture_failures,
        "failures": fixture_failures,
    }


def fixture_paths(root: Path, explicit: list[str] | None) -> list[Path]:
    if explicit:
        return [Path(item) if Path(item).is_absolute() else root / item for item in explicit]
    return sorted((root / DEFAULT_FIXTURE_DIR).glob("*.json"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--registry", default=str(DEFAULT_REGISTRY))
    parser.add_argument("--fixture", action="append", default=None)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    registry_path = Path(args.registry)
    if not registry_path.is_absolute():
        registry_path = root / registry_path

    failures: list[str] = []
    if not registry_path.is_file():
        failures.append("missing_status_enum_registry")
        registry: dict[str, Any] = {}
        registry_summary = {"axis_count": 0, "allowed_value_count": 0, "seed_surface_count": 0}
        allowed_by_field = {field: set() for field in REQUIRED_AXIS_FIELDS}
    else:
        registry = load_json(registry_path)
        registry_failures, registry_summary, allowed_by_field = validate_registry(root, registry)
        failures.extend(registry_failures)

    fixture_results: list[dict[str, Any]] = []
    for path in fixture_paths(root, args.fixture):
        if not path.is_file():
            failures.append(f"fixture_path_missing:{display_path(path, root)}")
            continue
        result = validate_fixture(root, load_json(path), allowed_by_field)
        result["path"] = display_path(path, root)
        fixture_results.append(result)
        failures.extend(result["failures"])

    expected_green = sum(1 for item in fixture_results if item["expected_verdict"] == "GREEN")
    expected_red = sum(1 for item in fixture_results if item["expected_verdict"] == "RED")
    result = {
        "authority_boundary": "AC-0.2 local/static status enum and drift fixture evidence only; no status mutation, live required-context authority, full manifest/PRD conformance, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven",
        "status_enum_registry_published": registry.get("claim_boundary", {}).get("status_enum_registry_published") is True,
        "status_drift_fixture_contract_measured": not failures,
        **registry_summary,
        "fixture_count": len(fixture_results),
        "expected_green_fixture_count": expected_green,
        "expected_red_fixture_count": expected_red,
        "fixture_results": fixture_results,
        "status_mutation_performed": False,
        "protected_branch_authority_proven": False,
        "live_required_context_execution_proven": False,
        "full_manifest_prd_conformance_proven": False,
        "status_drift_live_gate_proven": False,
        "p0_0_green": False,
        "phase0_complete": False,
        "production_ready": False,
        "hyperscaler_grade": False,
        "verdict": "PASS" if not failures else "FAIL",
        "failures": sorted(set(failures)),
    }
    rendered = json.dumps(result, sort_keys=True)
    if args.json or not failures:
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
