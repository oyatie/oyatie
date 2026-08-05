#!/usr/bin/env python3
"""Validate the HR preview cloud deployment evidence Plan/Spec/RED fixture.

This guard binds t_53c075b7 to source-backed HR non-claims and to the
DeploymentOpsContract without claiming a live cluster, production rollout, GA
readiness, or hyperscaler deployment. The fixture is allowed to describe future
Build/IaC evidence, but every live/deployed/runtime attachment must stay false
until a later reviewed Build card provides source-backed evidence.
"""
from __future__ import annotations

import argparse
import contextlib
import copy
import io
import json
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_MANIFEST = (
    REPO_ROOT
    / "specs"
    / "fixtures"
    / "hr-cloud-deployment"
    / "preview-deployment-evidence-plan.json"
)
HR_PRD_PATH = REPO_ROOT / "specs" / "microservices" / "hr.json"
DEPLOYMENT_OPS_PATH = REPO_ROOT / "specs" / "deployment-ops-contract.json"
HR_OPENAPI_PATH = REPO_ROOT / "oya" / "hr" / "contracts" / "openapi-v1.yaml"
HR_OPENAPI_META_PATH = REPO_ROOT / "oya" / "hr" / "contracts" / "openapi-v1.meta.yaml"
HR_RUNTIME_PATH = (
    REPO_ROOT / "oya" / "hr" / "crates" / "oya-hr-employment-infrastructure" / "src" / "lib.rs"
)
HR_POSTGRES_PATH = (
    REPO_ROOT
    / "oya"
    / "hr"
    / "crates"
    / "oya-hr-employment-storage-adapter-postgres"
    / "src"
    / "lib.rs"
)

EXPECTED_FIXTURE_IDS = [
    "hr_preview_deployment_topology_fixture",
    "hr_tenant_isolation_fixture",
    "hr_rollback_observability_slo_fixture",
    "hr_secret_handling_fixture",
    "hr_local_replay_and_live_na_fixture",
    "hr_no_ga_cloud_claim_fixture",
]

EXPECTED_EVIDENCE_REQUIREMENT_IDS = [
    "argocd-application-sync",
    "argocd-application-health",
    "git-revision-pin",
    "cosign-sbom-provenance",
    "tenant-namespace-observed",
    "resource-quota-observed",
    "network-policy-observed",
    "service-account-observed",
    "deployment-available-observed",
    "readiness-healthz-green",
    "gateway-httproute-accepted",
    "otel-resource-identity-observed",
    "deployment-audit-event-recorded",
    "rollback-plan-attached",
    "secret-provider-binding-verified",
    "local-loopback-replay-attached",
]

REQUIRED_SOURCE_REFS = {
    "specs/microservices/hr.json#acceptance_criteria[AC-12]",
    "specs/microservices/hr.json#acceptance_criteria[AC-13]",
    "specs/microservices/hr.json#acceptance_criteria[AC-14]",
    "specs/microservices/hr.json#decision_log[D-14]",
    "specs/microservices/hr.json#decision_log[D-15]",
    "specs/microservices/hr.json#decision_log[D-16]",
    "specs/deployment-ops-contract.json#deployment_authority",
    "specs/deployment-ops-contract.json#cluster_fleet",
    "oya/hr/contracts/openapi-v1.yaml#/components/schemas/HrHealth",
    "oya/hr/contracts/openapi-v1.meta.yaml#change_notes[CS-ENT-HR-008]",
    "oya/hr/crates/oya-hr-employment-infrastructure/src/lib.rs#HrHealthResponse",
    "oya/hr/crates/oya-hr-employment-storage-adapter-postgres/src/lib.rs#HrPostgresStorageCapabilities",
    "kanban:t_71e1bfaf#runtime-listener-boundary",
    "kanban:t_9daca2db#durable-storage-rls-seam",
    "kanban:t_a0611892#tenant-rbac-admission-scope",
    "kanban:t_3a7d8b2c#audit-event-class-contract",
    "kanban:t_53c075b7#hr-cloud-deployment-plan-red",
}

RUNTIME_FALSE_FLAGS = [
    "productionDeploymentClaimed",
    "gaClaimed",
    "hyperscalerClaimed",
    "deployedListener",
    "cloudDeploymentEvidenceAttached",
    "runtimeCloudIoAttached",
    "runtimeDatabaseAttached",
    "workflowExecutionAttached",
    "payrollNetworkCallAttached",
    "sensitiveDataFetchAttached",
    "runtimeAuditEmissionAttached",
    "generatedJsonHandEdited",
    "releasePleaseAssumed",
]

HEALTH_FALSE_FLAGS = [
    "deployedListener",
    "storageAttached",
    "workflowExecution",
    "payrollNetworkCall",
    "sensitiveDataFetch",
    "runtimeAuditEmission",
    "cloudDeployment",
]

ALLOWED_OFFICIAL_DOC_PREFIXES = (
    "https://argo-cd.readthedocs.io/",
    "https://kubernetes.io/docs/",
    "https://gateway-api.sigs.k8s.io/",
    "https://opentelemetry.io/docs/",
    "https://cloudevents.io/",
    "https://docs.sigstore.dev/",
    "https://openbao.org/docs/",
)

EXPECTED_RED_STATUS = "RED_UNTIL_LIVE_DEPLOYMENT_BUILD"


def fail(message: str) -> NoReturn:
    print(f"HR cloud deployment evidence plan check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def rel(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def text(value: object) -> str:
    if isinstance(value, dict):
        return " ".join(text(item) for item in value.values())
    if isinstance(value, list):
        return " ".join(text(item) for item in value)
    return str(value).lower()


def load_json(path: Path, label: str) -> dict[str, Any]:
    try:
        candidate = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {label}: {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON {label} {rel(path)}: {exc}")
    require(isinstance(candidate, dict), f"{label} must be a JSON object")
    return candidate


def read_text(path: Path, label: str) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing {label}: {rel(path)}")


def require_contains_all(values: object, expected: set[str] | list[str], label: str) -> None:
    require(isinstance(values, list), f"{label} must be a list")
    actual = {str(item) for item in values}
    missing = sorted(set(expected) - actual)
    require(not missing, f"{label} missing {missing}")


def require_terms(value: object, required_terms: list[str], label: str) -> None:
    haystack = text(value)
    missing = [term for term in required_terms if term.lower() not in haystack]
    require(not missing, f"{label} missing terms {missing}")


def require_all_false(mapping: object, keys: list[str], label: str) -> None:
    require(isinstance(mapping, dict), f"{label} must be an object")
    for key in keys:
        require(mapping.get(key) is False, f"{label}.{key} must be false")


def object_by_id(rows: object, key: str, expected_ids: list[str], label: str) -> dict[str, dict[str, Any]]:
    require(isinstance(rows, list), f"{label} must be a list")
    by_id: dict[str, dict[str, Any]] = {}
    for row in rows:
        require(isinstance(row, dict), f"each {label} row must be an object")
        row_id = str(row.get(key, ""))
        require(row_id, f"{label} row missing {key}")
        require(row_id not in by_id, f"duplicate {label} {row_id}")
        by_id[row_id] = row
    actual_ids = list(by_id)
    require(actual_ids == expected_ids, f"{label} ids/order must be {expected_ids}; got {actual_ids}")
    return by_id


def validate_prd_nonclaims(prd: dict[str, Any]) -> None:
    require(prd.get("_meta", {}).get("spec_id") == "PRD-HR", "HR PRD spec_id must be PRD-HR")
    acceptance_rows = prd.get("acceptance_criteria")
    require(isinstance(acceptance_rows, list), "HR PRD acceptance_criteria must be a list")
    ac_by_id = {row.get("id"): row for row in acceptance_rows if isinstance(row, dict)}
    for ac_id in ["AC-12", "AC-13", "AC-14"]:
        require(ac_id in ac_by_id, f"HR PRD missing {ac_id}")
    require_terms(
        ac_by_id["AC-12"],
        [
            "deployed listener",
            "storage",
            "Workflow execution",
            "Payroll calls",
            "sensitive-data fetch",
            "runtime audit emission",
        ],
        "AC-12 runtime/listener non-claims",
    )
    require_terms(
        ac_by_id["AC-13"],
        ["durable backend", "Postgres/RLS", "sensitive data retrieval", "audit-chain emission"],
        "AC-13 durable storage non-claims",
    )
    require_terms(
        ac_by_id["AC-14"],
        ["Workflow engine", "payroll calculation", "filing rail", "cloud deployment", "runtime audit emission"],
        "AC-14 rulepack/cloud non-claims",
    )

    decisions = prd.get("decision_log")
    require(isinstance(decisions, list), "HR PRD decision_log must be a list")
    decision_by_id = {row.get("id"): row for row in decisions if isinstance(row, dict)}
    for decision_id in ["D-14", "D-15", "D-16"]:
        require(decision_id in decision_by_id, f"HR PRD missing decision {decision_id}")
    require_terms(
        decision_by_id["D-14"],
        [
            "does not deploy a listener",
            "persist state",
            "execute Workflow",
            "call Payroll",
            "retrieve sensitive HR data",
            "emit runtime audit-chain events",
        ],
        "D-14 runtime adapter boundary",
    )
    require_terms(
        decision_by_id["D-15"],
        ["volatile", "does not attach", "durable storage", "audit-chain emission"],
        "D-15 storage boundary",
    )
    require_terms(
        decision_by_id["D-16"],
        ["cloud deployment", "production statutory correctness", "runtime audit-chain emission"],
        "D-16 rulepack/cloud boundary",
    )

    scope = prd.get("scope", {}).get("in_scope_per_wave", {})
    require("high-volume group HR operations" in scope.get("GA", []), "HR GA scope must retain scale proof gap")


def validate_openapi_nonclaims(openapi: dict[str, Any], meta_text: str) -> None:
    info = openapi.get("info", {})
    require(info.get("x-oyatie-contract-status") == "preview-not-deployed", "HR OpenAPI must stay preview-not-deployed")
    require_terms(
        info.get("x-oyatie-non-claims", []),
        ["no production-deployed HTTP runtime", "no cloud service integration claim"],
        "HR OpenAPI info non-claims",
    )
    require("CS-ENT-HR-008" in meta_text, "HR OpenAPI metadata must retain CS-ENT-HR-008")
    require("cloud deployment" in meta_text.lower(), "CS-ENT-HR-008 must preserve cloud deployment non-claim")

    schemas = openapi.get("components", {}).get("schemas", {})
    health = schemas.get("HrHealth")
    require(isinstance(health, dict), "HR OpenAPI must expose HrHealth schema")
    properties = health.get("properties", {})
    require(isinstance(properties, dict), "HrHealth properties must be an object")
    for flag in HEALTH_FALSE_FLAGS:
        prop = properties.get(flag, {})
        require(isinstance(prop, dict), f"HrHealth.{flag} must be an object")
        require(prop.get("const") is False, f"HrHealth.{flag} must const false")
    require_terms(health.get("x-oyatie-non-claims", []), ["cloud deployment"], "HrHealth non-claim")


def validate_source_boundaries() -> None:
    validate_prd_nonclaims(load_json(HR_PRD_PATH, "HR PRD"))
    validate_openapi_nonclaims(
        load_json(HR_OPENAPI_PATH, "HR OpenAPI contract"),
        read_text(HR_OPENAPI_META_PATH, "HR OpenAPI metadata"),
    )

    runtime_text = read_text(HR_RUNTIME_PATH, "HR runtime adapter source")
    require_terms(
        runtime_text,
        [
            "does not persist HR records",
            "retrieve sensitive data",
            "emit runtime audit-chain events",
            "deploy cloud I/O",
            "pub struct HrHealthResponse",
            "pub cloud_deployment: bool",
        ],
        "HR runtime adapter source boundary",
    )

    postgres_text = read_text(HR_POSTGRES_PATH, "HR Postgres/RLS storage source")
    require_terms(
        postgres_text,
        [
            "does not open database connections",
            "run migrations",
            "sensitive HR data",
            "emit audit-chain events",
            "deploy cloud I/O",
            "pub cloud_io_attached: bool",
        ],
        "HR Postgres/RLS storage source boundary",
    )

    deployment_ops = load_json(DEPLOYMENT_OPS_PATH, "deployment ops contract")
    authority = deployment_ops.get("deployment_authority", {})
    require(authority.get("primary") == "opentofu", "DeploymentOpsContract primary authority must stay opentofu")
    roots = authority.get("roots", [])
    require(any(row.get("path") == "infra/cloudflare" for row in roots if isinstance(row, dict)), "Cloudflare edge root must stay infra/cloudflare")
    cluster_fleet = deployment_ops.get("cluster_fleet", {})
    require("Cluster API + Talos + Argo CD" in str(cluster_fleet.get("model", "")), "cluster fleet model must stay CAPI/Talos/Argo CD")
    node_config = deployment_ops.get("node_config_serving_plane", {})
    require(node_config.get("status") == "specified_not_implemented", "node config serving plane must remain non-live")
    require(node_config.get("serving_endpoint", {}).get("live_endpoint_in_repo") is False, "node config live endpoint must not be in repo")
    require_contains_all(
        node_config.get("safe_local_validation", []),
        [
            "Fixture assignment registry rows can validate MAC/UUID uniqueness, required fields, revocation, and digest shape without live hardware.",
            "Fixture machineconfig files can validate digest/custody refs without containing real secrets.",
            "`gen-media.sh node --dry-run` remains a no-network command-shape check and does not claim a serving plane exists.",
        ],
        "DeploymentOpsContract safe local validation",
    )


def validate_fixture_paths(paths: object, label: str) -> None:
    require(isinstance(paths, list) and paths, f"{label} must be a non-empty list")
    for raw in paths:
        path = str(raw)
        require(path.startswith("evidence/cloud-deployment/hr/"), f"{label} must stay under HR cloud evidence root: {path}")
        require(".." not in Path(path).parts, f"{label} must not traverse directories: {path}")
        require(not path.endswith(".generated.json"), f"{label} must not hand-edit generated JSON: {path}")


def validate_manifest(manifest: dict[str, Any]) -> None:
    require(manifest.get("fixture_plan_id") == "HR-CLOUD-DEPLOYMENT-EVIDENCE-PLAN-RED-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_53c075b7", "manifest must bind kanban task t_53c075b7")
    require(manifest.get("service") == "hr", "manifest service must be hr")
    require(manifest.get("status") == "preview_deployment_evidence_red", "manifest status must stay preview_deployment_evidence_red")
    require_terms(
        manifest.get("claim_boundary", ""),
        ["no production deploy", "no GA", "no hyperscaler", "no live cloud deployment"],
        "manifest claim boundary",
    )
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_SOURCE_REFS, "source_authority_refs")
    require_all_false(manifest.get("runtime_claims"), RUNTIME_FALSE_FLAGS, "runtime_claims")

    topology = manifest.get("preview_deployment_topology", {})
    require(isinstance(topology, dict), "preview_deployment_topology must be an object")
    for key in ["gateway", "runtime", "storage", "tenant_rbac", "audit_chain", "observability", "rollout_controller"]:
        require(key in topology, f"preview_deployment_topology missing {key}")
    require_terms(topology, ["Gateway API", "Argo CD", "prebound", "Postgres/RLS", "Tenant/RBAC", "audit", "OpenTelemetry"], "topology")

    tenant = manifest.get("tenant_isolation", {})
    require(isinstance(tenant, dict), "tenant_isolation must be an object")
    require_terms(tenant, ["tenant namespace", "legal_entity_id", "RLS", "PDP", "NetworkPolicy", "ResourceQuota", "ServiceAccount"], "tenant isolation")
    require(tenant.get("cross_tenant_default") == "fail_closed", "tenant isolation must fail closed by default")

    rollback = manifest.get("rollback_observability_slo", {})
    require(isinstance(rollback, dict), "rollback_observability_slo must be an object")
    require(rollback.get("health_check_path") == "/hr/v1/healthz", "rollback/SLO health check must use HR healthz")
    require_terms(rollback, ["rollback", "OpenTelemetry", "SLO", "audit event", "N/A until live"], "rollback/observability/SLO")

    secrets = manifest.get("secret_handling", {})
    require(isinstance(secrets, dict), "secret_handling must be an object")
    require(secrets.get("raw_secret_material_in_repo") is False, "raw secret material must not be in repo")
    require(secrets.get("secret_values_in_fixture") is False, "fixture must not contain secret values")
    require_terms(secrets, ["OpenBao", "SecretProvider", "references only", "rotation"], "secret handling")

    local_live = manifest.get("local_vs_live_evidence", {})
    require(isinstance(local_live, dict), "local_vs_live_evidence must be an object")
    require(local_live.get("local_replay_available") is True, "local replay must be available")
    require(local_live.get("live_deployment_available") is False, "live deployment must not be claimed")
    require(local_live.get("production_rollout_required") is False, "production rollout must be explicitly N/A")
    require_terms(local_live, ["loopback", "JSON", "N/A", "no live cluster"], "local-vs-live evidence")

    fixtures = object_by_id(manifest.get("fixtures"), "fixture_id", EXPECTED_FIXTURE_IDS, "fixtures")
    for fixture_id, fixture in fixtures.items():
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must stay {EXPECTED_RED_STATUS}")
        validate_fixture_paths(fixture.get("future_evidence_refs"), f"{fixture_id}.future_evidence_refs")
        require_contains_all(fixture.get("source_authority_refs"), ["kanban:t_53c075b7#hr-cloud-deployment-plan-red"], f"{fixture_id}.source_authority_refs")

    require_terms(fixtures["hr_preview_deployment_topology_fixture"], ["Gateway API", "Argo CD", "prebound listener", "Postgres/RLS", "Tenant/RBAC", "OpenTelemetry"], "topology fixture")
    require_terms(fixtures["hr_tenant_isolation_fixture"], ["tenant namespace", "legal_entity_id", "RLS", "PDP", "NetworkPolicy", "ResourceQuota"], "tenant fixture")
    require_terms(fixtures["hr_rollback_observability_slo_fixture"], ["rollback", "healthz", "SLO", "OpenTelemetry", "audit event"], "rollback fixture")
    require_terms(fixtures["hr_secret_handling_fixture"], ["OpenBao", "SecretProvider", "no raw secret", "rotation"], "secret fixture")
    require_terms(fixtures["hr_local_replay_and_live_na_fixture"], ["loopback", "local replay", "live deployment N/A", "production rollout N/A"], "local/live fixture")
    require_terms(fixtures["hr_no_ga_cloud_claim_fixture"], ["no GA", "no hyperscaler", "no cloud deployment", "no Release Please", "generated JSON not touched"], "no-claim fixture")

    requirements = object_by_id(
        manifest.get("evidence_requirements"),
        "requirement_id",
        EXPECTED_EVIDENCE_REQUIREMENT_IDS,
        "evidence_requirements",
    )
    for requirement_id, requirement in requirements.items():
        require(requirement.get("status") == "red_until_build_evidence_attached", f"{requirement_id} must remain RED")
        require(requirement.get("runtime_evidence_attached") is False, f"{requirement_id} runtime evidence must be false")
        require(requirement.get("expected_evidence_ref", "").startswith("evidence/cloud-deployment/hr/"), f"{requirement_id} evidence ref must stay HR-scoped")
        require(not str(requirement.get("expected_evidence_ref", "")).endswith(".generated.json"), f"{requirement_id} evidence ref must not be generated JSON")
        official_url = str(requirement.get("official_doc_url", ""))
        require(official_url.startswith(ALLOWED_OFFICIAL_DOC_PREFIXES), f"{requirement_id} official_doc_url is not approved: {official_url}")

    splits = manifest.get("future_build_splits", [])
    require(isinstance(splits, list) and len(splits) >= 3, "future_build_splits must name at least three exact downstream build/review slices")
    for split in splits:
        require(isinstance(split, dict), "each future_build_splits row must be an object")
        require(split.get("title"), "future build split missing title")
        require(split.get("conflict_class"), f"future build split {split.get('title')} missing conflict_class")
        allowed_paths = split.get("allowed_paths")
        require(isinstance(allowed_paths, list) and allowed_paths, f"future build split {split.get('title')} missing allowed_paths")
        if any(str(path).startswith(("cloud/", "registry/", "specs/")) for path in allowed_paths):
            require("serialized" in str(split.get("conflict_class", "")), f"shared-root split {split.get('title')} must be serialized")
        for path in allowed_paths:
            raw_path = str(path)
            require(".." not in Path(raw_path).parts, f"future split path must not traverse dirs: {raw_path}")
            require(not raw_path.endswith(".generated.json"), f"future split must not hand-edit generated JSON: {raw_path}")
        require(split.get("acceptance"), f"future build split {split.get('title')} missing acceptance")
        require(split.get("verification"), f"future build split {split.get('title')} missing verification")
        review_lenses = split.get("review_lenses")
        require(
            isinstance(review_lenses, list) and review_lenses,
            f"future build split {split.get('title')} missing review_lenses",
        )


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_source_boundaries()
    validate_manifest(live_manifest)

    def expect_rejected(label: str, mutator: Callable[[dict[str, Any]], None]) -> None:
        candidate = copy.deepcopy(live_manifest)
        mutator(candidate)
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                validate_manifest(candidate)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected("missing fixture", lambda data: data["fixtures"].pop())
    expect_rejected("missing evidence requirement", lambda data: data["evidence_requirements"].pop())
    expect_rejected("production deployment claim", lambda data: data["runtime_claims"].update({"productionDeploymentClaimed": True}))
    expect_rejected("live deployment claim", lambda data: data["local_vs_live_evidence"].update({"live_deployment_available": True}))
    expect_rejected("raw secret material", lambda data: data["secret_handling"].update({"raw_secret_material_in_repo": True}))
    expect_rejected("generated evidence ref", lambda data: data["fixtures"][0].update({"future_evidence_refs": ["evidence/cloud-deployment/hr/bad.generated.json"]}))
    expect_rejected("missing serialized shared-root conflict", lambda data: data["future_build_splits"][2].update({"conflict_class": "product-vertical:erp/hr cloud deployment"}))
    print("HR cloud deployment evidence plan self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST), help="HR cloud deployment evidence plan JSON path")
    parser.add_argument("--self-test", action="store_true", help="run fail-closed validator self-tests")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    manifest_path = Path(args.manifest)
    if not manifest_path.is_absolute():
        manifest_path = REPO_ROOT / manifest_path
    manifest = load_json(manifest_path, "HR cloud deployment evidence plan")
    if args.self_test:
        run_self_tests(manifest)
        return
    validate_source_boundaries()
    validate_manifest(manifest)
    print("HR cloud deployment evidence plan check passed")


if __name__ == "__main__":
    main()
