#!/usr/bin/env python3
"""Validate the Oyatie Cloud hyperscaler parity taxonomy artifact.

This is intentionally stdlib-only so it can run in self-hosted dogfood CI lanes
without pulling external schema tooling.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "specs" / "cloud-hyperscaler-parity-taxonomy.json"

REQUIRED_CATEGORIES = {
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

PROVIDERS = {"aws", "google_cloud", "azure", "oci"}
OFFICIAL_SOURCE_DOMAINS = {
    "aws": ("https://aws.amazon.com/", "https://docs.aws.amazon.com/"),
    "google_cloud": ("https://cloud.google.com/", "https://docs.cloud.google.com/"),
    "azure": ("https://azure.microsoft.com/", "https://learn.microsoft.com/"),
    "oci": ("https://www.oracle.com/", "https://docs.oracle.com/"),
    "kubernetes": ("https://kubernetes.io/",),
    "cncf": ("https://github.com/cncf/", "https://www.cncf.io/"),
}
REQUIRED_NONCLAIMS = {
    "hyperscaler_mature",
    "provider_feature_parity",
    "production_ready",
    "tenant_workload_ready",
    "public_sla_or_slo",
    "live_provider_provisioning",
}
REQUIRED_CONTROLS = {"strict_separation", "pure_dogfood", "self_hosted_ci_lane", "no_external_hyperscaler_runtime_dependency"}
FORBIDDEN_CONTROL_MARKERS = {"github_actions_fallback", "external_saas_ci", "public_cloud_runtime_dependency"}
FORBIDDEN_EVIDENCE_LANES = {"github actions", "external saas", "github-actions", "public cloud"}
REQUIRED_EVIDENCE_CLASSES = {
    "official-provider-category-evidence",
    "machine-readable resource contract",
    "implementation or adapter boundary",
    "targeted tests plus governance gate evidence",
    "measured operational evidence before production claim",
}
VAGUE_EVIDENCE_MARKERS = {"todo", "tbd", "later", "placeholder", "fixme"}
FORBIDDEN_CAN_CLAIM_PHRASES = {
    "feature parity",
    "same feature parity",
    "hyperscaler mature",
    "hyperscaler-mature",
    "hyperscaler maturity",
    "reaches hyperscaler maturity",
    "production ready",
    "production-ready",
    "production readiness",
    "production-readiness",
    "tenant workload ready",
    "tenant/product workload readiness",
    "public sla",
    "public slo",
    "live provider provisioning",
    "provisions real cloud resources",
}


def fail(message: str) -> NoReturn:
    print(f"cloud hyperscaler parity taxonomy check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def flattened_text(value: object) -> str:
    if isinstance(value, dict):
        return " ".join(flattened_text(item) for item in value.values())
    if isinstance(value, (list, tuple, set)):
        return " ".join(flattened_text(item) for item in value)
    return str(value).lower()


def normalized_claim_text(value: object) -> str:
    return re.sub(r"[^a-z0-9]+", " ", flattened_text(value)).strip()


def contains_forbidden_claim(value: object) -> bool:
    text = f" {normalized_claim_text(value)} "
    forbidden = {f" {normalized_claim_text(phrase)} " for phrase in FORBIDDEN_CAN_CLAIM_PHRASES}
    return any(phrase in text for phrase in forbidden)


def main() -> None:
    require(SPEC_PATH.exists(), f"missing {SPEC_PATH.relative_to(REPO_ROOT)}")
    try:
        spec = json.loads(SPEC_PATH.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON: {exc}")

    for field in [
        "spec_id",
        "title",
        "status",
        "retrieved_at",
        "purpose",
        "official_source_evidence",
        "local_authority",
        "controls",
        "strict_separation_constraints",
        "pure_dogfood_constraints",
        "evidence_vocabulary",
        "category_taxonomy",
        "local_oyatie_mapping",
        "claim_matrix",
        "nonclaims",
        "next_goal_mapping",
    ]:
        require(field in spec, f"missing top-level field {field!r}")

    require(spec["status"] == "Proposed-target", "status must stay Proposed-target until runtime evidence exists")
    require(set(spec["controls"]) >= REQUIRED_CONTROLS, "controls must preserve strict dogfood separation")
    require(not (set(spec["controls"]) & FORBIDDEN_CONTROL_MARKERS), "controls must not include external-SaaS/GitHub Actions fallback markers")
    require(spec["strict_separation_constraints"].get("no_external_saas_ci") is True, "strict separation must forbid external SaaS CI")
    require(spec["strict_separation_constraints"].get("no_live_provider_apply") is True, "strict separation must forbid live provider apply/provisioning")
    require(spec["strict_separation_constraints"].get("no_github_actions_fallback") is True, "strict separation must forbid GitHub Actions fallback")
    require(spec["strict_separation_constraints"].get("no_public_cloud_runtime_dependency") is True, "strict separation must forbid public cloud runtime dependency")
    allowed_lanes_text = flattened_text(spec["strict_separation_constraints"].get("allowed_evidence_lanes", []))
    require(not any(marker in allowed_lanes_text for marker in FORBIDDEN_EVIDENCE_LANES), "allowed evidence lanes must not permit external SaaS/GitHub Actions/public-cloud runtime")
    require(spec["pure_dogfood_constraints"].get("self_hosted_github_kubernetes_ci_lane") is True, "pure dogfood must require GitHub (interim)/Kubernetes CI lane")
    require(spec["pure_dogfood_constraints"].get("dogfood_resource_substrate_required_before_external_provider_apply") is True, "pure dogfood must require dogfood resource substrate before external provider apply")
    require(spec["pure_dogfood_constraints"].get("vfkit_linux_or_kubernetes_cluster_tests_must_be_recorded_before_kubernetes_readiness_claim") is True, "pure dogfood must require vfkit/Linux or Kubernetes cluster test evidence before Kubernetes readiness claims")
    require(spec["pure_dogfood_constraints"].get("g007_must_reconcile_historical_jenkins_wording") is True, "pure dogfood must require G007 Jenkins/bespoke-CI reconciliation")
    require(set(spec["evidence_vocabulary"].get("required_category_evidence_classes", [])) >= REQUIRED_EVIDENCE_CLASSES, "evidence vocabulary must include required category evidence classes")

    source_providers = {source.get("provider") for source in spec["official_source_evidence"]}
    require(PROVIDERS <= source_providers, "official source evidence must include AWS, Google Cloud, Azure, and OCI")
    require("kubernetes" in source_providers and "cncf" in source_providers, "official source evidence must include Kubernetes and CNCF")
    for source in spec["official_source_evidence"]:
        provider = source.get("provider")
        url = source.get("url", "")
        require(provider in OFFICIAL_SOURCE_DOMAINS, f"source has unsupported provider: {source!r}")
        require(url.startswith(OFFICIAL_SOURCE_DOMAINS[provider]), f"source URL is not official for {provider}: {url}")
        require(source.get("evidence_use"), f"source missing evidence_use: {source!r}")
        require(isinstance(source.get("category_coverage"), list) and source["category_coverage"], f"source missing category_coverage: {source!r}")

    categories = spec["category_taxonomy"]
    require(isinstance(categories, list) and categories, "category_taxonomy must be a non-empty list")
    category_ids = {category.get("id") for category in categories}
    require(REQUIRED_CATEGORIES <= category_ids, f"missing categories: {sorted(REQUIRED_CATEGORIES - category_ids)}")
    coverage_by_provider = {}
    for source in spec["official_source_evidence"]:
        coverage_by_provider.setdefault(source["provider"], set()).update(source["category_coverage"])

    for category in categories:
        category_id = category.get("id")
        examples = category.get("provider_examples", {})
        require(category.get("target_capability"), f"{category_id}: missing target_capability")
        if category_id != "cloud_native_platform_contract":
            require(PROVIDERS <= set(examples), f"{category_id}: provider_examples must cover all four hyperscalers")
        category_evidence = set(category.get("required_evidence", []))
        require(category_evidence, f"{category_id}: missing required_evidence")
        require(REQUIRED_EVIDENCE_CLASSES <= category_evidence, f"{category_id}: missing required evidence classes")
        vague = VAGUE_EVIDENCE_MARKERS & set(normalized_claim_text(category.get("required_evidence", [])).split())
        require(not vague, f"{category_id}: vague evidence markers are forbidden: {sorted(vague)}")
        gates = set(category.get("hyperscaler_gates", []))
        require(gates, f"{category_id}: must map to hyperscaler gates")
        require(any(gate.startswith("HG-") for gate in gates), f"{category_id}: gate ids must use HG-* ids")
        source_providers_for_category = {provider for provider, covered in coverage_by_provider.items() if category_id in covered}
        if category_id == "cloud_native_platform_contract":
            require({"kubernetes", "cncf"} <= source_providers_for_category, "cloud_native_platform_contract must be grounded in Kubernetes and CNCF sources")
        else:
            require(PROVIDERS <= source_providers_for_category, f"{category_id}: official source coverage must include every hyperscaler provider")

    mapping_ids = {mapping.get("category_id") for mapping in spec["local_oyatie_mapping"]}
    require(REQUIRED_CATEGORIES <= mapping_ids, "local_oyatie_mapping must cover every required category")
    for mapping in spec["local_oyatie_mapping"]:
        require(mapping.get("claim_status") in {"target_spec_only", "metadata_foundation", "evidence_required"}, f"invalid claim_status in {mapping!r}")
        require(mapping.get("honest_claim") and mapping.get("cannot_claim_yet"), f"mapping lacks claim/nonclaim text: {mapping!r}")
        require(set(mapping.get("blocked_claim_families", [])) >= REQUIRED_NONCLAIMS, f"mapping must carry every required blocked claim family: {mapping!r}")
        honest_claim_text = flattened_text(mapping.get("honest_claim", ""))
        require(not contains_forbidden_claim(honest_claim_text), f"mapping honest_claim contains forbidden readiness/parity wording: {mapping!r}")
        cannot_claim_text = flattened_text(mapping.get("cannot_claim_yet", []))
        require("feature parity" in cannot_claim_text and "production" in cannot_claim_text, f"mapping must explicitly keep parity/production claims in cannot_claim_yet: {mapping!r}")

    matrix = spec["claim_matrix"]
    for field in ["can_claim_now", "cannot_claim_yet", "evidence_required_before_claim"]:
        require(isinstance(matrix.get(field), list) and matrix[field], f"claim_matrix.{field} must be a non-empty list")
    can_claim_text = flattened_text(matrix["can_claim_now"])
    require(not contains_forbidden_claim(can_claim_text), "claim_matrix.can_claim_now contains forbidden readiness/parity wording")

    nonclaims = {item.get("id") for item in spec["nonclaims"]}
    require(REQUIRED_NONCLAIMS <= nonclaims, f"missing nonclaims: {sorted(REQUIRED_NONCLAIMS - nonclaims)}")

    next_goal_ids = set(spec["next_goal_mapping"].values())
    require({"G002", "G003", "G004", "G005", "G006", "G007"} <= next_goal_ids, "next_goal_mapping must connect taxonomy to remaining implementation goals")
    require(spec["next_goal_mapping"].get("dogfood_ci_claim_path") == "G007", "dogfood CI claim path must be explicitly mapped to G007")

    claim_families = {item.get("claim_family"): flattened_text(item.get("requires", [])) for item in matrix["evidence_required_before_claim"]}
    require("strict_dogfood_ci" in claim_families, "strict_dogfood_ci evidence requirement is mandatory")
    strict_ci_text = claim_families["strict_dogfood_ci"]
    require("no external saas fallback" in strict_ci_text, "strict_dogfood_ci must forbid external SaaS fallback")
    require("self-hosted" in strict_ci_text and "g007" in strict_ci_text, "strict_dogfood_ci must require G007 self-hosted evidence")

    print(f"cloud hyperscaler parity taxonomy check passed: {SPEC_PATH.relative_to(REPO_ROOT)}")


def run_self_tests() -> None:
    """Run adversarial mutation probes against the checked-in validator."""
    try:
        baseline = json.loads(SPEC_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {SPEC_PATH.relative_to(REPO_ROOT)}")

    def expect_rejected(label: str, mutator: Callable[[dict], None]) -> None:
        candidate = json.loads(json.dumps(baseline))
        mutator(candidate)
        temp_path = SPEC_PATH.with_suffix(".selftest.json")
        original_path = SPEC_PATH.read_text(encoding="utf-8")
        try:
            SPEC_PATH.write_text(json.dumps(candidate, indent=2) + "\n", encoding="utf-8")
            try:
                main()
            except SystemExit as exc:
                require(exc.code != 0, f"self-test {label!r} exited successfully")
            else:
                fail(f"self-test mutation was accepted: {label}")
        finally:
            SPEC_PATH.write_text(original_path, encoding="utf-8")
            if temp_path.exists():
                temp_path.unlink()

    expect_rejected(
        "forbidden production/parity can_claim_now",
        lambda data: data["claim_matrix"]["can_claim_now"][0].update(
            {"claim": "Oyatie Cloud is production ready with same feature parity and public SLA"}
        ),
    )
    expect_rejected("unofficial source URL", lambda data: data["official_source_evidence"][0].update({"url": "https://example.com/not-official"}))
    expect_rejected("missing category blocked claim families", lambda data: data["local_oyatie_mapping"][0].update({"blocked_claim_families": ["provider_feature_parity"]}))
    expect_rejected("vague evidence marker", lambda data: data["category_taxonomy"][0].update({"required_evidence": ["TODO evidence later"]}))
    expect_rejected("standalone vague evidence marker", lambda data: data["category_taxonomy"][0].update({"required_evidence": ["TODO"]}))
    expect_rejected("external SaaS fallback dogfood gap", lambda data: data["claim_matrix"]["evidence_required_before_claim"][-1].update({"requires": ["self-hosted evidence requirements"]}))
    expect_rejected("GitHub Actions fallback disabled", lambda data: data["strict_separation_constraints"].update({"no_github_actions_fallback": False}))
    expect_rejected("public cloud runtime dependency allowed", lambda data: data["strict_separation_constraints"].update({"no_public_cloud_runtime_dependency": False}))
    expect_rejected("GitHub Actions allowed evidence lane", lambda data: data["strict_separation_constraints"]["allowed_evidence_lanes"].append("GitHub Actions"))
    expect_rejected("GitHub Actions control marker", lambda data: data["controls"].append("github_actions_fallback"))
    expect_rejected("missing dogfood substrate constraint", lambda data: data["pure_dogfood_constraints"].pop("dogfood_resource_substrate_required_before_external_provider_apply", None))
    expect_rejected("hyperscaler maturity overclaim variant", lambda data: data["local_oyatie_mapping"][0].update({"honest_claim": "This reaches hyperscaler maturity targets."}))
    expect_rejected(
        "hyphenated feature-parity overclaim variant",
        lambda data: data["claim_matrix"]["can_claim_now"][0].update({"claim": "Oyatie has feature-parity with AWS."}),
    )
    expect_rejected(
        "hyphenated production-readiness overclaim variant",
        lambda data: data["claim_matrix"]["can_claim_now"][0].update({"claim": "Oyatie has production-readiness for tenant workloads."}),
    )
    print("cloud hyperscaler parity taxonomy self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
