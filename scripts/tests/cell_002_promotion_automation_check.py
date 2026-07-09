#!/usr/bin/env python3
"""Validate CELL-002 promotion/rebalancer/lifecycle/autosharding contract fixtures."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "specs" / "cell-002-promotion-automation-contract.json"
FIXTURE_PATH = REPO_ROOT / "specs" / "fixtures" / "cell-002-promotion-automation" / "rollback-audit-row.json"
ROOT_HUB_PATH = REPO_ROOT / "specs" / "root-hub-pointers.json"
CELL_LIFECYCLE_MANIFEST_PATH = REPO_ROOT / "cloud" / "cell-lifecycle" / "manifest.json"
CELL_REBALANCER_MANIFEST_PATH = REPO_ROOT / "cloud" / "cell-rebalancer" / "manifest.json"

REQUIRED_SOURCE_ADRS = {"ADR-0341", "ADR-0348", "ADR-0351"}
REQUIRED_GATE_INPUT_IDS = {
    "G1_error_budget",
    "G2_warm_soak",
    "G3_canary_cohort",
    "G4_cross_cell_mesh",
    "G5_tenant_class_coverage",
    "G6_compliance_pack_coverage",
}
REQUIRED_GATE_FIELDS = {
    "id",
    "name",
    "source_adr",
    "evidence_authority",
    "required_evidence_fields",
    "refusal_behavior",
}
REQUIRED_CANDIDATE_FILTERS = {
    "residency_domain",
    "compliance_packs",
    "cell_placement_class",
    "capacity_headroom",
    "cedar_permit",
}
REQUIRED_SHARDING_SUBBLOCKS = {"autosharding", "auto_rebalance", "dynamic_sharding"}
REQUIRED_DYNAMIC_THRESHOLDS = {
    "hot_split_threshold_p99_ms",
    "hot_split_utilization_threshold_percent",
    "cold_merge_utilization_threshold_percent",
    "cold_merge_minimum_quiet_hours",
}
REQUIRED_ROLLBACK_AUDIT_FIELDS = {
    "event_type",
    "tenant_id",
    "source_cell_id",
    "target_cell_id",
    "cell_id",
    "pre_state",
    "post_state",
    "residency_check_result",
    "compliance_pack_check_result",
    "cedar_permit_id",
    "initiated_by",
    "audit_chain_id",
    "rollback_pointer",
    "evidence_pack_id",
}
REQUIRED_NONCLAIMS = {
    "no_runtime_cell_orchestrator",
    "no_live_tenant_migration",
    "no_audit_chain_writer",
    "no_provider_or_kubernetes_mutation",
    "no_production_readiness_claim",
    "no_measured_slo_claim",
}
FORBIDDEN_POSITIVE_PATTERNS = [
    re.compile(pattern)
    for pattern in [
        r"\bruntime\s+(auto[-_ ]?rebalance|autosharding|dynamic\s+sharding|cell\s+promotion)\s+(is\s+)?(implemented|ready|available|live)\b",
        r"\blive\s+(tenant\s+)?migration\s+(is\s+)?(implemented|ready|available|enabled)\b",
        r"\bproduction\s+ready\b",
        r"\bmeasured\s+slo\b.{0,30}\b(available|green|passed|ready)\b",
        r"\baudit[-_ ]chain\s+writer\s+(is\s+)?(implemented|ready|available|live)\b",
    ]
]


def fail(message: str) -> NoReturn:
    print(f"CELL-002 promotion automation check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def text(value: object) -> str:
    if isinstance(value, dict):
        return " ".join(text(v) for v in value.values())
    if isinstance(value, (list, tuple, set)):
        return " ".join(text(v) for v in value)
    return str(value).lower()


def normalized(value: object) -> str:
    return re.sub(r"[^a-z0-9]+", " ", text(value)).strip()


def contains_forbidden_positive_claim(value: object) -> bool:
    haystack = f" {normalized(value)} "
    return any(pattern.search(haystack) for pattern in FORBIDDEN_POSITIVE_PATTERNS)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path.relative_to(REPO_ROOT)}: {exc}")


def validate_sharding_manifest(path_label: str, manifest: dict) -> None:
    block = manifest.get("sharding_automation")
    require(isinstance(block, dict), f"{path_label}: sharding_automation must be an object")
    require(REQUIRED_SHARDING_SUBBLOCKS <= set(block), f"{path_label}: sharding_automation missing sub-blocks")

    autosharding = block.get("autosharding", {})
    if isinstance(autosharding, str):
        mode = autosharding
        intended = autosharding
    else:
        mode = autosharding.get("mode")
        intended = autosharding.get("intended_control_plane")
    require(mode != "manual" and intended == "control_plane_driven", f"{path_label}: autosharding must preserve control_plane_driven intent and refuse manual mode")

    auto_rebalance = block.get("auto_rebalance", {})
    require(auto_rebalance.get("honors_residency") is True, f"{path_label}: auto_rebalance must honor residency")
    require(auto_rebalance.get("honors_compliance_packs") is True, f"{path_label}: auto_rebalance must honor compliance packs")
    require(auto_rebalance.get("trigger_load_skew_threshold_percent") == 30, f"{path_label}: auto_rebalance threshold must be 30 percent")
    if auto_rebalance.get("enabled") is True:
        require(auto_rebalance.get("audit_chain_emit") is True, f"{path_label}: enabled auto_rebalance must emit audit-chain events")

    dynamic = block.get("dynamic_sharding", {})
    require(REQUIRED_DYNAMIC_THRESHOLDS <= set(dynamic), f"{path_label}: dynamic_sharding thresholds incomplete")
    require(dynamic.get("hot_split_threshold_p99_ms") == 50, f"{path_label}: hot split p99 threshold must be 50 ms")
    require(dynamic.get("hot_split_utilization_threshold_percent") == 80, f"{path_label}: hot split utilization threshold must be 80 percent")
    require(dynamic.get("cold_merge_utilization_threshold_percent") == 20, f"{path_label}: cold merge utilization threshold must be 20 percent")
    require(dynamic.get("cold_merge_minimum_quiet_hours") == 24, f"{path_label}: cold merge quiet window must be 24 hours")
    if dynamic.get("enabled") is True:
        require(dynamic.get("audit_chain_emit") is True, f"{path_label}: enabled dynamic_sharding must emit audit-chain events")


def validate_documents(spec: dict, fixture: dict, root_hub: dict, lifecycle_manifest: dict, rebalancer_manifest: dict) -> None:
    require(spec.get("spec_id") == "CELL-002-PROMOTION-AUTOMATION-CONTRACT", "unexpected spec_id")
    require(spec.get("authoring_task") == "CELL-002", "authoring_task must be CELL-002")
    require(spec.get("status") == "Proposed-target", "status must remain Proposed-target until runtime evidence exists")
    require(set(spec.get("source_adrs", [])) >= REQUIRED_SOURCE_ADRS, "source_adrs must include ADR-0341, ADR-0348, and ADR-0351")
    authority = spec.get("authority", {})
    require(authority.get("accepted_boundary_adr") == "ADR-0351", "ADR-0351 must be the accepted service-boundary authority")
    proposed = authority.get("proposed_adr_boundary", {})
    require(proposed.get("ADR-0341") == "planning-context-only", "ADR-0341 must remain planning-context-only")
    require(proposed.get("ADR-0348") == "planning-context-only", "ADR-0348 must remain planning-context-only")
    require(authority.get("runtime_mutation_authority") == "none", "runtime mutation authority must be none for this slice")

    controls = spec.get("claim_controls", {})
    for key in [
        "metadata_only",
        "service_plan_only",
        "fixture_only",
        "no_runtime_mutation",
        "no_live_migration",
        "no_audit_chain_writer",
        "strict_separation",
    ]:
        require(controls.get(key) is True, f"claim_controls.{key} must be true")
    require(not contains_forbidden_positive_claim(controls.get("can_claim_now", [])), "can_claim_now contains forbidden runtime/readiness claim")
    require(REQUIRED_NONCLAIMS <= set(controls.get("blocked_claim_families", [])), "claim_controls missing blocked claim families")

    nonclaim_ids = {item.get("id") for item in spec.get("nonclaims", [])}
    require(REQUIRED_NONCLAIMS <= nonclaim_ids, f"missing nonclaims {sorted(REQUIRED_NONCLAIMS - nonclaim_ids)}")

    gate = spec.get("promotion_gate", {})
    require(gate.get("gate_id") == "cell_promotion_six_input_gate", "promotion_gate.gate_id mismatch")
    inputs = gate.get("six_inputs", [])
    require({entry.get("id") for entry in inputs} == REQUIRED_GATE_INPUT_IDS, "promotion gate must define exactly the six ADR-0341 inputs")
    for entry in inputs:
        require(REQUIRED_GATE_FIELDS <= set(entry), f"{entry.get('id')}: gate input fields incomplete")
        require(entry.get("source_adr") == "ADR-0341", f"{entry.get('id')}: gate input must cite ADR-0341")
        require(entry.get("refusal_behavior") == "fail_closed_no_promotion", f"{entry.get('id')}: gate input must fail closed")
        require(entry.get("required_evidence_fields"), f"{entry.get('id')}: required evidence fields must be non-empty")

    lifecycle = spec.get("cell_lifecycle_service_plan", {})
    require(lifecycle.get("service") == "cell-lifecycle", "cell_lifecycle_service_plan.service mismatch")
    require(lifecycle.get("source_path") == "cloud/cell-lifecycle/PRD.md", "cell-lifecycle source_path mismatch")
    require(lifecycle.get("owns") == "logical_cell_state_machine", "cell-lifecycle must own the logical cell state machine")
    require(lifecycle.get("delegates_tenant_migration_to") == "cell-rebalancer", "cell-lifecycle must delegate tenant migration to cell-rebalancer")
    require(lifecycle.get("does_not_own") == ["infrastructure_provisioning", "tenant_migration", "traffic_routing"], "cell-lifecycle boundary must stay narrow")

    rebalancer = spec.get("cell_rebalancer_service_plan", {})
    require(rebalancer.get("service") == "cell-rebalancer", "cell_rebalancer_service_plan.service mismatch")
    require(rebalancer.get("source_path") == "cloud/cell-rebalancer/PRD.md", "cell-rebalancer source_path mismatch")
    require(rebalancer.get("owns") == "across_cell_tenant_migration_workflow", "cell-rebalancer must own across-cell tenant migration workflow")
    require(rebalancer.get("does_not_own") == ["cell_lifecycle_state_machine", "first_time_placement", "within_cell_shuffle_sharding"], "cell-rebalancer boundary must stay narrow")
    rebalance = rebalancer.get("residency_honoring_rebalance", {})
    require(rebalance.get("trigger") == "auto_rebalance", "rebalance trigger must be auto_rebalance")
    require(REQUIRED_CANDIDATE_FILTERS <= set(rebalance.get("candidate_filters", [])), "residency-honoring rebalance filters incomplete")
    require(rebalance.get("refusal_behavior") == "fail_closed_operator_escalation", "rebalance refusal behavior must fail closed")
    require(rebalance.get("audit_fixture") == str(FIXTURE_PATH.relative_to(REPO_ROOT)), "rebalance audit fixture path mismatch")

    sharding = spec.get("sharding_automation_manifest_contract", {})
    require(sharding.get("canonical_autosharding_mode") == "control_plane_driven", "canonical autosharding mode must be control_plane_driven")
    require(REQUIRED_SHARDING_SUBBLOCKS <= set(sharding.get("required_subblocks", [])), "required sharding sub-blocks incomplete")
    require(sharding.get("default_thresholds", {}).get("auto_rebalance_load_skew_threshold_percent") == 30, "default auto-rebalance threshold mismatch")
    require(sharding.get("default_thresholds", {}).get("hot_split_threshold_p99_ms") == 50, "default hot split p99 mismatch")
    require(sharding.get("default_thresholds", {}).get("hot_split_utilization_threshold_percent") == 80, "default hot split utilization mismatch")
    require(sharding.get("default_thresholds", {}).get("cold_merge_utilization_threshold_percent") == 20, "default cold merge utilization mismatch")
    require(sharding.get("default_thresholds", {}).get("cold_merge_minimum_quiet_hours") == 24, "default cold merge quiet window mismatch")
    require(
        set(sharding.get("service_manifest_inputs", [])) == {"cloud/cell-lifecycle/manifest.json", "cloud/cell-rebalancer/manifest.json"},
        "service manifest inputs must be exactly cell-lifecycle and cell-rebalancer",
    )

    validate_sharding_manifest("cloud/cell-lifecycle/manifest.json", lifecycle_manifest)
    validate_sharding_manifest("cloud/cell-rebalancer/manifest.json", rebalancer_manifest)

    rollback = spec.get("rollback_audit_fixture", {})
    require(rollback.get("path") == str(FIXTURE_PATH.relative_to(REPO_ROOT)), "rollback fixture path mismatch")
    require(REQUIRED_ROLLBACK_AUDIT_FIELDS <= set(rollback.get("required_fields", [])), "rollback audit required fields incomplete")
    require(rollback.get("non_claim") and "does not emit" in rollback.get("non_claim"), "rollback fixture must state non-emission boundary")

    require(fixture.get("contract") == str(SPEC_PATH.relative_to(REPO_ROOT)), "fixture contract path mismatch")
    require(fixture.get("fixture_kind") == "rollback_audit_row", "fixture_kind must be rollback_audit_row")
    row = fixture.get("audit_row", {})
    require(REQUIRED_ROLLBACK_AUDIT_FIELDS <= set(row), "fixture audit row missing required fields")
    require(row.get("event_type") == "cell.rebalance.rollback_audit_fixture", "fixture event_type mismatch")
    require(row.get("post_state") == "RolledBack", "fixture post_state must be RolledBack")
    require(row.get("residency_check_result") == "honored", "fixture residency result must be honored")
    require(row.get("compliance_pack_check_result") == "honored", "fixture compliance-pack result must be honored")
    require(row.get("initiated_by") == "control_plane:cell-orchestrator", "fixture initiated_by must be the control-plane cell orchestrator")
    require(str(row.get("rollback_pointer", "")).startswith("rollback://cell-rebalancer/"), "fixture rollback pointer must be cell-rebalancer scoped")
    require(fixture.get("non_claims") and "not emitted" in " ".join(fixture.get("non_claims", [])), "fixture must carry not-emitted nonclaim")

    root_entry = root_hub.get("entry_points", {}).get("spec_cell_002_promotion_automation_contract", {})
    require(root_entry.get("current_path") == "/specs/cell-002-promotion-automation-contract.json", "root hub missing CELL-002 spec pointer")
    require(root_entry.get("kind") == "spec", "root hub CELL-002 entry must be kind=spec")
    require(root_entry.get("conflict_class") == "repo-governance-specs-docs", "root hub CELL-002 conflict class mismatch")


def main() -> None:
    validate_documents(
        load_json(SPEC_PATH),
        load_json(FIXTURE_PATH),
        load_json(ROOT_HUB_PATH),
        load_json(CELL_LIFECYCLE_MANIFEST_PATH),
        load_json(CELL_REBALANCER_MANIFEST_PATH),
    )
    print(f"CELL-002 promotion automation check passed: {SPEC_PATH.relative_to(REPO_ROOT)}")


def run_self_tests() -> None:
    spec = load_json(SPEC_PATH)
    fixture = load_json(FIXTURE_PATH)
    root_hub = load_json(ROOT_HUB_PATH)
    lifecycle_manifest = load_json(CELL_LIFECYCLE_MANIFEST_PATH)
    rebalancer_manifest = load_json(CELL_REBALANCER_MANIFEST_PATH)

    def expect_rejected(label: str, mutator: Callable[[dict, dict, dict, dict, dict], None]) -> None:
        candidate_spec = json.loads(json.dumps(spec))
        candidate_fixture = json.loads(json.dumps(fixture))
        candidate_root_hub = json.loads(json.dumps(root_hub))
        candidate_lifecycle_manifest = json.loads(json.dumps(lifecycle_manifest))
        candidate_rebalancer_manifest = json.loads(json.dumps(rebalancer_manifest))
        mutator(candidate_spec, candidate_fixture, candidate_root_hub, candidate_lifecycle_manifest, candidate_rebalancer_manifest)
        try:
            validate_documents(candidate_spec, candidate_fixture, candidate_root_hub, candidate_lifecycle_manifest, candidate_rebalancer_manifest)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected("missing G6 gate", lambda data, _fixture, _root, _life, _reb: data["promotion_gate"].update({"six_inputs": data["promotion_gate"]["six_inputs"][:-1]}))
    expect_rejected("manual autosharding", lambda data, _fixture, _root, _life, _reb: data["sharding_automation_manifest_contract"].update({"canonical_autosharding_mode": "manual"}))
    expect_rejected("missing residency filter", lambda data, _fixture, _root, _life, _reb: data["cell_rebalancer_service_plan"]["residency_honoring_rebalance"].update({"candidate_filters": ["capacity_headroom"]}))
    expect_rejected("missing rollback pointer", lambda _data, fixture_data, _root, _life, _reb: fixture_data["audit_row"].pop("rollback_pointer"))
    expect_rejected("runtime mutation control disabled", lambda data, _fixture, _root, _life, _reb: data["claim_controls"].update({"no_runtime_mutation": False}))
    expect_rejected("runtime overclaim", lambda data, _fixture, _root, _life, _reb: data["claim_controls"].update({"can_claim_now": ["runtime auto-rebalance is implemented"]}))
    expect_rejected("manifest missing dynamic sharding", lambda _data, _fixture, _root, life, _reb: life["sharding_automation"].pop("dynamic_sharding"))
    expect_rejected("manifest residency false", lambda _data, _fixture, _root, _life, reb: reb["sharding_automation"]["auto_rebalance"].update({"honors_residency": False}))
    print("CELL-002 promotion automation self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
