#!/usr/bin/env python3
"""Validate AC-0.12 Phase-0 aggregate-exit fixtures locally.

This checker is local/static fixture evidence only. It proves the aggregate exit
shape fails closed when any required Phase-0 subcondition is false, missing, or
unknown. It never posts statuses, mutates branch protection, or claims P0.0
green / Phase-0 completion / production readiness.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

FIXTURE_DIR = Path("specs/fixtures/phase0-exit-gate")
GOOD_FIXTURE = FIXTURE_DIR / "tc-0.12-good-all-subconditions-green.json"
SINGLE_FALSE_FIXTURE = FIXTURE_DIR / "tc-0.12-bad-single-false-subconditions.json"
DEFAULT_FIXTURES = [
    GOOD_FIXTURE,
    SINGLE_FALSE_FIXTURE,
    FIXTURE_DIR / "tc-0.12-bad-missing-required-subcondition.json",
    FIXTURE_DIR / "tc-0.12-current-red-p0-0-live-context-missing.json",
]
LIVE_FALSE_FLAGS = {
    "aggregate_exit_live": False,
    "protected_branch_authority_proven": False,
    "status_mutation_performed": False,
    "live_required_context_execution_proven": False,
    "p0_0_green": False,
    "phase0_complete": False,
    "production_ready": False,
    "hyperscaler_grade": False,
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


def fixture_id(fixture: dict[str, Any], path: str | Path) -> str:
    value = fixture.get("fixture_id")
    return value if isinstance(value, str) and value else Path(path).name


def expected_verdict(fixture: dict[str, Any]) -> str:
    value = fixture.get("expected_verdict")
    return value if value in {"GREEN", "RED"} else "RED"


def required_subconditions() -> list[str]:
    good = load_json(GOOD_FIXTURE)
    single_false = load_json(SINGLE_FALSE_FIXTURE)
    good_names = sorted((good.get("subconditions") or {}).keys()) if isinstance(good.get("subconditions"), dict) else []
    declared_names = sorted(string_list(single_false.get("subcondition_names")))
    if good_names != declared_names:
        raise ValueError(
            "aggregate fixture required subcondition lists diverge between "
            f"{GOOD_FIXTURE} and {SINGLE_FALSE_FIXTURE}: "
            f"good={good_names} declared={declared_names}"
        )
    return declared_names


def evaluate_subconditions(subconditions: Any, required: set[str]) -> tuple[bool, list[str], list[str], list[str]]:
    if not isinstance(subconditions, dict):
        return False, ["missing_or_malformed_subconditions"], sorted(required), []

    present = set(subconditions)
    missing = sorted(required - present)
    unknown = sorted(present - required)
    false_or_invalid = sorted(
        name for name in required & present if subconditions.get(name) is not True
    )

    violations: list[str] = []
    if missing:
        violations.append("missing_required_subcondition")
    if unknown:
        violations.append("unknown_subcondition")
    if false_or_invalid:
        violations.append("false_or_non_true_subcondition")

    return not violations, violations, missing, false_or_invalid


def claim_boundary_violations(fixture: dict[str, Any], observed_green: bool) -> list[str]:
    boundary = fixture.get("claim_boundary")
    violations: list[str] = []
    if isinstance(boundary, dict):
        if boundary.get("p0_0_green") is True or boundary.get("phase0_complete") is True:
            violations.append("fixture_claims_current_phase0_green")
        if not observed_green:
            if boundary.get("p0_0_green") is not False or boundary.get("phase0_complete") is not False:
                violations.append("red_fixture_missing_false_claim_boundary")
    return violations


def validate_case(case: dict[str, Any], required: set[str]) -> dict[str, Any]:
    observed_green, violations, missing, false_or_invalid = evaluate_subconditions(case.get("subconditions"), required)
    case_id = case.get("case_id") if isinstance(case.get("case_id"), str) else "unknown-case"
    forced_false = case.get("forced_false") if isinstance(case.get("forced_false"), str) else None
    failures: list[str] = []
    if case.get("expected_verdict") == "RED" and observed_green:
        failures.append(f"{case_id}: RED aggregate subcondition case passed")
    if forced_false:
        if forced_false not in required:
            failures.append(f"{case_id}: forced_false is not a required subcondition")
        if forced_false not in false_or_invalid:
            failures.append(f"{case_id}: forced_false was not observed false_or_non_true")
        if violations != ["false_or_non_true_subcondition"] or false_or_invalid != [forced_false]:
            failures.append(
                f"{case_id}: single_false_case_not_exactly_one_false_subcondition "
                f"forced_false={forced_false} observed_false_or_non_true={false_or_invalid} "
                f"violations={violations}"
            )
    return {
        "case_id": case_id,
        "forced_false": forced_false,
        "observed_verdict": "GREEN" if observed_green else "RED",
        "violations": violations,
        "missing_subconditions": missing,
        "false_or_non_true_subconditions": false_or_invalid,
        "case_passed": not failures,
        "failures": failures,
    }


def validate_fixture(path: str | Path, required_names: list[str]) -> dict[str, Any]:
    required = set(required_names)
    fixture = load_json(path)
    fid = fixture_id(fixture, path)
    expected = expected_verdict(fixture)
    observed_green, violations, missing, false_or_invalid = evaluate_subconditions(fixture.get("subconditions"), required)
    failures: list[str] = []

    if expected == "GREEN" and not observed_green:
        failures.append(f"{fid}: GREEN aggregate fixture produced violations {violations}")
    if expected == "RED" and observed_green:
        failures.append(f"{fid}: RED aggregate fixture passed")

    expected_false = set(string_list(fixture.get("expected_false_or_missing_subconditions")))
    if expected_false:
        observed_false = set(missing) | set(false_or_invalid)
        if not expected_false <= observed_false:
            failures.append(f"{fid}: expected false/missing subconditions not observed {sorted(expected_false - observed_false)}")

    failures.extend(f"{fid}: {violation}" for violation in claim_boundary_violations(fixture, observed_green))

    case_results: list[dict[str, Any]] = []
    if path == SINGLE_FALSE_FIXTURE or "subcondition_names" in fixture or "cases" in fixture:
        declared = sorted(string_list(fixture.get("subcondition_names")))
        if declared != required_names:
            failures.append(f"{fid}: subcondition_names do not mirror required set")
        cases = object_list(fixture.get("cases"))
        forced = [case.get("forced_false") for case in cases if isinstance(case.get("forced_false"), str)]
        missing_cases = sorted(required - set(forced))
        duplicate_cases = sorted({name for name in forced if forced.count(name) > 1})
        if missing_cases:
            failures.append(f"{fid}: missing_case_for_required_subcondition {missing_cases}")
        if duplicate_cases:
            failures.append(f"{fid}: duplicate_case_for_required_subcondition {duplicate_cases}")
        for case in cases:
            result = validate_case(case, required)
            case_results.append(result)
            failures.extend(result["failures"])

    return {
        "path": str(path),
        "fixture_id": fid,
        "expected_verdict": expected,
        "observed_verdict": "GREEN" if observed_green else "RED",
        "violations": violations,
        "missing_subconditions": missing,
        "false_or_non_true_subconditions": false_or_invalid,
        "case_results": case_results,
        "fixture_passed": not failures,
        "failures": failures,
    }


def fixture_paths(explicit: list[str] | None) -> list[str]:
    return explicit if explicit else [str(path) for path in DEFAULT_FIXTURES]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", action="append", default=None, help="Fixture path to evaluate; repeat to override defaults")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    required_names = required_subconditions()
    results = [validate_fixture(path, required_names) for path in fixture_paths(args.fixture)]
    failures = [failure for result in results for failure in result["failures"]]
    single_false_results = next((r for r in results if Path(r["path"]).name == SINGLE_FALSE_FIXTURE.name), None)
    case_count = len(single_false_results["case_results"]) if single_false_results else 0

    output = {
        "authority_boundary": "AC-0.12 aggregate-exit local/static fixture evidence only; this checker never posts statuses, mutates branch protection, or claims live Phase-0 completion",
        "required_subcondition_count": len(required_names),
        "required_subconditions": required_names,
        "single_false_case_count": case_count,
        "fixture_results": results,
        "local_fixture_contract_proven": not failures,
        "aggregate_exit_local_static_proven": not failures,
        **LIVE_FALSE_FLAGS,
        "verdict": "PASS" if not failures else "FAIL",
        "failures": failures,
    }
    rendered = json.dumps(output, sort_keys=True)
    if args.json or not failures:
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
