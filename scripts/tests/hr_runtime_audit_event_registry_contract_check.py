#!/usr/bin/env python3
"""Validate the HR runtime audit-chain event-class registry contract.

This is a Plan/Spec/RED guard for t_3a7d8b2c. It proves that current HR
artifacts still emit only metadata envelopes while requiring the audit-chain-owned
registry to name the minimal event classes HR needs before later producer cards
write runtime audit-chain rows.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
REGISTRY_PATH = REPO_ROOT / "specs" / "audit-event-class-registry.json"
HR_PRD_PATH = REPO_ROOT / "specs" / "microservices" / "hr.json"
HR_APP_PATH = REPO_ROOT / "oya" / "hr" / "crates" / "oya-hr-employment-app" / "src" / "lib.rs"
HR_INFRA_PATH = REPO_ROOT / "oya" / "hr" / "crates" / "oya-hr-employment-infrastructure" / "src" / "lib.rs"
HR_STORAGE_PATH = REPO_ROOT / "oya" / "hr" / "crates" / "oya-hr-employment-storage-adapter-inmemory" / "src" / "lib.rs"

REQUIRED_ENVELOPE_FIELDS = {
    "tenant_id",
    "audit_id",
    "schema_version",
    "source_microservice",
    "cell_id",
    "jurisdiction_code",
    "trace_id",
    "span_id",
    "event_id",
    "sub_scope_path",
    "hlc_timestamp",
    "cost_usd_minor_units",
    "co2_grams",
    "watt_hours",
    "provider",
    "region",
    "carbon_intensity_source",
}

REQUIRED_HR_CLASSES: dict[str, dict[str, Any]] = {
    "HrLifecycleAuditPrepared": {
        "retention_class": "regulated",
        "cedar_decision_event": False,
        "payload_fields": {
            "legal_entity_id",
            "employee_id",
            "lifecycle_kind",
            "aggregate_ref",
            "evidence_ref",
            "idempotency_key",
            "payload_data_class",
        },
        "dimension_keys": {"tenant_id", "source_microservice", "legal_entity_id", "lifecycle_kind"},
    },
    "HrSensitiveReadPolicyEvaluated": {
        "retention_class": "legal_hold_capable",
        "cedar_decision_event": True,
        "payload_fields": {
            "legal_entity_id",
            "actor_employee_id",
            "subject_employee_id",
            "data_kind",
            "purpose",
            "legal_basis",
            "policy_ref",
            "basis_evidence_ref",
            "request_evidence_ref",
            "read_log_evidence_ref",
            "decision_status",
            "payload_data_class",
        },
        "dimension_keys": {"tenant_id", "source_microservice", "legal_entity_id", "data_kind", "decision_status"},
    },
    "HrWorkflowDispatchPlanned": {
        "retention_class": "regulated",
        "cedar_decision_event": False,
        "payload_fields": {
            "legal_entity_id",
            "workflow_ref",
            "obligation_kind",
            "jurisdiction",
            "required_step_count",
            "evidence_ref_count",
            "idempotency_key",
        },
        "dimension_keys": {"tenant_id", "source_microservice", "legal_entity_id", "obligation_kind"},
    },
    "HrLeavePayrollImpactPlanned": {
        "retention_class": "financial_control",
        "cedar_decision_event": False,
        "payload_fields": {
            "legal_entity_id",
            "employee_id",
            "leave_request_id",
            "approver_id",
            "decision",
            "routing_mode",
            "workflow_ref",
            "payroll_period",
            "payroll_impact_kind",
            "decision_evidence_ref",
            "routing_evidence_ref",
            "payroll_impact_evidence_ref",
            "idempotency_key",
        },
        "dimension_keys": {"tenant_id", "source_microservice", "legal_entity_id", "payroll_period", "payroll_impact_kind"},
    },
    "HrStatutoryFilingManifestPrepared": {
        "retention_class": "regulated",
        "cedar_decision_event": False,
        "payload_fields": {
            "legal_entity_id",
            "filing_kind",
            "rulepack_ref",
            "rulepack_effective_date",
            "source_version",
            "approval_evidence_ref",
            "filing_evidence_ref",
            "object_store_digest_ref",
            "idempotency_key",
        },
        "dimension_keys": {"tenant_id", "source_microservice", "legal_entity_id", "filing_kind"},
    },
}

RUNTIME_PRODUCER_FORBIDDEN = [
    re.compile(r"evidence/audit-chain\.jsonl"),
    re.compile(r"append_(audit|event)", re.IGNORECASE),
    re.compile(r"emit_(runtime_)?audit", re.IGNORECASE),
    re.compile(r"audit_chain_.*(append|emit|producer)", re.IGNORECASE),
    re.compile(r"worm.*put", re.IGNORECASE),
    re.compile(r"object_store.*put", re.IGNORECASE),
]


def fail(message: str) -> NoReturn:
    print(f"HR runtime audit event registry contract check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def rel(path: Path) -> str:
    return str(path.relative_to(REPO_ROOT))


def load_json(path: Path) -> dict[str, Any]:
    try:
        candidate = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {rel(path)}: {exc}")
    require(isinstance(candidate, dict), f"{rel(path)} must be a JSON object")
    return candidate


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing {rel(path)}")


def normalized_boundary_text(text: str) -> str:
    """Collapse Rust doc-comment wrapping before checking boundary claims."""
    return re.sub(r"\s+", " ", text.lower().replace("//!", " "))


def registry_event_enum(registry: dict[str, Any]) -> set[str]:
    enum_values = registry.get("definitions", {}).get("eventClassName", {}).get("enum", [])
    require(isinstance(enum_values, list), "registry definitions.eventClassName.enum must be a list")
    return {str(value) for value in enum_values}


def registered_class_rows(registry: dict[str, Any]) -> dict[str, dict[str, Any]]:
    rows = registry.get("x-registered_classes", [])
    require(isinstance(rows, list), "registry x-registered_classes must be a list")
    by_name: dict[str, dict[str, Any]] = {}
    for row in rows:
        require(isinstance(row, dict), "each x-registered_classes row must be an object")
        event_class = str(row.get("event_class", ""))
        require(event_class, "x-registered_classes row missing event_class")
        require(event_class not in by_name, f"duplicate x-registered_classes row {event_class}")
        by_name[event_class] = row
    return by_name


def registry_examples_by_name(registry: dict[str, Any]) -> dict[str, dict[str, Any]]:
    examples = registry.get("examples", [])
    require(isinstance(examples, list), "registry examples must be a list")
    by_name: dict[str, dict[str, Any]] = {}
    for example in examples:
        require(isinstance(example, dict), "each registry example must be an object")
        for class_row in example.get("classes", []):
            require(isinstance(class_row, dict), "each example class row must be an object")
            event_class = str(class_row.get("event_class", ""))
            if event_class:
                require(event_class not in by_name, f"duplicate example class row {event_class}")
                by_name[event_class] = {"example": example, "class_row": class_row}
    return by_name


def validate_hr_prd(prd: dict[str, Any]) -> None:
    decisions = {str(item.get("id")): str(item.get("decision", "")) for item in prd.get("decision_log", []) if isinstance(item, dict)}
    for decision_id in ["D-11", "D-12", "D-13", "D-14", "D-15", "D-16"]:
        require(decision_id in decisions, f"HR PRD missing {decision_id}")
        lower = decisions[decision_id].lower()
        require("audit" in lower, f"{decision_id} must preserve an audit-chain/audit-envelope boundary")
    for decision_id in ["D-11", "D-12", "D-13", "D-14", "D-15", "D-16"]:
        lower = decisions[decision_id].lower()
        if decision_id in {"D-11", "D-12", "D-13", "D-14", "D-15"}:
            require("audit-chain emission" in lower or "audit-chain events" in lower or "runtime audit emission" in lower, f"{decision_id} must preserve no-runtime-audit-chain-emission wording")
        if decision_id == "D-16":
            require("runtime audit-chain emission" in lower, "D-16 must preserve no runtime audit-chain emission claim")
            require("statutory" in lower and "filing" in lower, "D-16 must preserve statutory filing boundary context")


def validate_hr_runtime_remains_metadata_only() -> None:
    texts = {
        HR_APP_PATH: read_text(HR_APP_PATH),
        HR_INFRA_PATH: read_text(HR_INFRA_PATH),
        HR_STORAGE_PATH: read_text(HR_STORAGE_PATH),
    }
    app_text = normalized_boundary_text(texts[HR_APP_PATH])
    require("metadata-only" in app_text, "app layer must remain metadata-only")
    require("does not persist data" in app_text, "app layer must preserve no-persistence claim")
    require("emit audit-chain records" in app_text, "app layer must explicitly preserve no audit-chain record emission claim")
    for topic in [
        "audit.hr.employment.lifecycle",
        "workflow.hr.compliance.dispatch",
        "integration.hr.payroll.leave-impact",
        "audit.hr.sensitive-read.policy",
    ]:
        require(topic in app_text, f"app layer missing metadata envelope topic {topic}")

    infra_text = normalized_boundary_text(texts[HR_INFRA_PATH])
    for phrase in [
        "does not persist hr records",
        "retrieve sensitive data",
        "execute workflow",
        "call payroll",
        "emit runtime audit-chain events",
        "deployed_listener: false",
        "storage_attached: false",
        "workflow_execution: false",
        "payroll_network_call: false",
        "sensitive_data_fetch: false",
    ]:
        require(phrase in infra_text, f"runtime adapter must preserve boundary phrase {phrase!r}")

    storage_text = normalized_boundary_text(texts[HR_STORAGE_PATH])
    for phrase in [
        "not for production",
        "does not persist to durable storage",
        "retrieve sensitive data",
        "execute workflow",
        "call payroll",
        "emit audit-chain events",
        "audit_chain_emission_attached: false",
    ]:
        require(phrase in storage_text, f"in-memory storage adapter must preserve boundary phrase {phrase!r}")

    for path, text in texts.items():
        for pattern in RUNTIME_PRODUCER_FORBIDDEN:
            require(not pattern.search(text), f"{rel(path)} appears to contain forbidden runtime producer pattern {pattern.pattern!r}")


def validate_registry_contract(registry: dict[str, Any]) -> None:
    event_enum = registry_event_enum(registry)
    registered_rows = registered_class_rows(registry)
    example_rows = registry_examples_by_name(registry)

    required_names = set(REQUIRED_HR_CLASSES)
    missing_enum = sorted(required_names - event_enum)
    require(not missing_enum, f"definitions.eventClassName.enum missing HR runtime classes {missing_enum}")

    missing_registered = sorted(required_names - set(registered_rows))
    require(not missing_registered, f"x-registered_classes missing HR runtime classes {missing_registered}")

    missing_examples = sorted(required_names - set(example_rows))
    require(not missing_examples, f"examples[].classes missing HR runtime class contracts {missing_examples}")

    hr_example_names = {
        str(bundle["example"].get("example_name", ""))
        for name, bundle in example_rows.items()
        if name in REQUIRED_HR_CLASSES
    }
    require(hr_example_names == {"HR runtime audit event classes"}, f"HR classes must live in one named example; got {sorted(hr_example_names)}")

    for event_class, requirements in REQUIRED_HR_CLASSES.items():
        registered = registered_rows[event_class]
        require(registered.get("originating_adr") == "ADR-0003", f"{event_class} must originate from ADR-0003 audit-chain authority")
        require(registered.get("category") == "hr-runtime-audit", f"{event_class} category must be hr-runtime-audit")
        require("HR" in str(registered.get("summary", "")), f"{event_class} summary must name HR")

        bundle = example_rows[event_class]
        example = bundle["example"]
        class_row = bundle["class_row"]
        require(set(example.get("mandatory_envelope_fields", [])) >= REQUIRED_ENVELOPE_FIELDS, f"{event_class} example missing mandatory v2 envelope fields")
        require(class_row.get("originating_adr") == "ADR-0003", f"{event_class} class row must originate from ADR-0003")
        require(class_row.get("retention_class") == requirements["retention_class"], f"{event_class} retention_class mismatch")
        require("audit-chain" in class_row.get("emission_targets", []), f"{event_class} must target audit-chain")
        require("otel-trace" in class_row.get("emission_targets", []), f"{event_class} must target otel-trace")
        require("compliance-evidence-bundle" in class_row.get("emission_targets", []), f"{event_class} must target compliance evidence bundle")
        require(class_row.get("cedar_decision_event") is requirements["cedar_decision_event"], f"{event_class} cedar_decision_event mismatch")
        require(class_row.get("schema_version") == "2.0.0", f"{event_class} must use schema_version 2.0.0")
        description = str(class_row.get("description", "")).lower()
        require("future" in description or "later" in description, f"{event_class} description must preserve future-producer/non-runtime boundary")
        field_names = {str(field.get("name")) for field in class_row.get("payload_fields", []) if isinstance(field, dict)}
        missing_fields = sorted(set(requirements["payload_fields"]) - field_names)
        require(not missing_fields, f"{event_class} payload_fields missing {missing_fields}")
        budget = class_row.get("cardinality_budget", {})
        require(isinstance(budget, dict), f"{event_class} cardinality_budget must be an object")
        dimension_keys = set(str(key) for key in budget.get("dimension_keys", []))
        missing_dimensions = sorted(set(requirements["dimension_keys"]) - dimension_keys)
        require(not missing_dimensions, f"{event_class} cardinality budget missing dimensions {missing_dimensions}")
        require(budget.get("max_distinct_per_cell_per_day", 0) <= 1000000, f"{event_class} cardinality budget must stay bounded")
        require(budget.get("high_cardinality_allowed") is False, f"{event_class} must not allow high-cardinality dimensions")

    update_policy = next(iter(example_rows[name]["example"] for name in REQUIRED_HR_CLASSES)).get("update_policy", {})
    require(update_policy.get("requires_validator_lane") == "cloud-ci/Rust gate packet: audit-event-class-registered", "HR registry update_policy must keep cloud-ci validator lane")


def main() -> None:
    validate_hr_prd(load_json(HR_PRD_PATH))
    validate_hr_runtime_remains_metadata_only()
    validate_registry_contract(load_json(REGISTRY_PATH))
    print("HR runtime audit event registry contract check passed: HR event classes and metadata-only runtime boundary verified")


if __name__ == "__main__":
    main()
