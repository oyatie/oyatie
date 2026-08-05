#!/usr/bin/env python3
"""Fail-closed validator for future QK-03 privacy/data-governance dogfood receipts."""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
KIT_ID = "QK-03-privacy-data-governance"
EVIDENCE_SLUG = "qk-03-privacy-data-governance"
EXPECTED_DOGFOOD_ENVIRONMENT = "oyatie-dogfood-cell"
EXPECTED_EVIDENCE_ROOT = REPO_ROOT / "evidence" / "cloud" / "quality-kits" / EVIDENCE_SLUG / "runs"
SCRIPT_RELATIVE_PATH = "scripts/tests/qk_03_privacy_data_governance_future_harness_check.py"

SCENARIOS = {
    "QK-03-privacy-data-governance-S01": "personal-data inventory",
    "QK-03-privacy-data-governance-S02": "residency enforcement",
    "QK-03-privacy-data-governance-S03": "retention expiry",
    "QK-03-privacy-data-governance-S04": "deletion/erasure",
    "QK-03-privacy-data-governance-S05": "export/portability",
    "QK-03-privacy-data-governance-S06": "redaction in logs/telemetry",
}
REQUIRED_FIELDS = {
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
DIGEST_FIELDS = {"source_commit", "command", "dogfood_environment", "artifact_digest"}
REQUIRED_OUTPUTS = {
    "data_flow_map",
    "dsr_delete_export_proof",
    "residency_enforcement_test",
    "telemetry_redaction_check",
}
RESULT_SUMMARY_REQUIRED_KEYS = {
    "output_key",
    "expected_value_or_threshold",
    "observed_value",
    "artifact_ref",
    "evaluation_status",
}
PROVENANCE_FIELD = "privacy_data_governance_provenance"
REQUIRED_PROVENANCE_KEYS = {
    "data_flow_inventory_ref",
    "dsr_delete_receipt_ref",
    "dsr_export_receipt_ref",
    "residency_policy_ref",
    "residency_enforcement_receipt_ref",
    "retention_policy_ref",
    "retention_expiry_receipt_ref",
    "telemetry_redaction_receipt_ref",
}
RECEIPT_PROVENANCE_KEYS = {
    "data_flow_inventory_ref",
    "dsr_delete_receipt_ref",
    "dsr_export_receipt_ref",
    "residency_enforcement_receipt_ref",
    "retention_expiry_receipt_ref",
    "telemetry_redaction_receipt_ref",
}
ALLOWED_STATUSES = {"pending", "failed", "blocked", "passed_after_future_runtime_evidence"}
PLACEHOLDER_VALUES = {"", "todo", "tbd", "pending", "placeholder", "none", "null", "<run_id>"}
FORBIDDEN_FALLBACK_MARKERS = {
    "external_saas_runner",
    "github_actions_runner",
    "public_cloud_provider_runtime",
    "external saas",
    "github actions",
    "public cloud provider",
}
STATIC_SOURCE_PREFIXES = ("specs/", "docs/", "libs/", "registry/catalog/", "templates/")
EVIDENCE_REF_PREFIX = f"evidence/cloud/quality-kits/{EVIDENCE_SLUG}/"
SHA256_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")
SOURCE_COMMIT_PATTERN = re.compile(r"^[0-9a-f]{7,64}$")
RFC3339ISH_PATTERN = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")


def fail(message: str) -> NoReturn:
    print(f"QK-03 privacy/data-governance future harness check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def rel(path: Path) -> str:
    return str(path.resolve().relative_to(REPO_ROOT))


def text(value: object) -> str:
    if isinstance(value, dict):
        return " ".join(text(item) for item in value.values())
    if isinstance(value, (list, tuple, set)):
        return " ".join(text(item) for item in value)
    return str(value).lower()


def canonical_emit_path(value: str) -> Path:
    raw = Path(value)
    path = raw if raw.is_absolute() else REPO_ROOT / raw
    path = path.resolve()
    require(path.suffix == ".json", "--emit-evidence must point at a JSON run receipt")
    require(path.is_relative_to(EXPECTED_EVIDENCE_ROOT), f"--emit-evidence must stay under {rel(EXPECTED_EVIDENCE_ROOT)}")
    require(path.name not in {"<run_id>.json", "run_id.json", "latest.json"}, "--emit-evidence must use a concrete run id, not a placeholder")
    require("<" not in path.name and ">" not in path.name, "--emit-evidence must not contain placeholder angle brackets")
    return path


def expected_command(evidence_path: Path, dogfood_environment: str) -> str:
    return (
        f"python3 {SCRIPT_RELATIVE_PATH} "
        f"--dogfood-environment {dogfood_environment} "
        f"--emit-evidence {rel(evidence_path)}"
    )


def load_json(path: Path) -> dict:
    try:
        candidate = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing dogfood run receipt: {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON run receipt {rel(path)}: {exc}")
    require(isinstance(candidate, dict), "dogfood run receipt must be a JSON object")
    return candidate


def is_placeholder(value: object) -> bool:
    return str(value).strip().lower() in PLACEHOLDER_VALUES


def require_non_placeholder(record: dict, field: str) -> None:
    require(field in record, f"missing required field {field}")
    require(not is_placeholder(record[field]), f"{field} must be concrete")


def require_evidence_ref(value: object, label: str) -> str:
    ref = str(value)
    lower = ref.lower()
    require(not is_placeholder(ref), f"{label} must be concrete")
    require(not lower.startswith(STATIC_SOURCE_PREFIXES), f"{label} must be run evidence, not static source text: {ref}")
    require(lower.startswith(EVIDENCE_REF_PREFIX), f"{label} must stay under {EVIDENCE_REF_PREFIX}: {ref}")
    return ref


def validate_result_summary(record: dict, status: str) -> None:
    summary = record.get("result_summary")
    require(isinstance(summary, list) and summary, "result_summary must be a non-empty list")
    by_output: dict[str, dict] = {}
    for entry in summary:
        require(isinstance(entry, dict), "each result_summary entry must be an object")
        require(RESULT_SUMMARY_REQUIRED_KEYS <= set(entry), f"result_summary entry missing keys {sorted(RESULT_SUMMARY_REQUIRED_KEYS - set(entry))}")
        output_key = str(entry.get("output_key"))
        require(output_key in REQUIRED_OUTPUTS, f"unexpected result_summary output_key {output_key!r}")
        require(output_key not in by_output, f"duplicate result_summary output_key {output_key}")
        require_evidence_ref(entry.get("artifact_ref"), f"result_summary[{output_key}].artifact_ref")
        require(not is_placeholder(entry.get("observed_value")), f"result_summary[{output_key}].observed_value must be concrete")
        require(not is_placeholder(entry.get("expected_value_or_threshold")), f"result_summary[{output_key}].expected_value_or_threshold must be concrete")
        evaluation_status = str(entry.get("evaluation_status"))
        require(evaluation_status in {"pending", "failed", "blocked", "passed"}, f"result_summary[{output_key}].evaluation_status has unsupported value {evaluation_status!r}")
        by_output[output_key] = entry
    require(REQUIRED_OUTPUTS <= set(by_output), f"result_summary missing outputs {sorted(REQUIRED_OUTPUTS - set(by_output))}")
    if status == "passed_after_future_runtime_evidence":
        not_passed = [key for key, entry in by_output.items() if entry.get("evaluation_status") != "passed"]
        require(not not_passed, f"passed_after_future_runtime_evidence is fabricated unless every output passed; not passed: {not_passed}")


def validate_provenance(record: dict) -> None:
    provenance = record.get(PROVENANCE_FIELD)
    require(isinstance(provenance, dict), f"missing {PROVENANCE_FIELD} object")
    require(REQUIRED_PROVENANCE_KEYS <= set(provenance), f"{PROVENANCE_FIELD} missing keys {sorted(REQUIRED_PROVENANCE_KEYS - set(provenance))}")
    for key in REQUIRED_PROVENANCE_KEYS:
        require(not is_placeholder(provenance.get(key)), f"{PROVENANCE_FIELD}.{key} must be concrete")
    for key in RECEIPT_PROVENANCE_KEYS:
        require_evidence_ref(provenance[key], f"{PROVENANCE_FIELD}.{key}")


def validate_digest_fields(record: dict) -> None:
    for field in DIGEST_FIELDS:
        require_non_placeholder(record, field)
    require(SOURCE_COMMIT_PATTERN.match(str(record["source_commit"])), "source_commit must be a concrete git commit hex digest")
    require(SHA256_PATTERN.match(str(record["artifact_digest"])), "artifact_digest must be sha256:<64 lowercase hex chars>")


def validate_record(record: dict, evidence_path: Path, dogfood_environment: str) -> None:
    missing = REQUIRED_FIELDS - set(record)
    require(not missing, f"missing required fields {sorted(missing)}")
    require(record["kit_id"] == KIT_ID, f"kit_id must be {KIT_ID}")
    require(record["scenario_id"] in SCENARIOS, f"scenario_id must be one of {sorted(SCENARIOS)}")
    require(record["run_id"] == evidence_path.stem, "run_id must match the emitted receipt filename stem")
    require(dogfood_environment == EXPECTED_DOGFOOD_ENVIRONMENT, f"dogfood environment must be {EXPECTED_DOGFOOD_ENVIRONMENT}")
    require(record["dogfood_environment"] == EXPECTED_DOGFOOD_ENVIRONMENT, f"record dogfood_environment must be {EXPECTED_DOGFOOD_ENVIRONMENT}")
    command = str(record["command"])
    require(command == expected_command(evidence_path, dogfood_environment), "command field must exactly match the invoked dogfood harness command")
    lower_text = text(record)
    for marker in FORBIDDEN_FALLBACK_MARKERS:
        require(marker not in lower_text, f"forbidden fallback marker present: {marker}")
    status = str(record["status"])
    require(status in ALLOWED_STATUSES, f"status must be one of {sorted(ALLOWED_STATUSES)}")
    validate_digest_fields(record)
    require(not is_placeholder(record.get("reviewer")), "reviewer must be concrete; placeholder review cannot mark QK-03 evidence green")
    require(RFC3339ISH_PATTERN.match(str(record.get("created_at"))), "created_at must be UTC RFC3339-like timestamp ending in Z")
    evidence_window = record.get("evidence_window")
    require(isinstance(evidence_window, dict), "evidence_window must be an object")
    require({"started_at", "ended_at"} <= set(evidence_window), "evidence_window must include started_at and ended_at")
    validate_provenance(record)
    validate_result_summary(record, status)


def baseline_record(evidence_path: Path) -> dict:
    command = expected_command(evidence_path, EXPECTED_DOGFOOD_ENVIRONMENT)
    return {
        "kit_id": KIT_ID,
        "scenario_id": "QK-03-privacy-data-governance-S01",
        "run_id": evidence_path.stem,
        "dogfood_environment": EXPECTED_DOGFOOD_ENVIRONMENT,
        "command": command,
        "status": "blocked",
        "artifact_digest": "sha256:" + "a" * 64,
        "reviewer": "qk03-red-check-self-test",
        "created_at": "2026-07-01T00:00:00Z",
        "source_commit": "abcdef1234567890",
        "evidence_window": {"started_at": "2026-07-01T00:00:00Z", "ended_at": "2026-07-01T00:01:00Z"},
        PROVENANCE_FIELD: {
            "data_flow_inventory_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-data-flow-map.json",
            "dsr_delete_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-dsr-delete.json",
            "dsr_export_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-dsr-export.json",
            "residency_policy_ref": "libs/oya-data-boundary-kernel/fixtures/retention_qk03_dsr_evidence_contract.txt",
            "residency_enforcement_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-residency.json",
            "retention_policy_ref": "libs/oya-data-boundary-kernel/src/retention_policy.rs",
            "retention_expiry_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-retention-expiry.json",
            "telemetry_redaction_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-telemetry-redaction.json",
        },
        "result_summary": [
            {
                "output_key": output_key,
                "expected_value_or_threshold": "future dogfood receipt must provide measured proof",
                "observed_value": "blocked until future implementation emits dogfood evidence",
                "artifact_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-{output_key}.json",
                "evaluation_status": "blocked",
            }
            for output_key in sorted(REQUIRED_OUTPUTS)
        ],
    }


def run_self_tests() -> None:
    evidence_path = canonical_emit_path(f"{EVIDENCE_REF_PREFIX}runs/self-test-blocked.json")
    valid = baseline_record(evidence_path)
    validate_record(valid, evidence_path, EXPECTED_DOGFOOD_ENVIRONMENT)

    def expect_rejected(label: str, mutator: Callable[[dict], None]) -> None:
        candidate = json.loads(json.dumps(valid))
        mutator(candidate)
        try:
            validate_record(candidate, evidence_path, EXPECTED_DOGFOOD_ENVIRONMENT)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected("static source text artifact", lambda data: data["result_summary"][0].update({"artifact_ref": "specs/cloud-production-quality-kits-target.json"}))
    expect_rejected("missing DSR provenance", lambda data: data[PROVENANCE_FIELD].pop("dsr_delete_receipt_ref"))
    expect_rejected("missing residency provenance", lambda data: data[PROVENANCE_FIELD].pop("residency_enforcement_receipt_ref"))
    expect_rejected("missing retention provenance", lambda data: data[PROVENANCE_FIELD].pop("retention_expiry_receipt_ref"))
    expect_rejected("missing digest field", lambda data: data.update({"artifact_digest": ""}))
    expect_rejected("github actions fallback", lambda data: data.update({"dogfood_environment": "github_actions_runner"}))

    def fabricate_passed_status(data: dict) -> None:
        data["status"] = "passed_after_future_runtime_evidence"
        data["result_summary"][0]["evaluation_status"] = "blocked"

    expect_rejected("fabricated passed_after_future_runtime_evidence", fabricate_passed_status)
    print("QK-03 privacy/data-governance future harness self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dogfood-environment", help="must be oyatie-dogfood-cell")
    parser.add_argument("--emit-evidence", help="future dogfood run receipt path under evidence/cloud/quality-kits/qk-03-privacy-data-governance/runs/<run_id>.json")
    parser.add_argument("--self-test", action="store_true", help="run fail-closed validator self-tests")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.self_test:
        run_self_tests()
        return
    require(args.dogfood_environment, "--dogfood-environment is required")
    require(args.emit_evidence, "--emit-evidence is required")
    evidence_path = canonical_emit_path(args.emit_evidence)
    record = load_json(evidence_path)
    validate_record(record, evidence_path, args.dogfood_environment)
    print(f"QK-03 privacy/data-governance future harness receipt check passed: {rel(evidence_path)}")


if __name__ == "__main__":
    main()
