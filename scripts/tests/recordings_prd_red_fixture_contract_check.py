#!/usr/bin/env python3
"""Fail-closed RED checker for the recordings PRD fixture/contract replay gate.

This is a metadata/source-lock guard. It validates that the recordings RED
fixture manifest is grounded in Accepted PRD-RECORDINGS, the accepted source-map
review, and the OpenAPI/AsyncAPI/proto contract files, then remains RED until
future Build cards create source-backed replay artifacts.
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
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "recordings-prd" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "recordings-prd" / "replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "recordings.json"
MANIFEST_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
INVENTORY_MANIFEST_PATH = REPO_ROOT / "oya" / "recordings" / "manifest.json"

EXPECTED_FIXTURE_IDS = [
    "recordings_authority_source_lock_fixture",
    "recordings_ingest_contract_replay_fixture",
    "recordings_cedar_scope_playback_search_denials_fixture",
    "recordings_transcript_redaction_overlay_hash_binding_fixture",
    "recordings_retention_legal_hold_mode_matrix_fixture",
    "recordings_export_merkle_four_eyes_fixture",
    "recordings_asyncapi_lifecycle_events_fixture",
    "recordings_proto_ingest_legal_hold_fixture",
    "recordings_pii_dsr_retention_kms_fixture",
    "recordings_translate_handoff_boundary_fixture",
    "recordings_ui_accessibility_evidence_fixture",
    "recordings_inventory_provenance_rejection_fixture",
    "recordings_build_parentage_fixture",
    "recordings_no_implementation_readiness_claim_fixture",
]

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/recordings.json",
    "specs/microservices/manifests-index.json#microservices[name=recordings]",
    "oya/recordings/manifest.json",
    "oya/recordings/contracts/openapi/recordings.yaml",
    "oya/recordings/contracts/asyncapi/recordings-events.yaml",
    "oya/recordings/contracts/proto/recordings.proto",
    "kanban:t_6635c6b5#recordings-source-lock",
    "kanban:t_8ad99810#recordings-source-lock-review",
    "kanban:t_72ac2adf#recordings-red-contract-replay",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/recordings/manifest.json",
    "oya/recordings/catalog/*",
    "oya/recordings/runbooks/*",
    "oya/recordings/slos/*",
    "oya/recordings/policy/*",
    "oya/recordings/dashboards/*",
    "oya/recordings/iac/*",
    "registry/catalog/oya-recordings-domain.yaml",
    "microservices/recordings/manifest.json",
    "microservices/recordings/contracts/*",
    "microservices/recordings/IP-*.md",
}

REQUIRED_AC_IDS = ["AC-REC-01", "AC-REC-02", "AC-REC-03", "AC-REC-04", "AC-REC-05", "AC-REC-06"]
REQUIRED_FIXTURE_FAMILIES = [
    "recordings_ingest_contract_replay",
    "recordings_cedar_scope_playback_search_denials",
    "recordings_transcript_redaction_overlay_hash_binding",
    "recordings_retention_legal_hold_mode_matrix",
    "recordings_export_merkle_four_eyes",
    "recordings_ui_accessibility_journey_replay",
]
REQUIRED_CONTRACT_REPLAY_KEYS = {"openapi", "asyncapi", "proto"}
CONTRACT_SOURCE_FILES = {
    "openapi": "oya/recordings/contracts/openapi/recordings.yaml",
    "asyncapi": "oya/recordings/contracts/asyncapi/recordings-events.yaml",
    "proto": "oya/recordings/contracts/proto/recordings.proto",
}
RUST_INGEST_SOURCE_FILE = "oya/recordings/crates/oya-recordings-domain/src/lib.rs"
RUST_INGEST_SYMBOL_SNIPPETS = [
    "pub struct RecordingIngestRequestCreate",
    "pub struct RecordingIngestRequest",
    "pub struct RecordingIngestReceipt",
    "pub enum RecordingIngestDecision",
    "pub fn classify_idempotent_ingest",
]
EXPECTED_RED_STATUS = "RED_UNTIL_REPLAY_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"

OPENAPI_REQUIRED_SNIPPETS = [
    "/recordings:",
    "/recordings/{recording_id}:",
    "/recordings/{recording_id}/playback-session:",
    "/recordings/{recording_id}/transcript:",
    "/recordings/{recording_id}/redactions:",
    "/search:",
    "/share-links:",
    "/legal-holds:",
    "/legal-holds/{hold_id}/release:",
    "/ediscovery/exports:",
    "/exports:",
    "/ingest/presign:",
    "/ingest/finalize:",
    "X-Oyatie-Tenant",
    "X-Oyatie-Pack",
    "mTLS",
    "content_hash",
    "consent_banner_confirmed",
]
ASYNCAPI_REQUIRED_SNIPPETS = [
    "recordings.recording.ingested.v1",
    "recordings.recording.published.v1",
    "recordings.recording.played.v1",
    "recordings.recording.shared.v1",
    "recordings.recording.redacted.v1",
    "recordings.recording.deleted.v1",
    "recordings.transcript.ready.v1",
    "recordings.translation.ready.v1",
    "recordings.summary.ready.v1",
    "recordings.legalhold.engaged.v1",
    "recordings.legalhold.released.v1",
    "recordings.ediscovery.export.v1",
    "recordings.retention.applied.v1",
    "recordings.watermark.rotated.v1",
    "meet.session.ended.v1",
    "messenger.huddle.ended.v1",
    "consent_banner_confirmed",
    "kms_shred_executed",
    "merkle_root",
]
PROTO_REQUIRED_SNIPPETS = [
    "message RecordingIngestRequest",
    "message RecordingIngestResponse",
    "message LegalHoldEngageRequest",
    "message LegalHold",
    "service RecordingsIngest",
    "service RecordingsLegalHold",
    "rpc Ingest",
    "rpc Engage",
    "idempotency_key",
    "content_hash_verified",
    "producer_spiffe_id",
    "parent_audit_chain_ref",
    "paired_approver",
    "audit_chain_seal_ref",
]


def fail(message: str) -> NoReturn:
    print(f"recordings PRD RED fixture contract check failed: {message}", file=sys.stderr)
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
            artifact.startswith("specs/fixtures/recordings-prd/replay/"),
            f"future replay artifact must stay under specs/fixtures/recordings-prd/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_prd_source_lock(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "recordings PRD _meta must be an object")
    require(meta.get("spec_id") == "PRD-RECORDINGS", "recordings PRD spec_id must be PRD-RECORDINGS")
    require(meta.get("status") == "Accepted", "recordings PRD status must be Accepted")
    require(meta.get("source_lock_task") == "t_6635c6b5", "recordings PRD source_lock_task drifted")
    require(meta.get("review_gate_task") == "t_8ad99810", "recordings PRD review_gate_task drifted")
    identity = prd.get("identity", {})
    require(isinstance(identity, dict), "recordings PRD identity must be an object")
    require(identity.get("product_id") == "recordings", "recordings PRD identity.product_id must be recordings")
    require_contains_all(identity.get("context_model"), ["personal", "work"], "recordings PRD identity.context_model")

    source_authority = prd.get("source_authority", {})
    require(isinstance(source_authority, dict), "recordings PRD source_authority must be an object")
    require(source_authority.get("canonical_prd_path") == "specs/microservices/recordings.json", "canonical PRD path drifted")
    require("No live service".lower() in str(source_authority.get("claim_boundary", "")).lower(), "claim boundary must deny live service readiness")

    acs = prd.get("acceptance_criteria")
    require(isinstance(acs, list), "recordings PRD acceptance_criteria must be a list")
    ac_ids = [str(item.get("id")) for item in acs if isinstance(item, dict)]
    require(ac_ids == REQUIRED_AC_IDS, f"recordings PRD AC ids/order must be {REQUIRED_AC_IDS}; got {ac_ids}")

    plan = prd.get("test_fixture_plan", {})
    require(isinstance(plan, dict), "recordings PRD test_fixture_plan must be an object")
    require(plan.get("red_fixture_contract_card") == "t_72ac2adf", "recordings test_fixture_plan must bind t_72ac2adf")
    fixture_names = [str(item.get("name")) for item in plan.get("required_fixtures", []) if isinstance(item, dict)]
    require(fixture_names == REQUIRED_FIXTURE_FAMILIES, f"recordings PRD fixture family order drifted: {fixture_names}")

    lower_prd = text(prd)
    for term in [
        "content_hash",
        "consent_banner_confirmed",
        "mTLS".lower(),
        "X-Oyatie-Tenant".lower(),
        "X-Oyatie-Pack".lower(),
        "Cedar".lower(),
        "four-eyes",
        "Merkle".lower(),
        "proof-of-erasure",
        "translate handoff",
    ]:
        require(term in lower_prd, f"recordings PRD must retain boundary term {term!r}")
    enforcement = prd.get("enforcement_status", {})
    require(isinstance(enforcement, dict), "recordings PRD enforcement_status must be an object")
    require(enforcement.get("runtime_product_readiness") == "not_claimed", "recordings PRD must not claim runtime product readiness")


def validate_manifest_index_source_lock(index: dict[str, Any]) -> None:
    rows = index.get("microservices")
    require(isinstance(rows, list), "manifests-index microservices must be a list")
    recordings_rows = [row for row in rows if isinstance(row, dict) and row.get("name") == "recordings"]
    require(len(recordings_rows) == 1, f"manifests-index must contain exactly one recordings row; got {len(recordings_rows)}")
    row = recordings_rows[0]
    require(row.get("manifest") == "oya/recordings/manifest.json", "recordings manifest-index row must point to oya/recordings/manifest.json")
    require(row.get("authority_status") == "source-authority-reconciled-by-t_ff8bab02", "recordings authority_status drifted")
    require("inventory/provenance only" in str(row.get("authority_boundary", "")).lower(), "recordings authority boundary must state inventory/provenance only")


def validate_inventory_source_lock(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "recordings", "recordings inventory manifest microservice must be recordings")
    contracts = inventory.get("contracts", {})
    require(isinstance(contracts, dict), "recordings inventory contracts must be an object")
    require_contains_all(contracts.get("openapi"), ["microservices/recordings/contracts/openapi/recordings.yaml"], "recordings inventory openapi provenance pointers")
    require_contains_all(contracts.get("asyncapi"), ["microservices/recordings/contracts/asyncapi/recordings-events.yaml"], "recordings inventory asyncapi provenance pointers")
    require_contains_all(contracts.get("proto"), ["microservices/recordings/contracts/proto/recordings.proto"], "recordings inventory proto provenance pointers")


def validate_contract_source_files() -> None:
    snippets = {
        "openapi": OPENAPI_REQUIRED_SNIPPETS,
        "asyncapi": ASYNCAPI_REQUIRED_SNIPPETS,
        "proto": PROTO_REQUIRED_SNIPPETS,
    }
    for family, raw_path in CONTRACT_SOURCE_FILES.items():
        path = REPO_ROOT / raw_path
        require(path.exists(), f"recordings {family} source contract must exist at {raw_path}")
        require(path.is_file(), f"recordings {family} source contract must be a file: {raw_path}")
        require(not raw_path.endswith(GENERATED_SUFFIX), f"recordings {family} source contract must not be a generated face: {raw_path}")
        require(raw_path.startswith("oya/recordings/contracts/"), f"recordings {family} source contract must live under oya/recordings/contracts/: {raw_path}")
        body = path.read_text(encoding="utf-8")
        missing = [snippet for snippet in snippets[family] if snippet not in body]
        require(not missing, f"recordings {family} source contract missing required replay snippets {missing}")


def validate_rust_ingest_symbols() -> None:
    path = REPO_ROOT / RUST_INGEST_SOURCE_FILE
    require(path.exists(), f"recordings Rust ingest domain source must exist at {RUST_INGEST_SOURCE_FILE}")
    require(path.is_file(), f"recordings Rust ingest domain source must be a file: {RUST_INGEST_SOURCE_FILE}")
    require(
        not RUST_INGEST_SOURCE_FILE.endswith(GENERATED_SUFFIX),
        f"recordings Rust ingest domain source must not be a generated face: {RUST_INGEST_SOURCE_FILE}",
    )
    body = path.read_text(encoding="utf-8")
    missing = [snippet for snippet in RUST_INGEST_SYMBOL_SNIPPETS if snippet not in body]
    require(not missing, f"recordings Rust ingest domain source missing required replay symbols {missing}")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "RECORDINGS-PRD-RED-FIXTURE-CONTRACT-PLAN-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_72ac2adf", "manifest must bind to kanban task t_72ac2adf")
    require(manifest.get("parent_source_lock_task") == "t_6635c6b5", "manifest must bind to source-lock task t_6635c6b5")
    require(manifest.get("review_fix_parent_task") == "t_8ad99810", "manifest must bind to review/fix parent t_8ad99810")
    require("runtime" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny runtime claims")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")
    require_contains_all(manifest.get("required_prd_acceptance_criteria"), REQUIRED_AC_IDS, "required_prd_acceptance_criteria")
    require_contains_all(manifest.get("required_fixture_families_from_source_map"), REQUIRED_FIXTURE_FAMILIES, "required_fixture_families_from_source_map")
    require(manifest.get("future_replay_root") == "specs/fixtures/recordings-prd/replay/", "future_replay_root must be source-locked")

    replay = manifest.get("contract_replay_expectations")
    require(isinstance(replay, dict), "contract_replay_expectations must be an object")
    require(set(replay) == REQUIRED_CONTRACT_REPLAY_KEYS, f"contract_replay_expectations keys must be {sorted(REQUIRED_CONTRACT_REPLAY_KEYS)}")
    for key, expected_path in CONTRACT_SOURCE_FILES.items():
        section = replay[key]
        require(isinstance(section, dict), f"contract_replay_expectations.{key} must be an object")
        require(section.get("source_path") == expected_path, f"{key} source_path drifted")
        require(section.get("legacy_manifest_pointer", "").startswith("microservices/recordings/contracts/"), f"{key} must record legacy manifest pointer")
        require(isinstance(section.get("must_assert"), list) and len(section["must_assert"]) >= 4, f"{key} must name replay assertions")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match recordings RED plan; got {actual_ids}")
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_replay_artifacts(fixture)

    require_fixture_terms(by_id["recordings_authority_source_lock_fixture"], "must_assert", ["PRD-RECORDINGS", "t_6635c6b5", "t_8ad99810", "provenance"], "authority source-lock fixture")
    require_fixture_terms(by_id["recordings_ingest_contract_replay_fixture"], "must_assert", ["tenant_id", "content_hash", "consent_banner_confirmed", "idempotency_key", "producer_spiffe_id", "parent_audit_chain_ref"], "ingest fixture")
    require_fixture_terms(by_id["recordings_cedar_scope_playback_search_denials_fixture"], "must_reject", ["cross-tenant", "missing pack header", "Cedar", "expired share link", "revoked share link", "view cap"], "Cedar/playback/search fixture")
    require_fixture_terms(by_id["recordings_transcript_redaction_overlay_hash_binding_fixture"], "must_reject", ["destructive redaction", "source recording hash", "translation", "summary", "model-training"], "transcript/redaction fixture")
    require_fixture_terms(by_id["recordings_retention_legal_hold_mode_matrix_fixture"], "must_assert", ["worm_record", "purge_on_request", "two-person", "proof-of-erasure", "KMS shred"], "retention/legal-hold fixture")
    require_fixture_terms(by_id["recordings_export_merkle_four_eyes_fixture"], "must_assert", ["eDiscovery", "regular export", "signed URL", "Merkle", "paired approver", "audit-chain"], "export fixture")
    require_fixture_terms(by_id["recordings_asyncapi_lifecycle_events_fixture"], "must_assert", ["recording lifecycle", "transcript/translation/summary", "legal hold", "retention", "meet", "messenger"], "AsyncAPI lifecycle fixture")
    require_fixture_terms(by_id["recordings_proto_ingest_legal_hold_fixture"], "must_assert", ["idempotency_key", "content_hash_verified", "producer_spiffe_id", "parent_audit_chain_ref", "paired_approver", "audit_chain_seal_ref"], "proto fixture")
    require_fixture_terms(by_id["recordings_pii_dsr_retention_kms_fixture"], "must_reject", ["purge tombstone", "transcript text", "speaker quasi identifiers", "KMS shred", "model-training"], "PII/DSR/KMS fixture")
    require_fixture_terms(by_id["recordings_translate_handoff_boundary_fixture"], "must_reject", ["direct cross-product", "translate", "intelligence", "policy basis", "audit seal"], "translate boundary fixture")
    require_fixture_terms(by_id["recordings_ui_accessibility_evidence_fixture"], "must_assert", ["WCAG 2.2 AA", "keyboard", "screen-reader", "KR/en-US", "N/A"], "UI/accessibility fixture")
    require_fixture_terms(by_id["recordings_inventory_provenance_rejection_fixture"], "must_reject", ["oya/recordings/manifest.json", "catalog", "SLO/dashboard", "microservices/recordings/manifest.json", "foundry"], "inventory rejection fixture")
    require_fixture_terms(by_id["recordings_build_parentage_fixture"], "must_assert", ["t_6635c6b5", "t_8ad99810", "t_72ac2adf", "allowed path", "generated JSON"], "build parentage fixture")
    require_fixture_terms(by_id["recordings_no_implementation_readiness_claim_fixture"], "must_assert", ["runtime_product_readiness not_claimed", "release_governance", "generated_files not_touched", "RED guards only"], "no-readiness-claim fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("green ci alone" in lower_manifest, "manifest must reject green-CI-alone UI readiness")
    require("no implementation readiness is claimed by fixtures alone" in lower_manifest, "manifest must preserve fixture-only readiness boundary")
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
            "RED: future recordings PRD OpenAPI/AsyncAPI/proto replay artifacts are missing under "
            f"{rel(replay_root)}: {preview}{suffix}"
        )
    fail("future contract replay is not implemented; this RED-only checker must be extended by a Build card before green status")


def validate_ingest_replay_slice(by_id: dict[str, dict[str, Any]]) -> None:
    fixture = by_id["recordings_ingest_contract_replay_fixture"]
    replay_artifacts = fixture.get("future_replay_artifacts", [])
    require(len(replay_artifacts) == 1, "ingest fixture must name exactly one replay artifact")
    replay_path = REPO_ROOT / str(replay_artifacts[0])
    artifact = load_json(replay_path, "recordings ingest replay artifact")
    require(
        artifact.get("slice_id") == "recordings_ingest_idempotency_domain_slice",
        "ingest replay artifact slice_id drifted",
    )
    require(
        artifact.get("fixture_id") == "recordings_ingest_contract_replay_fixture",
        "ingest replay artifact fixture_id drifted",
    )
    require(
        artifact.get("kanban_task") == "t_d519e221",
        "ingest replay artifact must bind Build card t_d519e221",
    )
    boundary = str(artifact.get("claim_boundary", "")).lower()
    for term in ["bounded", "no http handler", "no grpc server", "no broader recordings"]:
        require(term in boundary, f"ingest replay claim_boundary missing {term!r}")

    require_contains_all(
        artifact.get("source_authority_refs"),
        [
            "specs/microservices/recordings.json#acceptance_criteria[AC-REC-01]",
            "specs/fixtures/recordings-prd/red-fixtures.json#fixtures[recordings_ingest_contract_replay_fixture]",
            "oya/recordings/contracts/openapi/recordings.yaml#/paths/~1ingest~1presign",
            "oya/recordings/contracts/openapi/recordings.yaml#/paths/~1ingest~1finalize",
            "oya/recordings/contracts/asyncapi/recordings-events.yaml#channels/meet.session.ended.v1",
            "oya/recordings/contracts/asyncapi/recordings-events.yaml#channels/messenger.huddle.ended.v1",
            "oya/recordings/contracts/asyncapi/recordings-events.yaml#channels/recordings.recording.ingested.v1",
            "oya/recordings/contracts/proto/recordings.proto#RecordingIngestRequest",
            "oya/recordings/contracts/proto/recordings.proto#RecordingIngestResponse",
            "oya/recordings/contracts/proto/recordings.proto#RecordingsIngest.Ingest",
            "oya/recordings/crates/oya-recordings-domain/src/lib.rs#RecordingIngestRequest",
            "oya/recordings/crates/oya-recordings-domain/src/lib.rs#RecordingIngestReceipt",
            "oya/recordings/crates/oya-recordings-domain/src/lib.rs#classify_idempotent_ingest",
        ],
        "ingest replay source_authority_refs",
    )
    surfaces = artifact.get("replayed_contract_surfaces", {})
    require(isinstance(surfaces, dict), "ingest replay surfaces must be an object")
    require_contains_all(surfaces.get("openapi_paths"), ["POST /ingest/presign", "POST /ingest/finalize"], "ingest replay OpenAPI paths")
    require_contains_all(surfaces.get("asyncapi_channels"), ["meet.session.ended.v1", "messenger.huddle.ended.v1", "recordings.recording.ingested.v1"], "ingest replay AsyncAPI channels")
    require_contains_all(surfaces.get("proto_symbols"), ["oyatie.recordings.v1.RecordingIngestRequest", "oyatie.recordings.v1.RecordingIngestResponse", "RecordingsIngest.Ingest"], "ingest replay proto symbols")
    require_contains_all(surfaces.get("rust_symbols"), ["RecordingIngestRequestCreate", "RecordingIngestRequest", "RecordingIngestReceipt", "RecordingIngestDecision", "classify_idempotent_ingest"], "ingest replay Rust symbols")
    validate_rust_ingest_symbols()
    require_fixture_terms(
        artifact,
        "must_assert",
        ["tenant_id", "source_kind", "content_hash", "context_kind", "consent_banner_confirmed", "cell_id", "parent_audit_chain_ref", "idempotency_key", "producer_spiffe_id", "content_hash_verified", "AlreadyAccepted", "N/A"],
        "ingest replay artifact",
    )
    require_fixture_terms(
        artifact,
        "must_reject",
        ["without consent banner", "without content_hash", "mismatched content hash", "SPIFFE", "parent audit-chain", "HTTP handler", "gRPC server", "UI readiness"],
        "ingest replay artifact",
    )
    slo = artifact.get("observability_slo_evidence", {})
    ui = artifact.get("ui_accessibility_evidence", {})
    require(isinstance(slo, dict) and slo.get("runtime_path") is False, "ingest replay must record SLO N/A")
    require(isinstance(ui, dict) and ui.get("runtime_ui_path") is False, "ingest replay must record UI/accessibility N/A")
    print("recordings PRD ingest replay slice passed")


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "fixture_kind": "self_test",
                "source_authority_refs": ["specs/microservices/recordings.json"],
                "future_replay_artifacts": [f"specs/fixtures/recordings-prd/replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["recordings_authority_source_lock_fixture"]["must_assert"] = ["PRD-RECORDINGS", "t_6635c6b5", "t_8ad99810", "provenance"]
    by_id["recordings_ingest_contract_replay_fixture"]["must_assert"] = ["tenant_id", "content_hash", "consent_banner_confirmed", "idempotency_key", "producer_spiffe_id", "parent_audit_chain_ref"]
    by_id["recordings_cedar_scope_playback_search_denials_fixture"]["must_reject"] = ["cross-tenant", "missing pack header", "Cedar", "expired share link", "revoked share link", "view cap"]
    by_id["recordings_transcript_redaction_overlay_hash_binding_fixture"]["must_reject"] = ["destructive redaction", "source recording hash", "translation", "summary", "model-training"]
    by_id["recordings_retention_legal_hold_mode_matrix_fixture"]["must_assert"] = ["worm_record", "purge_on_request", "two-person", "proof-of-erasure", "KMS shred"]
    by_id["recordings_export_merkle_four_eyes_fixture"]["must_assert"] = ["eDiscovery", "regular export", "signed URL", "Merkle", "paired approver", "audit-chain"]
    by_id["recordings_asyncapi_lifecycle_events_fixture"]["must_assert"] = ["recording lifecycle", "transcript/translation/summary", "legal hold", "retention", "meet", "messenger"]
    by_id["recordings_proto_ingest_legal_hold_fixture"]["must_assert"] = ["idempotency_key", "content_hash_verified", "producer_spiffe_id", "parent_audit_chain_ref", "paired_approver", "audit_chain_seal_ref"]
    by_id["recordings_pii_dsr_retention_kms_fixture"]["must_reject"] = ["purge tombstone", "transcript text", "speaker quasi identifiers", "KMS shred", "model-training"]
    by_id["recordings_translate_handoff_boundary_fixture"]["must_reject"] = ["direct cross-product", "translate", "intelligence", "policy basis", "audit seal"]
    by_id["recordings_ui_accessibility_evidence_fixture"]["must_assert"] = ["WCAG 2.2 AA", "keyboard", "screen-reader", "KR/en-US", "N/A"]
    by_id["recordings_inventory_provenance_rejection_fixture"]["must_reject"] = ["oya/recordings/manifest.json", "catalog", "SLO/dashboard", "microservices/recordings/manifest.json", "foundry"]
    by_id["recordings_build_parentage_fixture"]["must_assert"] = ["t_6635c6b5", "t_8ad99810", "t_72ac2adf", "allowed path", "generated JSON"]
    by_id["recordings_no_implementation_readiness_claim_fixture"]["must_assert"] = ["runtime_product_readiness not_claimed", "release_governance", "generated_files not_touched", "RED guards only"]
    return {
        "fixture_plan_id": "RECORDINGS-PRD-RED-FIXTURE-CONTRACT-PLAN-001",
        "kanban_task": "t_72ac2adf",
        "parent_source_lock_task": "t_6635c6b5",
        "review_fix_parent_task": "t_8ad99810",
        "claim_boundary": "metadata/fixture-only; no runtime handlers or production claim",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "required_prd_acceptance_criteria": REQUIRED_AC_IDS[:],
        "required_fixture_families_from_source_map": REQUIRED_FIXTURE_FAMILIES[:],
        "contract_replay_expectations": {
            "openapi": {"source_path": "oya/recordings/contracts/openapi/recordings.yaml", "legacy_manifest_pointer": "microservices/recordings/contracts/openapi/recordings.yaml", "must_assert": ["a", "b", "c", "d"]},
            "asyncapi": {"source_path": "oya/recordings/contracts/asyncapi/recordings-events.yaml", "legacy_manifest_pointer": "microservices/recordings/contracts/asyncapi/recordings-events.yaml", "must_assert": ["a", "b", "c", "d"]},
            "proto": {"source_path": "oya/recordings/contracts/proto/recordings.proto", "legacy_manifest_pointer": "microservices/recordings/contracts/proto/recordings.proto", "must_assert": ["a", "b", "c", "d"]},
        },
        "future_replay_root": "specs/fixtures/recordings-prd/replay/",
        "global_non_claims": ["green CI alone is insufficient", "No implementation readiness is claimed by fixtures alone"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_prd_source_lock(load_json(PRD_PATH, "recordings PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "recordings inventory manifest"))
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
    expect_rejected("missing AC id", lambda data: data.update({"required_prd_acceptance_criteria": ["AC-REC-01"]}))
    expect_rejected("missing source-map fixture family", lambda data: data.update({"required_fixture_families_from_source_map": ["recordings_ingest_contract_replay"]}))
    expect_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/recordings/manifest.json"))
    expect_rejected("ingest content_hash gap", lambda data: data["fixtures"][1].update({"must_assert": ["tenant_id"]}))
    expect_rejected("Cedar share-link denial gap", lambda data: data["fixtures"][2].update({"must_reject": ["cross-tenant"]}))
    expect_rejected("retention KMS proof gap", lambda data: data["fixtures"][4].update({"must_assert": ["worm_record"]}))
    expect_rejected("API replay sources missing", lambda data: data["contract_replay_expectations"].pop("proto"))
    expect_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/recordings-prd/replay/bad.generated.json"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("recordings PRD RED fixture contract self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST), help="RED fixture manifest JSON path")
    parser.add_argument("--replay-root", default=str(DEFAULT_REPLAY_ROOT), help="future replay artifact root")
    parser.add_argument("--self-test", action="store_true", help="run fail-closed validator self-tests")
    parser.add_argument(
        "--slice",
        choices=["ingest"],
        help="validate a bounded Build replay slice while keeping full normal mode RED until every future artifact exists",
    )
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
    validate_prd_source_lock(load_json(PRD_PATH, "recordings PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "recordings inventory manifest"))
    validate_contract_source_files()
    by_id = validate_manifest(manifest)
    if args.slice == "ingest":
        validate_ingest_replay_slice(by_id)
        return
    validate_replay_artifacts(by_id, replay_root)


if __name__ == "__main__":
    main()
