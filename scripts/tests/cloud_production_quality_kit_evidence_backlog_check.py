#!/usr/bin/env python3
"""Validate machine-checkable production-quality kit backlog/evidence records."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Callable, NoReturn
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "specs" / "cloud-production-quality-kit-evidence-backlog.json"
TARGET_PATH = REPO_ROOT / "specs" / "cloud-production-quality-kits-target.json"
OBSERVABILITY_EVIDENCE_PATH = REPO_ROOT / "specs" / "cloud-observability-slo-evidence-contract.json"
HYPERSCALER_GATES_PATH = REPO_ROOT / "specs" / "hyperscaler-gates.json"

REQUIRED_OBJECTIVE_DOMAINS = {
    "overload_fairness",
    "shuffle_sharding_cell_isolation",
    "failover_dr",
    "progressive_delivery",
    "privacy_residency",
    "cost_finops",
    "abuse_threat_scenarios",
}
REQUIRED_TARGET_DOMAIN = "k8s_pod_security"
REQUIRED_ALL_DOMAINS = REQUIRED_OBJECTIVE_DOMAINS | {REQUIRED_TARGET_DOMAIN}
REQUIRED_EVIDENCE_FIELDS = {
    "kit_id",
    "scenario_id",
    "run_id",
    "dogfood_environment",
    "command",
    "status",
    "artifact_digest",
    "reviewer",
    "created_at",
    "source_commit",
    "evidence_window",
    "result_summary",
}
REQUIRED_BLOCKED_CLAIMS = {
    "no_green_runtime_evidence",
    "no_production_readiness_claim",
    "no_public_sla_slo_claim",
    "no_tenant_workload_claim",
    "no_hyperscaler_maturity_claim",
    "no_external_saas_or_public_cloud_fallback",
}
OFFICIAL_SOURCE_HOST_SUFFIXES = {
    "aws.amazon.com",
    "docs.aws.amazon.com",
    "sre.google",
    "nist.gov",
    "focus.finops.org",
    "kubernetes.io",
    "owasp.org",
    "cheatsheetseries.owasp.org",
}
EXPECTED_SOURCE_URL_BY_DOMAIN = {
    "overload_fairness": "https://aws.amazon.com/builders-library/using-load-shedding-to-avoid-overload/",
    "shuffle_sharding_cell_isolation": "https://aws.amazon.com/builders-library/workload-isolation-using-shuffle-sharding/",
    "failover_dr": "https://docs.aws.amazon.com/wellarchitected/latest/framework/rel_planning_for_recovery_disaster_recovery.html",
    "progressive_delivery": "https://sre.google/workbook/canarying-releases/",
    "privacy_residency": "https://www.nist.gov/privacy-framework/privacy-framework",
    "cost_finops": "https://focus.finops.org/focus-specification/v1-3/",
    "k8s_pod_security": "https://kubernetes.io/docs/concepts/security/pod-security-standards/",
    "abuse_threat_scenarios": "https://cheatsheetseries.owasp.org/cheatsheets/Bot_Management_and_Anti-Automation_Cheat_Sheet.html",
}
REQUIRED_KIT_EVIDENCE_OUTPUTS = {
    "QK-01-overload-fairness": {"shed_rate_curve", "fairness_index", "cascading_failure_check"},
    "QK-02-shuffle-shard-isolation": {"correlated_impact_probability_matrix", "blast_radius_bound", "noisy_neighbor_isolation_drill"},
    "QK-03-privacy-data-governance": {"data_flow_map", "dsr_delete_export_proof", "residency_enforcement_test", "telemetry_redaction_check"},
    "QK-04-canary-prr": {
        "canary_eval_report",
        "prr_signoff",
        "rollback_drill_receipt",
        "backup_restore_drill_receipt",
        "rto_rpo_restore_drill_receipt",
        "cell_failover_drill_receipt",
        "dependency_failure_recovery_receipt",
    },
    "QK-05-focus-cost-export": {"focus_schema_validation", "cost_attribution_reconciliation", "invoice_reconciliation"},
    "QK-06-k8s-pod-security": {"admission_policy_test_results", "restricted_profile_receipt", "privileged_exception_register"},
    "QK-07-abuse-fraud-ddos": {"abuse_drill_results", "ingress_threshold_report", "suspension_appeals_round_trip"},
}
REQUIRED_QK04_DR_RECEIPTS = {
    "backup_restore_drill_receipt",
    "rto_rpo_restore_drill_receipt",
    "cell_failover_drill_receipt",
    "dependency_failure_recovery_receipt",
}
REQUIRED_QK04_DR_SCENARIO_OUTPUTS = {
    "QK-04-canary-prr-DR01": {"backup_restore_drill_receipt", "rto_rpo_restore_drill_receipt"},
    "QK-04-canary-prr-DR02": {"cell_failover_drill_receipt"},
    "QK-04-canary-prr-DR03": {"rto_rpo_restore_drill_receipt"},
    "QK-04-canary-prr-DR04": {"dependency_failure_recovery_receipt"},
}
REQUIRED_RESULT_SUMMARY_KEYS = {
    "output_key",
    "expected_value_or_threshold",
    "observed_value",
    "artifact_ref",
    "evaluation_status",
}
FORBIDDEN_POSITIVE_PATTERNS = [
    re.compile(pattern)
    for pattern in [
        r"\bproduction\s+ready\b",
        r"\bprod\s+ready\b",
        r"\b(production|prod)\s+(is\s+)?(grade|capable|viable|suitable)\b",
        r"\bga\s+ready\b",
        r"\bgenerally\s+available\b",
        r"\btenant\s+workloads?\b.{0,40}\b(can\s+run|ready|safe|supported|enabled)\b",
        r"\btenant\s+workloads?\b.{0,40}\b(operate|operates|run|runs)\s+safely\b",
        r"\bcustomer\s+workloads?\b.{0,40}\b(can\s+run|ready|safe|supported|enabled)\b",
        r"\bcustomer\s+workloads?\b.{0,40}\b(operate|operates|run|runs)\s+safely\b",
        r"\bpublic\s+(sla|slo|service\s*level\s*agreement)\b.{0,40}\b(ready|available|exists?|enabled|published|achieved)\b",
        r"\bavailability\b.{0,40}\b(achieved|met|ready|green|passed|published)\b",
        r"\b(hyperscaler|hyperscale)\b.{0,30}\b(mature|maturity|grade|ready|readiness|established|achieved)\b",
        r"\b(production|prod)\s+readiness\b.{0,40}\b(achieved|established|ready|complete|met)\b",
        r"\b(sla|slo)\b.{0,40}\b(achieved|met|ready|available|green|passed|published|fulfilled|satisfied)\b",
        r"\btenant\s+workload\s+readiness\b.{0,40}\b(achieved|established|ready|met)\b",
        r"\bhyperscaler\s+quality\s+bar\b.{0,40}\b(met|achieved|passed|green)\b",
        r"\bruntime\b.{0,20}\bis\s+green\b",
        r"\b(harness|drill|runtime\s+evidence|gate)\b.{0,30}\b(implemented|green|passed|available|live)\b",
        r"\bgreen\s+(runtime\s+)?evidence\b",
        r"\bexternal\s+(saas|public\s+cloud)\s+fallback\b",
        r"\bgithub\s+actions\s+fallback\b",
    ]
]


def fail(message: str) -> NoReturn:
    print(f"cloud production-quality kit evidence backlog check failed: {message}", file=sys.stderr)
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


def contains_forbidden_positive(value: object) -> bool:
    haystack = f" {normalized(value)} "
    return any(pattern.search(haystack) for pattern in FORBIDDEN_POSITIVE_PATTERNS)


def source_host_allowed(url: str) -> bool:
    parsed = urlparse(url)
    require(parsed.scheme == "https", f"official source URL must use https: {url}")
    host = parsed.netloc.lower()
    return any(host == suffix or host.endswith(f".{suffix}") for suffix in OFFICIAL_SOURCE_HOST_SUFFIXES)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path.relative_to(REPO_ROOT)}: {exc}")


def sanitized_for_positive_claim_scan(spec: dict) -> dict:
    candidate = json.loads(json.dumps(spec))
    candidate.get("claim_controls", {}).pop("cannot_claim_yet", None)
    candidate.get("claim_controls", {}).pop("blocked_claim_families", None)
    candidate.pop("nonclaims", None)
    for kit in candidate.get("kit_backlog", []):
        kit.pop("blocked_claim_families", None)
        kit.get("evidence_gate", {}).pop("claim_status", None)
        kit.get("evidence_gate", {}).pop("required_before_green", None)
        kit.get("evidence_gate", {}).pop("not_green_reason", None)
    for domain in candidate.get("objective_domain_coverage", {}).values():
        if isinstance(domain, dict):
            domain.pop("claim_status", None)
    return candidate


def validate(spec: dict) -> None:
    target = load_json(TARGET_PATH)
    observability = load_json(OBSERVABILITY_EVIDENCE_PATH)
    gates = load_json(HYPERSCALER_GATES_PATH)

    for field in [
        "spec_id",
        "title",
        "status",
        "source_quality_kits_target",
        "source_observability_slo_evidence",
        "source_hyperscaler_gates",
        "purpose",
        "claim_controls",
        "official_source_evidence",
        "kit_backlog",
        "objective_domain_coverage",
        "machine_check_surfaces",
        "nonclaims",
        "next_goal_links",
    ]:
        require(field in spec, f"missing top-level field {field!r}")

    require(spec["spec_id"] == "EXE-CLOUD-PRODUCTION-QUALITY-KIT-EVIDENCE-BACKLOG", "unexpected spec_id")
    require(spec["status"] == "Proposed-target", "status must remain Proposed-target")
    require(spec["source_quality_kits_target"] == str(TARGET_PATH.relative_to(REPO_ROOT)), "source_quality_kits_target must point to target kit spec")
    require(spec["source_observability_slo_evidence"] == str(OBSERVABILITY_EVIDENCE_PATH.relative_to(REPO_ROOT)), "source_observability_slo_evidence must point to G005")
    require(spec["source_hyperscaler_gates"] == str(HYPERSCALER_GATES_PATH.relative_to(REPO_ROOT)), "source_hyperscaler_gates must point to hyperscaler gates")

    controls = spec["claim_controls"]
    for key in [
        "machine_checkable_backlog_only",
        "evidence_required",
        "strict_separation",
        "pure_dogfood",
        "no_green_runtime_evidence",
        "no_production_readiness_claim",
        "no_public_sla_slo_claim",
        "no_tenant_workload_readiness",
        "no_hyperscaler_maturity_claim",
        "no_external_saas_or_public_cloud_fallback",
    ]:
        require(controls.get(key) is True, f"claim_controls.{key} must be true")
    require(set(controls.get("blocked_claim_families", [])) >= REQUIRED_BLOCKED_CLAIMS, "claim_controls missing blocked claim families")
    require(not contains_forbidden_positive(controls.get("can_claim_now", [])), "can_claim_now contains forbidden positive claim")
    require(not contains_forbidden_positive(sanitized_for_positive_claim_scan(spec)), "spec contains forbidden positive claim wording outside blocked/nonclaim fields")

    source_rows = spec["official_source_evidence"]
    require(isinstance(source_rows, list) and len(source_rows) == len(REQUIRED_ALL_DOMAINS), "official_source_evidence must include exactly one official/upstream evidence row for every kit family")
    source_domains = {row.get("domain") for row in source_rows}
    require(source_domains == REQUIRED_ALL_DOMAINS, "official_source_evidence must exactly cover required objective domains")
    source_by_domain = {row.get("domain"): row for row in source_rows}
    for domain, expected_url in EXPECTED_SOURCE_URL_BY_DOMAIN.items():
        require(domain in source_by_domain, f"official_source_evidence missing {domain}")
        row = source_by_domain[domain]
        require(row.get("url") == expected_url, f"{domain}: expected source URL {expected_url}, got {row.get('url')}")
        require(row.get("url") and source_host_allowed(row["url"]), f"official source URL is not allowlisted: {row.get('url')}")
        require(row.get("source_status") in {"official", "upstream"}, f"{domain}: source_status must be official/upstream")
        require(row.get("title"), f"{domain}: source title required")

    target_kits = {kit["id"]: kit for kit in target["kits"]}
    require(len(target_kits) == 7, "target quality kit spec must define seven kits")
    rows = spec["kit_backlog"]
    require(isinstance(rows, list) and rows, "kit_backlog must be a non-empty list")
    row_ids = {row.get("kit_id") for row in rows}
    require(len(rows) == len(row_ids), "kit_backlog contains duplicate kit_id rows")
    require(len(rows) == len(target_kits), "kit_backlog row count must match target kit count")
    require(set(target_kits) == row_ids, f"kit_backlog must exactly cover target kits; missing {sorted(set(target_kits)-row_ids)} extra {sorted(row_ids-set(target_kits))}")

    domains_from_rows: dict[str, set[str]] = {}
    for candidate_row in rows:
        for domain in candidate_row.get("objective_domains", []):
            domains_from_rows.setdefault(domain, set()).add(candidate_row.get("kit_id"))
    require(set(domains_from_rows) == REQUIRED_ALL_DOMAINS, "kit rows must exactly cover required objective domains")

    for row in rows:
        kit_id = row["kit_id"]
        target_kit = target_kits[kit_id]
        require(row.get("source_target_kit_id") == kit_id, f"{kit_id}: source_target_kit_id must match kit_id")
        require(row.get("source_gate") == target_kit.get("gate"), f"{kit_id}: source_gate must match target gate")
        domains = set(row.get("objective_domains", []))
        require(domains, f"{kit_id}: objective_domains required")
        require(domains <= REQUIRED_ALL_DOMAINS, f"{kit_id}: objective_domains contains unsupported domains {sorted(domains - REQUIRED_ALL_DOMAINS)}")
        if kit_id == "QK-06-k8s-pod-security":
            require(REQUIRED_TARGET_DOMAIN in domains, "QK-06 must preserve the target k8s pod-security kit even though it is not one of the seven user-enumerated domains")
        source_scenarios = set(row.get("source_scenarios", []))
        require(set(target_kit.get("scenarios", [])) <= source_scenarios, f"{kit_id}: source scenarios do not cover target scenarios")
        source_controls = set(row.get("source_controls", []))
        require(set(target_kit.get("controls", [])) <= source_controls, f"{kit_id}: source controls do not cover target controls")
        require(row.get("source_harness") == target_kit.get("harness"), f"{kit_id}: source_harness must preserve target harness wording")
        require(row.get("source_evidence") == target_kit.get("evidence"), f"{kit_id}: source_evidence must preserve target evidence wording")
        outputs = set(row.get("machine_check_outputs", []))
        require(REQUIRED_KIT_EVIDENCE_OUTPUTS[kit_id] <= outputs, f"{kit_id}: machine_check_outputs missing {sorted(REQUIRED_KIT_EVIDENCE_OUTPUTS[kit_id] - outputs)}")

        harness = row.get("harness_backlog", {})
        require(harness.get("status") == "pending_implementation", f"{kit_id}: harness status must be pending_implementation")
        require(harness.get("runtime_status") == "not_implemented", f"{kit_id}: runtime_status must be not_implemented")
        require(harness.get("evidence_status") == "evidence_required", f"{kit_id}: evidence_status must be evidence_required")
        require(harness.get("dogfood_only") is True, f"{kit_id}: harness must be dogfood_only")
        require(harness.get("machine_check_type"), f"{kit_id}: machine_check_type required")

        schema = row.get("evidence_record_schema", {})
        require(schema.get("record_status") == "schema_only", f"{kit_id}: evidence schema must be schema_only")
        require(REQUIRED_EVIDENCE_FIELDS <= set(schema.get("required_fields", [])), f"{kit_id}: evidence schema missing fields {sorted(REQUIRED_EVIDENCE_FIELDS - set(schema.get('required_fields', [])))}")
        require(schema.get("allowed_statuses") == ["pending", "failed", "blocked", "passed_after_future_runtime_evidence"], f"{kit_id}: allowed_statuses must preserve no-current-green semantics")
        require(REQUIRED_KIT_EVIDENCE_OUTPUTS[kit_id] <= set(schema.get("kit_specific_required_outputs", [])), f"{kit_id}: evidence schema must bind kit-specific output keys")
        require(set(schema.get("kit_specific_required_outputs", [])) <= outputs, f"{kit_id}: schema output keys must be a subset of kit outputs")
        require(REQUIRED_RESULT_SUMMARY_KEYS <= set(schema.get("result_summary_required_keys", [])), f"{kit_id}: evidence schema must define kit-specific result summary keys")

        scenarios = row.get("scenario_backlog", [])
        require(len(scenarios) >= len(target_kit.get("scenarios", [])), f"{kit_id}: scenario_backlog must cover target scenarios")
        scenario_names = {scenario.get("source_scenario") for scenario in scenarios}
        require(set(target_kit.get("scenarios", [])) <= scenario_names, f"{kit_id}: scenario_backlog missing target scenarios")
        for scenario in scenarios:
            require(scenario.get("status") == "backlog_pending", f"{kit_id}/{scenario.get('scenario_id')}: scenario status must be backlog_pending")
            require(scenario.get("requires_dogfood_environment") is True, f"{kit_id}/{scenario.get('scenario_id')}: dogfood environment required")
            require(REQUIRED_EVIDENCE_FIELDS <= set(scenario.get("evidence_fields", [])), f"{kit_id}/{scenario.get('scenario_id')}: scenario evidence fields incomplete")
            require(set(scenario.get("machine_check_outputs", [])) <= outputs and scenario.get("machine_check_outputs"), f"{kit_id}/{scenario.get('scenario_id')}: scenario machine_check_outputs must be non-empty subset of kit outputs")

        gate = row.get("evidence_gate", {})
        require(gate.get("claim_status") == "blocked_until_required_evidence_is_green", f"{kit_id}: claim_status must remain blocked")
        require(gate.get("green_status") == "not_claimed", f"{kit_id}: green_status must be not_claimed")
        require(set(gate.get("required_before_green", [])) >= {"implemented_harness", "dogfood_run_receipt", "evidence_record", "reviewer_approval", "validator_pass"}, f"{kit_id}: required_before_green incomplete")
        require(gate.get("feeds_production_100_gate") == target_kit.get("gate"), f"{kit_id}: production gate mapping mismatch")

        require(set(row.get("blocked_claim_families", [])) >= REQUIRED_BLOCKED_CLAIMS, f"{kit_id}: missing blocked claim families")
        require(not contains_forbidden_positive(row.get("honest_claim", "")), f"{kit_id}: honest_claim contains forbidden positive claim")
        if kit_id == "QK-04-canary-prr":
            dr_extension = row.get("dr_backlog_extension", {})
            require(dr_extension, "QK-04: dr_backlog_extension is required for failover/DR coverage")
            require(dr_extension.get("status") == "backlog_pending", "QK-04: DR extension status must be backlog_pending")
            require(dr_extension.get("claim_status") == "evidence_required_not_green", "QK-04: DR extension must remain evidence_required_not_green")
            require(dr_extension.get("source_domain") == "failover_dr", "QK-04: DR extension source_domain must be failover_dr")
            require(dr_extension.get("official_source_url") == EXPECTED_SOURCE_URL_BY_DOMAIN["failover_dr"], "QK-04: DR extension must bind the official failover/DR source")
            require(set(dr_extension.get("required_dr_receipts", [])) == REQUIRED_QK04_DR_RECEIPTS, "QK-04: DR extension must require all DR receipts exactly")
            require(REQUIRED_QK04_DR_RECEIPTS <= outputs, "QK-04: DR receipts must be machine-checkable kit outputs")
            require(set(dr_extension.get("required_dr_scenarios", [])) == set(REQUIRED_QK04_DR_SCENARIO_OUTPUTS), "QK-04: DR extension must require DR01..DR04 scenarios")
            scenario_by_id = {scenario.get("scenario_id"): scenario for scenario in scenarios}
            for scenario_id, required_outputs in REQUIRED_QK04_DR_SCENARIO_OUTPUTS.items():
                require(scenario_id in scenario_by_id, f"QK-04: missing required DR scenario {scenario_id}")
                scenario_outputs = set(scenario_by_id[scenario_id].get("machine_check_outputs", []))
                require(required_outputs <= scenario_outputs, f"QK-04/{scenario_id}: DR scenario outputs missing {sorted(required_outputs - scenario_outputs)}")
            required_before_green = set(gate.get("required_before_green", []))
            require(REQUIRED_QK04_DR_RECEIPTS <= required_before_green, "QK-04: evidence gate must require DR receipts before green")
            require(set(REQUIRED_QK04_DR_SCENARIO_OUTPUTS) <= required_before_green, "QK-04: evidence gate must require DR scenarios before green")

    domain_coverage = spec["objective_domain_coverage"]
    require(set(domain_coverage) == REQUIRED_ALL_DOMAINS, "objective_domain_coverage must exactly cover required domains")
    for domain, row in domain_coverage.items():
        kit_ids = set(row.get("kit_ids", []))
        require(kit_ids and kit_ids <= row_ids, f"{domain}: coverage references unknown kit ids")
        require(kit_ids == domains_from_rows.get(domain, set()), f"{domain}: objective_domain_coverage.kit_ids must match kit rows declaring that domain")
        require(row.get("source_url") == EXPECTED_SOURCE_URL_BY_DOMAIN[domain], f"{domain}: objective_domain_coverage.source_url must match official/upstream source")
        require(source_host_allowed(row["source_url"]), f"{domain}: objective_domain_coverage.source_url is not an allowlisted HTTPS source")
        require(row.get("claim_status") == "backlog_only_evidence_required", f"{domain}: claim_status must be backlog_only_evidence_required")

    surfaces = {surface.get("id"): surface for surface in spec["machine_check_surfaces"]}
    require("g006-validator" in surfaces, "machine_check_surfaces must include g006-validator")
    require(surfaces["g006-validator"].get("command") == "python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py", "g006-validator command mismatch")

    nonclaim_ids = {item.get("id") for item in spec["nonclaims"]}
    require(REQUIRED_BLOCKED_CLAIMS <= nonclaim_ids, f"missing nonclaims {sorted(REQUIRED_BLOCKED_CLAIMS - nonclaim_ids)}")
    require(observability.get("spec_id") == "EXE-CLOUD-OBSERVABILITY-SLO-EVIDENCE-CONTRACT", "unexpected G005 observability source")
    require(gates.get("hyperscaler_mature_claim_rule", {}).get("claim_status") == "blocked_until_required_evidence_is_green", "hyperscaler claim rule must remain blocked")
    require(spec["next_goal_links"].get("dogfood_ci_toolchain") == "G007", "G007 link required")
    require(spec["next_goal_links"].get("final_quality_gate") == "G008", "G008 link required")


def main() -> None:
    validate(load_json(SPEC_PATH))
    print(f"cloud production-quality kit evidence backlog check passed: {SPEC_PATH.relative_to(REPO_ROOT)}")


def run_self_tests() -> None:
    baseline = load_json(SPEC_PATH)

    def expect_rejected(label: str, mutator: Callable[[dict], None]) -> None:
        candidate = json.loads(json.dumps(baseline))
        mutator(candidate)
        try:
            validate(candidate)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    def add_unsupported_domain(data: dict) -> None:
        data["kit_backlog"][0]["objective_domains"].append("unsupported_extra_domain")
        data["objective_domain_coverage"]["unsupported_extra_domain"] = {
            "kit_ids": [data["kit_backlog"][0]["kit_id"]],
            "source_url": "http://example.com/bogus",
            "claim_status": "backlog_only_evidence_required",
        }

    expect_rejected("missing kit", lambda data: data.update({"kit_backlog": data["kit_backlog"][1:]}))
    expect_rejected("unknown kit", lambda data: data["kit_backlog"][0].update({"kit_id": "QK-99-unknown"}))
    expect_rejected("disabled strict separation", lambda data: data["claim_controls"].update({"strict_separation": False}))
    expect_rejected("external SaaS source", lambda data: data["official_source_evidence"][0].update({"url": "https://example.com/not-official"}))
    expect_rejected("harness implemented overclaim", lambda data: data["kit_backlog"][0]["harness_backlog"].update({"runtime_status": "implemented"}))
    expect_rejected("green evidence overclaim", lambda data: data["kit_backlog"][0]["evidence_gate"].update({"green_status": "green"}))
    expect_rejected("missing evidence field", lambda data: data["kit_backlog"][0]["evidence_record_schema"].update({"required_fields": ["kit_id"]}))
    expect_rejected("missing target scenario", lambda data: data["kit_backlog"][0].update({"source_scenarios": []}))
    expect_rejected("changed source harness", lambda data: data["kit_backlog"][0].update({"source_harness": "generic harness"}))
    expect_rejected("missing source evidence", lambda data: data["kit_backlog"][0].update({"source_evidence": "generic result"}))
    expect_rejected("missing machine-check output", lambda data: data["kit_backlog"][0].update({"machine_check_outputs": []}))
    expect_rejected("generic evidence schema", lambda data: data["kit_backlog"][0]["evidence_record_schema"].update({"kit_specific_required_outputs": ["generic_output"]}))
    expect_rejected("missing result summary keys", lambda data: data["kit_backlog"][0]["evidence_record_schema"].update({"result_summary_required_keys": []}))
    expect_rejected("missing Kubernetes source", lambda data: data.update({"official_source_evidence": [row for row in data["official_source_evidence"] if row["domain"] != "k8s_pod_security"]}))
    expect_rejected("wrong FinOps source URL", lambda data: data["official_source_evidence"][-3].update({"url": "https://aws.amazon.com/builders-library/using-load-shedding-to-avoid-overload/"}))
    expect_rejected("missing failover domain", lambda data: data["objective_domain_coverage"].pop("failover_dr"))
    expect_rejected("extra objective domain", add_unsupported_domain)
    expect_rejected("non-https coverage source URL", lambda data: data["objective_domain_coverage"]["overload_fairness"].update({"source_url": "http://aws.amazon.com/builders-library/using-load-shedding-to-avoid-overload/"}))
    expect_rejected("missing DR extension", lambda data: data["kit_backlog"][3].pop("dr_backlog_extension"))
    expect_rejected("missing DR receipt output", lambda data: data["kit_backlog"][3].update({"machine_check_outputs": [output for output in data["kit_backlog"][3]["machine_check_outputs"] if output != "dependency_failure_recovery_receipt"]}))
    expect_rejected("missing DR scenario", lambda data: data["kit_backlog"][3].update({"scenario_backlog": [scenario for scenario in data["kit_backlog"][3]["scenario_backlog"] if scenario["scenario_id"] != "QK-04-canary-prr-DR04"]}))
    expect_rejected("production ready claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "production ready"}))
    expect_rejected("production grade claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "production-grade"}))
    expect_rejected("production capable claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "production capable"}))
    expect_rejected("production viable claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "production viable"}))
    expect_rejected("production is viable claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "production is viable"}))
    expect_rejected("prod viable claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "prod viable"}))
    expect_rejected("production suitable claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "production suitable"}))
    expect_rejected("production is suitable claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "production is suitable"}))
    expect_rejected("ga ready claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "GA ready"}))
    expect_rejected("generally available claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "generally available"}))
    expect_rejected("customer workload claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "customer workloads are supported"}))
    expect_rejected("customer workload operate safely claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "customer workloads operate safely"}))
    expect_rejected("customer workload operates safely claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "customer workload operates safely"}))
    expect_rejected("customer workload runs safely claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "customer workload runs safely"}))
    expect_rejected("availability achieved claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "99.9 availability achieved"}))
    expect_rejected("tenant workload claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "tenant workloads can run"}))
    expect_rejected("tenant workload run safely claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "tenant workloads run safely"}))
    expect_rejected("tenant workload operates safely claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "tenant workload operates safely"}))
    expect_rejected("tenant workload runs safely claim", lambda data: data["kit_backlog"][0].update({"honest_claim": "tenant workload runs safely"}))
    expect_rejected("public service-level claim", lambda data: data["claim_controls"].update({"can_claim_now": ["public service level agreement ready"]}))
    expect_rejected("sla achieved claim", lambda data: data["claim_controls"].update({"can_claim_now": ["SLA achieved"]}))
    expect_rejected("sla fulfilled claim", lambda data: data["claim_controls"].update({"can_claim_now": ["SLA fulfilled"]}))
    expect_rejected("slo achieved claim", lambda data: data["claim_controls"].update({"can_claim_now": ["SLO achieved"]}))
    expect_rejected("slo satisfied claim", lambda data: data["claim_controls"].update({"can_claim_now": ["SLO satisfied"]}))
    expect_rejected("tenant workload readiness claim", lambda data: data["doubt_driven_review"].update({"resolution": "tenant workload readiness achieved"}))
    expect_rejected("hyperscale grade claim", lambda data: data["doubt_driven_review"].update({"resolution": "hyperscale grade readiness established"}))
    expect_rejected("hyperscaler quality bar claim", lambda data: data["doubt_driven_review"].update({"resolution": "hyperscaler quality bar met"}))
    expect_rejected("runtime green claim", lambda data: data["doubt_driven_review"].update({"resolution": "runtime is green"}))
    print("cloud production-quality kit evidence backlog self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
