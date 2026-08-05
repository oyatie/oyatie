#!/usr/bin/env python3
"""Fail-closed RED checker for the Social PRD fixture/contract replay gate.

This is a metadata/source-lock guard for RED-SOCIAL-002. It validates that the
social RED fixture manifest is grounded in the locked Draft social PRD/source map,
current oya/social inventory provenance, and current OpenAPI/AsyncAPI/proto paths.
It remains RED until future Build cards create source-backed replay fixtures and
runtime implementation under the locked oya/social home.
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
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "social" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "social" / "replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "social.json"
MANIFEST_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
INVENTORY_MANIFEST_PATH = REPO_ROOT / "oya" / "social" / "manifest.json"
ADR_0334_PATH = REPO_ROOT / "docs" / "decisions" / "ADR-0334-shorts-microservice-merged-into-social.md"

EXPECTED_FIXTURE_IDS = [
    "social_prd_authority_source_lock_fixture",
    "social_prd_ac01_personal_post_pillar_fixture",
    "social_prd_ac02_context_switch_reconstructs_surfaces_fixture",
    "social_prd_ac03_story_ttl_purge_fixture",
    "social_prd_ac04_professional_crosspost_consent_fixture",
    "social_prd_ac05_ar_no_biometric_persistence_fixture",
    "social_prd_ac06_collab_ownership_consent_fixture",
    "social_prd_produced_contracts_fixture",
    "social_prd_api_contract_replay_fixture",
    "social_prd_security_privacy_policy_fixture",
    "social_prd_browser_accessibility_evidence_fixture",
    "social_prd_retired_authority_rejection_fixture",
    "social_prd_build_parentage_fixture",
]

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/social.json",
    "specs/microservices/social.json#authority_resolution",
    "specs/microservices/social.json#acceptance_criteria",
    "specs/microservices/social.json#contracts.produces",
    "specs/microservices/manifests-index.json#microservices[name=social]",
    "oya/social/manifest.json#source_authority",
    "docs/decisions/ADR-0334-shorts-microservice-merged-into-social.md",
    "kanban:t_df502234#PLAN/SPEC-SOCIAL-001",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/social/manifest.json",
    "oya/social/contracts/*",
    "oya/social/catalog/*",
    "oya/social/IPs/*",
    "microservices/social/manifest.json",
    "microservices/social/IP-001..IP-015*.md",
    "microservices/shorts/*",
    "Foundry active authority",
}

REQUIRED_AC_IDS = [
    "AC-SOCIAL-01",
    "AC-SOCIAL-02",
    "AC-SOCIAL-03",
    "AC-SOCIAL-04",
    "AC-SOCIAL-05",
    "AC-SOCIAL-06",
]
REQUIRED_PRODUCED_CONTRACTS = [
    "community.social.post.v1",
    "community.social.story.v1",
    "community.social.engagement.v1",
    "community.social.moderation_decision.v1",
]
REQUIRED_CONTRACT_REPLAY_KEYS = {"openapi", "asyncapi", "proto"}
CONTRACT_SOURCE_FILES = {
    "openapi": "oya/social/contracts/openapi/social.yaml",
    "asyncapi": "oya/social/contracts/asyncapi/social-events.yaml",
    "proto": "oya/social/contracts/proto/social.proto",
}
IMPLEMENTATION_SOURCE_FILES = {
    "cargo_manifest": "oya/social/crates/oya-social-domain/Cargo.toml",
    "domain_lib": "oya/social/crates/oya-social-domain/src/lib.rs",
    "domain_tests": "oya/social/crates/oya-social-domain/tests/social_workplace_slice.rs",
}
EXPECTED_RED_STATUS = "RED_UNTIL_SOCIAL_REPLAY_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"


def fail(message: str) -> NoReturn:
    print(f"social PRD RED fixture contract check failed: {message}", file=sys.stderr)
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
            artifact.startswith("specs/fixtures/social/replay/"),
            f"future replay artifact must stay under specs/fixtures/social/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_prd_source_lock(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "social PRD _meta must be an object")
    require(meta.get("spec_id") == "PRD-SOCIAL", "social PRD spec_id must be PRD-SOCIAL")
    require(meta.get("status") == "Draft", "social PRD status must remain Draft for RED planning")
    require(meta.get("authority_lock_task") == "t_df502234", "social authority lock task drifted")

    authority = prd.get("authority_resolution", {})
    require(isinstance(authority, dict), "authority_resolution must be an object")
    require(authority.get("current_authority") == "/specs/microservices/social.json", "social current_authority drifted")
    require("draft retained" in str(authority.get("prd_status_disposition", "")).lower(), "authority must retain Draft PRD boundary")
    manifest_inventory_state = str(authority.get("manifest_inventory_state", ""))
    require("microservices/social/manifest.json" in manifest_inventory_state, "authority must record legacy social manifest disposition")
    require("must not be restored" in manifest_inventory_state.lower(), "legacy social manifest must not be restored")
    retired_mapping = authority.get("retired_authority_mapping", {})
    require(isinstance(retired_mapping, dict), "retired_authority_mapping must be an object")
    require("intelligence" in str(retired_mapping.get("foundry", {})).lower(), "retired Foundry disposition must route through intelligence")

    acs = prd.get("acceptance_criteria")
    require(isinstance(acs, list), "social PRD acceptance_criteria must be a list")
    ac_ids = [str(item.get("id")) for item in acs if isinstance(item, dict)]
    require(ac_ids == REQUIRED_AC_IDS, f"social PRD AC ids/order must be {REQUIRED_AC_IDS}; got {ac_ids}")

    contracts = prd.get("contracts", {})
    require(isinstance(contracts, dict), "social PRD contracts must be an object")
    require_contains_all(contracts.get("produces"), REQUIRED_PRODUCED_CONTRACTS, "social PRD produced contracts")

    isolation = prd.get("dual_context_isolation", {})
    require(isinstance(isolation, dict), "social dual_context_isolation must be an object")
    for key in ["separate_auth_realms", "separate_audit_chains", "no_cross_feed", "no_cross_search", "no_cross_suggest"]:
        require(isolation.get(key) is True, f"dual_context_isolation.{key} must remain true")

    lower_prd = text(prd)
    for term in [
        "context_kind=personal",
        "ownership_pillar=personal",
        "story artifact is purged",
        "tombstone audit event",
        "SocialContentPublished",
        "explicit consent token",
        "biometric data derived from face-detection is never persisted",
        "collaborative artifact is owned by personal pillar",
        "Cedar policy denies any cross-pillar read or join",
        "personal social notifications never route to work notification channels",
        "KR",
        "US",
        "EU",
    ]:
        require(term.lower() in lower_prd, f"social PRD must retain term {term!r}")


def validate_manifest_index_source_lock(index: dict[str, Any]) -> None:
    rows = index.get("microservices")
    require(isinstance(rows, list), "manifests-index microservices must be a list")
    social_rows = [row for row in rows if isinstance(row, dict) and row.get("name") == "social"]
    require(len(social_rows) == 1, f"manifests-index must contain exactly one social row; got {len(social_rows)}")
    row = social_rows[0]
    require(row.get("manifest") == "oya/social/manifest.json", "social manifest-index row must point to oya/social/manifest.json")
    require(row.get("prd") == "specs/microservices/social.json", "social manifest-index row must point to specs/microservices/social.json")
    require(row.get("authority_status") == "source-map-locked-by-t_df502234", "social authority_status drifted")
    boundary = str(row.get("authority_boundary", "")).lower()
    require("draft prd/source-authority" in boundary, "social authority boundary must retain Draft PRD/source-authority wording")
    require("inventory/provenance only" in boundary, "social authority boundary must mark inventory/provenance only")
    require("no runtime/product-readiness claim" in boundary, "social authority boundary must deny runtime/product-readiness claim")


def validate_inventory_source_lock(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "social", "social inventory manifest microservice must be social")
    source = inventory.get("source_authority", {})
    require(isinstance(source, dict), "social inventory source_authority must be an object")
    require(source.get("prd") == "specs/microservices/social.json", "social inventory source_authority.prd drifted")
    require(source.get("prd_status") == "Draft-retained-by-t_df502234", "social inventory prd_status drifted")
    require(source.get("manifest_index_row") == "specs/microservices/manifests-index.json#microservices[name=social]", "social manifest_index_row drifted")
    require("not runtime or product-readiness authority" in str(source.get("manifest_pointer_state", "")).lower(), "inventory must reject runtime/product readiness authority")
    require("must not be restored" in str(source.get("legacy_root_disposition", "")).lower(), "legacy social root must not be restored")
    require("boundary-consistency input only" in str(source.get("community_boundary", "")).lower(), "community boundary must stay consistency-only")
    retired_guard = source.get("retired_authority_guard", {})
    require(isinstance(retired_guard, dict), "retired_authority_guard must be an object")
    require("intelligence" in str(retired_guard.get("foundry", "")).lower(), "Foundry successor must be intelligence")
    require("ADR-0334" in str(retired_guard.get("shorts", "")), "shorts retirement must cite ADR-0334")

    contracts = inventory.get("contracts", {})
    require(isinstance(contracts, dict), "social inventory contracts must be an object")
    require_contains_all(contracts.get("openapi"), [CONTRACT_SOURCE_FILES["openapi"]], "social inventory openapi pointers")
    require_contains_all(contracts.get("asyncapi"), [CONTRACT_SOURCE_FILES["asyncapi"]], "social inventory asyncapi pointers")
    require_contains_all(contracts.get("proto"), [CONTRACT_SOURCE_FILES["proto"]], "social inventory proto pointers")


def validate_contract_source_files() -> None:
    for family, raw_path in CONTRACT_SOURCE_FILES.items():
        path = REPO_ROOT / raw_path
        require(path.exists(), f"social {family} source contract must exist at {raw_path}")
        require(path.is_file(), f"social {family} source contract must be a file: {raw_path}")
        require(not raw_path.endswith(GENERATED_SUFFIX), f"social {family} source contract must not be a generated face: {raw_path}")
        require(raw_path.startswith("oya/social/contracts/"), f"social {family} source contract must live under oya/social/contracts/: {raw_path}")


def validate_service_local_first_slice() -> None:
    for label, raw_path in IMPLEMENTATION_SOURCE_FILES.items():
        path = REPO_ROOT / raw_path
        require(path.exists(), f"social first-slice {label} must exist at {raw_path}")
        require(path.is_file(), f"social first-slice {label} must be a file: {raw_path}")
        require(raw_path.startswith("oya/social/"), f"social first-slice {label} must stay under oya/social/: {raw_path}")
        require(not raw_path.endswith(GENERATED_SUFFIX), f"social first-slice {label} must not be generated JSON: {raw_path}")
    lib = (REPO_ROOT / IMPLEMENTATION_SOURCE_FILES["domain_lib"]).read_text(encoding="utf-8")
    for term in [
        "pub enum ContextKind",
        "pub enum OwnershipPillar",
        "pub struct SocialPost",
        "pub struct SocialFeedIndex",
        "to_post_published_event",
        "community.social.post.v1",
        "SocialContentPublished",
        "ShortVideo",
        "retired_standalone_shorts_authority",
    ]:
        require(term in lib, f"social first-slice implementation missing {term!r}")
    tests = (REPO_ROOT / IMPLEMENTATION_SOURCE_FILES["domain_tests"]).read_text(encoding="utf-8")
    for term in [
        "personal_post_pillar_is_immutable",
        "professional_feed",
        "professional_search",
        "ontology",
        "post_contract_replay_emits_context_pillar_idempotency",
        "short_video_is_a_social_media_flavor",
    ]:
        require(term in tests, f"social first-slice tests missing {term!r}")


def validate_adr_0334() -> None:
    try:
        content = ADR_0334_PATH.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing ADR-0334: {rel(ADR_0334_PATH)}")
    lower = content.lower()
    for term in ["shorts µservice retired", "absorbed into social", "same `post` aggregate", "`media.kind = short_video`", "old shorts contracts", "new contracts bind through social"]:
        require(term.lower() in lower, f"ADR-0334 must retain term {term!r}")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "SOCIAL-PRD-RED-FIXTURE-CONTRACT-PLAN-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_b70fe698", "manifest must bind to kanban task t_b70fe698")
    require(manifest.get("parent_plan_spec_task") == "t_df502234", "manifest must bind to parent Plan/Spec task t_df502234")
    claim_boundary = str(manifest.get("claim_boundary", "")).lower()
    for term in ["runtime", "generated json", "production readiness", "ui readiness"]:
        require(term in claim_boundary, f"claim_boundary must explicitly deny {term}")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")
    require_contains_all(manifest.get("required_prd_acceptance_criteria"), REQUIRED_AC_IDS, "required_prd_acceptance_criteria")
    require_contains_all(manifest.get("required_produced_contracts"), REQUIRED_PRODUCED_CONTRACTS, "required_produced_contracts")
    require(manifest.get("future_replay_root") == "specs/fixtures/social/replay/", "future_replay_root must be source-locked")

    replay = manifest.get("contract_replay_expectations")
    require(isinstance(replay, dict), "contract_replay_expectations must be an object")
    require(set(replay) == REQUIRED_CONTRACT_REPLAY_KEYS, f"contract_replay_expectations keys must be {sorted(REQUIRED_CONTRACT_REPLAY_KEYS)}")
    for key, expected_path in CONTRACT_SOURCE_FILES.items():
        section = replay[key]
        require(isinstance(section, dict), f"contract_replay_expectations.{key} must be an object")
        require(section.get("source_path") == expected_path, f"{key} source_path drifted")
        require(isinstance(section.get("must_assert"), list) and len(section["must_assert"]) >= 4, f"{key} must name replay assertions")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match social RED plan; got {actual_ids}")
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_replay_artifacts(fixture)

    require_fixture_terms(by_id["social_prd_authority_source_lock_fixture"], "must_assert", ["PRD-SOCIAL", "t_df502234", "inventory/provenance", "microservices/social/manifest.json"], "authority source-lock fixture")
    require_fixture_terms(by_id["social_prd_authority_source_lock_fixture"], "must_reject", ["runtime readiness", "t_b70fe698", "community FD-001"], "authority source-lock fixture")
    require_fixture_terms(by_id["social_prd_ac01_personal_post_pillar_fixture"], "must_assert", ["context_kind=personal", "ownership_pillar=personal", "professional feed", "professional search", "Ontology work-graph"], "AC-SOCIAL-01 fixture")
    require_fixture_terms(by_id["social_prd_ac02_context_switch_reconstructs_surfaces_fixture"], "must_assert", ["TenantContextSwitcher", "feed", "stories ring", "close-friends", "AR camera", "zero cross-context"], "AC-SOCIAL-02 fixture")
    require_fixture_terms(by_id["social_prd_ac03_story_ttl_purge_fixture"], "must_assert", ["ttl_expires_at", "CDN", "search index", "Ontology graph", "tombstone audit"], "AC-SOCIAL-03 fixture")
    require_fixture_terms(by_id["social_prd_ac04_professional_crosspost_consent_fixture"], "must_assert", ["SocialContentPublished", "consent token", "professional context", "personal context is never consulted", "Workflow/Ontology"], "AC-SOCIAL-04 fixture")
    require_fixture_terms(by_id["social_prd_ac05_ar_no_biometric_persistence_fixture"], "must_assert", ["non-sensitive annotation", "in-session only", "face geometry never leaves device", "server payload excludes biometric", "GDPR Art.9"], "AC-SOCIAL-05 fixture")
    require_fixture_terms(by_id["social_prd_ac06_collab_ownership_consent_fixture"], "must_assert", ["ownership_pillar=personal", "explicit cross-user consent", "audit logged", "collab_owner_ids"], "AC-SOCIAL-06 fixture")
    require_fixture_terms(by_id["social_prd_produced_contracts_fixture"], "must_assert", REQUIRED_PRODUCED_CONTRACTS, "produced-contract fixture")
    require_fixture_sources(by_id["social_prd_api_contract_replay_fixture"], ["openapi", "asyncapi", "proto"], "API contract replay fixture")
    require_fixture_terms(by_id["social_prd_security_privacy_policy_fixture"], "must_assert", ["Cedar deny-by-default", "personal content is never employer-visible", "KR", "US", "EU"], "security/privacy fixture")
    require_fixture_terms(by_id["social_prd_browser_accessibility_evidence_fixture"], "must_assert", ["personal post isolation", "work context switch", "story TTL", "AR no-biometric", "professional cross-post consent", "WCAG 2.2 AA"], "browser/accessibility fixture")
    require_fixture_terms(by_id["social_prd_retired_authority_rejection_fixture"], "must_reject", ["Foundry", "standalone shorts", "microservices/shorts", "short-video implementation outside social"], "retired authority fixture")
    require_fixture_terms(by_id["social_prd_retired_authority_rejection_fixture"], "allowed_successors", ["intelligence", "Workflow", "Ontology", "social short-video flavor"], "retired authority fixture")
    require_fixture_terms(by_id["social_prd_build_parentage_fixture"], "must_assert", ["t_df502234", "t_b70fe698", "oya/social/**", "no generated JSON", "microservices/social/manifest.json", "standalone shorts"], "build parentage fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("green ci alone" in lower_manifest, "manifest must reject green-CI-alone UI readiness")
    require("generated json hand edits" in lower_manifest, "manifest must reject generated JSON hand edits")
    return by_id


def validate_replay_payload(fixture: dict[str, Any], artifact: str, payload: dict[str, Any]) -> None:
    fixture_id = str(fixture["fixture_id"])
    require(payload.get("fixture_id") == fixture_id, f"{artifact} fixture_id must be {fixture_id}")
    require(payload.get("kanban_task") == "t_4c158581", f"{artifact} must bind to Build task t_4c158581")
    status = payload.get("replay_status")
    require(
        status in {"passed_first_social_workplace_slice", "deferred_not_claimed_for_first_slice"},
        f"{artifact} has invalid replay_status {status!r}",
    )
    claim_boundary = str(payload.get("claim_boundary", "")).lower()
    for term in ["no production readiness", "no hyperscaler", "no ui readiness"]:
        require(term in claim_boundary, f"{artifact} claim_boundary must include {term!r}")
    impl_refs = payload.get("implementation_refs", [])
    require(isinstance(impl_refs, list), f"{artifact} implementation_refs must be a list")
    for raw_ref in impl_refs:
        ref = str(raw_ref)
        require(ref.startswith("oya/social/"), f"{artifact} implementation ref must stay under oya/social/: {ref}")
        require(not ref.endswith(GENERATED_SUFFIX), f"{artifact} implementation ref must not be generated JSON: {ref}")
    assertions = text(payload.get("replay_assertions", []))
    rejections = text(payload.get("replay_rejections", []))
    if fixture_id == "social_prd_ac01_personal_post_pillar_fixture":
        require(status == "passed_first_social_workplace_slice", f"{artifact} AC-01 must pass in first slice")
        for term in ["context_kind=personal", "ownership_pillar=personal", "professional feed", "professional search", "ontology"]:
            require(term in assertions, f"{artifact} AC-01 replay assertions missing {term!r}")
        for term in ["personal artifact visible in professional feed", "personal artifact visible in professional search"]:
            require(term in rejections, f"{artifact} AC-01 replay rejections missing {term!r}")
    if fixture_id == "social_prd_produced_contracts_fixture":
        require(status == "passed_first_social_workplace_slice", f"{artifact} produced-contract fixture must pass in first slice")
        for term in ["community.social.post.v1", "socialcontentpublished", "idempotency", "context_kind", "ownership_pillar"]:
            require(term in assertions, f"{artifact} produced-contract replay missing {term!r}")
    if fixture_id == "social_prd_api_contract_replay_fixture":
        require(status == "passed_first_social_workplace_slice", f"{artifact} API replay fixture must pass in first slice")
        source_refs = text(payload.get("source_contract_refs", []))
        for term in ["openapi", "asyncapi", "proto"]:
            require(term in source_refs or term in assertions, f"{artifact} API replay must cite {term}")
    if fixture_id == "social_prd_retired_authority_rejection_fixture":
        require("standalone shorts" in rejections, f"{artifact} must reject standalone shorts")
        require("social media flavor" in assertions, f"{artifact} must retain short-video as social media flavor")


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
            "RED: future social PRD OpenAPI/AsyncAPI/proto/user-story/security replay artifacts are missing under "
            f"{rel(replay_root)}: {preview}{suffix}"
        )
    for fixture in by_id.values():
        for raw in fixture["future_replay_artifacts"]:
            artifact = str(raw)
            payload = load_json(REPO_ROOT / artifact, f"social replay artifact {artifact}")
            validate_replay_payload(fixture, artifact, payload)


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "fixture_kind": "self_test",
                "source_authority_refs": ["specs/microservices/social.json"],
                "future_replay_artifacts": [f"specs/fixtures/social/replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["social_prd_authority_source_lock_fixture"]["must_assert"] = ["PRD-SOCIAL", "t_df502234", "inventory/provenance", "microservices/social/manifest.json"]
    by_id["social_prd_authority_source_lock_fixture"]["must_reject"] = ["runtime readiness", "t_b70fe698", "community FD-001"]
    by_id["social_prd_ac01_personal_post_pillar_fixture"]["must_assert"] = ["context_kind=personal", "ownership_pillar=personal", "professional feed", "professional search", "Ontology work-graph"]
    by_id["social_prd_ac02_context_switch_reconstructs_surfaces_fixture"]["must_assert"] = ["TenantContextSwitcher", "feed", "stories ring", "close-friends", "AR camera", "zero cross-context"]
    by_id["social_prd_ac03_story_ttl_purge_fixture"]["must_assert"] = ["ttl_expires_at", "CDN", "search index", "Ontology graph", "tombstone audit"]
    by_id["social_prd_ac04_professional_crosspost_consent_fixture"]["must_assert"] = ["SocialContentPublished", "consent token", "professional context", "personal context is never consulted", "Workflow/Ontology"]
    by_id["social_prd_ac05_ar_no_biometric_persistence_fixture"]["must_assert"] = ["non-sensitive annotation", "in-session only", "face geometry never leaves device", "server payload excludes biometric", "GDPR Art.9"]
    by_id["social_prd_ac06_collab_ownership_consent_fixture"]["must_assert"] = ["ownership_pillar=personal", "explicit cross-user consent", "audit logged", "collab_owner_ids"]
    by_id["social_prd_produced_contracts_fixture"]["must_assert"] = REQUIRED_PRODUCED_CONTRACTS[:]
    by_id["social_prd_api_contract_replay_fixture"]["source_authority_refs"] = ["oya/social/contracts/openapi/social.yaml", "oya/social/contracts/asyncapi/social-events.yaml", "oya/social/contracts/proto/social.proto"]
    by_id["social_prd_security_privacy_policy_fixture"]["must_assert"] = ["Cedar deny-by-default", "personal content is never employer-visible", "KR", "US", "EU"]
    by_id["social_prd_browser_accessibility_evidence_fixture"]["must_assert"] = ["personal post isolation", "work context switch", "story TTL", "AR no-biometric", "professional cross-post consent", "WCAG 2.2 AA"]
    by_id["social_prd_retired_authority_rejection_fixture"]["must_reject"] = ["Foundry", "standalone shorts", "microservices/shorts", "short-video implementation outside social"]
    by_id["social_prd_retired_authority_rejection_fixture"]["allowed_successors"] = ["intelligence", "Workflow", "Ontology", "social short-video flavor"]
    by_id["social_prd_build_parentage_fixture"]["must_assert"] = ["t_df502234", "t_b70fe698", "oya/social/**", "no generated JSON", "microservices/social/manifest.json", "standalone shorts"]
    return {
        "fixture_plan_id": "SOCIAL-PRD-RED-FIXTURE-CONTRACT-PLAN-001",
        "kanban_task": "t_b70fe698",
        "parent_plan_spec_task": "t_df502234",
        "claim_boundary": "metadata/fixture-only; no runtime handlers or production readiness, UI readiness, or generated JSON hand edits",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "required_prd_acceptance_criteria": REQUIRED_AC_IDS[:],
        "required_produced_contracts": REQUIRED_PRODUCED_CONTRACTS[:],
        "contract_replay_expectations": {
            "openapi": {"source_path": CONTRACT_SOURCE_FILES["openapi"], "must_assert": ["a", "b", "c", "d"]},
            "asyncapi": {"source_path": CONTRACT_SOURCE_FILES["asyncapi"], "must_assert": ["a", "b", "c", "d"]},
            "proto": {"source_path": CONTRACT_SOURCE_FILES["proto"], "must_assert": ["a", "b", "c", "d"]},
        },
        "future_replay_root": "specs/fixtures/social/replay/",
        "global_non_claims": ["green CI alone is insufficient", "generated JSON hand edits are forbidden"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_prd_source_lock(load_json(PRD_PATH, "social PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "social inventory manifest"))
    validate_contract_source_files()
    validate_adr_0334()
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
    expect_rejected("missing AC id", lambda data: data.update({"required_prd_acceptance_criteria": ["AC-SOCIAL-01"]}))
    expect_rejected("missing produced contract", lambda data: data.update({"required_produced_contracts": ["community.social.post.v1"]}))
    expect_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/social/manifest.json"))
    expect_rejected("AC-01 pillar assertion gap", lambda data: data["fixtures"][1].update({"must_assert": ["context_kind=personal"]}))
    expect_rejected("AC-03 purge assertion gap", lambda data: data["fixtures"][3].update({"must_assert": ["ttl_expires_at"]}))
    expect_rejected("AC-05 biometric assertion gap", lambda data: data["fixtures"][5].update({"must_assert": ["non-sensitive annotation"]}))
    expect_rejected("API replay sources missing", lambda data: data["fixtures"][8].update({"source_authority_refs": ["oya/social/contracts/openapi/social.yaml"]}))
    expect_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/social/replay/bad.generated.json"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("social PRD RED fixture contract self-tests passed")


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
    validate_prd_source_lock(load_json(PRD_PATH, "social PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "social inventory manifest"))
    validate_contract_source_files()
    validate_service_local_first_slice()
    validate_adr_0334()
    by_id = validate_manifest(manifest)
    validate_replay_artifacts(by_id, replay_root)
    print("social PRD first-slice replay artifacts passed")


if __name__ == "__main__":
    main()
