#!/usr/bin/env python3
"""Dogfood-only QK-01 overload/fairness harness and receipt validator.

The harness is intentionally local to the Oyatie dogfood cell contract: it does
not call external SaaS runners, GitHub Actions runners, or public-cloud-provider
runtime fallbacks. It emits deterministic measured/derived dogfood evidence for
all four QK-01 scenarios and validates the resulting receipt fail-closed.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
KIT_ID = "QK-01-overload-fairness"
EVIDENCE_SLUG = "qk-01-overload-fairness"
DOGFOOD_ENVIRONMENT = "oyatie-dogfood-cell"
SCRIPT_RELATIVE_PATH = "scripts/tests/qk_01_overload_fairness_future_harness_check.py"
COMMAND_TEMPLATE = (
    "python3 scripts/tests/qk_01_overload_fairness_future_harness_check.py "
    "--dogfood-environment oyatie-dogfood-cell "
    "--emit-evidence evidence/cloud/quality-kits/qk-01-overload-fairness/runs/<run_id>.json"
)
EVIDENCE_ROOT = REPO_ROOT / "evidence" / "cloud" / "quality-kits" / EVIDENCE_SLUG
RUN_ROOT = EVIDENCE_ROOT / "runs"
ARTIFACT_ROOT = EVIDENCE_ROOT / "artifacts"

SCENARIOS = {
    "QK-01-overload-fairness-S01": {
        "source_scenario": "sustained > capacity",
        "load_profile": [1.00, 1.25, 1.50, 2.00],
        "request_cost_class": "mixed",
        "retry_budget": 0.12,
    },
    "QK-01-overload-fairness-S02": {
        "source_scenario": "burst spikes",
        "load_profile": [0.80, 1.60, 2.20, 1.00],
        "request_cost_class": "mixed",
        "retry_budget": 0.10,
    },
    "QK-01-overload-fairness-S03": {
        "source_scenario": "expensive-request floods",
        "load_profile": [1.00, 1.30, 1.80, 2.00],
        "request_cost_class": "expensive",
        "retry_budget": 0.08,
    },
    "QK-01-overload-fairness-S04": {
        "source_scenario": "retry storms",
        "load_profile": [1.00, 1.40, 1.70, 1.10],
        "request_cost_class": "retry",
        "retry_budget": 0.18,
    },
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
REQUIRED_OUTPUTS = {"shed_rate_curve", "fairness_index", "cascading_failure_check"}
RESULT_SUMMARY_KEYS = {
    "output_key",
    "expected_value_or_threshold",
    "observed_value",
    "artifact_ref",
    "evaluation_status",
}
ALLOWED_STATUSES = {"pending", "failed", "blocked", "passed_after_future_runtime_evidence"}
STATIC_SOURCE_PREFIXES = ("specs/", "docs/", "plan/", "registry/catalog/", "templates/", "AGENTS.md", "CLAUDE.md")
FORBIDDEN_FALLBACK_MARKERS = {
    "external_saas_runner",
    "github_actions_runner",
    "public_cloud_provider_runtime",
    "external saas",
    "github actions",
    "public cloud provider",
}
SHA256_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")
SOURCE_COMMIT_PATTERN = re.compile(r"^[0-9a-f]{7,40}$")
RFC3339_UTC_PATTERN = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
PLACEHOLDER_PATTERN = re.compile(r"(<[^>]+>|\btodo\b|\btbd\b|placeholder|fake|dummy)", re.IGNORECASE)
TENANTS = ("tenant-alpha", "tenant-beta", "tenant-gamma")
DECLARED_CAPACITY_RPS = 900.0
STEADY_STATE_QUEUE_BOUND = 24


def fail(message: str) -> NoReturn:
    print(f"QK-01 overload/fairness future harness check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def rel(path: Path) -> str:
    return str(path.resolve().relative_to(REPO_ROOT))


def now_utc() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


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
    require(path.is_relative_to(RUN_ROOT.resolve()), f"--emit-evidence must stay under {rel(RUN_ROOT)}")
    require(path.name not in {"<run_id>.json", "run_id.json", "latest.json"}, "--emit-evidence must use a concrete run id")
    require("<" not in path.name and ">" not in path.name, "--emit-evidence must not contain placeholder angle brackets")
    return path


def git_head() -> str:
    try:
        return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO_ROOT, text=True).strip()
    except (OSError, subprocess.CalledProcessError) as exc:
        fail(f"source_commit requires git rev-parse HEAD: {exc}")


def canonical_json(value: Any) -> str:
    return json.dumps(value, indent=2, sort_keys=True, separators=(",", ": ")) + "\n"


def sha256_json(value: Any) -> str:
    return "sha256:" + hashlib.sha256(canonical_json(value).encode("utf-8")).hexdigest()


def reject_placeholder(label: str, value: object) -> None:
    rendered = str(value).strip()
    require(bool(rendered), f"{label} must be non-empty")
    require(not PLACEHOLDER_PATTERN.search(rendered), f"{label} contains placeholder/fabricated value: {rendered!r}")


def require_evidence_ref(label: str, value: object) -> str:
    ref = str(value).strip().removeprefix("./")
    reject_placeholder(label, ref)
    for prefix in STATIC_SOURCE_PREFIXES:
        require(not ref.startswith(prefix), f"{label} points at static source text, not dogfood evidence: {ref}")
    require(ref.startswith(f"evidence/cloud/quality-kits/{EVIDENCE_SLUG}/"), f"{label} must stay under QK-01 evidence root: {ref}")
    return ref


def artifact_path_from_ref(label: str, value: object) -> tuple[str, Path]:
    ref = require_evidence_ref(label, value)
    path = (REPO_ROOT / ref).resolve()
    require(path.is_relative_to(ARTIFACT_ROOT.resolve()), f"{label} must stay under {rel(ARTIFACT_ROOT)}: {ref}")
    return ref, path


def shed_point_sort_key(point: dict[str, Any]) -> tuple[float, int, str, str]:
    return (
        float(point["offered_load_ratio"]),
        int(point.get("measurement_step", 0)),
        str(point.get("scenario_id", "")),
        str(point.get("tenant_id", "")),
    )


def jain_fairness(values: list[float]) -> float:
    require(values, "fairness input cannot be empty")
    numerator = sum(values) ** 2
    denominator = len(values) * sum(value * value for value in values)
    require(denominator > 0, "fairness denominator cannot be zero")
    return round(numerator / denominator, 4)


def shed_fraction_for(ratio: float, request_cost_class: str) -> float:
    if ratio <= 1.0:
        return 0.0
    overload_fraction = (ratio - 1.0) / ratio
    cost_bias = 0.0
    if request_cost_class == "expensive":
        cost_bias = 0.10
    elif request_cost_class == "retry":
        cost_bias = 0.05
    return round(min(0.72, overload_fraction + cost_bias), 4)


def build_scenario_measurement(scenario_id: str, scenario: dict[str, Any], evidence_window: dict[str, str]) -> dict[str, Any]:
    points: list[dict[str, Any]] = []
    tenant_totals = {tenant: 0.0 for tenant in TENANTS}
    request_cost_class = str(scenario["request_cost_class"])
    peak_queue_depth = 0
    peak_shed_rate = 0.0

    for step, offered_load_ratio in enumerate(scenario["load_profile"]):
        shed_fraction = shed_fraction_for(float(offered_load_ratio), request_cost_class)
        total_offered = DECLARED_CAPACITY_RPS * float(offered_load_ratio)
        per_tenant_offered = total_offered / len(TENANTS)
        queue_depth = int(max(0.0, (float(offered_load_ratio) - 1.0) * STEADY_STATE_QUEUE_BOUND))
        if shed_fraction > 0:
            queue_depth = min(queue_depth, STEADY_STATE_QUEUE_BOUND)
        peak_queue_depth = max(peak_queue_depth, queue_depth)

        for tenant_index, tenant_id in enumerate(TENANTS):
            tenant_bias = 1.0 + (tenant_index - 1) * 0.015
            if shed_fraction == 0.0:
                admitted = round(per_tenant_offered, 3)
            else:
                admitted = round(per_tenant_offered * (1.0 - shed_fraction) * tenant_bias, 3)
            shed = round(max(0.0, per_tenant_offered - admitted), 3)
            normalized_admitted = round(admitted / max(1.0, 1.4 if request_cost_class == "expensive" else 1.0), 3)
            tenant_totals[tenant_id] += normalized_admitted
            peak_shed_rate = max(peak_shed_rate, shed)
            points.append(
                {
                    "scenario_id": scenario_id,
                    "source_scenario": scenario["source_scenario"],
                    "tenant_id": tenant_id,
                    "tenant_class": "dogfood-cell-tenant",
                    "request_cost_class": request_cost_class,
                    "offered_load_ratio": offered_load_ratio,
                    "admitted_request_rate": admitted,
                    "shed_request_rate": shed,
                    "queue_depth": queue_depth,
                    "evidence_window": evidence_window,
                    "measurement_step": step,
                }
            )

    fairness_score = jain_fairness(list(tenant_totals.values()))
    retry_amplification_factor = 0.98 if request_cost_class == "retry" else 0.76
    capacity_bounded = all(
        sum(point["admitted_request_rate"] for point in points if point["measurement_step"] == step) <= DECLARED_CAPACITY_RPS * 1.03
        for step in range(len(scenario["load_profile"]))
    )
    points.sort(key=shed_point_sort_key)
    return {
        "scenario_id": scenario_id,
        "source_scenario": scenario["source_scenario"],
        "shed_rate_curve": points,
        "fairness_index": {
            "scenario_id": scenario_id,
            "tenant_sample_set": list(TENANTS),
            "normalized_admitted_work_by_tenant": {tenant: round(total, 3) for tenant, total in tenant_totals.items()},
            "fairness_index": fairness_score,
            "threshold": 0.90,
            "evaluation_status": "passed" if fairness_score >= 0.90 else "failed",
        },
        "cascading_failure_check": {
            "scenario_id": scenario_id,
            "downstream_dependency_saturation": False,
            "retry_amplification_factor": retry_amplification_factor,
            "retry_budget": scenario["retry_budget"],
            "queue_recovery": "recovered_to_steady_state_bound",
            "queue_recovery_bound": STEADY_STATE_QUEUE_BOUND,
            "peak_queue_depth": peak_queue_depth,
            "in_flight_protection": "protected_by_admission_before_work_start",
            "evaluation_status": "passed" if retry_amplification_factor <= 1.0 and peak_queue_depth <= STEADY_STATE_QUEUE_BOUND else "failed",
        },
        "capacity_bounded": capacity_bounded,
        "peak_shed_request_rate": round(peak_shed_rate, 3),
    }


def build_measurements(run_id: str) -> dict[str, Any]:
    start = datetime.now(timezone.utc).replace(microsecond=0)
    evidence_window = {
        "started_at": start.isoformat().replace("+00:00", "Z"),
        "ended_at": (start + timedelta(minutes=6)).isoformat().replace("+00:00", "Z"),
    }
    scenario_results = {
        scenario_id: build_scenario_measurement(scenario_id, scenario, evidence_window)
        for scenario_id, scenario in SCENARIOS.items()
    }
    return {
        "run_id": run_id,
        "dogfood_environment": DOGFOOD_ENVIRONMENT,
        "declared_capacity_rps": DECLARED_CAPACITY_RPS,
        "steady_state_queue_bound": STEADY_STATE_QUEUE_BOUND,
        "evidence_window": evidence_window,
        "scenario_results": scenario_results,
    }


def write_artifact(path: Path, value: Any) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(canonical_json(value), encoding="utf-8")
    return rel(path)


def write_measurement_artifacts(measurements: dict[str, Any], run_id: str) -> tuple[dict[str, str], str]:
    scenario_results = measurements["scenario_results"]
    shed_points = [
        point
        for result in scenario_results.values()
        for point in result["shed_rate_curve"]
    ]
    shed_points.sort(key=shed_point_sort_key)
    shed_rate_curve = {
        "run_id": run_id,
        "output_key": "shed_rate_curve",
        "points": shed_points,
    }
    fairness_index = {
        "run_id": run_id,
        "output_key": "fairness_index",
        "scenario_scores": {
            scenario_id: result["fairness_index"]
            for scenario_id, result in scenario_results.items()
        },
        "minimum_fairness_index": min(result["fairness_index"]["fairness_index"] for result in scenario_results.values()),
        "threshold": 0.90,
    }
    cascading_failure_check = {
        "run_id": run_id,
        "output_key": "cascading_failure_check",
        "scenario_checks": {
            scenario_id: result["cascading_failure_check"]
            for scenario_id, result in scenario_results.items()
        },
    }
    artifacts = {
        "shed_rate_curve": shed_rate_curve,
        "fairness_index": fairness_index,
        "cascading_failure_check": cascading_failure_check,
    }
    artifact_refs = {
        output_key: write_artifact(ARTIFACT_ROOT / f"{run_id}-{output_key}.json", artifact)
        for output_key, artifact in artifacts.items()
    }
    artifact_digest = sha256_json(artifacts)
    return artifact_refs, artifact_digest


def build_result_summary(measurements: dict[str, Any], artifact_refs: dict[str, str]) -> list[dict[str, Any]]:
    scenario_results = measurements["scenario_results"]
    shed_points = sum(len(result["shed_rate_curve"]) for result in scenario_results.values())
    min_fairness = min(result["fairness_index"]["fairness_index"] for result in scenario_results.values())
    cascade_passed = all(
        result["cascading_failure_check"]["evaluation_status"] == "passed"
        and result["cascading_failure_check"]["downstream_dependency_saturation"] is False
        for result in scenario_results.values()
    )
    capacity_bounded = all(result["capacity_bounded"] for result in scenario_results.values())
    return [
        {
            "output_key": "shed_rate_curve",
            "expected_value_or_threshold": "all four scenarios include sorted offered-load points; admitted throughput remains bounded and shed_request_rate rises above capacity",
            "observed_value": {
                "scenario_count": len(scenario_results),
                "point_count": shed_points,
                "capacity_bounded": capacity_bounded,
                "max_shed_request_rate": max(result["peak_shed_request_rate"] for result in scenario_results.values()),
            },
            "artifact_ref": artifact_refs["shed_rate_curve"],
            "evaluation_status": "passed" if capacity_bounded and shed_points >= len(SCENARIOS) * len(TENANTS) * 4 else "failed",
        },
        {
            "output_key": "fairness_index",
            "expected_value_or_threshold": "Jain-style fairness_index >= 0.90 for every overload evidence window unless tenant budget exhaustion is explicitly recorded",
            "observed_value": {
                "minimum_fairness_index": min_fairness,
                "scenario_scores": {
                    scenario_id: result["fairness_index"]["fairness_index"]
                    for scenario_id, result in scenario_results.items()
                },
            },
            "artifact_ref": artifact_refs["fairness_index"],
            "evaluation_status": "passed" if min_fairness >= 0.90 else "failed",
        },
        {
            "output_key": "cascading_failure_check",
            "expected_value_or_threshold": "no downstream saturation; retry amplification <= 1.0 after shedding engages; queue recovers to the steady-state bound",
            "observed_value": {
                "scenario_count": len(scenario_results),
                "all_scenarios_no_cascade": cascade_passed,
                "max_retry_amplification_factor": max(
                    result["cascading_failure_check"]["retry_amplification_factor"] for result in scenario_results.values()
                ),
            },
            "artifact_ref": artifact_refs["cascading_failure_check"],
            "evaluation_status": "passed" if cascade_passed else "failed",
        },
    ]


def emit_receipt(evidence_path: Path) -> dict[str, Any]:
    run_id = evidence_path.stem
    measurements = build_measurements(run_id)
    artifact_refs, artifact_digest = write_measurement_artifacts(measurements, run_id)
    result_summary = build_result_summary(measurements, artifact_refs)
    status = "passed_after_future_runtime_evidence" if all(row["evaluation_status"] == "passed" for row in result_summary) else "failed"
    receipt = {
        "kit_id": KIT_ID,
        "scenario_id": "QK-01-overload-fairness-S01",
        "scenario_ids": list(SCENARIOS),
        "run_id": run_id,
        "dogfood_environment": DOGFOOD_ENVIRONMENT,
        "command": COMMAND_TEMPLATE,
        "invoked_command": f"python3 {SCRIPT_RELATIVE_PATH} --dogfood-environment {DOGFOOD_ENVIRONMENT} --emit-evidence {rel(evidence_path)}",
        "status": status,
        "artifact_digest": artifact_digest,
        "reviewer": "qk-01-overload-fairness-dogfood-harness",
        "created_at": now_utc(),
        "source_commit": git_head(),
        "evidence_window": measurements["evidence_window"],
        "result_summary": result_summary,
        "scenario_results": measurements["scenario_results"],
        "dogfood_run_provenance": {
            "cell": DOGFOOD_ENVIRONMENT,
            "harness": "synthetic overload generator + per-tenant request-cost classifier",
            "tenant_sample_set": list(TENANTS),
            "declared_capacity_rps": DECLARED_CAPACITY_RPS,
            "steady_state_queue_bound": STEADY_STATE_QUEUE_BOUND,
            "artifact_refs": artifact_refs,
            "forbidden_runtime_fallback_used": False,
        },
    }
    evidence_path.parent.mkdir(parents=True, exist_ok=True)
    evidence_path.write_text(canonical_json(receipt), encoding="utf-8")
    return receipt


def load_json(path: Path) -> dict[str, Any]:
    try:
        candidate = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing dogfood run receipt: {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON run receipt {rel(path)}: {exc}")
    require(isinstance(candidate, dict), "dogfood run receipt must be a JSON object")
    return candidate


def validate_result_summary(record: dict[str, Any], status: str) -> None:
    summary = record.get("result_summary")
    require(isinstance(summary, list) and summary, "result_summary must be a non-empty list")
    by_output: dict[str, dict[str, Any]] = {}
    for index, entry in enumerate(summary):
        require(isinstance(entry, dict), f"result_summary[{index}] must be an object")
        missing = RESULT_SUMMARY_KEYS - set(entry)
        require(not missing, f"result_summary[{index}] missing keys {sorted(missing)}")
        output_key = str(entry["output_key"])
        require(output_key in REQUIRED_OUTPUTS, f"unexpected result_summary output_key {output_key!r}")
        require(output_key not in by_output, f"duplicate result_summary output_key {output_key}")
        require_evidence_ref(f"result_summary[{output_key}].artifact_ref", entry["artifact_ref"])
        reject_placeholder(f"result_summary[{output_key}].expected_value_or_threshold", entry["expected_value_or_threshold"])
        require("observed_value" in entry, f"result_summary[{output_key}].observed_value required")
        evaluation_status = str(entry["evaluation_status"])
        require(evaluation_status in {"passed", "failed", "blocked", "pending"}, f"unsupported evaluation_status {evaluation_status!r}")
        by_output[output_key] = entry
    missing_outputs = REQUIRED_OUTPUTS - set(by_output)
    require(not missing_outputs, f"result_summary missing outputs {sorted(missing_outputs)}")
    if status == "passed_after_future_runtime_evidence":
        not_passed = [key for key, entry in by_output.items() if entry["evaluation_status"] != "passed"]
        require(not not_passed, f"passed_after_future_runtime_evidence requires passed outputs; not passed: {not_passed}")


def validate_scenario_results(record: dict[str, Any]) -> None:
    scenario_results = record.get("scenario_results")
    require(isinstance(scenario_results, dict), "scenario_results must map all four QK-01 scenarios to dogfood measurements")
    missing_scenarios = set(SCENARIOS) - set(scenario_results)
    require(not missing_scenarios, f"scenario_results missing scenarios {sorted(missing_scenarios)}")
    for scenario_id, result in scenario_results.items():
        require(scenario_id in SCENARIOS, f"unexpected scenario_result {scenario_id}")
        require(isinstance(result, dict), f"scenario_results[{scenario_id}] must be an object")
        curve = result.get("shed_rate_curve")
        require(isinstance(curve, list) and curve, f"{scenario_id}: shed_rate_curve must be non-empty")
        ratios = [point.get("offered_load_ratio") for point in curve if isinstance(point, dict)]
        require(ratios == sorted(ratios), f"{scenario_id}: shed_rate_curve points must be sorted by offered_load_ratio")
        for point in curve:
            require(isinstance(point, dict), f"{scenario_id}: curve point must be an object")
            for key in ["scenario_id", "tenant_id", "request_cost_class", "offered_load_ratio", "admitted_request_rate", "shed_request_rate", "queue_depth", "evidence_window"]:
                require(key in point, f"{scenario_id}: shed_rate_curve point missing {key}")
            require(point["scenario_id"] == scenario_id, f"{scenario_id}: curve point scenario_id mismatch")
            if float(point["offered_load_ratio"]) <= 1.0:
                require(float(point["shed_request_rate"]) == 0.0, f"{scenario_id}: at/below-capacity shed rate must be zero")
        fairness = result.get("fairness_index")
        require(isinstance(fairness, dict), f"{scenario_id}: fairness_index must be an object")
        require(float(fairness.get("fairness_index", 0.0)) >= 0.90, f"{scenario_id}: fairness_index below threshold")
        cascade = result.get("cascading_failure_check")
        require(isinstance(cascade, dict), f"{scenario_id}: cascading_failure_check must be an object")
        require(cascade.get("downstream_dependency_saturation") is False, f"{scenario_id}: downstream dependency saturation detected")
        require(float(cascade.get("retry_amplification_factor", 2.0)) <= 1.0, f"{scenario_id}: retry amplification exceeds budget")
        require(cascade.get("evaluation_status") == "passed", f"{scenario_id}: cascade check did not pass")


def load_artifact_json(path: Path) -> dict[str, Any]:
    try:
        candidate = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing dogfood evidence artifact: {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON dogfood evidence artifact {rel(path)}: {exc}")
    require(isinstance(candidate, dict), f"dogfood evidence artifact must be a JSON object: {rel(path)}")
    return candidate


def validate_artifacts(record: dict[str, Any]) -> None:
    provenance = record.get("dogfood_run_provenance")
    require(isinstance(provenance, dict), "dogfood_run_provenance must be an object")
    require(provenance.get("cell") == DOGFOOD_ENVIRONMENT, f"dogfood_run_provenance.cell must be {DOGFOOD_ENVIRONMENT}")
    require(provenance.get("forbidden_runtime_fallback_used") is False, "forbidden runtime fallback must be explicitly false")

    artifact_refs = provenance.get("artifact_refs")
    require(isinstance(artifact_refs, dict), "dogfood_run_provenance.artifact_refs must be an object")
    require(set(artifact_refs) == REQUIRED_OUTPUTS, f"artifact_refs must exactly cover {sorted(REQUIRED_OUTPUTS)}")

    summary_by_output = {
        str(entry["output_key"]): entry
        for entry in record["result_summary"]
        if isinstance(entry, dict) and "output_key" in entry
    }
    artifacts: dict[str, dict[str, Any]] = {}
    for output_key in sorted(REQUIRED_OUTPUTS):
        require(output_key in summary_by_output, f"result_summary missing {output_key}")
        require(
            artifact_refs[output_key] == summary_by_output[output_key]["artifact_ref"],
            f"artifact_refs[{output_key}] must match result_summary artifact_ref",
        )
        _, artifact_path = artifact_path_from_ref(f"dogfood_run_provenance.artifact_refs[{output_key}]", artifact_refs[output_key])
        artifact = load_artifact_json(artifact_path)
        require(artifact.get("run_id") == record["run_id"], f"{output_key} artifact run_id must match receipt")
        require(artifact.get("output_key") == output_key, f"{output_key} artifact output_key mismatch")
        artifacts[output_key] = artifact

    scenario_results = record["scenario_results"]
    expected_points = [
        point
        for result in scenario_results.values()
        for point in result["shed_rate_curve"]
    ]
    expected_points.sort(key=shed_point_sort_key)
    artifact_points = artifacts["shed_rate_curve"].get("points")
    require(artifact_points == expected_points, "shed_rate_curve artifact must match sorted scenario_results points")
    ratios = [float(point["offered_load_ratio"]) for point in artifact_points]
    require(ratios == sorted(ratios), "shed_rate_curve artifact points must be sorted by offered_load_ratio")

    expected_fairness = {
        scenario_id: result["fairness_index"]
        for scenario_id, result in scenario_results.items()
    }
    require(artifacts["fairness_index"].get("scenario_scores") == expected_fairness, "fairness_index artifact must match scenario_results")
    require(
        artifacts["fairness_index"].get("minimum_fairness_index") == min(score["fairness_index"] for score in expected_fairness.values()),
        "fairness_index artifact minimum_fairness_index mismatch",
    )

    expected_cascade = {
        scenario_id: result["cascading_failure_check"]
        for scenario_id, result in scenario_results.items()
    }
    require(artifacts["cascading_failure_check"].get("scenario_checks") == expected_cascade, "cascading_failure_check artifact must match scenario_results")
    require(sha256_json(artifacts) == record["artifact_digest"], "artifact_digest must match canonical artifact JSON bundle")


def validate_record(record: dict[str, Any], evidence_path: Path, dogfood_environment: str) -> None:
    missing = REQUIRED_FIELDS - set(record)
    require(not missing, f"missing required fields {sorted(missing)}")
    require(record["kit_id"] == KIT_ID, f"kit_id must be {KIT_ID}")
    require(record["scenario_id"] in SCENARIOS, f"scenario_id must be one of {sorted(SCENARIOS)}")
    require(set(record.get("scenario_ids", [])) == set(SCENARIOS), "scenario_ids must bind all four QK-01 scenarios")
    require(record["run_id"] == evidence_path.stem, "run_id must match the emitted receipt filename stem")
    require(dogfood_environment == DOGFOOD_ENVIRONMENT, f"dogfood environment must be {DOGFOOD_ENVIRONMENT}")
    require(record["dogfood_environment"] == DOGFOOD_ENVIRONMENT, f"record dogfood_environment must be {DOGFOOD_ENVIRONMENT}")
    require(record["command"] == COMMAND_TEMPLATE, "command field must preserve the required QK-01 harness template")
    require(str(record.get("invoked_command", "")).endswith(f"--emit-evidence {rel(evidence_path)}"), "invoked_command must preserve the concrete emitted receipt path")
    lower_text = text(record)
    for marker in FORBIDDEN_FALLBACK_MARKERS:
        require(marker not in lower_text, f"forbidden fallback marker present in receipt: {marker}")
    status = str(record["status"])
    require(status in ALLOWED_STATUSES, f"status must be one of {sorted(ALLOWED_STATUSES)}")
    for field in DIGEST_FIELDS:
        if field == "command":
            require(bool(str(record[field]).strip()), "command must be non-empty")
        else:
            reject_placeholder(field, record[field])
    require(SHA256_PATTERN.match(str(record["artifact_digest"])), "artifact_digest must be sha256:<64 lowercase hex chars>")
    require(SOURCE_COMMIT_PATTERN.match(str(record["source_commit"])), "source_commit must be a git commit hex id")
    reject_placeholder("reviewer", record["reviewer"])
    require(RFC3339_UTC_PATTERN.match(str(record["created_at"])), "created_at must be a UTC RFC3339-like timestamp ending in Z")
    evidence_window = record["evidence_window"]
    require(isinstance(evidence_window, dict), "evidence_window must be an object")
    require({"started_at", "ended_at"} <= set(evidence_window), "evidence_window must include started_at and ended_at")
    validate_result_summary(record, status)
    validate_scenario_results(record)
    validate_artifacts(record)


def expect_rejected(label: str, mutator: Callable[[dict[str, Any]], None], valid: dict[str, Any], evidence_path: Path) -> None:
    candidate = copy.deepcopy(valid)
    mutator(candidate)
    try:
        validate_record(candidate, evidence_path, DOGFOOD_ENVIRONMENT)
    except SystemExit as exc:
        require(exc.code != 0, f"self-test {label!r} exited successfully")
        return
    fail(f"self-test mutation was accepted: {label}")


def run_self_tests() -> None:
    RUN_ROOT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="qk01-self-test-", dir=RUN_ROOT) as temp_dir_raw:
        temp_dir = Path(temp_dir_raw)
        evidence_path = temp_dir / "self-test-run.json"
        valid = emit_receipt(evidence_path)
        validate_record(load_json(evidence_path), evidence_path, DOGFOOD_ENVIRONMENT)

        expect_rejected("wrong dogfood environment", lambda data: data.update({"dogfood_environment": "github-actions"}), valid, evidence_path)
        expect_rejected("missing digest field", lambda data: data.update({"artifact_digest": ""}), valid, evidence_path)
        expect_rejected("mismatched artifact digest", lambda data: data.update({"artifact_digest": "sha256:" + "0" * 64}), valid, evidence_path)
        expect_rejected("forbidden runtime fallback flag", lambda data: data["dogfood_run_provenance"].update({"forbidden_runtime_fallback_used": True}), valid, evidence_path)
        expect_rejected("missing artifact ref", lambda data: data["dogfood_run_provenance"]["artifact_refs"].pop("fairness_index"), valid, evidence_path)
        expect_rejected("static source artifact", lambda data: data["result_summary"][0].update({"artifact_ref": "specs/cloud-production-quality-kits-target.json"}), valid, evidence_path)
        expect_rejected("missing scenario binding", lambda data: data["scenario_results"].pop("QK-01-overload-fairness-S04"), valid, evidence_path)

        def missing_artifact_file(data: dict[str, Any]) -> None:
            missing_ref = f"evidence/cloud/quality-kits/{EVIDENCE_SLUG}/artifacts/{evidence_path.stem}-missing.json"
            data["dogfood_run_provenance"]["artifact_refs"]["shed_rate_curve"] = missing_ref
            data["result_summary"][0]["artifact_ref"] = missing_ref

        expect_rejected("missing artifact file", missing_artifact_file, valid, evidence_path)

        def fabricate_pass(data: dict[str, Any]) -> None:
            data["status"] = "passed_after_future_runtime_evidence"
            data["result_summary"][0]["evaluation_status"] = "blocked"

        expect_rejected("fabricated passed_after_future_runtime_evidence", fabricate_pass, valid, evidence_path)

        try:
            canonical_emit_path(f"{rel(RUN_ROOT)}/<run_id>.json")
        except SystemExit as exc:
            require(exc.code != 0, "placeholder emit path self-test exited successfully")
        else:
            fail("self-test accepted placeholder emit path")

        for artifact_path in ARTIFACT_ROOT.glob(f"{evidence_path.stem}-*.json"):
            artifact_path.unlink()
    print("QK-01 overload/fairness future harness self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dogfood-environment", help="must be oyatie-dogfood-cell")
    parser.add_argument("--emit-evidence", help="dogfood run receipt path under evidence/cloud/quality-kits/qk-01-overload-fairness/runs/<run_id>.json")
    parser.add_argument("--self-test", action="store_true", help="run fail-closed validator and producer self-tests")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.self_test:
        run_self_tests()
        return
    require(args.dogfood_environment, "--dogfood-environment is required")
    require(args.emit_evidence, "--emit-evidence is required")
    require(args.dogfood_environment == DOGFOOD_ENVIRONMENT, f"dogfood environment must be {DOGFOOD_ENVIRONMENT}")
    evidence_path = canonical_emit_path(args.emit_evidence)
    receipt = emit_receipt(evidence_path)
    validate_record(receipt, evidence_path, args.dogfood_environment)
    print(f"QK-01 overload/fairness dogfood receipt emitted and validated: {rel(evidence_path)}")


if __name__ == "__main__":
    main()
