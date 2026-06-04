#!/usr/bin/env python3
"""Validate AC-0.16 automation-ratchet coverage without live authority claims.

This checker is local/static fixture evidence only. It evaluates the checked-in
Phase-0 automation matrix, the coverage registry, and the declared BAD/GREEN
fixtures so enforceable requirements cannot hide as operator procedure, map back
to `oya` CLI authority, or leave seed rows unmapped. It never posts statuses,
mutates branch protection, or claims P0.0/Phase-0 green.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

DEFAULT_MATRIX = "specs/phase0-automation-matrix.json"
DEFAULT_COVERAGE_REGISTRY = "specs/phase0-automation-coverage-registry.json"

ALLOWED_CLASSIFICATIONS = {
    "automated_blocking_now",
    "automated_advisory_until_p0_0",
    "controller_owned_in_phase_1",
    "not_automatable_human_judgment",
}
REQUIRED_ROW_FIELDS = [
    "id",
    "source_artifact",
    "requirement",
    "classification",
    "owner",
    "target_gate_or_controller",
    "blocking_fixture",
    "retirement_phase",
    "evidence_path",
    "no_new_oya_cli_surface",
]
OYA_CLI_AUTHORITY_RE = re.compile(
    r"(?:\b(?:bin/)?oya\s+(?:verify|gate)\b|\blocal\s+oya\s+(?:gate|verify|output)\b)",
    re.IGNORECASE,
)
LIVE_FALSE_FLAGS = {
    "automation_ratchet_live": False,
    "protected_branch_authority_proven": False,
    "status_mutation_performed": False,
    "live_required_context_execution_proven": False,
    "p0_0_green": False,
    "phase0_complete": False,
}


def load_json(path: str | Path) -> dict[str, Any]:
    with Path(path).open() as fh:
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


def non_empty(value: Any) -> bool:
    if isinstance(value, str):
        return bool(value.strip())
    if isinstance(value, list):
        return bool(value)
    if isinstance(value, dict):
        return bool(value)
    return value is not None


def iter_strings(value: Any) -> list[str]:
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        return [text for item in value for text in iter_strings(item)]
    if isinstance(value, dict):
        return [text for item in value.values() for text in iter_strings(item)]
    return []


def hard_authority_text(row: dict[str, Any]) -> str:
    parts: list[str] = []
    for key in ("target_gate_or_controller", "evidence_path", "blocking_fixture"):
        value = row.get(key)
        if isinstance(value, str):
            parts.append(value)
    return "\n".join(parts)


def negates_oya_authority_reference(text: str, match: re.Match[str]) -> bool:
    window = text[max(0, match.start() - 90) : min(len(text), match.end() + 120)].lower()
    return bool(
        re.search(r"\bis\s+not\s+(?:protected-branch\s+|ci\s+)?authority\b", window)
        or re.search(r"\bnot\s+(?:protected-branch\s+|ci\s+)?authority\b", window)
        or re.search(r"\bprevent\b.{0,90}\bfrom\s+becoming\b.{0,90}\bauthority\b", window)
        or re.search(r"\bmust\s+not\b.{0,120}\b(?:authority|satisf(?:y|ied))\b", window)
        or re.search(r"\bcannot\b.{0,120}\b(?:authority|satisf(?:y|ied))\b", window)
    )


def has_oya_cli_authority_reference(text: str, *, allow_negated: bool = False) -> bool:
    for match in OYA_CLI_AUTHORITY_RE.finditer(text):
        if allow_negated and negates_oya_authority_reference(text, match):
            continue
        return True
    return False


def row_maps_to_oya_cli_authority(row: dict[str, Any]) -> bool:
    if row.get("no_new_oya_cli_surface") is not True:
        return True
    if has_oya_cli_authority_reference(hard_authority_text(row)):
        return True
    for key in ("source_artifact", "requirement", "claim_boundary", "human_judgment_reason", "owner", "retirement_phase"):
        value = row.get(key)
        if isinstance(value, str) and has_oya_cli_authority_reference(value, allow_negated=True):
            return True
    return False


def unique_ids(rows: list[dict[str, Any]]) -> tuple[set[str], set[str]]:
    seen: set[str] = set()
    duplicates: set[str] = set()
    for row in rows:
        row_id = row.get("id")
        if not isinstance(row_id, str) or not row_id:
            continue
        if row_id in seen:
            duplicates.add(row_id)
        seen.add(row_id)
    return seen, duplicates


def validate_rows(
    rows: list[dict[str, Any]],
    *,
    required_fields: list[str],
    required_row_ids: list[str] | None = None,
    allowed_classifications: set[str] = ALLOWED_CLASSIFICATIONS,
) -> list[str]:
    violations: list[str] = []
    row_ids, duplicates = unique_ids(rows)
    if duplicates:
        violations.append("duplicate_row_id")

    if required_row_ids:
        missing = sorted(set(required_row_ids) - row_ids)
        if missing:
            violations.append("missing_required_row_id")

    for row in rows:
        if any(not non_empty(row.get(field)) for field in required_fields):
            violations.append("missing_or_empty_required_field")
        classification = row.get("classification")
        if classification not in allowed_classifications:
            violations.append("unknown_classification")
        if classification == "not_automatable_human_judgment" and row.get("enforceable_or_automatable") is True:
            violations.append("enforceable_or_automatable_marked_human_judgment")
        if row_maps_to_oya_cli_authority(row):
            violations.append("blocking_invariant_mapped_to_oya_cli")
        if classification == "not_automatable_human_judgment":
            reason = row.get("human_judgment_reason")
            if not isinstance(reason, str) or not reason.strip():
                violations.append("human_judgment_missing_irreducible_reason")
    return sorted(set(violations))


def validate_coverage_registry(registry: dict[str, Any], row_ids: set[str]) -> dict[str, Any]:
    violations: list[str] = []
    required_subject_ids = set(string_list(registry.get("required_subject_ids")))
    coverage_subjects = object_list(registry.get("coverage_subjects"))
    subject_ids = {subject.get("id") for subject in coverage_subjects if isinstance(subject.get("id"), str)}

    if set(string_list(registry.get("row_ids"))):
        row_ids = set(string_list(registry.get("row_ids")))

    missing_subjects = sorted(required_subject_ids - subject_ids)
    if missing_subjects:
        violations.append("missing_required_coverage_subject_id")

    duplicate_subjects = sorted(s for s in subject_ids if sum(1 for subject in coverage_subjects if subject.get("id") == s) > 1)
    if duplicate_subjects:
        violations.append("duplicate_coverage_subject_id")

    mapped_row_ids: set[str] = set()
    for subject in coverage_subjects:
        mapped = set(string_list(subject.get("mapped_row_ids")))
        if not mapped:
            violations.append("coverage_subject_without_rows")
        mapped_row_ids.update(mapped)

    missing_mapped_rows = sorted(mapped_row_ids - row_ids)
    if missing_mapped_rows:
        violations.append("coverage_mapped_row_missing")

    unmapped_rows = sorted(row_ids - mapped_row_ids)
    if unmapped_rows:
        violations.append("coverage_row_unmapped")

    for text in iter_strings(registry):
        if has_oya_cli_authority_reference(text):
            violations.append("blocking_invariant_mapped_to_oya_cli")
            break

    claim_boundary = registry.get("claim_boundary")
    if not isinstance(claim_boundary, dict) or claim_boundary.get("p0_0_green") is not False or claim_boundary.get("phase0_complete") is not False:
        violations.append("green_claim_boundary_without_live_authority")

    return {
        "required_subject_count": len(required_subject_ids),
        "coverage_subject_count": len(subject_ids),
        "mapped_row_count": len(mapped_row_ids),
        "missing_required_subject_ids": missing_subjects,
        "missing_mapped_row_ids": missing_mapped_rows,
        "unmapped_row_ids": unmapped_rows,
        "violations": sorted(set(violations)),
    }


def fixture_paths(matrix: dict[str, Any], explicit: list[str] | None) -> list[str]:
    if explicit:
        return explicit
    fixture_set = matrix.get("fixture_set") if isinstance(matrix.get("fixture_set"), dict) else {}
    return string_list(fixture_set.get("all_fixture_paths"))


def expected_from_fixture(fixture: dict[str, Any]) -> tuple[str, set[str], str]:
    expected_verdict = fixture.get("expected_verdict")
    if expected_verdict not in {"GREEN", "RED"}:
        expected_verdict = "RED"
    expected_violations = set(string_list(fixture.get("expected_violations")))
    fixture_id = fixture.get("fixture_id") if isinstance(fixture.get("fixture_id"), str) else "unknown-fixture"
    return expected_verdict, expected_violations, fixture_id


def validate_fixture(path: str, matrix: dict[str, Any], row_ids: set[str]) -> dict[str, Any]:
    fixture = load_json(path)
    expected_verdict, expected_violations, fixture_id = expected_from_fixture(fixture)
    required_fields = string_list(matrix.get("required_row_fields")) or REQUIRED_ROW_FIELDS
    observed_violations: list[str] = []

    if "rows" in fixture:
        observed_violations.extend(
            validate_rows(
                object_list(fixture.get("rows")),
                required_fields=required_fields,
                required_row_ids=string_list(fixture.get("required_row_ids")) or None,
            )
        )
    if "coverage_subjects" in fixture:
        observed_violations.extend(validate_coverage_registry(fixture, row_ids)["violations"])
    if "rows" not in fixture and "coverage_subjects" not in fixture:
        observed_violations.append("fixture_missing_rows_or_coverage_subjects")

    observed = set(observed_violations)
    fixture_failures: list[str] = []
    if expected_verdict == "GREEN":
        if observed:
            fixture_failures.append(f"{fixture_id}: GREEN automation-ratchet fixture produced violations {sorted(observed)}")
        if expected_violations:
            fixture_failures.append(f"{fixture_id}: GREEN fixture must not list expected_violations")
    else:
        if not observed:
            fixture_failures.append(f"{fixture_id}: RED automation-ratchet fixture must produce violations")
        missing_expected = sorted(expected_violations - observed)
        if missing_expected:
            fixture_failures.append(f"{fixture_id}: expected violations were not observed {missing_expected}")

    return {
        "path": path,
        "fixture_id": fixture_id,
        "expected_verdict": expected_verdict,
        "expected_violations": sorted(expected_violations),
        "observed_violations": sorted(observed),
        "fixture_passed": not fixture_failures,
        "failures": fixture_failures,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--matrix", default=DEFAULT_MATRIX)
    parser.add_argument("--coverage-registry", default=DEFAULT_COVERAGE_REGISTRY)
    parser.add_argument("--fixture", action="append", default=None, help="Fixture path to evaluate; repeat to override matrix fixture_set")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    matrix = load_json(args.matrix)
    coverage_registry = load_json(args.coverage_registry)
    required_fields = string_list(matrix.get("required_row_fields")) or REQUIRED_ROW_FIELDS
    allowed_classifications = set(string_list(matrix.get("classifications"))) or ALLOWED_CLASSIFICATIONS
    rows = object_list(matrix.get("seed_rows"))
    row_ids, duplicate_ids = unique_ids(rows)

    failures: list[str] = []
    matrix_violations = validate_rows(
        rows,
        required_fields=required_fields,
        required_row_ids=string_list(matrix.get("required_seed_row_ids")),
        allowed_classifications=allowed_classifications,
    )
    if allowed_classifications != ALLOWED_CLASSIFICATIONS:
        matrix_violations.append("classification_set_drift")
    if set(required_fields) != set(REQUIRED_ROW_FIELDS):
        matrix_violations.append("required_row_fields_drift")
    failures.extend(f"matrix:{violation}" for violation in sorted(set(matrix_violations)))

    coverage_summary = validate_coverage_registry(coverage_registry, row_ids)
    failures.extend(f"coverage_registry:{violation}" for violation in coverage_summary["violations"])

    fixtures = [validate_fixture(path, matrix, row_ids) for path in fixture_paths(matrix, args.fixture)]
    for fixture in fixtures:
        failures.extend(fixture["failures"])

    result = {
        "authority_boundary": "automation-ratchet local/static fixture evidence only; this checker never posts statuses, mutates branch protection, or claims live required-context authority",
        "matrix": args.matrix,
        "coverage_registry": args.coverage_registry,
        "required_row_fields": required_fields,
        "allowed_classifications": sorted(allowed_classifications),
        "matrix_summary": {
            "row_count": len(rows),
            "required_seed_row_count": len(string_list(matrix.get("required_seed_row_ids"))),
            "duplicate_row_ids": sorted(duplicate_ids),
            "violations": sorted(set(matrix_violations)),
        },
        "coverage_registry_summary": coverage_summary,
        "fixture_results": fixtures,
        "local_fixture_contract_proven": not failures,
        "coverage_registry_local_static_proven": not failures,
        **LIVE_FALSE_FLAGS,
        "verdict": "PASS" if not failures else "FAIL",
        "failures": failures,
    }
    rendered = json.dumps(result, sort_keys=True)
    if args.json or not failures:
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
