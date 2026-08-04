#!/usr/bin/env python3
"""Validate the CONF-001 release-candidate conformance evidence packet.

This is a stdlib-only evidence/gate validator. It validates the packet shape and
claim ceiling; it intentionally does not claim runtime SLOs, public SLA/SLOs,
production readiness, branch-protected enforcement, or hyperscaler maturity.
"""
from __future__ import annotations

import argparse
import copy
import json
import re
import sys
from pathlib import Path
from typing import Any, NoReturn, cast
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_PACKET = REPO_ROOT / "evidence" / "conformance" / "conf-001-release-candidate-conformance-20260702.json"

REQUIRED_SECTIONS = {
    "competitor_benchmark_row",
    "performance_target",
    "load_test_section",
    "openslo_error_budget_policy",
    "maturity_claim_evidence",
    "six_axis_hyperscaler_conformance_fixture",
}
REQUIRED_AXES = {
    "axis-1-pipeline",
    "axis-2-directory",
    "axis-3-naming",
    "axis-4-standards",
    "axis-5-practices",
    "axis-6-policies",
}
REQUIRED_CLAIM_CEILING_FLAGS = {
    "no_product_runtime_change",
    "no_cloud_runtime_change",
    "no_measured_slo_claim",
    "no_public_sla_slo_claim",
    "no_production_readiness_claim",
    "no_hyperscaler_maturity_claim",
    "no_branch_protection_claim",
}
REQUIRED_LOAD_RECEIPT_FIELDS = {
    "tool and script path",
    "release_candidate_id",
    "source_commit",
    "dogfood_environment",
    "target_surface",
    "p50_p95_p99_p999_latency_results",
    "throughput_results",
    "capacity_breakpoint",
    "error_budget_impact",
    "rollback_or_fail_shed_decision_rule",
    "reviewer",
}
REQUIRED_SLO_FIELDS = {
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
OFFICIAL_SOURCE_HOST_SUFFIXES = {
    "docs.aws.amazon.com",
    "aws.amazon.com",
    "sre.google",
    "learn.microsoft.com",
    "openslo.com",
}
FORBIDDEN_POSITIVE_CLAIM_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in [
        r"\bproduction[-\s]?ready\b.{0,60}\b(achieved|green|passed|complete|met|available|enabled|satisfied)\b",
        r"\bproduction\s+readiness\b.{0,60}\b(achieved|green|passed|complete|met|available|enabled|satisfied)\b",
        r"\bhyperscaler[-\s]?(grade|mature|maturity|ready|readiness)\b.{0,60}\b(achieved|green|passed|complete|met|available|enabled|satisfied)\b",
        r"\b(public\s+)?(sla|slo)\b.{0,60}\b(achieved|green|passed|published|met|available|enabled|satisfied)\b",
        r"\btenant\s+workloads?\b.{0,60}\b(can\s+run|ready|safe|supported|enabled)\b",
        r"\bfeature[-\s]?parity\b.{0,60}\b(achieved|green|passed|complete|met)\b",
        r"\bbeats?\b.{0,30}\b(aws|azure|google|competitor)\b",
        r"\bbetter\s+than\b.{0,30}\b(aws|azure|google|competitor)\b",
    ]
]


def fail(message: str) -> NoReturn:
    print(f"CONF-001 hyperscaler conformance check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def rel(path: Path) -> str:
    return str(path.resolve().relative_to(REPO_ROOT))


def load_packet(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing packet {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {rel(path)}: {exc}")


def text(value: object) -> str:
    if isinstance(value, dict):
        return " ".join(text(item) for item in value.values())
    if isinstance(value, (list, tuple, set)):
        return " ".join(text(item) for item in value)
    return str(value)


def contains_forbidden_positive_claim(value: object) -> bool:
    rendered = re.sub(r"\s+", " ", text(value))
    return any(pattern.search(rendered) for pattern in FORBIDDEN_POSITIVE_CLAIM_PATTERNS)


def source_host_allowed(url: str) -> bool:
    parsed = urlparse(url)
    require(parsed.scheme == "https", f"official source URL must use https: {url}")
    host = parsed.netloc.lower()
    return any(host == suffix or host.endswith(f".{suffix}") for suffix in OFFICIAL_SOURCE_HOST_SUFFIXES)


def require_list(value: object, label: str) -> list[Any]:
    require(isinstance(value, list) and value, f"{label} must be a non-empty list")
    return cast(list[Any], value)


def validate(packet: dict[str, Any]) -> None:
    for field in [
        "artifact_id",
        "task_id",
        "created_at",
        "source_commit",
        "artifact_type",
        "purpose",
        "release_candidate",
        "claim_ceiling",
        "authority",
        "competitor_benchmark_row",
        "performance_target",
        "load_test_section",
        "openslo_error_budget_policy",
        "maturity_claim_evidence",
        "six_axis_hyperscaler_conformance_fixture",
        "acceptance_criteria_trace",
        "validation",
    ]:
        require(field in packet, f"missing top-level field {field!r}")

    require(packet["task_id"] == "t_7dd07ee9", "task_id must bind packet to CONF-001 Kanban task")
    require(packet["artifact_type"] == "release_candidate_conformance_evidence_gate_fixture", "unexpected artifact_type")

    release_candidate = packet["release_candidate"]
    require(release_candidate.get("id") == "rc-conf-001-governance-evidence-20260702", "unexpected release candidate id")
    attached_sections = set(require_list(release_candidate.get("attached_evidence_sections"), "release_candidate.attached_evidence_sections"))
    require(attached_sections == REQUIRED_SECTIONS, f"release candidate must attach exactly required sections; got {sorted(attached_sections)}")
    non_goals = set(require_list(release_candidate.get("non_goals"), "release_candidate.non_goals"))
    require("no product or cloud runtime mutation" in non_goals, "non_goals must preserve no-runtime-mutation boundary")
    require("no production-readiness or hyperscaler-maturity promotion" in non_goals, "non_goals must block positive maturity promotion")

    claim_ceiling = packet["claim_ceiling"]
    require(claim_ceiling.get("current_claim_tier") == "target_non_claim", "current_claim_tier must remain target_non_claim")
    for flag in REQUIRED_CLAIM_CEILING_FLAGS:
        require(claim_ceiling.get(flag) is True, f"claim_ceiling.{flag} must be true")
    require(not contains_forbidden_positive_claim(claim_ceiling.get("can_claim_now", [])), "claim_ceiling.can_claim_now contains forbidden positive claim wording")
    cannot_claim = " ".join(str(item).lower() for item in require_list(claim_ceiling.get("cannot_claim_yet"), "claim_ceiling.cannot_claim_yet"))
    for phrase in ["production", "hyperscaler", "measured slo", "public sla", "tenant workload", "branch-protected"]:
        require(phrase in cannot_claim, f"claim_ceiling.cannot_claim_yet must mention {phrase!r}")

    authority = {row.get("id"): row for row in require_list(packet.get("authority"), "authority")}
    for adr in ["ADR-0062", "ADR-0123", "ADR-0128", "ADR-0133", "ADR-0134"]:
        require(adr in authority, f"authority missing {adr}")
    require(authority["ADR-0134"].get("status") == "Proposed", "ADR-0134 must remain Proposed")
    require(authority["ADR-0134"].get("authority_use") == "advisory_remediation_backlog_only", "ADR-0134 must be advisory only")
    forbidden_uses = set(authority["ADR-0134"].get("not_used_for", []))
    require({"binding merge authority", "branch-protection status checks", "claim promotion"} <= forbidden_uses, "ADR-0134 not_used_for must block binding elevation")

    benchmark = packet["competitor_benchmark_row"]
    require(benchmark.get("benchmark_id") == "CONF-001-BENCH-REL-SLO-GATE", "unexpected benchmark_id")
    source_rows = require_list(benchmark.get("industry_references"), "competitor_benchmark_row.industry_references")
    providers = {row.get("provider") for row in source_rows}
    require({"aws", "google_sre", "azure", "openslo"} <= providers, "benchmark must cite AWS, Google SRE, Azure, and OpenSLO")
    for row in source_rows:
        require(row.get("source_status") == "official", f"benchmark source is not official: {row!r}")
        require(row.get("url") and source_host_allowed(row["url"]), f"benchmark URL is not on official allowlist: {row.get('url')}")
        require(row.get("observed_strength"), f"benchmark source missing observed_strength: {row!r}")
    require(require_list(benchmark.get("oyatie_adopt_decisions"), "benchmark.oyatie_adopt_decisions"), "adopt decisions required")
    require(require_list(benchmark.get("oyatie_improve_beyond_actions"), "benchmark.oyatie_improve_beyond_actions"), "improve-beyond actions required")
    require(not contains_forbidden_positive_claim(benchmark.get("claim_boundary", "")), "benchmark claim boundary contains forbidden positive claim")

    perf = packet["performance_target"]
    require(perf.get("measurement_status") == "target_not_measured_in_this_slice", "performance target must remain target-only for this slice")
    latency = perf.get("latency_budget", {})
    require(latency.get("read_only_p99_ms") == 50, "read-only p99 target must preserve ADR-0062 50ms")
    require(latency.get("mutation_p99_ms") == 200, "mutation p99 target must preserve ADR-0062 200ms")
    for percentile in ["p50_required_before_claim", "p95_required_before_claim", "p99_required_before_claim", "p999_required_before_claim"]:
        require(latency.get(percentile) is True, f"latency_budget.{percentile} must be true")
    throughput = perf.get("throughput_budget", {})
    require(throughput.get("per_cell_baseline_rps") == 10000, "per-cell baseline RPS must preserve ADR-0062 target")
    require(throughput.get("aggregate_via_cell_sharding_rps") == 100000, "aggregate RPS must preserve ADR-0062 target")
    error_budget = perf.get("error_budget_allocation", {})
    for field in ["openslo_policy_required", "fast_burn_threshold_required", "slow_burn_threshold_required", "release_freeze_or_throttle_required_on_exhaustion"]:
        require(error_budget.get(field) is True, f"error_budget_allocation.{field} must be true")

    load = packet["load_test_section"]
    require(load.get("status") == "receipt_shape_required_no_runtime_run_in_this_slice", "load-test section must not claim a runtime run")
    require({"k6", "locust", "vegeta"} <= set(load.get("accepted_tools", [])), "load-test accepted tools must include k6, locust, and vegeta")
    require(REQUIRED_LOAD_RECEIPT_FIELDS <= set(load.get("required_before_any_positive_readiness_claim", [])), "load-test section missing future receipt fields")
    explicit_na = load.get("explicit_na_for_this_slice", {})
    for field in ["runtime_load_test_run", "measured_latency", "measured_throughput"]:
        require(field in explicit_na and "N/A" in explicit_na[field], f"load-test explicit N/A missing for {field}")

    openslo = packet["openslo_error_budget_policy"]
    require(openslo.get("measurement_status") == "policy_shape_only_not_measured", "OpenSLO policy must remain not measured")
    require(openslo.get("document_format") == "OpenSLO", "OpenSLO policy must use OpenSLO document format")
    require(openslo.get("telemetry_standard") == "OpenTelemetry", "OpenSLO policy must use OpenTelemetry")
    require(REQUIRED_SLO_FIELDS <= set(openslo.get("required_slo_fields", [])), "OpenSLO policy missing required SLO fields")
    require({"5m", "30m", "1h", "2h", "6h", "1d", "3d"} <= set(openslo.get("burn_rate_windows", [])), "OpenSLO policy missing burn-rate windows")
    decision_text = " ".join(openslo.get("release_decision_rules", [])).lower()
    for phrase in ["fast-burn", "slow-burn", "error-budget exhaustion", "external hyperscaler console"]:
        require(phrase in decision_text, f"OpenSLO release decision rules must mention {phrase}")
    require("no measured SLO" in openslo.get("nonclaim_boundary", ""), "OpenSLO nonclaim boundary must block measured SLO")

    maturity = packet["maturity_claim_evidence"]
    require(maturity.get("current_tier") == "target_non_claim", "maturity evidence must remain target_non_claim")
    for field in ["required_before_mechanically_enforced", "required_before_production_ready", "required_before_hyperscaler_grade"]:
        require_list(maturity.get(field), f"maturity_claim_evidence.{field}")
    mechanically = set(maturity["required_before_mechanically_enforced"])
    require({"cloud_ci_required_context", "branch_protection_mapping", "Rust_gate_or_controller", "known_BAD_fixture_RED", "known_GOOD_fixture_GREEN", "current_SHA_status"} <= mechanically, "mechanically enforced prerequisites incomplete")
    production = set(maturity["required_before_production_ready"])
    require({"SLOs_and_error_budget_policy", "load_capacity_breakpoint", "performance_regression_guard", "tenant_isolation_negative_tests"} <= production, "production-ready prerequisites incomplete")
    hyper = set(maturity["required_before_hyperscaler_grade"])
    require({"all_production_ready_evidence", "error_budget_release_policy_enforced", "peak_load_or_capacity_drill", "90_day_sustained_SLO_window"} <= hyper, "hyperscaler-grade prerequisites incomplete")

    fixture = packet["six_axis_hyperscaler_conformance_fixture"]
    require(fixture.get("fixture_id") == "CONF-001-ADR-0133-SIX-AXIS-FIXTURE", "unexpected six-axis fixture id")
    require(fixture.get("source_adr") == "ADR-0133", "six-axis fixture must cite ADR-0133")
    axes = require_list(fixture.get("axes"), "six_axis_hyperscaler_conformance_fixture.axes")
    axis_ids = [axis.get("axis_id") for axis in axes]
    require(set(axis_ids) == REQUIRED_AXES and len(axis_ids) == len(REQUIRED_AXES), f"six-axis fixture must include each required axis exactly once; got {axis_ids}")
    for axis in axes:
        require(require_list(axis.get("industry_baseline"), f"axis {axis.get('axis_id')} industry_baseline"), "industry baseline required")
        require(axis.get("release_candidate_requirement"), f"axis {axis.get('axis_id')} missing release_candidate_requirement")
    assertions = set(require_list(fixture.get("fixture_assertions"), "six_axis_hyperscaler_conformance_fixture.fixture_assertions"))
    require("all six ADR-0133 axes are present exactly once" in assertions, "six-axis fixture must assert complete axis coverage")
    require("ADR-0134 is treated as Proposed/advisory only" in assertions, "six-axis fixture must assert ADR-0134 advisory treatment")

    trace = require_list(packet.get("acceptance_criteria_trace"), "acceptance_criteria_trace")
    trace_sections = {row.get("section") for row in trace if row.get("status") == "attached"}
    require(trace_sections == REQUIRED_SECTIONS, f"acceptance trace must attach each required section; got {sorted(trace_sections)}")

    validation = packet["validation"]
    require(validation.get("command") == "python3 scripts/tests/conf_001_hyperscaler_conformance_check.py", "validation command mismatch")
    require(validation.get("self_test_command") == "python3 scripts/tests/conf_001_hyperscaler_conformance_check.py --self-test", "self-test command mismatch")
    require(validation.get("expected_result") == "shape_valid_nonclaim", "expected_result must remain shape_valid_nonclaim")


def run_self_tests(packet: dict[str, Any]) -> None:
    def expect_rejected(label: str, mutator: Any) -> None:
        candidate = copy.deepcopy(packet)
        mutator(candidate)
        try:
            validate(candidate)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected(
        "missing attached section",
        lambda data: data["release_candidate"]["attached_evidence_sections"].remove("load_test_section"),
    )
    expect_rejected(
        "ADR-0134 elevated to binding",
        lambda data: data["authority"][4].update({"status": "Accepted", "authority_use": "binding_merge_authority"}),
    )
    expect_rejected(
        "missing six-axis row",
        lambda data: data["six_axis_hyperscaler_conformance_fixture"]["axes"].pop(),
    )
    expect_rejected(
        "positive production readiness claim",
        lambda data: data["claim_ceiling"]["can_claim_now"].append("production readiness achieved and green for tenant workloads"),
    )
    expect_rejected(
        "unofficial benchmark URL",
        lambda data: data["competitor_benchmark_row"]["industry_references"][0].update({"url": "https://example.com/aws-like"}),
    )
    expect_rejected(
        "load-test runtime overclaim",
        lambda data: data["load_test_section"].update({"status": "runtime_load_test_passed"}),
    )
    expect_rejected(
        "OpenSLO measured overclaim",
        lambda data: data["openslo_error_budget_policy"].update({"measurement_status": "measured_slo_passed"}),
    )
    expect_rejected(
        "maturity tier promotion",
        lambda data: data["maturity_claim_evidence"].update({"current_tier": "hyperscaler_grade"}),
    )
    print("CONF-001 hyperscaler conformance self-tests passed")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packet", default=str(DEFAULT_PACKET), help="packet path to validate")
    parser.add_argument("--self-test", action="store_true", help="run adversarial validator self-tests")
    args = parser.parse_args()

    packet_path = Path(args.packet)
    if not packet_path.is_absolute():
        packet_path = REPO_ROOT / packet_path
    packet = load_packet(packet_path)
    if args.self_test:
        run_self_tests(packet)
    validate(packet)
    print(f"CONF-001 hyperscaler conformance check passed: {rel(packet_path)} (shape_valid_nonclaim)")


if __name__ == "__main__":
    main()
