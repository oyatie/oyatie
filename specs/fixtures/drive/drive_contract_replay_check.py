#!/usr/bin/env python3
"""Fail-closed RED checker for Drive source-map contract replay fixtures.

This is a RED-only metadata/source-lock guard. It validates that the Drive RED
fixture manifest is grounded in specs/microservices/drive.json, treats
oya/drive/manifest.json and legacy microservices/drive/** pointers as
inventory/provenance only, covers the source-mapped OpenAPI/AsyncAPI/proto replay
expectations, and remains RED until future Build cards create source-backed
replay artifacts under specs/fixtures/drive/replay/.
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

REPO_ROOT = Path(__file__).resolve().parents[3]
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "drive" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "drive" / "replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "drive.json"
MANIFEST_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
INVENTORY_MANIFEST_PATH = REPO_ROOT / "oya" / "drive" / "manifest.json"

EXPECTED_FIXTURE_IDS = [
    "drive_authority_source_map_fixture",
    "drive_file_folder_crud_contract_fixture",
    "drive_upload_download_range_multipart_fixture",
    "drive_permissions_acl_inheritance_fixture",
    "drive_share_link_ttl_revocation_view_cap_fixture",
    "drive_preview_sandbox_fixture",
    "drive_dlp_virus_scan_fixture",
    "drive_search_index_authorization_fixture",
    "drive_sync_conflict_resolution_fixture",
    "drive_immutability_legal_hold_fixture",
    "drive_audit_chain_emission_fixture",
    "drive_ontology_projection_lag_fixture",
    "drive_policy_denied_share_download_negative_fixture",
    "drive_cross_tenant_leakage_negative_fixture",
    "drive_stale_scan_negative_fixture",
    "drive_residency_pack_mismatch_negative_fixture",
    "drive_generated_face_no_hand_edit_fixture",
    "drive_build_parentage_fixture",
]

SELECTED_BUILD_FIXTURE_IDS = [
    "drive_file_folder_crud_contract_fixture",
]

SELECTED_REPLAY_STATUS = "GREEN_FOR_SELECTED_BUILD_SLICE"

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/drive.json",
    "specs/microservices/drive.json#source_authority",
    "specs/microservices/manifests-index.json#microservices[name=drive]",
    "oya/drive/manifest.json",
    "kanban:t_3425d88d#drive-source-lock-review",
    "kanban:t_d9d9add5#drive-red-contract-fixtures",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/drive/manifest.json",
    "oya/drive/contracts/*",
    "oya/drive/catalog/*",
    "oya/drive/IPs/*",
    "microservices/drive/contracts/*",
    "microservices/drive/capabilities/*",
    "microservices/drive/IP-*.md",
    "microservices/drive/slos/*",
    "microservices/drive/manifest.json",
}

REQUIRED_AC_IDS = [
    "AC-DRIVE-01",
    "AC-DRIVE-02",
    "AC-DRIVE-03",
    "AC-DRIVE-04",
    "AC-DRIVE-05",
    "AC-DRIVE-06",
    "AC-DRIVE-07",
]

REQUIRED_PRODUCED_CONTRACTS = [
    "drive.file.v1",
    "drive.folder.v1",
    "drive.upload_session.v1",
    "drive.share_link.v1",
    "drive.permission.v1",
    "drive.scan_verdict.v1",
    "drive.sync_delta.v1",
    "drive.preview.v1",
    "drive.immutability_record.v1",
    "audit.drive.policy.v1",
]

CONTRACT_SOURCE_FILES = {
    "openapi": "oya/drive/contracts/openapi/drive.yaml",
    "asyncapi": "oya/drive/contracts/asyncapi/drive-events.yaml",
    "proto": "oya/drive/contracts/proto/drive.proto",
}

LEGACY_CONTRACT_POINTERS = {
    "openapi": "microservices/drive/contracts/openapi/drive.yaml",
    "asyncapi": "microservices/drive/contracts/asyncapi/drive-events.yaml",
    "proto": "microservices/drive/contracts/proto/drive.proto",
}

EXPECTED_RED_STATUS = "RED_UNTIL_DRIVE_REPLAY_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"


def fail(message: str) -> NoReturn:
    print(f"drive RED fixture contract check failed: {message}", file=sys.stderr)
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
            artifact.startswith("specs/fixtures/drive/replay/"),
            f"future replay artifact must stay under specs/fixtures/drive/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_prd_source_lock(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "drive PRD _meta must be an object")
    require(meta.get("spec_id") == "PRD-DRIVE", "drive PRD spec_id must be PRD-DRIVE")
    require(meta.get("status") == "Draft", "drive PRD status must remain Draft until later Build/Review gates")

    source = prd.get("source_authority", {})
    require(isinstance(source, dict), "drive source_authority must be present")
    require(source.get("status") == "source_lock_reviewed_for_red_fixture_fanout", "drive source_authority status drifted")
    require(source.get("canonical_prd_path") == "specs/microservices/drive.json", "canonical PRD path drifted")
    require(source.get("canonical_service_root") == "oya/drive/", "canonical service root must be oya/drive/")
    require(source.get("required_review_gate") == "t_3425d88d", "drive required review gate must stay t_3425d88d")
    require_terms(source, ["inventory/provenance only", "runtime/product-readiness", "red fixture design"], "drive source authority")

    identity = prd.get("identity", {})
    require(isinstance(identity, dict), "drive identity must be an object")
    require(identity.get("product_id") == "drive", "drive identity.product_id must be drive")
    require(identity.get("canonical_service_root") == "oya/drive/", "drive identity canonical_service_root must be oya/drive/")
    require_contains_all(identity.get("context_model"), ["personal", "work", "admin-audit"], "drive context_model")

    acs = prd.get("acceptance_criteria")
    require(isinstance(acs, list), "drive acceptance_criteria must be a list")
    ac_ids = [str(item.get("id")) for item in acs if isinstance(item, dict)]
    require(ac_ids == REQUIRED_AC_IDS, f"drive AC ids/order must be {REQUIRED_AC_IDS}; got {ac_ids}")

    contracts = prd.get("contracts", {})
    require(isinstance(contracts, dict), "drive contracts must be an object")
    require_contains_all(contracts.get("produces"), REQUIRED_PRODUCED_CONTRACTS, "drive produced contracts")
    surfaces = contracts.get("api_surfaces")
    require(isinstance(surfaces, list), "drive contracts.api_surfaces must be a list")
    planned_paths = {str(item.get("planned_path")) for item in surfaces if isinstance(item, dict)}
    require_contains_all(planned_paths, list(CONTRACT_SOURCE_FILES.values()), "drive planned contract paths")

    lower_prd = text(prd)
    for term in [
        "scan-pending quarantine",
        "policy_gated_share_link",
        "immutability_record_anchor",
        "authorization_trimmed_derivative",
        "typed_file_lifecycle_event",
        "direct cross-product database read",
        "legal hold",
        "ontology",
    ]:
        require(term in lower_prd, f"drive PRD must retain source term {term!r}")


def validate_manifest_index_source_lock(index: dict[str, Any]) -> None:
    rows = index.get("microservices")
    require(isinstance(rows, list), "manifests-index microservices must be a list")
    drive_rows = [row for row in rows if isinstance(row, dict) and row.get("name") == "drive"]
    require(len(drive_rows) == 1, f"manifests-index must contain exactly one drive row; got {len(drive_rows)}")
    row = drive_rows[0]
    require(row.get("manifest") == "oya/drive/manifest.json", "drive manifest-index row must point to oya/drive/manifest.json")
    require(row.get("fd001_material") is False, "drive fd001_material must remain false for this RED slice")
    require(row.get("authority_status") == "source-authority-reconciled-by-t_ff8bab02", "drive authority_status drifted")
    boundary = str(row.get("authority_boundary", "")).lower()
    require("inventory/provenance only" in boundary, "drive authority boundary must state inventory/provenance only")
    require("no specs/microservices/drive.json" in boundary, "drive manifest-index stale no-PRD provenance note must remain a known non-blocker")
    require("no runtime/product-readiness claim" in boundary, "drive manifest-index row must deny readiness claims")


def validate_inventory_source_lock(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "drive", "drive inventory manifest microservice must be drive")
    contexts = {str(item.get("name")) for item in inventory.get("bounded_contexts", []) if isinstance(item, dict)}
    require_contains_all(
        contexts,
        ["file-store", "folder-hierarchy", "upload", "sync", "share-link", "permissions", "search-index", "preview", "dlp-virus-scan", "immutability-tier"],
        "drive bounded contexts",
    )
    contracts = inventory.get("contracts", {})
    require(isinstance(contracts, dict), "drive inventory contracts must be an object")
    require_contains_all(contracts.get("openapi"), [LEGACY_CONTRACT_POINTERS["openapi"]], "drive inventory openapi provenance pointers")
    require_contains_all(contracts.get("asyncapi"), [LEGACY_CONTRACT_POINTERS["asyncapi"]], "drive inventory asyncapi provenance pointers")
    require_contains_all(contracts.get("proto"), [LEGACY_CONTRACT_POINTERS["proto"]], "drive inventory proto provenance pointers")


def validate_contract_source_files() -> None:
    for family, raw_path in CONTRACT_SOURCE_FILES.items():
        path = REPO_ROOT / raw_path
        require(path.exists(), f"drive {family} source contract must exist at {raw_path}")
        require(path.is_file(), f"drive {family} source contract must be a file: {raw_path}")
        require(not raw_path.endswith(GENERATED_SUFFIX), f"drive {family} source contract must not be a generated face: {raw_path}")
        require(raw_path.startswith("oya/drive/contracts/"), f"drive {family} source contract must live under oya/drive/contracts/: {raw_path}")
    openapi_text = (REPO_ROOT / CONTRACT_SOURCE_FILES["openapi"]).read_text(encoding="utf-8")
    require_terms(openapi_text, ["/files", "/folders", "/upload/sessions", "/download/{file_id}", "/share-links", "/permissions", "/search", "/preview/{file_id}", "/scan/{file_id}/verdict", "/immutability/files/{file_id}"], "drive OpenAPI paths")
    asyncapi_text = (REPO_ROOT / CONTRACT_SOURCE_FILES["asyncapi"]).read_text(encoding="utf-8")
    require_terms(asyncapi_text, ["drive.file.lifecycle.v1", "drive.share.v1", "drive.permissions.v1", "drive.sync.v1", "drive.scan.v1", "audit.drive.legal_hold.v1", "audit_chain_seal_id"], "drive AsyncAPI channels")
    proto_text = (REPO_ROOT / CONTRACT_SOURCE_FILES["proto"]).read_text(encoding="utf-8")
    require_terms(proto_text, ["service FileStore", "service FolderHierarchy", "service Upload", "service Download", "service Sync", "service ShareLinkService", "service Permissions", "service Search", "service Preview", "service Scan", "service Immutability"], "drive proto services")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "DRIVE-RED-FIXTURE-CONTRACT-PLAN-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_d9d9add5", "manifest must bind to kanban task t_d9d9add5")
    require(manifest.get("review_fix_parent_task") == "t_3425d88d", "manifest must bind to review parent t_3425d88d")
    require("runtime" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny runtime claims")
    require("production readiness" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny production readiness")
    require("generated json hand edits" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must deny generated JSON hand edits")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")
    require_contains_all(manifest.get("required_prd_acceptance_criteria"), REQUIRED_AC_IDS, "required_prd_acceptance_criteria")
    require_contains_all(manifest.get("required_produced_contracts"), REQUIRED_PRODUCED_CONTRACTS, "required_produced_contracts")

    replay = manifest.get("contract_replay_expectations")
    require(isinstance(replay, dict), "contract_replay_expectations must be an object")
    require(set(replay) == set(CONTRACT_SOURCE_FILES), f"contract_replay_expectations keys must be {sorted(CONTRACT_SOURCE_FILES)}")
    for key, expected_path in CONTRACT_SOURCE_FILES.items():
        section = replay[key]
        require(isinstance(section, dict), f"contract_replay_expectations.{key} must be an object")
        require(section.get("source_path") == expected_path, f"{key} source_path drifted")
        require(section.get("legacy_manifest_pointer") == LEGACY_CONTRACT_POINTERS[key], f"{key} legacy_manifest_pointer drifted")
        require_terms(section.get("must_assert", []), ["replay"], f"{key} replay assertions")
        require_terms(section.get("must_reject", []), ["legacy", "before"], f"{key} replay negative cases")

    require(manifest.get("future_replay_root") == "specs/fixtures/drive/replay/", "future_replay_root must be source-locked")
    require_terms(manifest.get("browser_user_story_accessibility_gate", []), ["WCAG", "keyboard", "N/A", "green CI alone"], "browser/user-story/accessibility gate")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match drive RED plan; got {actual_ids}")
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_replay_artifacts(fixture)

    require_fixture_sources(by_id["drive_authority_source_map_fixture"], ["specs/microservices/drive.json", "manifests-index", "oya/drive/manifest.json"], "authority source-map fixture")
    require_fixture_terms(by_id["drive_authority_source_map_fixture"], "must_assert", ["PRD-DRIVE", "source_lock_reviewed_for_red_fixture_fanout", "inventory/provenance only"], "authority source-map fixture")
    require_fixture_terms(by_id["drive_file_folder_crud_contract_fixture"], "must_assert", ["tenant/context", "content_hash", "audit-chain"], "file/folder CRUD fixture")
    require_fixture_terms(by_id["drive_upload_download_range_multipart_fixture"], "must_assert", ["multipart", "Upload-Offset", "206 Content-Range", "signed URL ttl_seconds"], "upload/download fixture")
    require_fixture_terms(by_id["drive_permissions_acl_inheritance_fixture"], "must_reject", ["cross-tenant grant", "Cedar policy basis", "personal/work context"], "permissions fixture")
    require_fixture_terms(by_id["drive_share_link_ttl_revocation_view_cap_fixture"], "must_reject", ["expired share link", "revoked share link", "view cap exhausted", "policy-denied share"], "share-link fixture")
    require_fixture_terms(by_id["drive_preview_sandbox_fixture"], "must_reject", ["unreviewed preview sandbox escape", "preview before scan verdict", "thumbnail side-channel"], "preview fixture")
    require_fixture_terms(by_id["drive_dlp_virus_scan_fixture"], "must_reject", ["stale virus/DLP scan", "malicious file visible", "DLP flagged file searchable"], "DLP/virus fixture")
    require_fixture_terms(by_id["drive_search_index_authorization_fixture"], "must_reject", ["cross-tenant search hit", "unauthorized snippet", "personal/work context leakage"], "search fixture")
    require_fixture_terms(by_id["drive_sync_conflict_resolution_fixture"], "must_assert", ["deterministic conflict artifacts", "both object versions", "SyncConflictDetected"], "sync fixture")
    require_fixture_terms(by_id["drive_immutability_legal_hold_fixture"], "must_reject", ["legal-hold deletion", "retention-expiry", "release without two_person_approver_id"], "immutability/legal hold fixture")
    require_fixture_terms(by_id["drive_audit_chain_emission_fixture"], "must_assert", ["DrivePolicyDenied", "audit_chain_seal_id", "signature_ed25519"], "audit-chain fixture")
    require_fixture_terms(by_id["drive_ontology_projection_lag_fixture"], "must_assert", ["File projection lag_budget_seconds 60", "LegalHold projection lag_budget_seconds 60"], "ontology fixture")
    require_fixture_terms(by_id["drive_policy_denied_share_download_negative_fixture"], "must_reject", ["policy-denied share/download", "signed_url", "audit ref"], "policy-denied negative fixture")
    require_fixture_terms(by_id["drive_cross_tenant_leakage_negative_fixture"], "must_reject", ["cross-tenant leakage", "tenant B", "sync cursor crosses tenant"], "cross-tenant negative fixture")
    require_fixture_terms(by_id["drive_stale_scan_negative_fixture"], "must_reject", ["stale virus/DLP scan", "scan-pending preview", "quarantined download"], "stale scan negative fixture")
    require_fixture_terms(by_id["drive_residency_pack_mismatch_negative_fixture"], "must_reject", ["residency/pack mismatch", "pack_tag missing", "regional policy mismatch ignored"], "residency/pack fixture")
    require_fixture_terms(by_id["drive_generated_face_no_hand_edit_fixture"], "must_reject", ["*.generated.json", "hand edit"], "generated face fixture")
    require_fixture_terms(by_id["drive_build_parentage_fixture"], "must_assert", ["t_3425d88d", "t_d9d9add5", "allowed path", "generated-face"], "build parentage fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("green ci alone is insufficient" in lower_manifest, "manifest must reject green-CI-alone UI readiness")
    require("microservices/drive/manifest.json" in lower_manifest, "manifest must explicitly keep retired drive manifest path non-authoritative")
    return by_id


def validate_selected_replay_artifact(fixture_id: str, path: Path, artifact: dict[str, Any]) -> None:
    require(artifact.get("fixture_id") == fixture_id, f"{rel(path)} fixture_id must be {fixture_id}")
    require(
        artifact.get("status") == SELECTED_REPLAY_STATUS,
        f"{rel(path)} status must be {SELECTED_REPLAY_STATUS}",
    )
    require(artifact.get("selected_by") == "DRIVE-RED-001", f"{rel(path)} must name DRIVE-RED-001")
    require(artifact.get("build_task") == "t_13abe2e5", f"{rel(path)} must bind to Build task t_13abe2e5")

    implementation_refs = artifact.get("implementation_refs")
    if not isinstance(implementation_refs, list) or not implementation_refs:
        fail(f"{rel(path)} implementation_refs missing")
    for raw in implementation_refs:
        implementation_ref = str(raw)
        require(
            implementation_ref.startswith("oya/drive/"),
            f"{rel(path)} implementation ref must stay under oya/drive/: {implementation_ref}",
        )
        require(
            not implementation_ref.endswith(GENERATED_SUFFIX),
            f"{rel(path)} implementation ref must not be generated JSON: {implementation_ref}",
        )

    contract_refs = artifact.get("contract_refs")
    if not isinstance(contract_refs, dict):
        fail(f"{rel(path)} contract_refs must be an object")
    require_terms(
        contract_refs,
        ["/files", "/folders", "FileStore", "FolderHierarchy", "drive.file.lifecycle.v1"],
        f"{rel(path)} contract refs",
    )
    require_terms(
        artifact.get("must_assert", []),
        [
            "tenant/context",
            "content_hash",
            "object_version_ref",
            "scan-pending",
            "scan verdict records audit evidence",
            "soft-delete",
            "legal-hold denial",
            "legal-hold",
            "audit-chain",
        ],
        f"{rel(path)} assertions",
    )
    require_terms(
        artifact.get("must_reject", []),
        ["file visible before scan verdict", "legal-hold delete succeeds", "microservices/drive"],
        f"{rel(path)} negative assertions",
    )
    require_terms(
        artifact.get("verification_commands", []),
        ["cargo test -p oya-drive-domain", fixture_id, "drive_contract_replay_check.py"],
        f"{rel(path)} verification commands",
    )
    require_terms(
        artifact.get("non_claims", []),
        ["No upload/download runtime", "No browser UI readiness claim", "No production readiness"],
        f"{rel(path)} non-claims",
    )


def validate_replay_artifacts(by_id: dict[str, dict[str, Any]], replay_root: Path) -> None:
    missing_selected: list[str] = []
    selected_paths: list[tuple[str, Path]] = []
    deferred_missing: list[str] = []
    for fixture_id, fixture in by_id.items():
        for raw in fixture["future_replay_artifacts"]:
            rel_path = Path(str(raw))
            expected = REPO_ROOT / rel_path
            if fixture_id in SELECTED_BUILD_FIXTURE_IDS:
                selected_paths.append((fixture_id, expected))
                if not expected.exists():
                    missing_selected.append(str(rel_path))
            elif not expected.exists():
                deferred_missing.append(str(rel_path))
    if missing_selected:
        fail(
            "RED: selected Drive Build replay artifacts are missing under "
            f"{rel(replay_root)}: {', '.join(missing_selected)}"
        )

    for fixture_id, path in selected_paths:
        artifact = load_json(path, f"selected Drive replay artifact {fixture_id}")
        validate_selected_replay_artifact(fixture_id, path, artifact)

    if deferred_missing:
        preview = ", ".join(deferred_missing[:8])
        suffix = "" if len(deferred_missing) <= 8 else f" ... (+{len(deferred_missing) - 8} more)"
        print(
            "drive selected Build replay passed; deferred RED fixtures remain backlog under "
            f"{rel(replay_root)}: {preview}{suffix}"
        )
        return
    print("drive selected Build replay passed; all RED replay artifacts are present")


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "fixture_kind": "self_test",
                "source_authority_refs": ["specs/microservices/drive.json"],
                "future_replay_artifacts": [f"specs/fixtures/drive/replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["drive_authority_source_map_fixture"].update({"source_authority_refs": ["specs/microservices/drive.json", "specs/microservices/manifests-index.json#microservices[name=drive]", "oya/drive/manifest.json"], "must_assert": ["PRD-DRIVE", "source_lock_reviewed_for_red_fixture_fanout", "inventory/provenance only"]})
    by_id["drive_file_folder_crud_contract_fixture"]["must_assert"] = ["tenant/context", "content_hash", "audit-chain"]
    by_id["drive_upload_download_range_multipart_fixture"]["must_assert"] = ["multipart", "Upload-Offset", "206 Content-Range", "signed URL ttl_seconds"]
    by_id["drive_permissions_acl_inheritance_fixture"]["must_reject"] = ["cross-tenant grant", "Cedar policy basis", "personal/work context"]
    by_id["drive_share_link_ttl_revocation_view_cap_fixture"]["must_reject"] = ["expired share link", "revoked share link", "view cap exhausted", "policy-denied share"]
    by_id["drive_preview_sandbox_fixture"]["must_reject"] = ["unreviewed preview sandbox escape", "preview before scan verdict", "thumbnail side-channel"]
    by_id["drive_dlp_virus_scan_fixture"]["must_reject"] = ["stale virus/DLP scan", "malicious file visible", "DLP flagged file searchable"]
    by_id["drive_search_index_authorization_fixture"]["must_reject"] = ["cross-tenant search hit", "unauthorized snippet", "personal/work context leakage"]
    by_id["drive_sync_conflict_resolution_fixture"]["must_assert"] = ["deterministic conflict artifacts", "both object versions", "SyncConflictDetected"]
    by_id["drive_immutability_legal_hold_fixture"]["must_reject"] = ["legal-hold deletion", "retention-expiry", "release without two_person_approver_id"]
    by_id["drive_audit_chain_emission_fixture"]["must_assert"] = ["DrivePolicyDenied", "audit_chain_seal_id", "signature_ed25519"]
    by_id["drive_ontology_projection_lag_fixture"]["must_assert"] = ["File projection lag_budget_seconds 60", "LegalHold projection lag_budget_seconds 60"]
    by_id["drive_policy_denied_share_download_negative_fixture"]["must_reject"] = ["policy-denied share/download", "signed_url", "audit ref"]
    by_id["drive_cross_tenant_leakage_negative_fixture"]["must_reject"] = ["cross-tenant leakage", "tenant B", "sync cursor crosses tenant"]
    by_id["drive_stale_scan_negative_fixture"]["must_reject"] = ["stale virus/DLP scan", "scan-pending preview", "quarantined download"]
    by_id["drive_residency_pack_mismatch_negative_fixture"]["must_reject"] = ["residency/pack mismatch", "pack_tag missing", "regional policy mismatch ignored"]
    by_id["drive_generated_face_no_hand_edit_fixture"]["must_reject"] = ["*.generated.json", "hand edit"]
    by_id["drive_build_parentage_fixture"]["must_assert"] = ["t_3425d88d", "t_d9d9add5", "allowed path", "generated-face"]
    return {
        "fixture_plan_id": "DRIVE-RED-FIXTURE-CONTRACT-PLAN-001",
        "kanban_task": "t_d9d9add5",
        "review_fix_parent_task": "t_3425d88d",
        "claim_boundary": "metadata/fixture-only; no runtime handlers or production readiness claim; no generated JSON hand edits",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "required_prd_acceptance_criteria": REQUIRED_AC_IDS[:],
        "required_produced_contracts": REQUIRED_PRODUCED_CONTRACTS[:],
        "contract_replay_expectations": {
            "openapi": {"source_path": CONTRACT_SOURCE_FILES["openapi"], "legacy_manifest_pointer": LEGACY_CONTRACT_POINTERS["openapi"], "must_assert": ["replay"], "must_reject": ["legacy", "before"]},
            "asyncapi": {"source_path": CONTRACT_SOURCE_FILES["asyncapi"], "legacy_manifest_pointer": LEGACY_CONTRACT_POINTERS["asyncapi"], "must_assert": ["replay"], "must_reject": ["legacy", "before"]},
            "proto": {"source_path": CONTRACT_SOURCE_FILES["proto"], "legacy_manifest_pointer": LEGACY_CONTRACT_POINTERS["proto"], "must_assert": ["replay"], "must_reject": ["legacy", "before"]},
        },
        "future_replay_root": "specs/fixtures/drive/replay/",
        "browser_user_story_accessibility_gate": ["WCAG", "keyboard", "N/A", "green CI alone"],
        "global_non_claims": ["microservices/drive/manifest.json", "green CI alone is insufficient"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_prd_source_lock(load_json(PRD_PATH, "drive PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "drive inventory manifest"))
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
    expect_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/drive/manifest.json"))
    expect_rejected("OpenAPI replay source missing", lambda data: data["contract_replay_expectations"].pop("openapi"))
    expect_rejected("share-link negative gap", lambda data: data["fixtures"][4].update({"must_reject": ["expired share link"]}))
    expect_rejected("preview sandbox negative gap", lambda data: data["fixtures"][5].update({"must_reject": ["preview before scan verdict"]}))
    expect_rejected("legal-hold negative gap", lambda data: data["fixtures"][9].update({"must_reject": ["legal-hold deletion"]}))
    expect_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/drive/replay/bad.generated.json"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("drive RED fixture contract self-tests passed")


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
    validate_prd_source_lock(load_json(PRD_PATH, "drive PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "drive inventory manifest"))
    validate_contract_source_files()
    by_id = validate_manifest(manifest)
    validate_replay_artifacts(by_id, replay_root)


if __name__ == "__main__":
    main()
