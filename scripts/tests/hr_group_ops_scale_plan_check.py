#!/usr/bin/env python3
"""Validate the HR high-volume group operations Plan/Spec/RED fixture.

This guard binds t_8d0ae2f3 to the HR PRD GA scale gap without claiming GA,
production load testing, live traffic, UI/native implementation, or runtime
deployment. The fixture may describe future performance/concurrency evidence,
but every runtime/live attachment stays false until a later reviewed Build card
provides source-backed evidence.
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
    / "hr-group-ops-scale"
    / "high-volume-group-hr-plan.json"
)
HR_PRD_PATH = REPO_ROOT / "specs" / "microservices" / "hr.json"
HR_DOMAIN_PATH = REPO_ROOT / "oya" / "hr" / "crates" / "oya-hr-employment-domain" / "src" / "lib.rs"
HR_APP_PRIVACY_TEST_PATH = (
    REPO_ROOT / "oya" / "hr" / "crates" / "oya-hr-employment-app" / "tests" / "privacy.rs"
)
HR_APP_LEAVE_TEST_PATH = (
    REPO_ROOT / "oya" / "hr" / "crates" / "oya-hr-employment-app" / "tests" / "leave.rs"
)
HR_POSTGRES_TEST_PATH = (
    REPO_ROOT
    / "oya"
    / "hr"
    / "crates"
    / "oya-hr-employment-storage-adapter-postgres"
    / "tests"
    / "storage.rs"
)
HR_CLOUD_DEPLOYMENT_FIXTURE_PATH = (
    REPO_ROOT
    / "specs"
    / "fixtures"
    / "hr-cloud-deployment"
    / "preview-deployment-evidence-plan.json"
)

EXPECTED_SCENARIO_IDS = [
    "multi_entity_employee_lifecycle_burst",
    "leave_attendance_payroll_bridge_load",
    "sensitive_read_policy_storm",
    "compliance_threshold_crossing_batch",
    "cross_tenant_negative_replay_load",
    "rollback_observability_slo_saturation_drill",
]

EXPECTED_EVIDENCE_REQUIREMENT_IDS = [
    "synthetic-workload-manifest",
    "fixture-data-provenance",
    "deterministic-policy-replay",
    "tenant-legal-entity-isolation-negative-replay",
    "rls-plan-and-idempotency-proof",
    "sensitive-read-audit-coverage",
    "payroll-accuracy-sample",
    "compliance-threshold-coverage",
    "concurrency-conflict-retry-profile",
    "slo-latency-throughput-budget",
    "saturation-backpressure-profile",
    "rollback-plan-and-abort-criteria",
    "observability-trace-metric-log-correlation",
    "no-live-production-load-confirmation",
]

REQUIRED_SOURCE_REFS = {
    "specs/microservices/hr.json#scope.in_scope_per_wave.GA[high-volume group HR operations]",
    "specs/microservices/hr.json#acceptance_criteria[AC-01]",
    "specs/microservices/hr.json#acceptance_criteria[AC-02]",
    "specs/microservices/hr.json#acceptance_criteria[AC-03]",
    "specs/microservices/hr.json#acceptance_criteria[AC-04]",
    "specs/microservices/hr.json#acceptance_criteria[AC-05]",
    "specs/microservices/hr.json#acceptance_criteria[AC-09]",
    "specs/microservices/hr.json#acceptance_criteria[AC-12]",
    "specs/microservices/hr.json#acceptance_criteria[AC-13]",
    "specs/microservices/hr.json#metrics.employee_lifecycle_policy_bypass_count",
    "specs/microservices/hr.json#metrics.kr_obligation_trigger_coverage_pct",
    "specs/microservices/hr.json#metrics.attendance_to_payroll_derivation_accuracy_pct",
    "specs/microservices/hr.json#metrics.sensitive_hr_read_audit_coverage_pct",
    "specs/microservices/hr.json#risks[R-01]",
    "specs/microservices/hr.json#risks[R-02]",
    "specs/microservices/hr.json#risks[R-03]",
    "oya/hr/crates/oya-hr-employment-domain/src/lib.rs#EmployeeCreate",
    "oya/hr/crates/oya-hr-employment-domain/src/lib.rs#LegalEntityWorkforceSnapshot",
    "oya/hr/crates/oya-hr-employment-domain/src/lib.rs#LeavePayrollImpactInput",
    "oya/hr/crates/oya-hr-employment-domain/src/lib.rs#SensitiveHrReadInput",
    "oya/hr/crates/oya-hr-employment-storage-adapter-postgres/tests/storage.rs#postgres_contract_declares_rls_and_rollback_without_runtime_overclaims",
    "specs/fixtures/hr-cloud-deployment/preview-deployment-evidence-plan.json#rollback_observability_slo",
    "kanban:t_9daca2db#durable-storage-rls-seam",
    "kanban:t_53c075b7#preview-cloud-evidence-topology",
    "kanban:t_c506abc9#hr-user-story-e2e-evidence-plan",
    "kanban:t_8d0ae2f3#high-volume-group-hr-plan-red",
}

RUNTIME_FALSE_FLAGS = [
    "productionLoadTestClaimed",
    "liveTrafficClaimed",
    "gaClaimed",
    "hyperscalerClaimed",
    "uiNativeImplementationClaimed",
    "deployedListenerAttached",
    "runtimeDatabaseAttached",
    "workflowExecutionAttached",
    "payrollCalculationAttached",
    "sensitiveDataFetchAttached",
    "runtimeAuditEmissionAttached",
    "cloudDeploymentEvidenceAttached",
    "generatedJsonHandEdited",
    "releasePleaseAssumed",
]

EXPECTED_RED_STATUS = "RED_UNTIL_PERFORMANCE_BUILD_EVIDENCE_ATTACHED"
EVIDENCE_REF_PREFIX = "evidence/performance/hr-group-ops-scale/"


def fail(message: str) -> NoReturn:
    print(f"HR group ops scale plan check failed: {message}", file=sys.stderr)
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


def validate_hr_prd(prd: dict[str, Any]) -> None:
    require(prd.get("_meta", {}).get("spec_id") == "PRD-HR", "HR PRD spec_id must be PRD-HR")
    scope = prd.get("scope", {}).get("in_scope_per_wave", {})
    require("high-volume group HR operations" in scope.get("GA", []), "HR GA scope must retain high-volume group HR operations")

    acceptance_rows = prd.get("acceptance_criteria")
    require(isinstance(acceptance_rows, list), "HR PRD acceptance_criteria must be a list")
    ac_by_id = {row.get("id"): row for row in acceptance_rows if isinstance(row, dict)}
    for ac_id in ["AC-01", "AC-02", "AC-03", "AC-04", "AC-05", "AC-09", "AC-12", "AC-13"]:
        require(ac_id in ac_by_id, f"HR PRD missing {ac_id}")
    require_terms(ac_by_id["AC-01"], ["tenant_id", "legal_entity_id", "manager_id", "audit evidence"], "AC-01 lifecycle metadata")
    require_terms(ac_by_id["AC-02"], ["10 employees", "Workflow", "MoelFiled", "Active evidence"], "AC-02 KR 10 threshold")
    require_terms(ac_by_id["AC-03"], ["30 employees", "council roster", "meeting cadence", "minutes evidence"], "AC-03 KR 30 threshold")
    require_terms(ac_by_id["AC-04"], ["leave", "attendance", "manager", "payroll-impact evidence"], "AC-04 leave/attendance payroll evidence")
    require_terms(ac_by_id["AC-05"], ["purpose-bound access", "legal basis", "sensitive-read logging"], "AC-05 sensitive-read evidence")
    require_terms(ac_by_id["AC-09"], ["oya-ci-required", "Buck2", "OpenAPI semver", "evidence emission"], "AC-09 promotion evidence")
    require_terms(ac_by_id["AC-12"], ["bounded body limits", "storage", "Workflow execution", "Payroll calls", "runtime audit emission"], "AC-12 runtime non-claims")
    require_terms(ac_by_id["AC-13"], ["tenant", "legal-entity", "idempotency", "Postgres/RLS"], "AC-13 durable storage proof")

    metrics = prd.get("metrics")
    require(isinstance(metrics, list), "HR PRD metrics must be a list")
    metric_by_name = {row.get("name"): row for row in metrics if isinstance(row, dict)}
    expected_targets = {
        "employee_lifecycle_policy_bypass_count": 0,
        "kr_obligation_trigger_coverage_pct": 100,
        "attendance_to_payroll_derivation_accuracy_pct": 99.99,
        "sensitive_hr_read_audit_coverage_pct": 100,
    }
    for name, target in expected_targets.items():
        row = metric_by_name.get(name)
        require(isinstance(row, dict), f"HR PRD missing metric {name}")
        require(row.get("targets", {}).get("GA") == target, f"HR PRD metric {name} GA target must be {target}")

    risks = prd.get("risks")
    require(isinstance(risks, list), "HR PRD risks must be a list")
    risk_by_id = {row.get("id"): row for row in risks if isinstance(row, dict)}
    require_terms(risk_by_id.get("R-01", {}), ["labor-law threshold", "TenantTier", "legal-entity"], "R-01 threshold risk")
    require_terms(risk_by_id.get("R-02", {}), ["sensitive HR data", "purpose-bound", "audit"], "R-02 sensitive read risk")
    require_terms(risk_by_id.get("R-03", {}), ["attendance", "payroll accuracy"], "R-03 attendance/payroll risk")

    decisions = prd.get("decision_log")
    require(isinstance(decisions, list), "HR PRD decision_log must be a list")
    decision_by_id = {row.get("id"): row for row in decisions if isinstance(row, dict)}
    for decision_id in ["D-06", "D-09", "D-10", "D-11", "D-14", "D-15", "D-16"]:
        require(decision_id in decision_by_id, f"HR PRD missing decision {decision_id}")
    require_terms(decision_by_id["D-06"], ["metadata-only", "runtime dispatch", "persistence", "cloud adapters"], "D-06 app boundary")
    require_terms(decision_by_id["D-09"], ["oya-ci-required", "Buck2-backed", "cargo-nextest-only", "transitional"], "D-09 promotion boundary")
    require_terms(decision_by_id["D-10"], ["leave", "metadata-only", "payroll calculation", "storage"], "D-10 leave boundary")
    require_terms(decision_by_id["D-11"], ["sensitive-read", "metadata-only", "data retrieval", "audit-chain emission"], "D-11 sensitive boundary")
    require_terms(decision_by_id["D-14"], ["does not deploy a listener", "persist state", "execute Workflow", "call Payroll"], "D-14 runtime boundary")
    require_terms(decision_by_id["D-15"], ["volatile", "idempotency", "durable storage", "audit-chain emission"], "D-15 storage boundary")
    require_terms(decision_by_id["D-16"], ["cloud deployment", "production statutory correctness", "runtime audit-chain emission"], "D-16 rulepack boundary")


def validate_source_boundaries() -> None:
    validate_hr_prd(load_json(HR_PRD_PATH, "HR PRD"))

    domain_text = read_text(HR_DOMAIN_PATH, "HR domain source")
    require_terms(
        domain_text,
        [
            "pub struct EmployeeCreate",
            "pub struct LegalEntityWorkforceSnapshot",
            "pub struct LeavePayrollImpactInput",
            "pub struct SensitiveHrReadInput",
            "pub fn evaluate_labor_compliance",
            "tenant_id",
            "legal_entity_id",
            "idempotency_key",
            "does not perform storage, workflow dispatch, payroll",
        ],
        "HR domain source scale foundations",
    )

    privacy_test_text = read_text(HR_APP_PRIVACY_TEST_PATH, "HR app privacy tests")
    require_terms(
        privacy_test_text,
        [
            "sensitive_read_runtime_boundary_fails_closed_without_tenant_rbac_scope",
            "MissingTenantRbacScopeEvidence",
            "MissingSensitiveReadAuditContract",
            "sensitive_data_fetch",
            "raw_sensitive_data_echo",
        ],
        "HR app privacy runtime boundary tests",
    )

    leave_test_text = read_text(HR_APP_LEAVE_TEST_PATH, "HR app leave tests")
    require_terms(
        leave_test_text,
        [
            "leave_payroll_impact_envelope_is_metadata_only",
            "integration.hr.payroll.leave-impact",
            "payroll_calculation_attached",
            "payroll_network_call",
            "workflow_execution",
            "storage_attached",
        ],
        "HR app leave/payroll boundary tests",
    )

    postgres_test_text = read_text(HR_POSTGRES_TEST_PATH, "HR Postgres/RLS storage tests")
    require_terms(
        postgres_test_text,
        [
            "postgres_contract_declares_rls_and_rollback_without_runtime_overclaims",
            "ENABLE ROW LEVEL SECURITY",
            "current_setting('oyatie.tenant_id', true)",
            "UNIQUE (tenant_id, legal_entity_id, idempotency_key)",
            "TenantMismatch",
            "UnsafeMetadata",
        ],
        "HR Postgres/RLS scale boundary tests",
    )

    cloud_fixture = load_json(HR_CLOUD_DEPLOYMENT_FIXTURE_PATH, "HR cloud deployment fixture")
    require(cloud_fixture.get("kanban_task") == "t_53c075b7", "HR cloud fixture must bind t_53c075b7")
    require_all_false(
        cloud_fixture.get("runtime_claims"),
        ["productionDeploymentClaimed", "gaClaimed", "runtimeCloudIoAttached"],
        "HR cloud runtime_claims",
    )
    require_terms(cloud_fixture.get("rollback_observability_slo", {}), ["rollback", "OpenTelemetry", "SLO", "N/A until live"], "HR cloud rollback/SLO source")


def validate_evidence_paths(paths: object, label: str) -> None:
    require(isinstance(paths, list) and paths, f"{label} must be a non-empty list")
    for raw in paths:
        path = str(raw)
        require(path.startswith(EVIDENCE_REF_PREFIX), f"{label} must stay under {EVIDENCE_REF_PREFIX}: {path}")
        require(".." not in Path(path).parts, f"{label} must not traverse directories: {path}")
        require(not path.endswith(".generated.json"), f"{label} must not hand-edit generated JSON: {path}")


def validate_load_profile(profile: object, scenario_id: str) -> None:
    require(isinstance(profile, dict), f"{scenario_id}.load_profile must be an object")
    require(int(profile.get("tenants", 0)) >= 2, f"{scenario_id}.load_profile must cover multiple tenants")
    require(int(profile.get("legal_entities_per_tenant", 0)) >= 2, f"{scenario_id}.load_profile must cover multiple legal entities")
    require(int(profile.get("employees_per_legal_entity", 0)) >= 30, f"{scenario_id}.load_profile must reach KR 30-employee council threshold")
    require(int(profile.get("operations", 0)) >= 1000, f"{scenario_id}.load_profile must model high-volume operations")
    require(str(profile.get("production_load", "")).lower() == "false", f"{scenario_id}.load_profile.production_load must be false")


def validate_manifest(manifest: dict[str, Any]) -> None:
    require(manifest.get("fixture_plan_id") == "HR-GROUP-OPS-SCALE-PLAN-RED-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_8d0ae2f3", "manifest must bind kanban task t_8d0ae2f3")
    require(manifest.get("service") == "hr", "manifest service must be hr")
    require(manifest.get("status") == "high_volume_group_hr_scale_red", "manifest status must stay high_volume_group_hr_scale_red")
    require_terms(
        manifest.get("claim_boundary", ""),
        ["Plan/Spec/RED", "no production load", "no GA", "no live traffic", "no UI/native"],
        "manifest claim boundary",
    )
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_SOURCE_REFS, "source_authority_refs")
    require_all_false(manifest.get("runtime_claims"), RUNTIME_FALSE_FLAGS, "runtime_claims")

    determinism = manifest.get("policy_determinism", {})
    require(isinstance(determinism, dict), "policy_determinism must be an object")
    require(determinism.get("repeated_replay_required") is True, "policy_determinism.repeated_replay_required must be true")
    require(determinism.get("wall_clock_randomness_allowed") is False, "policy_determinism.wall_clock_randomness_allowed must be false")
    require_terms(determinism, ["same input", "same output", "idempotency", "stable ordering", "decision hash"], "policy determinism")

    isolation = manifest.get("tenant_legal_entity_isolation", {})
    require(isinstance(isolation, dict), "tenant_legal_entity_isolation must be an object")
    require(isolation.get("cross_tenant_default") == "fail_closed", "tenant isolation must fail closed")
    require_terms(isolation, ["tenant_id", "legal_entity_id", "RLS", "PDP", "negative replay", "no aggregate leakage"], "tenant/legal-entity isolation")

    scenarios = object_by_id(manifest.get("scale_scenarios"), "scenario_id", EXPECTED_SCENARIO_IDS, "scale_scenarios")
    scenario_terms = {
        "multi_entity_employee_lifecycle_burst": ["employee lifecycle", "manager", "tenant_id", "legal_entity_id", "audit"],
        "leave_attendance_payroll_bridge_load": ["leave", "attendance", "payroll-impact", "payroll accuracy", "metadata-only"],
        "sensitive_read_policy_storm": ["sensitive-read", "purpose", "legal basis", "audit", "no sensitive data fetch"],
        "compliance_threshold_crossing_batch": ["10", "30", "Workflow", "rulepack", "KR"],
        "cross_tenant_negative_replay_load": ["cross-tenant", "fail closed", "RLS", "PDP", "legal_entity_id"],
        "rollback_observability_slo_saturation_drill": ["SLO", "rollback", "OpenTelemetry", "saturation", "abort"],
    }
    for scenario_id, scenario in scenarios.items():
        require(scenario.get("expected_red_status") == EXPECTED_RED_STATUS, f"{scenario_id} must stay {EXPECTED_RED_STATUS}")
        validate_load_profile(scenario.get("load_profile"), scenario_id)
        validate_evidence_paths(scenario.get("future_evidence_refs"), f"{scenario_id}.future_evidence_refs")
        require_contains_all(scenario.get("source_authority_refs"), ["kanban:t_8d0ae2f3#high-volume-group-hr-plan-red"], f"{scenario_id}.source_authority_refs")
        require_terms(scenario, scenario_terms[scenario_id], f"{scenario_id} scenario")

    requirements = object_by_id(
        manifest.get("evidence_requirements"),
        "requirement_id",
        EXPECTED_EVIDENCE_REQUIREMENT_IDS,
        "evidence_requirements",
    )
    for requirement_id, requirement in requirements.items():
        require(requirement.get("status") == "red_until_build_evidence_attached", f"{requirement_id} must remain RED")
        require(requirement.get("runtime_evidence_attached") is False, f"{requirement_id} runtime evidence must be false")
        expected_ref = str(requirement.get("expected_evidence_ref", ""))
        require(expected_ref.startswith(EVIDENCE_REF_PREFIX), f"{requirement_id} evidence ref must stay HR performance-scoped")
        require(not expected_ref.endswith(".generated.json"), f"{requirement_id} evidence ref must not be generated JSON")

    rollout = manifest.get("rollout_rollback_observability", {})
    require(isinstance(rollout, dict), "rollout_rollback_observability must be an object")
    require(rollout.get("production_rollout_required") is False, "production rollout must be explicitly N/A")
    require_terms(rollout, ["rollback", "OpenTelemetry", "SLO", "burn-rate", "N/A until live"], "rollout/rollback/observability")

    splits = manifest.get("future_build_splits", [])
    require(isinstance(splits, list) and len(splits) >= 4, "future_build_splits must name at least four exact downstream build/review slices")
    for split in splits:
        require(isinstance(split, dict), "each future_build_splits row must be an object")
        title = str(split.get("title", ""))
        require(title, "future build split missing title")
        conflict_class = str(split.get("conflict_class", ""))
        require(conflict_class, f"future build split {title} missing conflict_class")
        allowed_paths = split.get("allowed_paths")
        require(isinstance(allowed_paths, list) and allowed_paths, f"future build split {title} missing allowed_paths")
        if any(str(path).startswith(("cloud/", "registry/", "specs/", "scripts/", "Cargo.toml", "Cargo.lock")) for path in allowed_paths):
            require("serialized" in conflict_class, f"shared-root split {title} must be serialized")
        for path in allowed_paths:
            raw_path = str(path)
            require(".." not in Path(raw_path).parts, f"future split path must not traverse dirs: {raw_path}")
            require(not raw_path.endswith(".generated.json"), f"future split must not hand-edit generated JSON: {raw_path}")
        require(split.get("acceptance"), f"future build split {title} missing acceptance")
        require(split.get("verification"), f"future build split {title} missing verification")
        review_lenses = split.get("review_lenses")
        require(isinstance(review_lenses, list) and review_lenses, f"future build split {title} missing review_lenses")

    handoff = manifest.get("review_handoff", {})
    require(isinstance(handoff, dict), "review_handoff must be an object")
    require_contains_all(handoff.get("required_lenses"), ["performance", "privacy", "data-integrity", "SRE"], "review_handoff.required_lenses")
    require_contains_all(
        handoff.get("local_verification"),
        [
            "python3 scripts/tests/hr_group_ops_scale_plan_check.py",
            "python3 scripts/tests/hr_group_ops_scale_plan_check.py --self-test",
            "python3 -m json.tool specs/fixtures/hr-group-ops-scale/high-volume-group-hr-plan.json",
        ],
        "review_handoff.local_verification",
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

    expect_rejected("missing scenario", lambda data: data["scale_scenarios"].pop())
    expect_rejected("production load claim", lambda data: data["runtime_claims"].update({"productionLoadTestClaimed": True}))
    expect_rejected("UI/native claim", lambda data: data["runtime_claims"].update({"uiNativeImplementationClaimed": True}))
    expect_rejected("non-deterministic policy", lambda data: data["policy_determinism"].update({"wall_clock_randomness_allowed": True}))
    expect_rejected("cross tenant allowed", lambda data: data["tenant_legal_entity_isolation"].update({"cross_tenant_default": "allow"}))
    expect_rejected("generated evidence ref", lambda data: data["scale_scenarios"][0].update({"future_evidence_refs": [f"{EVIDENCE_REF_PREFIX}bad.generated.json"]}))
    expect_rejected("missing evidence requirement", lambda data: data["evidence_requirements"].pop())
    expect_rejected("missing serialized shared-root conflict", lambda data: data["future_build_splits"][3].update({"conflict_class": "product-vertical:erp/hr performance evidence"}))
    print("HR group ops scale plan self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST), help="HR group ops scale Plan/Spec/RED JSON path")
    parser.add_argument("--self-test", action="store_true", help="run fail-closed validator self-tests")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    manifest_path = Path(args.manifest)
    if not manifest_path.is_absolute():
        manifest_path = REPO_ROOT / manifest_path
    manifest = load_json(manifest_path, "HR group ops scale plan")
    if args.self_test:
        run_self_tests(manifest)
        return
    validate_source_boundaries()
    validate_manifest(manifest)
    print("HR group ops scale plan check passed")


if __name__ == "__main__":
    main()
