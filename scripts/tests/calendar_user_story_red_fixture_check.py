#!/usr/bin/env python3
"""Fail-closed checker for the Calendar CRUD/browser/accessibility RED story seed.

This guard validates that the user-story fixture manifest is grounded in the
Accepted PRD-CALENDAR authority and the API/contract-only PR #1145 replay
evidence, while staying RED until a future UI/story replay supplies browser,
keyboard, accessibility, i18n, responsive, loading, and error-state evidence.
"""
from __future__ import annotations

import argparse
import contextlib
import copy
import io
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "calendar-prd" / "user-story-red-fixtures.json"
DEFAULT_STORY_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "calendar-prd" / "user-story-replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "calendar.json"
PR_1145_MERGE_COMMIT = "a3c764dd2acc7d033b8871a0f549e1d3055a0be8"
EXPECTED_PLAN_ID = "CALENDAR-USER-STORY-RED-001"
EXPECTED_TASK = "t_fa36f517"
EXPECTED_PARENT = "t_310e7906"
EXPECTED_MANIFEST_STATUS = "expected-red-browser-user-story-accessibility-seed"
EXPECTED_RED_STATUS = "RED_UNTIL_BROWSER_USER_STORY_REPLAY_EXISTS"
GENERATED_SUFFIX = ".generated.json"

REQUIRED_AC_IDS = ["AC-01", "AC-02", "AC-03", "AC-04", "AC-05"]
REQUIRED_PERSONAS = [
    "employee_end_user",
    "personal_user",
    "manager_or_approver",
    "compliance_officer",
    "scheduler_integrator",
]
EXPECTED_FIXTURE_IDS = [
    "calendar_user_story_employee_work_event_crud_fixture",
    "calendar_user_story_personal_freebusy_privacy_fixture",
    "calendar_user_story_manager_action_card_workflow_fixture",
    "calendar_user_story_compliance_legal_hold_fixture",
    "calendar_user_story_accessibility_i18n_responsive_fixture",
    "calendar_user_story_pr1145_api_replay_link_fixture",
]
REQUIRED_SOURCE_REFS = {
    "specs/microservices/calendar.json#acceptance_criteria[AC-01..AC-05]",
    "specs/microservices/calendar.json#user_experience",
    "specs/microservices/calendar.json#target_users",
    "specs/fixtures/calendar-prd/red-fixtures.json#browser_user_story_accessibility_checklist",
    "github:pr#1145",
    "git:a3c764dd2acc7d033b8871a0f549e1d3055a0be8:specs/fixtures/calendar-prd/calendar_prd_replay_check.py",
    "git:a3c764dd2acc7d033b8871a0f549e1d3055a0be8:specs/fixtures/calendar-prd/replay/ux/calendar-browser-accessibility-evidence.fixture.json",
}
REQUIRED_PR_1145_REPLAY_ARTIFACTS = {
    "git:a3c764dd2acc7d033b8871a0f549e1d3055a0be8:specs/fixtures/calendar-prd/calendar_prd_replay_check.py",
    "git:a3c764dd2acc7d033b8871a0f549e1d3055a0be8:specs/fixtures/calendar-prd/replay/openapi/calendar-openapi-v1-replay.fixture.json",
    "git:a3c764dd2acc7d033b8871a0f549e1d3055a0be8:specs/fixtures/calendar-prd/replay/asyncapi/calendar-asyncapi-v1-replay.fixture.json",
    "git:a3c764dd2acc7d033b8871a0f549e1d3055a0be8:specs/fixtures/calendar-prd/replay/proto/calendar-proto-v1-replay.fixture.json",
    "git:a3c764dd2acc7d033b8871a0f549e1d3055a0be8:specs/fixtures/calendar-prd/replay/ux/calendar-browser-accessibility-evidence.fixture.json",
}


def fail(message: str) -> NoReturn:
    print(f"calendar user-story RED fixture check failed: {message}", file=sys.stderr)
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
        return " ".join(f"{key} {text(item)}" for key, item in value.items())
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


def require_contains_all(haystack: object, needles: set[str] | list[str], label: str) -> None:
    values = set(str(item) for item in haystack) if isinstance(haystack, list) else set()
    missing = sorted(set(needles) - values)
    require(not missing, f"{label} missing {missing}")


def require_terms(value: object, terms: list[str], label: str) -> None:
    haystack = text(value)
    missing = [term for term in terms if term.lower() not in haystack]
    require(not missing, f"{label} missing terms {missing}")


def repo_ref_exists(ref: str) -> bool:
    if ref.startswith("kanban:") or ref.startswith("github:pr#") or ref.startswith("https://github.com/"):
        return True
    if ref.startswith("git:"):
        _, commit, path = ref.split(":", 2)
        result = subprocess.run(
            ["git", "-C", str(REPO_ROOT), "cat-file", "-e", f"{commit}:{path}"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        return result.returncode == 0
    path_part = ref.split("#", 1)[0]
    return bool(path_part) and (REPO_ROOT / path_part).exists()


def validate_repo_refs(refs: list[str], label: str) -> None:
    missing = [ref for ref in refs if not repo_ref_exists(ref)]
    require(not missing, f"{label} has non-existent repo/git refs {missing}")


def fixture_by_id(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    fixtures = manifest.get("fixtures")
    require(isinstance(fixtures, list), "fixtures must be a list")
    by_id: dict[str, dict[str, Any]] = {}
    for fixture in fixtures:
        require(isinstance(fixture, dict), "each fixture must be an object")
        fixture_id = str(fixture.get("fixture_id", ""))
        require(fixture_id, "fixture missing fixture_id")
        require(fixture_id not in by_id, f"duplicate fixture_id {fixture_id}")
        by_id[fixture_id] = fixture
    return by_id


def validate_prd_source_lock(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "calendar PRD _meta must be an object")
    require(meta.get("spec_id") == "PRD-CALENDAR", "calendar PRD spec_id must be PRD-CALENDAR")
    require(meta.get("status") == "Accepted", "calendar PRD must remain Accepted")

    identity = prd.get("identity", {})
    require(isinstance(identity, dict), "calendar PRD identity must be an object")
    require(identity.get("product_id") == "calendar", "calendar PRD identity.product_id must be calendar")
    require(identity.get("user_facing_surface") is True, "calendar PRD must remain user-facing")
    require_contains_all(identity.get("context_model"), ["personal", "work"], "calendar context_model")

    target_users = prd.get("target_users")
    require(isinstance(target_users, list), "calendar PRD target_users must be a list")
    personas = [str(item.get("persona")) for item in target_users if isinstance(item, dict)]
    require(personas == REQUIRED_PERSONAS, f"calendar target personas/order drifted: {personas}")

    acs = prd.get("acceptance_criteria")
    require(isinstance(acs, list), "calendar PRD acceptance_criteria must be a list")
    ac_ids = [str(item.get("id")) for item in acs if isinstance(item, dict)]
    require(ac_ids == REQUIRED_AC_IDS, f"calendar PRD AC ids/order must be {REQUIRED_AC_IDS}; got {ac_ids}")

    ux = prd.get("user_experience", {})
    require(isinstance(ux, dict), "calendar PRD user_experience must be an object")
    require_terms(ux, ["WCAG 2.2 AA", "keyboard", "KR", "en-US", "loading", "policy_denied", "legal_hold"], "calendar user_experience")
    require(ux.get("keyboard_navigation_coverage_pct") == 100, "calendar keyboard navigation coverage must stay 100%")

    lower_prd = text(prd)
    for term in [
        "employee_end_user",
        "personal_user",
        "manager_or_approver",
        "compliance_officer",
        "scheduler_integrator",
        "details never cross pillar",
        "minimum necessary disclosure",
        "workflow_handoff_without_policy_basis",
        "silent personal availability mining",
    ]:
        require(term in lower_prd, f"calendar PRD must retain source term {term!r}")


def validate_future_story_artifacts(fixture: dict[str, Any]) -> None:
    artifacts = fixture.get("future_story_replay_artifacts")
    require(isinstance(artifacts, list) and artifacts, f"{fixture.get('fixture_id')} must name future_story_replay_artifacts")
    for raw in artifacts:
        artifact = str(raw)
        require(
            artifact.startswith("specs/fixtures/calendar-prd/user-story-replay/"),
            f"future story replay artifact must stay under specs/fixtures/calendar-prd/user-story-replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future story replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future story replay artifact must not traverse directories: {artifact}")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == EXPECTED_PLAN_ID, "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == EXPECTED_TASK, f"manifest must bind to {EXPECTED_TASK}")
    require(manifest.get("parent_learning_task") == EXPECTED_PARENT, f"manifest must bind to parent {EXPECTED_PARENT}")
    require(manifest.get("source_contract_pr") == "https://github.com/jason931225/oyatie/pull/1145", "manifest must link PR #1145")
    require(manifest.get("source_contract_merge_commit") == PR_1145_MERGE_COMMIT, "manifest must pin the PR #1145 merge commit")
    require(manifest.get("status") == EXPECTED_MANIFEST_STATUS, f"manifest status must remain {EXPECTED_MANIFEST_STATUS}")

    claim_boundary = str(manifest.get("claim_boundary", ""))
    require_terms(claim_boundary, ["RED", "fixture", "no runtime", "no UI readiness", "no production", "no GA"], "claim_boundary")
    require("customer-availability" in claim_boundary or "customer availability" in claim_boundary, "claim_boundary must deny customer availability claims")

    source_refs = manifest.get("source_authority_refs")
    require_contains_all(source_refs, REQUIRED_SOURCE_REFS, "source_authority_refs")
    validate_repo_refs(list(source_refs), "source_authority_refs")

    replay = manifest.get("existing_api_contract_replay_evidence")
    require(isinstance(replay, dict), "existing_api_contract_replay_evidence must be an object")
    require(replay.get("pr") == 1145, "existing API replay evidence must cite PR #1145")
    require(replay.get("browser_live_ui_accessibility_status") == "explicit_no_ui_NA", "PR #1145 browser/live UI status must remain explicit_no_ui_NA")
    require_contains_all(replay.get("artifact_refs"), REQUIRED_PR_1145_REPLAY_ARTIFACTS, "PR #1145 replay artifact_refs")
    validate_repo_refs(list(replay.get("artifact_refs", [])), "PR #1145 replay artifact_refs")
    require_terms(replay, ["OpenAPI", "AsyncAPI", "proto", "explicit no-UI N/A", "API/contract-only"], "existing API replay evidence")

    personas = manifest.get("required_personas")
    require(personas == REQUIRED_PERSONAS, f"required_personas must be {REQUIRED_PERSONAS}")
    require_contains_all(manifest.get("required_prd_acceptance_criteria"), REQUIRED_AC_IDS, "required_prd_acceptance_criteria")

    checklist = manifest.get("accessibility_i18n_responsive_checklist")
    require_terms(
        checklist,
        [
            "WCAG 2.2 AA",
            "keyboard",
            "focus order",
            "screen-reader",
            "responsive",
            "loading",
            "error",
            "KR",
            "en-US",
            "green CI alone",
        ],
        "accessibility/i18n/responsive checklist",
    )

    runtime_policy = manifest.get("runtime_evidence_policy")
    require_terms(runtime_policy, ["no UI exists", "explicit N/A", "fail-closed", "browser", "Playwright", "green CI alone"], "runtime_evidence_policy")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match the RED story plan; got {actual_ids}")
    for fixture_id, fixture in by_id.items():
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_story_artifacts(fixture)
        validate_repo_refs(list(fixture.get("source_authority_refs", [])), f"{fixture_id} source refs")

    require_terms(by_id["calendar_user_story_employee_work_event_crud_fixture"], ["AC-01", "AC-05", "create", "update", "audit", "retention", "legal hold", "visible"], "employee CRUD fixture")
    require_terms(by_id["calendar_user_story_employee_work_event_crud_fixture"].get("must_reject"), ["runtime handler", "production-ready UI", "green CI alone"], "employee CRUD fixture must_reject")
    require_terms(by_id["calendar_user_story_personal_freebusy_privacy_fixture"], ["AC-02", "free/busy", "403", "minimum necessary disclosure"], "personal free/busy fixture")
    require_terms(by_id["calendar_user_story_personal_freebusy_privacy_fixture"].get("must_reject"), ["personal title", "personal description", "personal location", "personal attendees", "silent personal availability mining"], "personal free/busy fixture must_reject")
    require_terms(by_id["calendar_user_story_manager_action_card_workflow_fixture"], ["AC-03", "action card", "policy_basis", "Workflow", "audit_ref"], "manager workflow fixture")
    require_terms(by_id["calendar_user_story_manager_action_card_workflow_fixture"].get("must_reject"), ["implicit Workflow inference", "without policy_basis", "without audit_ref"], "manager workflow fixture must_reject")
    require_terms(by_id["calendar_user_story_compliance_legal_hold_fixture"], ["AC-04", "legal hold", "attendee history", "action cards", "audit chain", "chain-of-custody"], "compliance legal-hold fixture")
    require_terms(by_id["calendar_user_story_compliance_legal_hold_fixture"].get("must_reject"), ["purge", "hard delete", "chain-of-custody gap"], "compliance legal-hold fixture must_reject")
    require_terms(by_id["calendar_user_story_accessibility_i18n_responsive_fixture"], ["WCAG 2.2 AA", "keyboard", "focus order", "screen-reader", "responsive", "loading", "error", "KR", "en-US"], "accessibility fixture")
    require_terms(by_id["calendar_user_story_accessibility_i18n_responsive_fixture"].get("must_reject"), ["green CI alone", "keyboard traversal missing", "screen-reader labels missing"], "accessibility fixture must_reject")
    require_terms(by_id["calendar_user_story_pr1145_api_replay_link_fixture"], ["PR #1145", "OpenAPI", "AsyncAPI", "proto", "explicit no-UI N/A", "API/contract-only"], "PR #1145 API replay fixture")
    require_terms(by_id["calendar_user_story_pr1145_api_replay_link_fixture"].get("must_reject"), ["browser evidence from API-only", "UI readiness", "customer availability"], "PR #1145 API replay fixture must_reject")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("production readiness" not in lower_manifest, "manifest must not claim production readiness")
    require("green ci alone is insufficient" in lower_manifest, "manifest must reject green-CI-alone readiness")
    return by_id


def validate_story_replay_artifacts(by_id: dict[str, dict[str, Any]], story_replay_root: Path) -> None:
    missing: list[str] = []
    for fixture in by_id.values():
        for raw in fixture["future_story_replay_artifacts"]:
            rel_path = Path(str(raw))
            expected = REPO_ROOT / rel_path
            if not expected.exists():
                missing.append(str(rel_path))
    if missing:
        preview = ", ".join(missing[:8])
        suffix = "" if len(missing) <= 8 else f" ... (+{len(missing) - 8} more)"
        fail(
            "RED: future calendar CRUD/browser/accessibility story replay artifacts are missing under "
            f"{rel(story_replay_root)}: {preview}{suffix}"
        )
    fail("future browser/user-story/accessibility replay exists but this RED-only checker has not been upgraded by a Build/UI story card")


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "fixture_kind": "self_test",
                "source_authority_refs": ["specs/microservices/calendar.json"],
                "future_story_replay_artifacts": [f"specs/fixtures/calendar-prd/user-story-replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["calendar_user_story_employee_work_event_crud_fixture"].update({
        "source_authority_refs": ["specs/microservices/calendar.json#acceptance_criteria[AC-01]", "specs/microservices/calendar.json#acceptance_criteria[AC-05]"],
        "must_assert": ["AC-01", "AC-05", "create", "update", "audit", "retention", "legal hold", "visible"],
        "must_reject": ["runtime handler", "production-ready UI", "green CI alone"],
    })
    by_id["calendar_user_story_personal_freebusy_privacy_fixture"].update({
        "source_authority_refs": ["specs/microservices/calendar.json#acceptance_criteria[AC-02]"],
        "must_assert": ["AC-02", "free/busy", "403", "minimum necessary disclosure"],
        "must_reject": ["personal title", "personal description", "personal location", "personal attendees", "silent personal availability mining"],
    })
    by_id["calendar_user_story_manager_action_card_workflow_fixture"].update({
        "source_authority_refs": ["specs/microservices/calendar.json#acceptance_criteria[AC-03]"],
        "must_assert": ["AC-03", "action card", "policy_basis", "Workflow", "audit_ref"],
        "must_reject": ["implicit Workflow inference", "without policy_basis", "without audit_ref"],
    })
    by_id["calendar_user_story_compliance_legal_hold_fixture"].update({
        "source_authority_refs": ["specs/microservices/calendar.json#acceptance_criteria[AC-04]"],
        "must_assert": ["AC-04", "legal hold", "attendee history", "action cards", "audit chain", "chain-of-custody"],
        "must_reject": ["purge", "hard delete", "chain-of-custody gap"],
    })
    by_id["calendar_user_story_accessibility_i18n_responsive_fixture"].update({
        "source_authority_refs": ["specs/microservices/calendar.json#user_experience"],
        "must_assert": ["WCAG 2.2 AA", "keyboard", "focus order", "screen-reader", "responsive", "loading", "error", "KR", "en-US"],
        "must_reject": ["green CI alone", "keyboard traversal missing", "screen-reader labels missing"],
    })
    by_id["calendar_user_story_pr1145_api_replay_link_fixture"].update({
        "source_authority_refs": sorted(REQUIRED_PR_1145_REPLAY_ARTIFACTS),
        "must_assert": ["PR #1145", "OpenAPI", "AsyncAPI", "proto", "explicit no-UI N/A", "API/contract-only"],
        "must_reject": ["browser evidence from API-only", "UI readiness", "customer availability"],
    })
    return {
        "fixture_plan_id": EXPECTED_PLAN_ID,
        "kanban_task": EXPECTED_TASK,
        "parent_learning_task": EXPECTED_PARENT,
        "source_contract_pr": "https://github.com/jason931225/oyatie/pull/1145",
        "source_contract_merge_commit": PR_1145_MERGE_COMMIT,
        "status": EXPECTED_MANIFEST_STATUS,
        "claim_boundary": "RED fixture only; no runtime, no UI readiness, no production, no GA, no customer-availability claim",
        "source_authority_refs": sorted(REQUIRED_SOURCE_REFS),
        "existing_api_contract_replay_evidence": {
            "pr": 1145,
            "browser_live_ui_accessibility_status": "explicit_no_ui_NA",
            "artifact_refs": sorted(REQUIRED_PR_1145_REPLAY_ARTIFACTS),
            "notes": ["OpenAPI", "AsyncAPI", "proto", "explicit no-UI N/A", "API/contract-only"],
        },
        "required_personas": REQUIRED_PERSONAS[:],
        "required_prd_acceptance_criteria": REQUIRED_AC_IDS[:],
        "accessibility_i18n_responsive_checklist": ["WCAG 2.2 AA", "keyboard", "focus order", "screen-reader", "responsive", "loading", "error", "KR", "en-US", "green CI alone is insufficient"],
        "runtime_evidence_policy": ["when no UI exists record explicit N/A", "fail-closed browser Playwright evidence pending", "green CI alone is insufficient"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_prd_source_lock(load_json(PRD_PATH, "calendar PRD"))
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

    expect_rejected("missing persona", lambda data: data.update({"required_personas": ["employee_end_user"]}))
    expect_rejected("missing WCAG checklist", lambda data: data.update({"accessibility_i18n_responsive_checklist": ["keyboard"]}))
    expect_rejected("missing PR 1145 replay", lambda data: data["existing_api_contract_replay_evidence"].update({"artifact_refs": []}))
    expect_rejected("forged top-level green status", lambda data: data.update({"status": "passed_after_build_skeleton_replay"}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    expect_rejected("missing personal privacy rejection", lambda data: data["fixtures"][1].update({"must_reject": ["personal title"]}))
    expect_rejected("bad future replay root", lambda data: data["fixtures"][0].update({"future_story_replay_artifacts": ["specs/fixtures/calendar-prd/replay/bad.fixture.json"]}))
    expect_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_story_replay_artifacts": ["specs/fixtures/calendar-prd/user-story-replay/bad.generated.json"]}))
    print("calendar user-story RED fixture self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST), help="user-story RED fixture manifest JSON path")
    parser.add_argument("--story-replay-root", default=str(DEFAULT_STORY_REPLAY_ROOT), help="future browser/user-story replay artifact root")
    parser.add_argument("--self-test", action="store_true", help="run fail-closed validator self-tests")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    manifest_path = Path(args.manifest)
    if not manifest_path.is_absolute():
        manifest_path = REPO_ROOT / manifest_path
    story_replay_root = Path(args.story_replay_root)
    if not story_replay_root.is_absolute():
        story_replay_root = REPO_ROOT / story_replay_root
    manifest = load_json(manifest_path, "calendar user-story RED fixture manifest")
    if args.self_test:
        run_self_tests(manifest)
        return
    validate_prd_source_lock(load_json(PRD_PATH, "calendar PRD"))
    by_id = validate_manifest(manifest)
    validate_story_replay_artifacts(by_id, story_replay_root)


if __name__ == "__main__":
    main()
