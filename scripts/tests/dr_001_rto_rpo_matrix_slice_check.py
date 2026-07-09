#!/usr/bin/env python3
"""Focused contract check for DR-001 RTO/RPO matrix slice.

This is intentionally contract/fixture-only. It validates the ADR-0343 first
slice without claiming runtime pack activation, live DR execution, or auditor
readiness.
"""

from __future__ import annotations

import argparse
import copy
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_SCHEMA_PATH = REPO_ROOT / "specs" / "microservices" / "manifest-schema.json"
COMPLIANCE_FLOORS_PATH = REPO_ROOT / "specs" / "compliance-pack-floors.json"
DR_BC_PATH = REPO_ROOT / "specs" / "dr-business-continuity.json"
FIXTURE_PATH = REPO_ROOT / "specs" / "fixtures" / "dr-rto-rpo-matrix" / "dr-001-dashboard-manifest.fixture.json"
ADR_PATH = REPO_ROOT / "docs" / "decisions" / "ADR-0343-dr-rto-rpo-matrix-per-microservice-per-compliance-pack.md"

REQUIRED_DR_FIELDS = {
    "rto_p99_seconds",
    "rpo_p99_seconds",
    "multi_region_active_active",
    "backup_substrate",
    "failover_runbook",
}
OPTIONAL_DR_FIELDS = {"dr_tier", "last_drill_evidence_id", "replication_shape"}
EXPECTED_PACK_FLOORS: dict[str, dict[str, Any]] = {
    "HIPAA-2024": {
        "rto_p99_seconds": 3600,
        "rpo_p99_seconds": 300,
        "multi_region_required": True,
        "drill_cadence_required": "quarterly",
    },
    "PCI-DSS-L1-v4": {
        "rto_p99_seconds": 86400,
        "rpo_p99_seconds": 3600,
        "multi_region_required": False,
        "drill_cadence_required": "annual",
    },
    "SOC2-T2": {
        "rto_p99_seconds": 14400,
        "rpo_p99_seconds": 900,
        "multi_region_required": False,
        "drill_cadence_required": "annual",
    },
    "EU-AI-ACT-2024-HIGH-RISK": {
        "rto_p99_seconds": 1800,
        "rpo_p99_seconds": 300,
        "multi_region_required": True,
        "drill_cadence_required": "quarterly",
    },
    "KR-CSAP-v3.1": {
        "rto_p99_seconds": 3600,
        "rpo_p99_seconds": 900,
        "multi_region_required": True,
        "drill_cadence_required": "semi-annual",
    },
    "ISO27001-2022": {
        "rto_p99_seconds": 14400,
        "rpo_p99_seconds": 3600,
        "multi_region_required": False,
        "drill_cadence_required": "annual",
    },
    "SOX-404": {
        "rto_p99_seconds": 14400,
        "rpo_p99_seconds": 3600,
        "multi_region_required": False,
        "drill_cadence_required": "annual",
    },
    "KR-PIPA-2023-amendment": {
        "rto_p99_seconds": 14400,
        "rpo_p99_seconds": 900,
        "multi_region_required": False,
        "drill_cadence_required": "semi-annual",
    },
}

CADENCE_RANK = {
    "quarterly-plus-ad-hoc": 0,
    "quarterly": 1,
    "semi-annual": 2,
    "annual": 3,
    "annual-tabletop": 4,
}
CADENCE_MAX_AGE_DAYS = {
    "quarterly-plus-ad-hoc": 92,
    "quarterly": 92,
    "semi-annual": 183,
    "annual": 366,
    "annual-tabletop": 366,
}
FORBIDDEN_POSITIVE_CLAIMS = {
    "production ready",
    "runtime control loop",
    "live pack activation",
    "tenant activation ready",
    "auditor accepted",
    "certified",
    "drill executed in production",
}
REQUIRED_NONCLAIMS = {
    "no_runtime_dr_execution",
    "no_pack_activation_runtime_gate",
    "no_auditor_or_certification_claim",
    "no_tenant_workload_or_production_readiness_claim",
}


class CheckFailure(SystemExit):
    pass


def fail(message: str) -> None:
    raise CheckFailure(f"DR-001 RTO/RPO matrix check failed: {message}")


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def rel(path: Path) -> str:
    return str(path.relative_to(REPO_ROOT))


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {rel(path)}: {exc}")


def parse_instant(value: str, field: str) -> datetime:
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        fail(f"{field} must be an RFC3339 timestamp: {exc}")
    if parsed.tzinfo is None:
        fail(f"{field} must include timezone")
    return parsed.astimezone(timezone.utc)


def contains_forbidden_positive_claim(value: Any) -> bool:
    text = json.dumps(value, sort_keys=True).lower() if not isinstance(value, str) else value.lower()
    return any(marker in text for marker in FORBIDDEN_POSITIVE_CLAIMS)


def validate_manifest_schema(schema: dict[str, Any]) -> set[str]:
    required_top_level = set(schema.get("required", []))
    require("microservice" in required_top_level, "manifest schema must keep microservice as a required top-level field")
    require("sharding_automation" in required_top_level, "manifest schema must keep existing required sharding_automation gate")

    dr = schema.get("properties", {}).get("dr")
    require(isinstance(dr, dict), "manifest schema must define properties.dr")
    require(set(dr.get("required", [])) >= REQUIRED_DR_FIELDS, "manifest schema dr.required missing required DR fields")
    properties = dr.get("properties", {})
    require(set(properties) >= REQUIRED_DR_FIELDS | OPTIONAL_DR_FIELDS, "manifest schema dr.properties missing required/optional DR fields")
    require(properties["rto_p99_seconds"].get("type") == "integer", "dr.rto_p99_seconds must be integer")
    require(properties["rpo_p99_seconds"].get("type") == "integer", "dr.rpo_p99_seconds must be integer")
    require(properties["multi_region_active_active"].get("type") == "boolean", "dr.multi_region_active_active must be boolean")
    require(properties["backup_substrate"].get("minItems") == 1, "dr.backup_substrate must require at least one substrate")
    allowlist = set(properties["backup_substrate"].get("items", {}).get("enum", []))
    require(len(allowlist) >= 10, "dr.backup_substrate allowlist must preserve ADR-0343 substrate coverage")
    require("postgres_wal_g" in allowlist, "dr.backup_substrate allowlist must include postgres_wal_g")
    require("audit_chain_merkle_seal" in allowlist, "dr.backup_substrate allowlist must include audit_chain_merkle_seal")
    runbook_pattern = properties["failover_runbook"].get("pattern")
    require(runbook_pattern == r"^runbooks/[A-Za-z0-9._/-]+\.md$", "dr.failover_runbook must stay scoped to microservice runbooks/*.md")
    require("T1" in properties["dr_tier"].get("enum", []), "dr.dr_tier must preserve ADR-0241 T1..T4 shorthand")
    return allowlist


def validate_floor_table(table: dict[str, Any], manifest_allowlist: set[str]) -> dict[str, dict[str, Any]]:
    meta = table.get("_meta", {})
    require(meta.get("binding_adr") == "ADR-0343", "compliance-pack floors must bind to ADR-0343")
    require("MIN" in meta.get("stringency_algorithm", ""), "floor table must document MIN-over-upper-bound stringency algorithm")
    require(set(meta.get("backup_substrate_allowlist", [])) == manifest_allowlist, "floor table backup_substrate_allowlist must match manifest schema enum")

    packs = table.get("packs")
    require(isinstance(packs, list) and packs, "compliance-pack floors must define packs")
    by_id: dict[str, dict[str, Any]] = {}
    for pack in packs:
        pack_id = pack.get("pack_id")
        require(pack_id not in by_id, f"duplicate pack floor {pack_id}")
        by_id[pack_id] = pack
        floor = pack.get("dr_floor", {})
        for field in ["rto_p99_seconds", "rpo_p99_seconds", "multi_region_required", "drill_cadence_required"]:
            require(field in floor, f"{pack_id}: dr_floor missing {field}")
        require(pack.get("_meta", {}).get("cedar_fragment_id", "").startswith("pack-"), f"{pack_id}: missing pack-scoped Cedar DR floor fragment id")
        require(pack.get("_meta", {}).get("regulator_citations"), f"{pack_id}: missing regulator citations")

    require(set(EXPECTED_PACK_FLOORS) <= set(by_id), f"missing expected pack floors {sorted(set(EXPECTED_PACK_FLOORS) - set(by_id))}")
    for pack_id, expected in EXPECTED_PACK_FLOORS.items():
        actual = by_id[pack_id]["dr_floor"]
        for field, value in expected.items():
            require(actual.get(field) == value, f"{pack_id}: expected {field}={value!r}, got {actual.get(field)!r}")
    require("process_floors" in by_id["SOX-404"]["dr_floor"], "SOX-404 must preserve process_floors refinement")
    require(
        "general_ledger_journal_entry" in by_id["SOX-404"]["dr_floor"]["process_floors"],
        "SOX-404 process_floors must include general_ledger_journal_entry",
    )
    require("data_class_floors" in by_id["KR-PIPA-2023-amendment"]["dr_floor"], "KR-PIPA must preserve data_class_floors refinement")
    require(
        "PI_KR_RESIDENT_REGISTRATION_NUMBER" in by_id["KR-PIPA-2023-amendment"]["dr_floor"]["data_class_floors"],
        "KR-PIPA data_class_floors must include PI_KR_RESIDENT_REGISTRATION_NUMBER",
    )
    return by_id


def validate_dr_business_continuity(dr_bc: dict[str, Any]) -> None:
    tiers = {tier.get("tier"): tier for tier in dr_bc.get("properties", {}).get("tiers", {}).get("default", [])}
    require(set(tiers) == {"T1", "T2", "T3", "T4"}, "DR business-continuity tier defaults must preserve T1..T4")
    require(tiers["T1"].get("drill_cadence") == "quarterly-plus-ad-hoc", "T1 drill cadence must be quarterly-plus-ad-hoc")
    require(tiers["T4"].get("rpo_seconds") == 3600, "T4 RPO default must remain 3600 seconds")


def select_floor(pack: dict[str, Any], refinement: dict[str, str] | None) -> dict[str, Any]:
    floor = pack["dr_floor"]
    if not refinement:
        return floor
    if "process" in refinement:
        return floor.get("process_floors", {}).get(refinement["process"], floor)
    if "data_class" in refinement:
        return floor.get("data_class_floors", {}).get(refinement["data_class"], floor)
    return floor


def shortest_cadence(cadences: list[str]) -> str:
    for cadence in cadences:
        require(cadence in CADENCE_RANK, f"unknown drill cadence {cadence!r}")
    return min(cadences, key=lambda cadence: CADENCE_RANK[cadence])


def compute_effective_dr(fixture: dict[str, Any], pack_floors: dict[str, dict[str, Any]]) -> dict[str, Any]:
    manifest = fixture["manifest_dr_block"]
    refinements = fixture.get("pack_refinements", {})
    selected_pack_floors = []
    for pack_id in fixture["activated_pack_ids"]:
        require(pack_id in pack_floors, f"fixture activates unknown pack {pack_id}")
        selected_pack_floors.append(select_floor(pack_floors[pack_id], refinements.get(pack_id)))

    rto_values = [manifest["rto_p99_seconds"]] + [floor["rto_p99_seconds"] for floor in selected_pack_floors]
    rpo_values = [manifest["rpo_p99_seconds"]] + [floor["rpo_p99_seconds"] for floor in selected_pack_floors]
    cadences = [fixture["manifest_drill_cadence_default"]] + [floor["drill_cadence_required"] for floor in selected_pack_floors]
    return {
        "rto_p99_seconds": min(rto_values),
        "rpo_p99_seconds": min(rpo_values),
        "multi_region_active_active": bool(manifest["multi_region_active_active"] or any(floor["multi_region_required"] for floor in selected_pack_floors)),
        "drill_cadence": shortest_cadence(cadences),
    }


def validate_fixture(fixture: dict[str, Any], manifest_allowlist: set[str], pack_floors: dict[str, dict[str, Any]]) -> None:
    meta = fixture.get("_meta", {})
    require(meta.get("fixture_id") == "DR-001-dashboard-manifest-fixture", "fixture_id mismatch")
    require(meta.get("status") == "contract_fixture_only", "fixture must remain contract_fixture_only")
    require(set(meta.get("nonclaims", [])) >= REQUIRED_NONCLAIMS, f"fixture nonclaims missing {sorted(REQUIRED_NONCLAIMS - set(meta.get('nonclaims', [])))}")
    require(not contains_forbidden_positive_claim(meta.get("can_claim_now", [])), "fixture can_claim_now contains forbidden runtime/readiness claim")
    require(not contains_forbidden_positive_claim(fixture.get("dashboard_manifest", {})), "dashboard fixture contains forbidden runtime/readiness claim")

    sources = fixture.get("sources", {})
    require(sources.get("adr") == rel(ADR_PATH), "fixture source adr path mismatch")
    require(sources.get("manifest_schema") == rel(MANIFEST_SCHEMA_PATH), "fixture source manifest_schema path mismatch")
    require(sources.get("compliance_pack_floors") == rel(COMPLIANCE_FLOORS_PATH), "fixture source compliance_pack_floors path mismatch")

    microservice_root = fixture.get("microservice_root")
    require(isinstance(microservice_root, str) and microservice_root.startswith("oya/"), "fixture microservice_root must point to an oya/* service fixture root")
    manifest = fixture.get("manifest_dr_block")
    require(isinstance(manifest, dict), "fixture missing manifest_dr_block")
    require(REQUIRED_DR_FIELDS <= set(manifest), f"manifest_dr_block missing {sorted(REQUIRED_DR_FIELDS - set(manifest))}")
    require(set(manifest.get("backup_substrate", [])) <= manifest_allowlist, "manifest_dr_block uses non-allowlisted backup_substrate")
    runbook_rel = manifest.get("failover_runbook", "")
    require(re.fullmatch(r"runbooks/[A-Za-z0-9._/-]+\.md", runbook_rel or "") is not None, "manifest failover_runbook must be runbooks/*.md")
    runbook_path = REPO_ROOT / microservice_root / runbook_rel
    require(runbook_path.is_file(), f"manifest failover_runbook does not resolve: {rel(runbook_path)}")

    evidence = fixture.get("drill_evidence", {})
    require(evidence.get("evidence_id") == manifest.get("last_drill_evidence_id"), "drill evidence id must match manifest.last_drill_evidence_id")
    require(evidence.get("status") == "fresh", "drill evidence fixture must represent fresh evidence")
    effective = compute_effective_dr(fixture, pack_floors)
    require(fixture.get("expected_effective_dr") == effective, f"expected_effective_dr must match computed value {effective}")

    as_of = parse_instant(fixture.get("as_of", ""), "as_of")
    executed_at = parse_instant(evidence.get("executed_at", ""), "drill_evidence.executed_at")
    age_days = (as_of - executed_at).total_seconds() / 86400
    max_age = CADENCE_MAX_AGE_DAYS[effective["drill_cadence"]]
    require(age_days <= max_age, f"drill evidence age {age_days:.1f}d exceeds {effective['drill_cadence']} max {max_age}d")

    dashboard = fixture.get("dashboard_manifest", {})
    require(dashboard.get("dashboard_id") == "per-pack-dr-floor-satisfaction", "dashboard_manifest dashboard_id mismatch")
    require(dashboard.get("status") == "fixture-only", "dashboard_manifest must remain fixture-only")
    require(dashboard.get("generated_from") == [rel(COMPLIANCE_FLOORS_PATH), "manifest_dr_block"], "dashboard_manifest generated_from mismatch")
    rows = dashboard.get("rows", [])
    require(isinstance(rows, list) and len(rows) == 1, "dashboard_manifest fixture must contain exactly one row")
    row = rows[0]
    require(row.get("microservice") == fixture.get("microservice"), "dashboard row microservice mismatch")
    require(row.get("activated_pack_ids") == fixture.get("activated_pack_ids"), "dashboard row activated_pack_ids mismatch")
    require(row.get("declared_dr") == manifest, "dashboard row declared_dr must mirror manifest_dr_block")
    require(row.get("effective_dr") == effective, "dashboard row effective_dr must match computed effective DR")
    require(row.get("failover_runbook") == rel(runbook_path), "dashboard row failover_runbook must resolve to service runbook")
    require(row.get("last_drill_evidence_id") == evidence.get("evidence_id"), "dashboard row drill evidence mismatch")
    require(row.get("drill_evidence_fresh") is True, "dashboard row must mark drill_evidence_fresh=true for this fixture")
    require(row.get("floor_satisfaction") == "satisfied", "dashboard row floor_satisfaction must be satisfied")


def validate_all(manifest_schema: dict[str, Any], floors: dict[str, Any], dr_bc: dict[str, Any], fixture: dict[str, Any]) -> None:
    manifest_allowlist = validate_manifest_schema(manifest_schema)
    pack_floors = validate_floor_table(floors, manifest_allowlist)
    validate_dr_business_continuity(dr_bc)
    validate_fixture(fixture, manifest_allowlist, pack_floors)


def run_self_tests(manifest_schema: dict[str, Any], floors: dict[str, Any], dr_bc: dict[str, Any], fixture: dict[str, Any]) -> None:
    def expect_rejected(label: str, mutator: Callable[[dict[str, Any]], None]) -> None:
        candidate = copy.deepcopy(fixture)
        mutator(candidate)
        try:
            validate_all(manifest_schema, floors, dr_bc, candidate)
        except CheckFailure:
            return
        fail(f"self-test mutation was accepted: {label}")

    expect_rejected("missing required manifest field", lambda data: data["manifest_dr_block"].pop("rto_p99_seconds"))
    expect_rejected("incorrect max-over-seconds algorithm", lambda data: data.update({"expected_effective_dr": {**data["expected_effective_dr"], "rto_p99_seconds": 7200}}))
    expect_rejected("non-resolving runbook", lambda data: data["manifest_dr_block"].update({"failover_runbook": "runbooks/missing-dr-runbook.md"}))
    expect_rejected("stale drill evidence", lambda data: data["drill_evidence"].update({"executed_at": "2025-12-01T00:00:00Z"}))
    expect_rejected("dashboard effective value drift", lambda data: data["dashboard_manifest"]["rows"][0].update({"effective_dr": {**data["expected_effective_dr"], "rpo_p99_seconds": 900}}))
    expect_rejected("runtime readiness overclaim", lambda data: data["_meta"].update({"can_claim_now": ["production ready live pack activation"]}))
    print("DR-001 RTO/RPO matrix self-tests passed")


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate DR-001 ADR-0343 first-slice contract fixture")
    parser.add_argument("--self-test", action="store_true", help="run mutation self-tests")
    args = parser.parse_args()

    manifest_schema = load_json(MANIFEST_SCHEMA_PATH)
    floors = load_json(COMPLIANCE_FLOORS_PATH)
    dr_bc = load_json(DR_BC_PATH)
    fixture = load_json(FIXTURE_PATH)
    validate_all(manifest_schema, floors, dr_bc, fixture)
    if args.self_test:
        run_self_tests(manifest_schema, floors, dr_bc, fixture)
    else:
        print(f"DR-001 RTO/RPO matrix slice check passed: {rel(FIXTURE_PATH)}")


if __name__ == "__main__":
    try:
        main()
    except CheckFailure as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
