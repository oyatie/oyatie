#!/usr/bin/env python3
"""Validate the RESILIENCE-001 messenger runtime-control-loop evidence contract."""
from __future__ import annotations

import copy
import json
import sys
from collections.abc import Callable
from pathlib import Path
from typing import Any, NoReturn

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


def parse_scalar(value: str) -> Any:
    if value in {"true", "false"}:
        return value == "true"
    if value in {"null", "~"}:
        return None
    if value.startswith(('"', "'")) and value.endswith(('"', "'")):
        return value[1:-1]
    try:
        return int(value)
    except ValueError:
        pass
    try:
        return float(value)
    except ValueError:
        return value


def yaml_entries(path: Path) -> list[tuple[int, str]]:
    entries: list[tuple[int, str]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        entries.append((len(line) - len(line.lstrip(" ")), line.strip()))
    return entries


def parse_yaml_block(entries: list[tuple[int, str]], index: int, indent: int) -> tuple[Any, int]:
    if index >= len(entries) or entries[index][0] < indent:
        return {}, index

    if entries[index][0] == indent and entries[index][1].startswith("- "):
        values: list[Any] = []
        while index < len(entries):
            item_indent, item_text = entries[index]
            if item_indent < indent or item_indent != indent or not item_text.startswith("- "):
                break

            payload = item_text[2:].strip()
            index += 1
            if payload and ":" in payload:
                key, raw_value = payload.split(":", 1)
                item: dict[str, Any] = {}
                raw_value = raw_value.strip()
                if raw_value:
                    item[key.strip()] = parse_scalar(raw_value)
                elif index < len(entries) and entries[index][0] > item_indent:
                    child, index = parse_yaml_block(entries, index, entries[index][0])
                    item[key.strip()] = child
                while index < len(entries) and entries[index][0] > item_indent:
                    child, index = parse_yaml_block(entries, index, entries[index][0])
                    if isinstance(child, dict):
                        item.update(child)
                values.append(item)
            elif payload:
                values.append(parse_scalar(payload))
            elif index < len(entries) and entries[index][0] > item_indent:
                child, index = parse_yaml_block(entries, index, entries[index][0])
                values.append(child)
        return values, index

    mapping: dict[str, Any] = {}
    while index < len(entries):
        item_indent, item_text = entries[index]
        if item_indent < indent:
            break
        if item_indent != indent:
            index += 1
            continue
        if ":" not in item_text:
            fail(f"invalid YAML-like mapping line: {item_text}")
        key, raw_value = item_text.split(":", 1)
        raw_value = raw_value.strip()
        index += 1
        if raw_value:
            mapping[key.strip()] = parse_scalar(raw_value)
        elif index < len(entries) and entries[index][0] > item_indent:
            child, index = parse_yaml_block(entries, index, entries[index][0])
            mapping[key.strip()] = child
        else:
            mapping[key.strip()] = None
    return mapping, index


def load_yaml_object(path: Path) -> dict[str, Any]:
    try:
        entries = yaml_entries(path)
    except FileNotFoundError:
        fail(f"missing {path.relative_to(REPO_ROOT)}")
    parsed, _ = parse_yaml_block(entries, 0, 0)
    if not isinstance(parsed, dict):
        fail(f"invalid YAML object in {path.relative_to(REPO_ROOT)}")
    return parsed


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


def validate_identity(contract: dict[str, Any], manifest: dict[str, Any]) -> None:
    require(contract.get("contract_id") == "RESILIENCE-001-MESSENGER-RUNTIME-CONTROL-LOOP", "unexpected contract_id")
    require(contract.get("source_task") == "t_c127bb35", "contract must bind to Kanban task t_c127bb35")
    require(contract.get("status") == "Evidence-contract-only", "status must remain Evidence-contract-only")

    service = contract.get("service", {})
    require(service.get("name") == "messenger", "contract service must be messenger")
    require(service.get("path") == "oya/messenger", "contract service.path must be oya/messenger")
    require(manifest.get("microservice") == "messenger", "manifest microservice must be messenger")


def validate_authority(contract: dict[str, Any]) -> None:
    authority = contract.get("authority", {})
    accepted_adrs = set(authority.get("accepted_adrs", []))
    proposed_context_adrs = set(authority.get("proposed_context_adrs", []))
    require(accepted_adrs >= REQUIRED_ACCEPTED_ADRS, "missing accepted ADR anchors")
    require(proposed_context_adrs >= REQUIRED_PROPOSED_CONTEXT_ADRS, "missing Proposed ADR context anchors")
    require(authority.get("proposed_context_only") is True, "Proposed ADRs must remain context-only")


def validate_claim_controls(contract: dict[str, Any]) -> None:
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
    require(set(contract.get("nonclaims", [])) >= REQUIRED_NONCLAIMS, "nonclaims are incomplete")


def validate_loop_sections(loop: dict[str, Any]) -> None:
    for key in [
        "chaos_catalog",
        "brownout_signal",
        "slo_composition",
        "tail_sampling",
        "status_page_projection",
        "disaster_mode_evidence",
    ]:
        require(key in loop, f"runtime_control_loop missing {key}")


def validate_chaos_catalog(chaos: dict[str, Any], *, check_files: bool = True) -> None:
    require(chaos.get("catalog_status") == "scenario_catalog_only", "chaos catalog must be scenario_catalog_only")
    require(chaos.get("engine") == "Chaos Mesh 2.x", "chaos catalog must pin Chaos Mesh 2.x")
    scenario_ids = {row.get("id") for row in chaos.get("scenarios", [])}
    require(scenario_ids >= REQUIRED_CHAOS_SCENARIOS, "chaos catalog missing required scenarios")
    scenario_refs = [row.get("path") for row in chaos.get("scenarios", [])]
    require(all(isinstance(path, str) for path in scenario_refs), "chaos scenario paths must be strings")
    if check_files:
        for path_text in scenario_refs:
            path = rel(path_text)
            require(path.exists(), f"missing chaos scenario file {path_text}")
            document = load_yaml_object(path)
            labels = document.get("metadata", {}).get("labels", {})
            annotations = document.get("metadata", {}).get("annotations", {})
            require(document.get("kind") == "Workflow", f"{path_text} must declare a Chaos Mesh Workflow")
            require(labels.get("app.kubernetes.io/name") == "messenger", f"{path_text} must target messenger")
            require(labels.get("oya.dev/claim-status") == "scenario-catalog-only", f"{path_text} must carry scenario-catalog-only claim status")
            require(annotations.get("oya.dev/nonclaim") == "no-live-chaos-execution", f"{path_text} must carry a no-live-chaos-execution nonclaim")
    require(
        set(chaos.get("slo_gate_refs", [])) >= {"message-send-availability", "message-send-latency"},
        "chaos catalog must gate on messenger availability and latency SLOs",
    )


def validate_brownout(loop: dict[str, Any], brownout_spec: dict[str, Any]) -> None:
    expected_header, expected_metric, expected_classes, expected_audit_class = brownout_defaults(brownout_spec)
    brownout = loop["brownout_signal"]
    require(brownout.get("header_name") == expected_header, "brownout header must match specs/brownout-degradation-signal.json")
    require(brownout.get("metric_name") == expected_metric, "brownout metric must match specs/brownout-degradation-signal.json")
    require(set(brownout.get("classes", [])) == expected_classes, "brownout classes must match the canonical spec")
    require(brownout.get("audit_chain_class") == expected_audit_class, "brownout audit class must match the canonical spec")


def validate_slo_composition(composition: dict[str, Any], *, check_files: bool = True) -> None:
    composition_path = composition.get("composition_ref")
    require(composition_path == "oya/messenger/slos/composition.openslo.yaml", "unexpected SLO composition ref")
    require(composition.get("composition_kind") == "critical_path", "messenger composition must use critical_path")
    require("message-send-availability" in composition.get("child_slo_refs", []), "composition must include message-send availability")
    require("message-send-latency" in composition.get("child_slo_refs", []), "composition must include message-send latency")
    if check_files:
        path = rel(composition_path)
        require(path.exists(), f"missing {composition_path}")
        document = load_yaml_object(path)
        metadata = document.get("metadata", {})
        labels = metadata.get("labels", {})
        annotations = metadata.get("annotations", {})
        spec = document.get("spec", {})
        child_refs = {row.get("slo_ref") for row in spec.get("children", [])}
        require(document.get("kind") == "SLOComposition", "composition file must use SLOComposition")
        require(spec.get("composition_kind") == "critical_path", "composition file must declare critical_path")
        require(
            any(str(ref).endswith("message-send-availability.openslo.yaml") for ref in child_refs)
            and any(str(ref).endswith("message-send-latency.openslo.yaml") for ref in child_refs),
            "composition file must cite messenger SLO children",
        )
        require(labels.get("claim_status") == "evidence_contract_only", "composition file must stay claim-bounded")
        require("no measured SLO" in annotations.get("oya.dev/nonclaim", ""), "composition file must declare measured-SLO nonclaim")


def validate_tail_sampling(tail: dict[str, Any], manifest: dict[str, Any]) -> None:
    require(tail.get("manifest_field_ref") == "oya/messenger/manifest.json#/observability_trace_sampling_recipe", "tail sampling must bind to manifest field")
    require(tail.get("head_bps") == 100, "tail sampling head_bps must be the 1% baseline")
    require(set(tail.get("tail_policies", [])) >= REQUIRED_TAIL_POLICIES, "tail sampling policies incomplete")
    require(tail.get("decision_wait_seconds") == 30, "tail sampling decision_wait_seconds must be 30")
    manifest_recipe = manifest.get("observability_trace_sampling_recipe", {})
    require(manifest_recipe.get("head_bps") == tail.get("head_bps"), "manifest head_bps must match contract")
    require(set(manifest_recipe.get("tail_policies", [])) >= REQUIRED_TAIL_POLICIES, "manifest tail policies incomplete")
    require(manifest_recipe.get("p99_latency_threshold_ms") == 100, "manifest p99 threshold must bind messenger latency SLO")


def validate_status_page_projection(projection: dict[str, Any]) -> None:
    require(projection.get("projection_status") == "projection_contract_only", "status page projection must be contract-only")
    require(projection.get("component") == "messenger", "status page component must be messenger")
    require(set(projection.get("status_enum_map", {})) >= REQUIRED_STATUS_ENUMS, "status enum map incomplete")
    require("Statuspage.io-compatible" in projection.get("api_shape", ""), "status page projection must preserve Statuspage.io-compatible shape")


def validate_disaster_mode_evidence(disaster: dict[str, Any]) -> None:
    require(disaster.get("source_adr") == "ADR-0306", "disaster mode evidence must cite ADR-0306")
    require(disaster.get("source_adr_status") == "Proposed/context-only", "ADR-0306 must remain Proposed/context-only")
    require(set(disaster.get("headers", [])) >= REQUIRED_DISASTER_HEADERS, "disaster-mode headers incomplete")
    require(disaster.get("emergency_services_never_throttle") is True, "emergency-services non-throttle invariant required")


def validate_runtime_evidence(contract: dict[str, Any]) -> None:
    evidence = set(contract.get("evidence_required_before_runtime_claim", []))
    require(
        evidence
        >= {
            "chaos_drill_receipt",
            "measured_slo_window",
            "brownout_state_transition_receipt",
            "tail_sampling_fidelity_receipt",
            "status_page_projection_receipt",
            "dr_or_disaster_mode_drill_receipt",
        },
        "runtime evidence-before-claim set incomplete",
    )


def validate_contract(
    contract: dict[str, Any],
    manifest: dict[str, Any],
    brownout_spec: dict[str, Any],
    *,
    check_files: bool = True,
) -> None:
    loop = contract.get("runtime_control_loop", {})
    validate_identity(contract, manifest)
    validate_authority(contract)
    validate_claim_controls(contract)
    validate_loop_sections(loop)
    validate_chaos_catalog(loop["chaos_catalog"], check_files=check_files)
    validate_brownout(loop, brownout_spec)
    validate_slo_composition(loop["slo_composition"], check_files=check_files)
    validate_tail_sampling(loop["tail_sampling"], manifest)
    validate_status_page_projection(loop["status_page_projection"])
    validate_disaster_mode_evidence(loop["disaster_mode_evidence"])
    validate_runtime_evidence(contract)


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
        lambda c: c["authority"].update({"proposed_context_only": False}),
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
    expect_failure(
        contract,
        manifest,
        brownout_spec,
        lambda c: c["authority"].__setitem__("accepted_adrs", []),
        "missing accepted ADR anchors",
    )
    expect_failure(
        contract,
        manifest,
        brownout_spec,
        lambda c: c.__setitem__("nonclaims", []),
        "nonclaims are incomplete",
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
