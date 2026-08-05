#!/usr/bin/env python3
"""Fail-closed RED checker for meet source-map and contract replay fixtures.

This is a RED-only metadata/source-lock guard. It validates that the meet fixture
manifest is grounded in specs/microservices/meet.json, treats oya/meet/manifest.json
as inventory/provenance only, rejects legacy microservices/meet/** live-destination
claims, and remains RED until future Build cards create source-backed replay
artifacts for the service-local OpenAPI/AsyncAPI/proto contracts and the scoped
workspace shell projection.
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
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "meet" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "meet" / "replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "meet.json"
MANIFEST_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
INVENTORY_MANIFEST_PATH = REPO_ROOT / "oya" / "meet" / "manifest.json"

EXPECTED_FIXTURE_IDS = [
    "meet_authority_source_map_fixture",
    "meet_retired_path_hygiene_fixture",
    "meet_contract_replay_openapi_fixture",
    "meet_contract_replay_asyncapi_fixture",
    "meet_contract_replay_proto_fixture",
    "meet_workspace_projection_scope_fixture",
    "meet_consent_privacy_security_negative_fixture",
    "meet_browser_accessibility_gate_fixture",
    "meet_generated_face_no_hand_edit_fixture",
    "meet_no_runtime_product_readiness_overclaim_fixture",
    "meet_build_parentage_fixture",
]

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/meet.json",
    "specs/microservices/manifests-index.json#microservices[name=meet]",
    "oya/meet/manifest.json",
    "kanban:t_7f81620b#meet-source-lock-review",
    "kanban:t_409487e4#meet-red-source-map-contract-replay",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/meet/manifest.json",
    "oya/meet/contracts/*",
    "oya/meet/catalog/*",
    "oya/meet/IPs/*",
    "microservices/meet/contracts/*",
    "microservices/meet/capabilities/*",
    "microservices/meet/IP-*.md",
    "microservices/meet/slos/*",
}

EXPECTED_RED_STATUS = "RED_UNTIL_MEET_REPLAY_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"

CONTRACT_SOURCE_FILES = {
    "openapi": "oya/meet/contracts/openapi/meet.yaml",
    "asyncapi": "oya/meet/contracts/asyncapi/meet-events.yaml",
    "proto": "oya/meet/contracts/proto/meet.proto",
    "workspace_projection": "contracts/openapi/workspace/workspace-meet-v1.yaml",
}

LEGACY_CONTRACT_POINTERS = {
    "openapi": "microservices/meet/contracts/openapi/meet.yaml",
    "asyncapi": "microservices/meet/contracts/asyncapi/meet-events.yaml",
    "proto": "microservices/meet/contracts/proto/meet.proto",
}

REQUIRED_FIXTURE_SOURCE_LINE_REFS = {
    "authority_resolution": "specs/microservices/meet.json:15-30",
    "legacy_disposition": "specs/microservices/meet.json:146-192",
    "privacy_security": "specs/microservices/meet.json:281-329",
    "red_fixture_strategy": "specs/microservices/meet.json:341-375",
    "api_contract_replay": "specs/microservices/meet.json:376-392",
    "browser_accessibility": "specs/microservices/meet.json:394-410",
    "generated_faces_policy": "specs/microservices/meet.json:419-439",
    "non_claims": "specs/microservices/meet.json:467-472",
}

REQUIRED_CONTRACT_ASSERTION_TERMS = [
    "room create/read/archive",
    "meeting instance start/end",
    "participant role/lobby/consent",
    "recording start/finalize",
    "transcript/caption/summary",
    "tenant mismatch",
    "Cedar deny",
    "idempotency conflict",
    "missing consent",
    "stale source-map paths",
]


def fail(message: str) -> NoReturn:
    print(f"meet RED source-map contract replay check failed: {message}", file=sys.stderr)
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


def require_contains_all(haystack: object, needles: set[str] | list[str], label: str) -> None:
    values = set(str(item) for item in haystack) if isinstance(haystack, (list, set, tuple)) else set()
    missing = sorted(set(needles) - values)
    require(not missing, f"{label} missing {missing}")


def require_terms(value: object, terms: list[str], label: str) -> None:
    haystack = text(value)
    missing = [term for term in terms if term.lower() not in haystack]
    require(not missing, f"{label} missing terms {missing}")


def require_fixture_terms(fixture: dict[str, Any], key: str, terms: list[str], label: str) -> None:
    require_terms(fixture.get(key, []), terms, f"{label} {key}")


def require_fixture_sources(fixture: dict[str, Any], sources: list[str], label: str) -> None:
    values = set(str(item) for item in fixture.get("source_authority_refs", []))
    missing = [source for source in sources if not any(source in value for value in values)]
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
            artifact.startswith("specs/fixtures/meet/replay/"),
            f"future replay artifact must stay under specs/fixtures/meet/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_prd_source_lock(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "meet PRD _meta must be an object")
    require(meta.get("spec_id") == "PRD-MEET", "meet PRD spec_id must be PRD-MEET")
    require(meta.get("status") == "Accepted", "meet PRD status must be Accepted")
    require(meta.get("authority_lock_task") == "t_13b685a0", "meet source-lock authority task drifted")

    authority = prd.get("authority_resolution", {})
    require(isinstance(authority, dict), "meet authority_resolution must be an object")
    require(authority.get("current_authority") == "/specs/microservices/meet.json", "meet current_authority must be /specs/microservices/meet.json")
    require(authority.get("manifest_index_ref") == "/specs/microservices/manifests-index.json#microservices[name=meet]", "meet manifest_index_ref drifted")
    require(authority.get("manifest_inventory_ref") == "/oya/meet/manifest.json", "meet manifest_inventory_ref drifted")
    require(authority.get("canonical_service_root") == "/oya/meet/", "meet canonical_service_root must be /oya/meet/")
    require_terms(authority, ["microservices/meet/", "legacy provenance", "future cards", "no handler", "production readiness"], "meet authority_resolution")

    identity = prd.get("identity", {})
    require(isinstance(identity, dict), "meet identity must be an object")
    require(identity.get("product_id") == "meet", "meet identity.product_id must be meet")
    require(identity.get("user_facing_surface") is True, "meet must remain user-facing for browser/a11y gate planning")

    source_map = prd.get("source_reference_map", {})
    require(isinstance(source_map, dict), "meet source_reference_map must be an object")
    verified_paths = {str(item.get("current_path")) for item in source_map.get("verified_current_paths", []) if isinstance(item, dict)}
    require_contains_all(verified_paths, list(CONTRACT_SOURCE_FILES.values()), "meet verified current contract paths")
    legacy_rows = source_map.get("legacy_manifest_reference_disposition")
    require(isinstance(legacy_rows, list), "meet legacy_manifest_reference_disposition must be a list")
    legacy_text = text(legacy_rows)
    require("microservices/meet/contracts/openapi/meet.yaml" in legacy_text, "meet legacy OpenAPI pointer disposition missing")
    require("quarantined_unresolved_legacy_ip_references" in legacy_text, "meet unresolved legacy IP disposition missing")

    replay = prd.get("api_contract_replay_expectations", {})
    require(isinstance(replay, dict), "meet api_contract_replay_expectations must be an object")
    replay_surfaces = text(replay.get("required_surfaces", []))
    missing_surfaces = [path for path in CONTRACT_SOURCE_FILES.values() if path.lower() not in replay_surfaces]
    require(not missing_surfaces, f"meet replay required surfaces missing {missing_surfaces}")
    require_terms(replay.get("must_cover", []), REQUIRED_CONTRACT_ASSERTION_TERMS, "meet replay must_cover")
    require("no contract replay has passed yet" in str(replay.get("non_claim", "")).lower(), "meet replay non_claim must stay explicit")

    fixture_strategy = prd.get("red_fixture_strategy", {})
    require(isinstance(fixture_strategy, dict), "meet red_fixture_strategy must be an object")
    require_terms(fixture_strategy, ["RED-MEET-SOURCE-MAP-01", "RED-MEET-LEGACY-PATH-01", "RED-MEET-CONTRACT-01", "RED-MEET-CONSENT-01", "RED-MEET-A11Y-01"], "meet red_fixture_strategy")

    lower_prd = text(prd)
    for term in ["No meet runtime implementation", "No legacy microservices/meet/** destination authority", "No generated JSON hand edits"]:
        require(term.lower() in lower_prd, f"meet PRD must retain non-claim {term!r}")


def validate_manifest_index_source_lock(index: dict[str, Any]) -> None:
    rows = index.get("microservices")
    require(isinstance(rows, list), "manifests-index microservices must be a list")
    meet_rows = [row for row in rows if isinstance(row, dict) and row.get("name") == "meet"]
    require(len(meet_rows) == 1, f"manifests-index must contain exactly one meet row; got {len(meet_rows)}")
    row = meet_rows[0]
    require(row.get("manifest") == "oya/meet/manifest.json", "meet manifest-index row must point to oya/meet/manifest.json")
    require(row.get("fd001_material") is False, "meet manifest-index fd001_material must remain false")
    require(row.get("authority_status") == "source-authority-reconciled-by-t_ff8bab02", "meet authority_status drifted")
    boundary = str(row.get("authority_boundary", "")).lower()
    require("inventory/provenance only" in boundary, "meet authority boundary must state inventory/provenance only")
    require("no specs/microservices/meet.json" in boundary, "meet manifest-index row must preserve pre-lock no-PRD provenance note")
    require("no runtime/product-readiness claim" in boundary, "meet manifest-index row must deny readiness claims")


def validate_inventory_source_lock(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "meet", "meet inventory manifest microservice must be meet")
    contracts = inventory.get("contracts", {})
    require(isinstance(contracts, dict), "meet inventory contracts must be an object")
    require_contains_all(contracts.get("openapi"), [LEGACY_CONTRACT_POINTERS["openapi"]], "meet inventory openapi provenance pointers")
    require_contains_all(contracts.get("asyncapi"), [LEGACY_CONTRACT_POINTERS["asyncapi"]], "meet inventory asyncapi provenance pointers")
    require_contains_all(contracts.get("proto"), [LEGACY_CONTRACT_POINTERS["proto"]], "meet inventory proto provenance pointers")
    lower_inventory = text(inventory)
    require("microservices/meet/ip-001" in lower_inventory, "meet inventory must retain legacy IP provenance for quarantine checks")


def validate_contract_source_files() -> None:
    for family, raw_path in CONTRACT_SOURCE_FILES.items():
        path = REPO_ROOT / raw_path
        require(path.exists(), f"meet {family} source contract must exist at {raw_path}")
        require(path.is_file(), f"meet {family} source contract must be a file: {raw_path}")
        require(not raw_path.endswith(GENERATED_SUFFIX), f"meet {family} source contract must not be a generated face: {raw_path}")
        if family == "workspace_projection":
            require(raw_path.startswith("contracts/openapi/workspace/"), f"workspace projection must live under contracts/openapi/workspace/: {raw_path}")
        else:
            require(raw_path.startswith("oya/meet/contracts/"), f"meet {family} source contract must live under oya/meet/contracts/: {raw_path}")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "MEET-SOURCE-MAP-CONTRACT-REPLAY-RED-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_409487e4", "manifest must bind to kanban task t_409487e4")
    require(manifest.get("review_fix_parent_task") == "t_7f81620b", "manifest must bind to review parent t_7f81620b")
    require("runtime" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny runtime claims")
    require("production readiness" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny production readiness")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")
    line_refs = manifest.get("source_lock_line_refs")
    require(isinstance(line_refs, dict), "source_lock_line_refs must be an object")
    for key, expected in REQUIRED_FIXTURE_SOURCE_LINE_REFS.items():
        require(line_refs.get(key) == expected, f"source_lock_line_refs.{key} must cite {expected}")

    replay = manifest.get("contract_replay_expectations")
    require(isinstance(replay, dict), "contract_replay_expectations must be an object")
    require(set(replay) == set(CONTRACT_SOURCE_FILES), f"contract_replay_expectations keys must be {sorted(CONTRACT_SOURCE_FILES)}")
    for key, expected_path in CONTRACT_SOURCE_FILES.items():
        section = replay[key]
        require(isinstance(section, dict), f"contract_replay_expectations.{key} must be an object")
        require(section.get("source_path") == expected_path, f"{key} source_path drifted")
        if key in LEGACY_CONTRACT_POINTERS:
            require(section.get("legacy_manifest_pointer") == LEGACY_CONTRACT_POINTERS[key], f"{key} legacy_manifest_pointer drifted")
        else:
            require("workspace shell integration" in text(section), "workspace projection must be explicitly scoped")
        require_terms(section.get("must_assert", []), ["tenant", "idempotency", "audit"], f"{key} replay assertions")
        require_terms(section.get("must_reject", []), ["legacy", "missing consent"], f"{key} replay negative cases")

    require(manifest.get("future_replay_root") == "specs/fixtures/meet/replay/", "future_replay_root must be source-locked")
    require_terms(manifest.get("browser_user_story_accessibility_gate", []), ["WCAG 2.2 AA", "keyboard", "screen-reader", "N/A"], "browser/user-story/accessibility gate")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match meet RED plan; got {actual_ids}")
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_replay_artifacts(fixture)

    require_fixture_sources(by_id["meet_authority_source_map_fixture"], ["specs/microservices/meet.json", "manifests-index", "oya/meet/manifest.json"], "authority source-map fixture")
    require_fixture_terms(by_id["meet_authority_source_map_fixture"], "must_assert", ["PRD-MEET", "current_authority", "manifest-index", "inventory/provenance only"], "authority source-map fixture")
    require_fixture_terms(by_id["meet_retired_path_hygiene_fixture"], "must_reject", ["microservices/meet/** as live destination", "microservices/meet/contracts", "microservices/meet/IP-", "legacy root"], "retired path hygiene fixture")
    require_fixture_sources(by_id["meet_contract_replay_openapi_fixture"], [CONTRACT_SOURCE_FILES["openapi"]], "OpenAPI replay fixture")
    require_fixture_sources(by_id["meet_contract_replay_asyncapi_fixture"], [CONTRACT_SOURCE_FILES["asyncapi"]], "AsyncAPI replay fixture")
    require_fixture_sources(by_id["meet_contract_replay_proto_fixture"], [CONTRACT_SOURCE_FILES["proto"]], "proto replay fixture")
    require_fixture_sources(by_id["meet_workspace_projection_scope_fixture"], [CONTRACT_SOURCE_FILES["workspace_projection"]], "workspace projection fixture")
    require_fixture_terms(by_id["meet_consent_privacy_security_negative_fixture"], "must_reject", ["recording without consent", "transcription without consent", "lobby bypass", "silent E2E downgrade"], "privacy/security fixture")
    require_fixture_terms(by_id["meet_browser_accessibility_gate_fixture"], "must_assert", ["WCAG 2.2 AA", "keyboard", "screen-reader", "N/A rationale"], "browser/a11y fixture")
    require_fixture_terms(by_id["meet_generated_face_no_hand_edit_fixture"], "must_reject", ["*.generated.json", "hand edit"], "generated face fixture")
    require_fixture_terms(by_id["meet_no_runtime_product_readiness_overclaim_fixture"], "must_reject", ["production readiness", "live SLO", "GA", "media-plane", "green CI alone"], "no-overclaim fixture")
    require_fixture_terms(by_id["meet_build_parentage_fixture"], "must_assert", ["t_7f81620b", "t_409487e4", "allowed path", "generated-face"], "build parentage fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("microservices/meet/** destination authority is not authorized" in lower_manifest, "manifest must explicitly reject legacy meet destination authority")
    return by_id


def validate_replay_artifacts(by_id: dict[str, dict[str, Any]], replay_root: Path) -> None:
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
            "RED: future meet source-map and OpenAPI/AsyncAPI/proto/workspace replay artifacts are missing under "
            f"{rel(replay_root)}: {preview}{suffix}"
        )
    fail("future meet contract replay is not implemented; this RED-only checker must be extended by a Build card before green status")


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "fixture_kind": "self_test",
                "source_authority_refs": ["specs/microservices/meet.json"],
                "future_replay_artifacts": [f"specs/fixtures/meet/replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["meet_authority_source_map_fixture"]["source_authority_refs"] = ["specs/microservices/meet.json", "specs/microservices/manifests-index.json#microservices[name=meet]", "oya/meet/manifest.json"]
    by_id["meet_authority_source_map_fixture"]["must_assert"] = ["PRD-MEET", "current_authority", "manifest-index", "inventory/provenance only"]
    by_id["meet_retired_path_hygiene_fixture"]["must_reject"] = ["microservices/meet/** as live destination", "microservices/meet/contracts", "microservices/meet/IP-", "legacy root"]
    by_id["meet_contract_replay_openapi_fixture"]["source_authority_refs"] = [CONTRACT_SOURCE_FILES["openapi"]]
    by_id["meet_contract_replay_asyncapi_fixture"]["source_authority_refs"] = [CONTRACT_SOURCE_FILES["asyncapi"]]
    by_id["meet_contract_replay_proto_fixture"]["source_authority_refs"] = [CONTRACT_SOURCE_FILES["proto"]]
    by_id["meet_workspace_projection_scope_fixture"]["source_authority_refs"] = [CONTRACT_SOURCE_FILES["workspace_projection"]]
    by_id["meet_consent_privacy_security_negative_fixture"]["must_reject"] = ["recording without consent", "transcription without consent", "lobby bypass", "silent E2E downgrade"]
    by_id["meet_browser_accessibility_gate_fixture"]["must_assert"] = ["WCAG 2.2 AA", "keyboard", "screen-reader", "N/A rationale"]
    by_id["meet_generated_face_no_hand_edit_fixture"]["must_reject"] = ["*.generated.json", "hand edit"]
    by_id["meet_no_runtime_product_readiness_overclaim_fixture"]["must_reject"] = ["production readiness", "live SLO", "GA", "media-plane", "green CI alone"]
    by_id["meet_build_parentage_fixture"]["must_assert"] = ["t_7f81620b", "t_409487e4", "allowed path", "generated-face"]
    return {
        "fixture_plan_id": "MEET-SOURCE-MAP-CONTRACT-REPLAY-RED-001",
        "kanban_task": "t_409487e4",
        "review_fix_parent_task": "t_7f81620b",
        "claim_boundary": "metadata/fixture-only; no runtime handlers or production readiness claim",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "source_lock_line_refs": REQUIRED_FIXTURE_SOURCE_LINE_REFS.copy(),
        "contract_replay_expectations": {
            "openapi": {"source_path": CONTRACT_SOURCE_FILES["openapi"], "legacy_manifest_pointer": LEGACY_CONTRACT_POINTERS["openapi"], "must_assert": ["tenant", "idempotency", "audit"], "must_reject": ["legacy", "missing consent"]},
            "asyncapi": {"source_path": CONTRACT_SOURCE_FILES["asyncapi"], "legacy_manifest_pointer": LEGACY_CONTRACT_POINTERS["asyncapi"], "must_assert": ["tenant", "idempotency", "audit"], "must_reject": ["legacy", "missing consent"]},
            "proto": {"source_path": CONTRACT_SOURCE_FILES["proto"], "legacy_manifest_pointer": LEGACY_CONTRACT_POINTERS["proto"], "must_assert": ["tenant", "idempotency", "audit"], "must_reject": ["legacy", "missing consent"]},
            "workspace_projection": {"source_path": CONTRACT_SOURCE_FILES["workspace_projection"], "scope_boundary": "workspace shell integration only when explicitly scoped", "must_assert": ["tenant", "idempotency", "audit"], "must_reject": ["legacy", "missing consent"]},
        },
        "future_replay_root": "specs/fixtures/meet/replay/",
        "browser_user_story_accessibility_gate": ["WCAG 2.2 AA", "keyboard", "screen-reader", "N/A"],
        "global_non_claims": ["microservices/meet/** destination authority is not authorized"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_prd_source_lock(load_json(PRD_PATH, "meet PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "meet inventory manifest"))
    validate_contract_source_files()
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
    expect_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/meet/manifest.json"))
    expect_rejected("source lock line ref missing", lambda data: data["source_lock_line_refs"].pop("non_claims"))
    expect_rejected("OpenAPI replay source missing", lambda data: data["fixtures"][2].update({"source_authority_refs": ["oya/meet/contracts/asyncapi/meet-events.yaml"]}))
    expect_rejected("legacy path rejection gap", lambda data: data["fixtures"][1].update({"must_reject": ["microservices/meet/contracts"]}))
    expect_rejected("consent negative gap", lambda data: data["fixtures"][6].update({"must_reject": ["recording without consent"]}))
    expect_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/meet/replay/bad.generated.json"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("meet RED source-map contract replay self-tests passed")


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
    validate_prd_source_lock(load_json(PRD_PATH, "meet PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "meet inventory manifest"))
    validate_contract_source_files()
    by_id = validate_manifest(manifest)
    validate_replay_artifacts(by_id, replay_root)


if __name__ == "__main__":
    main()
