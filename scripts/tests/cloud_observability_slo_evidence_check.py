#!/usr/bin/env python3
"""Validate Oyatie Cloud observability/SLO evidence contracts for cloud resource parity."""
from __future__ import annotations

import hashlib
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
REQUIRED_MEASURED_WINDOW_RECEIPT_METADATA_FIELDS = {
    "source_commit",
    "artifact_digest",
    "reviewer",
}
REQUIRED_MEASURED_WINDOW_RECEIPT_FIELDS = REQUIRED_SLI_EVIDENCE_FIELDS | {
    "source_commit",
    "artifact_digest",
}
REQUIRED_AUDIT_CHAIN_RECEIPT_FIELDS = REQUIRED_CORRELATION_FIELDS | {
    "correlated_receipts",
    "evidence_digest",
    "source_commit",
    "artifact_digest",
    "reviewer",
}
REQUIRED_AUDIT_CHAIN_POLICY_SNAPSHOT_FIELDS = {
    "policy_engine",
    "policy_bundle_digest",
    "policy_version",
    "decision_id",
    "subject",
    "action",
    "resource",
    "effect",
    "captured_at",
}
REQUIRED_AUDIT_CHAIN_RECEIPT_METADATA_FIELDS = {
    "source_commit",
    "artifact_digest",
    "reviewer",
}
REQUIRED_AUDIT_CHAIN_CORRELATED_RECEIPTS = {
    "operation_outcome_receipt",
    "lifecycle_transition_receipt",
    "slo_gate_receipt",
}
REQUIRED_AUDIT_CHAIN_RECEIPT_FAMILIES = REQUIRED_AUDIT_CHAIN_CORRELATED_RECEIPTS | {
    "audit_chain_correlation_receipt",
}
REQUIRED_AUDIT_CHAIN_CONSISTENCY_FIELDS = {
    "operation_id",
    "resource_orn",
    "tenant_account_project",
    "region_cell",
    "trace_id",
    "span_id",
    "audit_chain_id",
}
HEX_DIGEST = re.compile(r"^sha256:[0-9a-f]{64}$")
GIT_COMMIT_SHA = re.compile(r"^[0-9a-f]{40}$")
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
REQUIRED_OTEL_SAMPLE_RECEIPT_CORRELATION_FIELDS = {
    "trace_id",
    "span_id",
    "operation_id",
    "resource_orn",
}
REQUIRED_OTEL_SAMPLE_RECEIPT_METADATA_FIELDS = {
    "source_commit",
    "artifact_digest",
    "reviewer",
}
REQUIRED_OTEL_SAMPLE_BLOCKS = {
    "metric_sample",
    "structured_log_sample",
}
REQUIRED_OTEL_SAMPLE_RECEIPT_FIELDS = (
    REQUIRED_OTEL_RESOURCE_ATTRIBUTES
    | REQUIRED_OTEL_SAMPLE_RECEIPT_CORRELATION_FIELDS
    | REQUIRED_OTEL_SAMPLE_RECEIPT_METADATA_FIELDS
    | REQUIRED_OTEL_SAMPLE_BLOCKS
)
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
        r"\b(runtime\s+|otel\s+)?exporter\b.{0,40}\b(available|implemented|live|ready)\b",
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


def canonical_json_bytes(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def sha256_digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical_json_bytes(value)).hexdigest()


def normalized_query(value: object) -> str:
    return str(value).replace("\r\n", "\n").replace("\r", "\n").strip()


def measured_window_query_digest_payload(receipt: dict) -> dict:
    return {
        "datasource": receipt.get("datasource"),
        "query_language": receipt.get("query_language", "promql"),
        "slo_id": receipt.get("slo_id"),
        "indicator": receipt.get("indicator"),
        "window_start": receipt.get("window_start"),
        "window_end": receipt.get("window_end"),
        "numerator_query": normalized_query(receipt.get("numerator_query", "")),
        "denominator_query": normalized_query(receipt.get("denominator_query", "")),
        "telemetry_storage_boundary": receipt.get(
            "telemetry_storage_boundary",
            "dogfood telemetry storage only; no external hyperscaler console or unmanaged tenant system",
        ),
    }


def measured_window_receipt_violations(receipt: object) -> list[str]:
    if not isinstance(receipt, dict):
        return ["measured_slo_window receipt must be an object"]
    violations = [
        f"missing required field: {field}"
        for field in sorted(REQUIRED_MEASURED_WINDOW_RECEIPT_FIELDS)
        if field not in receipt
    ]
    datasource = str(receipt.get("datasource", ""))
    datasource_normalized = normalized(datasource)
    if datasource and "dogfood" not in datasource_normalized:
        violations.append("datasource must be Oyatie-owned dogfood telemetry storage")
    forbidden_source_phrases = [
        "external hyperscaler",
        "provider console",
        "aws console",
        "gcp console",
        "azure portal",
        "unmanaged tenant",
    ]
    if any(phrase in datasource_normalized for phrase in forbidden_source_phrases):
        violations.append("datasource must reject external hyperscaler consoles and unmanaged tenant systems")
    for field in ("query_digest", "evidence_digest"):
        value = receipt.get(field)
        if value is not None and not HEX_DIGEST.match(str(value)):
            violations.append(f"{field} must match sha256:<64 hex>")
    if "artifact_digest" in receipt and not HEX_DIGEST.match(str(receipt.get("artifact_digest", ""))):
        violations.append("artifact_digest must match sha256:<64 hex>")
    if "source_commit" in receipt and not GIT_COMMIT_SHA.match(str(receipt.get("source_commit", ""))):
        violations.append("source_commit must be a 40-hex git commit SHA")
    if "reviewer" in receipt and not str(receipt.get("reviewer", "")).strip():
        violations.append("reviewer must be non-empty")
    if "query_digest" in receipt:
        expected_query_digest = sha256_digest(measured_window_query_digest_payload(receipt))
        if receipt.get("query_digest") != expected_query_digest:
            violations.append("query_digest must match canonical dogfood telemetry query payload")
    if "evidence_digest" in receipt:
        evidence_payload = dict(receipt)
        evidence_payload.pop("evidence_digest", None)
        expected_evidence_digest = sha256_digest(evidence_payload)
        if receipt.get("evidence_digest") != expected_evidence_digest:
            violations.append("evidence_digest must match canonical receipt with evidence_digest omitted")
    for field in ("measured_value", "error_budget_remaining", "burn_rate"):
        value = receipt.get(field)
        if isinstance(value, dict) and value.get("status") == "not_measured_yet":
            if value.get("explicit_na") is not True:
                violations.append(f"{field} explicit N/A must set explicit_na=true")
        elif value is None and field in receipt:
            violations.append(f"{field} must be explicit N/A object when not implemented")
        elif field in receipt and not isinstance(value, dict):
            violations.append(f"{field} must remain an explicit object in metadata-only producer output")
    return violations


def validate_measured_slo_window_query_digest_receipt_producer(producer: object, rows: list[dict]) -> None:
    require(isinstance(producer, dict), "measured SLO window query-digest producer packet must be an object")
    producer = dict(producer)
    require(producer.get("status") == "green_shape_valid_not_measured", "measured SLO producer status must be GREEN shape only without measured claims")
    require(producer.get("receipt_type") == "measured_slo_window", "measured SLO producer receipt_type mismatch")
    require(producer.get("owner_task") == "t_abde9192", "measured SLO producer owner_task must identify this BUILD/GREEN card")
    require(producer.get("claim_tier") == "target_non_claim/metadata_green_receipt_shape", "measured SLO producer claim_tier must stay target/non-claim GREEN shape")

    schema = producer.get("receipt_schema", {})
    require(REQUIRED_MEASURED_WINDOW_RECEIPT_FIELDS <= set(schema.get("required_fields", [])), "measured SLO receipt required_fields incomplete")
    require(REQUIRED_MEASURED_WINDOW_RECEIPT_METADATA_FIELDS <= set(schema.get("mandatory_receipt_metadata", [])), "measured SLO receipt metadata fields incomplete")
    require({"measured_value", "error_budget_remaining", "burn_rate"} <= set(schema.get("explicit_na_fields_until_measured", [])), "measured SLO receipt must keep not-yet-measured values explicit")

    rules = producer.get("digest_rules", {})
    boundary = text(rules.get("dogfood_storage_boundary", ""))
    require("dogfood" in boundary and "external hyperscaler" in boundary and "unmanaged tenant" in boundary, "measured SLO producer dogfood boundary must reject external/unmanaged sources")
    query_payload_fields = set(rules.get("query_digest_payload_fields", []))
    require({"datasource", "query_language", "slo_id", "indicator", "window_start", "window_end", "numerator_query", "denominator_query", "telemetry_storage_boundary"} <= query_payload_fields, "query_digest payload fields incomplete")
    require(rules.get("evidence_digest_omits") == "evidence_digest", "evidence_digest rule must omit evidence_digest while hashing")

    controls = producer.get("claim_ceiling", {})
    for key in ["no_measured_slo_claim", "no_public_sla_slo_claim", "no_production_readiness_claim", "no_tenant_workload_readiness", "no_hyperscaler_maturity_claim"]:
        require(controls.get(key) is True, f"measured SLO producer claim ceiling {key} must remain true")

    green_violations = measured_window_receipt_violations(producer.get("example_green_receipt", {}))
    require(not green_violations, "measured SLO GREEN receipt invalid: " + "; ".join(green_violations))
    red = producer.get("red_fixture_assertion", {})
    expected_field_names = set(red.get("expected_missing_fields", []))
    require(expected_field_names == {"window_start", "window_end", "query_digest", "reviewer"}, "measured SLO RED fixture must assert window_start/window_end/query_digest/reviewer")
    expected_missing = {f"missing required field: {field}" for field in expected_field_names}
    red_violations = set(measured_window_receipt_violations(red.get("bad_missing_required_fields", {})))
    require(expected_missing <= red_violations, f"measured SLO RED fixture should fail for {sorted(expected_missing)}, got {sorted(red_violations)}")

    rows_by_id = {row["resource_contract_id"]: row for row in rows}
    row_map = producer.get("row_to_producer_map", [])
    require(isinstance(row_map, list) and row_map, "measured SLO row_to_producer_map must be non-empty")
    mapped_ids = {entry.get("resource_contract_id") for entry in row_map}
    require(mapped_ids == set(rows_by_id), f"measured SLO row_to_producer_map coverage mismatch: {sorted(set(rows_by_id) ^ mapped_ids)}")
    for entry in row_map:
        cid = entry.get("resource_contract_id")
        source = rows_by_id[cid]
        require(entry.get("service") == source["service"], f"{cid}: measured SLO producer map service mismatch")
        require(entry.get("category_id") == source["category_id"], f"{cid}: measured SLO producer map category mismatch")
        require(entry.get("resource_type") == source["resource_type"], f"{cid}: measured SLO producer map resource_type mismatch")
        require(entry.get("openslo_artifact_pattern") == source["slo_profile"]["artifact_pattern"], f"{cid}: measured SLO OpenSLO artifact pattern mismatch")
        require(str(entry.get("measured_window_receipt_pattern", "")).startswith("evidence/observability/"), f"{cid}: measured SLO receipt pattern must stay in evidence/observability")
        require(entry.get("receipt_type") == "measured_slo_window", f"{cid}: measured SLO producer map receipt_type mismatch")
        require(entry.get("producer_status") == "green_shape_valid_not_measured", f"{cid}: measured SLO producer map must remain GREEN shape/not-measured")
        require("dogfood" in text(entry.get("datasource_scope", "")), f"{cid}: measured SLO producer map must use dogfood telemetry scope")

    categories = {row["category_id"] for row in rows}
    category_rollup = producer.get("category_rollup_coverage", [])
    require({entry.get("category_id") for entry in category_rollup} == categories, "measured SLO category rollup coverage mismatch")
    for entry in category_rollup:
        category_id = entry["category_id"]
        expected_ids = {row["resource_contract_id"] for row in rows if row["category_id"] == category_id}
        require(set(entry.get("resource_contract_ids", [])) == expected_ids, f"{category_id}: measured SLO rollup resource ids mismatch")
        require(entry.get("producer_status") == "green_shape_valid_not_measured", f"{category_id}: measured SLO rollup must remain GREEN shape/not-measured")


def missing_otel_receipt_sample_fields(sample: object) -> list[str]:
    if not isinstance(sample, dict):
        return sorted(REQUIRED_OTEL_SAMPLE_RECEIPT_FIELDS)
    missing = {
        field
        for field in REQUIRED_OTEL_SAMPLE_RECEIPT_FIELDS
        if not sample.get(field)
    }
    for block in REQUIRED_OTEL_SAMPLE_BLOCKS:
        if not isinstance(sample.get(block), dict) or not sample.get(block):
            missing.add(block)
    return sorted(missing)


def validate_otel_trace_metric_log_sample_receipt_producer(producer: object, rows: list[dict]) -> None:
    require(isinstance(producer, dict), "OTel sample receipt producer packet must be an object")
    require(producer.get("status") == "plan_spec_red_only", "OTel sample producer status must remain plan_spec_red_only")
    require(producer.get("receipt_type") == "otel_trace_metric_log_sample", "OTel sample producer receipt_type mismatch")
    require(producer.get("owner_task") == "t_5b8f743d", "OTel sample producer owner_task must identify this Plan/Spec/RED card")
    require(producer.get("claim_tier") == "target_non_claim/spec_ready_fixture_plan", "OTel sample producer claim_tier must stay target/spec/RED")

    schema = producer.get("receipt_schema", {})
    require(REQUIRED_OTEL_SAMPLE_RECEIPT_FIELDS <= set(schema.get("required_fields", [])), "OTel sample receipt required_fields incomplete")
    require(REQUIRED_OTEL_RESOURCE_ATTRIBUTES <= set(schema.get("required_resource_attributes", [])), "OTel sample receipt resource attributes incomplete")
    require(REQUIRED_OTEL_SAMPLE_RECEIPT_CORRELATION_FIELDS <= set(schema.get("required_correlation_fields", [])), "OTel sample receipt correlation fields incomplete")
    require(REQUIRED_OTEL_SAMPLE_BLOCKS <= set(schema.get("required_sample_blocks", [])), "OTel sample receipt blocks incomplete")
    require(REQUIRED_OTEL_SAMPLE_RECEIPT_METADATA_FIELDS <= set(schema.get("mandatory_receipt_metadata", [])), "OTel sample receipt metadata fields incomplete")

    future = producer.get("future_evidence_receipt_fields", {})
    require(REQUIRED_EVIDENCE_BEFORE_CLAIM <= set(future), "OTel producer must name every evidence receipt required before claim")
    require(set(future.get("burn_rate_windows", [])) == {"5m", "30m", "1h", "2h", "6h", "1d", "3d"}, "OTel producer burn-rate windows mismatch")
    for receipt_name, fields in future.items():
        if receipt_name.endswith("receipt") or receipt_name.endswith("sample") or receipt_name == "measured_slo_window":
            require(REQUIRED_OTEL_SAMPLE_RECEIPT_METADATA_FIELDS <= set(fields), f"{receipt_name}: source_commit/artifact_digest/reviewer must be named where applicable")

    good_missing = missing_otel_receipt_sample_fields(producer.get("example_good_receipt", {}))
    require(not good_missing, f"OTel sample good fixture missing fields {good_missing}")
    red = producer.get("red_fixture_assertion", {})
    expected_missing = set(red.get("expected_missing_fields", []))
    require(expected_missing == REQUIRED_OTEL_SAMPLE_RECEIPT_CORRELATION_FIELDS, "OTel sample RED fixture must assert trace_id/span_id/operation_id/resource_orn")
    red_missing = set(missing_otel_receipt_sample_fields(red.get("bad_missing_required_fields", {})))
    require(red_missing == expected_missing, f"OTel sample RED fixture should fail only for {sorted(expected_missing)}, got {sorted(red_missing)}")

    rows_by_id = {row["resource_contract_id"]: row for row in rows}
    row_map = producer.get("row_to_producer_map", [])
    require(isinstance(row_map, list) and row_map, "OTel sample row_to_producer_map must be non-empty")
    mapped_ids = {entry.get("resource_contract_id") for entry in row_map}
    require(mapped_ids == set(rows_by_id), f"OTel sample row_to_producer_map coverage mismatch: {sorted(set(rows_by_id) ^ mapped_ids)}")
    for entry in row_map:
        cid = entry.get("resource_contract_id")
        source = rows_by_id[cid]
        require(entry.get("service") == source["service"], f"{cid}: OTel producer map service mismatch")
        require(entry.get("category_id") == source["category_id"], f"{cid}: OTel producer map category mismatch")
        require(entry.get("resource_type") == source["resource_type"], f"{cid}: OTel producer map resource_type mismatch")
        require(entry.get("receipt_type") == "otel_trace_metric_log_sample", f"{cid}: OTel producer map receipt_type mismatch")
        require(entry.get("producer_status") == "plan_spec_red_only", f"{cid}: OTel producer map must remain plan/spec/RED only")
        require(str(entry.get("otel_receipt_producer", "")).startswith(f"{source['service']}."), f"{cid}: OTel producer must be service-scoped")
        require({"trace", "metric", "structured_log"} <= set(entry.get("sample_kinds", [])), f"{cid}: OTel producer map must name trace/metric/log samples")

    expected_pairs = {(row["category_id"], row["service"]) for row in rows}
    rollup = producer.get("category_service_rollup", [])
    mapped_pairs = {(entry.get("category_id"), entry.get("service")) for entry in rollup}
    require(mapped_pairs == expected_pairs, f"OTel sample category/service rollup mismatch: {sorted(expected_pairs ^ mapped_pairs)}")
    for entry in rollup:
        category_id = entry["category_id"]
        service = entry["service"]
        expected_ids = {row["resource_contract_id"] for row in rows if row["category_id"] == category_id and row["service"] == service}
        require(set(entry.get("resource_contract_ids", [])) == expected_ids, f"{category_id}/{service}: OTel rollup resource ids mismatch")
        require(entry.get("producer_status") == "plan_spec_red_only", f"{category_id}/{service}: OTel rollup must remain plan/spec/RED only")



def audit_chain_receipt_violations(receipt: object) -> list[str]:
    if not isinstance(receipt, dict):
        return ["audit_chain_correlation_receipt must be an object"]
    violations = [
        f"missing required field: {field}"
        for field in sorted(REQUIRED_AUDIT_CHAIN_RECEIPT_FIELDS)
        if field not in receipt
    ]
    if receipt.get("receipt_type") not in (None, "audit_chain_correlation_receipt"):
        violations.append("receipt_type must be audit_chain_correlation_receipt")
    for field in ("artifact_digest", "evidence_digest"):
        value = receipt.get(field)
        if value is not None and not HEX_DIGEST.match(str(value)):
            violations.append(f"{field} must match sha256:<64 hex>")

    policy = receipt.get("policy_snapshot")
    if "policy_snapshot" in receipt:
        if not isinstance(policy, dict):
            violations.append("policy_snapshot must be an object")
        else:
            for field in sorted(REQUIRED_AUDIT_CHAIN_POLICY_SNAPSHOT_FIELDS):
                if field not in policy:
                    violations.append(f"missing policy_snapshot.{field}")

    correlated = receipt.get("correlated_receipts")
    if "correlated_receipts" in receipt:
        if not isinstance(correlated, dict):
            violations.append("correlated_receipts must be an object")
        else:
            missing_receipts = REQUIRED_AUDIT_CHAIN_CORRELATED_RECEIPTS - set(correlated)
            for family in sorted(missing_receipts):
                violations.append(f"missing correlated receipt: {family}")
            for family in sorted(REQUIRED_AUDIT_CHAIN_CORRELATED_RECEIPTS & set(correlated)):
                child = correlated.get(family)
                if not isinstance(child, dict):
                    violations.append(f"{family} must be an object")
                    continue
                if child.get("receipt_type") not in (None, family):
                    violations.append(f"{family}.receipt_type must match {family}")
                for field in sorted(REQUIRED_AUDIT_CHAIN_RECEIPT_METADATA_FIELDS):
                    if field not in child:
                        violations.append(f"missing {family}.{field}")
                for field in sorted(REQUIRED_AUDIT_CHAIN_CONSISTENCY_FIELDS):
                    if field not in child:
                        violations.append(f"missing {family}.{field}")
                    elif field in receipt and child.get(field) != receipt.get(field):
                        violations.append(f"inconsistent {family}.{field}")
    return violations


def validate_audit_chain_correlation_receipt_producer(producer: object, rows: list[dict]) -> None:
    if not isinstance(producer, dict):
        fail("audit-chain correlation receipt producer packet must be an object")
    require(producer.get("status") == "green_shape_valid_not_live", "audit-chain producer status must be GREEN shape only without live runtime claims")
    require(producer.get("receipt_type") == "audit_chain_correlation_receipt", "audit-chain producer receipt_type mismatch")
    require(producer.get("owner_task") == "t_22dd6002", "audit-chain producer owner_task must identify this BUILD/GREEN card")
    require(producer.get("claim_tier") == "target_non_claim/metadata_green_receipt_shape", "audit-chain producer claim_tier must stay target/non-claim GREEN shape")

    controls = producer.get("claim_ceiling", {})
    for key in [
        "metadata_only",
        "evidence_contract_only",
        "no_audit_chain_runtime_append_claim",
        "no_runtime_observability_engine",
        "no_measured_slo_claim",
        "no_public_sla_slo_claim",
        "no_status_page_or_incident_workflow_claim",
        "no_production_readiness_claim",
        "no_tenant_workload_readiness",
        "no_hyperscaler_maturity_claim",
    ]:
        require(controls.get(key) is True, f"audit-chain producer claim ceiling {key} must remain true")

    schema = producer.get("receipt_schema", {})
    require(REQUIRED_AUDIT_CHAIN_RECEIPT_FIELDS <= set(schema.get("required_fields", [])), "audit-chain receipt required_fields incomplete")
    require(REQUIRED_AUDIT_CHAIN_RECEIPT_METADATA_FIELDS <= set(schema.get("mandatory_receipt_metadata", [])), "audit-chain receipt metadata fields incomplete")
    require(REQUIRED_AUDIT_CHAIN_POLICY_SNAPSHOT_FIELDS <= set(schema.get("required_policy_snapshot_fields", [])), "audit-chain policy_snapshot fields incomplete")
    require(REQUIRED_AUDIT_CHAIN_RECEIPT_FAMILIES <= set(schema.get("minimum_receipt_families", [])), "audit-chain receipt families incomplete")
    require(REQUIRED_AUDIT_CHAIN_CORRELATED_RECEIPTS <= set(schema.get("correlated_receipts_required", [])), "audit-chain correlated receipt requirements incomplete")
    require(REQUIRED_AUDIT_CHAIN_CONSISTENCY_FIELDS <= set(schema.get("consistency_tuple_fields", [])), "audit-chain consistency tuple fields incomplete")

    green_violations = audit_chain_receipt_violations(producer.get("example_green_receipt", {}))
    require(not green_violations, "audit-chain GREEN receipt invalid: " + "; ".join(green_violations))
    red_expected = producer.get("red_fixture_assertion", {}).get("expected_violations_by_fixture", {})
    require(set(red_expected) == {"missing_audit_chain_id", "mismatched_operation_trace_resource", "absent_policy_snapshot", "absent_common_provenance"}, "audit-chain RED fixtures must cover missing audit_chain_id, mismatch, policy_snapshot, and provenance")
    require(set(red_expected.get("missing_audit_chain_id", [])) == {"missing required field: audit_chain_id"}, "audit-chain missing-audit-chain-id RED assertion mismatch")
    require({"inconsistent operation_outcome_receipt.operation_id", "inconsistent lifecycle_transition_receipt.resource_orn", "inconsistent slo_gate_receipt.trace_id"} <= set(red_expected.get("mismatched_operation_trace_resource", [])), "audit-chain mismatch RED assertion incomplete")
    require(set(red_expected.get("absent_policy_snapshot", [])) == {"missing required field: policy_snapshot"}, "audit-chain absent-policy RED assertion mismatch")
    require({"missing required field: source_commit", "missing required field: artifact_digest", "missing required field: reviewer"} <= set(red_expected.get("absent_common_provenance", [])), "audit-chain provenance RED assertion incomplete")

    rows_by_id = {row["resource_contract_id"]: row for row in rows}
    row_map = producer.get("row_to_producer_map", [])
    require(isinstance(row_map, list) and row_map, "audit-chain row_to_producer_map must be non-empty")
    mapped_ids = {entry.get("resource_contract_id") for entry in row_map}
    require(mapped_ids == set(rows_by_id), f"audit-chain row_to_producer_map coverage mismatch: {sorted(set(rows_by_id) ^ mapped_ids)}")
    for entry in row_map:
        cid = entry.get("resource_contract_id")
        source = rows_by_id[cid]
        require(entry.get("service") == source["service"], f"{cid}: audit-chain producer map service mismatch")
        require(entry.get("category_id") == source["category_id"], f"{cid}: audit-chain producer map category mismatch")
        require(entry.get("resource_type") == source["resource_type"], f"{cid}: audit-chain producer map resource_type mismatch")
        require(entry.get("receipt_type") == "audit_chain_correlation_receipt", f"{cid}: audit-chain producer map receipt_type mismatch")
        require(entry.get("producer_status") == "green_shape_valid_not_live", f"{cid}: audit-chain producer map must remain GREEN shape/not-live")
        require(str(entry.get("audit_chain_receipt_pattern", "")).startswith("evidence/audit-chain/"), f"{cid}: audit-chain receipt pattern must stay in evidence/audit-chain")
        require(REQUIRED_AUDIT_CHAIN_RECEIPT_FAMILIES <= set(entry.get("receipt_families", [])), f"{cid}: audit-chain producer map must name every minimum receipt family")

    categories = {row["category_id"] for row in rows}
    category_rollup = producer.get("category_rollup_coverage", [])
    require({entry.get("category_id") for entry in category_rollup} == categories, "audit-chain category rollup coverage mismatch")
    for entry in category_rollup:
        category_id = entry["category_id"]
        expected_ids = {row["resource_contract_id"] for row in rows if row["category_id"] == category_id}
        require(set(entry.get("resource_contract_ids", [])) == expected_ids, f"{category_id}: audit-chain rollup resource ids mismatch")
        require(entry.get("producer_status") == "green_shape_valid_not_live", f"{category_id}: audit-chain rollup must remain GREEN shape/not-live")
        for pattern in entry.get("audit_chain_receipt_patterns", []):
            require(str(pattern).startswith("evidence/audit-chain/"), f"{category_id}: audit-chain rollup patterns must stay in evidence/audit-chain")

    consumers = producer.get("consumer_boundaries", {})
    for consumer in ("SREOPS", "TrustCenter", "RESILIENCE"):
        require(consumer in consumers and "does not" in text(consumers[consumer]), f"audit-chain consumer boundary for {consumer} must remain non-live")



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
    measured_producer = candidate.get("measured_slo_window_query_digest_receipt_producer", {})
    for key in (
        "claim_ceiling",
        "source_citations",
        "red_fixture_assertion",
        "example_green_receipt",
        "future_implementation_boundary",
        "status",
        "receipt_type",
        "row_to_producer_map",
        "category_rollup_coverage",
    ):
        measured_producer.pop(key, None)
    measured_producer.get("digest_rules", {}).pop("forbidden_sources", None)
    candidate.pop("audit_chain_correlation_receipt_producer", None)
    producer = candidate.get("otel_trace_metric_log_sample_receipt_producer", {})
    for key in ("claim_ceiling", "source_citations", "prerequisite_overlap_cards"):
        producer.pop(key, None)
    producer.get("red_fixture_assertion", {}).pop("assertion", None)
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
        "measured_slo_window_query_digest_receipt_producer",
        "otel_trace_metric_log_sample_receipt_producer",
        "audit_chain_correlation_receipt_producer",
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

    validate_measured_slo_window_query_digest_receipt_producer(spec["measured_slo_window_query_digest_receipt_producer"], rows)
    validate_otel_trace_metric_log_sample_receipt_producer(spec["otel_trace_metric_log_sample_receipt_producer"], rows)
    validate_audit_chain_correlation_receipt_producer(spec["audit_chain_correlation_receipt_producer"], rows)

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


def describe_otel_red_fixture() -> None:
    producer = load_json(SPEC_PATH)["otel_trace_metric_log_sample_receipt_producer"]
    red = producer["red_fixture_assertion"]
    missing = missing_otel_receipt_sample_fields(red["bad_missing_required_fields"])
    expected = sorted(red["expected_missing_fields"])
    require(missing == expected, f"OTel RED fixture mismatch: expected {expected}, got {missing}")
    print(f"otel trace/metric/log sample RED fixture rejected: missing {', '.join(missing)}")



def describe_audit_chain_red_fixtures() -> None:
    producer = load_json(SPEC_PATH)["audit_chain_correlation_receipt_producer"]
    expected = producer["red_fixture_assertion"]["expected_violations_by_fixture"]
    for fixture_id, violations in sorted(expected.items()):
        require(violations, f"audit-chain RED fixture {fixture_id} must name expected violations")
        print(f"audit-chain correlation RED fixture {fixture_id}: {'; '.join(violations)}")



def expect_audit_chain_receipt_valid(path: Path) -> None:
    violations = audit_chain_receipt_violations(load_json(path))
    require(not violations, "audit-chain correlation receipt invalid: " + "; ".join(violations))
    print(f"valid audit_chain_correlation_receipt: {path.name}")



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

    def mutate_measured_green_receipt(data: dict, updates: dict) -> None:
        receipt = data["measured_slo_window_query_digest_receipt_producer"]["example_green_receipt"]
        receipt.update(updates)
        receipt["query_digest"] = sha256_digest(measured_window_query_digest_payload(receipt))
        evidence_payload = dict(receipt)
        evidence_payload.pop("evidence_digest", None)
        receipt["evidence_digest"] = sha256_digest(evidence_payload)

    expect_rejected("missing contract row", lambda data: data.update({"resource_observability": data["resource_observability"][1:]}))
    expect_rejected("missing measured SLO query-digest producer packet", lambda data: data.pop("measured_slo_window_query_digest_receipt_producer", None))
    expect_rejected("missing measured SLO producer row", lambda data: data["measured_slo_window_query_digest_receipt_producer"].update({"row_to_producer_map": data["measured_slo_window_query_digest_receipt_producer"]["row_to_producer_map"][1:]}))
    expect_rejected("measured SLO external datasource", lambda data: mutate_measured_green_receipt(data, {"datasource": "external_hyperscaler_console:aws"}))
    expect_rejected("measured SLO spaced external datasource", lambda data: mutate_measured_green_receipt(data, {"datasource": "oyatie_dogfood external hyperscaler console"}))
    expect_rejected("measured SLO spaced unmanaged tenant datasource", lambda data: mutate_measured_green_receipt(data, {"datasource": "oyatie_dogfood unmanaged tenant system"}))
    expect_rejected("measured SLO empty source commit", lambda data: mutate_measured_green_receipt(data, {"source_commit": ""}))
    expect_rejected("measured SLO malformed artifact digest", lambda data: mutate_measured_green_receipt(data, {"artifact_digest": "not-a-sha256-digest"}))
    expect_rejected("measured SLO empty reviewer", lambda data: mutate_measured_green_receipt(data, {"reviewer": ""}))
    expect_rejected("measured SLO query digest tamper", lambda data: data["measured_slo_window_query_digest_receipt_producer"]["example_green_receipt"].update({"query_digest": "sha256:" + "0" * 64}))
    expect_rejected("measured SLO evidence digest tamper", lambda data: data["measured_slo_window_query_digest_receipt_producer"]["example_green_receipt"].update({"evidence_digest": "sha256:" + "0" * 64}))
    expect_rejected("measured SLO RED fixture incomplete expected fields", lambda data: data["measured_slo_window_query_digest_receipt_producer"]["red_fixture_assertion"].update({"expected_missing_fields": ["query_digest"]}))
    expect_rejected("missing OTel receipt producer packet", lambda data: data.pop("otel_trace_metric_log_sample_receipt_producer", None))
    expect_rejected("missing OTel receipt producer row", lambda data: data["otel_trace_metric_log_sample_receipt_producer"].update({"row_to_producer_map": data["otel_trace_metric_log_sample_receipt_producer"]["row_to_producer_map"][1:]}))
    expect_rejected("OTel good receipt missing trace_id", lambda data: data["otel_trace_metric_log_sample_receipt_producer"]["example_good_receipt"].pop("trace_id"))
    expect_rejected("OTel RED fixture incomplete expected fields", lambda data: data["otel_trace_metric_log_sample_receipt_producer"]["red_fixture_assertion"].update({"expected_missing_fields": ["trace_id"]}))
    expect_rejected("missing audit-chain receipt producer packet", lambda data: data.pop("audit_chain_correlation_receipt_producer", None))
    expect_rejected("missing audit-chain producer row", lambda data: data["audit_chain_correlation_receipt_producer"].update({"row_to_producer_map": data["audit_chain_correlation_receipt_producer"]["row_to_producer_map"][1:]}))
    expect_rejected("audit-chain GREEN missing audit_chain_id", lambda data: data["audit_chain_correlation_receipt_producer"]["example_green_receipt"].pop("audit_chain_id"))
    expect_rejected("audit-chain mismatched operation id", lambda data: data["audit_chain_correlation_receipt_producer"]["example_green_receipt"]["correlated_receipts"]["operation_outcome_receipt"].update({"operation_id": "op-mismatched"}))
    expect_rejected("audit-chain missing policy snapshot", lambda data: data["audit_chain_correlation_receipt_producer"]["example_green_receipt"].pop("policy_snapshot"))
    expect_rejected("audit-chain missing common provenance", lambda data: data["audit_chain_correlation_receipt_producer"]["example_green_receipt"].pop("reviewer"))
    expect_rejected("audit-chain RED fixture incomplete expected fields", lambda data: data["audit_chain_correlation_receipt_producer"]["red_fixture_assertion"].update({"expected_violations_by_fixture": {"missing_audit_chain_id": []}}))

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
    expect_rejected("runtime exporter overclaim", lambda data: data.update({"purpose": "runtime exporter is implemented"}))
    expect_rejected("public service level agreement synonym", lambda data: data["claim_controls"].update({"can_claim_now": ["public service level agreement ready"]}))
    expect_rejected("prod customer traffic synonym", lambda data: data["gate_alignment"]["HG-OPS"].update({"note": "prod ready for customer traffic"}))
    expect_rejected("hyperscale grade synonym", lambda data: data["doubt_driven_review"].update({"resolution": "hyperscale grade readiness established"}))
    expect_rejected("missing HG-OBS alignment", lambda data: data["gate_alignment"].pop("HG-OBS"))
    expect_rejected("incomplete HG-OPS alignment", lambda data: data["gate_alignment"]["HG-OPS"].update({"source_requires": ["progressive_delivery_or_explicit_nonproduction_scope"]}))
    expect_rejected("missing HG-OPS requirement mapping", lambda data: data["gate_alignment"]["HG-OPS"].update({"requirement_mapping": {"progressive_delivery_or_explicit_nonproduction_scope": ["slo_profile.promotion_gate"]}}))
    expect_rejected("category rollup mismatch", lambda data: data["category_rollup"][data["resource_observability"][0]["category_id"]].update({"resource_contract_ids": []}))
    print("cloud observability SLO evidence self-tests passed")


if __name__ == "__main__":
    args = sys.argv[1:]
    if "--self-test" in args:
        run_self_tests()
    if "--show-otel-red-fixture" in args:
        describe_otel_red_fixture()
    if "--show-audit-chain-red-fixtures" in args:
        describe_audit_chain_red_fixtures()
    if "--expect-audit-chain-receipt-valid" in args:
        index = args.index("--expect-audit-chain-receipt-valid")
        try:
            fixture_path = Path(args[index + 1])
        except IndexError:
            fail("--expect-audit-chain-receipt-valid requires a fixture path")
        expect_audit_chain_receipt_valid(fixture_path)
    main()
