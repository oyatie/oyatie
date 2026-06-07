#!/usr/bin/env python3
"""Validate Oyatie Cloud observability/SLO evidence contracts for cloud resource parity."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "specs" / "cloud-observability-slo-evidence-contract.json"
RESOURCE_CATALOG_PATH = REPO_ROOT / "specs" / "cloud-resource-contract-parity-catalog.json"
ENFORCEABILITY_PATH = REPO_ROOT / "specs" / "cloud-enforceability-facets.json"
OBSERVABILITY_TARGET_PATH = REPO_ROOT / "specs" / "cloud-observability-slo-target.json"
HYPERSCALER_GATES_PATH = REPO_ROOT / "specs" / "hyperscaler-gates.json"
OPENSLO_CANONICAL_ENVELOPE_PATH = REPO_ROOT / "specs" / "openslo" / "canonical-envelope-schema.json"

REQUIRED_OTEL_RESOURCE_ATTRIBUTES = {
    "service.name",
    "service.namespace",
    "service.instance.id",
    "service.version",
    "deployment.environment.name",
    "oya.tenant",
    "oya.account",
    "oya.project",
    "oya.region",
    "oya.cell",
    "oya.resource.orn",
    "oya.resource.type",
    "oya.operation.id",
}
REQUIRED_CORRELATION_FIELDS = {
    "trace_id",
    "span_id",
    "operation_id",
    "audit_chain_id",
    "resource_orn",
    "tenant_account_project",
    "region_cell",
    "policy_snapshot",
}
REQUIRED_SLI_EVIDENCE_FIELDS = {
    "slo_id",
    "indicator",
    "objective",
    "window_start",
    "window_end",
    "numerator_query",
    "denominator_query",
    "sample_count",
    "datasource",
    "query_digest",
    "measured_value",
    "error_budget_remaining",
    "burn_rate",
    "evidence_digest",
    "reviewer",
}
REQUIRED_EVENT_FIELDS = {
    "event_type",
    "operation_id",
    "resource_orn",
    "tenant_account_project",
    "region_cell",
    "trace_id",
    "span_id",
    "audit_chain_id",
    "occurred_at",
    "schema_version",
}
REQUIRED_EVIDENCE_BEFORE_CLAIM = {
    "measured_slo_window",
    "burn_rate_alert_receipt",
    "otel_trace_metric_log_sample",
    "audit_chain_correlation_sample",
    "progressive_delivery_gate_receipt",
}
REQUIRED_NONCLAIMS = {
    "no_measured_slo_claim",
    "no_public_sla_slo_claim",
    "no_runtime_observability_engine",
    "no_production_readiness_claim",
    "no_tenant_workload_claim",
    "no_hyperscaler_maturity_claim",
}
FORBIDDEN_POSITIVE_PATTERNS = [
    re.compile(pattern)
    for pattern in [
        r"\bmeasured\s+(slo|availability|latency|recovery|burn\s*rate)\b.{0,40}\b(green|exists?|available|achieved|passed|ready)\b",
        r"\bpublic\s+(sla|slo|service\s*level\s*agreement)\b.{0,40}\b(ready|available|exists?|enabled|published|achieved)\b",
        r"\b(sla|slo|service\s*level\s*agreement)\b.{0,20}\bbacked\b",
        r"\b(prod|production)\s+ready\b",
        r"\b(prod|production)\b.{0,40}\b(customer\s*traffic|tenant\s*traffic|readiness\s*(achieved|established|ready)|available)\b",
        r"\btenant\s+workloads?\b.{0,40}\b(can\s+run|ready|safe|safely\s+run|supported|enabled)\b",
        r"\b(runtime\s+)?observability\s+engine\b.{0,40}\b(available|implemented|live|ready)\b",
        r"\botel\s+collector\b.{0,40}\b(available|implemented|live|ready)\b",
        r"\bslo\s+engine\b.{0,40}\b(available|implemented|live|ready)\b",
        r"\bburn\s*rate\s+alert(ing)?\b.{0,40}\b(live|available|implemented|ready)\b",
        r"\bdashboards?\b.{0,40}\b(implemented|available|live|ready)\b",
        r"\b(hyperscaler|hyperscale)\b.{0,30}\b(mature|maturity|grade|readiness|ready|established|achieved)\b",
        r"\baws[-\s]*grade\b",
        r"\bfeature\s+parity\b.{0,40}\b(achieved|ready|exists?|available|complete)\b",
    ]
]


def fail(message: str) -> NoReturn:
    print(f"cloud observability SLO evidence check failed: {message}", file=sys.stderr)
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
    for row in candidate.get("resource_observability", []):
        row.pop("blocked_claim_families", None)
        row.get("slo_profile", {}).pop("evidence_required_before_claim", None)
    for row in candidate.get("category_rollup", {}).values():
        row.pop("blocked_claim_families", None)
    return candidate


def validate(spec: dict) -> None:
    catalog = load_json(RESOURCE_CATALOG_PATH)
    enforceability = load_json(ENFORCEABILITY_PATH)
    target = load_json(OBSERVABILITY_TARGET_PATH)
    gates = load_json(HYPERSCALER_GATES_PATH)
    openslo_envelope = load_json(OPENSLO_CANONICAL_ENVELOPE_PATH)

    for field in [
        "spec_id",
        "title",
        "status",
        "source_resource_catalog",
        "source_enforceability_facets",
        "source_observability_target",
        "source_hyperscaler_gates",
        "source_openslo_canonical_envelope",
        "source_otel_semconv",
        "purpose",
        "claim_controls",
        "evidence_vocabulary",
        "resource_observability",
        "category_rollup",
        "gate_alignment",
        "nonclaims",
        "next_goal_links",
    ]:
        require(field in spec, f"missing top-level field {field!r}")

    require(spec["spec_id"] == "EXE-CLOUD-OBSERVABILITY-SLO-EVIDENCE-CONTRACT", "unexpected spec_id")
    require(spec["status"] == "Proposed-target", "status must remain Proposed-target until measured evidence exists")
    require(spec["source_resource_catalog"] == str(RESOURCE_CATALOG_PATH.relative_to(REPO_ROOT)), "source_resource_catalog must point to G002 catalog")
    require(spec["source_enforceability_facets"] == str(ENFORCEABILITY_PATH.relative_to(REPO_ROOT)), "source_enforceability_facets must point to G004 facets")
    require(spec["source_observability_target"] == str(OBSERVABILITY_TARGET_PATH.relative_to(REPO_ROOT)), "source_observability_target must point to cloud observability target")
    require(spec["source_hyperscaler_gates"] == str(HYPERSCALER_GATES_PATH.relative_to(REPO_ROOT)), "source_hyperscaler_gates must point to hyperscaler gates")
    require(spec["source_openslo_canonical_envelope"] == str(OPENSLO_CANONICAL_ENVELOPE_PATH.relative_to(REPO_ROOT)), "source_openslo_canonical_envelope must point to canonical OpenSLO envelope")
    semconv = spec["source_otel_semconv"]
    require(semconv.get("version") == "1.41.0", "OpenTelemetry semantic convention version must be pinned to 1.41.0")
    require("opentelemetry.io/docs/specs/semconv" in semconv.get("registry_url", ""), "OpenTelemetry semconv registry URL required")
    require("opentelemetry.io/docs/specs/semconv/resource/service" in semconv.get("service_url", ""), "OpenTelemetry service semconv URL required")

    controls = spec["claim_controls"]
    for key in [
        "metadata_only",
        "evidence_contract_only",
        "strict_separation",
        "pure_dogfood",
        "no_measured_slo_claim",
        "no_public_sla_slo_claim",
        "no_runtime_observability_engine",
        "no_production_readiness_claim",
        "no_tenant_workload_readiness",
        "no_hyperscaler_maturity_claim",
    ]:
        require(controls.get(key) is True, f"claim_controls.{key} must be true")
    require(not contains_forbidden_positive(controls.get("can_claim_now", [])), "claim_controls.can_claim_now contains forbidden positive claim")
    require(REQUIRED_NONCLAIMS <= set(controls.get("blocked_claim_families", [])), "claim_controls missing blocked claim families")
    require(not contains_forbidden_positive(sanitized_for_positive_claim_scan(spec)), "spec contains forbidden positive claim wording outside blocked/nonclaim fields")

    vocab = spec["evidence_vocabulary"]
    canonical_windows = {
        window[name]
        for window in openslo_envelope.get("burn_rate_alert_pattern", {}).get("windows", [])
        for name in ("alert_window", "long_window", "short_window")
    }
    require(canonical_windows == {"5m", "30m", "1h", "2h", "6h", "1d", "3d"}, "canonical OpenSLO burn-rate envelope must define the expected multi-window set")
    require(REQUIRED_OTEL_RESOURCE_ATTRIBUTES <= set(vocab.get("otel_resource_attributes", [])), "OTel resource attributes incomplete")
    require({"deployment.environment"} <= set(vocab.get("otel_compatibility_aliases", [])), "legacy OTel compatibility alias for deployment.environment required")
    require(REQUIRED_CORRELATION_FIELDS <= set(vocab.get("correlation_fields", [])), "correlation fields incomplete")
    require(REQUIRED_SLI_EVIDENCE_FIELDS <= set(vocab.get("sli_evidence_fields", [])), "SLI evidence fields incomplete")
    require(REQUIRED_EVENT_FIELDS <= set(vocab.get("event_fields", [])), "event fields incomplete")
    require(canonical_windows <= set(vocab.get("burn_rate_windows", [])), "burn-rate windows incomplete")
    require(REQUIRED_EVIDENCE_BEFORE_CLAIM <= set(vocab.get("evidence_required_before_claim", [])), "evidence_required_before_claim vocabulary incomplete")
    require(vocab.get("slo_document_format") == "OpenSLO", "slo_document_format must be OpenSLO")
    require(vocab.get("telemetry_standard") == "OpenTelemetry", "telemetry_standard must be OpenTelemetry")
    require(vocab.get("evidence_status") == "evidence_required", "evidence_status must remain evidence_required")

    contract_by_id = {contract["id"]: contract for contract in catalog["resource_contracts"]}
    enforceability_ids = {row["resource_contract_id"] for row in enforceability["resource_enforceability"]}
    require(set(contract_by_id) <= enforceability_ids, "G004 enforceability source must cover every G002 contract")

    rows = spec["resource_observability"]
    require(isinstance(rows, list) and rows, "resource_observability must be non-empty")
    row_ids = {row.get("resource_contract_id") for row in rows}
    require(set(contract_by_id) <= row_ids, f"missing observability rows for contracts {sorted(set(contract_by_id) - row_ids)}")
    require(row_ids <= set(contract_by_id), f"unknown observability contract ids {sorted(row_ids - set(contract_by_id))}")

    for row in rows:
        cid = row["resource_contract_id"]
        source = contract_by_id[cid]
        require(row.get("service") == source["service"], f"{cid}: service mismatch with G002 catalog")
        require(row.get("category_id") == source["category_id"], f"{cid}: category mismatch with G002 catalog")
        require(row.get("resource_type") == source["resource_type"], f"{cid}: resource_type mismatch with G002 catalog")
        telemetry = row.get("telemetry_profile", {})
        require(telemetry.get("standard") == "OpenTelemetry", f"{cid}: telemetry standard must be OpenTelemetry")
        require(telemetry.get("semantic_convention_version") == semconv["version"], f"{cid}: telemetry semantic_convention_version must match source_otel_semconv.version")
        require(telemetry.get("runtime_status") == "target_contract_only", f"{cid}: telemetry runtime_status must be target_contract_only")
        require(REQUIRED_OTEL_RESOURCE_ATTRIBUTES <= set(telemetry.get("resource_attributes", [])), f"{cid}: missing OTel resource attributes")
        require({"cloud.control_plane.operation", "cloud.resource.lifecycle"} <= set(telemetry.get("required_spans", [])), f"{cid}: required spans incomplete")
        require({"operation.duration", "operation.count", "operation.errors", "resource.state", "quota.saturation"} <= set(telemetry.get("required_metrics", [])), f"{cid}: required metrics incomplete")
        require(REQUIRED_EVENT_FIELDS <= set(telemetry.get("event_fields", [])), f"{cid}: event fields incomplete")
        require(REQUIRED_CORRELATION_FIELDS <= set(telemetry.get("correlation_fields", [])), f"{cid}: correlation fields incomplete")

        slo = row.get("slo_profile", {})
        require(slo.get("openslo_required") is True, f"{cid}: OpenSLO artifact must be required")
        require(slo.get("runtime_status") == "evidence_required", f"{cid}: SLO runtime_status must be evidence_required")
        require(str(slo.get("artifact_pattern", "")).endswith("/slos/*.openslo.yaml"), f"{cid}: SLO artifact pattern must target OpenSLO files")
        require({"availability", "latency", "control_plane_success"} <= set(slo.get("required_sli_types", [])), f"{cid}: required SLI types incomplete")
        require(REQUIRED_SLI_EVIDENCE_FIELDS <= set(slo.get("evidence_fields", [])), f"{cid}: SLI evidence fields incomplete")
        require(canonical_windows <= set(slo.get("burn_rate_windows", [])), f"{cid}: burn-rate windows incomplete")
        require(REQUIRED_EVIDENCE_BEFORE_CLAIM <= set(slo.get("evidence_required_before_claim", [])), f"{cid}: evidence_required_before_claim incomplete")
        require(slo.get("promotion_gate") == "blocked_until_required_evidence_is_green", f"{cid}: promotion gate must be evidence-blocked")

        events = row.get("event_evidence_profile", {})
        require(events.get("runtime_status") == "evidence_required", f"{cid}: event evidence runtime_status must be evidence_required")
        require(REQUIRED_EVENT_FIELDS <= set(events.get("required_event_fields", [])), f"{cid}: event evidence fields incomplete")
        require(REQUIRED_CORRELATION_FIELDS <= set(events.get("required_correlation_fields", [])), f"{cid}: event evidence correlation fields incomplete")
        require({"operation_outcome_receipt", "lifecycle_transition_receipt", "audit_chain_correlation_receipt", "slo_gate_receipt"} <= set(events.get("minimum_receipts_before_claim", [])), f"{cid}: event receipts before claim incomplete")

        blocked = set(row.get("blocked_claim_families", []))
        require(REQUIRED_NONCLAIMS <= blocked, f"{cid}: missing blocked claim families {sorted(REQUIRED_NONCLAIMS - blocked)}")
        require(not contains_forbidden_positive(row.get("honest_claim", "")), f"{cid}: honest_claim contains forbidden positive claim")

    categories = {contract["category_id"] for contract in catalog["resource_contracts"]}
    rollup = spec["category_rollup"]
    require(set(rollup) >= categories, "category_rollup must cover every G002 category")
    for category_id in categories:
        row = rollup[category_id]
        ids = set(row.get("resource_contract_ids", []))
        expected = {cid for cid, contract in contract_by_id.items() if contract["category_id"] == category_id}
        require(ids == expected, f"{category_id}: rollup contract ids do not match G002")
        require(row.get("evidence_status") == "evidence_required", f"{category_id}: evidence_status must be evidence_required")
        require(row.get("runtime_status") == "target_contract_only", f"{category_id}: runtime_status must be target_contract_only")
        require(REQUIRED_NONCLAIMS <= set(row.get("blocked_claim_families", [])), f"{category_id}: rollup missing blocked claim families")

    gates_by_id = {gate["id"]: gate for gate in gates["gates"]}
    alignment = spec["gate_alignment"]
    require("HG-OBS" in alignment and "HG-OPS" in alignment, "gate_alignment must cover HG-OBS and HG-OPS")
    require(set(gates_by_id["HG-OBS"]["requires"]) <= set(alignment["HG-OBS"].get("source_requires", [])), "HG-OBS source requires incomplete")
    require(set(gates_by_id["HG-OPS"]["requires"]) <= set(alignment["HG-OPS"].get("source_requires", [])), "HG-OPS source requires incomplete")
    require(alignment["HG-OBS"].get("evidence_status") == "evidence_required", "HG-OBS evidence status must be evidence_required")
    require(alignment["HG-OPS"].get("evidence_status") == "evidence_required", "HG-OPS evidence status must be evidence_required")
    for gate_id in ("HG-OBS", "HG-OPS"):
        mapping = alignment[gate_id].get("requirement_mapping", {})
        missing = set(gates_by_id[gate_id]["requires"]) - set(mapping)
        require(not missing, f"{gate_id} requirement_mapping missing {sorted(missing)}")
        for req, mapped_fields in mapping.items():
            require(isinstance(mapped_fields, list) and mapped_fields, f"{gate_id}.{req}: mapping must list concrete fields")

    require(target.get("spec_id") == "EXE-CLOUD-OBSERVABILITY-SLO-TARGET", "unexpected observability target source")
    require("OpenTelemetry" in target.get("telemetry", {}).get("standard", ""), "source observability target must require OpenTelemetry")
    require("burn-rate" in target.get("slo_model", {}).get("error_budget", ""), "source observability target must require burn-rate evidence")

    nonclaim_ids = {item.get("id") for item in spec["nonclaims"]}
    require(REQUIRED_NONCLAIMS <= nonclaim_ids, f"missing nonclaims {sorted(REQUIRED_NONCLAIMS - nonclaim_ids)}")
    require(spec["next_goal_links"].get("production_quality_kits") == "G006", "G006 link required")
    require(spec["next_goal_links"].get("dogfood_ci_toolchain") == "G007", "G007 link required")
    require(spec["next_goal_links"].get("final_quality_gate") == "G008", "G008 link required")


def main() -> None:
    validate(load_json(SPEC_PATH))
    print(f"cloud observability SLO evidence check passed: {SPEC_PATH.relative_to(REPO_ROOT)}")


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

    expect_rejected("missing contract row", lambda data: data.update({"resource_observability": data["resource_observability"][1:]}))
    expect_rejected("missing OTel resource attribute", lambda data: data["resource_observability"][0]["telemetry_profile"].update({"resource_attributes": ["service.name"]}))
    expect_rejected("telemetry runtime overclaim", lambda data: data["resource_observability"][0]["telemetry_profile"].update({"runtime_status": "runtime_available"}))
    expect_rejected("row OTel semconv version drift", lambda data: data["resource_observability"][0]["telemetry_profile"].update({"semantic_convention_version": "0.0.0"}))
    expect_rejected("OpenSLO not required", lambda data: data["resource_observability"][0]["slo_profile"].update({"openslo_required": False}))
    expect_rejected("missing burn-rate window", lambda data: data["resource_observability"][0]["slo_profile"].update({"burn_rate_windows": ["5m"]}))
    expect_rejected("missing evidence before claim", lambda data: data["resource_observability"][0]["slo_profile"].update({"evidence_required_before_claim": ["measured_slo_window"]}))
    expect_rejected("missing event receipt", lambda data: data["resource_observability"][0]["event_evidence_profile"].update({"minimum_receipts_before_claim": ["operation_outcome_receipt"]}))
    expect_rejected("disabled strict separation", lambda data: data["claim_controls"].update({"strict_separation": False}))
    expect_rejected("measured SLO overclaim", lambda data: data["doubt_driven_review"].update({"resolution": "measured SLO is green"}))
    expect_rejected("public SLA overclaim", lambda data: data["claim_controls"].update({"can_claim_now": ["public SLA is available"]}))
    expect_rejected("production readiness overclaim", lambda data: data["gate_alignment"]["HG-OPS"].update({"note": "production ready"}))
    expect_rejected("tenant workload overclaim", lambda data: data["resource_observability"][0].update({"honest_claim": "tenant workload ready"}))
    expect_rejected("runtime collector overclaim", lambda data: data.update({"purpose": "OTel collector is implemented"}))
    expect_rejected("public service level agreement synonym", lambda data: data["claim_controls"].update({"can_claim_now": ["public service level agreement ready"]}))
    expect_rejected("prod customer traffic synonym", lambda data: data["gate_alignment"]["HG-OPS"].update({"note": "prod ready for customer traffic"}))
    expect_rejected("hyperscale grade synonym", lambda data: data["doubt_driven_review"].update({"resolution": "hyperscale grade readiness established"}))
    expect_rejected("missing HG-OBS alignment", lambda data: data["gate_alignment"].pop("HG-OBS"))
    expect_rejected("incomplete HG-OPS alignment", lambda data: data["gate_alignment"]["HG-OPS"].update({"source_requires": ["progressive_delivery_or_explicit_nonproduction_scope"]}))
    expect_rejected("missing HG-OPS requirement mapping", lambda data: data["gate_alignment"]["HG-OPS"].update({"requirement_mapping": {"progressive_delivery_or_explicit_nonproduction_scope": ["slo_profile.promotion_gate"]}}))
    expect_rejected("category rollup mismatch", lambda data: data["category_rollup"][data["resource_observability"][0]["category_id"]].update({"resource_contract_ids": []}))
    print("cloud observability SLO evidence self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
