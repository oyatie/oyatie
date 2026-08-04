#!/usr/bin/env python3
"""Fail-closed RED checker for the calendar PRD fixture/contract replay gate.

This is a metadata/source-lock guard. It validates that the calendar RED fixture
manifest is grounded in Accepted PRD-CALENDAR, the manifest-index source lock, and
calendar inventory provenance, then validates source-backed OpenAPI/AsyncAPI/proto
replay fixtures once a Build lane supplies them.
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
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "calendar-prd" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "calendar-prd" / "replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "calendar.json"
MANIFEST_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
INVENTORY_MANIFEST_PATH = REPO_ROOT / "oya" / "calendar" / "manifest.json"
OPENAPI_CONTRACT_PATH = REPO_ROOT / "oya" / "calendar" / "contracts" / "openapi" / "calendar.yaml"
ASYNCAPI_CONTRACT_PATH = REPO_ROOT / "oya" / "calendar" / "contracts" / "asyncapi" / "calendar-events.yaml"
PROTO_CONTRACT_PATH = REPO_ROOT / "oya" / "calendar" / "contracts" / "proto" / "calendar.proto"

EXPECTED_FIXTURE_IDS = [
    "calendar_prd_authority_source_lock_fixture",
    "calendar_prd_ac01_work_event_org_pillar_audit_fixture",
    "calendar_prd_ac02_personal_detail_projection_fixture",
    "calendar_prd_ac03_action_card_workflow_handoff_fixture",
    "calendar_prd_ac04_legal_hold_preservation_fixture",
    "calendar_prd_ac05_jurisdiction_retention_ux_fixture",
    "calendar_prd_produced_contracts_fixture",
    "calendar_prd_api_contract_replay_fixture",
    "calendar_prd_personal_work_pillar_boundary_fixture",
    "calendar_prd_browser_accessibility_evidence_fixture",
    "calendar_prd_inventory_provenance_rejection_fixture",
    "calendar_prd_build_parentage_fixture",
]

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/calendar.json",
    "specs/microservices/manifests-index.json#microservices[name=calendar]",
    "oya/calendar/manifest.json",
    "/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_9aca64a9/calendar-source-map-and-backlog.md",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/calendar/manifest.json",
    "oya/calendar/contracts/*",
    "oya/calendar/catalog/*",
    "oya/calendar/IPs/*",
    "microservices/calendar/manifest.json",
    "microservices/calendar/contracts/*",
}

REQUIRED_AC_IDS = ["AC-01", "AC-02", "AC-03", "AC-04", "AC-05"]
REQUIRED_PRODUCED_CONTRACTS = [
    "calendar.event.v1",
    "calendar.availability_projection.v1",
    "calendar.action_card.v1",
    "calendar.workflow_handoff.v1",
]
REQUIRED_CONTRACT_REPLAY_KEYS = {"openapi", "asyncapi", "proto"}
EXPECTED_RED_STATUS = "RED_UNTIL_REPLAY_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"


def fail(message: str) -> NoReturn:
    print(f"calendar PRD RED fixture contract check failed: {message}", file=sys.stderr)
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


def load_text(path: Path, label: str) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing {label}: {rel(path)}")


def require_contains_all(haystack: object, needles: set[str] | list[str], label: str) -> None:
    values = set(str(item) for item in haystack) if isinstance(haystack, list) else set()
    missing = sorted(set(needles) - values)
    require(not missing, f"{label} missing {missing}")


def require_fixture_terms(fixture: dict[str, Any], key: str, required_terms: list[str], label: str) -> None:
    haystack = text(fixture.get(key, []))
    missing = [term for term in required_terms if term.lower() not in haystack]
    require(not missing, f"{label} {key} missing terms {missing}")


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
    require(isinstance(artifacts, list) and artifacts, f"{fixture.get('fixture_id')} must name future_replay_artifacts")
    for raw in artifacts:
        artifact = str(raw)
        require(
            artifact.startswith("specs/fixtures/calendar-prd/replay/"),
            f"future replay artifact must stay under specs/fixtures/calendar-prd/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_prd_source_lock(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "calendar PRD _meta must be an object")
    require(meta.get("spec_id") == "PRD-CALENDAR", "calendar PRD spec_id must be PRD-CALENDAR")
    require(meta.get("status") == "Accepted", "calendar PRD status must be Accepted")
    identity = prd.get("identity", {})
    require(isinstance(identity, dict), "calendar PRD identity must be an object")
    require(identity.get("product_id") == "calendar", "calendar PRD identity.product_id must be calendar")
    require_contains_all(identity.get("context_model"), ["personal", "work"], "calendar PRD identity.context_model")

    acs = prd.get("acceptance_criteria")
    require(isinstance(acs, list), "calendar PRD acceptance_criteria must be a list")
    ac_ids = [str(item.get("id")) for item in acs if isinstance(item, dict)]
    require(ac_ids == REQUIRED_AC_IDS, f"calendar PRD AC ids/order must be {REQUIRED_AC_IDS}; got {ac_ids}")

    contracts = prd.get("contracts", {})
    require(isinstance(contracts, dict), "calendar PRD contracts must be an object")
    require_contains_all(contracts.get("produces"), REQUIRED_PRODUCED_CONTRACTS, "calendar PRD produced contracts")

    lower_prd = text(prd)
    for term in [
        "silent personal availability mining",
        "workflow_handoff_without_policy_basis",
        "third-party calendar as source of truth",
        "details never cross pillar",
        "minimum necessary disclosure",
    ]:
        require(term in lower_prd, f"calendar PRD must retain boundary term {term!r}")


def validate_manifest_index_source_lock(index: dict[str, Any]) -> None:
    rows = index.get("microservices")
    require(isinstance(rows, list), "manifests-index microservices must be a list")
    calendar_rows = [row for row in rows if isinstance(row, dict) and row.get("name") == "calendar"]
    require(len(calendar_rows) == 1, f"manifests-index must contain exactly one calendar row; got {len(calendar_rows)}")
    row = calendar_rows[0]
    reconciled = (
        row.get("manifest") == "oya/calendar/manifest.json"
        and row.get("prd") == "specs/microservices/calendar.json"
        and row.get("authority_status") == "source-authority-reconciled-by-t_ff8bab02"
        and "inventory/provenance only" in str(row.get("authority_boundary", "")).lower()
    )
    legacy_source_mapped = row.get("manifest") == "microservices/calendar/manifest.json" and row.get("fd001_material") is False
    require(
        reconciled or legacy_source_mapped,
        "calendar manifest-index row must be either source-authority reconciled or the source-mapped legacy row accepted by the RED replay fixtures",
    )


def validate_inventory_source_lock(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "calendar", "calendar inventory manifest microservice must be calendar")
    contracts = inventory.get("contracts", {})
    require(isinstance(contracts, dict), "calendar inventory contracts must be an object")
    require_contains_all(contracts.get("openapi"), ["microservices/calendar/contracts/openapi/calendar.yaml"], "calendar inventory openapi provenance pointers")
    require_contains_all(contracts.get("asyncapi"), ["microservices/calendar/contracts/asyncapi/calendar-events.yaml"], "calendar inventory asyncapi provenance pointers")
    require_contains_all(contracts.get("proto"), ["microservices/calendar/contracts/proto/calendar.proto"], "calendar inventory proto provenance pointers")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "CALENDAR-PRD-RED-FIXTURE-CONTRACT-PLAN-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_46ab5900", "manifest must bind to kanban task t_46ab5900")
    require(manifest.get("parent_plan_spec_task") == "t_9aca64a9", "manifest must bind to parent Plan/Spec task t_9aca64a9")
    require("runtime" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny runtime claims")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")
    require_contains_all(manifest.get("required_prd_acceptance_criteria"), REQUIRED_AC_IDS, "required_prd_acceptance_criteria")
    require_contains_all(manifest.get("required_produced_contracts"), REQUIRED_PRODUCED_CONTRACTS, "required_produced_contracts")
    require(manifest.get("future_replay_root") == "specs/fixtures/calendar-prd/replay/", "future_replay_root must be source-locked")

    replay = manifest.get("contract_replay_expectations")
    require(isinstance(replay, dict), "contract_replay_expectations must be an object")
    require(set(replay) == REQUIRED_CONTRACT_REPLAY_KEYS, f"contract_replay_expectations keys must be {sorted(REQUIRED_CONTRACT_REPLAY_KEYS)}")
    for key, expected_path in {
        "openapi": "oya/calendar/contracts/openapi/calendar.yaml",
        "asyncapi": "oya/calendar/contracts/asyncapi/calendar-events.yaml",
        "proto": "oya/calendar/contracts/proto/calendar.proto",
    }.items():
        section = replay[key]
        require(isinstance(section, dict), f"contract_replay_expectations.{key} must be an object")
        require(section.get("source_path") == expected_path, f"{key} source_path drifted")
        require(section.get("legacy_manifest_pointer", "").startswith("microservices/calendar/contracts/"), f"{key} must record legacy manifest pointer")
        require(isinstance(section.get("must_assert"), list) and len(section["must_assert"]) >= 4, f"{key} must name replay assertions")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match calendar RED plan; got {actual_ids}")
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_replay_artifacts(fixture)

    require_fixture_terms(by_id["calendar_prd_authority_source_lock_fixture"], "must_assert", ["PRD-CALENDAR", "manifest-index", "inventory"], "authority source-lock fixture")
    require_fixture_terms(by_id["calendar_prd_ac01_work_event_org_pillar_audit_fixture"], "must_assert", ["Org pillar", "tenant-DEK", "retention", "audit"], "AC-01 fixture")
    require_fixture_terms(by_id["calendar_prd_ac02_personal_detail_projection_fixture"], "must_reject", ["personal title", "personal description", "personal location", "personal attendees", "silent personal availability mining"], "AC-02 fixture")
    require_fixture_terms(by_id["calendar_prd_ac03_action_card_workflow_handoff_fixture"], "must_reject", ["implicit Workflow inference", "policy_basis", "audit_ref"], "AC-03 fixture")
    require_fixture_terms(by_id["calendar_prd_ac04_legal_hold_preservation_fixture"], "must_assert", ["event", "attendee history", "action cards", "audit chain"], "AC-04 fixture")
    require_fixture_terms(by_id["calendar_prd_ac05_jurisdiction_retention_ux_fixture"], "must_assert", ["KR", "EU", "US", "admin UX"], "AC-05 fixture")
    require_fixture_terms(by_id["calendar_prd_produced_contracts_fixture"], "must_assert", REQUIRED_PRODUCED_CONTRACTS, "produced-contract fixture")
    require_fixture_sources(by_id["calendar_prd_api_contract_replay_fixture"], ["openapi", "asyncapi", "proto"], "API contract replay fixture")
    require_fixture_terms(by_id["calendar_prd_personal_work_pillar_boundary_fixture"], "must_reject", ["silent_calendar_mining", "third-party calendar as source of truth"], "pillar boundary fixture")
    require_fixture_terms(by_id["calendar_prd_browser_accessibility_evidence_fixture"], "must_assert", ["WCAG 2.2 AA", "keyboard", "KR", "N/A"], "browser/accessibility fixture")
    require_fixture_terms(by_id["calendar_prd_inventory_provenance_rejection_fixture"], "must_reject", ["oya/calendar/manifest.json", "microservices/calendar/manifest.json", "microservices/calendar/contracts"], "inventory rejection fixture")
    require_fixture_terms(by_id["calendar_prd_build_parentage_fixture"], "must_assert", ["t_9aca64a9", "t_46ab5900", "allowed path"], "build parentage fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("green ci alone" in lower_manifest, "manifest must reject green-CI-alone UI readiness")
    return by_id


def validate_replay_artifacts(by_id: dict[str, dict[str, Any]], replay_root: Path) -> None:
    validate_contract_sources()
    missing: list[str] = []
    for fixture in by_id.values():
        for raw in fixture["future_replay_artifacts"]:
            rel_path = Path(str(raw))
            expected = REPO_ROOT / rel_path
            if not expected.exists():
                missing.append(str(rel_path))
    if missing:
        preview = ", ".join(missing[:8])
        suffix = "" if len(missing) <= 8 else f" ... (+{len(missing) - 8} more)"
        fail(
            "RED: future calendar PRD OpenAPI/AsyncAPI/proto replay artifacts are missing under "
            f"{rel(replay_root)}: {preview}{suffix}"
        )
    for fixture_id, fixture in by_id.items():
        for raw in fixture["future_replay_artifacts"]:
            validate_replay_fixture(load_json(REPO_ROOT / Path(str(raw)), f"replay fixture {raw}"), fixture_id, fixture)
    print("calendar PRD Build replay fixtures passed")


def require_text_terms(source: str, required_terms: list[str], label: str) -> None:
    lower = source.lower()
    missing = [term for term in required_terms if term.lower() not in lower]
    require(not missing, f"{label} missing terms {missing}")


def validate_contract_sources() -> None:
    openapi = load_text(OPENAPI_CONTRACT_PATH, "calendar OpenAPI contract")
    asyncapi = load_text(ASYNCAPI_CONTRACT_PATH, "calendar AsyncAPI contract")
    proto = load_text(PROTO_CONTRACT_PATH, "calendar proto contract")
    require_text_terms(
        openapi,
        [
            "ownership_pillar",
            "retention_policy_id",
            "legal_hold_ids",
            "audit_ref",
            "ActionCardApprovalRequest",
            "WorkflowHandoffAuditLog",
            "policy_basis",
            "FreeBusyProjection",
            "NO title, description, location, or attendee fields",
        ],
        "OpenAPI replay source",
    )
    require_text_terms(
        asyncapi,
        [
            "calendar.event.v1",
            "calendar.availability_projection.v1",
            "calendar.action_card.v1",
            "calendar.workflow_handoff.v1",
            "policy_basis",
            "audit_ref",
            "WorkflowHandoffCreated",
            "attendee_count_bucket",
        ],
        "AsyncAPI replay source",
    )
    require_text_terms(
        proto,
        [
            "message ActionCard",
            "message WorkflowHandoffAuditLog",
            "policy_basis",
            "audit_ref",
            "legal_hold_ids",
            "ownership_pillar",
            "message FreeBusyProjection",
            "NO event title, description, location, or attendee fields",
        ],
        "proto replay source",
    )


def validate_replay_fixture(replay: dict[str, Any], expected_fixture_id: str, red_fixture: dict[str, Any]) -> None:
    require(replay.get("fixture_id") == expected_fixture_id, f"replay fixture id must be {expected_fixture_id}")
    require(replay.get("build_kanban_task") == "t_7b437a23", f"{expected_fixture_id} must bind to Build task t_7b437a23")
    require(replay.get("red_fixture_task") == "t_46ab5900", f"{expected_fixture_id} must bind to RED task t_46ab5900")
    require(replay.get("status") == "passed_after_build_skeleton_replay", f"{expected_fixture_id} must be marked passed_after_build_skeleton_replay")
    require_contains_all(replay.get("source_authority_refs"), red_fixture.get("source_authority_refs", []), f"{expected_fixture_id} source_authority_refs")
    require(isinstance(replay.get("assertions_passed"), list) and replay["assertions_passed"], f"{expected_fixture_id} must record assertions_passed")
    require(isinstance(replay.get("negative_assertions_rejected"), list) and replay["negative_assertions_rejected"], f"{expected_fixture_id} must record negative_assertions_rejected")
    replay_text = text(replay)
    require("production readiness" not in replay_text and "customer availability" not in replay_text and "ga readiness" not in replay_text, f"{expected_fixture_id} must not make production/GA/customer-availability claims")
    require_fixture_terms(replay, "assertions_passed", red_fixture.get("must_assert", []), expected_fixture_id)
    require_fixture_terms(replay, "negative_assertions_rejected", red_fixture.get("must_reject", []), expected_fixture_id)


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "fixture_kind": "self_test",
                "source_authority_refs": ["specs/microservices/calendar.json"],
                "future_replay_artifacts": [f"specs/fixtures/calendar-prd/replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["calendar_prd_authority_source_lock_fixture"]["must_assert"] = ["PRD-CALENDAR", "manifest-index", "inventory"]
    by_id["calendar_prd_ac01_work_event_org_pillar_audit_fixture"]["must_assert"] = ["Org pillar", "tenant-DEK", "retention", "audit"]
    by_id["calendar_prd_ac02_personal_detail_projection_fixture"]["must_reject"] = ["personal title", "personal description", "personal location", "personal attendees", "silent personal availability mining"]
    by_id["calendar_prd_ac03_action_card_workflow_handoff_fixture"]["must_reject"] = ["implicit Workflow inference", "policy_basis", "audit_ref"]
    by_id["calendar_prd_ac04_legal_hold_preservation_fixture"]["must_assert"] = ["event", "attendee history", "action cards", "audit chain"]
    by_id["calendar_prd_ac05_jurisdiction_retention_ux_fixture"]["must_assert"] = ["KR", "EU", "US", "admin UX"]
    by_id["calendar_prd_produced_contracts_fixture"]["must_assert"] = REQUIRED_PRODUCED_CONTRACTS[:]
    by_id["calendar_prd_api_contract_replay_fixture"]["source_authority_refs"] = ["oya/calendar/contracts/openapi/calendar.yaml", "oya/calendar/contracts/asyncapi/calendar-events.yaml", "oya/calendar/contracts/proto/calendar.proto"]
    by_id["calendar_prd_personal_work_pillar_boundary_fixture"]["must_reject"] = ["silent_calendar_mining", "third-party calendar as source of truth"]
    by_id["calendar_prd_browser_accessibility_evidence_fixture"]["must_assert"] = ["WCAG 2.2 AA", "keyboard", "KR", "N/A"]
    by_id["calendar_prd_inventory_provenance_rejection_fixture"]["must_reject"] = ["oya/calendar/manifest.json", "microservices/calendar/manifest.json", "microservices/calendar/contracts"]
    by_id["calendar_prd_build_parentage_fixture"]["must_assert"] = ["t_9aca64a9", "t_46ab5900", "allowed path"]
    return {
        "fixture_plan_id": "CALENDAR-PRD-RED-FIXTURE-CONTRACT-PLAN-001",
        "kanban_task": "t_46ab5900",
        "parent_plan_spec_task": "t_9aca64a9",
        "claim_boundary": "metadata/fixture-only; no runtime handlers or production claim",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "required_prd_acceptance_criteria": REQUIRED_AC_IDS[:],
        "required_produced_contracts": REQUIRED_PRODUCED_CONTRACTS[:],
        "contract_replay_expectations": {
            "openapi": {"source_path": "oya/calendar/contracts/openapi/calendar.yaml", "legacy_manifest_pointer": "microservices/calendar/contracts/openapi/calendar.yaml", "must_assert": ["a", "b", "c", "d"]},
            "asyncapi": {"source_path": "oya/calendar/contracts/asyncapi/calendar-events.yaml", "legacy_manifest_pointer": "microservices/calendar/contracts/asyncapi/calendar-events.yaml", "must_assert": ["a", "b", "c", "d"]},
            "proto": {"source_path": "oya/calendar/contracts/proto/calendar.proto", "legacy_manifest_pointer": "microservices/calendar/contracts/proto/calendar.proto", "must_assert": ["a", "b", "c", "d"]},
        },
        "future_replay_root": "specs/fixtures/calendar-prd/replay/",
        "global_non_claims": ["green CI alone is insufficient"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_prd_source_lock(load_json(PRD_PATH, "calendar PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "calendar inventory manifest"))
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

    expect_rejected("missing fixture id", lambda data: data["fixtures"].pop())
    expect_rejected("missing AC id", lambda data: data.update({"required_prd_acceptance_criteria": ["AC-01"]}))
    expect_rejected("missing produced contract", lambda data: data.update({"required_produced_contracts": ["calendar.event.v1"]}))
    expect_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/calendar/manifest.json"))
    expect_rejected("AC-02 personal detail leak rejection gap", lambda data: data["fixtures"][2].update({"must_reject": ["personal title"]}))
    expect_rejected("AC-03 workflow policy gap", lambda data: data["fixtures"][3].update({"must_reject": ["implicit Workflow inference"]}))
    expect_rejected("API replay sources missing", lambda data: data["fixtures"][7].update({"source_authority_refs": ["oya/calendar/contracts/openapi/calendar.yaml"]}))
    expect_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/calendar-prd/replay/bad.generated.json"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("calendar PRD RED fixture contract self-tests passed")


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
    validate_prd_source_lock(load_json(PRD_PATH, "calendar PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "calendar inventory manifest"))
    by_id = validate_manifest(manifest)
    validate_replay_artifacts(by_id, replay_root)


if __name__ == "__main__":
    main()
