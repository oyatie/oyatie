#!/usr/bin/env python3
"""Fail-closed RED checker for tasks workplace fixture and contract seams.

This checker intentionally remains RED until a downstream Build card creates
source-backed replay artifacts under specs/fixtures/tasks/replay/. It validates
that the RED fixture manifest is grounded in specs/microservices/tasks.json and
in the parent Plan/Spec slice manifest, then fails on missing replay artifacts.
"""
from __future__ import annotations

import argparse
import contextlib
import copy
import io
import json
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "tasks" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "tasks" / "replay"
TASKS_SOURCE_LOCK = REPO_ROOT / "specs" / "microservices" / "tasks.json"
PLAN_SLICES = REPO_ROOT / "plan" / "tasks" / "tasks-workplace-backlog-slices.json"
TASKS_INVENTORY = REPO_ROOT / "oya" / "tasks" / "manifest.json"
OPENAPI_CONTRACT = REPO_ROOT / "oya" / "tasks" / "contracts" / "openapi" / "tasks.yaml"
ASYNCAPI_CONTRACT = REPO_ROOT / "oya" / "tasks" / "contracts" / "asyncapi" / "tasks-events.yaml"
PROTO_CONTRACT = REPO_ROOT / "oya" / "tasks" / "contracts" / "proto" / "tasks.proto"

EXPECTED_FIXTURE_IDS = [
    "tasks_source_map_authority_fixture",
    "tasks_task_store_lifecycle_contract_fixture",
    "tasks_project_view_accessibility_fixture",
    "tasks_dependency_bulk_refusal_fixture",
    "tasks_recurrence_idempotency_timezone_fixture",
    "tasks_search_auth_projection_fixture",
    "tasks_importer_webhook_idempotency_fixture",
    "tasks_ai_assist_fairness_refusal_fixture",
]
EXPECTED_PLAN_SLICE_IDS = [
    "TASKS-PS-00",
    "TASKS-PS-01",
    "TASKS-PS-02",
    "TASKS-PS-03",
    "TASKS-PS-04",
    "TASKS-PS-05",
    "TASKS-PS-06",
    "TASKS-PS-07",
]
REQUIRED_BOUNDED_CONTEXTS = {
    "task-store",
    "project-list",
    "dependency-graph",
    "recurrence",
    "search-index",
    "view-engine",
    "importers",
}
REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/tasks.json",
    "specs/microservices/tasks.json#red_fixture_strategy",
    "plan/tasks/tasks-workplace-source-backed-backlog.md",
    "plan/tasks/tasks-workplace-backlog-slices.json",
    "oya/tasks/manifest.json#inventory_context_not_live_authority",
    "kanban:t_516944f9#source-lock",
    "kanban:t_2e92e2f3#plan-spec",
    "kanban:t_91fc5a1f#red",
}
REQUIRED_CONTRACT_SURFACES = {
    "oya/tasks/contracts/openapi/tasks.yaml",
    "oya/tasks/contracts/asyncapi/tasks-events.yaml",
    "oya/tasks/contracts/proto/tasks.proto",
}
EXPECTED_RED_STATUS = "RED_UNTIL_TASKS_REPLAY_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"
BUILD_TASK = "t_4221a299"
REPLAY_PASS_STATUS = "PASSED_SELECTED_SLICE_REPLAY"
REQUIRED_SELECTED_SLICE_IDS = {"TASKS-PS-00", "TASKS-PS-01"}


def fail(message: str) -> NoReturn:
    print(f"tasks RED fixture contract check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def rel(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def text(value: object) -> str:
    if isinstance(value, dict):
        return " ".join(text(item) for item in value.values())
    if isinstance(value, list):
        return " ".join(text(item) for item in value)
    return str(value).lower()


def load_json(path: Path, label: str) -> dict[str, Any]:
    try:
        candidate = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {label}: {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON {label} {rel(path)}: {exc}")
    require(isinstance(candidate, dict), f"{label} must be a JSON object")
    return candidate


def read_text(path: Path, label: str) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing {label}: {rel(path)}")


def require_contains_all(haystack: object, needles: set[str] | list[str], label: str) -> None:
    values = set(str(item) for item in haystack) if isinstance(haystack, list) else set()
    missing = sorted(set(needles) - values)
    require(not missing, f"{label} missing {missing}")


def require_terms(haystack: object, terms: list[str], label: str) -> None:
    lowered = text(haystack)
    missing = [term for term in terms if term.lower() not in lowered]
    require(not missing, f"{label} missing terms {missing}")


def require_fixture_sources(fixture: dict[str, Any], required_sources: list[str], label: str) -> None:
    values = set(str(item) for item in fixture.get("source_authority_refs", []))
    missing = [source for source in required_sources if not any(source in value for value in values)]
    require(not missing, f"{label} source_authority_refs missing {missing}")


def fixture_by_id(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    fixtures = manifest.get("fixtures")
    require(isinstance(fixtures, list), "manifest.fixtures must be a list")
    by_id: dict[str, dict[str, Any]] = {}
    for fixture in fixtures:
        require(isinstance(fixture, dict), "each fixture must be an object")
        fixture_id = str(fixture.get("fixture_id", ""))
        require(fixture_id, "fixture missing fixture_id")
        require(fixture_id not in by_id, f"duplicate fixture_id {fixture_id}")
        by_id[fixture_id] = fixture
    return by_id


def validate_future_replay_artifacts(fixture: dict[str, Any]) -> None:
    artifacts = fixture.get("future_replay_artifacts")
    fixture_id = fixture.get("fixture_id")
    require(isinstance(artifacts, list) and artifacts, f"{fixture_id} must name future_replay_artifacts")
    for raw in artifacts:
        artifact = str(raw)
        require(
            artifact.startswith("specs/fixtures/tasks/replay/"),
            f"future replay artifact must stay under specs/fixtures/tasks/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_tasks_source_lock(source_lock: dict[str, Any]) -> None:
    meta = source_lock.get("_meta", {})
    require(isinstance(meta, dict), "tasks source lock _meta must be an object")
    require(meta.get("spec_id") == "PRD-TASKS", "tasks source lock spec_id must be PRD-TASKS")
    require(meta.get("status") == "Accepted", "tasks source lock status must remain Accepted")
    require(meta.get("authority_lock_task") == "t_516944f9", "tasks source lock must bind to t_516944f9")

    authority = source_lock.get("authority_resolution", {})
    require(isinstance(authority, dict), "authority_resolution must be an object")
    require(authority.get("current_authority") == "/specs/microservices/tasks.json", "current authority drifted")
    require(authority.get("canonical_service_root") == "/oya/tasks/", "canonical service root must stay /oya/tasks/")
    require(authority.get("source_lock_disposition") == "resolved_by_this_artifact", "source lock disposition drifted")
    require_terms(authority, ["microservices/tasks/", "legacy", "no handler", "readiness claim"], "authority resolution")

    identity = source_lock.get("identity", {})
    require(isinstance(identity, dict), "identity must be an object")
    require(identity.get("product_id") == "tasks", "identity.product_id must be tasks")
    require(identity.get("user_facing_surface") is True, "tasks must remain user-facing for UI/a11y gating")
    contexts = identity.get("bounded_contexts_from_inventory")
    require_contains_all(contexts, REQUIRED_BOUNDED_CONTEXTS, "identity bounded_contexts_from_inventory")

    strategy = source_lock.get("red_fixture_strategy", {})
    require(isinstance(strategy, dict), "red_fixture_strategy must be present")
    require_terms(
        strategy,
        [
            "RED-TASKS-SOURCE-MAP-01",
            "RED-TASKS-CONTRACT-01",
            "RED-TASKS-CYCLE-01",
            "RED-TASKS-RECURRENCE-01",
            "RED-TASKS-SEARCH-AUTH-01",
            "RED-TASKS-AI-AUTO-ASSIGN-01",
            "RED-TASKS-A11Y-01",
        ],
        "red_fixture_strategy",
    )
    api = source_lock.get("api_contract_replay_expectations", {})
    require(isinstance(api, dict), "api_contract_replay_expectations must be present")
    require_contains_all(api.get("required_surfaces"), REQUIRED_CONTRACT_SURFACES, "api required surfaces")
    require_terms(api.get("must_cover", []), ["bulk update", "webhook", "AI-assist", "dependency cycles"], "api replay must_cover")

    a11y = source_lock.get("browser_user_story_accessibility_expectations", {})
    require(isinstance(a11y, dict), "browser_user_story_accessibility_expectations must be present")
    require_terms(a11y, ["WCAG 2.2 AA", "keyboard-only", "screen-reader", "creates a task", "board"], "browser/a11y expectations")


def validate_plan_slices(plan: dict[str, Any]) -> None:
    require(plan.get("task_id") == "t_2e92e2f3", "plan slice manifest must bind to t_2e92e2f3")
    require(plan.get("source_authority") == "specs/microservices/tasks.json", "plan source authority drifted")
    require_terms(plan.get("claim_boundary", ""), ["no runtime", "generated json", "readiness"], "plan claim boundary")
    lifecycle = plan.get("lifecycle")
    require_terms(lifecycle, ["RED fixture/contract", "Build", "Review/fix"], "plan lifecycle")
    slices = plan.get("slices")
    require(isinstance(slices, list), "plan.slices must be a list")
    actual_ids = [str(item.get("id")) for item in slices if isinstance(item, dict)]
    require(actual_ids == EXPECTED_PLAN_SLICE_IDS, f"plan slice ids/order must be {EXPECTED_PLAN_SLICE_IDS}; got {actual_ids}")
    by_id = {str(item.get("id")): item for item in slices if isinstance(item, dict)}
    for slice_id in EXPECTED_PLAN_SLICE_IDS:
        slice_data = by_id[slice_id]
        allowed = text(slice_data.get("allowed_prefixes", []))
        require("plan/tasks/" in allowed or slice_id == "TASKS-PS-00", f"{slice_id} must preserve plan/tasks allowed prefix")
        for required_key in ["fixture_plan", "api_contract_replay", "browser_user_story_accessibility"]:
            require(required_key in slice_data, f"{slice_id} missing {required_key}")
    require_terms(by_id["TASKS-PS-00"], ["source-map", "stale", "microservices/tasks"], "TASKS-PS-00")
    require_terms(by_id["TASKS-PS-03"], ["self-edge", "multi-hop cycle", "atomic", "bulk"], "TASKS-PS-03")
    require_terms(by_id["TASKS-PS-04"], ["timezone", "idempotency", "backfill", "unbounded"], "TASKS-PS-04")
    require_terms(by_id["TASKS-PS-05"], ["cross-tenant", "legal-hold", "stale-index"], "TASKS-PS-05")
    require_terms(by_id["TASKS-PS-06"], ["duplicate", "webhook", "dead-letter", "replay"], "TASKS-PS-06")
    require_terms(by_id["TASKS-PS-07"], ["fairness", "protected-class", "human override", "employment"], "TASKS-PS-07")


def validate_tasks_inventory(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "tasks", "tasks inventory microservice must be tasks")
    bounded = inventory.get("bounded_contexts")
    if not isinstance(bounded, list):
        fail("tasks inventory bounded_contexts must be a list")
    actual = {str(item.get("name")) for item in bounded if isinstance(item, dict)}
    require(REQUIRED_BOUNDED_CONTEXTS <= actual, f"tasks inventory missing bounded contexts {sorted(REQUIRED_BOUNDED_CONTEXTS - actual)}")
    require_terms(inventory.get("contracts", {}), ["microservices/tasks/contracts/openapi/tasks.yaml", "microservices/tasks/contracts/asyncapi/tasks-events.yaml", "microservices/tasks/contracts/proto/tasks.proto"], "inventory legacy contracts")
    require_terms(inventory.get("capabilities", []), ["T0-suggest", "T1-assist", "T2-auto"], "inventory capabilities")


def validate_contract_texts() -> None:
    openapi = read_text(OPENAPI_CONTRACT, "tasks OpenAPI contract")
    asyncapi = read_text(ASYNCAPI_CONTRACT, "tasks AsyncAPI contract")
    proto = read_text(PROTO_CONTRACT, "tasks proto contract")
    require_terms(openapi, ["Create task", "List tasks", "DependencyCycleRefusal", "bulk", "search", "importers"], "OpenAPI contract")
    require_terms(asyncapi, ["tasks.task.lifecycle.v1", "tasks.task.dependency.v1", "tasks.task.recurrence.v1", "tasks.task.bulk.v1", "tasks.task.io.v1"], "AsyncAPI contract")
    require_terms(proto, ["service TaskStore", "service ProjectList", "DependencyKind", "recurrence_rule", "ImportSource"], "proto contract")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "TASKS-WORKPLACE-RED-FIXTURE-CONTRACT-PLAN-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_91fc5a1f", "manifest must bind to kanban task t_91fc5a1f")
    require(manifest.get("parent_plan_spec_task") == "t_2e92e2f3", "manifest must bind to parent Plan/Spec task t_2e92e2f3")
    require(manifest.get("source_lock_task") == "t_516944f9", "manifest must bind to source lock task t_516944f9")
    require(manifest.get("follow_on_build_task") == "t_4221a299", "manifest must bind to follow-on Build task t_4221a299")
    require_terms(manifest.get("claim_boundary", ""), ["no runtime", "generated JSON hand edits", "Release Please", "product-readiness"], "claim_boundary")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("required_plan_slice_ids"), EXPECTED_PLAN_SLICE_IDS, "required_plan_slice_ids")
    require_contains_all(manifest.get("required_bounded_contexts"), REQUIRED_BOUNDED_CONTEXTS, "required_bounded_contexts")
    require_contains_all(manifest.get("required_contract_surfaces"), REQUIRED_CONTRACT_SURFACES, "required_contract_surfaces")
    require(manifest.get("future_replay_root") == "specs/fixtures/tasks/replay/", "future_replay_root must be source-locked")
    require(manifest.get("default_future_build_home") == "oya/tasks/crates/oya-tasks-domain", "default future Build home must match first implementation slice")
    require_terms(manifest.get("global_non_claims", []), ["green CI alone", "legacy microservices/tasks", "generated JSON", "readiness"], "global_non_claims")

    handoff = manifest.get("build_handoff", {})
    require(isinstance(handoff, dict), "build_handoff must be an object")
    require(handoff.get("red_command") == "python3 scripts/tests/tasks_red_fixture_contract_check.py", "red command drifted")
    require("expected_red_failure" in handoff, "build_handoff missing expected_red_failure")
    require_terms(handoff, ["t_4221a299", "selected slice"], "build_handoff")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match tasks RED plan; got {actual_ids}")
    for expected_slice, fixture_id in zip(EXPECTED_PLAN_SLICE_IDS, EXPECTED_FIXTURE_IDS, strict=True):
        fixture = by_id[fixture_id]
        require(fixture.get("slice_id") == expected_slice, f"{fixture_id} must bind to {expected_slice}")
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_replay_artifacts(fixture)

    require_fixture_sources(by_id["tasks_source_map_authority_fixture"], ["tasks.json#authority_resolution", "TASKS-PS-00", "legacy_manifest_reference_disposition"], "source-map fixture")
    require_terms(by_id["tasks_source_map_authority_fixture"].get("must_reject", []), ["missing specs/microservices/tasks.json", "microservices/tasks", "readiness"], "source-map fixture rejections")
    require_terms(by_id["tasks_task_store_lifecycle_contract_fixture"], ["create/read/update/archive/delete", "idempotency", "tenant mismatch", "legal-hold", "RLS/tenant-DEK", "audit"], "task-store fixture")
    require_terms(by_id["tasks_project_view_accessibility_fixture"], ["project/list/board", "keyboard", "screen-reader", "WCAG 2.2 AA", "green CI alone"], "project-view fixture")
    require_terms(by_id["tasks_dependency_bulk_refusal_fixture"], ["self-edge", "multi-hop", "batch patch", "atomicity", "audit-chain"], "dependency/bulk fixture")
    require_terms(by_id["tasks_recurrence_idempotency_timezone_fixture"], ["RRULE", "timezone", "idempotency", "backfill", "unbounded"], "recurrence fixture")
    require_terms(by_id["tasks_search_auth_projection_fixture"], ["tenant/RBAC/legal-hold", "stale-index", "cross-tenant", "redaction"], "search fixture")
    require_terms(by_id["tasks_importer_webhook_idempotency_fixture"], ["duplicate", "correlation", "idempotent", "webhook signing", "dead-letter"], "importer/webhook fixture")
    require_terms(by_id["tasks_ai_assist_fairness_refusal_fixture"], ["T2 auto-assign", "fairness", "protected-class", "human override", "employment"], "AI-assist fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    return by_id


def validate_replay_artifacts(by_id: dict[str, dict[str, Any]], replay_root: Path) -> None:
    missing_selected: list[str] = []
    deferred_missing: list[str] = []
    validated_selected: list[str] = []
    for fixture in by_id.values():
        slice_id = str(fixture.get("slice_id"))
        for raw in fixture["future_replay_artifacts"]:
            rel_path = Path(str(raw))
            expected = REPO_ROOT / rel_path
            if expected.exists():
                validate_replay_artifact(fixture, rel_path)
                if slice_id in REQUIRED_SELECTED_SLICE_IDS:
                    validated_selected.append(str(rel_path))
                continue
            if slice_id in REQUIRED_SELECTED_SLICE_IDS:
                missing_selected.append(str(rel_path))
            else:
                deferred_missing.append(str(rel_path))
    if missing_selected:
        preview = ", ".join(missing_selected[:8])
        suffix = "" if len(missing_selected) <= 8 else f" ... (+{len(missing_selected) - 8} more)"
        fail(f"RED: selected tasks replay artifacts are missing under {rel(replay_root)}: {preview}{suffix}")
    require(validated_selected, "selected tasks replay artifacts must exist before GREEN")
    selected = ", ".join(sorted(REQUIRED_SELECTED_SLICE_IDS))
    print(
        f"tasks selected replay artifacts passed for {selected}; "
        f"deferred RED artifacts remain: {len(deferred_missing)}"
    )


def validate_replay_artifact(fixture: dict[str, Any], rel_path: Path) -> None:
    artifact = load_json(REPO_ROOT / rel_path, f"replay artifact {rel_path}")
    fixture_id = str(fixture.get("fixture_id"))
    slice_id = str(fixture.get("slice_id"))
    label = f"replay artifact {rel_path}"

    require(artifact.get("parent_fixture_id") == fixture_id, f"{label} must bind parent_fixture_id {fixture_id}")
    require(artifact.get("plan_slice_id") == slice_id, f"{label} must bind plan_slice_id {slice_id}")
    require(artifact.get("replay_artifact_path") == str(rel_path), f"{label} replay_artifact_path drifted")
    require(artifact.get("red_gate_task") == "t_91fc5a1f", f"{label} must bind RED gate t_91fc5a1f")
    require(artifact.get("build_task") == BUILD_TASK, f"{label} must bind Build task {BUILD_TASK}")
    require(artifact.get("status") == REPLAY_PASS_STATUS, f"{label} status must be {REPLAY_PASS_STATUS}")
    require(artifact.get("red_status_closed") == EXPECTED_RED_STATUS, f"{label} must close {EXPECTED_RED_STATUS}")
    require(artifact.get("source_map_ref") == "specs/microservices/tasks.json", f"{label} must cite tasks source map")

    require_replay_source_paths(artifact, rel_path, label)
    require_terms(artifact.get("assertions_covered", []), list(fixture.get("must_assert", [])), f"{label} assertions_covered")
    require_terms(artifact.get("rejections_covered", []), list(fixture.get("must_reject", [])), f"{label} rejections_covered")

    lower_artifact = text(artifact)
    for forbidden in ["passed_after_future_runtime_evidence", "production-ready"]:
        require(forbidden not in lower_artifact, f"{label} must not contain forbidden claim {forbidden!r}")
    for required in ["no runtime readiness", "no generated json hand edits", "no production readiness claim", "deferred fixtures remain red"]:
        require(required in lower_artifact, f"{label} non_claims missing {required!r}")
    if slice_id == "TASKS-PS-00":
        require_terms(artifact, ["source-map", "microservices/tasks/**", "inventory/provenance", "ADR-0131", "ADR-0512"], label)
    if slice_id == "TASKS-PS-01":
        require_terms(artifact, ["OpenAPI", "proto", "create/read/update/archive/delete", "tenant mismatch", "legal-hold", "audit"], label)
        ui_evidence = artifact.get("browser_user_story_accessibility_evidence", {})
        require(isinstance(ui_evidence, dict), f"{label} must carry browser_user_story_accessibility_evidence")
        require(ui_evidence.get("runtime_ui_changed") is False, f"{label} must record UI N/A for API/contract replay only")


def require_replay_source_paths(artifact: dict[str, Any], rel_path: Path, label: str) -> None:
    raw_source_paths = artifact.get("source_paths")
    require(isinstance(raw_source_paths, list) and raw_source_paths, f"{label} must name source_paths")
    source_paths = list(raw_source_paths) if isinstance(raw_source_paths, list) else []
    source_text = text(source_paths)
    require("specs/microservices/tasks.json" in source_text, f"{label} must source from specs/microservices/tasks.json")
    family = rel_path.parts[-2]
    if family == "source-map":
        require("oya/tasks/manifest.json" in source_text, f"{label} must source inventory/provenance boundary")
        require("plan/tasks/tasks-workplace-backlog-slices.json" in source_text, f"{label} must source Plan/Spec slice manifest")
    if family == "task-store":
        require("oya/tasks/contracts/openapi/tasks.yaml" in source_text, f"{label} must source OpenAPI task-store contract")
        require("oya/tasks/contracts/proto/tasks.proto" in source_text, f"{label} must source proto task-store contract")
    for raw in source_paths:
        source = str(raw).split("#", 1)[0]
        if source.startswith("kanban:"):
            continue
        require(not source.endswith(GENERATED_SUFFIX), f"{label} source path must not be a generated face: {source}")
        require(".." not in Path(source).parts, f"{label} source path must not traverse directories: {source}")
        require((REPO_ROOT / source).exists(), f"{label} source path missing: {source}")


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    fixture_terms = {
        "tasks_source_map_authority_fixture": {
            "must_assert": ["every downstream tasks Plan/Spec, RED, Build, Review/fix, Merge/Rollout/E2E, and Learning card cites specs/microservices/tasks.json", "oya/tasks/manifest.json remains inventory/provenance and is not readiness evidence", "ADR-0131 and ADR-0512 keep canonical service paths under oya/tasks/**"],
            "must_reject": ["missing specs/microservices/tasks.json citation", "live destination claim from microservices/tasks/**", "runtime readiness claim from manifest, SLO, catalog, runbook, or contract inventory alone"],
            "sources": ["specs/microservices/tasks.json#authority_resolution", "plan/tasks/tasks-workplace-backlog-slices.json#TASKS-PS-00", "oya/tasks/manifest.json#legacy_manifest_reference_disposition"],
        },
        "tasks_task_store_lifecycle_contract_fixture": {
            "must_assert": ["create/read/update/archive/delete lifecycle replay", "idempotency key and optimistic-version conflict behavior", "tenant mismatch denial, Cedar deny-by-default, RLS/tenant-DEK expectation, legal-hold and retention refusal", "audit metadata for task create/update/archive/legal-hold"],
            "must_reject": ["malformed custom fields", "cross-tenant task read or write", "hard delete while legal hold is open", "task-store readiness without OpenAPI/proto replay evidence"],
            "sources": ["specs/microservices/tasks.json#api_contract_replay_expectations", "plan/tasks/tasks-workplace-backlog-slices.json#TASKS-PS-01", "oya/tasks/contracts/openapi/tasks.yaml", "oya/tasks/contracts/proto/tasks.proto"],
        },
        "tasks_project_view_accessibility_fixture": {
            "must_assert": ["project/list/board/sprint/milestone CRUD expectations", "board status transition and drag/drop conflict feedback", "keyboard drag/drop alternative, focus order, screen-reader announcements, reduced motion, high contrast, no spinner-only waits", "browser/user-story/WCAG 2.2 AA evidence or explicit UI N/A for API-only work"],
            "must_reject": ["UI completion with green CI alone", "board/list reads without authorization filtering", "spinner-only waiting state", "missing accessibility evidence for user-facing task list/board/project views"],
            "sources": ["specs/microservices/tasks.json#browser_user_story_accessibility_expectations", "plan/tasks/tasks-workplace-backlog-slices.json#TASKS-PS-02", "oya/tasks/contracts/openapi/tasks.yaml", "oya/tasks/contracts/asyncapi/tasks-events.yaml"],
        },
        "tasks_dependency_bulk_refusal_fixture": {
            "must_assert": ["self-edge refusal", "two-node and multi-hop dependency cycle refusal", "cycle-introducing batch patch refusal", "atomicity or explicit partial-failure report where allowed", "throttling/backpressure and audit-chain refusal metadata"],
            "must_reject": ["cycle-creating edge accepted", "bulk update partially corrupts dependency graph silently", "refusal without auditable reason", "dependency/bulk readiness without contract replay"],
            "sources": ["specs/microservices/tasks.json#R-TASKS-01", "plan/tasks/tasks-workplace-backlog-slices.json#TASKS-PS-03", "oya/tasks/contracts/openapi/tasks.yaml", "oya/tasks/contracts/asyncapi/tasks-events.yaml", "oya/tasks/contracts/proto/tasks.proto"],
        },
        "tasks_recurrence_idempotency_timezone_fixture": {
            "must_assert": ["bounded RRULE subset", "timezone transition correctness", "idempotency and duplicate prevention", "backfill limits, paused/cancelled series, legal-hold interaction", "unbounded expansion backpressure"],
            "must_reject": ["duplicate materialisation on replay", "unbounded recurrence expansion", "timezone-wrong generated task", "backfill beyond configured bounds without refusal"],
            "sources": ["specs/microservices/tasks.json#R-TASKS-04", "plan/tasks/tasks-workplace-backlog-slices.json#TASKS-PS-04", "oya/tasks/contracts/openapi/tasks.yaml", "oya/tasks/contracts/asyncapi/tasks-events.yaml", "oya/tasks/contracts/proto/tasks.proto"],
        },
        "tasks_search_auth_projection_fixture": {
            "must_assert": ["authorized search/filter/saved-view result replay", "tenant/RBAC/legal-hold filtering", "stale-index rebuild safety", "delete/update propagation", "confidential or regulated task content redaction"],
            "must_reject": ["cross-tenant search result", "legal-hold hidden task surfaced to unauthorized actor", "stale projection treated as fresh", "saved view bypassing authorization"],
            "sources": ["specs/microservices/tasks.json#R-TASKS-02", "plan/tasks/tasks-workplace-backlog-slices.json#TASKS-PS-05", "oya/tasks/contracts/openapi/tasks.yaml", "oya/tasks/contracts/asyncapi/tasks-events.yaml"],
        },
        "tasks_importer_webhook_idempotency_fixture": {
            "must_assert": ["importer mapping review", "duplicate detection and correlation identifiers", "idempotent writes", "webhook signing and authorization", "retry/dead-letter/replay and audit-chain event expectations"],
            "must_reject": ["external source treated as canonical source of truth", "unsigned or unauthorized webhook fanout", "duplicate importer payload creates duplicate task", "dead-letter replay without correlation id"],
            "sources": ["specs/microservices/tasks.json#api_contract_replay_expectations", "plan/tasks/tasks-workplace-backlog-slices.json#TASKS-PS-06", "oya/tasks/contracts/openapi/tasks.yaml", "oya/tasks/contracts/asyncapi/tasks-events.yaml", "oya/tasks/contracts/proto/tasks.proto"],
        },
        "tasks_ai_assist_fairness_refusal_fixture": {
            "must_assert": ["T0 suggestions and T1 categorisation/priority assistance are disclosed", "T2 auto-assign refused unless fairness evidence exists", "protected-class non-use", "human override and audit evidence", "EU AI Act and employment-context refusal controls", "accessible recommendation disclosure, confidence/refusal state, keyboard override, and screen-reader announcements if UI touched"],
            "must_reject": ["protected-class input used for assignment", "auto-assignment without fairness evidence", "hidden automation without disclosure", "employment-impacting recommendation without human override"],
            "sources": ["specs/microservices/tasks.json#R-TASKS-03", "plan/tasks/tasks-workplace-backlog-slices.json#TASKS-PS-07", "oya/tasks/contracts/openapi/tasks.yaml", "oya/tasks/contracts/asyncapi/tasks-events.yaml", "oya/tasks/manifest.json#capabilities"],
        },
    }
    replay_dirs = ["source-map", "task-store", "project-view", "dependency-bulk", "recurrence", "search-auth", "importer-webhook", "ai-assist"]
    for slice_id, fixture_id, replay_dir in zip(EXPECTED_PLAN_SLICE_IDS, EXPECTED_FIXTURE_IDS, replay_dirs, strict=True):
        terms = fixture_terms[fixture_id]
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "slice_id": slice_id,
                "fixture_kind": "self_test",
                "source_authority_refs": terms["sources"],
                "future_replay_artifacts": [f"specs/fixtures/tasks/replay/{replay_dir}/{fixture_id}.replay.json"],
                "must_assert": terms["must_assert"],
                "must_reject": terms["must_reject"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    return {
        "fixture_plan_id": "TASKS-WORKPLACE-RED-FIXTURE-CONTRACT-PLAN-001",
        "kanban_task": "t_91fc5a1f",
        "parent_plan_spec_task": "t_2e92e2f3",
        "source_lock_task": "t_516944f9",
        "follow_on_build_task": "t_4221a299",
        "service": "tasks",
        "claim_boundary": "RED fixture only; no runtime, no generated JSON hand edits, no Release Please, no product-readiness claims",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": ["oya/tasks/manifest.json", "legacy microservices/tasks/** references from oya/tasks/manifest.json"],
        "required_plan_slice_ids": EXPECTED_PLAN_SLICE_IDS[:],
        "required_bounded_contexts": sorted(REQUIRED_BOUNDED_CONTEXTS),
        "required_contract_surfaces": sorted(REQUIRED_CONTRACT_SURFACES),
        "future_replay_root": "specs/fixtures/tasks/replay/",
        "default_future_build_home": "oya/tasks/crates/oya-tasks-domain",
        "build_handoff": {
            "selected_first_slice": "TASKS-PS-00 source-map guard, then TASKS-PS-01 task-store lifecycle if the source-map guard is green",
            "red_command": "python3 scripts/tests/tasks_red_fixture_contract_check.py",
            "self_test_command": "python3 scripts/tests/tasks_red_fixture_contract_check.py --self-test",
            "json_command": "python3 -m json.tool specs/fixtures/tasks/red-fixtures.json",
            "expected_red_failure": "RED: future tasks replay artifacts are missing under specs/fixtures/tasks/replay/",
            "follow_on_instruction": "Build t_4221a299 implements selected slice only.",
        },
        "global_non_claims": ["green CI alone is insufficient", "legacy microservices/tasks/** paths are stale", "no generated JSON hand edits", "no readiness claim"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_tasks_source_lock(load_json(TASKS_SOURCE_LOCK, "tasks source lock"))
    validate_plan_slices(load_json(PLAN_SLICES, "tasks Plan/Spec slice manifest"))
    validate_tasks_inventory(load_json(TASKS_INVENTORY, "tasks inventory manifest"))
    validate_contract_texts()
    validate_manifest(live_manifest)
    valid = baseline_manifest()
    validate_manifest(valid)

    def expect_rejected(label: str, mutator: Callable[[dict[str, Any]], None]) -> None:
        candidate = copy.deepcopy(valid)
        mutator(candidate)
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                validate_manifest(candidate)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected("missing fixture", lambda data: data["fixtures"].pop())
    expect_rejected("missing source authority", lambda data: data.update({"source_authority_refs": ["specs/microservices/tasks.json"]}))
    expect_rejected("wrong slice order", lambda data: data["fixtures"].reverse())
    expect_rejected("generated replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/tasks/replay/bad.generated.json"]}))
    expect_rejected("source-map rejection gap", lambda data: data["fixtures"][0].update({"must_reject": ["missing specs/microservices/tasks.json citation"]}))
    expect_rejected("task-store legal-hold gap", lambda data: data["fixtures"][1].update({"must_assert": ["create/read/update/archive/delete lifecycle replay"]}))
    expect_rejected("a11y evidence gap", lambda data: data["fixtures"][2].update({"must_assert": ["project/list/board/sprint/milestone CRUD expectations"]}))
    expect_rejected("dependency bulk atomicity gap", lambda data: data["fixtures"][3].update({"must_assert": ["self-edge refusal"]}))
    expect_rejected("recurrence timezone gap", lambda data: data["fixtures"][4].update({"must_assert": ["bounded RRULE subset"], "must_reject": ["duplicate materialisation on replay"]}))
    expect_rejected("search cross-tenant gap", lambda data: data["fixtures"][5].update({"must_reject": ["stale projection treated as fresh"]}))
    expect_rejected("importer webhook signing gap", lambda data: data["fixtures"][6].update({"must_assert": ["importer mapping review"]}))
    expect_rejected("AI fairness gap", lambda data: data["fixtures"][7].update({"must_assert": ["T0 suggestions and T1 categorisation/priority assistance are disclosed"], "must_reject": ["hidden automation without disclosure"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("tasks RED fixture contract self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST), help="RED fixture manifest JSON path")
    parser.add_argument("--replay-root", default=str(DEFAULT_REPLAY_ROOT), help="future replay artifact root")
    parser.add_argument("--self-test", action="store_true", help="run fail-closed validator self-tests")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    manifest_path = Path(args.manifest)
    if not manifest_path.is_absolute():
        manifest_path = REPO_ROOT / manifest_path
    replay_root = Path(args.replay_root)
    if not replay_root.is_absolute():
        replay_root = REPO_ROOT / replay_root
    manifest = load_json(manifest_path, "RED fixture manifest")
    if args.self_test:
        run_self_tests(manifest)
        return
    validate_tasks_source_lock(load_json(TASKS_SOURCE_LOCK, "tasks source lock"))
    validate_plan_slices(load_json(PLAN_SLICES, "tasks Plan/Spec slice manifest"))
    validate_tasks_inventory(load_json(TASKS_INVENTORY, "tasks inventory manifest"))
    validate_contract_texts()
    by_id = validate_manifest(manifest)
    validate_replay_artifacts(by_id, replay_root)


if __name__ == "__main__":
    main()
