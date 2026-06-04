#!/usr/bin/env python3
"""Validate P0.0 tenant-pipeline isolation fixture coverage without live claims.

This checker is local/static fixture evidence only. It proves that the checked-in
contract and baseline GOOD/BAD fixtures exercise the required tenant-pipeline
separation surfaces; it does not claim live cloud-ci execution, tenant-facing
readiness, security readiness, or Phase-0 completion.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REQUIRED_SURFACES = [
    "identity",
    "secrets",
    "runners",
    "workspaces",
    "caches",
    "artifacts",
    "logs_evidence",
    "release_ledgers",
    "deploy_targets",
    "status_callbacks",
    "audit_events",
]

ALIASES = {
    "identity": {"identity"},
    "secrets": {"secrets", "secret_scope", "secret_lease"},
    "runners": {"runners", "runner_pool"},
    "workspaces": {"workspaces", "workspace_volume"},
    "caches": {"caches", "cache_namespace"},
    "artifacts": {"artifacts", "artifact_namespace"},
    "logs_evidence": {"logs_evidence", "log_evidence_namespace"},
    "release_ledgers": {"release_ledgers", "release_ledger"},
    "deploy_targets": {"deploy_targets", "deploy_target"},
    "status_callbacks": {"status_callbacks", "status_callback_identity"},
    "audit_events": {"audit_events", "audit_event_stream"},
}

DEFAULT_CONTRACT = "specs/toolchain-tenant-isolation-fixtures.json"
DEFAULT_GOOD_BASELINE_FIXTURE = "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json"
DEFAULT_BAD_BASELINE_FIXTURE = "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.3-bad-cross-tenant-shared-cache.json"


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


def surface_is_present(required: str, observed: set[str]) -> bool:
    return bool(ALIASES[required] & observed)


def missing_required_surfaces(separated_surfaces: list[str]) -> list[str]:
    observed = set(separated_surfaces)
    return [surface for surface in REQUIRED_SURFACES if not surface_is_present(surface, observed)]


def internal_bypass_without_breakglass(model: dict[str, Any]) -> bool:
    bypass = model.get("internal_bypass")
    if isinstance(bypass, dict):
        return bypass.get("allowed_without_ttl_breakglass") is True
    if isinstance(bypass, str):
        lowered = bypass.lower()
        return "ttl" not in lowered or "breakglass" not in lowered or "audit" not in lowered
    return False


def evaluate_tenant_model(model: dict[str, Any]) -> list[str]:
    violations: list[str] = []
    separated = string_list(model.get("separate_surfaces")) + string_list(model.get("partitioned_surfaces"))
    if missing_required_surfaces(separated):
        violations.append("tenant_surface_separation_incomplete")
    if string_list(model.get("shared_surfaces")):
        violations.append("tenant_surfaces_shared")
    if internal_bypass_without_breakglass(model):
        violations.append("internal_bypass_without_breakglass")
    return violations


def require(condition: bool, failures: list[str], message: str) -> None:
    if not condition:
        failures.append(message)


def validate_contract(contract: dict[str, Any], failures: list[str]) -> dict[str, Any]:
    required = string_list(contract.get("required_separation_surfaces"))
    for surface in REQUIRED_SURFACES:
        require(surface in required, failures, f"contract.required_separation_surfaces missing {surface}")

    fixtures = contract.get("fixtures")
    if not isinstance(fixtures, list):
        failures.append("contract.fixtures must be a list")
        fixtures = []

    seen_green = False
    seen_red = False
    contract_results: list[dict[str, Any]] = []
    for index, fixture in enumerate(fixtures):
        if not isinstance(fixture, dict):
            failures.append(f"contract.fixtures[{index}] must be an object")
            continue
        fixture_id = fixture.get("fixture_id", f"index-{index}")
        verdict = fixture.get("expected_verdict")
        if verdict == "GREEN":
            seen_green = True
            expected_violations = string_list(fixture.get("expected_violations"))
            separated = string_list(fixture.get("separate_surfaces"))
            missing = missing_required_surfaces(separated)
            require(not expected_violations, failures, f"{fixture_id}: GREEN fixture must not list expected violations")
            require(not missing, failures, f"{fixture_id}: GREEN fixture missing separated surfaces {missing}")
            require(isinstance(fixture.get("breakglass"), str) and fixture["breakglass"], failures, f"{fixture_id}: GREEN fixture missing breakglass contract")
            require(isinstance(fixture.get("separation_model"), str) and fixture["separation_model"], failures, f"{fixture_id}: GREEN fixture missing separation_model")
            contract_results.append({"fixture_id": fixture_id, "expected_verdict": verdict, "missing_surfaces": missing})
        elif verdict == "RED":
            seen_red = True
            expected_violations = string_list(fixture.get("expected_violations"))
            shared = string_list(fixture.get("shared_surfaces"))
            require(bool(expected_violations), failures, f"{fixture_id}: RED fixture must list expected violations")
            require(bool(shared), failures, f"{fixture_id}: RED fixture must expose shared surfaces")
            require(fixture.get("internal_bypass_without_breakglass") is True, failures, f"{fixture_id}: RED fixture must cover internal bypass without breakglass")
            contract_results.append({"fixture_id": fixture_id, "expected_verdict": verdict, "shared_surfaces": shared})
        else:
            failures.append(f"{fixture_id}: unsupported expected_verdict {verdict!r}")

    require(seen_green, failures, "contract must include a GREEN target fixture")
    require(seen_red, failures, "contract must include a RED negative fixture")
    return {"required_surfaces": required, "contract_fixture_results": contract_results}


def validate_baseline_fixture(path: str, expected_verdict: str, failures: list[str]) -> dict[str, Any]:
    fixture = load_json(path)
    fixture_id = fixture.get("fixture_id", path)
    require(fixture.get("expected_verdict") == expected_verdict, failures, f"{fixture_id}: expected_verdict must be {expected_verdict}")
    model = fixture.get("tenant_pipeline_model")
    if not isinstance(model, dict):
        failures.append(f"{fixture_id}: tenant_pipeline_model must be an object")
        model = {}
    observed_violations = evaluate_tenant_model(model)
    expected_violations = string_list(fixture.get("expected_violations"))
    if expected_verdict == "GREEN":
        require(not observed_violations, failures, f"{fixture_id}: GREEN tenant model has violations {observed_violations}")
        require(not expected_violations, failures, f"{fixture_id}: GREEN fixture must not list expected violations")
    else:
        tenant_violation_set = {
            "tenant_surface_separation_incomplete",
            "tenant_surfaces_shared",
            "internal_bypass_without_breakglass",
        }
        require(bool(observed_violations), failures, f"{fixture_id}: RED tenant model must produce tenant violations")
        require(tenant_violation_set.issubset(set(expected_violations)), failures, f"{fixture_id}: RED fixture expected_violations must include all tenant isolation violation classes")
        require(set(observed_violations).issubset(set(expected_violations)), failures, f"{fixture_id}: observed tenant violations not listed in expected_violations")
    return {
        "fixture_id": fixture_id,
        "expected_verdict": expected_verdict,
        "observed_tenant_violations": observed_violations,
        "tenant_surfaces_complete": not missing_required_surfaces(string_list(model.get("separate_surfaces")) + string_list(model.get("partitioned_surfaces"))),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", default=DEFAULT_CONTRACT)
    parser.add_argument("--good-baseline-fixture", default=DEFAULT_GOOD_BASELINE_FIXTURE)
    parser.add_argument("--bad-baseline-fixture", default=DEFAULT_BAD_BASELINE_FIXTURE)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []
    contract_summary = validate_contract(load_json(args.contract), failures)
    baseline_results = [
        validate_baseline_fixture(args.good_baseline_fixture, "GREEN", failures),
        validate_baseline_fixture(args.bad_baseline_fixture, "RED", failures),
    ]

    result = {
        "authority_boundary": "tenant-isolation fixture evidence only; this checker never claims live cloud-ci execution or tenant-facing readiness",
        "contract": args.contract,
        "required_surfaces": REQUIRED_SURFACES,
        "contract_fixture_results": contract_summary["contract_fixture_results"],
        "baseline_fixture_results": baseline_results,
        "local_fixture_contract_proven": not failures,
        "live_required_context_execution_proven": False,
        "tenant_facing_ready": False,
        "security_ready": False,
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
