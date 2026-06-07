#!/usr/bin/env python3
"""Validate the Oyatie Cloud control-plane operation contract companion spec."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "specs" / "cloud-control-plane-operation-contract.json"
CONTROL_PLANE_PATH = REPO_ROOT / "specs" / "cloud-control-plane-canonical.json"
RESOURCE_CATALOG_PATH = REPO_ROOT / "specs" / "cloud-resource-contract-parity-catalog.json"

REQUIRED_STAGES = ["api_gateway", "resource_registry", "operation_ledger", "workflow_reconciler", "backend_actuation_boundary"]
REQUIRED_OPERATION_FIELDS = {
    "operation_id",
    "idempotency_key",
    "request_hash",
    "resource_orn",
    "desired_generation",
    "observed_generation",
    "state",
    "phase",
    "tenant_account_project",
    "region_cell",
    "principal",
    "audit_chain_id",
    "retry_policy",
    "cancellation",
    "compensation",
}
REQUIRED_OPERATION_STATES = {
    "accepted",
    "validating",
    "queued",
    "running",
    "waiting_for_reconciler",
    "succeeded",
    "failed",
    "cancel_requested",
    "cancelled",
    "compensating",
    "rolled_back",
}
REQUIRED_SEMANTICS = {"idempotent_retry", "resumable_after_restart", "cancel_safe", "compensating_action", "no_partial_apply_without_ledger"}
REQUIRED_RESOURCE_TRANSITIONS = {"create", "update", "delete", "suspend", "resume", "purge"}
FORBIDDEN_ACTUATION_MARKERS = {
    "terraform apply",
    "opentofu apply",
    "kubectl apply",
    "aws cli",
    "gcloud cli",
    "az cli",
    "oci cli",
    "aws provider api",
    "provider api",
    "provider apis",
    "live provider",
    "external providers execute",
    "external provider execution",
    "provider apis",
}
FORBIDDEN_CLAIM_MARKERS = {
    "feature parity",
    "feature-parity",
    "production ready",
    "production readiness",
    "tenant workloads can run",
    "tenant workload ready",
    "runtime reconciler availability",
    "runtime reconciler availability is achieved",
    "external providers execute through provider apis",
    "public sla",
    "public slo",
    "sla backed",
    "sla-backed",
    "99 9 availability",
    "99.9 availability",
    "hyperscaler mature",
    "hyperscaler maturity",
}


def fail(message: str) -> NoReturn:
    print(f"cloud control-plane operation contract check failed: {message}", file=sys.stderr)
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


def contains_any(value: object, markers: set[str]) -> bool:
    haystack = f" {normalized(value)} "
    return any(f" {normalized(marker)} " in haystack for marker in markers)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path.relative_to(REPO_ROOT)}: {exc}")


def validate(spec: dict) -> None:
    control_plane = load_json(CONTROL_PLANE_PATH)
    resource_catalog = load_json(RESOURCE_CATALOG_PATH)

    for field in [
        "spec_id",
        "title",
        "status",
        "source_control_plane",
        "source_resource_catalog",
        "claim_controls",
        "pipeline",
        "resource_registry_entry",
        "operation_ledger_entry",
        "operation_state_machine",
        "idempotency_retry_cancel_contract",
        "resource_state_transition_contract",
        "nonclaims",
        "next_goal_links",
    ]:
        require(field in spec, f"missing top-level field {field!r}")

    require(spec["status"] == "Proposed-target", "status must remain Proposed-target")
    require(spec["source_control_plane"] == str(CONTROL_PLANE_PATH.relative_to(REPO_ROOT)), "source_control_plane must point to canonical control-plane spec")
    require(spec["source_resource_catalog"] == str(RESOURCE_CATALOG_PATH.relative_to(REPO_ROOT)), "source_resource_catalog must point to G002 resource catalog")

    controls = spec["claim_controls"]
    require(controls.get("metadata_only") is True, "claim_controls must be metadata_only")
    require(controls.get("no_provider_apply") is True, "claim_controls must forbid provider apply")
    require(controls.get("no_runtime_reconciler_claim") is True, "claim_controls must forbid runtime reconciler claim")
    require(not contains_any(controls.get("can_claim_now", []), FORBIDDEN_CLAIM_MARKERS), "can_claim_now contains forbidden readiness/parity wording")
    require(not contains_any(spec, FORBIDDEN_ACTUATION_MARKERS), "spec contains forbidden provider apply/actuation wording")

    pipeline = spec["pipeline"]
    require([stage.get("id") for stage in pipeline.get("stages", [])] == REQUIRED_STAGES, "pipeline stages/order must match required control-plane sequence")
    require(not contains_any(pipeline, FORBIDDEN_ACTUATION_MARKERS), "pipeline contains forbidden live actuation marker")

    registry = spec["resource_registry_entry"]
    for field in ["resource_orn", "resource_type", "desired_spec", "desired_generation", "observed_generation", "tenant_account_project", "region_cell", "owner", "deletion_retention", "slo_tier", "policy_snapshot", "quota_snapshot", "billing_meter_bindings", "audit_event_bindings"]:
        require(field in registry.get("required_fields", []), f"resource registry missing required field {field}")
    require(bool(registry.get("generation_rules")), "resource registry must define generation_rules")
    require(bool(registry.get("identity_rules")), "resource registry must define identity_rules")

    ledger = spec["operation_ledger_entry"]
    require(REQUIRED_OPERATION_FIELDS <= set(ledger.get("required_fields", [])), f"operation ledger missing fields {sorted(REQUIRED_OPERATION_FIELDS - set(ledger.get('required_fields', [])))}")
    require(bool(ledger.get("identity")), "operation ledger must define identity semantics")
    require(ledger.get("durability", {}).get("write_before_ack") is True, "operation ledger must require write_before_ack")
    require(ledger.get("durability", {}).get("audit_chain_required") is True, "operation ledger must require audit_chain_required")
    require(REQUIRED_OPERATION_STATES <= set(spec["operation_state_machine"].get("states", [])), "operation state machine missing states")
    transitions = [tuple(item) for item in spec["operation_state_machine"].get("allowed_transitions", [])]
    require(bool(transitions), "operation state machine must define allowed_transitions")
    require(bool(spec["operation_state_machine"].get("transition_rules")), "operation state machine must define transition_rules")
    terminal_states = set(spec["operation_state_machine"].get("terminal_states", []))
    require(bool(terminal_states), "operation state machine must define terminal_states")
    outgoing_from_terminal = sorted({source for source, _target in transitions if source in terminal_states})
    require(not outgoing_from_terminal, f"terminal states must not have outgoing transitions: {outgoing_from_terminal}")
    reachable = {"accepted"}
    changed = True
    while changed:
        changed = False
        for source, target in transitions:
            if source in reachable and target not in reachable:
                reachable.add(target)
                changed = True
    missing_reachable = sorted(REQUIRED_OPERATION_STATES - reachable)
    require(not missing_reachable, f"operation states must be reachable from accepted: {missing_reachable}")
    require(("running", "compensating") in transitions, "running operations must have a compensation transition before terminal failure")
    require(("compensating", "rolled_back") in transitions, "compensating operations must be able to roll back")
    require(REQUIRED_SEMANTICS <= set(spec["idempotency_retry_cancel_contract"].get("required_semantics", [])), "missing idempotency/retry/cancel semantics")
    for block in ["idempotency", "retry_policy", "cancellation", "compensation"]:
        require(bool(spec["idempotency_retry_cancel_contract"].get(block)), f"idempotency/retry/cancel contract missing {block} block")
    require(not contains_any(spec["idempotency_retry_cancel_contract"], FORBIDDEN_ACTUATION_MARKERS), "operation semantics contain forbidden actuation marker")
    cancellation = spec["idempotency_retry_cancel_contract"].get("cancellation", {})
    cancellable_states = set(cancellation.get("accepted_states", []))
    missing_cancel_edges = sorted(state for state in cancellable_states if (state, "cancel_requested") not in transitions)
    require(not missing_cancel_edges, f"cancellable states missing cancel_requested transition: {missing_cancel_edges}")
    cancel_terminal_results = set(cancellation.get("terminal_result", []))
    missing_cancel_terminal_edges = sorted(state for state in cancel_terminal_results if ("cancel_requested", state) not in transitions)
    require(not missing_cancel_terminal_edges, f"cancel terminal results missing transitions from cancel_requested: {missing_cancel_terminal_edges}")

    transition_contract = spec["resource_state_transition_contract"]
    require(REQUIRED_RESOURCE_TRANSITIONS <= set(transition_contract.get("verbs", [])), "resource transition contract missing verbs")
    coverage_contract_ids = {contract["id"] for contract in resource_catalog["resource_contracts"]}
    covered_ids = set(transition_contract.get("applies_to_resource_contract_ids", []))
    require(coverage_contract_ids <= covered_ids, "resource transition contract must apply to every G002 resource contract")
    require(not contains_any(transition_contract, FORBIDDEN_ACTUATION_MARKERS), "resource transition contract contains forbidden actuation marker")

    nonclaim_ids = {item.get("id") for item in spec["nonclaims"]}
    require({"no_provider_apply", "no_runtime_reconciler", "no_production_readiness", "no_tenant_workload_readiness"} <= nonclaim_ids, "missing required nonclaims")
    for item in spec["nonclaims"]:
        require(not contains_any(item.get("statement", ""), FORBIDDEN_CLAIM_MARKERS), f"nonclaim {item.get('id')}: statement contains forbidden positive-claim wording")

    require(spec["next_goal_links"].get("authz_tenancy_audit_metering_billing") == "G004", "G004 link required")
    require(spec["next_goal_links"].get("observability_slo_evidence") == "G005", "G005 link required")
    require(spec["next_goal_links"].get("production_quality_kits") == "G006", "G006 link required")
    require(control_plane.get("spec_id") == "EXE-CLOUD-CONTROL-PLANE-CANONICAL", "unexpected control-plane source")


def main() -> None:
    validate(load_json(SPEC_PATH))
    print(f"cloud control-plane operation contract check passed: {SPEC_PATH.relative_to(REPO_ROOT)}")


def run_self_tests() -> None:
    baseline = load_json(SPEC_PATH)

    def expect_rejected(label: str, mutator: Callable[[dict], None]) -> None:
        original = SPEC_PATH.read_text(encoding="utf-8")
        candidate = json.loads(json.dumps(baseline))
        mutator(candidate)
        try:
            SPEC_PATH.write_text(json.dumps(candidate, indent=2) + "\n", encoding="utf-8")
            try:
                validate(load_json(SPEC_PATH))
            except SystemExit as exc:
                require(exc.code != 0, f"self-test {label!r} exited successfully")
            else:
                fail(f"self-test mutation was accepted: {label}")
        finally:
            SPEC_PATH.write_text(original, encoding="utf-8")

    expect_rejected("missing operation field", lambda data: data["operation_ledger_entry"].update({"required_fields": ["operation_id"]}))
    expect_rejected("missing cancel state", lambda data: data["operation_state_machine"].update({"states": ["accepted", "succeeded"]}))
    expect_rejected("terminal state outgoing edge", lambda data: data["operation_state_machine"]["allowed_transitions"].append(["succeeded", "running"]))
    expect_rejected("cancellation state without edge", lambda data: data["idempotency_retry_cancel_contract"]["cancellation"].update({"accepted_states": ["waiting_for_reconciler", "succeeded"]}))
    expect_rejected("provider apply marker", lambda data: data["pipeline"]["stages"][-1].update({"role": "run opentofu apply"}))
    expect_rejected("missing resource contract coverage", lambda data: data["resource_state_transition_contract"].update({"applies_to_resource_contract_ids": []}))
    expect_rejected("production overclaim", lambda data: data["claim_controls"].update({"can_claim_now": ["production readiness achieved"]}))
    expect_rejected("runtime reconciler overclaim", lambda data: data["claim_controls"].update({"can_claim_now": ["runtime reconciler availability is achieved"]}))
    expect_rejected("external provider API overclaim", lambda data: data["claim_controls"].update({"can_claim_now": ["external providers execute through provider APIs"]}))
    expect_rejected("tenant workload overclaim", lambda data: data["claim_controls"].update({"can_claim_now": ["tenant workload ready"]}))
    expect_rejected("SLA availability overclaim", lambda data: data["claim_controls"].update({"can_claim_now": ["SLA-backed 99.9 availability"]}))
    expect_rejected("top-level actuation marker", lambda data: data.update({"purpose": "run kubectl apply"}))
    expect_rejected("nonclaim actuation marker", lambda data: data["nonclaims"][0].update({"statement": "run kubectl apply"}))
    expect_rejected("empty state machine details", lambda data: data["operation_state_machine"].update({"terminal_states": [], "allowed_transitions": [], "transition_rules": []}))
    expect_rejected("missing compensation transitions", lambda data: data["operation_state_machine"].update({"allowed_transitions": [edge for edge in data["operation_state_machine"]["allowed_transitions"] if "compensating" not in edge]}))
    expect_rejected("missing registry rules", lambda data: data["resource_registry_entry"].update({"generation_rules": [], "identity_rules": []}))
    expect_rejected("missing ledger details", lambda data: data["operation_ledger_entry"].update({"identity": {}, "durability": {}}))
    expect_rejected("missing detailed idempotency blocks", lambda data: data["idempotency_retry_cancel_contract"].update({"idempotency": {}, "retry_policy": {}, "cancellation": {}, "compensation": {}}))
    print("cloud control-plane operation contract self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
