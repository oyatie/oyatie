#!/usr/bin/env python3
"""Fail closed when cloud compute/observability target specs grow fake service dirs."""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]

RESOURCE_CATALOG_PATH = REPO_ROOT / "specs" / "cloud-resource-catalog-target.json"
OBSERVABILITY_TARGET_PATH = REPO_ROOT / "specs" / "cloud-observability-slo-target.json"
TAXONOMY_PATH = REPO_ROOT / "specs" / "cloud-hyperscaler-parity-taxonomy.json"
DOC_COVERAGE_PATH = REPO_ROOT / "docs" / "DOC-COVERAGE.md"

GATE_ID = "cloud-compute-observability-absence-nonclaim"
ABSENT_PATHS = ("cloud/cloud-compute", "cloud/cloud-observability")
RESOURCE_BLOCKED_CLAIMS = {
    "no_tenant_workload_claim",
    "no_public_sla_slo_claim",
    "no_runtime_observability_engine",
    "no_production_readiness_claim",
}
OBSERVABILITY_BLOCKED_CLAIMS = {
    "no_runtime_observability_engine",
    "no_measured_slo_claim",
    "no_public_sla_slo_claim",
    "no_production_readiness_claim",
}
TAXONOMY_BLOCKED_CLAIMS = {
    "tenant_workload_ready",
    "public_sla_or_slo",
    "production_ready",
    "hyperscaler_mature",
    "live_provider_provisioning",
}


def fail(message: str) -> NoReturn:
    print(f"cloud absence/non-claim check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path.relative_to(REPO_ROOT)}: {exc}")


def validate_gate(spec: dict, path: Path, required_blocked_claims: set[str]) -> None:
    gate = spec.get("implementation_directory_absence_gate")
    require(isinstance(gate, dict), f"{path.relative_to(REPO_ROOT)} missing implementation_directory_absence_gate")
    require(gate.get("gate_id") == GATE_ID, f"{path.relative_to(REPO_ROOT)} has wrong gate_id")
    require(
        tuple(gate.get("paths_that_must_remain_absent_until_runtime_evidence", ())) == ABSENT_PATHS,
        f"{path.relative_to(REPO_ROOT)} must list exactly {ABSENT_PATHS}",
    )
    blocked = set(gate.get("blocked_claim_families", ()))
    require(
        required_blocked_claims <= blocked,
        f"{path.relative_to(REPO_ROOT)} missing blocked claims {sorted(required_blocked_claims - blocked)}",
    )
    require(
        gate.get("checker") == "scripts/tests/check_cloud_absence_nonclaim.py",
        f"{path.relative_to(REPO_ROOT)} must point back to this checker",
    )


def validate_absent_directories() -> None:
    for relative in ABSENT_PATHS:
        path = REPO_ROOT / relative
        require(not path.exists(), f"{relative} must remain absent until backed by runtime evidence")


def validate_doc_coverage() -> None:
    doc = DOC_COVERAGE_PATH.read_text(encoding="utf-8")
    for service in ("cloud-compute", "cloud-observability"):
        matching_rows = [line for line in doc.splitlines() if f"`{service}`" in line]
        require(matching_rows, f"DOC-COVERAGE missing {service} row")
        require(any("no implementation directory" in row for row in matching_rows), f"{service} row must call out implementation-directory absence")


def validate() -> None:
    validate_absent_directories()
    validate_gate(load_json(RESOURCE_CATALOG_PATH), RESOURCE_CATALOG_PATH, RESOURCE_BLOCKED_CLAIMS)
    validate_gate(load_json(OBSERVABILITY_TARGET_PATH), OBSERVABILITY_TARGET_PATH, OBSERVABILITY_BLOCKED_CLAIMS)
    validate_gate(load_json(TAXONOMY_PATH), TAXONOMY_PATH, TAXONOMY_BLOCKED_CLAIMS)
    validate_doc_coverage()


if __name__ == "__main__":
    validate()
    print("cloud absence/non-claim check passed")
