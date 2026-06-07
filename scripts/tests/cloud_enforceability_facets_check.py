#!/usr/bin/env python3
"""Validate authz/tenancy/audit/metering/billing enforceability facets for Oyatie Cloud contracts."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "specs" / "cloud-enforceability-facets.json"
RESOURCE_CATALOG_PATH = REPO_ROOT / "specs" / "cloud-resource-contract-parity-catalog.json"
OPERATION_CONTRACT_PATH = REPO_ROOT / "specs" / "cloud-control-plane-operation-contract.json"

REQUIRED_FACETS = {"cedar_policy", "tenant_scope", "audit", "quota_cost", "metering", "billing"}
REQUIRED_CEDAR_FIELDS = {"principal", "action", "resource_orn", "tenant_account_project", "region_cell", "policy_snapshot", "decision", "reason_code"}
REQUIRED_AUDIT_FIELDS = {"audit_event_type", "audit_chain_id", "operation_id", "resource_orn", "principal", "tenant_account_project", "region_cell", "previous_state", "next_state", "reason_code"}
REQUIRED_BILLING_FIELDS = {"meter_name", "unit", "aggregation", "billing_account", "cost_center", "currency", "rated_usage_ref"}
REQUIRED_NONCLAIMS = {"no_runtime_policy_engine", "no_billing_runtime", "no_audit_runtime", "no_tenant_workload_readiness"}
FORBIDDEN_MARKERS = {
    "production ready",
    "production readiness",
    "feature parity",
    "aws parity",
    "aws parity exists",
    "parity exists",
    "tenant workload ready",
    "tenant workloads can run",
    "tenant workloads can safely run",
    "public sla",
    "public slo",
    "sla backed",
    "sla-backed",
    "sla backed service",
    "live billing runtime",
    "runtime policy engine is available",
    "cedar runtime engine available",
    "audit runtime is available",
    "provider api",
    "provider apis",
    "provider apis are live",
    "provider apply",
    "production available service",
}


def fail(message: str) -> NoReturn:
    print(f"cloud enforceability facets check failed: {message}", file=sys.stderr)
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


def contains_forbidden(value: object) -> bool:
    haystack = f" {normalized(value)} "
    return any(f" {normalized(marker)} " in haystack for marker in FORBIDDEN_MARKERS)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path.relative_to(REPO_ROOT)}: {exc}")


def validate(spec: dict) -> None:
    catalog = load_json(RESOURCE_CATALOG_PATH)
    operation = load_json(OPERATION_CONTRACT_PATH)
    contract_ids = {contract["id"] for contract in catalog["resource_contracts"]}

    for field in [
        "spec_id",
        "title",
        "status",
        "source_resource_catalog",
        "source_operation_contract",
        "claim_controls",
        "facet_vocabulary",
        "resource_enforceability",
        "integration_points",
        "nonclaims",
        "next_goal_links",
    ]:
        require(field in spec, f"missing top-level field {field!r}")

    require(spec["status"] == "Proposed-target", "status must remain Proposed-target")
    require(spec["source_resource_catalog"] == str(RESOURCE_CATALOG_PATH.relative_to(REPO_ROOT)), "source_resource_catalog must point to G002 catalog")
    require(spec["source_operation_contract"] == str(OPERATION_CONTRACT_PATH.relative_to(REPO_ROOT)), "source_operation_contract must point to G003 operation contract")
    require(spec["claim_controls"].get("metadata_only") is True, "claim_controls must be metadata_only")
    require(spec["claim_controls"].get("no_runtime_policy_engine") is True, "must not claim runtime policy engine")
    require(spec["claim_controls"].get("no_billing_runtime") is True, "must not claim billing runtime")
    require(spec["claim_controls"].get("no_audit_runtime") is True, "must not claim audit runtime")
    require(spec["claim_controls"].get("strict_separation") is True, "must preserve strict separation")
    require(spec["claim_controls"].get("pure_dogfood") is True, "must preserve pure dogfood")
    require(spec["claim_controls"].get("no_tenant_workload_readiness") is True, "must not claim tenant workload readiness")
    require(not contains_forbidden(spec["claim_controls"].get("can_claim_now", [])), "can_claim_now contains forbidden positive claim")
    positive_claim_scan = json.loads(json.dumps(spec))
    positive_claim_scan.get("claim_controls", {}).pop("cannot_claim_yet", None)
    for item in positive_claim_scan.get("nonclaims", []):
        item.pop("id", None)
    for row in positive_claim_scan.get("resource_enforceability", []):
        row.pop("blocked_claim_families", None)
    require(not contains_forbidden(positive_claim_scan), "spec contains forbidden positive claim wording outside allowed blocked-claim fields")

    vocab = spec["facet_vocabulary"]
    require(REQUIRED_FACETS <= set(vocab.get("required_facets", [])), "facet vocabulary missing required facets")
    require(REQUIRED_CEDAR_FIELDS <= set(vocab.get("cedar_decision_fields", [])), "cedar decision fields incomplete")
    require(REQUIRED_AUDIT_FIELDS <= set(vocab.get("audit_event_fields", [])), "audit event fields incomplete")
    require(REQUIRED_BILLING_FIELDS <= set(vocab.get("billing_meter_fields", [])), "billing meter fields incomplete")

    rows = spec["resource_enforceability"]
    require(isinstance(rows, list) and rows, "resource_enforceability must be non-empty")
    row_ids = {row.get("resource_contract_id") for row in rows}
    require(contract_ids <= row_ids, f"missing enforceability rows for contracts {sorted(contract_ids - row_ids)}")
    for row in rows:
        cid = row.get("resource_contract_id")
        require(cid in contract_ids, f"unknown resource contract id {cid!r}")
        require(REQUIRED_FACETS <= set(row.get("facets", [])), f"{cid}: missing required facets")
        cedar = row.get("cedar_policy", {})
        require(REQUIRED_CEDAR_FIELDS <= set(cedar.get("decision_fields", [])), f"{cid}: cedar decision fields incomplete")
        require(cedar.get("default") == "deny", f"{cid}: Cedar default must be deny")
        tenant = row.get("tenant_scope", {})
        require(tenant.get("required") is True and tenant.get("fields") == ["tenant", "account", "project"], f"{cid}: tenant/account/project scope required")
        audit = row.get("audit", {})
        require(REQUIRED_AUDIT_FIELDS <= set(audit.get("event_fields", [])), f"{cid}: audit fields incomplete")
        quota = row.get("quota_cost", {})
        require(quota.get("admission") == "fail_closed", f"{cid}: quota admission must fail closed")
        metering = row.get("metering", {})
        require(REQUIRED_BILLING_FIELDS <= set(metering.get("fields", [])), f"{cid}: metering fields incomplete")
        require(metering.get("integration_point") == "oya-meter", f"{cid}: metering integration point must be oya-meter")
        require(metering.get("runtime_status") == "integration_point_only", f"{cid}: metering runtime_status must be integration_point_only")
        billing = row.get("billing", {})
        require(REQUIRED_BILLING_FIELDS <= set(billing.get("fields", [])), f"{cid}: billing fields incomplete")
        require(billing.get("integration_point") == "oya-billing", f"{cid}: billing integration point must be oya-billing")
        require(billing.get("runtime_status") == "integration_point_only", f"{cid}: billing runtime_status must be integration_point_only")
        require(set(row.get("blocked_claim_families", [])) >= REQUIRED_NONCLAIMS, f"{cid}: missing blocked claim families")
        require(not contains_forbidden(row.get("honest_claim", "")), f"{cid}: honest_claim contains forbidden wording")

    integrations = spec["integration_points"]
    require(integrations.get("cedar") and integrations.get("tenancy") and integrations.get("audit") and integrations.get("metering") and integrations.get("billing"), "all integration points required")
    require(integrations["cedar"].get("runtime_status") == "metadata_contract_only", "cedar runtime_status must be metadata_contract_only")
    require(integrations["tenancy"].get("runtime_status") == "integration_point_only", "tenancy runtime_status must be integration_point_only")
    require(integrations["audit"].get("runtime_status") == "integration_point_only", "audit runtime_status must be integration_point_only")
    require(integrations["billing"].get("runtime_status") == "integration_point_only", "billing runtime_status must be integration_point_only")
    require(integrations["metering"].get("runtime_status") == "integration_point_only", "metering runtime_status must be integration_point_only")

    nonclaim_ids = {item.get("id") for item in spec["nonclaims"]}
    require(REQUIRED_NONCLAIMS <= nonclaim_ids, f"missing nonclaims {sorted(REQUIRED_NONCLAIMS - nonclaim_ids)}")
    for item in spec["nonclaims"]:
        require(not contains_forbidden(item.get("statement", "")), f"nonclaim {item.get('id')}: statement contains forbidden positive claim")

    require(operation.get("spec_id") == "EXE-CLOUD-CONTROL-PLANE-OPERATION-CONTRACT", "unexpected operation contract source")
    require(spec["next_goal_links"].get("observability_slo_evidence") == "G005", "G005 link required")
    require(spec["next_goal_links"].get("production_quality_kits") == "G006", "G006 link required")
    require(spec["next_goal_links"].get("dogfood_ci_lane") == "G007", "G007 link required")


def main() -> None:
    validate(load_json(SPEC_PATH))
    print(f"cloud enforceability facets check passed: {SPEC_PATH.relative_to(REPO_ROOT)}")


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

    expect_rejected("missing contract row", lambda data: data.update({"resource_enforceability": data["resource_enforceability"][1:]}))
    expect_rejected("allow by default", lambda data: data["resource_enforceability"][0]["cedar_policy"].update({"default": "allow"}))
    expect_rejected("missing tenant project scope", lambda data: data["resource_enforceability"][0]["tenant_scope"].update({"fields": ["tenant", "account"]}))
    expect_rejected("quota not fail closed", lambda data: data["resource_enforceability"][0]["quota_cost"].update({"admission": "best_effort"}))
    expect_rejected("billing runtime claim", lambda data: data["claim_controls"].update({"can_claim_now": ["live billing runtime is available"]}))
    expect_rejected("audit runtime control disabled", lambda data: data["claim_controls"].update({"no_audit_runtime": False}))
    expect_rejected("strict separation disabled", lambda data: data["claim_controls"].update({"strict_separation": False}))
    expect_rejected("tenant workload control disabled", lambda data: data["claim_controls"].update({"no_tenant_workload_readiness": False}))
    expect_rejected("tenant workload overclaim", lambda data: data["resource_enforceability"][0].update({"honest_claim": "tenant workload ready"}))
    expect_rejected("missing billing currency field", lambda data: data["resource_enforceability"][0]["billing"].update({"fields": [field for field in data["resource_enforceability"][0]["billing"]["fields"] if field != "currency"]}))
    expect_rejected("cedar runtime available", lambda data: data["integration_points"]["cedar"].update({"runtime_status": "runtime_available"}))
    expect_rejected("audit runtime available", lambda data: data["integration_points"]["audit"].update({"runtime_status": "runtime_available"}))
    expect_rejected("tenancy runtime available", lambda data: data["integration_points"]["tenancy"].update({"runtime_status": "runtime_available"}))
    expect_rejected("row metering runtime available", lambda data: data["resource_enforceability"][0]["metering"].update({"runtime_status": "runtime_available"}))
    expect_rejected("row billing runtime available", lambda data: data["resource_enforceability"][0]["billing"].update({"runtime_status": "runtime_available"}))
    expect_rejected("wrong billing integration point", lambda data: data["resource_enforceability"][0]["billing"].update({"integration_point": "oya-meter"}))
    expect_rejected("Cedar runtime overclaim", lambda data: data.update({"purpose": "Cedar runtime engine available"}))
    expect_rejected("safe tenant workload overclaim", lambda data: data["integration_points"]["tenancy"].update({"note": "Tenant workloads can safely run"}))
    expect_rejected("provider APIs live overclaim", lambda data: data["doubt_driven_review"].update({"residual_risk": "Provider APIs are live"}))
    expect_rejected("production available overclaim", lambda data: data["doubt_driven_review"].update({"resolution": "Production available service"}))
    expect_rejected("AWS parity overclaim", lambda data: data.update({"purpose": "AWS parity exists"}))
    expect_rejected("SLA-backed overclaim", lambda data: data["integration_points"]["audit"].update({"note": "SLA-backed service"}))
    print("cloud enforceability facets self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
