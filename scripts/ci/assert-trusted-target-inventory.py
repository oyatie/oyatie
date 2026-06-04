#!/usr/bin/env python3
"""Validate P0.0 trusted target-inventory fixture coverage without live claims.

This checker is local/static fixture evidence only. It proves that the checked-in
trusted target-inventory schema and baseline GOOD/BAD fixtures exercise the
requirement that Buck2 build/test targets come from trusted dev/controller state
before candidate checkout. It does not claim live cloud-ci execution, protected
branch authority, P0.0 green, or Phase-0 completion.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

REQUIRED_INVENTORY_FIELDS = [
    "candidate_sha",
    "claim_boundary",
    "inventory_source",
    "captured_before_candidate_checkout",
    "candidate_checkout_after_inventory",
    "no_candidate_authored_discovery",
    "build_targets",
    "test_targets",
    "expected_verdict",
    "expected_violations",
    "fixture_id",
    "source_test",
]

TRUSTED_SOURCE = "trusted_dev_or_controller_state"
CANDIDATE_SOURCE = "candidate_pr_bytes"
TRUSTED_TARGET_VIOLATIONS = {
    "target_inventory_not_trusted",
    "inventory_not_captured_before_candidate_checkout",
    "candidate_can_author_target_inventory",
    "empty_required_targets",
    "malformed_buck2_target",
    "green_claim_boundary_without_live_authority",
}
SHA_RE = re.compile(r"^[0-9a-fA-F]{40}$")
TARGET_RE = re.compile(r"^(?:[A-Za-z0-9_.-]+)?//[A-Za-z0-9_./-]*:[A-Za-z0-9_+=.,@~-]+$")

DEFAULT_SCHEMA = "specs/phase0-trusted-target-inventory-schema.json"
DEFAULT_GOOD_FIXTURE = "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-good-trusted-target-inventory.json"
DEFAULT_BAD_FIXTURE = "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-bad-candidate-sourced-target-inventory.json"


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


def require(condition: bool, failures: list[str], message: str) -> None:
    if not condition:
        failures.append(message)


def validate_schema(schema: dict[str, Any], failures: list[str]) -> dict[str, Any]:
    required = string_list(schema.get("required"))
    for field in REQUIRED_INVENTORY_FIELDS:
        require(field in required, failures, f"schema.required missing {field}")
    require(schema.get("additionalProperties") is False, failures, "schema.additionalProperties must be false")

    properties = schema.get("properties")
    if not isinstance(properties, dict):
        failures.append("schema.properties must be an object")
        properties = {}

    inventory_source = properties.get("inventory_source") if isinstance(properties.get("inventory_source"), dict) else {}
    for field in REQUIRED_INVENTORY_FIELDS:
        require(field in properties, failures, f"schema.properties missing {field}")

    source_enum = set(string_list(inventory_source.get("enum")))
    require(source_enum == {TRUSTED_SOURCE, CANDIDATE_SOURCE}, failures, "schema.inventory_source enum must be exactly trusted_dev_or_controller_state and candidate_pr_bytes")

    verdict = properties.get("expected_verdict") if isinstance(properties.get("expected_verdict"), dict) else {}
    verdict_enum = set(string_list(verdict.get("enum")))
    require(verdict_enum == {"GREEN", "RED"}, failures, "schema.expected_verdict enum must be exactly GREEN and RED")

    claim_boundary = properties.get("claim_boundary") if isinstance(properties.get("claim_boundary"), dict) else {}
    claim_required = set(string_list(claim_boundary.get("required")))
    require({"p0_0_green", "phase0_complete"}.issubset(claim_required), failures, "schema.claim_boundary must require p0_0_green and phase0_complete")
    require(claim_boundary.get("additionalProperties") is False, failures, "schema.claim_boundary.additionalProperties must be false")

    for field in ["build_targets", "test_targets"]:
        target_field = properties.get(field) if isinstance(properties.get(field), dict) else {}
        require(target_field.get("minItems") == 1, failures, f"schema.{field}.minItems must be 1")

    return {
        "required_fields": required,
        "inventory_source_values": sorted(source_enum),
        "expected_verdict_values": sorted(verdict_enum),
    }


def target_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, str)]


def malformed_targets(targets: list[str]) -> list[str]:
    return [target for target in targets if not TARGET_RE.match(target)]


def inventory_fixture_shape_failures(fixture: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    unknown_fields = sorted(set(fixture) - set(REQUIRED_INVENTORY_FIELDS))
    if unknown_fields:
        failures.append(f"unexpected top-level fields: {unknown_fields}")
    candidate_sha = fixture.get("candidate_sha")
    if not isinstance(candidate_sha, str) or not SHA_RE.match(candidate_sha):
        failures.append("candidate_sha must be a 40-character hexadecimal SHA")
    return failures


def inventory_fixture_violations(fixture: dict[str, Any]) -> list[str]:
    violations: list[str] = []
    if fixture.get("inventory_source") != TRUSTED_SOURCE:
        violations.append("target_inventory_not_trusted")
    if fixture.get("captured_before_candidate_checkout") is not True or fixture.get("candidate_checkout_after_inventory") is not True:
        violations.append("inventory_not_captured_before_candidate_checkout")
    if fixture.get("no_candidate_authored_discovery") is not True:
        violations.append("candidate_can_author_target_inventory")

    build_targets = target_list(fixture.get("build_targets"))
    test_targets = target_list(fixture.get("test_targets"))
    if not build_targets or not test_targets:
        violations.append("empty_required_targets")
    if malformed_targets(build_targets + test_targets):
        violations.append("malformed_buck2_target")

    claim_boundary = fixture.get("claim_boundary")
    if not isinstance(claim_boundary, dict) or claim_boundary.get("p0_0_green") is not False or claim_boundary.get("phase0_complete") is not False:
        violations.append("green_claim_boundary_without_live_authority")
    return violations


def validate_fixture(path: str, expected_verdict: str, failures: list[str]) -> dict[str, Any]:
    fixture = load_json(path)
    fixture_id = fixture.get("fixture_id", path)
    require(fixture.get("expected_verdict") == expected_verdict, failures, f"{fixture_id}: expected_verdict must be {expected_verdict}")
    shape_failures = inventory_fixture_shape_failures(fixture)
    for shape_failure in shape_failures:
        failures.append(f"{fixture_id}: {shape_failure}")
    observed_violations = inventory_fixture_violations(fixture)
    expected_violations = set(string_list(fixture.get("expected_violations")))
    if expected_verdict == "GREEN":
        require(not observed_violations, failures, f"{fixture_id}: GOOD trusted-target inventory has violations {observed_violations}")
        require(not expected_violations, failures, f"{fixture_id}: GOOD fixture must not list expected violations")
    else:
        require(bool(observed_violations), failures, f"{fixture_id}: RED trusted-target inventory must produce violations")
        require(TRUSTED_TARGET_VIOLATIONS.issubset(expected_violations), failures, f"{fixture_id}: RED fixture expected_violations must include all trusted-target violation classes")
        require(set(observed_violations).issubset(expected_violations), failures, f"{fixture_id}: observed trusted-target violations not listed in expected_violations")
    return {
        "fixture_id": fixture_id,
        "expected_verdict": expected_verdict,
        "shape_failures": shape_failures,
        "observed_trusted_target_violations": observed_violations,
        "malformed_targets": malformed_targets(target_list(fixture.get("build_targets")) + target_list(fixture.get("test_targets"))),
        "trusted_inventory": not shape_failures and not observed_violations,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--schema", default=DEFAULT_SCHEMA)
    parser.add_argument("--good-fixture", default=DEFAULT_GOOD_FIXTURE)
    parser.add_argument("--bad-fixture", default=DEFAULT_BAD_FIXTURE)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []
    schema_summary = validate_schema(load_json(args.schema), failures)
    fixture_results = [
        validate_fixture(args.good_fixture, "GREEN", failures),
        validate_fixture(args.bad_fixture, "RED", failures),
    ]

    result = {
        "authority_boundary": "trusted target-inventory fixture evidence only; this checker never claims live cloud-ci execution or protected-branch authority",
        "schema": args.schema,
        "required_inventory_fields": REQUIRED_INVENTORY_FIELDS,
        "required_trusted_target_violations": sorted(TRUSTED_TARGET_VIOLATIONS),
        "schema_summary": schema_summary,
        "fixture_results": fixture_results,
        "local_fixture_contract_proven": not failures,
        "candidate_pr_bytes_are_data_only_locally_proven": not failures,
        "trusted_target_inventory_live_authority_proven": False,
        "trusted_controller_inventory_live": False,
        "live_required_context_execution_proven": False,
        "p0_0_green": False,
        "phase0_complete": False,
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
