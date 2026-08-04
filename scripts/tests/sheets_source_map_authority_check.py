#!/usr/bin/env python3
"""Fail-closed RED checker for the sheets source-map fixture/contract replay gate.

This is a metadata/source-lock guard. It validates that the sheets RED fixture
manifest is grounded in the accepted source map, manifest-index source lock, and
sheets inventory provenance, then remains RED until future Build cards create
source-backed OpenAPI/AsyncAPI/proto replay fixtures.
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
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "sheets" / "source-map" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "sheets" / "source-map" / "replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "sheets.json"
MANIFEST_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
INVENTORY_MANIFEST_PATH = REPO_ROOT / "oya" / "sheets" / "manifest.json"
CANDIDATE_CASES = {
    "valid_sheets_source_map_metadata_fixture": REPO_ROOT
    / "specs"
    / "fixtures"
    / "sheets"
    / "source-map"
    / "valid-sheets-source-map.json",
    "rejects_legacy_microservices_manifest_fixture": REPO_ROOT
    / "specs"
    / "fixtures"
    / "sheets"
    / "source-map"
    / "rejects-legacy-microservices-manifest.json",
    "rejects_runtime_readiness_claim_fixture": REPO_ROOT
    / "specs"
    / "fixtures"
    / "sheets"
    / "source-map"
    / "rejects-runtime-readiness-claim.json",
}

EXPECTED_FIXTURE_IDS = [
    "sheets_source_map_authority_lock_fixture",
    "sheets_workbook_session_contract_fixture",
    "sheets_cell_range_formula_contract_fixture",
    "sheets_collaboration_conflict_contract_fixture",
    "sheets_import_export_contract_fixture",
    "sheets_security_data_policy_fixture",
    "sheets_browser_accessibility_evidence_fixture",
    "sheets_inventory_provenance_rejection_fixture",
    "sheets_retired_ai_intelligence_boundary_fixture",
    "sheets_build_parentage_fixture",
]

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/sheets.json",
    "specs/microservices/manifests-index.json#microservices[name=sheets]",
    "oya/sheets/manifest.json",
    "kanban:t_0820889c#sheets-source-lock-spec",
    "kanban:t_0b31b98c#review-fix-sheets-source-lock",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/sheets/manifest.json",
    "oya/sheets/contracts/*",
    "oya/sheets/catalog/*",
    "oya/sheets/IPs/*",
    "oya/sheets/crates/*",
    "microservices/sheets/manifest.json",
    "microservices/sheets/contracts/*",
    "dirty oya/sheets/** inventory",
}

REQUIRED_REPLAY_KEYS = {"openapi", "asyncapi", "proto"}
CONTRACT_SOURCE_FILES = {
    "openapi": "oya/sheets/contracts/openapi/sheets.yaml",
    "asyncapi": "oya/sheets/contracts/asyncapi/sheets-events.yaml",
    "proto": "oya/sheets/contracts/proto/sheets.proto",
}
EXPECTED_RED_STATUS = "RED_UNTIL_REPLAY_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"
BUILD_TASK = "t_10bf9a15"
REPLAY_PASS_STATUS = "SOURCE_BACKED_REPLAY_PASS"


def fail(message: str) -> NoReturn:
    print(f"sheets source-map RED fixture contract check failed: {message}", file=sys.stderr)
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
            artifact.startswith("specs/fixtures/sheets/source-map/replay/"),
            f"future replay artifact must stay under specs/fixtures/sheets/source-map/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_source_map(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "sheets source map _meta must be an object")
    require(meta.get("spec_id") == "PRD-SHEETS-SOURCE-MAP", "sheets source map spec_id must be PRD-SHEETS-SOURCE-MAP")
    require(meta.get("status") == "Accepted", "sheets source map status must be Accepted")
    require(meta.get("authority_lock_task") == "t_0820889c", "sheets source map must bind to source-lock task t_0820889c")

    authority = prd.get("authority_resolution", {})
    require(isinstance(authority, dict), "sheets authority_resolution must be an object")
    require(authority.get("current_authority") == "specs/microservices/sheets.json", "sheets current authority path drifted")
    require(authority.get("review_gate_task") == "t_0b31b98c", "sheets review gate task must be t_0b31b98c")
    require("no handler" in text(authority.get("claim_boundary", "")), "claim boundary must deny runtime readiness")

    identity = prd.get("identity", {})
    require(isinstance(identity, dict), "sheets identity must be an object")
    require(identity.get("product_id") == "sheets", "sheets identity.product_id must be sheets")
    require(identity.get("user_facing_surface") is True, "sheets user_facing_surface must be true")
    require(identity.get("fd001_material") is False, "sheets must not claim FD-001 materiality")
    require_contains_all(identity.get("context_model"), ["work", "tenant-scoped collaboration"], "sheets identity.context_model")

    source_map = prd.get("source_inventory_map", {})
    require(isinstance(source_map, dict), "sheets source_inventory_map must be an object")
    manifest_source = source_map.get("manifest_source", {})
    require(isinstance(manifest_source, dict), "manifest_source must be an object")
    require(manifest_source.get("status") == "inventory_provenance_only", "manifest source must remain inventory/provenance only")
    require_contains_all(
        manifest_source.get("must_not_be_used_as"),
        ["implementation-readiness evidence", "runtime-readiness evidence", "product-readiness evidence"],
        "manifest_source.must_not_be_used_as",
    )

    evidence = prd.get("evidence_expectations", {})
    require(isinstance(evidence, dict), "evidence_expectations must be an object")
    replay = evidence.get("api_contract_replay", {})
    require(isinstance(replay, dict), "api_contract_replay must be an object")
    require_contains_all(replay.get("surfaces"), list(CONTRACT_SOURCE_FILES.values()), "api_contract_replay.surfaces")
    for term in [
        "open workbook session",
        "read/write cell and range",
        "formula authoring",
        "CRDT/collaboration merge",
        "XLSX import/export",
        "connected-query refresh",
    ]:
        require(term.lower() in text(replay.get("minimum_scenarios", [])), f"api replay scenarios must include {term!r}")

    browser = evidence.get("browser_user_story_accessibility", {})
    require(isinstance(browser, dict), "browser_user_story_accessibility must be an object")
    for term in ["keyboard", "formula", "collaboration", "protected range", "import/export", "screen-reader"]:
        require(term.lower() in text(browser.get("minimum_scenarios", [])), f"browser/accessibility scenarios must include {term!r}")

    policy = evidence.get("security_data_policy", {})
    require(isinstance(policy, dict), "security_data_policy must be an object")
    for term in ["Cedar", "data-class", "legal hold", "T2 automation"]:
        require(term.lower() in text(policy.get("minimum_scenarios", [])), f"security/data policy scenarios must include {term!r}")

    red_plan = prd.get("red_fixture_plan", {})
    require(isinstance(red_plan, dict), "red_fixture_plan must be an object")
    require(red_plan.get("red_gate_task") == "t_48991095", "red fixture plan must bind this task")
    require_contains_all(
        red_plan.get("candidate_fixture_paths"),
        [
            "specs/fixtures/sheets/source-map/valid-sheets-source-map.json",
            "specs/fixtures/sheets/source-map/rejects-legacy-microservices-manifest.json",
            "specs/fixtures/sheets/source-map/rejects-runtime-readiness-claim.json",
            "scripts/tests/sheets_source_map_authority_check.py",
        ],
        "red_fixture_plan.candidate_fixture_paths",
    )


def validate_manifest_index_source_lock(index: dict[str, Any]) -> None:
    rows = index.get("microservices")
    require(isinstance(rows, list), "manifests-index microservices must be a list")
    sheets_rows = [row for row in rows if isinstance(row, dict) and row.get("name") == "sheets"]
    require(len(sheets_rows) == 1, f"manifests-index must contain exactly one sheets row; got {len(sheets_rows)}")
    row = sheets_rows[0]
    require(row.get("manifest") == "oya/sheets/manifest.json", "sheets manifest-index row must point to oya/sheets/manifest.json")
    require(row.get("fd001_material") is False, "sheets manifest-index row must not be FD-001 material")
    require(row.get("authority_status") == "source-authority-reconciled-by-t_ff8bab02", "sheets authority_status drifted")
    require("inventory/provenance only" in str(row.get("authority_boundary", "")).lower(), "sheets authority boundary must state inventory/provenance only")
    require("runtime/product-readiness" in str(row.get("authority_boundary", "")).lower(), "sheets authority boundary must deny runtime/product readiness")


def validate_inventory_source_lock(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "sheets", "sheets inventory manifest microservice must be sheets")
    contracts = inventory.get("contracts", {})
    require(isinstance(contracts, dict), "sheets inventory contracts must be an object")
    require_contains_all(contracts.get("openapi"), ["microservices/sheets/contracts/openapi/sheets.yaml"], "sheets inventory openapi provenance pointers")
    require_contains_all(contracts.get("asyncapi"), ["microservices/sheets/contracts/asyncapi/sheets-events.yaml"], "sheets inventory asyncapi provenance pointers")
    require_contains_all(contracts.get("proto"), ["microservices/sheets/contracts/proto/sheets.proto"], "sheets inventory proto provenance pointers")
    tiers = {str(item.get("tier")) for item in inventory.get("capabilities", []) if isinstance(item, dict)}
    require({"T0", "T1", "T2"}.issubset(tiers), "sheets inventory must retain T0/T1/T2 capability provenance")


def validate_contract_source_files() -> None:
    for family, raw_path in CONTRACT_SOURCE_FILES.items():
        path = REPO_ROOT / raw_path
        require(path.exists(), f"sheets {family} source contract must exist at {raw_path}")
        require(path.is_file(), f"sheets {family} source contract must be a file: {raw_path}")
        require(not raw_path.endswith(GENERATED_SUFFIX), f"sheets {family} source contract must not be a generated face: {raw_path}")
        require(raw_path.startswith("oya/sheets/contracts/"), f"sheets {family} source contract must live under oya/sheets/contracts/: {raw_path}")


def validate_candidate_cases() -> None:
    for expected_id, path in CANDIDATE_CASES.items():
        case = load_json(path, f"candidate fixture {expected_id}")
        require(case.get("fixture_id") == expected_id, f"{rel(path)} fixture_id drifted")
        require(case.get("source_map_ref") == "specs/microservices/sheets.json", f"{expected_id} must cite sheets source map")
        require(case.get("kanban_task") == "t_48991095", f"{expected_id} must bind to this RED task")
        decision = case.get("expected_checker_decision")
        require(decision in {"METADATA_VALID_REPLAY_STILL_RED", "REJECT"}, f"{expected_id} has unexpected checker decision")

    valid = load_json(CANDIDATE_CASES["valid_sheets_source_map_metadata_fixture"], "valid sheets source-map fixture")
    require_fixture_terms(
        valid,
        "must_assert",
        ["PRD-SHEETS-SOURCE-MAP", "oya/sheets", "OpenAPI/AsyncAPI/proto", "browser/user-story/accessibility", "security/data-policy"],
        "valid source-map fixture",
    )
    require_fixture_terms(
        valid,
        "must_not_claim",
        ["runtime readiness", "product readiness", "FD-001 materiality", "hyperscaler maturity"],
        "valid source-map fixture",
    )

    legacy = load_json(CANDIDATE_CASES["rejects_legacy_microservices_manifest_fixture"], "legacy rejection fixture")
    require_fixture_terms(legacy, "must_reject", ["microservices/sheets/manifest.json", "restoration", "legacy microservices/sheets"], "legacy rejection fixture")

    readiness = load_json(CANDIDATE_CASES["rejects_runtime_readiness_claim_fixture"], "runtime readiness rejection fixture")
    require_fixture_terms(readiness, "must_reject", ["runtime readiness", "product readiness", "green CI alone", "live SLO"], "runtime readiness rejection fixture")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "SHEETS-SOURCE-MAP-RED-FIXTURE-CONTRACT-PLAN-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_48991095", "manifest must bind to kanban task t_48991095")
    require(manifest.get("parent_plan_spec_task") == "t_0820889c", "manifest must bind to parent source-lock task t_0820889c")
    require(manifest.get("review_fix_parent_task") == "t_0b31b98c", "manifest must bind to source-lock Review/fix task t_0b31b98c")
    require("runtime" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny runtime claims")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")
    require(manifest.get("future_replay_root") == "specs/fixtures/sheets/source-map/replay/", "future_replay_root must be source-locked")

    replay = manifest.get("contract_replay_expectations")
    require(isinstance(replay, dict), "contract_replay_expectations must be an object")
    require(set(replay) == REQUIRED_REPLAY_KEYS, f"contract_replay_expectations keys must be {sorted(REQUIRED_REPLAY_KEYS)}")
    for key, expected_path in CONTRACT_SOURCE_FILES.items():
        section = replay[key]
        require(isinstance(section, dict), f"contract_replay_expectations.{key} must be an object")
        require(section.get("source_path") == expected_path, f"{key} source_path drifted")
        require(section.get("legacy_manifest_pointer", "").startswith("microservices/sheets/contracts/"), f"{key} must record legacy manifest pointer")
        require(isinstance(section.get("must_assert"), list) and len(section["must_assert"]) >= 4, f"{key} must name replay assertions")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match sheets RED plan; got {actual_ids}")
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_replay_artifacts(fixture)

    require_fixture_terms(by_id["sheets_source_map_authority_lock_fixture"], "must_assert", ["PRD-SHEETS-SOURCE-MAP", "t_0820889c", "t_0b31b98c", "oya/sheets"], "authority source-lock fixture")
    require_fixture_terms(by_id["sheets_workbook_session_contract_fixture"], "must_assert", ["workbook session", "tenant/RBAC", "license decision", "audit_chain_seal"], "workbook session fixture")
    require_fixture_terms(by_id["sheets_cell_range_formula_contract_fixture"], "must_assert", ["cell/range", "formula", "recalculation", "dependency graph", "data_class"], "cell/range/formula fixture")
    require_fixture_terms(by_id["sheets_collaboration_conflict_contract_fixture"], "must_assert", ["CRDT", "conflict-surfaced", "no silent data loss", "conflict visibility"], "collaboration fixture")
    require_fixture_terms(by_id["sheets_import_export_contract_fixture"], "must_assert", ["XLSX", "AV scanning", "sandbox", "ACL-aware export masking"], "import/export fixture")
    require_fixture_terms(by_id["sheets_security_data_policy_fixture"], "must_assert", ["tenant/RBAC", "Cedar", "data-class", "legal hold", "T2 automation", "audit-chain"], "security/data/policy fixture")
    require_fixture_terms(by_id["sheets_browser_accessibility_evidence_fixture"], "must_assert", ["keyboard-only", "grid navigation", "formula error", "WCAG 2.2 AA"], "browser/accessibility fixture")
    require_fixture_terms(by_id["sheets_inventory_provenance_rejection_fixture"], "must_reject", ["oya/sheets/manifest.json", "microservices/sheets/manifest.json", "dirty oya/sheets tree"], "inventory rejection fixture")
    require_fixture_terms(by_id["sheets_retired_ai_intelligence_boundary_fixture"], "must_reject", ["foundry-runtime", "T2 auto", "consent_acknowledged", "prompt injection", "PII"], "retired AI/intelligence fixture")
    require_fixture_terms(by_id["sheets_build_parentage_fixture"], "must_assert", ["t_0820889c", "t_0b31b98c", "t_48991095", "allowed path"], "build parentage fixture")
    require_fixture_sources(by_id["sheets_workbook_session_contract_fixture"], ["openapi", "proto"], "workbook session fixture")
    require_fixture_sources(by_id["sheets_collaboration_conflict_contract_fixture"], ["asyncapi", "proto"], "collaboration fixture")
    require_fixture_sources(by_id["sheets_import_export_contract_fixture"], ["openapi", "asyncapi"], "import/export fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("green-ci-alone" in lower_manifest or "green ci alone" in lower_manifest, "manifest must reject green-CI-alone UI readiness")
    require("microservices/sheets/manifest.json restoration" in lower_manifest, "manifest must reject legacy microservices/sheets manifest restoration")
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
            "RED: future sheets OpenAPI/AsyncAPI/proto replay artifacts are missing under "
            f"{rel(replay_root)}: {preview}{suffix}"
        )
    for fixture in by_id.values():
        for raw in fixture["future_replay_artifacts"]:
            validate_replay_artifact(fixture, Path(str(raw)))
    print("sheets source-map replay artifacts passed")


def validate_replay_artifact(fixture: dict[str, Any], rel_path: Path) -> None:
    artifact = load_json(REPO_ROOT / rel_path, f"replay artifact {rel_path}")
    fixture_id = str(fixture.get("fixture_id"))
    label = f"replay artifact {rel_path}"

    require(artifact.get("parent_fixture_id") == fixture_id, f"{label} must bind parent_fixture_id {fixture_id}")
    require(artifact.get("replay_artifact_path") == str(rel_path), f"{label} replay_artifact_path drifted")
    require(artifact.get("red_gate_task") == "t_48991095", f"{label} must bind RED gate t_48991095")
    require(artifact.get("build_task") == BUILD_TASK, f"{label} must bind Build task {BUILD_TASK}")
    require(artifact.get("status") == REPLAY_PASS_STATUS, f"{label} status must be {REPLAY_PASS_STATUS}")
    require(artifact.get("red_status_closed") == EXPECTED_RED_STATUS, f"{label} must close {EXPECTED_RED_STATUS}")
    require(artifact.get("source_map_ref") == "specs/microservices/sheets.json", f"{label} must cite sheets source map")

    require_replay_source_paths(artifact, rel_path, label)
    require_terms(artifact.get("assertions_covered", []), list(fixture.get("must_assert", [])), f"{label} assertions_covered")
    require_terms(artifact.get("rejections_covered", []), list(fixture.get("must_reject", [])), f"{label} rejections_covered")

    lower_artifact = text(artifact)
    for forbidden in ["passed_after_future_runtime_evidence"]:
        require(forbidden not in lower_artifact, f"{label} must not contain forbidden claim {forbidden!r}")
    for required in ["no runtime readiness", "no generated json hand edits", "green ci alone is not ui readiness"]:
        require(required in lower_artifact, f"{label} non_claims missing {required!r}")
    if fixture.get("fixture_kind") == "browser_user_story_accessibility_contract":
        require("runtime_ui_changed" in artifact.get("browser_user_story_accessibility_evidence", {}), f"{label} must carry UI evidence applicability")
        require("no browser/ui runtime path changes" in lower_artifact, f"{label} must explain browser/a11y N/A rationale")


def require_terms(value: object, terms: list[str], label: str) -> None:
    haystack = text(value)
    missing = [term for term in terms if term.lower() not in haystack]
    require(not missing, f"{label} missing terms {missing}")


def require_replay_source_paths(artifact: dict[str, Any], rel_path: Path, label: str) -> None:
    raw_source_paths = artifact.get("source_paths")
    require(isinstance(raw_source_paths, list) and raw_source_paths, f"{label} must name source_paths")
    source_paths = list(raw_source_paths) if isinstance(raw_source_paths, list) else []
    source_text = text(source_paths)
    require("specs/microservices/sheets.json" in source_text, f"{label} must source from specs/microservices/sheets.json")
    family = rel_path.parts[-2]
    if family in CONTRACT_SOURCE_FILES:
        require(CONTRACT_SOURCE_FILES[family] in source_text, f"{label} must source from {CONTRACT_SOURCE_FILES[family]}")
    if family == "ux":
        require("specs/design-system/" in source_text, f"{label} must source UX/a11y design-system evidence")
    if family == "policy":
        require("oya/sheets/" in source_text and ("policy" in source_text or "cedar" in source_text), f"{label} must source policy/Cedar evidence")
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
                "source_authority_refs": ["specs/microservices/sheets.json"],
                "future_replay_artifacts": [f"specs/fixtures/sheets/source-map/replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["sheets_source_map_authority_lock_fixture"]["must_assert"] = ["PRD-SHEETS-SOURCE-MAP", "t_0820889c", "t_0b31b98c", "oya/sheets"]
    by_id["sheets_workbook_session_contract_fixture"]["source_authority_refs"] = ["oya/sheets/contracts/openapi/sheets.yaml", "oya/sheets/contracts/proto/sheets.proto"]
    by_id["sheets_workbook_session_contract_fixture"]["must_assert"] = ["workbook session", "tenant/RBAC", "license decision", "audit_chain_seal"]
    by_id["sheets_cell_range_formula_contract_fixture"]["must_assert"] = ["cell/range", "formula", "recalculation", "dependency graph", "data_class"]
    by_id["sheets_collaboration_conflict_contract_fixture"]["source_authority_refs"] = ["oya/sheets/contracts/asyncapi/sheets-events.yaml", "oya/sheets/contracts/proto/sheets.proto"]
    by_id["sheets_collaboration_conflict_contract_fixture"]["must_assert"] = ["CRDT", "conflict-surfaced", "no silent data loss", "conflict visibility"]
    by_id["sheets_import_export_contract_fixture"]["source_authority_refs"] = ["oya/sheets/contracts/openapi/sheets.yaml", "oya/sheets/contracts/asyncapi/sheets-events.yaml"]
    by_id["sheets_import_export_contract_fixture"]["must_assert"] = ["XLSX", "AV scanning", "sandbox", "ACL-aware export masking"]
    by_id["sheets_security_data_policy_fixture"]["must_assert"] = ["tenant/RBAC", "Cedar", "data-class", "legal hold", "T2 automation", "audit-chain"]
    by_id["sheets_browser_accessibility_evidence_fixture"]["must_assert"] = ["keyboard-only", "grid navigation", "formula error", "WCAG 2.2 AA"]
    by_id["sheets_inventory_provenance_rejection_fixture"]["must_reject"] = ["oya/sheets/manifest.json", "microservices/sheets/manifest.json", "dirty oya/sheets tree"]
    by_id["sheets_retired_ai_intelligence_boundary_fixture"]["must_reject"] = ["foundry-runtime", "T2 auto", "consent_acknowledged", "prompt injection", "PII"]
    by_id["sheets_build_parentage_fixture"]["must_assert"] = ["t_0820889c", "t_0b31b98c", "t_48991095", "allowed path"]
    return {
        "fixture_plan_id": "SHEETS-SOURCE-MAP-RED-FIXTURE-CONTRACT-PLAN-001",
        "kanban_task": "t_48991095",
        "parent_plan_spec_task": "t_0820889c",
        "review_fix_parent_task": "t_0b31b98c",
        "claim_boundary": "metadata/fixture-only; no runtime handlers or production claim",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "contract_replay_expectations": {
            "openapi": {"source_path": "oya/sheets/contracts/openapi/sheets.yaml", "legacy_manifest_pointer": "microservices/sheets/contracts/openapi/sheets.yaml", "must_assert": ["a", "b", "c", "d"]},
            "asyncapi": {"source_path": "oya/sheets/contracts/asyncapi/sheets-events.yaml", "legacy_manifest_pointer": "microservices/sheets/contracts/asyncapi/sheets-events.yaml", "must_assert": ["a", "b", "c", "d"]},
            "proto": {"source_path": "oya/sheets/contracts/proto/sheets.proto", "legacy_manifest_pointer": "microservices/sheets/contracts/proto/sheets.proto", "must_assert": ["a", "b", "c", "d"]},
        },
        "future_replay_root": "specs/fixtures/sheets/source-map/replay/",
        "global_non_claims": ["green CI alone is insufficient", "no microservices/sheets/manifest.json restoration"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_source_map(load_json(PRD_PATH, "sheets source map"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "sheets inventory manifest"))
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
    expect_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/sheets/manifest.json"))
    expect_rejected("workbook session source missing", lambda data: data["fixtures"][1].update({"source_authority_refs": ["oya/sheets/contracts/openapi/sheets.yaml"]}))
    expect_rejected("cell formula coverage gap", lambda data: data["fixtures"][2].update({"must_assert": ["cell/range"]}))
    expect_rejected("collab no-silent-loss gap", lambda data: data["fixtures"][3].update({"must_assert": ["CRDT"]}))
    expect_rejected("import/export AV gap", lambda data: data["fixtures"][4].update({"must_assert": ["XLSX"]}))
    expect_rejected("security data policy gap", lambda data: data["fixtures"][5].update({"must_assert": ["Cedar"]}))
    expect_rejected("browser accessibility gap", lambda data: data["fixtures"][6].update({"must_assert": ["keyboard-only"]}))
    expect_rejected("retired AI boundary gap", lambda data: data["fixtures"][8].update({"must_reject": ["foundry-runtime"]}))
    expect_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/sheets/source-map/replay/bad.generated.json"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("sheets source-map RED fixture contract self-tests passed")


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
    validate_source_map(load_json(PRD_PATH, "sheets source map"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "sheets inventory manifest"))
    validate_contract_source_files()
    validate_candidate_cases()
    by_id = validate_manifest(manifest)
    validate_replay_artifacts(by_id, replay_root)


if __name__ == "__main__":
    main()
