#!/usr/bin/env python3
"""Fail closed when the P0.0 CI-enforcement baseline omits executable fixtures.

P0.0 relies on a checked-in RED gap packet until cloud-ci/oya-ci required
status is live. That packet is only useful if every fixture file under the
baseline directory is enumerated, paired when it is a RED/GREEN policy case,
and reachable from a Buck2 target instead of operator memory.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(os.environ.get("OYA_REPO_ROOT", Path(__file__).resolve().parents[2])).resolve()

BASELINE_DEFAULT = "specs/phase0-ci-enforcement-baseline.json"

STALE_REQUIRED_CONTEXT_PHRASES = [
    "live GitHub branch protection still lacks cloud-ci-required/oya-ci-required",
    "current branch protection still lacks cloud-ci-required/oya-ci-required",
]

REQUIRED_CONTEXT_NARRATIVE_DOCS = [
    "specs/phase0-automation-matrix.json",
    "specs/phase0-claim-evidence-map.json",
]


def repo_path(path: str | Path) -> Path:
    p = Path(path)
    return p if p.is_absolute() else REPO_ROOT / p


def load_json(path: Path) -> Any:
    with path.open() as fh:
        return json.load(fh)


def as_list(value: Any, field: str, failures: list[str]) -> list[Any]:
    if isinstance(value, list):
        return value
    failures.append(f"{field}: expected list")
    return []


def rel(path: Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def fixture_verdict(path: Path) -> str | None:
    data = load_json(path)
    verdict = data.get("expected_verdict")
    return verdict if isinstance(verdict, str) else None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--baseline",
        default=BASELINE_DEFAULT,
        help="Baseline JSON path, relative to the repository root by default.",
    )
    args = parser.parse_args()

    failures: list[str] = []
    baseline_path = repo_path(args.baseline)
    baseline = load_json(baseline_path)
    fixture_set = baseline.get("fixture_set")
    if not isinstance(fixture_set, dict):
        print("phase0-ci-baseline-catalog: RED", file=sys.stderr)
        print("- fixture_set: expected object", file=sys.stderr)
        return 1

    fixture_dir_value = fixture_set.get("fixture_directory")
    if not isinstance(fixture_dir_value, str):
        failures.append("fixture_set.fixture_directory: expected string")
        fixture_dir = REPO_ROOT / "<missing>"
    else:
        fixture_dir = repo_path(fixture_dir_value)

    actual_fixture_paths = sorted(
        rel(path) for path in fixture_dir.glob("*.json") if path.is_file()
    )
    listed_fixture_paths = sorted(
        str(item)
        for item in as_list(fixture_set.get("all_fixture_paths"), "fixture_set.all_fixture_paths", failures)
        if isinstance(item, str)
    )
    listed_set = set(listed_fixture_paths)
    actual_set = set(actual_fixture_paths)

    missing_from_catalog = sorted(actual_set - listed_set)
    stale_catalog_entries = sorted(listed_set - actual_set)
    for item in missing_from_catalog:
        failures.append(f"fixture_set.all_fixture_paths missing actual fixture: {item}")
    for item in stale_catalog_entries:
        failures.append(f"fixture_set.all_fixture_paths lists missing fixture: {item}")

    # Single-field fixture references must exist and, when they point at the
    # baseline fixture directory, must also be in all_fixture_paths.
    for field in [
        "current_red_result_fixture",
        "result_schema",
        "override_packet_schema",
        "trusted_target_inventory_schema",
        "tenant_fixture_contract",
    ]:
        value = fixture_set.get(field)
        if not isinstance(value, str):
            failures.append(f"fixture_set.{field}: expected string path")
            continue
        path = repo_path(value)
        if not path.is_file():
            failures.append(f"fixture_set.{field}: missing path {value}")
        if fixture_dir in path.parents and value not in listed_set:
            failures.append(f"fixture_set.{field}: {value} is not included in all_fixture_paths")

    # Multi-field fixture references must also be closed over the all-fixture catalog.
    for field in ["result_bundle_fixture_paths", "trusted_target_inventory_fixture_paths"]:
        for value in as_list(fixture_set.get(field), f"fixture_set.{field}", failures):
            if not isinstance(value, str):
                failures.append(f"fixture_set.{field}: non-string path {value!r}")
                continue
            path = repo_path(value)
            if not path.is_file():
                failures.append(f"fixture_set.{field}: missing path {value}")
            if fixture_dir in path.parents and value not in listed_set:
                failures.append(f"fixture_set.{field}: {value} is not included in all_fixture_paths")

    fixture_by_basename = {Path(path).name: path for path in actual_fixture_paths}
    red_pair_basenames: set[str] = set()
    reachable_fixture_basenames: set[str] = set()
    pair_count = 0
    for pair in as_list(fixture_set.get("required_red_green_pairs"), "fixture_set.required_red_green_pairs", failures):
        pair_count += 1
        if not isinstance(pair, dict):
            failures.append(f"required_red_green_pairs[{pair_count - 1}]: expected object")
            continue
        tc_id = pair.get("tc_id")
        red = pair.get("red")
        green = pair.get("green")
        for key, value in [("tc_id", tc_id), ("red", red), ("green", green)]:
            if not isinstance(value, str) or not value:
                failures.append(f"required_red_green_pairs[{pair_count - 1}].{key}: expected non-empty string")
        if not isinstance(red, str) or not isinstance(green, str):
            continue
        red_path = fixture_by_basename.get(red)
        green_path = fixture_by_basename.get(green)
        if red_path is None:
            failures.append(f"required_red_green_pairs[{pair_count - 1}].red: unknown fixture {red}")
            continue
        if green_path is None:
            failures.append(f"required_red_green_pairs[{pair_count - 1}].green: unknown fixture {green}")
            continue
        red_pair_basenames.add(red)
        reachable_fixture_basenames.update({red, green})
        red_verdict = fixture_verdict(repo_path(red_path))
        green_verdict = fixture_verdict(repo_path(green_path))
        if red_verdict != "RED":
            failures.append(f"{red}: red pair fixture must declare expected_verdict=RED, got {red_verdict!r}")
        if green_verdict != "GREEN":
            failures.append(f"{green}: green pair fixture must declare expected_verdict=GREEN, got {green_verdict!r}")
        if not isinstance(tc_id, str) or not tc_id.startswith("TC-0.0"):
            failures.append(f"required_red_green_pairs[{pair_count - 1}].tc_id: expected TC-0.0* id")

    for field in ["result_bundle_fixture_paths", "trusted_target_inventory_fixture_paths"]:
        for value in fixture_set.get(field, []):
            if isinstance(value, str):
                reachable_fixture_basenames.add(Path(value).name)

    current_red = fixture_set.get("current_red_result_fixture")
    if isinstance(current_red, str):
        reachable_fixture_basenames.add(Path(current_red).name)

    for path in actual_fixture_paths:
        basename = Path(path).name
        data = load_json(repo_path(path))
        expected_verdict = data.get("expected_verdict")
        if expected_verdict == "RED" and basename not in red_pair_basenames:
            failures.append(f"{basename}: RED expected_verdict fixture is not covered by required_red_green_pairs")
        if expected_verdict in {"RED", "GREEN"} and basename not in reachable_fixture_basenames:
            failures.append(f"{basename}: expected_verdict fixture is not reachable from any catalog execution bucket")

    if isinstance(current_red, str) and repo_path(current_red).is_file():
        data = load_json(repo_path(current_red))
        if data.get("observed_verdict") != "RED":
            failures.append(f"{current_red}: current RED gap result must keep observed_verdict=RED")
        boundary = data.get("claim_boundary")
        if not isinstance(boundary, dict) or boundary.get("p0_0_green") is not False or boundary.get("phase0_complete") is not False:
            failures.append(f"{current_red}: current RED gap result must declare p0_0_green=false and phase0_complete=false")

    false_green_paths = [
        p
        for p in as_list(fixture_set.get("result_bundle_fixture_paths"), "fixture_set.result_bundle_fixture_paths", failures)
        if isinstance(p, str) and "false-green" in p
    ]
    if not false_green_paths:
        failures.append("fixture_set.result_bundle_fixture_paths: missing false-green result-bundle fixture")
    for value in false_green_paths:
        data = load_json(repo_path(value))
        boundary = data.get("claim_boundary")
        if not isinstance(boundary, dict) or boundary.get("p0_0_green") is not True or boundary.get("phase0_complete") is not True:
            failures.append(f"{value}: false-green fixture must exercise p0_0_green=true and phase0_complete=true")

    claim_boundary = baseline.get("claim_boundary")
    if not isinstance(claim_boundary, dict):
        failures.append("claim_boundary: expected object")
    else:
        if claim_boundary.get("p0_0_green") is not False:
            failures.append("claim_boundary.p0_0_green must remain false in the RED gap packet")
        if claim_boundary.get("phase0_complete") is not False:
            failures.append("claim_boundary.phase0_complete must remain false in the RED gap packet")

    automation_mapping = baseline.get("automation_mapping")
    if not isinstance(automation_mapping, dict):
        failures.append("automation_mapping: expected object")
    else:
        if automation_mapping.get("source_app_binding_check_target") != "//:phase0-required-status-source-check":
            failures.append("automation_mapping.source_app_binding_check_target must be //:phase0-required-status-source-check")
        if automation_mapping.get("source_app_binding_check_script") != "scripts/ci/assert-required-status-source.py":
            failures.append("automation_mapping.source_app_binding_check_script must be scripts/ci/assert-required-status-source.py")
        if automation_mapping.get("source_app_binding_test") != "scripts/tests/phase0_required_status_source_check.test.sh":
            failures.append("automation_mapping.source_app_binding_test must be scripts/tests/phase0_required_status_source_check.test.sh")
        if automation_mapping.get("tenant_isolation_check_target") != "//:phase0-tenant-isolation-fixture-check":
            failures.append("automation_mapping.tenant_isolation_check_target must be //:phase0-tenant-isolation-fixture-check")
        if automation_mapping.get("tenant_isolation_check_script") != "scripts/ci/assert-tenant-pipeline-isolation.py":
            failures.append("automation_mapping.tenant_isolation_check_script must be scripts/ci/assert-tenant-pipeline-isolation.py")
        if automation_mapping.get("tenant_isolation_test") != "scripts/tests/phase0_tenant_isolation_fixture_check.test.sh":
            failures.append("automation_mapping.tenant_isolation_test must be scripts/tests/phase0_tenant_isolation_fixture_check.test.sh")
        if automation_mapping.get("override_kill_switch_check_target") != "//:phase0-override-kill-switch-check":
            failures.append("automation_mapping.override_kill_switch_check_target must be //:phase0-override-kill-switch-check")
        if automation_mapping.get("override_kill_switch_check_script") != "scripts/ci/assert-override-kill-switch.py":
            failures.append("automation_mapping.override_kill_switch_check_script must be scripts/ci/assert-override-kill-switch.py")
        if automation_mapping.get("override_kill_switch_test") != "scripts/tests/phase0_override_kill_switch_check.test.sh":
            failures.append("automation_mapping.override_kill_switch_test must be scripts/tests/phase0_override_kill_switch_check.test.sh")

    for doc in REQUIRED_CONTEXT_NARRATIVE_DOCS:
        path = repo_path(doc)
        if not path.is_file():
            failures.append(f"required-context narrative doc missing: {doc}")
            continue
        text = path.read_text()
        for phrase in STALE_REQUIRED_CONTEXT_PHRASES:
            if phrase in text:
                failures.append(
                    f"{doc}: stale required-context gap phrase must be replaced with current context-present/source-app-unbound wording"
                )

    source_fixture_dir_value = fixture_set.get("required_status_source_fixture_directory")
    if not isinstance(source_fixture_dir_value, str):
        failures.append("fixture_set.required_status_source_fixture_directory: expected string")
        source_fixture_dir = REPO_ROOT / "<missing-required-status-source>"
    else:
        source_fixture_dir = repo_path(source_fixture_dir_value)
    actual_source_fixture_paths = sorted(
        rel(path) for path in source_fixture_dir.glob("*.json") if path.is_file()
    )
    listed_source_fixture_paths = sorted(
        str(item)
        for item in as_list(
            fixture_set.get("required_status_source_fixture_paths"),
            "fixture_set.required_status_source_fixture_paths",
            failures,
        )
        if isinstance(item, str)
    )
    for item in sorted(set(actual_source_fixture_paths) - set(listed_source_fixture_paths)):
        failures.append(f"fixture_set.required_status_source_fixture_paths missing actual fixture: {item}")
    for item in sorted(set(listed_source_fixture_paths) - set(actual_source_fixture_paths)):
        failures.append(f"fixture_set.required_status_source_fixture_paths lists missing fixture: {item}")
    for value in listed_source_fixture_paths:
        path = repo_path(value)
        if not path.is_file():
            failures.append(f"fixture_set.required_status_source_fixture_paths: missing path {value}")
            continue
        data = load_json(path)
        if data.get("contexts") is None:
            failures.append(f"{value}: required-status source fixture must include contexts")

    if failures:
        print("phase0-ci-baseline-catalog: RED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(
        json.dumps(
            {
                "verdict": "PASS",
                "baseline": rel(baseline_path),
                "fixture_directory": rel(fixture_dir),
                "fixture_count": len(actual_fixture_paths),
                "required_red_green_pairs": pair_count,
                "result_bundle_fixtures": len(fixture_set.get("result_bundle_fixture_paths", [])),
                "trusted_target_inventory_fixtures": len(fixture_set.get("trusted_target_inventory_fixture_paths", [])),
                "required_status_source_fixtures": len(actual_source_fixture_paths),
                "claim_boundary": {
                    "p0_0_green": False,
                    "phase0_complete": False,
                },
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
