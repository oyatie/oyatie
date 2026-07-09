#!/usr/bin/env python3
"""Validate the RESILIENCE-001 messenger runtime-control-loop evidence contract."""
from __future__ import annotations

import copy
import json
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
CONTRACT_PATH = REPO_ROOT / "oya" / "messenger" / "resilience" / "runtime-control-loop-contract.json"
MANIFEST_PATH = REPO_ROOT / "oya" / "messenger" / "manifest.json"
BROWNOUT_SPEC_PATH = REPO_ROOT / "specs" / "brownout-degradation-signal.json"

REQUIRED_ACCEPTED_ADRS = {
    "ADR-0165",
    "ADR-0168",
    "ADR-0176",
    "ADR-0180",
    "ADR-0186",
    "ADR-0210",
    "ADR-0241",
}
REQUIRED_PROPOSED_CONTEXT_ADRS = {"ADR-0263", "ADR-0306"}
REQUIRED_CHAOS_SCENARIOS = {
    "pod-kill",
    "network-delay-100ms",
    "dependency-failure",
    "disk-slow-1000ms",
}
REQUIRED_TAIL_POLICIES = {
    "status_code=ERROR",
    "latency_p99",
    "new_endpoint_warmup",
    "slo_burn",
    "audit_event",
    "random_baseline=0.01",
}
REQUIRED_STATUS_ENUMS = {
    "operational",
    "degraded_performance",
    "partial_outage",
    "major_outage",
    "under_maintenance",
}
REQUIRED_DISASTER_HEADERS = {
    "X-Oya-Disaster-Mode-Active",
    "X-Oya-DR-Pair-Cell",
    "X-Oya-Load-Shed-Tier",
}
REQUIRED_NONCLAIMS = {
    "no_live_chaos_execution",
    "no_runtime_control_loop",
    "no_measured_slo_claim",
    "no_public_sla_slo_claim",
    "no_status_page_live_claim",
    "no_disaster_mode_runtime_claim",
    "no_production_readiness_claim",
    "no_tenant_traffic_claim",
}


class ValidationFailure(Exception):
    """Raised for a RESILIENCE-001 invariant failure."""


def fail(message: str) -> NoReturn:
    raise ValidationFailure(message)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def load_json(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        fail(f"missing {path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path.relative_to(REPO_ROOT)}: {exc}")


def rel(path_text: str) -> Path:
    return REPO_ROOT / path_text


def relative(path: Path) -> str:
    return str(path.relative_to(REPO_ROOT))


def brownout_defaults(spec: dict[str, Any]) -> tuple[str, str, set[str], str]:
    props = spec.get("properties", {})
    header = props.get("header_name", {}).get("const")
    metric = props.get("gauge", {}).get("properties", {}).get("metric_name", {}).get("const")
    class_defaults = props.get("classes", {}).get("default", [])
    classes = {row.get("class") for row in class_defaults}
    audit_class = props.get("audit_chain_class", {}).get("const")
    return header, metric, classes, audit_class


def validate_contract(
    contract: dict[str, Any],
    manifest: dict[str, Any],
    brownout_spec: dict[str, Any],
    *,
    check_files: bool = True,
) -> None:
    require(contract.get("contract_id") == "RESILIENCE-001-MESSENGER-RUNTIME-CONTROL-LOOP", "unexpected contract_id")
    require(contract.get("source_task") == "t_c127bb35", "contract must bind to Kanban task t_c127bb35")
    require(contract.get("status") == "Evidence-contract-only", "status must remain Evidence-contract-only")

    service = contract.get("service", {})
    require(service.get("name") == "messenger", "contract service must be messenger")
    require(service.get("path") == "oya/messenger", "contract service.path must be oya/messenger")
    require(manifest.get("microservice") == "messenger", "manifest microservice must be messenger")

    authority = contract.get("authority", {})
    require(REQUIRED_ACCEPTED_ADRS <= set(authority.get("accepted_adrs", [])), "missing accepted ADR anchors")
    require(REQUIRED_PROPOSED_CONTEXT_ADRS <= set(authority.get("proposed_context_adrs", [])), "missing Proposed ADR context anchors")
    require(authority.get("proposed_context_only") is True, "Proposed ADRs must remain context-only")

    controls = contract.get("claim_controls", {})
    for key in [
        "metadata_only",
        "evidence_contract_only",
        "strict_separation",
        "no_live_chaos_execution",
        "no_runtime_control_loop",
        "no_measured_slo_claim",
        "no_public_sla_slo_claim",
        "no_status_page_live_claim",
        "no_disaster_mode_runtime_claim",
        "no_production_readiness_claim",
        "no_tenant_traffic_claim",
        "proposed_adrs_not_elevated",
    ]:
        require(controls.get(key) is True, f"claim_controls.{key} must be true")
    require(REQUIRED_NONCLAIMS <= set(contract.get("nonclaims", [])), "nonclaims are incomplete")

    loop = contract.get("runtime_control_loop", {})
    for key in [
        "chaos_catalog",
        "brownout_signal",
        "slo_composition",
        "tail_sampling",
        "status_page_projection",
        "disaster_mode_evidence",
    ]:
        require(key in loop, f"runtime_control_loop missing {key}")

    chaos = loop["chaos_catalog"]
    require(chaos.get("catalog_status") == "scenario_catalog_only", "chaos catalog must be scenario_catalog_only")
    require(chaos.get("engine") == "Chaos Mesh 2.x", "chaos catalog must pin Chaos Mesh 2.x")
    scenario_ids = {row.get("id") for row in chaos.get("scenarios", [])}
    require(REQUIRED_CHAOS_SCENARIOS <= scenario_ids, "chaos catalog missing required scenarios")
    scenario_refs = [row.get("path") for row in chaos.get("scenarios", [])]
    require(all(isinstance(path, str) for path in scenario_refs), "chaos scenario paths must be strings")
    if check_files:
        for path_text in scenario_refs:
            path = rel(path_text)
            require(path.exists(), f"missing chaos scenario file {path_text}")
            text = path.read_text(encoding="utf-8")
            require("kind: Workflow" in text, f"{path_text} must declare a Chaos Mesh Workflow")
            require("app.kubernetes.io/name: messenger" in text, f"{path_text} must target messenger")
            require("scenario-catalog-only" in text, f"{path_text} must carry scenario-catalog-only claim status")
    require(
        {"message-send-availability", "message-send-latency"} <= set(chaos.get("slo_gate_refs", [])),
        "chaos catalog must gate on messenger availability and latency SLOs",
    )

    expected_header, expected_metric, expected_classes, expected_audit_class = brownout_defaults(brownout_spec)
    brownout = loop["brownout_signal"]
    require(brownout.get("header_name") == expected_header, "brownout header must match specs/brownout-degradation-signal.json")
    require(brownout.get("metric_name") == expected_metric, "brownout metric must match specs/brownout-degradation-signal.json")
    require(set(brownout.get("classes", [])) == expected_classes, "brownout classes must match the canonical spec")
    require(brownout.get("audit_chain_class") == expected_audit_class, "brownout audit class must match the canonical spec")

    composition = loop["slo_composition"]
    composition_path = composition.get("composition_ref")
    require(composition_path == "oya/messenger/slos/composition.openslo.yaml", "unexpected SLO composition ref")
    require(composition.get("composition_kind") == "critical_path", "messenger composition must use critical_path")
    require("message-send-availability" in composition.get("child_slo_refs", []), "composition must include message-send availability")
    require("message-send-latency" in composition.get("child_slo_refs", []), "composition must include message-send latency")
    if check_files:
        path = rel(composition_path)
        require(path.exists(), f"missing {composition_path}")
        text = path.read_text(encoding="utf-8")
        require("kind: SLOComposition" in text, "composition file must use SLOComposition")
        require("composition_kind: critical_path" in text, "composition file must declare critical_path")
        require("message-send-availability" in text and "message-send-latency" in text, "composition file must cite messenger SLO children")
        require("evidence_contract_only" in text, "composition file must stay claim-bounded")

    tail = loop["tail_sampling"]
    require(tail.get("manifest_field_ref") == "oya/messenger/manifest.json#/observability_trace_sampling_recipe", "tail sampling must bind to manifest field")
    require(tail.get("head_bps") == 100, "tail sampling head_bps must be the 1% baseline")
    require(REQUIRED_TAIL_POLICIES <= set(tail.get("tail_policies", [])), "tail sampling policies incomplete")
    require(tail.get("decision_wait_seconds") == 30, "tail sampling decision_wait_seconds must be 30")
    manifest_recipe = manifest.get("observability_trace_sampling_recipe", {})
    require(manifest_recipe.get("head_bps") == tail.get("head_bps"), "manifest head_bps must match contract")
    require(REQUIRED_TAIL_POLICIES <= set(manifest_recipe.get("tail_policies", [])), "manifest tail policies incomplete")
    require(manifest_recipe.get("p99_latency_threshold_ms") == 100, "manifest p99 threshold must bind messenger latency SLO")

    projection = loop["status_page_projection"]
    require(projection.get("projection_status") == "projection_contract_only", "status page projection must be contract-only")
    require(projection.get("component") == "messenger", "status page component must be messenger")
    require(REQUIRED_STATUS_ENUMS <= set(projection.get("status_enum_map", {})), "status enum map incomplete")
    require("Statuspage.io-compatible" in projection.get("api_shape", ""), "status page projection must preserve Statuspage.io-compatible shape")

    disaster = loop["disaster_mode_evidence"]
    require(disaster.get("source_adr") == "ADR-0306", "disaster mode evidence must cite ADR-0306")
    require(disaster.get("source_adr_status") == "Proposed/context-only", "ADR-0306 must remain Proposed/context-only")
    require(REQUIRED_DISASTER_HEADERS <= set(disaster.get("headers", [])), "disaster-mode headers incomplete")
    require(disaster.get("emergency_services_never_throttle") is True, "emergency-services non-throttle invariant required")

    evidence = set(contract.get("evidence_required_before_runtime_claim", []))
    require(
        {
            "chaos_drill_receipt",
            "measured_slo_window",
            "brownout_state_transition_receipt",
            "tail_sampling_fidelity_receipt",
            "status_page_projection_receipt",
            "dr_or_disaster_mode_drill_receipt",
        } <= evidence,
        "runtime evidence-before-claim set incomplete",
    )


def validate() -> None:
    contract = load_json(CONTRACT_PATH)
    manifest = load_json(MANIFEST_PATH)
    brownout_spec = load_json(BROWNOUT_SPEC_PATH)
    validate_contract(contract, manifest, brownout_spec)


def expect_failure(
    contract: dict[str, Any],
    manifest: dict[str, Any],
    brownout_spec: dict[str, Any],
    mutator: Callable[[dict[str, Any]], None],
    expected_fragment: str,
) -> None:
    mutated = copy.deepcopy(contract)
    mutator(mutated)
    try:
        validate_contract(mutated, manifest, brownout_spec, check_files=False)
    except ValidationFailure as exc:
        require(expected_fragment in str(exc), f"self-test expected {expected_fragment!r}, got {exc}")
        return
    fail(f"self-test mutation did not fail: {expected_fragment}")


def self_test() -> None:
    contract = load_json(CONTRACT_PATH)
    manifest = load_json(MANIFEST_PATH)
    brownout_spec = load_json(BROWNOUT_SPEC_PATH)
    validate_contract(contract, manifest, brownout_spec)

    expect_failure(
        contract,
        manifest,
        brownout_spec,
        lambda c: c["authority"].__setitem__("proposed_context_only", False),
        "Proposed ADRs must remain context-only",
    )
    expect_failure(
        contract,
        manifest,
        brownout_spec,
        lambda c: c["runtime_control_loop"]["chaos_catalog"].__setitem__("scenarios", []),
        "chaos catalog missing required scenarios",
    )
    expect_failure(
        contract,
        manifest,
        brownout_spec,
        lambda c: c["runtime_control_loop"]["brownout_signal"].__setitem__("classes", ["nominal"]),
        "brownout classes must match",
    )
    expect_failure(
        contract,
        manifest,
        brownout_spec,
        lambda c: c["runtime_control_loop"]["tail_sampling"].__setitem__("tail_policies", ["random_baseline=0.01"]),
        "tail sampling policies incomplete",
    )
    expect_failure(
        contract,
        manifest,
        brownout_spec,
        lambda c: c["runtime_control_loop"]["status_page_projection"].__setitem__("projection_status", "live"),
        "status page projection must be contract-only",
    )


def main() -> int:
    try:
        if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
            self_test()
            print("RESILIENCE-001 runtime-control-loop self-test passed")
            return 0
        validate()
    except ValidationFailure as exc:
        print(f"RESILIENCE-001 runtime-control-loop check failed: {exc}", file=sys.stderr)
        return 1
    print("RESILIENCE-001 runtime-control-loop check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
