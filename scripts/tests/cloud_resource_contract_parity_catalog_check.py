#!/usr/bin/env python3
"""Validate the Oyatie Cloud resource-contract parity catalog companion spec."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "specs" / "cloud-resource-contract-parity-catalog.json"
TAXONOMY_PATH = REPO_ROOT / "specs" / "cloud-hyperscaler-parity-taxonomy.json"
BASE_CATALOG_PATH = REPO_ROOT / "specs" / "cloud-resource-catalog-target.json"

REQUIRED_FACETS = {
    "orn",
    "lifecycle_state",
    "quota_cost",
    "billing_meters",
    "audit_events",
    "tenant_account_project",
    "region_cell",
    "owner",
    "deletion_retention",
    "slo_tier",
}
REQUIRED_CATEGORY_IDS = {
    "identity_access_policy",
    "compute_instances",
    "containers_kubernetes",
    "serverless_functions",
    "storage_object_block_file",
    "networking_dns_edge",
    "databases_data_analytics",
    "kms_secrets_confidentiality",
    "observability_operations",
    "billing_finops_quotas",
    "marketplace_isv_ecosystem",
    "security_posture_guardrails",
    "cloud_native_platform_contract",
}
REQUIRED_NONCLAIMS = {
    "no_live_provider_provisioning",
    "no_provider_feature_parity_claim",
    "no_production_readiness_claim",
    "no_tenant_workload_claim",
    "no_public_sla_slo_claim",
}
FORBIDDEN_CAN_CLAIM_PHRASES = {
    "feature parity",
    "feature-parity",
    "aws equivalent behavior",
    "equivalent behavior",
    "same behavior as aws",
    "same behavior as google cloud",
    "same behavior as azure",
    "same behavior as oci",
    "production ready",
    "production-ready",
    "production readiness",
    "production-readiness",
    "tenant workload ready",
    "tenant workloads can run",
    "product workloads can run",
    "public sla",
    "public slo",
    "public 99.9 availability objective",
    "availability objective",
    "live provider provisioning",
    "provisions real cloud resources",
    "external providers through provider apis",
    "provider apis",
    "hyperscaler mature",
    "hyperscaler maturity",
}
FORBIDDEN_ACTUATION_MARKERS = {
    "terraform apply",
    "opentofu apply",
    "kubectl apply",
    "aws cli",
    "gcloud cli",
    "az cli",
    "oci cli",
    "aws provider api",
    "google cloud provider api",
    "gcp provider api",
    "azure provider api",
    "oci provider api",
    "provider api",
    "provider apis",
    "call aws provider api",
}


def fail(message: str) -> NoReturn:
    print(f"cloud resource contract parity catalog check failed: {message}", file=sys.stderr)
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


def contains_forbidden_claim(value: object) -> bool:
    haystack = f" {normalized(value)} "
    return any(f" {normalized(phrase)} " in haystack for phrase in FORBIDDEN_CAN_CLAIM_PHRASES)


def contains_forbidden_positive_claim(value: object) -> bool:
    """Detect positive overclaims without flagging canonical blocked-claim IDs."""
    candidate = json.loads(json.dumps(value))
    if isinstance(candidate, dict):
        candidate.pop("blocked_claim_families", None)
    return contains_forbidden_claim(candidate)


def contains_forbidden_actuation(value: object) -> bool:
    haystack = f" {normalized(value)} "
    return any(f" {normalized(marker)} " in haystack for marker in FORBIDDEN_ACTUATION_MARKERS)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path.relative_to(REPO_ROOT)}: {exc}")


def validate(spec: dict) -> None:
    taxonomy = load_json(TAXONOMY_PATH)
    base_catalog = load_json(BASE_CATALOG_PATH)

    for field in [
        "spec_id",
        "title",
        "status",
        "source_taxonomy",
        "base_catalog",
        "claim_controls",
        "required_facets",
        "resource_contracts",
        "category_coverage",
        "nonclaims",
        "next_goal_links",
    ]:
        require(field in spec, f"missing top-level field {field!r}")

    require(spec["status"] == "Proposed-target", "status must remain Proposed-target until runtime evidence exists")
    require(spec["source_taxonomy"] == str(TAXONOMY_PATH.relative_to(REPO_ROOT)), "source_taxonomy must point to G001 taxonomy")
    require(spec["base_catalog"] == str(BASE_CATALOG_PATH.relative_to(REPO_ROOT)), "base_catalog must point to the existing catalog target")
    require(set(spec["required_facets"]) >= REQUIRED_FACETS, "required_facets must include every control-plane facet")

    controls = spec["claim_controls"]
    require(controls.get("no_live_provider_apply") is True, "claim_controls must forbid live provider apply")
    require(controls.get("strict_separation") is True and controls.get("pure_dogfood") is True, "claim_controls must preserve strict separation and pure dogfood")
    require(controls.get("metadata_only") is True, "claim_controls must mark this catalog metadata-only")
    require(not contains_forbidden_claim(controls.get("can_claim_now", [])), "claim_controls.can_claim_now contains forbidden readiness/parity wording")

    taxonomy_ids = {c["id"] for c in taxonomy["category_taxonomy"]}
    require(REQUIRED_CATEGORY_IDS <= taxonomy_ids, "taxonomy is missing required categories")
    required_category_ids = taxonomy_ids

    contracts = spec["resource_contracts"]
    require(isinstance(contracts, list) and contracts, "resource_contracts must be a non-empty list")
    contract_ids = set()
    categories_by_contract = set()
    existing_services = {svc["service"] for svc in base_catalog.get("services", [])}
    for contract in contracts:
        cid = contract.get("id")
        require(cid and cid not in contract_ids, f"duplicate or missing contract id: {cid!r}")
        contract_ids.add(cid)
        category_id = contract.get("category_id")
        require(category_id in required_category_ids, f"{cid}: invalid or unsupported category_id {category_id!r}")
        categories_by_contract.add(category_id)
        require(contract.get("service") in existing_services or contract.get("service") == "cloud-marketplace", f"{cid}: service must reference existing catalog service or scoped cloud-marketplace target")
        facets = set(contract.get("facets", []))
        require(REQUIRED_FACETS <= facets, f"{cid}: missing required facets {sorted(REQUIRED_FACETS - facets)}")
        require(str(contract.get("orn_pattern", "")).startswith("orn:oyatie:cloud:"), f"{cid}: ORN pattern must use orn:oyatie:cloud prefix")
        require(contract.get("lifecycle_states"), f"{cid}: missing lifecycle_states")
        require(contract.get("quota_cost"), f"{cid}: missing quota_cost")
        require(contract.get("billing_meters"), f"{cid}: missing billing_meters")
        require(contract.get("audit_events"), f"{cid}: missing audit_events")
        require(contract.get("slo_tier"), f"{cid}: missing slo_tier")
        require(contract.get("deletion_retention"), f"{cid}: missing deletion_retention")
        require(contract.get("owner") and contract.get("tenant_account_project") and contract.get("region_cell"), f"{cid}: missing ownership/tenant/region facets")
        require(contract.get("actuation_status") in {"metadata_only", "adapter_boundary_only", "evidence_required"}, f"{cid}: invalid actuation_status")
        require(not contains_forbidden_actuation(contract), f"{cid}: contains forbidden live actuation marker")
        require(not contains_forbidden_claim(contract.get("honest_claim", "")), f"{cid}: honest_claim contains forbidden readiness/parity wording")
        blocked = set(contract.get("blocked_claim_families", []))
        require(REQUIRED_NONCLAIMS <= blocked, f"{cid}: missing blocked claim families {sorted(REQUIRED_NONCLAIMS - blocked)}")

    require(required_category_ids <= categories_by_contract, f"missing resource contracts for categories {sorted(required_category_ids - categories_by_contract)}")

    coverage = spec["category_coverage"]
    require(set(coverage) >= required_category_ids, "category_coverage must cover every taxonomy category")
    for category_id in required_category_ids:
        row = coverage[category_id]
        require(row.get("resource_contract_ids"), f"{category_id}: missing resource_contract_ids")
        require(set(row["resource_contract_ids"]) <= contract_ids, f"{category_id}: references unknown contract ids")
        require(row.get("claim_status") in {"metadata_only", "target_spec_only", "evidence_required"}, f"{category_id}: invalid claim_status")
        require(set(row.get("blocked_claim_families", [])) >= REQUIRED_NONCLAIMS, f"{category_id}: missing blocked claim families")
        require(not contains_forbidden_positive_claim(row), f"{category_id}: category_coverage contains forbidden readiness/parity wording")

    nonclaims = {item.get("id") for item in spec["nonclaims"]}
    require(REQUIRED_NONCLAIMS <= nonclaims, f"missing nonclaims {sorted(REQUIRED_NONCLAIMS - nonclaims)}")
    for item in spec["nonclaims"]:
        require(not contains_forbidden_positive_claim(item.get("statement", "")), f"nonclaim {item.get('id')}: statement contains forbidden positive-claim wording")
    require({"G003", "G004", "G005", "G006", "G007"} <= set(spec["next_goal_links"].values()), "next_goal_links must route remaining facets to later ultragoal stories")


def main() -> None:
    validate(load_json(SPEC_PATH))
    print(f"cloud resource contract parity catalog check passed: {SPEC_PATH.relative_to(REPO_ROOT)}")


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

    expect_rejected("missing facet", lambda data: data["resource_contracts"][0].update({"facets": ["orn"]}))
    expect_rejected("feature-parity overclaim", lambda data: data["claim_controls"].update({"can_claim_now": ["feature-parity with AWS"]}))
    expect_rejected("live apply marker", lambda data: data["resource_contracts"][0].update({"notes": "run terraform apply"}))
    expect_rejected("provider API actuation marker", lambda data: data["resource_contracts"][0].update({"notes": "call AWS provider API"}))
    expect_rejected("missing blocked claim family", lambda data: data["resource_contracts"][0].update({"blocked_claim_families": ["no_provider_feature_parity_claim"]}))
    expect_rejected("unknown resource category", lambda data: data["resource_contracts"][0].update({"category_id": "unsupported"}))
    expect_rejected("provider-equivalent behavior overclaim", lambda data: data["claim_controls"].update({"can_claim_now": ["AWS equivalent behavior"]}))
    expect_rejected("tenant workload overclaim in category coverage", lambda data: data["category_coverage"]["compute_instances"].update({"notes": "Tenant workloads can run here"}))
    expect_rejected("provider API overclaim in category coverage", lambda data: data["category_coverage"]["compute_instances"].update({"notes": "external providers through provider APIs"}))
    expect_rejected("availability objective overclaim in nonclaim statement", lambda data: data["nonclaims"][0].update({"statement": "public 99.9 availability objective is available"}))
    print("cloud resource contract parity catalog self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
