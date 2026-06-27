#!/usr/bin/env python3
"""Validate cloud compute/observability absence remains an explicit non-claim gate."""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
RESOURCE_TARGET_PATH = REPO_ROOT / "specs" / "cloud-resource-catalog-target.json"
OBSERVABILITY_TARGET_PATH = REPO_ROOT / "specs" / "cloud-observability-slo-target.json"
TAXONOMY_PATH = REPO_ROOT / "specs" / "cloud-hyperscaler-parity-taxonomy.json"
ABSENT_DIRS = {
    "cloud-compute": REPO_ROOT / "cloud" / "cloud-compute",
    "cloud-observability": REPO_ROOT / "cloud" / "cloud-observability",
}
REQUIRED_BLOCKED_CLAIMS = {
    "provider provisioning",
    "measured slo evidence",
    "tenant workload readiness",
    "hyperscaler feature parity",
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


def text(value: object) -> str:
    if isinstance(value, dict):
        return " ".join(text(v) for v in value.values())
    if isinstance(value, (list, tuple, set)):
        return " ".join(text(v) for v in value)
    return str(value).lower()


def validate_absent_directories() -> None:
    for label, path in ABSENT_DIRS.items():
        require(not path.exists(), f"{label} directory must remain absent until backed by runtime evidence: {path.relative_to(REPO_ROOT)}")


def validate_resource_target(spec: dict) -> None:
    services = {row.get("service"): row for row in spec.get("services", [])}
    require("cloud-compute" in services, "resource target must keep cloud-compute target vocabulary discoverable")
    compute = services["cloud-compute"]
    require({"VmInstance", "K8sCluster", "NodePool", "Function", "Container"} <= set(compute.get("resource_types", [])), "cloud-compute target resource types drifted")
    gate = spec.get("implementation_directory_absence_gate", {}).get("cloud-compute")
    require(gate, "resource target missing cloud-compute implementation_directory_absence_gate")
    require(gate.get("directory") == "cloud/cloud-compute", "cloud-compute absence gate must name cloud/cloud-compute")
    require(gate.get("status_when_directory_absent") == "target_non_claim_only", "cloud-compute absence status must be target_non_claim_only")
    gate_text = text(gate)
    require("stub service directory" in gate_text, "cloud-compute gate must block stub service-directory backfill")
    for phrase in ["live provider provisioning", "implemented compute runtime", "tenant workload readiness", "hyperscaler feature parity"]:
        require(phrase in gate_text, f"cloud-compute gate missing blocked claim {phrase!r}")
    require("target spec only" in text(spec.get("non_claims", [])), "resource target non_claims must keep target-spec-only language")


def validate_observability_target(spec: dict) -> None:
    gate = spec.get("implementation_directory_absence_gate", {}).get("cloud-observability")
    require(gate, "observability target missing cloud-observability implementation_directory_absence_gate")
    require(gate.get("directory") == "cloud/cloud-observability", "cloud-observability absence gate must name cloud/cloud-observability")
    require(gate.get("status_when_directory_absent") == "target_non_claim_only", "cloud-observability absence status must be target_non_claim_only")
    gate_text = text(gate)
    for phrase in ["stub collector", "slo engine", "measured-evidence claim", "runtime observability engine", "measured slo evidence", "public sla or slo", "production readiness"]:
        require(phrase in gate_text, f"cloud-observability gate missing non-claim phrase {phrase!r}")
    non_claims = text(spec.get("non_claims", []))
    for phrase in ["target spec only", "does not implement the otel collector", "no measured slo"]:
        require(phrase in non_claims, f"observability target non_claims missing {phrase!r}")


def validate_taxonomy(spec: dict) -> None:
    gate = spec.get("implementation_directory_absence_gate")
    require(gate, "taxonomy missing implementation_directory_absence_gate")
    require(gate.get("gate_id") == "cloud-compute-observability-absence-nonclaim", "taxonomy absence gate id drifted")
    require(set(gate.get("paths_that_must_remain_absent_until_runtime_evidence", [])) == {"cloud/cloud-compute", "cloud/cloud-observability"}, "taxonomy must name both absent implementation directories")
    require(gate.get("status_when_absent") == "target_non_claim_only", "taxonomy absence status must be target_non_claim_only")
    gate_text = text(gate)
    for phrase in REQUIRED_BLOCKED_CLAIMS:
        require(phrase in gate_text, f"taxonomy absence gate missing blocked claim phrase {phrase!r}")
    mappings = {row.get("category_id"): row for row in spec.get("local_oyatie_mapping", [])}
    for category_id in ["compute_instances", "observability_operations"]:
        require(category_id in mappings, f"taxonomy missing local mapping for {category_id}")
        require("feature parity" in text(mappings[category_id].get("cannot_claim_yet", [])), f"{category_id} mapping must keep feature parity blocked")
    can_claim_now = text(gate.get("can_claim_now", []))
    require("cloud/cloud-compute" in can_claim_now and "cloud/cloud-observability" in can_claim_now and "absent" in can_claim_now, "taxonomy can_claim_now must be limited to clean-branch absence")


def main() -> None:
    validate_absent_directories()
    validate_resource_target(load_json(RESOURCE_TARGET_PATH))
    validate_observability_target(load_json(OBSERVABILITY_TARGET_PATH))
    validate_taxonomy(load_json(TAXONOMY_PATH))
    print("cloud absence/non-claim check passed")


if __name__ == "__main__":
    main()
