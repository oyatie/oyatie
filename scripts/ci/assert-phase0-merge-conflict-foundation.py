#!/usr/bin/env python3
"""Validate AC-0.15 generated-artifact and merge-conflict foundation.

This is local/static fixture evidence only. It proves the checked-in seed
registry, conflict taxonomy, merge-tree readiness fixtures, and one-lane-one-path
fail-closed cases are wired through Buck2. It never posts statuses, mutates
branch protection, presses a provider merge button, proves full generated-output
coverage, claims Phase-1 Tide batching, or claims P0.0/Phase-0 completion.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

DEFAULT_REGISTRY = Path("specs/generated-artifact-registry.json")
REQUIRED_TAXONOMY_IDS = {
    "clean_merge",
    "merge_tree_conflict",
    "path_overlap_without_review",
    "generated_artifact_missing_registry",
    "generated_artifact_stale_output",
    "one_lane_one_path_violation",
    "phase1_tide_batched_projection_overclaim",
}
FALSE_CLAIMS = (
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "phase1_tide_batching_claimed",
    "full_repo_generated_artifact_coverage_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)
# Policy anchors: forbidden_true_or_missing_claim_p0_0_green, phase1_tide_batching_claimed
REQUIRED_ARTIFACT_FIELDS = (
    "id",
    "output_path",
    "generator",
    "source_paths",
    "regeneration_command",
    "owner_team",
    "commit_policy",
    "drift_gate",
    "stale_output_policy",
    "path_claims",
)
REQUIRED_AUTOMATED_CHAIN_TOKENS = (
    "//:phase0-merge-conflict-foundation-check",
    "scripts/ci/assert-phase0-merge-conflict-foundation.py",
    "scripts/tests/phase0_merge_conflict_foundation_check.test.sh",
    "git merge-tree --write-tree",
)


def load_json(path: Path) -> dict[str, Any]:
    with path.open() as fh:
        data = json.load(fh)
    if not isinstance(data, dict):
        raise TypeError(f"{path}: expected JSON object")
    return data


def string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, str)]


def object_list(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, dict)]


def display_path(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return str(path)


def read(path: Path) -> str:
    return path.read_text(errors="replace")


def validate_false_claims(mapping: dict[str, Any], failures: list[str], *, prefix: str = "") -> None:
    for claim in FALSE_CLAIMS:
        if mapping.get(claim) is not False:
            failures.append(f"{prefix}forbidden_true_or_missing_claim_{claim}")


def validate_registry(root: Path, registry: dict[str, Any]) -> tuple[list[str], dict[str, dict[str, Any]]]:
    failures: list[str] = []
    boundary = registry.get("claim_boundary") if isinstance(registry.get("claim_boundary"), dict) else {}
    if boundary.get("generated_artifact_registry_published") is not True:
        failures.append("generated_artifact_registry_not_published")
    if boundary.get("merge_tree_fixture_contract_measured") is not True:
        failures.append("merge_tree_fixture_contract_not_measured")
    validate_false_claims(boundary, failures)

    scope = registry.get("coverage_scope") if isinstance(registry.get("coverage_scope"), dict) else {}
    if scope.get("full_repo_generated_artifact_coverage_proven") is not False:
        failures.append("forbidden_true_or_missing_coverage_scope_full_repo_generated_artifact_coverage_proven")

    taxonomy = object_list(registry.get("conflict_taxonomy"))
    taxonomy_ids = {item.get("id") for item in taxonomy if isinstance(item.get("id"), str)}
    missing_taxonomy = sorted(REQUIRED_TAXONOMY_IDS - taxonomy_ids)
    for item in missing_taxonomy:
        failures.append(f"missing_conflict_taxonomy:{item}")

    readiness = registry.get("merge_tree_readiness") if isinstance(registry.get("merge_tree_readiness"), dict) else {}
    if readiness.get("engine") != "git merge-tree --write-tree":
        failures.append("missing_git_merge_tree_write_tree_engine")
    for key in ("mutates_working_tree", "mutates_index", "provider_side_merge_button_used", "phase1_tide_batching_claimed"):
        if readiness.get(key) is not False:
            failures.append(f"merge_tree_readiness_forbidden_true_or_missing:{key}")

    lane_policy = registry.get("lane_ownership_policy") if isinstance(registry.get("lane_ownership_policy"), dict) else {}
    for key in ("one_lane_one_path", "overlap_review_required", "generated_artifact_source_pairing_required", "owner_ack_required_for_overlap"):
        if lane_policy.get(key) is not True:
            failures.append(f"lane_ownership_policy_missing_true:{key}")

    automated_chain = "\n".join(str(item) for item in registry.get("automated_chain", []))
    for token in REQUIRED_AUTOMATED_CHAIN_TOKENS:
        if token not in automated_chain:
            failures.append(f"missing_automated_chain_token:{token}")

    artifacts = object_list(registry.get("registered_artifacts"))
    minimum = registry.get("minimum_registered_artifact_count")
    if isinstance(minimum, int) and len(artifacts) < minimum:
        failures.append("registered_artifact_count_below_minimum")
    if not artifacts:
        failures.append("missing_registered_artifacts")

    ids = [artifact.get("id") for artifact in artifacts if isinstance(artifact.get("id"), str)]
    for artifact_id, count in Counter(ids).items():
        if count > 1:
            failures.append(f"duplicate_artifact_id:{artifact_id}")

    artifact_by_id: dict[str, dict[str, Any]] = {}
    for artifact in artifacts:
        artifact_id = str(artifact.get("id") or "<missing-id>")
        artifact_by_id[artifact_id] = artifact
        for field in REQUIRED_ARTIFACT_FIELDS:
            if field not in artifact:
                failures.append(f"{artifact_id}:missing_required_artifact_field:{field}")
        output_path = artifact.get("output_path")
        if not isinstance(output_path, str) or not output_path:
            failures.append(f"{artifact_id}:artifact_output_path_missing_or_invalid")
        elif not (root / output_path).is_file():
            failures.append(f"{artifact_id}:artifact_output_path_missing:{output_path}")
        else:
            marker = artifact.get("generated_marker")
            if isinstance(marker, str) and marker and marker not in read(root / output_path):
                failures.append(f"{artifact_id}:generated_marker_missing:{output_path}")

        source_paths = string_list(artifact.get("source_paths"))
        if not source_paths:
            failures.append(f"{artifact_id}:artifact_source_paths_missing")
        for source in source_paths:
            if not (root / source).exists():
                failures.append(f"{artifact_id}:artifact_source_path_missing_or_invalid:{source}")
        regen = artifact.get("regeneration_command")
        if not isinstance(regen, str) or not regen or not (root / regen).is_file():
            failures.append(f"{artifact_id}:regeneration_command_missing_or_invalid")
        drift_gate = artifact.get("drift_gate")
        if drift_gate != "//:phase0-merge-conflict-foundation-check":
            failures.append(f"{artifact_id}:missing_phase0_drift_gate")
        path_claims = artifact.get("path_claims") if isinstance(artifact.get("path_claims"), dict) else {}
        if path_claims.get("overlap_review_required") is not True:
            failures.append(f"{artifact_id}:path_claim_missing_overlap_review_required")
        if path_claims.get("phase1_tide_batching_claimed") is not False:
            failures.append(f"{artifact_id}:path_claim_phase1_tide_batching_overclaim")

    fixture_set = registry.get("fixture_set") if isinstance(registry.get("fixture_set"), dict) else {}
    fixtures = string_list(fixture_set.get("required_fixture_paths"))
    if len(fixtures) < 5:
        failures.append("fixture_set_missing_required_fixture_paths")
    for fixture in fixtures:
        if not (root / fixture).is_file():
            failures.append(f"fixture_path_missing:{fixture}")

    return failures, artifact_by_id


def approved_overlap_paths(fixture: dict[str, Any]) -> set[str]:
    approved: set[str] = set()
    for item in object_list(fixture.get("approved_overlap_reviews")):
        path = item.get("path")
        review_id = item.get("review_id")
        owner_ack = item.get("owner_ack")
        if isinstance(path, str) and isinstance(review_id, str) and owner_ack is True:
            approved.add(path)
    return approved


def validate_fixture(fixture: dict[str, Any], artifact_by_id: dict[str, dict[str, Any]]) -> dict[str, Any]:
    fixture_id = fixture.get("fixture_id") if isinstance(fixture.get("fixture_id"), str) else "<missing-fixture-id>"
    expected_verdict = fixture.get("expected_verdict")
    if expected_verdict not in {"GREEN", "RED"}:
        expected_verdict = "RED"
    expected_violations = set(string_list(fixture.get("expected_violations")))
    observed: list[str] = []

    merge_tree = fixture.get("merge_tree_simulation") if isinstance(fixture.get("merge_tree_simulation"), dict) else {}
    if merge_tree.get("engine") != "git merge-tree --write-tree":
        observed.append("missing_git_merge_tree_write_tree_engine")
    if merge_tree.get("result") != "clean":
        observed.append("merge_tree_conflict")
    for key in ("mutates_working_tree", "mutates_index", "provider_side_merge_button_used"):
        if merge_tree.get(key) is not False:
            observed.append(f"merge_tree_simulation_forbidden_true_or_missing:{key}")

    lanes = object_list(fixture.get("lanes"))
    path_to_lanes: dict[str, list[str]] = defaultdict(list)
    for lane in lanes:
        lane_id = lane.get("lane_id") if isinstance(lane.get("lane_id"), str) else "<missing-lane-id>"
        if not isinstance(lane.get("owner_team"), str) or not lane.get("owner_team"):
            observed.append("one_lane_one_path_violation")
        for path in string_list(lane.get("owned_paths")):
            path_to_lanes[path].append(lane_id)
    approved = approved_overlap_paths(fixture)
    for path, owners in path_to_lanes.items():
        if len(owners) > 1 and path not in approved:
            observed.append("path_overlap_without_review")

    for change in object_list(fixture.get("generated_artifact_changes")):
        artifact_id = change.get("artifact_id")
        path = change.get("path")
        if not isinstance(artifact_id, str) or artifact_id not in artifact_by_id or change.get("registry_entry_present") is False:
            observed.append("generated_artifact_missing_registry")
            continue
        artifact = artifact_by_id[artifact_id]
        if path != artifact.get("output_path"):
            observed.append("generated_artifact_missing_registry")
        sources = set(string_list(change.get("source_paths")))
        required_sources = set(string_list(artifact.get("source_paths")))
        if not sources or not required_sources.issubset(sources):
            observed.append("generated_artifact_stale_output")
        if change.get("regeneration_command") != artifact.get("regeneration_command"):
            observed.append("generated_artifact_stale_output")

    boundary = fixture.get("claim_boundary") if isinstance(fixture.get("claim_boundary"), dict) else {}
    for claim in ("status_mutation_performed", "protected_branch_authority_proven", "live_required_context_execution_proven", "p0_0_green", "phase0_complete", "production_ready", "hyperscaler_grade"):
        if boundary.get(claim) is not False:
            observed.append(f"forbidden_true_or_missing_claim_{claim}")
    if boundary.get("phase1_tide_batching_claimed") is not False:
        observed.append("phase1_tide_batched_projection_overclaim")

    observed_set = set(observed)
    expectation_failures: list[str] = []
    if expected_verdict == "GREEN" and observed_set:
        expectation_failures.append("GREEN merge-conflict fixture produced violations")
    if expected_verdict == "RED" and not observed_set:
        expectation_failures.append("RED merge-conflict fixture must produce violations")
    missing_expected = sorted(expected_violations - observed_set)
    for item in missing_expected:
        expectation_failures.append(f"expected_violation_missing:{item}")

    return {
        "fixture_id": fixture_id,
        "expected_verdict": expected_verdict,
        "expected_violations": sorted(expected_violations),
        "observed_violations": sorted(observed_set),
        "expectation_failures": expectation_failures,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--registry", default=str(DEFAULT_REGISTRY))
    parser.add_argument("--fixture", action="append", default=[])
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    registry_path = Path(args.registry)
    if not registry_path.is_absolute():
        registry_path = root / registry_path

    failures: list[str] = []
    if not registry_path.is_file():
        registry: dict[str, Any] = {}
        artifact_by_id: dict[str, dict[str, Any]] = {}
        failures.append("missing_generated_artifact_registry")
    else:
        registry = load_json(registry_path)
        registry_failures, artifact_by_id = validate_registry(root, registry)
        failures.extend(registry_failures)

    fixture_paths = args.fixture or string_list((registry.get("fixture_set") if isinstance(registry.get("fixture_set"), dict) else {}).get("required_fixture_paths"))
    fixture_results: list[dict[str, Any]] = []
    for fixture_value in fixture_paths:
        fixture_path = Path(fixture_value)
        if not fixture_path.is_absolute():
            fixture_path = root / fixture_path
        if not fixture_path.is_file():
            failures.append(f"fixture_path_missing:{display_path(fixture_path, root)}")
            continue
        fixture = load_json(fixture_path)
        result = validate_fixture(fixture, artifact_by_id)
        result["path"] = display_path(fixture_path, root)
        fixture_results.append(result)
        failures.extend(result["expectation_failures"])

    registry_artifacts = object_list(registry.get("registered_artifacts")) if registry else []
    taxonomy_ids = {
        item.get("id") for item in object_list(registry.get("conflict_taxonomy")) if isinstance(item.get("id"), str)
    } if registry else set()
    expected_green = sum(1 for item in fixture_results if item.get("expected_verdict") == "GREEN")
    expected_red = sum(1 for item in fixture_results if item.get("expected_verdict") == "RED")
    unique_failures = sorted(set(failures))
    result = {
        "authority_boundary": "AC-0.15 local/static generated-artifact registry and merge-tree readiness evidence only; no status mutation, live required-context authority, Phase-1 Tide batching, full-repo generated-artifact coverage, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven",
        "generated_artifact_registry_published": bool(registry) and not unique_failures,
        "merge_tree_fixture_contract_measured": bool(registry) and not unique_failures,
        "status_mutation_performed": False,
        "protected_branch_authority_proven": False,
        "live_required_context_execution_proven": False,
        "phase1_tide_batching_claimed": False,
        "full_repo_generated_artifact_coverage_proven": False,
        "p0_0_green": False,
        "phase0_complete": False,
        "production_ready": False,
        "hyperscaler_grade": False,
        "registry": display_path(registry_path, root) if registry_path.exists() else str(registry_path),
        "registered_artifact_count": len(registry_artifacts),
        "taxonomy_count": len(taxonomy_ids),
        "fixture_count": len(fixture_results),
        "expected_green_fixture_count": expected_green,
        "expected_red_fixture_count": expected_red,
        "fixtures": fixture_results,
        "verdict": "PASS" if not unique_failures else "FAIL",
        "failures": unique_failures,
    }
    rendered = json.dumps(result, sort_keys=True)
    if args.json or result["verdict"] == "PASS":
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if result["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
