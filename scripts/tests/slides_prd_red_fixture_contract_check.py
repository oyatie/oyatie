#!/usr/bin/env python3
"""Fail-closed RED checker for the slides PRD fixture/contract replay gate.

This is a source-authority metadata and contract-plan guard. It validates that
Slides RED fixtures are grounded in specs/microservices/slides.json, the
manifest-index source lock, and oya/slides inventory provenance, then remains
RED until later Build cards provide source-backed OpenAPI/AsyncAPI/proto replay
artifacts plus browser/user-story/accessibility evidence.
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
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "slides" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "slides" / "replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "slides.json"
MANIFEST_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
INVENTORY_MANIFEST_PATH = REPO_ROOT / "oya" / "slides" / "manifest.json"

CANDIDATE_CASES = {
    "valid_slides_source_authority_metadata_fixture": REPO_ROOT / "specs" / "fixtures" / "slides" / "valid-slides-source-authority.json",
    "rejects_legacy_microservices_slides_manifest_fixture": REPO_ROOT / "specs" / "fixtures" / "slides" / "rejects-legacy-microservices-manifest.json",
    "rejects_runtime_readiness_claim_fixture": REPO_ROOT / "specs" / "fixtures" / "slides" / "rejects-runtime-readiness-claim.json",
    "rejects_retired_foundry_runtime_fixture": REPO_ROOT / "specs" / "fixtures" / "slides" / "rejects-retired-foundry-runtime.json",
}

EXPECTED_FIXTURE_IDS = [
    "slides_prd_authority_source_lock_fixture",
    "slides_deck_crud_open_save_contract_fixture",
    "slides_collaboration_crdt_cursor_conflict_fixture",
    "slides_present_broadcast_contract_fixture",
    "slides_import_export_contract_fixture",
    "slides_acl_share_version_history_fixture",
    "slides_accessibility_reduced_motion_fixture",
    "slides_ai_assist_tier_disclosure_fixture",
    "slides_data_retention_audit_fixture",
    "slides_integration_boundaries_fixture",
    "slides_browser_user_story_evidence_fixture",
    "slides_inventory_provenance_rejection_fixture",
    "slides_build_parentage_fixture",
]

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/slides.json",
    "specs/microservices/manifests-index.json#microservices[name=slides]",
    "oya/slides/manifest.json",
    "kanban:t_228dc16e#slides-prd-source-authority",
    "kanban:t_62dcb74a#review-fix-slides-source-authority-approved",
    "kanban:t_7e63e03d#red-fixture-contract",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/slides/manifest.json",
    "oya/slides/contracts/*",
    "oya/slides/capabilities/*",
    "oya/slides/IPs/*",
    "oya/slides/crates/*",
    "microservices/slides/manifest.json",
    "microservices/slides/contracts/*",
    "microservices/slides/PRD.md",
    "legacy microservices/slides/** provenance only",
}

REQUIRED_FIXTURE_FAMILY_IDS = [f"FIX-SLIDES-{idx:03d}" for idx in range(1, 11)]
REQUIRED_SERVICE_BOUNDARIES = ["sheets", "drive", "messenger", "social/community", "mail", "identity", "intelligence"]
REQUIRED_REPLAY_KEYS = {"openapi", "asyncapi", "proto"}
CONTRACT_SOURCE_FILES = {
    "openapi": "oya/slides/contracts/openapi/slides.yaml",
    "asyncapi": "oya/slides/contracts/asyncapi/slides-events.yaml",
    "proto": "oya/slides/contracts/proto/slides.proto",
}
EXPECTED_RED_STATUS = "RED_UNTIL_REPLAY_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"
BUILD_TASK = "t_14ab9752"
REPLAY_PASS_STATUS = "SOURCE_BACKED_REPLAY_PASS"


def fail(message: str) -> NoReturn:
    print(f"slides PRD RED fixture contract check failed: {message}", file=sys.stderr)
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
    values = set(str(item) for item in haystack) if isinstance(haystack, list) else set()
    missing = sorted(set(needles) - values)
    require(not missing, f"{label} missing {missing}")


def require_text_terms(value: object, required_terms: list[str], label: str) -> None:
    haystack = text(value)
    missing = [term for term in required_terms if term.lower() not in haystack]
    require(not missing, f"{label} missing terms {missing}")


def require_fixture_terms(fixture: dict[str, Any], key: str, required_terms: list[str], label: str) -> None:
    require_text_terms(fixture.get(key, []), required_terms, f"{label} {key}")


def require_fixture_sources(fixture: dict[str, Any], required_sources: list[str], label: str) -> None:
    values = set(str(item) for item in fixture.get("source_authority_refs", []))
    require("specs/microservices/slides.json" in values, f"{label} must cite specs/microservices/slides.json")
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
            artifact.startswith("specs/fixtures/slides/replay/"),
            f"future replay artifact must stay under specs/fixtures/slides/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_prd_source_lock(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "slides PRD _meta must be an object")
    require(meta.get("spec_id") == "PRD-SLIDES", "slides PRD spec_id must be PRD-SLIDES")
    require(meta.get("status") == "Draft", "slides source-authority status must remain Draft until later promotion authority")
    require(meta.get("authority_lock_task") == "t_228dc16e", "slides source lock must bind to t_228dc16e")

    authority = prd.get("authority_resolution", {})
    require(isinstance(authority, dict), "slides authority_resolution must be an object")
    require(str(authority.get("current_authority", "")).lstrip("/") == "specs/microservices/slides.json", "slides current authority path drifted")
    require(authority.get("canonical_service_home") == "oya/slides", "slides canonical service home must be oya/slides")
    require("runtime" in text(authority.get("claim_boundary", "")), "claim boundary must deny runtime readiness")
    require("product-readiness" in text(authority.get("claim_boundary", "")) or "product readiness" in text(authority.get("claim_boundary", "")), "claim boundary must deny product readiness")
    require_text_terms(authority.get("retired_authority_mapping", {}), ["foundry", "intelligence"], "retired authority mapping")

    source_map = prd.get("source_reference_map", {})
    require(isinstance(source_map, dict), "source_reference_map must be an object")
    quarantine = source_map.get("stale_path_quarantine", {})
    require(isinstance(quarantine, dict), "stale_path_quarantine must be an object")
    require(quarantine.get("legacy_prefix") == "microservices/slides/", "legacy slides prefix must stay quarantined")
    require(quarantine.get("current_home_remap_prefix") == "oya/slides/", "current-home remap must be oya/slides/")

    identity = prd.get("identity", {})
    require(isinstance(identity, dict), "slides identity must be an object")
    require(identity.get("product_id") == "slides", "slides identity.product_id must be slides")
    require(identity.get("user_facing_surface") is True, "slides user_facing_surface must be true")
    require(identity.get("canonical_service_home") == "oya/slides", "slides identity canonical home drifted")

    contracts = prd.get("contract_surfaces", {})
    require(isinstance(contracts, dict), "contract_surfaces must be an object")
    require(set(contracts) == {"rest_openapi", "asyncapi_ws_and_workflow_events", "proto_grpc_streaming"}, "contract_surfaces must cover REST/OpenAPI, AsyncAPI events, and proto/gRPC")
    expected_refs = {
        "rest_openapi": "oya/slides/contracts/openapi/slides.yaml",
        "asyncapi_ws_and_workflow_events": "oya/slides/contracts/asyncapi/slides-events.yaml",
        "proto_grpc_streaming": "oya/slides/contracts/proto/slides.proto",
    }
    for key, expected_ref in expected_refs.items():
        section = contracts.get(key, {})
        require(isinstance(section, dict), f"contract_surfaces.{key} must be an object")
        require(expected_ref in str(section.get("source_ref", "")), f"contract_surfaces.{key} source_ref drifted")
        require(isinstance(section.get("families"), list) and len(section["families"]) >= 3, f"contract_surfaces.{key} must declare families")
        require(isinstance(section.get("fixture_expectations"), list) and len(section["fixture_expectations"]) >= 3, f"contract_surfaces.{key} must declare fixture expectations")

    service_boundaries = prd.get("service_dependencies", {}).get("normative_boundaries", [])
    services = [str(item.get("service")) for item in service_boundaries if isinstance(item, dict)]
    require_contains_all(services, REQUIRED_SERVICE_BOUNDARIES, "service dependency normative boundaries")

    fixture_plan = prd.get("fixture_plan", {})
    require(isinstance(fixture_plan, dict), "fixture_plan must be an object")
    family_ids = [str(item.get("id")) for item in fixture_plan.get("red_fixture_families", []) if isinstance(item, dict)]
    require(family_ids == REQUIRED_FIXTURE_FAMILY_IDS, f"fixture family ids/order must be {REQUIRED_FIXTURE_FAMILY_IDS}; got {family_ids}")
    require_text_terms(fixture_plan, ["deck CRUD", "CRDT", "present", "import/export", "ACL", "accessibility", "AI", "audit"], "fixture_plan")

    accessibility = prd.get("accessibility_ux_evidence", {})
    require(isinstance(accessibility, dict), "accessibility_ux_evidence must be an object")
    require_text_terms(accessibility, ["WCAG 2.2 AA", "keyboard", "screen-reader", "reduced-motion", "browser/user-story"], "accessibility UX evidence")

    ai_bounds = prd.get("ai_automation_bounds", {})
    require(isinstance(ai_bounds, dict), "ai_automation_bounds must be an object")
    tiers = [str(item.get("tier")) for item in ai_bounds.get("capability_tiers", []) if isinstance(item, dict)]
    require_contains_all(tiers, ["T0", "T1", "T2"], "AI capability tiers")
    require_text_terms(ai_bounds, ["intelligence", "human-review", "provenance", "watermark", "refusal"], "AI automation bounds")

    data = prd.get("data_and_compliance", {})
    require(isinstance(data, dict), "data_and_compliance must be an object")
    require_contains_all(data.get("data_classes"), ["INTERNAL_ONLY", "AUDIT", "PII_QUASI"], "data_classes")
    require_text_terms(data, ["retention", "legal hold", "audit", "tenant payload"], "data/compliance")

    evidence = prd.get("evidence_expectations", {})
    require(isinstance(evidence, dict), "evidence_expectations must be an object")
    require_text_terms(evidence.get("red_build_future", []), ["REST/OpenAPI", "AsyncAPI", "proto/gRPC", "browser/user-story/accessibility", "deck", "collaboration", "import/export", "broadcast", "AI", "audit"], "red_build_future evidence expectations")

    lifecycle = prd.get("downstream_lifecycle", {})
    require(isinstance(lifecycle, dict), "downstream_lifecycle must be an object")
    require(lifecycle.get("current_card") == "t_228dc16e", "downstream lifecycle current card drifted")
    require(lifecycle.get("review_fix_card") == "t_62dcb74a", "downstream lifecycle review/fix card drifted")
    require_contains_all(lifecycle.get("planned_chain"), ["t_7e63e03d"], "downstream lifecycle planned chain")

    enforcement = prd.get("enforcement_status", {})
    require(isinstance(enforcement, dict), "enforcement_status must be an object")
    require(enforcement.get("runtime_readiness_claim") is False, "slides PRD must not claim runtime readiness")
    require(enforcement.get("product_readiness_claim") is False, "slides PRD must not claim product readiness")


def validate_manifest_index_source_lock(index: dict[str, Any]) -> None:
    rows = index.get("microservices")
    require(isinstance(rows, list), "manifests-index microservices must be a list")
    slides_rows = [row for row in rows if isinstance(row, dict) and row.get("name") == "slides"]
    require(len(slides_rows) == 1, f"manifests-index must contain exactly one slides row; got {len(slides_rows)}")
    row = slides_rows[0]
    require(row.get("manifest") == "oya/slides/manifest.json", "slides manifest-index row must point to oya/slides/manifest.json")
    require(row.get("fd001_material") is False, "slides manifest-index row must not be FD-001 material")
    require("inventory/provenance only" in str(row.get("authority_boundary", "")).lower(), "slides authority boundary must state inventory/provenance only")
    require("runtime/product-readiness" in str(row.get("authority_boundary", "")).lower(), "slides authority boundary must deny runtime/product readiness")


def validate_inventory_source_lock(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "slides", "slides inventory manifest microservice must be slides")
    contracts = inventory.get("contracts", {})
    require(isinstance(contracts, dict), "slides inventory contracts must be an object")
    require_contains_all(contracts.get("openapi"), ["microservices/slides/contracts/openapi/slides.yaml"], "slides inventory openapi provenance pointers")
    require_contains_all(contracts.get("asyncapi"), ["microservices/slides/contracts/asyncapi/slides-events.yaml"], "slides inventory asyncapi provenance pointers")
    require_contains_all(contracts.get("proto"), ["microservices/slides/contracts/proto/slides.proto"], "slides inventory proto provenance pointers")
    capability_names = [str(item.get("name")) for item in inventory.get("capabilities", []) if isinstance(item, dict)]
    require_contains_all(capability_names, ["T0-suggest", "T1-assist", "T2-auto"], "slides inventory capability provenance")


def validate_contract_source_files() -> None:
    for family, raw_path in CONTRACT_SOURCE_FILES.items():
        path = REPO_ROOT / raw_path
        require(path.exists(), f"slides {family} source contract must exist at {raw_path}")
        require(path.is_file(), f"slides {family} source contract must be a file: {raw_path}")
        require(not raw_path.endswith(GENERATED_SUFFIX), f"slides {family} source contract must not be a generated face: {raw_path}")
        require(raw_path.startswith("oya/slides/contracts/"), f"slides {family} source contract must live under oya/slides/contracts/: {raw_path}")


def validate_candidate_cases() -> None:
    for expected_id, path in CANDIDATE_CASES.items():
        case = load_json(path, f"candidate fixture {expected_id}")
        require(case.get("fixture_id") == expected_id, f"{rel(path)} fixture_id drifted")
        require(case.get("source_map_ref") == "specs/microservices/slides.json", f"{expected_id} must cite slides source map")
        require(case.get("kanban_task") == "t_7e63e03d", f"{expected_id} must bind to this RED task")
        decision = case.get("expected_checker_decision")
        require(decision in {"METADATA_VALID_REPLAY_STILL_RED", "REJECT"}, f"{expected_id} has unexpected checker decision")

    valid = load_json(CANDIDATE_CASES["valid_slides_source_authority_metadata_fixture"], "valid slides source-authority fixture")
    require_fixture_terms(valid, "must_assert", ["PRD-SLIDES", "oya/slides", "OpenAPI/AsyncAPI/proto", "browser/user-story/accessibility", "AI tier", "retention/audit"], "valid source-authority fixture")
    require_fixture_terms(valid, "must_not_claim", ["runtime readiness", "product readiness", "GA/customer availability", "hyperscaler maturity"], "valid source-authority fixture")

    legacy = load_json(CANDIDATE_CASES["rejects_legacy_microservices_slides_manifest_fixture"], "legacy rejection fixture")
    require_fixture_terms(legacy, "must_reject", ["microservices/slides/manifest.json", "restoration", "legacy microservices/slides"], "legacy rejection fixture")

    readiness = load_json(CANDIDATE_CASES["rejects_runtime_readiness_claim_fixture"], "runtime readiness rejection fixture")
    require_fixture_terms(readiness, "must_reject", ["runtime readiness", "product readiness", "green CI alone", "live SLO"], "runtime readiness rejection fixture")

    foundry = load_json(CANDIDATE_CASES["rejects_retired_foundry_runtime_fixture"], "retired Foundry rejection fixture")
    require_fixture_terms(foundry, "must_reject", ["Foundry runtime", "revive", "intelligence", "retired"], "retired Foundry rejection fixture")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "SLIDES-PRD-RED-FIXTURE-CONTRACT-PLAN-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_7e63e03d", "manifest must bind to kanban task t_7e63e03d")
    require(manifest.get("parent_plan_spec_task") == "t_228dc16e", "manifest must bind to source-authority task t_228dc16e")
    require(manifest.get("review_fix_parent_task") == "t_62dcb74a", "manifest must bind to approved Review/fix task t_62dcb74a")
    require("runtime" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny runtime claims")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")
    require_contains_all(manifest.get("required_fixture_family_ids"), REQUIRED_FIXTURE_FAMILY_IDS, "required_fixture_family_ids")
    require(manifest.get("future_replay_root") == "specs/fixtures/slides/replay/", "future_replay_root must be source-locked")

    replay = manifest.get("contract_replay_expectations")
    require(isinstance(replay, dict), "contract_replay_expectations must be an object")
    require(set(replay) == REQUIRED_REPLAY_KEYS, f"contract_replay_expectations keys must be {sorted(REQUIRED_REPLAY_KEYS)}")
    for key, expected_path in CONTRACT_SOURCE_FILES.items():
        section = replay[key]
        require(isinstance(section, dict), f"contract_replay_expectations.{key} must be an object")
        require(section.get("source_path") == expected_path, f"{key} source_path drifted")
        require(section.get("legacy_manifest_pointer", "").startswith("microservices/slides/contracts/"), f"{key} must record legacy manifest pointer")
        require(isinstance(section.get("must_assert"), list) and len(section["must_assert"]) >= 5, f"{key} must name replay assertions")

    browser = manifest.get("browser_user_story_accessibility_evidence_requirements")
    require(isinstance(browser, list) and len(browser) >= 6, "browser/user-story/accessibility requirements must be a non-empty machine-readable list")
    require_text_terms(browser, ["WCAG 2.2 AA", "keyboard", "screen-reader", "reduced-motion", "Chromium", "Firefox", "WebKit"], "browser/user-story/accessibility requirements")
    for item in browser:
        require(isinstance(item, dict), "each browser/accessibility requirement must be an object")
        require(item.get("journey_id") and item.get("user_story_ref"), "browser/accessibility requirement must name journey_id and user_story_ref")
        require(isinstance(item.get("required_evidence"), list) and item["required_evidence"], "browser/accessibility requirement must name required_evidence")

    boundaries = manifest.get("integration_boundary_expectations")
    require(isinstance(boundaries, list), "integration_boundary_expectations must be a list")
    boundary_services = [str(item.get("service")) for item in boundaries if isinstance(item, dict)]
    require_contains_all(boundary_services, REQUIRED_SERVICE_BOUNDARIES, "integration boundary services")
    require_text_terms(boundaries, ["unavailable", "ACL", "policy", "no readiness evidence"], "integration boundary expectations")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match slides RED plan; got {actual_ids}")
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_replay_artifacts(fixture)
        require_fixture_sources(fixture, ["specs/microservices/slides.json"], fixture_id)

    require_fixture_terms(by_id["slides_prd_authority_source_lock_fixture"], "must_assert", ["PRD-SLIDES", "t_228dc16e", "t_62dcb74a", "oya/slides"], "authority source-lock fixture")
    require_fixture_terms(by_id["slides_deck_crud_open_save_contract_fixture"], "must_assert", ["deck CRUD", "open", "save", "version", "audit"], "deck CRUD fixture")
    require_fixture_sources(by_id["slides_deck_crud_open_save_contract_fixture"], ["openapi", "proto"], "deck CRUD fixture")
    require_fixture_terms(by_id["slides_collaboration_crdt_cursor_conflict_fixture"], "must_assert", ["CRDT", "cursor", "conflict surfaced", "no silent loss", "duplicate op"], "collaboration fixture")
    require_fixture_sources(by_id["slides_collaboration_crdt_cursor_conflict_fixture"], ["asyncapi", "proto"], "collaboration fixture")
    require_fixture_terms(by_id["slides_present_broadcast_contract_fixture"], "must_assert", ["present mode", "broadcast", "LiveKit", "degradation", "audience"], "present/broadcast fixture")
    require_fixture_sources(by_id["slides_present_broadcast_contract_fixture"], ["openapi", "asyncapi"], "present/broadcast fixture")
    require_fixture_terms(by_id["slides_import_export_contract_fixture"], "must_assert", ["PDF", "PPTX", "MP4", "sandbox", "malware", "egress"], "import/export fixture")
    require_fixture_sources(by_id["slides_import_export_contract_fixture"], ["openapi", "asyncapi", "proto"], "import/export fixture")
    require_fixture_terms(by_id["slides_acl_share_version_history_fixture"], "must_assert", ["Cedar", "share", "per-slide ACL", "version history", "legal hold"], "ACL/share/version fixture")
    require_fixture_terms(by_id["slides_accessibility_reduced_motion_fixture"], "must_assert", ["WCAG 2.2 AA", "keyboard", "screen-reader", "reading order", "reduced-motion"], "accessibility fixture")
    require_fixture_terms(by_id["slides_ai_assist_tier_disclosure_fixture"], "must_assert", ["T0", "T1", "T2", "intelligence", "human-review", "watermark"], "AI fixture")
    require_fixture_terms(by_id["slides_data_retention_audit_fixture"], "must_assert", ["data-class", "retention", "legal hold", "audit-chain", "no tenant payload"], "data retention audit fixture")
    require_fixture_terms(by_id["slides_integration_boundaries_fixture"], "must_assert", REQUIRED_SERVICE_BOUNDARIES + ["sibling-unready", "ACL-revoked"], "integration boundaries fixture")
    require_fixture_terms(by_id["slides_browser_user_story_evidence_fixture"], "must_assert", ["create deck", "co-edit", "present", "import PPTX", "per-slide ACL", "alt-text", "browser"], "browser user-story fixture")
    require_fixture_terms(by_id["slides_inventory_provenance_rejection_fixture"], "must_reject", ["oya/slides/manifest.json", "microservices/slides/manifest.json", "legacy microservices/slides"], "inventory rejection fixture")
    require_fixture_terms(by_id["slides_build_parentage_fixture"], "must_assert", ["t_228dc16e", "t_62dcb74a", "t_7e63e03d", "allowed path"], "build parentage fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("green ci alone" in lower_manifest, "manifest must reject green-CI-alone readiness")
    require("microservices/slides/manifest.json restoration" in lower_manifest, "manifest must reject legacy microservices/slides manifest restoration")
    require("foundry runtime" in lower_manifest, "manifest must reject retired Foundry runtime revival")
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
            "RED: future slides OpenAPI/AsyncAPI/proto replay or browser/accessibility evidence artifacts are missing under "
            f"{rel(replay_root)}: {preview}{suffix}"
        )
    for fixture in by_id.values():
        for raw in fixture["future_replay_artifacts"]:
            validate_replay_artifact(fixture, Path(str(raw)))
    print("slides PRD replay artifacts passed")


def validate_replay_artifact(fixture: dict[str, Any], rel_path: Path) -> None:
    artifact = load_json(REPO_ROOT / rel_path, f"replay artifact {rel_path}")
    fixture_id = str(fixture.get("fixture_id"))
    label = f"replay artifact {rel_path}"

    require(artifact.get("parent_fixture_id") == fixture_id, f"{label} must bind parent_fixture_id {fixture_id}")
    require(artifact.get("replay_artifact_path") == str(rel_path), f"{label} replay_artifact_path drifted")
    require(artifact.get("red_gate_task") == "t_7e63e03d", f"{label} must bind RED gate t_7e63e03d")
    require(artifact.get("build_task") == BUILD_TASK, f"{label} must bind Build task {BUILD_TASK}")
    require(artifact.get("status") == REPLAY_PASS_STATUS, f"{label} status must be {REPLAY_PASS_STATUS}")
    require(artifact.get("red_status_closed") == EXPECTED_RED_STATUS, f"{label} must close {EXPECTED_RED_STATUS}")
    require(artifact.get("source_map_ref") == "specs/microservices/slides.json", f"{label} must cite slides PRD source map")

    require_replay_source_paths(artifact, fixture, label)
    require_terms(artifact.get("assertions_covered", []), list(fixture.get("must_assert", [])), f"{label} assertions_covered")
    require_terms(artifact.get("rejections_covered", []), list(fixture.get("must_reject", [])), f"{label} rejections_covered")

    lower_artifact = text(artifact)
    for forbidden in ["passed_after_future_runtime_evidence"]:
        require(forbidden not in lower_artifact, f"{label} must not contain forbidden claim {forbidden!r}")
    for required in ["no runtime readiness", "no generated json hand edits", "green ci alone is not ui readiness"]:
        require(required in lower_artifact, f"{label} non_claims missing {required!r}")
    if "browser" in fixture_id or "accessibility" in fixture_id:
        evidence = artifact.get("browser_user_story_accessibility_evidence", {})
        require(isinstance(evidence, dict), f"{label} must carry browser/accessibility evidence metadata")
        require("runtime_ui_changed" in evidence, f"{label} must carry UI evidence applicability")
        require("no browser/ui runtime path changes" in lower_artifact, f"{label} must explain browser/a11y N/A rationale")


def require_terms(value: object, terms: list[str], label: str) -> None:
    haystack = text(value)
    missing = [term for term in terms if term.lower() not in haystack]
    require(not missing, f"{label} missing terms {missing}")


def require_replay_source_paths(artifact: dict[str, Any], fixture: dict[str, Any], label: str) -> None:
    raw_source_paths = artifact.get("source_paths")
    require(isinstance(raw_source_paths, list) and raw_source_paths, f"{label} must name source_paths")
    source_paths = list(raw_source_paths) if isinstance(raw_source_paths, list) else []
    source_text = text(source_paths)
    require("specs/microservices/slides.json" in source_text, f"{label} must source from specs/microservices/slides.json")
    for source_ref in fixture.get("source_authority_refs", []):
        source_ref_text = str(source_ref)
        if source_ref_text.startswith("kanban:") or source_ref_text == "specs/microservices/slides.json":
            continue
        require(source_ref_text in source_text, f"{label} must source from {source_ref_text}")
    fixture_text = text(fixture)
    if "browser" in fixture_text or "accessibility" in fixture_text:
        require(
            "specs/design-system/" in source_text or "oya/slides/decisions/adr-sld-001" in source_text,
            f"{label} must source UX/a11y design-system or render-pipeline authority",
        )
    if "cedar" in fixture_text or "acl" in fixture_text or "policy" in fixture_text:
        require("oya/slides/" in source_text and ("policy" in source_text or "cedar" in source_text), f"{label} must source policy/Cedar evidence")
    for raw in source_paths:
        source = str(raw).split("#", 1)[0]
        if source.startswith("kanban:"):
            continue
        require(not source.endswith(GENERATED_SUFFIX), f"{label} source path must not be a generated face: {source}")
        require(".." not in Path(source).parts, f"{label} source path must not traverse directories: {source}")
        require((REPO_ROOT / source).exists(), f"{label} source path missing: {source}")


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "fixture_kind": "self_test",
                "source_authority_refs": ["specs/microservices/slides.json"],
                "future_replay_artifacts": [f"specs/fixtures/slides/replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["slides_prd_authority_source_lock_fixture"]["must_assert"] = ["PRD-SLIDES", "t_228dc16e", "t_62dcb74a", "oya/slides"]
    by_id["slides_deck_crud_open_save_contract_fixture"]["source_authority_refs"] += ["oya/slides/contracts/openapi/slides.yaml", "oya/slides/contracts/proto/slides.proto"]
    by_id["slides_deck_crud_open_save_contract_fixture"]["must_assert"] = ["deck CRUD", "open", "save", "version", "audit"]
    by_id["slides_collaboration_crdt_cursor_conflict_fixture"]["source_authority_refs"] += ["oya/slides/contracts/asyncapi/slides-events.yaml", "oya/slides/contracts/proto/slides.proto"]
    by_id["slides_collaboration_crdt_cursor_conflict_fixture"]["must_assert"] = ["CRDT", "cursor", "conflict surfaced", "no silent loss", "duplicate op"]
    by_id["slides_present_broadcast_contract_fixture"]["source_authority_refs"] += ["oya/slides/contracts/openapi/slides.yaml", "oya/slides/contracts/asyncapi/slides-events.yaml"]
    by_id["slides_present_broadcast_contract_fixture"]["must_assert"] = ["present mode", "broadcast", "LiveKit", "degradation", "audience"]
    by_id["slides_import_export_contract_fixture"]["source_authority_refs"] += ["oya/slides/contracts/openapi/slides.yaml", "oya/slides/contracts/asyncapi/slides-events.yaml", "oya/slides/contracts/proto/slides.proto"]
    by_id["slides_import_export_contract_fixture"]["must_assert"] = ["PDF", "PPTX", "MP4", "sandbox", "malware", "egress"]
    by_id["slides_acl_share_version_history_fixture"]["must_assert"] = ["Cedar", "share", "per-slide ACL", "version history", "legal hold"]
    by_id["slides_accessibility_reduced_motion_fixture"]["must_assert"] = ["WCAG 2.2 AA", "keyboard", "screen-reader", "reading order", "reduced-motion"]
    by_id["slides_ai_assist_tier_disclosure_fixture"]["must_assert"] = ["T0", "T1", "T2", "intelligence", "human-review", "watermark"]
    by_id["slides_data_retention_audit_fixture"]["must_assert"] = ["data-class", "retention", "legal hold", "audit-chain", "no tenant payload"]
    by_id["slides_integration_boundaries_fixture"]["must_assert"] = REQUIRED_SERVICE_BOUNDARIES + ["sibling-unready", "ACL-revoked"]
    by_id["slides_browser_user_story_evidence_fixture"]["must_assert"] = ["create deck", "co-edit", "present", "import PPTX", "per-slide ACL", "alt-text", "browser"]
    by_id["slides_inventory_provenance_rejection_fixture"]["must_reject"] = ["oya/slides/manifest.json", "microservices/slides/manifest.json", "legacy microservices/slides"]
    by_id["slides_build_parentage_fixture"]["must_assert"] = ["t_228dc16e", "t_62dcb74a", "t_7e63e03d", "allowed path"]
    return {
        "fixture_plan_id": "SLIDES-PRD-RED-FIXTURE-CONTRACT-PLAN-001",
        "kanban_task": "t_7e63e03d",
        "parent_plan_spec_task": "t_228dc16e",
        "review_fix_parent_task": "t_62dcb74a",
        "claim_boundary": "metadata/fixture-only; no runtime handlers, product readiness, GA, or production claim",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "required_fixture_family_ids": REQUIRED_FIXTURE_FAMILY_IDS[:],
        "contract_replay_expectations": {
            "openapi": {"source_path": "oya/slides/contracts/openapi/slides.yaml", "legacy_manifest_pointer": "microservices/slides/contracts/openapi/slides.yaml", "must_assert": ["a", "b", "c", "d", "e"]},
            "asyncapi": {"source_path": "oya/slides/contracts/asyncapi/slides-events.yaml", "legacy_manifest_pointer": "microservices/slides/contracts/asyncapi/slides-events.yaml", "must_assert": ["a", "b", "c", "d", "e"]},
            "proto": {"source_path": "oya/slides/contracts/proto/slides.proto", "legacy_manifest_pointer": "microservices/slides/contracts/proto/slides.proto", "must_assert": ["a", "b", "c", "d", "e"]},
        },
        "future_replay_root": "specs/fixtures/slides/replay/",
        "browser_user_story_accessibility_evidence_requirements": [
            {"journey_id": "self-test-1", "user_story_ref": "US-SLIDES-001", "required_evidence": ["WCAG 2.2 AA", "keyboard", "screen-reader", "Chromium", "Firefox", "WebKit", "reduced-motion"]},
            {"journey_id": "self-test-2", "user_story_ref": "US-SLIDES-003", "required_evidence": ["keyboard"]},
            {"journey_id": "self-test-3", "user_story_ref": "US-SLIDES-004", "required_evidence": ["keyboard"]},
            {"journey_id": "self-test-4", "user_story_ref": "US-SLIDES-005", "required_evidence": ["keyboard"]},
            {"journey_id": "self-test-5", "user_story_ref": "US-SLIDES-007", "required_evidence": ["keyboard"]},
            {"journey_id": "self-test-6", "user_story_ref": "US-SLIDES-009", "required_evidence": ["keyboard"]},
        ],
        "integration_boundary_expectations": [{"service": service, "must_assert": ["unavailable", "ACL", "policy", "no readiness evidence"]} for service in REQUIRED_SERVICE_BOUNDARIES],
        "global_non_claims": ["green CI alone is insufficient", "no microservices/slides/manifest.json restoration", "no Foundry runtime revival"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_prd_source_lock(load_json(PRD_PATH, "slides PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "slides inventory manifest"))
    validate_contract_source_files()
    validate_candidate_cases()
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
    expect_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/slides/manifest.json"))
    expect_rejected("deck API source missing", lambda data: data["fixtures"][1].update({"source_authority_refs": ["specs/microservices/slides.json", "oya/slides/contracts/openapi/slides.yaml"]}))
    expect_rejected("collaboration no-silent-loss gap", lambda data: data["fixtures"][2].update({"must_assert": ["CRDT"]}))
    expect_rejected("broadcast degradation gap", lambda data: data["fixtures"][3].update({"must_assert": ["present mode"]}))
    expect_rejected("import/export MP4 gap", lambda data: data["fixtures"][4].update({"must_assert": ["PDF", "PPTX"]}))
    expect_rejected("ACL version-history gap", lambda data: data["fixtures"][5].update({"must_assert": ["Cedar"]}))
    expect_rejected("accessibility reduced-motion gap", lambda data: data["fixtures"][6].update({"must_assert": ["WCAG 2.2 AA"]}))
    expect_rejected("AI tier disclosure gap", lambda data: data["fixtures"][7].update({"must_assert": ["T0"]}))
    expect_rejected("integration identity gap", lambda data: data["fixtures"][9].update({"must_assert": ["sheets", "drive"]}))
    expect_rejected("browser evidence not machine-readable", lambda data: data.update({"browser_user_story_accessibility_evidence_requirements": ["keyboard"]}))
    expect_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/slides/replay/bad.generated.json"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("slides PRD RED fixture contract self-tests passed")


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
    validate_prd_source_lock(load_json(PRD_PATH, "slides PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "slides inventory manifest"))
    validate_contract_source_files()
    validate_candidate_cases()
    by_id = validate_manifest(manifest)
    validate_replay_artifacts(by_id, replay_root)


if __name__ == "__main__":
    main()
