#!/usr/bin/env python3
"""Validate the payroll close audit-chain/WORM event-class registry contract.

This is a Plan/Spec/RED guard for t_06dc93ce. It proves that the current
payroll runtime remains metadata-only while requiring the audit-chain-owned
registry to name the minimal event classes Payroll needs before future producer
cards emit runtime audit rows or WORM/object-store digest references.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
REGISTRY_PATH = REPO_ROOT / "specs" / "audit-event-class-registry.json"
PAYROLL_PRD_PATH = REPO_ROOT / "specs" / "microservices" / "payroll.json"
PAYROLL_APP_PATH = REPO_ROOT / "oya" / "payroll" / "crates" / "oya-payroll-run-app" / "src" / "lib.rs"
PAYROLL_INFRA_PATH = REPO_ROOT / "oya" / "payroll" / "crates" / "oya-payroll-run-infrastructure" / "src" / "lib.rs"
PAYROLL_STORAGE_PATH = REPO_ROOT / "oya" / "payroll" / "crates" / "oya-payroll-run-storage-adapter-inmemory" / "src" / "lib.rs"

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

REQUIRED_PAYROLL_CLASSES: dict[str, set[str]] = {
    "PayrollTrialCloseEvidenceCommitted": {
        "legal_entity_id",
        "payroll_run_id",
        "payroll_period",
        "evidence_digest",
        "evidence_ref",
        "worm_storage_ref",
        "object_store_digest_ref",
        "rulepack_effective_date",
        "payee_count",
    },
    "PayrollStatutoryExportHashed": {
        "legal_entity_id",
        "payroll_run_id",
        "payroll_period",
        "export_kind",
        "export_hash",
        "receipt_ref",
        "rejection_reason",
        "rollback_repair_plan_ref",
        "object_store_digest_ref",
    },
    "PayrollRollbackQuarantineEvidenceEmitted": {
        "legal_entity_id",
        "payroll_run_id",
        "payroll_period",
        "failed_gate",
        "rollback_evidence_ref",
        "quarantine_ref",
        "repair_route_ref",
        "object_store_digest_ref",
    },
    "PayrollGlDispatchPrepared": {
        "legal_entity_id",
        "payroll_run_id",
        "payroll_period",
        "journal_batch_ref",
        "source_payroll_digest",
        "approval_evidence_ref",
        "accounting_dispatch_evidence_ref",
        "object_store_digest_ref",
    },
    "PayrollHrIntakeAccepted": {
        "legal_entity_id",
        "payroll_run_id",
        "payroll_period",
        "hr_source_topic",
        "source_hr_idempotency_key",
        "payroll_impact_evidence_ref",
        "payroll_intake_evidence_ref",
        "object_store_digest_ref",
    },
    "PayrollDurableStorageCommitRecorded": {
        "legal_entity_id",
        "payroll_run_id",
        "payroll_period",
        "storage_commit_ref",
        "reservation_ref",
        "idempotency_key",
        "evidence_ref_count",
        "worm_storage_ref",
        "object_store_digest_ref",
    },
}

RUNTIME_PRODUCER_FORBIDDEN = [
    re.compile(r"evidence/audit-chain\.jsonl"),
    re.compile(r"append_.*audit", re.IGNORECASE),
    re.compile(r"worm.*put", re.IGNORECASE),
    re.compile(r"object_store.*put", re.IGNORECASE),
]


def fail(message: str) -> NoReturn:
    print(f"payroll audit event registry contract check failed: {message}", file=sys.stderr)
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


def validate_payroll_prd(prd: dict[str, Any]) -> None:
    ac_by_id = {str(item.get("id")): item for item in prd.get("acceptance_criteria", []) if isinstance(item, dict)}
    for ac_id in ["AC-01", "AC-03", "AC-05", "AC-06", "AC-12"]:
        require(ac_id in ac_by_id, f"payroll PRD missing {ac_id}")
    persistence = prd.get("data_model", {}).get("persistence", [])
    evidence_rows = [row for row in persistence if isinstance(row, dict) and row.get("aggregate") == "PayrollEvidence"]
    require(len(evidence_rows) == 1, "payroll PRD must define exactly one PayrollEvidence persistence row")
    evidence_row = evidence_rows[0]
    require(evidence_row.get("store") == "audit-chain/WORM + object store digests", "PayrollEvidence store must remain audit-chain/WORM + object store digests")
    require(evidence_row.get("retention") == "jurisdiction bounded", "PayrollEvidence retention must remain jurisdiction bounded")


def validate_registry_contract(registry: dict[str, Any]) -> None:
    event_enum = registry_event_enum(registry)
    registered_rows = registered_class_rows(registry)
    example_rows = registry_examples_by_name(registry)

    missing_enum = sorted(set(REQUIRED_PAYROLL_CLASSES) - event_enum)
    require(not missing_enum, f"definitions.eventClassName.enum missing payroll classes {missing_enum}")

    missing_registered = sorted(set(REQUIRED_PAYROLL_CLASSES) - set(registered_rows))
    require(not missing_registered, f"x-registered_classes missing payroll classes {missing_registered}")

    missing_examples = sorted(set(REQUIRED_PAYROLL_CLASSES) - set(example_rows))
    require(not missing_examples, f"examples[].classes missing payroll class contracts {missing_examples}")

    payroll_example_names = {
        str(bundle["example"].get("example_name", "")) for bundle in example_rows.values() if bundle["class_row"].get("event_class") in REQUIRED_PAYROLL_CLASSES
    }
    require(payroll_example_names == {"payroll close evidence WORM classes"}, f"payroll classes must live in one named example; got {sorted(payroll_example_names)}")

    for event_class, required_fields in REQUIRED_PAYROLL_CLASSES.items():
        registered = registered_rows[event_class]
        require(registered.get("originating_adr") == "ADR-0003", f"{event_class} must originate from ADR-0003 audit-chain/WORM authority")
        require(registered.get("category") == "payroll-close-evidence", f"{event_class} category must be payroll-close-evidence")
        require("Payroll" in str(registered.get("summary", "")), f"{event_class} summary must name Payroll")

        bundle = example_rows[event_class]
        example = bundle["example"]
        class_row = bundle["class_row"]
        require(set(example.get("mandatory_envelope_fields", [])) >= REQUIRED_ENVELOPE_FIELDS, f"{event_class} example missing mandatory v2 envelope fields")
        require(class_row.get("originating_adr") == "ADR-0003", f"{event_class} class row must originate from ADR-0003")
        require(class_row.get("retention_class") == "financial_control", f"{event_class} must use financial_control retention")
        require("audit-chain" in class_row.get("emission_targets", []), f"{event_class} must target audit-chain")
        require("compliance-evidence-bundle" in class_row.get("emission_targets", []), f"{event_class} must target compliance evidence bundle")
        require(class_row.get("cedar_decision_event") is False, f"{event_class} is not a Cedar decision event")
        require(class_row.get("schema_version") == "2.0.0", f"{event_class} must use schema_version 2.0.0")
        field_names = {str(field.get("name")) for field in class_row.get("payload_fields", []) if isinstance(field, dict)}
        missing_fields = sorted(required_fields - field_names)
        require(not missing_fields, f"{event_class} payload_fields missing {missing_fields}")
        budget = class_row.get("cardinality_budget", {})
        require(isinstance(budget, dict), f"{event_class} cardinality_budget must be an object")
        dimension_keys = set(str(key) for key in budget.get("dimension_keys", []))
        for key in ["tenant_id", "source_microservice", "legal_entity_id", "payroll_run_id"]:
            require(key in dimension_keys, f"{event_class} cardinality budget missing dimension {key}")
        require(budget.get("max_distinct_per_cell_per_day", 0) <= 1000000, f"{event_class} cardinality budget must stay bounded")
        require(budget.get("high_cardinality_allowed") is False, f"{event_class} must not allow high cardinality")

    update_policy = next(iter(example_rows.values()))["example"].get("update_policy", {})
    require(update_policy.get("requires_validator_lane") == "cloud-ci/Rust gate packet: audit-event-class-registered", "payroll registry update_policy must keep cloud-ci validator lane")


def validate_payroll_runtime_remains_metadata_only() -> None:
    texts = {
        PAYROLL_APP_PATH: read_text(PAYROLL_APP_PATH),
        PAYROLL_INFRA_PATH: read_text(PAYROLL_INFRA_PATH),
        PAYROLL_STORAGE_PATH: read_text(PAYROLL_STORAGE_PATH),
    }
    app_text = texts[PAYROLL_APP_PATH].lower()
    require("metadata-only audit" in app_text, "app layer must remain metadata-only audit envelope producer")
    require("does not persist data" in app_text, "app layer must preserve no-persistence claim")
    infra_text = texts[PAYROLL_INFRA_PATH].lower()
    require("do not" in infra_text and "emit runtime audit-chain events" in infra_text, "runtime adapter must preserve no audit-chain emission claim")
    storage_text = texts[PAYROLL_STORAGE_PATH].lower()
    require("audit_chain_emission_attached: false" in storage_text, "storage capabilities must preserve audit_chain_emission_attached false")
    require("does not persist to durable storage" in storage_text, "in-memory storage must preserve no durable backend claim")
    for path, text in texts.items():
        for pattern in RUNTIME_PRODUCER_FORBIDDEN:
            require(not pattern.search(text), f"{rel(path)} appears to contain forbidden runtime producer pattern {pattern.pattern!r}")


def main() -> None:
    validate_payroll_prd(load_json(PAYROLL_PRD_PATH))
    validate_payroll_runtime_remains_metadata_only()
    validate_registry_contract(load_json(REGISTRY_PATH))
    print("payroll audit event registry contract check passed: payroll registry classes and metadata-only runtime boundary verified")


if __name__ == "__main__":
    main()
