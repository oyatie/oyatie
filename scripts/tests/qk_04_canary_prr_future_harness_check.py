#!/usr/bin/env python3
"""Fail-closed RED validator for future QK-04 canary/PRR + DR dogfood receipts.

This card specifies the future evidence producer contract only. The checker rejects
source-only, incomplete, or fabricated receipts and remains RED until a later Build
card wires real dogfood canary, PRR, rollback, and DR runtime verification.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
KIT_ID = "QK-04-canary-prr"
EVIDENCE_SLUG = "qk-04-canary-prr"
DOGFOOD_ENVIRONMENT = "oyatie-dogfood-cell"
SCRIPT_RELATIVE_PATH = "scripts/tests/qk_04_canary_prr_future_harness_check.py"
EVIDENCE_ROOT = REPO_ROOT / "evidence" / "cloud" / "quality-kits" / EVIDENCE_SLUG / "runs"
EVIDENCE_REF_PREFIX = f"evidence/cloud/quality-kits/{EVIDENCE_SLUG}/"

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
    "canary_eval_report",
    "prr_signoff",
    "rollback_drill_receipt",
    "backup_restore_drill_receipt",
    "rto_rpo_restore_drill_receipt",
    "cell_failover_drill_receipt",
    "dependency_failure_recovery_receipt",
}
REQUIRED_RESULT_SUMMARY_KEYS = {
    "output_key",
    "expected_value_or_threshold",
    "observed_value",
    "artifact_ref",
    "evaluation_status",
}
CANARY_PRR_OUTPUTS = {
    "canary_eval_report",
    "prr_signoff",
    "rollback_drill_receipt",
}
DR_OUTPUTS = {
    "backup_restore_drill_receipt",
    "rto_rpo_restore_drill_receipt",
    "cell_failover_drill_receipt",
    "dependency_failure_recovery_receipt",
}
SCENARIO_OUTPUTS = {
    "QK-04-canary-prr-S01": {"canary_eval_report", "prr_signoff"},
    "QK-04-canary-prr-S02": {"canary_eval_report", "prr_signoff"},
    "QK-04-canary-prr-S03": {"canary_eval_report", "rollback_drill_receipt"},
    "QK-04-canary-prr-S04": {"canary_eval_report", "rollback_drill_receipt"},
    "QK-04-canary-prr-DR01": {"backup_restore_drill_receipt", "rto_rpo_restore_drill_receipt", "rollback_drill_receipt"},
    "QK-04-canary-prr-DR02": {"cell_failover_drill_receipt", "rollback_drill_receipt"},
    "QK-04-canary-prr-DR03": {"rto_rpo_restore_drill_receipt", "backup_restore_drill_receipt"},
    "QK-04-canary-prr-DR04": {"dependency_failure_recovery_receipt", "rollback_drill_receipt"},
}
ALLOWED_STATUSES = {"pending", "failed", "blocked", "passed_after_future_runtime_evidence"}
FORBIDDEN_FALLBACK_MARKERS = {
    "external_saas_runner",
    "external saas",
    "github_actions_runner",
    "github actions",
    "public_cloud_provider_runtime",
    "public cloud provider",
}
STATIC_SOURCE_PREFIXES = (
    "specs/",
    "docs/",
    "plan/",
    "libs/",
    "registry/",
    "templates/",
    "README.md",
    "AGENTS.md",
    "CLAUDE.md",
)
PLACEHOLDER_PATTERN = re.compile(r"(<[^>]+>|\btodo\b|\btbd\b|placeholder|fake|dummy|null|none)", re.IGNORECASE)
SHA256_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")
SOURCE_COMMIT_PATTERN = re.compile(r"^[0-9a-f]{7,64}$")
RFC3339ISH_PATTERN = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")


def fail(message: str) -> NoReturn:
    print(f"QK-04 canary/PRR future harness check failed: {message}", file=sys.stderr)
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
    require(path.is_relative_to(EVIDENCE_ROOT), f"--emit-evidence must stay under {rel(EVIDENCE_ROOT)}")
    require(path.name not in {"<run_id>.json", "run_id.json", "latest.json"}, "--emit-evidence must use a concrete run id, not a placeholder")
    require("<" not in path.name and ">" not in path.name, "--emit-evidence must not contain placeholder angle brackets")
    return path


def expected_command(evidence_path: Path, dogfood_environment: str) -> str:
    return (
        f"python3 {SCRIPT_RELATIVE_PATH} "
        f"--dogfood-environment {dogfood_environment} "
        f"--emit-evidence {rel(evidence_path)}"
    )


def load_json(path: Path) -> dict[str, Any]:
    try:
        candidate = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing dogfood run receipt: {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON run receipt {rel(path)}: {exc}")
    require(isinstance(candidate, dict), "dogfood run receipt must be a JSON object")
    return candidate


def reject_placeholder(label: str, value: object) -> None:
    rendered = str(value).strip()
    require(rendered, f"{label} must be non-empty")
    require(not PLACEHOLDER_PATTERN.search(rendered), f"{label} contains a placeholder/fabricated value: {rendered!r}")


def require_evidence_ref(value: object, label: str) -> str:
    ref = str(value).strip()
    lower = ref.lower().removeprefix("./")
    reject_placeholder(label, ref)
    require(not lower.startswith(STATIC_SOURCE_PREFIXES), f"{label} points at static source text, not dogfood evidence: {ref}")
    require(lower.startswith(EVIDENCE_REF_PREFIX), f"{label} must stay under {EVIDENCE_REF_PREFIX}: {ref}")
    return ref


def result_rows(summary: object) -> list[dict[str, Any]]:
    if isinstance(summary, list):
        return [row for row in summary if isinstance(row, dict)]
    if isinstance(summary, dict):
        if isinstance(summary.get("outputs"), list):
            return [row for row in summary["outputs"] if isinstance(row, dict)]
        if REQUIRED_RESULT_SUMMARY_KEYS <= set(summary):
            return [summary]
        rows: list[dict[str, Any]] = []
        for value in summary.values():
            if isinstance(value, dict) and REQUIRED_RESULT_SUMMARY_KEYS <= set(value):
                rows.append(value)
        return rows
    return []


def validate_digest_fields(record: dict[str, Any]) -> None:
    for field in DIGEST_FIELDS:
        require(field in record, f"missing digest field: {field}")
        reject_placeholder(field, record[field])
    require(SOURCE_COMMIT_PATTERN.match(str(record["source_commit"])), "source_commit must be a concrete git commit hex digest")
    require(SHA256_PATTERN.match(str(record["artifact_digest"])), "artifact_digest must be sha256:<64 lowercase hex chars>")


def validate_result_summary(record: dict[str, Any]) -> None:
    rows = result_rows(record.get("result_summary"))
    require(rows, "result_summary must contain machine-checkable output rows")
    by_output: dict[str, dict[str, Any]] = {}
    for index, row in enumerate(rows):
        missing = REQUIRED_RESULT_SUMMARY_KEYS - set(row)
        require(not missing, f"result_summary row {index} missing keys: {sorted(missing)}")
        output_key = str(row.get("output_key"))
        require(output_key in REQUIRED_OUTPUTS, f"unexpected result_summary output_key {output_key!r}")
        require(output_key not in by_output, f"duplicate result_summary output_key {output_key}")
        require_evidence_ref(row.get("artifact_ref"), f"result_summary[{output_key}].artifact_ref")
        reject_placeholder(f"result_summary[{output_key}].observed_value", row.get("observed_value"))
        reject_placeholder(f"result_summary[{output_key}].expected_value_or_threshold", row.get("expected_value_or_threshold"))
        evaluation_status = str(row.get("evaluation_status"))
        require(evaluation_status in {"pending", "failed", "blocked", "passed"}, f"result_summary[{output_key}].evaluation_status has unsupported value {evaluation_status!r}")
        by_output[output_key] = row
    missing_outputs = REQUIRED_OUTPUTS - set(by_output)
    require(not missing_outputs, f"result_summary missing QK-04 output keys: {sorted(missing_outputs)}")
    require(CANARY_PRR_OUTPUTS <= set(by_output), "result_summary must include canary/PRR/rollback receipts")
    require(DR_OUTPUTS <= set(by_output), "result_summary must include backup/restore, RTO/RPO, cell-failover, and dependency-failure DR receipts")


def validate_scenario_results(record: dict[str, Any]) -> None:
    scenario_results = record.get("scenario_results")
    if not isinstance(scenario_results, dict):
        fail("scenario_results must map QK-04 S01..S04 and DR01..DR04 to dogfood results")
    missing_scenarios = set(SCENARIO_OUTPUTS) - set(scenario_results)
    require(not missing_scenarios, f"scenario_results missing scenarios: {sorted(missing_scenarios)}")
    for scenario_id, required_outputs in SCENARIO_OUTPUTS.items():
        entry = scenario_results.get(scenario_id)
        if not isinstance(entry, dict):
            fail(f"scenario_results[{scenario_id}] must be an object")
        outputs = set(entry.get("output_keys", []))
        if not outputs and isinstance(entry.get("result_summary"), list):
            outputs = {row.get("output_key") for row in entry["result_summary"] if isinstance(row, dict)}
        require(required_outputs <= outputs, f"scenario_results[{scenario_id}] missing outputs: {sorted(required_outputs - outputs)}")
        artifact_refs = entry.get("artifact_refs", [])
        require(isinstance(artifact_refs, list) and artifact_refs, f"scenario_results[{scenario_id}] must include dogfood artifact_refs")
        for index, artifact_ref in enumerate(artifact_refs):
            require_evidence_ref(artifact_ref, f"scenario_results[{scenario_id}].artifact_refs[{index}]")


def validate_record_contract(record: dict[str, Any], evidence_path: Path, dogfood_environment: str) -> None:
    missing = REQUIRED_FIELDS - set(record)
    require(not missing, f"missing required fields: {sorted(missing)}")
    require(record["kit_id"] == KIT_ID, f"kit_id must be {KIT_ID}")
    require(record["scenario_id"] in SCENARIO_OUTPUTS, f"scenario_id must be one of {sorted(SCENARIO_OUTPUTS)}")
    require(record["run_id"] == evidence_path.stem, "run_id must match the emitted receipt filename stem")
    require(dogfood_environment == DOGFOOD_ENVIRONMENT, f"dogfood environment must be {DOGFOOD_ENVIRONMENT}")
    require(record["dogfood_environment"] == DOGFOOD_ENVIRONMENT, f"record dogfood_environment must be {DOGFOOD_ENVIRONMENT}")
    require(record["command"] == expected_command(evidence_path, dogfood_environment), "command field must exactly match the invoked dogfood harness command")

    lower_text = text(record)
    for marker in FORBIDDEN_FALLBACK_MARKERS:
        require(marker not in lower_text, f"forbidden fallback marker present: {marker}")

    status = str(record["status"])
    require(status in ALLOWED_STATUSES, f"status must be one of {sorted(ALLOWED_STATUSES)}")
    if status == "passed_after_future_runtime_evidence":
        fail("fabricated passed_after_future_runtime_evidence status rejected; QK-04 still needs real dogfood runtime verification")

    validate_digest_fields(record)
    reject_placeholder("reviewer", record.get("reviewer"))
    require(RFC3339ISH_PATTERN.match(str(record.get("created_at"))), "created_at must be UTC RFC3339-like timestamp ending in Z")
    evidence_window = record.get("evidence_window")
    if not isinstance(evidence_window, dict):
        fail("evidence_window must be an object")
    require({"started_at", "ended_at"} <= set(evidence_window), "evidence_window must include started_at and ended_at")
    validate_result_summary(record)
    validate_scenario_results(record)


def baseline_record(evidence_path: Path) -> dict[str, Any]:
    command = expected_command(evidence_path, DOGFOOD_ENVIRONMENT)
    return {
        "kit_id": KIT_ID,
        "scenario_id": "QK-04-canary-prr-S01",
        "run_id": evidence_path.stem,
        "dogfood_environment": DOGFOOD_ENVIRONMENT,
        "command": command,
        "status": "blocked",
        "artifact_digest": "sha256:" + "b" * 64,
        "reviewer": "qk04-red-check-self-test",
        "created_at": "2026-07-01T00:00:00Z",
        "source_commit": "abcdef1234567890",
        "evidence_window": {"started_at": "2026-07-01T00:00:00Z", "ended_at": "2026-07-01T00:05:00Z"},
        "result_summary": [
            {
                "output_key": output_key,
                "expected_value_or_threshold": "future dogfood receipt must provide measured canary/PRR/rollback/DR proof",
                "observed_value": "blocked until future implementation emits dogfood evidence",
                "artifact_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-{output_key}.json",
                "evaluation_status": "blocked",
            }
            for output_key in sorted(REQUIRED_OUTPUTS)
        ],
        "scenario_results": {
            scenario_id: {
                "output_keys": sorted(outputs),
                "artifact_refs": [f"{EVIDENCE_REF_PREFIX}artifacts/self-test-{scenario_id.lower()}.json"],
            }
            for scenario_id, outputs in SCENARIO_OUTPUTS.items()
        },
    }


def run_self_tests() -> None:
    evidence_path = canonical_emit_path(f"{EVIDENCE_REF_PREFIX}runs/self-test-blocked.json")
    valid = baseline_record(evidence_path)
    validate_record_contract(valid, evidence_path, DOGFOOD_ENVIRONMENT)

    def expect_rejected(label: str, mutator: Callable[[dict[str, Any]], None]) -> None:
        candidate = json.loads(json.dumps(valid))
        mutator(candidate)
        try:
            validate_record_contract(candidate, evidence_path, DOGFOOD_ENVIRONMENT)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected("static source text artifact", lambda data: data["result_summary"][0].update({"artifact_ref": "specs/cloud-production-quality-kits-target.json"}))
    expect_rejected("missing canary report", lambda data: data.update({"result_summary": [row for row in data["result_summary"] if row["output_key"] != "canary_eval_report"]}))
    expect_rejected("missing PRR signoff", lambda data: data.update({"result_summary": [row for row in data["result_summary"] if row["output_key"] != "prr_signoff"]}))
    expect_rejected("missing rollback receipt", lambda data: data.update({"result_summary": [row for row in data["result_summary"] if row["output_key"] != "rollback_drill_receipt"]}))
    expect_rejected("missing backup/restore receipt", lambda data: data.update({"result_summary": [row for row in data["result_summary"] if row["output_key"] != "backup_restore_drill_receipt"]}))
    expect_rejected("missing RTO/RPO receipt", lambda data: data.update({"result_summary": [row for row in data["result_summary"] if row["output_key"] != "rto_rpo_restore_drill_receipt"]}))
    expect_rejected("missing cell failover receipt", lambda data: data.update({"result_summary": [row for row in data["result_summary"] if row["output_key"] != "cell_failover_drill_receipt"]}))
    expect_rejected("missing dependency failure receipt", lambda data: data.update({"result_summary": [row for row in data["result_summary"] if row["output_key"] != "dependency_failure_recovery_receipt"]}))
    expect_rejected("missing digest field", lambda data: data.pop("artifact_digest"))
    expect_rejected("missing DR scenario", lambda data: data["scenario_results"].pop("QK-04-canary-prr-DR04"))
    expect_rejected("missing scenario artifact ref", lambda data: data["scenario_results"]["QK-04-canary-prr-DR02"].update({"artifact_refs": []}))
    expect_rejected("github actions fallback", lambda data: data.update({"dogfood_environment": "github_actions_runner"}))
    expect_rejected("fabricated passed_after_future_runtime_evidence", lambda data: data.update({"status": "passed_after_future_runtime_evidence"}))
    print("QK-04 canary/PRR future harness self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dogfood-environment", help="must be oyatie-dogfood-cell")
    parser.add_argument("--emit-evidence", help="future dogfood run receipt path under evidence/cloud/quality-kits/qk-04-canary-prr/runs/<run_id>.json")
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
    validate_record_contract(record, evidence_path, args.dogfood_environment)
    fail("future dogfood runtime verification is not implemented; QK-04 remains RED until a real run receipt is produced")


if __name__ == "__main__":
    main()
