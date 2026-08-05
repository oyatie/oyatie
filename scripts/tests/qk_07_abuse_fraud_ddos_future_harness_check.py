#!/usr/bin/env python3
"""RED-only fail-closed check for future QK-07 abuse/fraud/DDoS dogfood evidence.

This is a Plan/Spec/RED guard, not the runtime abuse/fraud/DDoS harness.
It validates the future receipt shape enough to reject source-only or fabricated
claims, then remains red until a later Build card wires real dogfood runtime
verification and measured run receipts.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
KIT_ID = "QK-07-abuse-fraud-ddos"
EVIDENCE_SLUG = "qk-07-abuse-fraud-ddos"
DOGFOOD_ENVIRONMENT = "oyatie-dogfood-cell"
SCRIPT_RELATIVE_PATH = "scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py"
EXPECTED_EVIDENCE_ROOT = REPO_ROOT / "evidence" / "cloud" / "quality-kits" / EVIDENCE_SLUG / "runs"
EVIDENCE_REF_PREFIX = f"evidence/cloud/quality-kits/{EVIDENCE_SLUG}/"

SCENARIOS = {
    "QK-07-abuse-fraud-ddos-S01": "signup abuse",
    "QK-07-abuse-fraud-ddos-S02": "payment fraud",
    "QK-07-abuse-fraud-ddos-S03": "L3/4 + L7 DDoS",
    "QK-07-abuse-fraud-ddos-S04": "resource-exhaustion abuse",
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
    "abuse_drill_results",
    "ingress_threshold_report",
    "suspension_appeals_round_trip",
}
RESULT_SUMMARY_REQUIRED_KEYS = {
    "output_key",
    "expected_value_or_threshold",
    "observed_value",
    "artifact_ref",
    "evaluation_status",
}
PROVENANCE_FIELD = "abuse_fraud_ddos_provenance"
REQUIRED_PROVENANCE_KEYS = {
    "signup_abuse_drill_receipt_ref",
    "payment_fraud_drill_receipt_ref",
    "ddos_l3_l4_l7_drill_receipt_ref",
    "resource_exhaustion_drill_receipt_ref",
    "ingress_threshold_report_ref",
    "suspension_decision_receipt_ref",
    "appeals_round_trip_receipt_ref",
}
ALLOWED_STATUSES = {"pending", "failed", "blocked", "passed_after_future_runtime_evidence"}
PLACEHOLDER_PATTERN = re.compile(r"(^$|<[^>]+>|\btodo\b|\btbd\b|placeholder|fake|dummy|null|none)", re.IGNORECASE)
SHA256_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")
SOURCE_COMMIT_PATTERN = re.compile(r"^[0-9a-f]{7,64}$")
RFC3339ISH_PATTERN = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
STATIC_SOURCE_PREFIXES = (
    "specs/",
    "plan/",
    "docs/",
    "templates/",
    "registry/",
    "README.md",
    "AGENTS.md",
    "CLAUDE.md",
)
FORBIDDEN_FALLBACK_MARKERS = {
    "external_saas_runner",
    "github_actions_runner",
    "public_cloud_provider_runtime",
    "external saas",
    "github actions",
    "public cloud provider",
}


def fail(message: str) -> NoReturn:
    print(f"QK-07 abuse/fraud/DDoS future harness check failed: {message}", file=sys.stderr)
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


def is_placeholder(value: object) -> bool:
    return PLACEHOLDER_PATTERN.search(str(value).strip()) is not None


def require_non_placeholder(record: dict[str, Any], field: str) -> None:
    require(field in record, f"missing required field {field}")
    require(not is_placeholder(record[field]), f"{field} must be concrete")


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


def load_json(path: Path) -> dict[str, Any]:
    try:
        candidate = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing dogfood run receipt: {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON run receipt {rel(path)}: {exc}")
    require(isinstance(candidate, dict), "dogfood run receipt must be a JSON object")
    return candidate


def require_evidence_ref(value: object, label: str) -> str:
    ref = str(value).strip()
    lower = ref.lower().removeprefix("./")
    require(not is_placeholder(ref), f"{label} must be concrete")
    require(not lower.startswith(STATIC_SOURCE_PREFIXES), f"{label} must be run evidence, not static source text: {ref}")
    require(lower.startswith(EVIDENCE_REF_PREFIX), f"{label} must stay under {EVIDENCE_REF_PREFIX}: {ref}")
    return ref


def validate_digest_fields(record: dict[str, Any]) -> None:
    for field in DIGEST_FIELDS:
        require_non_placeholder(record, field)
    require(SOURCE_COMMIT_PATTERN.match(str(record["source_commit"])), "source_commit must be a concrete git commit hex digest")
    require(SHA256_PATTERN.match(str(record["artifact_digest"])), "artifact_digest must be sha256:<64 lowercase hex chars>")


def validate_provenance(record: dict[str, Any]) -> None:
    provenance_candidate = record.get(PROVENANCE_FIELD)
    if not isinstance(provenance_candidate, dict):
        fail(f"missing {PROVENANCE_FIELD} object")
    provenance: dict[str, Any] = provenance_candidate
    missing = sorted(REQUIRED_PROVENANCE_KEYS - set(provenance))
    require(not missing, f"{PROVENANCE_FIELD} missing keys {missing}")
    for key in sorted(REQUIRED_PROVENANCE_KEYS):
        require_evidence_ref(provenance[key], f"{PROVENANCE_FIELD}.{key}")


def validate_result_summary(record: dict[str, Any]) -> None:
    summary_candidate = record.get("result_summary")
    if not isinstance(summary_candidate, list) or not summary_candidate:
        fail("result_summary must be a non-empty list")
    summary: list[Any] = summary_candidate
    by_output: dict[str, dict[str, Any]] = {}
    for entry in summary:
        if not isinstance(entry, dict):
            fail("each result_summary entry must be an object")
        missing = sorted(RESULT_SUMMARY_REQUIRED_KEYS - set(entry))
        require(not missing, f"result_summary entry missing keys {missing}")
        output_key = str(entry.get("output_key"))
        require(output_key in REQUIRED_OUTPUTS, f"unexpected result_summary output_key {output_key!r}")
        require(output_key not in by_output, f"duplicate result_summary output_key {output_key}")
        require_evidence_ref(entry.get("artifact_ref"), f"result_summary[{output_key}].artifact_ref")
        require(not is_placeholder(entry.get("observed_value")), f"result_summary[{output_key}].observed_value must be concrete")
        require(
            not is_placeholder(entry.get("expected_value_or_threshold")),
            f"result_summary[{output_key}].expected_value_or_threshold must be concrete",
        )
        evaluation_status = str(entry.get("evaluation_status"))
        require(
            evaluation_status in {"pending", "failed", "blocked", "passed"},
            f"result_summary[{output_key}].evaluation_status has unsupported value {evaluation_status!r}",
        )
        by_output[output_key] = entry
    missing_outputs = sorted(REQUIRED_OUTPUTS - set(by_output))
    require(not missing_outputs, f"result_summary missing outputs {missing_outputs}")


def validate_scenario_results(record: dict[str, Any]) -> None:
    scenario_results_candidate = record.get("scenario_results")
    if not isinstance(scenario_results_candidate, dict):
        fail("scenario_results must map every QK-07 scenario id to dogfood results")
    scenario_results: dict[str, Any] = scenario_results_candidate
    missing = sorted(set(SCENARIOS) - set(scenario_results))
    require(not missing, f"scenario_results missing scenarios {missing}")
    for scenario_id, source_scenario in SCENARIOS.items():
        result = scenario_results.get(scenario_id)
        if not isinstance(result, dict):
            fail(f"scenario_results.{scenario_id} must be an object")
        require(result.get("source_scenario") == source_scenario, f"scenario_results.{scenario_id}.source_scenario must be {source_scenario!r}")
        require_evidence_ref(result.get("artifact_ref"), f"scenario_results.{scenario_id}.artifact_ref")
        require(not is_placeholder(result.get("observed_value")), f"scenario_results.{scenario_id}.observed_value must be concrete")


def validate_record(record: dict[str, Any], evidence_path: Path, dogfood_environment: str) -> None:
    missing = sorted(REQUIRED_FIELDS - set(record))
    require(not missing, f"missing required fields {missing}")
    require(record["kit_id"] == KIT_ID, f"kit_id must be {KIT_ID}")
    require(record["scenario_id"] in SCENARIOS, f"scenario_id must be one of {sorted(SCENARIOS)}")
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
        fail("fabricated passed_after_future_runtime_evidence status rejected by this RED-only check")

    validate_digest_fields(record)
    require(not is_placeholder(record.get("reviewer")), "reviewer must be concrete; placeholder review cannot mark QK-07 evidence green")
    require(RFC3339ISH_PATTERN.match(str(record.get("created_at"))), "created_at must be UTC RFC3339-like timestamp ending in Z")
    evidence_window_candidate = record.get("evidence_window")
    if not isinstance(evidence_window_candidate, dict):
        fail("evidence_window must be an object")
    evidence_window: dict[str, Any] = evidence_window_candidate
    require({"started_at", "ended_at"} <= set(evidence_window), "evidence_window must include started_at and ended_at")
    validate_provenance(record)
    validate_result_summary(record)
    validate_scenario_results(record)


def baseline_record(evidence_path: Path) -> dict[str, Any]:
    command = expected_command(evidence_path, DOGFOOD_ENVIRONMENT)
    return {
        "kit_id": KIT_ID,
        "scenario_id": "QK-07-abuse-fraud-ddos-S01",
        "run_id": evidence_path.stem,
        "dogfood_environment": DOGFOOD_ENVIRONMENT,
        "command": command,
        "status": "blocked",
        "artifact_digest": "sha256:" + "7" * 64,
        "reviewer": "qk07-red-check-self-test",
        "created_at": "2026-07-01T00:00:00Z",
        "source_commit": "abcdef1234567890",
        "evidence_window": {"started_at": "2026-07-01T00:00:00Z", "ended_at": "2026-07-01T00:05:00Z"},
        PROVENANCE_FIELD: {
            "signup_abuse_drill_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-signup-abuse.json",
            "payment_fraud_drill_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-payment-fraud.json",
            "ddos_l3_l4_l7_drill_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-ddos.json",
            "resource_exhaustion_drill_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-resource-exhaustion.json",
            "ingress_threshold_report_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-ingress-thresholds.json",
            "suspension_decision_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-suspension.json",
            "appeals_round_trip_receipt_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-appeals.json",
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
        "scenario_results": {
            scenario_id: {
                "source_scenario": source_scenario,
                "observed_value": "blocked until future implementation emits dogfood evidence",
                "artifact_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-{scenario_id.lower()}.json",
            }
            for scenario_id, source_scenario in SCENARIOS.items()
        },
    }


def run_self_tests() -> None:
    evidence_path = canonical_emit_path(f"{EVIDENCE_REF_PREFIX}runs/self-test-blocked.json")
    valid = baseline_record(evidence_path)
    validate_record(valid, evidence_path, DOGFOOD_ENVIRONMENT)

    def expect_rejected(label: str, mutator: Callable[[dict[str, Any]], None]) -> None:
        candidate = json.loads(json.dumps(valid))
        mutator(candidate)
        try:
            validate_record(candidate, evidence_path, DOGFOOD_ENVIRONMENT)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected("static source text result artifact", lambda data: data["result_summary"][0].update({"artifact_ref": "specs/cloud-production-quality-kits-target.json"}))
    expect_rejected("missing abuse drill provenance", lambda data: data[PROVENANCE_FIELD].pop("signup_abuse_drill_receipt_ref"))
    expect_rejected("missing ingress provenance", lambda data: data[PROVENANCE_FIELD].pop("ingress_threshold_report_ref"))
    expect_rejected("missing suspension provenance", lambda data: data[PROVENANCE_FIELD].pop("suspension_decision_receipt_ref"))
    expect_rejected("missing appeals provenance", lambda data: data[PROVENANCE_FIELD].pop("appeals_round_trip_receipt_ref"))
    expect_rejected("missing digest field", lambda data: data.update({"artifact_digest": ""}))
    expect_rejected("github actions fallback", lambda data: data.update({"dogfood_environment": "github_actions_runner"}))
    expect_rejected("missing scenario drill receipt", lambda data: data["scenario_results"].pop("QK-07-abuse-fraud-ddos-S03"))
    expect_rejected("fabricated passed_after_future_runtime_evidence", lambda data: data.update({"status": "passed_after_future_runtime_evidence"}))
    print("QK-07 abuse/fraud/DDoS future harness self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dogfood-environment", help="must be oyatie-dogfood-cell")
    parser.add_argument("--emit-evidence", help="future dogfood run receipt path under evidence/cloud/quality-kits/qk-07-abuse-fraud-ddos/runs/<run_id>.json")
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
    fail("future dogfood runtime verification is not implemented; QK-07 remains red until a real Build card emits measured dogfood receipts")


if __name__ == "__main__":
    main()
