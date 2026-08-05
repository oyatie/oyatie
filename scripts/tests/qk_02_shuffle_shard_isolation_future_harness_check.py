#!/usr/bin/env python3
"""RED-only fail-closed check for future QK-02 shuffle-shard dogfood evidence.

This script intentionally does not implement the runtime shuffle-shard simulator.
It validates the future receipt shape enough to reject source-only or fabricated
claims, then remains red until a later Build card wires real dogfood runtime
verification.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
KIT_ID = "QK-02-shuffle-shard-isolation"
EVIDENCE_SLUG = "qk-02-shuffle-shard-isolation"
DOGFOOD_ENVIRONMENT = "oyatie-dogfood-cell"
SCRIPT_PATH = "scripts/tests/qk_02_shuffle_shard_isolation_future_harness_check.py"
EVIDENCE_ROOT = Path(f"evidence/cloud/quality-kits/{EVIDENCE_SLUG}/runs")
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
DIGEST_FIELDS = {
    "source_commit",
    "command",
    "dogfood_environment",
    "artifact_digest",
}
REQUIRED_OUTPUTS = {
    "correlated_impact_probability_matrix",
    "blast_radius_bound",
    "noisy_neighbor_isolation_drill",
}
REQUIRED_SCENARIOS = {
    "QK-02-shuffle-shard-isolation-S01": "single-tenant fault",
    "QK-02-shuffle-shard-isolation-S02": "noisy-neighbor resource hog",
    "QK-02-shuffle-shard-isolation-S03": "poison-pill request",
}
RESULT_SUMMARY_KEYS = {
    "output_key",
    "expected_value_or_threshold",
    "observed_value",
    "artifact_ref",
    "evaluation_status",
}
TENANT_ASSIGNMENT_KEYS = {
    "tenant_id",
    "shard_set",
    "source_artifact_ref",
    "source_digest",
    "assignment_policy_version",
}
STATIC_SOURCE_PREFIXES = (
    "specs/",
    "plan/",
    "docs/",
    "README.md",
    "AGENTS.md",
    "CLAUDE.md",
)
PLACEHOLDER_RE = re.compile(r"(<[^>]+>|todo|tbd|placeholder|fake|dummy)", re.IGNORECASE)
SHA256_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
SOURCE_COMMIT_RE = re.compile(r"^[0-9a-f]{7,64}$")
RFC3339ISH_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
ALLOWED_STATUSES = {"pending", "failed", "blocked", "passed_after_future_runtime_evidence"}
FORBIDDEN_FALLBACK_MARKERS = {
    "external_saas_runner",
    "github_actions_runner",
    "public_cloud_provider_runtime",
    "external saas",
    "github actions",
    "public cloud provider",
}


def fail(message: str) -> NoReturn:
    print(f"qk-02 shuffle-shard isolation future harness check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def repo_relative(path_text: str) -> tuple[Path, Path]:
    raw = Path(path_text)
    absolute = raw if raw.is_absolute() else REPO_ROOT / raw
    absolute = absolute.resolve()
    try:
        relative = absolute.relative_to(REPO_ROOT)
    except ValueError:
        fail(f"evidence path must stay inside repo: {path_text}")
    return absolute, relative


def ensure_evidence_path(relative: Path) -> None:
    require(relative.suffix == ".json", f"evidence receipt must be JSON: {relative}")
    require(relative.name not in {"<run_id>.json", "run_id.json", "latest.json"}, "evidence receipt must use a concrete run id filename")
    require("<" not in relative.name and ">" not in relative.name, "evidence receipt filename must not contain placeholder angle brackets")
    root = str(EVIDENCE_ROOT)
    rel = str(relative)
    require(rel.startswith(f"{root}/"), f"evidence receipt must live under {EVIDENCE_ROOT}: {relative}")


def text(value: Any) -> str:
    if isinstance(value, dict):
        return " ".join(text(item) for item in value.values())
    if isinstance(value, (list, tuple, set)):
        return " ".join(text(item) for item in value)
    return str(value).lower()


def normalize_artifact_ref(value: Any) -> str:
    ref = str(value).strip()
    reject_placeholder("artifact reference", ref)
    path = Path(ref)
    if path.is_absolute():
        try:
            return str(path.resolve().relative_to(REPO_ROOT))
        except ValueError:
            fail(f"artifact reference must stay inside repo evidence, not external storage: {ref}")
    return ref.removeprefix("./")


def load_receipt(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing dogfood run receipt at {path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON receipt at {path.relative_to(REPO_ROOT)}: {exc}")


def reject_placeholder(label: str, value: Any) -> None:
    text = str(value).strip()
    require(text, f"{label} must be non-empty")
    require(not PLACEHOLDER_RE.search(text), f"{label} contains placeholder/fabricated value: {text!r}")


def reject_static_source_ref(label: str, value: Any) -> None:
    normalized = normalize_artifact_ref(value)
    for prefix in STATIC_SOURCE_PREFIXES:
        require(not normalized.startswith(prefix), f"{label} points at static source text, not dogfood evidence: {value}")
    require(normalized.startswith(EVIDENCE_REF_PREFIX), f"{label} must point at QK-02 dogfood evidence under {EVIDENCE_REF_PREFIX}: {value}")


def result_rows(result_summary: Any) -> list[dict[str, Any]]:
    if isinstance(result_summary, list):
        return [row for row in result_summary if isinstance(row, dict)]
    if isinstance(result_summary, dict):
        if "outputs" in result_summary and isinstance(result_summary["outputs"], list):
            return [row for row in result_summary["outputs"] if isinstance(row, dict)]
        if RESULT_SUMMARY_KEYS <= set(result_summary):
            return [result_summary]
        rows = []
        for value in result_summary.values():
            if isinstance(value, dict) and RESULT_SUMMARY_KEYS <= set(value):
                rows.append(value)
        return rows
    return []


def tenant_assignment_inputs(receipt: dict[str, Any]) -> list[dict[str, Any]]:
    direct = receipt.get("tenant_shard_assignment_inputs")
    if isinstance(direct, list):
        return [item for item in direct if isinstance(item, dict)]
    result_summary = receipt.get("result_summary")
    if isinstance(result_summary, dict):
        nested = result_summary.get("tenant_shard_assignment_inputs")
        if isinstance(nested, list):
            return [item for item in nested if isinstance(item, dict)]
    return []


def validate_receipt(receipt: dict[str, Any], evidence_relative: Path, *, terminal_red_fail: bool = True) -> None:
    missing = sorted(REQUIRED_FIELDS - set(receipt))
    require(not missing, f"receipt missing required fields: {', '.join(missing)}")
    require(receipt.get("kit_id") == KIT_ID, f"kit_id must be {KIT_ID}")
    require(receipt.get("scenario_id") in REQUIRED_SCENARIOS, f"scenario_id must be one of {sorted(REQUIRED_SCENARIOS)}")
    require(receipt.get("run_id") == evidence_relative.stem, "run_id must match the evidence receipt filename stem")
    require(receipt.get("dogfood_environment") == DOGFOOD_ENVIRONMENT, f"dogfood_environment must be {DOGFOOD_ENVIRONMENT}")
    status = str(receipt.get("status"))
    require(status in ALLOWED_STATUSES, f"status must be one of {sorted(ALLOWED_STATUSES)}")

    lower_text = text(receipt)
    for marker in FORBIDDEN_FALLBACK_MARKERS:
        require(marker not in lower_text, f"forbidden fallback marker present: {marker}")

    expected_command = (
        f"python3 {SCRIPT_PATH} --dogfood-environment {DOGFOOD_ENVIRONMENT} "
        f"--emit-evidence {evidence_relative}"
    )
    require(receipt.get("command") == expected_command, f"command must preserve the QK-02 harness invocation: {expected_command}")

    for field in sorted(DIGEST_FIELDS):
        require(field in receipt, f"missing digest field: {field}")
        reject_placeholder(field, receipt[field])
    require(SOURCE_COMMIT_RE.match(str(receipt.get("source_commit"))), "source_commit must be a concrete git commit hex digest")
    require(SHA256_RE.match(str(receipt.get("artifact_digest"))), "artifact_digest must be sha256:<64 lowercase hex chars>")
    reject_placeholder("reviewer", receipt.get("reviewer"))
    require(RFC3339ISH_RE.match(str(receipt.get("created_at"))), "created_at must be a UTC RFC3339-like timestamp ending in Z")
    evidence_window = receipt.get("evidence_window")
    if not isinstance(evidence_window, dict):
        fail("evidence_window must be an object")
    require({"started_at", "ended_at"} <= set(evidence_window), "evidence_window must include started_at and ended_at")

    assignments = tenant_assignment_inputs(receipt)
    require(assignments, "missing tenant/shard assignment inputs")
    for index, assignment in enumerate(assignments):
        missing_assignment = sorted(TENANT_ASSIGNMENT_KEYS - set(assignment))
        require(not missing_assignment, f"tenant_shard_assignment_inputs[{index}] missing keys: {', '.join(missing_assignment)}")
        reject_static_source_ref(f"tenant_shard_assignment_inputs[{index}].source_artifact_ref", assignment["source_artifact_ref"])
        reject_placeholder(f"tenant_shard_assignment_inputs[{index}].source_digest", assignment["source_digest"])
        require(SHA256_RE.match(str(assignment["source_digest"])), f"tenant_shard_assignment_inputs[{index}].source_digest must be sha256:<64 lowercase hex chars>")

    rows = result_rows(receipt.get("result_summary"))
    require(rows, "result_summary must contain machine-checkable output rows")
    output_keys = {row.get("output_key") for row in rows}
    missing_outputs = sorted(REQUIRED_OUTPUTS - output_keys)
    require(not missing_outputs, f"result_summary missing QK-02 output keys: {', '.join(missing_outputs)}")
    for index, row in enumerate(rows):
        missing_row = sorted(RESULT_SUMMARY_KEYS - set(row))
        require(not missing_row, f"result_summary row {index} missing keys: {', '.join(missing_row)}")
        reject_static_source_ref(f"result_summary[{index}].artifact_ref", row["artifact_ref"])

    scenario_results = receipt.get("scenario_results")
    if not isinstance(scenario_results, dict):
        fail("scenario_results must map every QK-02 scenario id to dogfood results")
    missing_scenarios = sorted(set(REQUIRED_SCENARIOS) - set(scenario_results))
    require(not missing_scenarios, f"scenario_results missing scenarios: {', '.join(missing_scenarios)}")
    for scenario_id in REQUIRED_SCENARIOS:
        require(isinstance(scenario_results.get(scenario_id), dict), f"scenario_results[{scenario_id}] must be an object")

    if status == "passed_after_future_runtime_evidence":
        fail(
            "fabricated passed_after_future_runtime_evidence status rejected; "
            "a future Build card must replace this RED check with real dogfood runtime verification"
        )
    if terminal_red_fail:
        fail(f"future dogfood runtime verification is not implemented; receipt status {receipt.get('status')!r} cannot satisfy QK-02 yet")


def baseline_receipt(evidence_relative: Path) -> dict[str, Any]:
    command = (
        f"python3 {SCRIPT_PATH} --dogfood-environment {DOGFOOD_ENVIRONMENT} "
        f"--emit-evidence {evidence_relative}"
    )
    return {
        "kit_id": KIT_ID,
        "scenario_id": "QK-02-shuffle-shard-isolation-S01",
        "run_id": evidence_relative.stem,
        "dogfood_environment": DOGFOOD_ENVIRONMENT,
        "command": command,
        "status": "blocked",
        "artifact_digest": "sha256:" + "a" * 64,
        "reviewer": "qk02-red-check-self-test",
        "created_at": "2026-07-01T00:00:00Z",
        "source_commit": "abcdef1234567890",
        "evidence_window": {"started_at": "2026-07-01T00:00:00Z", "ended_at": "2026-07-01T00:01:00Z"},
        "tenant_shard_assignment_inputs": [
            {
                "tenant_id": "tenant-self-test-a",
                "shard_set": ["cell-a/shard-01", "cell-b/shard-03"],
                "source_artifact_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-assignment-input.json",
                "source_digest": "sha256:" + "b" * 64,
                "assignment_policy_version": "self-test-policy-v1",
            }
        ],
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
                "artifact_ref": f"{EVIDENCE_REF_PREFIX}artifacts/self-test-{scenario_id}.json",
                "evaluation_status": "blocked",
            }
            for scenario_id, source_scenario in REQUIRED_SCENARIOS.items()
        },
    }


def run_self_tests() -> None:
    evidence_relative = EVIDENCE_ROOT / "self-test-blocked.json"
    ensure_evidence_path(evidence_relative)
    valid = baseline_receipt(evidence_relative)
    validate_receipt(valid, evidence_relative, terminal_red_fail=False)

    def expect_rejected(label: str, mutator: Callable[[dict[str, Any]], None]) -> None:
        candidate = json.loads(json.dumps(valid))
        mutator(candidate)
        try:
            validate_receipt(candidate, evidence_relative, terminal_red_fail=False)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected("absolute static source assignment ref", lambda data: data["tenant_shard_assignment_inputs"][0].update({"source_artifact_ref": str(REPO_ROOT / "specs" / "cloud-production-quality-kits-target.json")}))
    expect_rejected("relative static result artifact ref", lambda data: data["result_summary"][0].update({"artifact_ref": "plan/cloud-quality-kits/qk-02-shuffle-shard-isolation-plan-spec-red.md"}))
    expect_rejected("external SaaS fallback marker", lambda data: data.update({"dogfood_environment": "external_saas_runner"}))
    expect_rejected("missing tenant shard inputs", lambda data: data.pop("tenant_shard_assignment_inputs"))
    expect_rejected("missing QK-02 output", lambda data: data.update({"result_summary": data["result_summary"][1:]}))
    expect_rejected("fabricated positive status", lambda data: data.update({"status": "passed_after_future_runtime_evidence"}))
    expect_rejected("placeholder source digest", lambda data: data["tenant_shard_assignment_inputs"][0].update({"source_digest": "sha256:<digest>"}))
    expect_rejected("missing scenario result", lambda data: data["scenario_results"].pop("QK-02-shuffle-shard-isolation-S03"))
    print("qk-02 shuffle-shard isolation future harness self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dogfood-environment")
    parser.add_argument("--emit-evidence", help="Future QK-02 dogfood run receipt path")
    parser.add_argument("--self-test", action="store_true", help="run fail-closed validator self-tests")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.self_test:
        run_self_tests()
        return
    require(args.dogfood_environment, "--dogfood-environment is required")
    require(args.emit_evidence, "--emit-evidence is required")
    require(args.dogfood_environment == DOGFOOD_ENVIRONMENT, f"dogfood environment must be {DOGFOOD_ENVIRONMENT}")
    evidence_absolute, evidence_relative = repo_relative(args.emit_evidence)
    ensure_evidence_path(evidence_relative)
    receipt = load_receipt(evidence_absolute)
    validate_receipt(receipt, evidence_relative)


if __name__ == "__main__":
    main()
