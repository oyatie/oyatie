#!/usr/bin/env python3
"""Validate P0.0 structured result-bundle fixture coverage without live claims.

This checker is local/static fixture evidence only. It proves that the checked-in
structured result schema and RED/false-green fixtures exercise the required
result-bundle authority boundaries. It does not post statuses and does not claim
live cloud-ci execution, protected branch authority, P0.0 green, or Phase-0
completion.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

REQUIRED_BUNDLE_FIELDS = [
    "candidate_sha",
    "required_context",
    "producer",
    "fixture_results",
    "observed_verdict",
    "provenance",
    "claim_boundary",
]
REQUIRED_PRODUCER_FIELDS = ["context", "kind", "trusted_control_state"]
REQUIRED_FIXTURE_RESULT_FIELDS = ["fixture_id", "expected_verdict", "observed_verdict", "violations"]
REQUIRED_CONTEXTS = {"cloud-ci-required", "oya-ci-required"}
ALLOWED_CONTEXT_VALUES = REQUIRED_CONTEXTS | {"missing"}
TRUSTED_PRODUCER_KINDS = {"minimal_rust_bridge_adapter", "oya-ci-controller"}
EXPECTED_FALSE_GREEN_VIOLATIONS = {
    "missing_cloud_ci_required_context",
    "untrusted_or_legacy_status_producer",
    "candidate_bytes_can_weaken_result",
    "candidate_sourced_gate_definition",
    "fixture_result_mismatch",
    "red_expected_fixture_missing_violations",
    "green_bundle_without_green_fixture_results",
    "green_claim_boundary_without_live_authority",
}
CURRENT_RED_REQUIRED_VIOLATIONS = {
    "missing_cloud_ci_required_context",
    "untrusted_or_legacy_status_producer",
    "candidate_bytes_can_weaken_result",
    "candidate_sourced_gate_definition",
}
SHA_RE = re.compile(r"^[0-9a-fA-F]{40}$")
TIMESTAMP_RE = re.compile(r"^20[0-9]{2}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$")

DEFAULT_SCHEMA = "specs/phase0-ci-enforcement-result-schema.json"
DEFAULT_CURRENT_RED_FIXTURE = "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-current-red-gap-result.json"
DEFAULT_FALSE_GREEN_FIXTURE = "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.4-bad-result-bundle-false-green.json"


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


def enum_values(properties: dict[str, Any], field: str) -> set[str]:
    value = properties.get(field)
    if not isinstance(value, dict):
        return set()
    return set(string_list(value.get("enum")))


def validate_schema(schema: dict[str, Any], failures: list[str]) -> dict[str, Any]:
    required = string_list(schema.get("required"))
    for field in REQUIRED_BUNDLE_FIELDS:
        require(field in required, failures, f"schema.required missing {field}")
    require(schema.get("additionalProperties") is False, failures, "schema.additionalProperties must be false")

    properties = schema.get("properties")
    if not isinstance(properties, dict):
        failures.append("schema.properties must be an object")
        properties = {}
    for field in REQUIRED_BUNDLE_FIELDS:
        require(field in properties, failures, f"schema.properties missing {field}")

    candidate_sha = properties.get("candidate_sha") if isinstance(properties.get("candidate_sha"), dict) else {}
    require(candidate_sha.get("minLength") == 40 and candidate_sha.get("maxLength") == 40, failures, "schema.candidate_sha must require exactly 40 characters")
    require(candidate_sha.get("pattern") == "^[0-9a-fA-F]{40}$", failures, "schema.candidate_sha must require 40 hexadecimal characters")

    require(enum_values(properties, "required_context") == ALLOWED_CONTEXT_VALUES, failures, "schema.required_context enum must be exactly cloud-ci-required, oya-ci-required, and missing")
    require(enum_values(properties, "observed_verdict") == {"GREEN", "RED"}, failures, "schema.observed_verdict enum must be exactly GREEN and RED")

    producer = properties.get("producer") if isinstance(properties.get("producer"), dict) else {}
    producer_required = set(string_list(producer.get("required")))
    require(set(REQUIRED_PRODUCER_FIELDS).issubset(producer_required), failures, "schema.producer must require context, kind, and trusted_control_state")
    require(producer.get("additionalProperties") is False, failures, "schema.producer.additionalProperties must be false")
    producer_properties = producer.get("properties") if isinstance(producer.get("properties"), dict) else {}
    require(enum_values(producer_properties, "context") == ALLOWED_CONTEXT_VALUES, failures, "schema.producer.context enum must be exactly cloud-ci-required, oya-ci-required, and missing")

    claim_boundary = properties.get("claim_boundary") if isinstance(properties.get("claim_boundary"), dict) else {}
    claim_required = set(string_list(claim_boundary.get("required")))
    require({"p0_0_green", "phase0_complete"}.issubset(claim_required), failures, "schema.claim_boundary must require p0_0_green and phase0_complete")
    require(claim_boundary.get("additionalProperties") is False, failures, "schema.claim_boundary.additionalProperties must be false")

    fixture_results = properties.get("fixture_results") if isinstance(properties.get("fixture_results"), dict) else {}
    require(fixture_results.get("minItems") == 1, failures, "schema.fixture_results.minItems must be 1")
    items = fixture_results.get("items") if isinstance(fixture_results.get("items"), dict) else {}
    item_properties = items.get("properties") if isinstance(items.get("properties"), dict) else {}
    item_required = set(string_list(items.get("required")))
    require(set(REQUIRED_FIXTURE_RESULT_FIELDS).issubset(item_required), failures, "schema.fixture_results.items must require fixture_id, expected_verdict, observed_verdict, and violations")
    require(items.get("additionalProperties") is False, failures, "schema.fixture_results.items.additionalProperties must be false")
    require(enum_values(item_properties, "expected_verdict") == {"GREEN", "RED"}, failures, "schema.fixture_results.items.expected_verdict enum must be exactly GREEN and RED")
    require(enum_values(item_properties, "observed_verdict") == {"GREEN", "RED"}, failures, "schema.fixture_results.items.observed_verdict enum must be exactly GREEN and RED")

    provenance = properties.get("provenance") if isinstance(properties.get("provenance"), dict) else {}
    provenance_required = set(string_list(provenance.get("required")))
    require({"recorded_at", "sources"}.issubset(provenance_required), failures, "schema.provenance must require recorded_at and sources")
    require(provenance.get("additionalProperties") is False, failures, "schema.provenance.additionalProperties must be false")

    return {
        "required_fields": required,
        "required_context_values": sorted(enum_values(properties, "required_context")),
        "observed_verdict_values": sorted(enum_values(properties, "observed_verdict")),
    }


def bundle_shape_failures(bundle: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    unknown = sorted(set(bundle) - set(REQUIRED_BUNDLE_FIELDS))
    if unknown:
        failures.append(f"unexpected top-level fields: {unknown}")
    candidate_sha = bundle.get("candidate_sha")
    if not isinstance(candidate_sha, str) or not SHA_RE.match(candidate_sha):
        failures.append("candidate_sha must be a 40-character hexadecimal SHA")
    provenance = bundle.get("provenance")
    if not isinstance(provenance, dict):
        failures.append("provenance must be an object")
    else:
        recorded_at = provenance.get("recorded_at")
        if not isinstance(recorded_at, str) or not TIMESTAMP_RE.match(recorded_at):
            failures.append("provenance.recorded_at must be an ISO-8601 UTC second timestamp")
        if not string_list(provenance.get("sources")):
            failures.append("provenance.sources must be a non-empty string array")
    return failures


def evaluate_result_bundle(bundle: dict[str, Any]) -> list[str]:
    violations: list[str] = []
    if not isinstance(bundle, dict):
        return ["missing_or_malformed_result_bundle"]

    if not isinstance(bundle.get("candidate_sha"), str) or not SHA_RE.match(bundle.get("candidate_sha", "")):
        violations.append("invalid_candidate_sha")

    required_context = bundle.get("required_context")
    if required_context not in REQUIRED_CONTEXTS:
        violations.append("missing_cloud_ci_required_context")

    producer = bundle.get("producer")
    if not isinstance(producer, dict):
        producer = {}
    if (
        producer.get("trusted_control_state") is not True
        or producer.get("kind") not in TRUSTED_PRODUCER_KINDS
        or producer.get("context") != required_context
        or producer.get("context") not in REQUIRED_CONTEXTS
    ):
        violations.append("untrusted_or_legacy_status_producer")
    if producer.get("candidate_bytes_policy") != "untrusted_input_only":
        violations.append("candidate_bytes_can_weaken_result")
    if producer.get("gate_definition_source") != "trusted_dev_or_controller_state":
        violations.append("candidate_sourced_gate_definition")

    fixture_results = bundle.get("fixture_results")
    if not isinstance(fixture_results, list) or not fixture_results:
        violations.append("missing_or_malformed_result_bundle")
        return violations

    all_fixture_results_green = True
    for fixture in fixture_results:
        if not isinstance(fixture, dict):
            violations.append("missing_or_malformed_result_bundle")
            all_fixture_results_green = False
            continue
        expected = fixture.get("expected_verdict")
        observed = fixture.get("observed_verdict")
        if expected not in {"RED", "GREEN"} or observed not in {"RED", "GREEN"} or expected != observed:
            violations.append("fixture_result_mismatch")
        if expected == "RED" and not string_list(fixture.get("violations")):
            violations.append("red_expected_fixture_missing_violations")
        if expected != "GREEN" or observed != "GREEN":
            all_fixture_results_green = False

    observed_verdict = bundle.get("observed_verdict")
    claim_boundary = bundle.get("claim_boundary") if isinstance(bundle.get("claim_boundary"), dict) else {}
    claims_p0_green = claim_boundary.get("p0_0_green") is True
    claims_phase0_complete = claim_boundary.get("phase0_complete") is True

    if observed_verdict == "GREEN" and not all_fixture_results_green:
        violations.append("green_bundle_without_green_fixture_results")
    if observed_verdict == "GREEN" and violations and claims_p0_green:
        violations.append("green_claim_boundary_without_live_authority")
    if observed_verdict == "RED" and (claims_p0_green or claims_phase0_complete):
        violations.append("red_bundle_claims_green_boundary")

    return sorted(set(violations))


def validate_current_red(path: str, failures: list[str]) -> dict[str, Any]:
    bundle = load_json(path)
    shape_failures = bundle_shape_failures(bundle)
    for failure in shape_failures:
        failures.append(f"current RED result bundle: {failure}")
    require(bundle.get("observed_verdict") == "RED", failures, "current RED result bundle must keep observed_verdict=RED")
    boundary = bundle.get("claim_boundary") if isinstance(bundle.get("claim_boundary"), dict) else {}
    require(boundary.get("p0_0_green") is False and boundary.get("phase0_complete") is False, failures, "current RED result bundle must keep p0_0_green=false and phase0_complete=false")
    violations = set(evaluate_result_bundle(bundle))
    require("missing_or_malformed_result_bundle" not in violations, failures, "current RED result bundle must remain schema-shaped and non-empty")
    require(CURRENT_RED_REQUIRED_VIOLATIONS.issubset(violations), failures, "current RED result bundle must expose missing-context, untrusted-producer, candidate-bytes, and candidate-sourced violations")
    return {"path": path, "observed_verdict": bundle.get("observed_verdict"), "shape_failures": shape_failures, "observed_result_bundle_violations": sorted(violations)}


def validate_false_green(path: str, failures: list[str]) -> dict[str, Any]:
    bundle = load_json(path)
    shape_failures = bundle_shape_failures(bundle)
    for failure in shape_failures:
        failures.append(f"false-green result bundle: {failure}")
    require(bundle.get("observed_verdict") == "GREEN", failures, "false-green result bundle fixture must exercise observed_verdict=GREEN")
    boundary = bundle.get("claim_boundary") if isinstance(bundle.get("claim_boundary"), dict) else {}
    require(boundary.get("p0_0_green") is True and boundary.get("phase0_complete") is True, failures, "false-green result bundle must exercise p0_0_green=true and phase0_complete=true")
    violations = set(evaluate_result_bundle(bundle))
    require("missing_or_malformed_result_bundle" not in violations, failures, "false-green result bundle must remain schema-shaped and non-empty")
    require(EXPECTED_FALSE_GREEN_VIOLATIONS.issubset(violations), failures, "false-green result bundle must expose all required false-green violation classes")
    return {"path": path, "observed_verdict": bundle.get("observed_verdict"), "shape_failures": shape_failures, "observed_result_bundle_violations": sorted(violations)}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--schema", default=DEFAULT_SCHEMA)
    parser.add_argument("--current-red-fixture", default=DEFAULT_CURRENT_RED_FIXTURE)
    parser.add_argument("--false-green-fixture", default=DEFAULT_FALSE_GREEN_FIXTURE)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []
    schema_summary = validate_schema(load_json(args.schema), failures)
    current_red_result = validate_current_red(args.current_red_fixture, failures)
    false_green_result = validate_false_green(args.false_green_fixture, failures)

    result = {
        "authority_boundary": "structured result-bundle fixture evidence only; this checker never posts statuses or claims live required-context authority",
        "schema": args.schema,
        "required_bundle_fields": REQUIRED_BUNDLE_FIELDS,
        "required_false_green_violations": sorted(EXPECTED_FALSE_GREEN_VIOLATIONS),
        "current_red_required_violations": sorted(CURRENT_RED_REQUIRED_VIOLATIONS),
        "schema_summary": schema_summary,
        "current_red_result": current_red_result,
        "false_green_result": false_green_result,
        "local_fixture_contract_proven": not failures,
        "structured_result_bundle_live": False,
        "trusted_status_producer_live": False,
        "protected_branch_authority_proven": False,
        "status_mutation_performed": False,
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
