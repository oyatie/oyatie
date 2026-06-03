#!/usr/bin/env python3
"""Validate P0.0 override/kill-switch fixture coverage without live claims.

This checker is local/static fixture evidence only. It proves that the checked-in
override packet schema and baseline GOOD/BAD fixtures exercise the required
TTL, reviewer acknowledgment, audit-chain event, owner, blast-radius,
revert/fix follow-up, affected-context, degraded-gate, and no-new-oya-CLI
surfaces. It does not claim live cloud-ci execution, protected-flow override
authority, P0.0 green, or Phase-0 completion.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

REQUIRED_PACKET_FIELDS = [
    "action",
    "affected_contexts",
    "degraded_gate_ids",
    "ttl_expires_at",
    "reviewer_acknowledgment",
    "audit_chain_event",
    "owner",
    "blast_radius_statement",
    "revert_or_fix_follow_up",
    "no_new_oya_cli_surface",
]

ALLOWED_CONTEXTS = {"cloud-ci-required", "oya-ci-required"}
EXPECTED_ACTION = "temporarily_disable_or_degrade_gate"
TIMESTAMP_RE = re.compile(r"^20[0-9]{2}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$")
OVERRIDE_VIOLATIONS = {
    "override_missing_context_or_gate",
    "override_missing_ttl_reviewer_audit_or_revert",
    "override_new_oya_cli_surface",
}

DEFAULT_SCHEMA = "specs/phase0-override-packet-schema.json"
DEFAULT_GOOD_BASELINE_FIXTURE = "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json"
DEFAULT_BAD_BASELINE_FIXTURE = "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.2-bad-override-without-ttl-audit.json"


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


def non_empty_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def require(condition: bool, failures: list[str], message: str) -> None:
    if not condition:
        failures.append(message)


def validate_schema(schema: dict[str, Any], failures: list[str]) -> dict[str, Any]:
    required = string_list(schema.get("required"))
    for field in REQUIRED_PACKET_FIELDS:
        require(field in required, failures, f"schema.required missing {field}")
    require(schema.get("additionalProperties") is False, failures, "schema.additionalProperties must be false")

    properties = schema.get("properties")
    if not isinstance(properties, dict):
        failures.append("schema.properties must be an object")
        properties = {}

    action = properties.get("action") if isinstance(properties.get("action"), dict) else {}
    require(EXPECTED_ACTION in string_list(action.get("enum")), failures, f"schema.action.enum missing {EXPECTED_ACTION}")

    affected = properties.get("affected_contexts") if isinstance(properties.get("affected_contexts"), dict) else {}
    items = affected.get("items") if isinstance(affected.get("items"), dict) else {}
    enum_values = set(string_list(items.get("enum")))
    require(ALLOWED_CONTEXTS.issubset(enum_values), failures, "schema.affected_contexts must allow cloud-ci-required and oya-ci-required")

    no_new_oya = properties.get("no_new_oya_cli_surface") if isinstance(properties.get("no_new_oya_cli_surface"), dict) else {}
    require(no_new_oya.get("const") is True, failures, "schema.no_new_oya_cli_surface.const must be true")

    return {"required_fields": required, "allowed_contexts": sorted(enum_values)}


def override_packet_violations(packet: dict[str, Any]) -> list[str]:
    violations: list[str] = []
    if packet.get("action") != EXPECTED_ACTION:
        violations.append("override_invalid_action")

    contexts = string_list(packet.get("affected_contexts"))
    gates = string_list(packet.get("degraded_gate_ids"))
    if not contexts or any(context not in ALLOWED_CONTEXTS for context in contexts) or not gates:
        violations.append("override_missing_context_or_gate")

    ttl = packet.get("ttl_expires_at")
    ttl_valid = isinstance(ttl, str) and bool(TIMESTAMP_RE.match(ttl))
    required_text_fields = [
        "reviewer_acknowledgment",
        "audit_chain_event",
        "owner",
        "blast_radius_statement",
        "revert_or_fix_follow_up",
    ]
    if not ttl_valid or any(not non_empty_string(packet.get(field)) for field in required_text_fields):
        violations.append("override_missing_ttl_reviewer_audit_or_revert")

    if packet.get("no_new_oya_cli_surface") is not True:
        violations.append("override_new_oya_cli_surface")

    return violations


def validate_fixture(path: str, expected_verdict: str, failures: list[str]) -> dict[str, Any]:
    fixture = load_json(path)
    fixture_id = fixture.get("fixture_id", path)
    require(fixture.get("expected_verdict") == expected_verdict, failures, f"{fixture_id}: expected_verdict must be {expected_verdict}")
    packet = fixture.get("override_packet")
    if not isinstance(packet, dict):
        failures.append(f"{fixture_id}: override_packet must be an object")
        packet = {}
    observed_violations = override_packet_violations(packet)
    expected_violations = set(string_list(fixture.get("expected_violations")))
    if expected_verdict == "GREEN":
        require(not observed_violations, failures, f"{fixture_id}: GOOD override packet has violations {observed_violations}")
        require(not expected_violations, failures, f"{fixture_id}: GOOD fixture must not list expected violations")
    else:
        require(bool(observed_violations), failures, f"{fixture_id}: RED override packet must produce violations")
        require(OVERRIDE_VIOLATIONS.issubset(expected_violations), failures, f"{fixture_id}: RED fixture expected_violations must include all override violation classes")
        require(set(observed_violations).issubset(expected_violations), failures, f"{fixture_id}: observed override violations not listed in expected_violations")
    return {
        "fixture_id": fixture_id,
        "expected_verdict": expected_verdict,
        "observed_override_violations": observed_violations,
        "override_packet_valid": not observed_violations,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--schema", default=DEFAULT_SCHEMA)
    parser.add_argument("--good-baseline-fixture", default=DEFAULT_GOOD_BASELINE_FIXTURE)
    parser.add_argument("--bad-baseline-fixture", default=DEFAULT_BAD_BASELINE_FIXTURE)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []
    schema_summary = validate_schema(load_json(args.schema), failures)
    baseline_results = [
        validate_fixture(args.good_baseline_fixture, "GREEN", failures),
        validate_fixture(args.bad_baseline_fixture, "RED", failures),
    ]

    result = {
        "authority_boundary": "override/kill-switch fixture evidence only; this checker never claims live protected-flow override authority or status mutation",
        "schema": args.schema,
        "required_packet_fields": REQUIRED_PACKET_FIELDS,
        "required_override_violations": sorted(OVERRIDE_VIOLATIONS),
        "schema_summary": schema_summary,
        "baseline_fixture_results": baseline_results,
        "local_fixture_contract_proven": not failures,
        "live_required_context_execution_proven": False,
        "protected_flow_override_live": False,
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
