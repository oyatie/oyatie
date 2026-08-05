#!/usr/bin/env python3
"""Fail-closed RED checker for translate source-map and contract replay fixtures.

This is a RED-only metadata/source-lock guard. It validates that the translate
fixture manifest is grounded in specs/microservices/translate.json, treats
oya/translate/manifest.json and dirty oya/translate/** inventory as
provenance-only, rejects legacy microservices/translate/** live-destination
claims, normalizes retired Foundry/foundry-runtime authority to intelligence,
and remains RED until future Build cards create source-backed OpenAPI/AsyncAPI/
proto replay artifacts.
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
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "translate" / "source-map" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "translate" / "source-map" / "replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "translate.json"
MANIFEST_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
INVENTORY_MANIFEST_PATH = REPO_ROOT / "oya" / "translate" / "manifest.json"

EXPECTED_FIXTURE_IDS = [
    "translate_source_map_authority_lock_fixture",
    "translate_domain_coverage_fixture",
    "translate_legacy_manifest_rejection_fixture",
    "translate_contract_replay_openapi_fixture",
    "translate_contract_replay_asyncapi_fixture",
    "translate_contract_replay_proto_fixture",
    "translate_browser_accessibility_gate_fixture",
    "translate_provider_residency_data_policy_fixture",
    "translate_inventory_runtime_readiness_rejection_fixture",
    "translate_retired_foundry_intelligence_boundary_fixture",
    "translate_build_parentage_guard_fixture",
]

EXPECTED_DOMAIN_IDS = {
    "single_translation",
    "batch_translation",
    "bulk_translation",
    "language_detection",
    "translation_memory",
    "quality_estimation",
    "document_localization",
    "real_time_caption_translation",
    "glossary",
    "termbase",
}

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/translate.json",
    "specs/microservices/manifests-index.json#microservices[name=translate]",
    "oya/translate/manifest.json",
    "docs/decisions/ADR-0131-per-microservice-flat-layout.md",
    "docs/decisions/ADR-0512-canonical-monorepo-pattern.md",
    "kanban:t_ce0e47d2#translate-source-lock-review",
    "kanban:t_a966bab0#translate-red-source-map-contract-replay",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/translate/manifest.json",
    "dirty oya/translate/** inventory",
    "oya/translate/contracts/*",
    "oya/translate/catalog/*",
    "oya/translate/capabilities/*",
    "oya/translate/IPs/*",
    "oya/translate/slos/*",
    "microservices/translate/manifest.json",
    "microservices/translate/contracts/*",
    "microservices/translate/capabilities/*",
    "microservices/translate/IP-*.md",
    "microservices/translate/slos/*",
}

CONTRACT_SOURCE_FILES = {
    "openapi": "oya/translate/contracts/openapi/translate.yaml",
    "asyncapi": "oya/translate/contracts/asyncapi/translate-events.yaml",
    "proto": "oya/translate/contracts/proto/translate.proto",
}

LEGACY_CONTRACT_POINTERS = {
    "openapi": "microservices/translate/contracts/openapi/translate.yaml",
    "asyncapi": "microservices/translate/contracts/asyncapi/translate-events.yaml",
    "proto": "microservices/translate/contracts/proto/translate.proto",
}

CANDIDATE_CASES = {
    "valid_translate_source_map_metadata_fixture": REPO_ROOT
    / "specs"
    / "fixtures"
    / "translate"
    / "source-map"
    / "valid-translate-source-map.json",
    "rejects_legacy_microservices_manifest_fixture": REPO_ROOT
    / "specs"
    / "fixtures"
    / "translate"
    / "source-map"
    / "rejects-legacy-microservices-manifest.json",
    "rejects_runtime_readiness_claim_fixture": REPO_ROOT
    / "specs"
    / "fixtures"
    / "translate"
    / "source-map"
    / "rejects-runtime-readiness-claim.json",
    "rejects_retired_ai_substrate_live_owner_fixture": REPO_ROOT
    / "specs"
    / "fixtures"
    / "translate"
    / "source-map"
    / "rejects-retired-ai-substrate-live-owner.json",
    "rejects_build_before_review_fixture": REPO_ROOT
    / "specs"
    / "fixtures"
    / "translate"
    / "source-map"
    / "rejects-build-before-review.json",
}

EXPECTED_RED_STATUS = "RED_UNTIL_TRANSLATE_REPLAY_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"


def fail(message: str) -> NoReturn:
    print(f"translate source-map RED fixture contract check failed: {message}", file=sys.stderr)
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
    if isinstance(value, set):
        return " ".join(text(item) for item in sorted(value))
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


def repo_ref_path(raw_ref: object) -> Path | None:
    ref = str(raw_ref)
    if ref.startswith("$ref:"):
        ref = ref.removeprefix("$ref:")
    if ref.startswith(("kanban:", "http://", "https://")):
        return None
    ref = ref.split("#", 1)[0]
    if not ref or any(token in ref for token in ("*", "?", "[", "]")):
        return None
    if not ref.startswith(("docs/", "oya/", "scripts/", "specs/")):
        return None
    return REPO_ROOT / ref


def require_repo_refs_exist(refs: object, label: str) -> None:
    require(isinstance(refs, list), f"{label} must be a list")
    for raw_ref in refs:
        path = repo_ref_path(raw_ref)
        if path is not None:
            require(path.exists(), f"{label} references missing repo path: {raw_ref}")


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
            artifact.startswith("specs/fixtures/translate/source-map/replay/"),
            f"future replay artifact must stay under specs/fixtures/translate/source-map/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_source_map(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "translate source map _meta must be an object")
    require(meta.get("spec_id") == "PRD-TRANSLATE-SOURCE-MAP", "translate source map spec_id must be PRD-TRANSLATE-SOURCE-MAP")
    require(meta.get("status") == "Accepted", "translate source map status must be Accepted")
    require(meta.get("authority_lock_task") == "t_558fedfa", "translate source map must bind source-lock task t_558fedfa")

    authority = prd.get("authority_resolution", {})
    require(isinstance(authority, dict), "authority_resolution must be an object")
    require(authority.get("current_authority") == "specs/microservices/translate.json", "current authority path drifted")
    require(authority.get("manifest_index_ref") == "specs/microservices/manifests-index.json#microservices[name=translate]", "manifest_index_ref drifted")
    require(authority.get("manifest_inventory_ref") == "oya/translate/manifest.json", "manifest_inventory_ref drifted")
    require(authority.get("review_gate_task") == "t_ce0e47d2", "review gate task must be t_ce0e47d2")
    require("microservices/translate/manifest.json" in text(authority.get("prior_state_recorded", {})), "legacy manifest prior state must be recorded")
    require("absent" in text(authority.get("prior_state_recorded", {})), "legacy microservices manifest must stay absent in prior state")
    root_boundary = authority.get("root_boundary", {})
    require(isinstance(root_boundary, dict), "root_boundary must be an object")
    require(root_boundary.get("canonical_service_home") == "oya/translate/", "canonical service home must be oya/translate/")
    require(root_boundary.get("legacy_root") == "microservices/translate/", "legacy root must be recorded as microservices/translate/")
    require_terms(root_boundary, ["ADR-0131", "ADR-0512", "must not be restored"], "root boundary")
    require_terms(authority.get("claim_boundary", ""), ["no handler", "production readiness", "customer availability"], "claim boundary")

    inherits = prd.get("inherits", {})
    require(isinstance(inherits, dict), "inherits must be an object")
    require_repo_refs_exist(inherits.get("oyatie_specs", []), "inherits.oyatie_specs")
    require_repo_refs_exist(inherits.get("adrs", []), "inherits.adrs")
    require_repo_refs_exist(inherits.get("inventory_context_not_fanout_authority", []), "inherits.inventory_context_not_fanout_authority")
    require_repo_refs_exist(inherits.get("workplace_api_user_story_evidence_candidates", []), "inherits.workplace_api_user_story_evidence_candidates")

    identity = prd.get("identity", {})
    require(isinstance(identity, dict), "identity must be an object")
    require(identity.get("product_id") == "translate", "identity.product_id must be translate")
    require(identity.get("fd001_material") is False, "translate must not claim FD-001 materiality")
    require(identity.get("user_facing_surface") is True, "translate must remain user-facing for browser/a11y gates")

    source_map = prd.get("source_inventory_map", {})
    require(isinstance(source_map, dict), "source_inventory_map must be an object")
    manifest_source = source_map.get("manifest_source", {})
    require(isinstance(manifest_source, dict), "manifest_source must be an object")
    require(manifest_source.get("path") == "oya/translate/manifest.json", "manifest_source path drifted")
    require(manifest_source.get("status") == "inventory_provenance_only", "manifest source must remain inventory/provenance only")
    require_contains_all(
        manifest_source.get("must_not_be_used_as"),
        [
            "implementation-readiness evidence",
            "runtime-readiness evidence",
            "product-readiness evidence",
            "permission to restore microservices/translate/manifest.json",
        ],
        "manifest_source.must_not_be_used_as",
    )

    canonical_rows = source_map.get("canonical_path_map", [])
    require(isinstance(canonical_rows, list), "canonical_path_map must be a list")
    canonical_text = text(canonical_rows)
    for legacy, current in LEGACY_CONTRACT_POINTERS.items():
        require(current.lower() in canonical_text, f"canonical path map must record legacy {legacy} pointer")
        require(CONTRACT_SOURCE_FILES[legacy].lower() in canonical_text, f"canonical path map must map {legacy} to oya/translate")
    require("microservices/translate/manifest.json" not in text(source_map.get("canonical_live_manifest", "")), "source map must not expose a canonical legacy manifest")

    stale_handling = text(source_map.get("stale_or_retired_reference_handling", []))
    require("foundry" in stale_handling and "intelligence" in stale_handling, "retired Foundry wording must normalize to intelligence")
    require("dirty-tree oya/translate" in stale_handling, "dirty oya/translate inventory handling must be explicit")
    require("inventory/provenance only" in stale_handling, "dirty inventory must stay provenance only")

    domains = prd.get("functional_domain_map", {}).get("domains", [])
    require(isinstance(domains, list), "functional_domain_map.domains must be a list")
    domain_ids = {str(item.get("id")) for item in domains if isinstance(item, dict)}
    missing_domains = sorted(EXPECTED_DOMAIN_IDS - domain_ids)
    require(not missing_domains, f"translate functional domain coverage missing {missing_domains}")

    evidence = prd.get("evidence_expectations", {})
    require(isinstance(evidence, dict), "evidence_expectations must be an object")
    replay = evidence.get("api_contract_replay", {})
    require(isinstance(replay, dict), "api_contract_replay must be an object")
    require(replay.get("required_before_build_or_rollout") is True, "contract replay must be required before build/rollout")
    require_contains_all(replay.get("surfaces"), list(CONTRACT_SOURCE_FILES.values()), "api_contract_replay.surfaces")
    require_terms(
        replay.get("minimum_scenarios", []),
        [
            "single translation",
            "batch translation",
            "bulk and document",
            "language detection",
            "TM leverage",
            "glossary",
            "termbase",
            "QE scoring",
            "real-time caption",
            "provider routing",
            "residency",
        ],
        "api replay minimum_scenarios",
    )

    browser = evidence.get("browser_user_story_accessibility", {})
    require(isinstance(browser, dict), "browser_user_story_accessibility must be an object")
    require(browser.get("required_for_user_visible_slices") is True, "browser/a11y must be required for user-visible slices")
    require_terms(
        browser.get("minimum_scenarios", []),
        ["translation", "meeting", "caption", "glossary", "termbase", "document", "screen-reader"],
        "browser/user-story/accessibility scenarios",
    )
    require_terms(browser.get("accessibility_floor", []), ["WCAG 2.2 AA", "keyboard", "screen-reader", "focus"], "accessibility floor")

    policy = evidence.get("security_data_policy", {})
    require(isinstance(policy, dict), "security_data_policy must be an object")
    require(policy.get("required_for_data_or_policy_slices") is True, "security/data policy must be required")
    require_terms(
        policy.get("minimum_scenarios", []),
        ["Cedar", "residency", "cross-border", "human-review", "no plaintext", "T2 automation"],
        "security/data policy scenarios",
    )

    red_plan = prd.get("red_fixture_plan", {})
    require(isinstance(red_plan, dict), "red_fixture_plan must be an object")
    require(red_plan.get("fanout_blocked_until_review_task_complete") == "t_ce0e47d2", "RED fanout must be blocked until review task t_ce0e47d2")
    require_contains_all(
        red_plan.get("candidate_fixture_paths"),
        [
            "specs/fixtures/translate/source-map/valid-translate-source-map.json",
            "specs/fixtures/translate/source-map/rejects-legacy-microservices-manifest.json",
            "specs/fixtures/translate/source-map/rejects-runtime-readiness-claim.json",
            "specs/fixtures/translate/source-map/rejects-retired-ai-substrate-live-owner.json",
            "scripts/tests/translate_source_map_authority_check.py",
        ],
        "red_fixture_plan.candidate_fixture_paths",
    )
    require_terms(
        red_plan.get("must_fail_when", []),
        [
            "missing or invalid JSON",
            "microservices/translate/manifest.json",
            "oya/translate/manifest.json",
            "single/batch/bulk",
            "API/AsyncAPI/proto",
            "browser/user-story/accessibility",
            "provider-routing/residency/data-policy",
            "Build fanout before independent Review/fix",
        ],
        "red fixture must_fail_when",
    )

    lifecycle = prd.get("downstream_lifecycle_gates", {})
    require(isinstance(lifecycle, dict), "downstream_lifecycle_gates must be an object")
    review_fix = lifecycle.get("review_fix_source_lock", {})
    require(isinstance(review_fix, dict), "review_fix_source_lock must be an object")
    require(review_fix.get("task") == "t_ce0e47d2", "review/fix source-lock task drifted")
    require_terms(review_fix.get("required_before", []), ["RED fixture", "Build implementation"], "review/fix required_before")
    require(lifecycle.get("future_build", {}).get("status") == "blocked_pending_review_and_red", "future Build must remain blocked pending review and RED")

    fanout = prd.get("downstream_fanout_policy", {})
    require(isinstance(fanout, dict), "downstream_fanout_policy must be an object")
    require(fanout.get("review_required_before_red_or_build") is True, "review must be required before RED or Build")
    require(fanout.get("existing_review_child") == "t_ce0e47d2", "existing review child must be t_ce0e47d2")
    require_terms(fanout.get("deferred_until_review_approves", []), ["Build implementation"], "deferred fanout")

    claims = prd.get("claim_boundaries", {})
    require(isinstance(claims, dict), "claim_boundaries must be an object")
    for key in ["fd001_material", "production_ready", "runtime_implemented_by_this_artifact", "measured_slos_available", "live_e2e_verified", "inventory_manifest_promoted", "release_ready"]:
        require(claims.get(key) is False, f"claim_boundaries.{key} must be false")


def validate_manifest_index_source_lock(index: dict[str, Any]) -> None:
    rows = index.get("microservices")
    require(isinstance(rows, list), "manifests-index microservices must be a list")
    translate_rows = [row for row in rows if isinstance(row, dict) and row.get("name") == "translate"]
    require(len(translate_rows) == 1, f"manifests-index must contain exactly one translate row; got {len(translate_rows)}")
    row = translate_rows[0]
    require(row.get("manifest") == "oya/translate/manifest.json", "translate manifest-index row must point to oya/translate/manifest.json")
    require(row.get("fd001_material") is False, "translate manifest-index row must not be FD-001 material")
    require(row.get("authority_status") == "source-authority-reconciled-by-t_ff8bab02", "translate authority_status drifted")
    require(row.get("prd") in (None, "specs/microservices/translate.json"), "translate manifest-index PRD pointer must be absent or point to the source-lock PRD")
    boundary = str(row.get("authority_boundary", "")).lower()
    require("inventory/provenance only" in boundary, "translate authority boundary must state inventory/provenance only")
    require("runtime/product-readiness" in boundary, "translate manifest-index row must deny runtime/product readiness")


def validate_inventory_source_lock(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "translate", "translate inventory manifest microservice must be translate")
    contexts = {str(item.get("name")) for item in inventory.get("bounded_contexts", []) if isinstance(item, dict)}
    require({"bulk-translate", "document-localization", "language-detection", "quality-estimation", "real-time-stream", "termbase-and-glossary", "translate-router", "translation-memory"}.issubset(contexts), "translate inventory bounded contexts drifted")
    contracts = inventory.get("contracts", {})
    require(isinstance(contracts, dict), "translate inventory contracts must be an object")
    for family, legacy_pointer in LEGACY_CONTRACT_POINTERS.items():
        require_contains_all(contracts.get(family), [legacy_pointer], f"translate inventory {family} provenance pointers")
    tiers = {str(item.get("tier")) for item in inventory.get("capabilities", []) if isinstance(item, dict)}
    require({"T0", "T1", "T2"}.issubset(tiers), "translate inventory must retain T0/T1/T2 capability provenance")
    lower_inventory = text(inventory)
    require("foundry" in lower_inventory, "translate inventory must retain retired Foundry/foundry-runtime provenance for guard coverage")
    require("intelligence" in lower_inventory, "translate inventory must also name current intelligence dependency context")


def validate_contract_source_files() -> None:
    for family, raw_path in CONTRACT_SOURCE_FILES.items():
        path = REPO_ROOT / raw_path
        require(path.exists(), f"translate {family} source contract must exist at {raw_path}")
        require(path.is_file(), f"translate {family} source contract must be a file: {raw_path}")
        require(not raw_path.endswith(GENERATED_SUFFIX), f"translate {family} source contract must not be a generated face: {raw_path}")
        require(raw_path.startswith("oya/translate/contracts/"), f"translate {family} source contract must live under oya/translate/contracts/: {raw_path}")


def validate_candidate_cases() -> None:
    for expected_id, path in CANDIDATE_CASES.items():
        case = load_json(path, f"candidate fixture {expected_id}")
        require(case.get("fixture_id") == expected_id, f"{rel(path)} fixture_id drifted")
        require(case.get("source_map_ref") == "specs/microservices/translate.json", f"{expected_id} must cite translate source map")
        require(case.get("kanban_task") == "t_a966bab0", f"{expected_id} must bind to this RED task")
        decision = case.get("expected_checker_decision")
        require(decision in {"METADATA_VALID_REPLAY_STILL_RED", "REJECT"}, f"{expected_id} has unexpected checker decision")

    valid = load_json(CANDIDATE_CASES["valid_translate_source_map_metadata_fixture"], "valid translate source-map fixture")
    require_fixture_terms(
        valid,
        "must_assert",
        ["PRD-TRANSLATE-SOURCE-MAP", "oya/translate", "OpenAPI/AsyncAPI/proto", "browser/user-story/accessibility", "provider-routing/residency/data-policy"],
        "valid source-map fixture",
    )
    require_fixture_terms(
        valid,
        "must_not_claim",
        ["runtime readiness", "product readiness", "FD-001 materiality", "hyperscaler maturity"],
        "valid source-map fixture",
    )

    legacy = load_json(CANDIDATE_CASES["rejects_legacy_microservices_manifest_fixture"], "legacy rejection fixture")
    require_fixture_terms(legacy, "must_reject", ["microservices/translate/manifest.json", "restoration", "legacy microservices/translate"], "legacy rejection fixture")

    readiness = load_json(CANDIDATE_CASES["rejects_runtime_readiness_claim_fixture"], "runtime readiness rejection fixture")
    require_fixture_terms(readiness, "must_reject", ["runtime readiness", "product readiness", "green CI alone", "live SLO"], "runtime readiness rejection fixture")

    retired = load_json(CANDIDATE_CASES["rejects_retired_ai_substrate_live_owner_fixture"], "retired authority rejection fixture")
    require_fixture_terms(retired, "must_reject", ["Foundry", "foundry-runtime", "current intelligence authority", "T2 auto-translation"], "retired authority rejection fixture")
    require_fixture_terms(retired, "allowed_when", ["retired/provenance", "normalized to intelligence"], "retired authority allowed context")

    build = load_json(CANDIDATE_CASES["rejects_build_before_review_fixture"], "Build parentage rejection fixture")
    require_fixture_terms(build, "must_reject", ["Build implementation", "t_ce0e47d2", "t_a966bab0", "generated JSON hand edit"], "Build parentage rejection fixture")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "TRANSLATE-SOURCE-MAP-CONTRACT-REPLAY-RED-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_a966bab0", "manifest must bind to kanban task t_a966bab0")
    require(manifest.get("review_fix_parent_task") == "t_ce0e47d2", "manifest must bind source-lock Review/fix parent t_ce0e47d2")
    require(manifest.get("source_lock_task") == "t_558fedfa", "manifest must bind source-lock task t_558fedfa")
    require_terms(manifest.get("claim_boundary", ""), ["runtime", "production readiness", "generated JSON hand edits"], "claim_boundary")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_repo_refs_exist(manifest.get("source_authority_refs", []), "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")
    require_contains_all(manifest.get("domain_coverage_expectations"), EXPECTED_DOMAIN_IDS, "domain_coverage_expectations")

    replay = manifest.get("contract_replay_expectations")
    require(isinstance(replay, dict), "contract_replay_expectations must be an object")
    require(set(replay) == set(CONTRACT_SOURCE_FILES), f"contract_replay_expectations keys must be {sorted(CONTRACT_SOURCE_FILES)}")
    for key, expected_path in CONTRACT_SOURCE_FILES.items():
        section = replay[key]
        require(isinstance(section, dict), f"contract_replay_expectations.{key} must be an object")
        require(section.get("source_path") == expected_path, f"{key} source_path drifted")
        require(section.get("legacy_manifest_pointer") == LEGACY_CONTRACT_POINTERS[key], f"{key} legacy_manifest_pointer drifted")
        require_terms(section.get("must_assert", []), ["tenant", "audit"], f"{key} replay assertions")
        require_terms(section.get("must_reject", []), ["legacy", "residency" if key == "openapi" else "tenant"], f"{key} replay negatives")

    require(manifest.get("future_replay_root") == "specs/fixtures/translate/source-map/replay/", "future_replay_root must be source-locked")
    require_terms(manifest.get("browser_user_story_accessibility_gate", []), ["keyboard", "screen-reader", "N/A"], "browser/user-story/accessibility gate")
    require_terms(manifest.get("provider_residency_data_policy_gate", []), ["Cedar", "residency", "cross-border", "human-review", "no plaintext"], "provider/residency/data policy gate")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match translate RED plan; got {actual_ids}")
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        require_repo_refs_exist(fixture.get("source_authority_refs", []), f"{fixture_id}.source_authority_refs")
        validate_future_replay_artifacts(fixture)

    require_fixture_sources(by_id["translate_source_map_authority_lock_fixture"], ["specs/microservices/translate.json", "manifests-index", "oya/translate/manifest.json"], "authority source-map fixture")
    require_fixture_terms(by_id["translate_source_map_authority_lock_fixture"], "must_assert", ["PRD-TRANSLATE-SOURCE-MAP", "current_authority", "inventory/provenance only", "ADR-0131/ADR-0512"], "authority source-map fixture")
    require_fixture_terms(by_id["translate_domain_coverage_fixture"], "must_assert", sorted(EXPECTED_DOMAIN_IDS), "domain coverage fixture")
    require_fixture_terms(by_id["translate_legacy_manifest_rejection_fixture"], "must_reject", ["microservices/translate/manifest.json restoration", "microservices/translate/** as live destination", "legacy"], "legacy manifest fixture")
    require_fixture_sources(by_id["translate_contract_replay_openapi_fixture"], [CONTRACT_SOURCE_FILES["openapi"]], "OpenAPI replay fixture")
    require_fixture_sources(by_id["translate_contract_replay_asyncapi_fixture"], [CONTRACT_SOURCE_FILES["asyncapi"]], "AsyncAPI replay fixture")
    require_fixture_sources(by_id["translate_contract_replay_proto_fixture"], [CONTRACT_SOURCE_FILES["proto"]], "proto replay fixture")
    require_fixture_terms(by_id["translate_browser_accessibility_gate_fixture"], "must_assert", ["WCAG 2.2 AA", "keyboard-only", "screen-reader", "N/A rationale"], "browser/a11y fixture")
    require_fixture_terms(by_id["translate_provider_residency_data_policy_fixture"], "must_assert", ["tenant/RBAC", "Cedar", "residency-aware provider routing", "cross-border consent", "human-review"], "provider/data policy fixture")
    require_fixture_terms(by_id["translate_inventory_runtime_readiness_rejection_fixture"], "must_reject", ["runtime readiness", "product readiness", "dirty oya/translate/** inventory"], "inventory readiness fixture")
    require_fixture_terms(by_id["translate_retired_foundry_intelligence_boundary_fixture"], "must_reject", ["Foundry", "foundry-runtime", "T2 auto-translation"], "retired Foundry fixture")
    require_fixture_terms(by_id["translate_retired_foundry_intelligence_boundary_fixture"], "must_assert", ["retired/provenance", "intelligence"], "retired Foundry fixture")
    require_fixture_terms(by_id["translate_build_parentage_guard_fixture"], "must_assert", ["t_ce0e47d2", "t_a966bab0", "allowed path", "generated-face"], "build parentage fixture")
    require_fixture_terms(by_id["translate_build_parentage_guard_fixture"], "must_reject", ["Build without source-lock review parent", "Build without RED fixture parent", "runtime handler in RED card"], "build parentage fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("microservices/translate/manifest.json restoration is not authorized" in lower_manifest, "manifest must explicitly reject legacy translate manifest restoration")
    require("active foundry or foundry-runtime authority is not authorized" in lower_manifest, "manifest must explicitly reject active Foundry authority")
    require("green ci alone is insufficient" in lower_manifest, "manifest must reject green-CI-alone UI readiness")
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
            "RED: future translate source-map and OpenAPI/AsyncAPI/proto replay artifacts are missing under "
            f"{rel(replay_root)}: {preview}{suffix}"
        )
    fail("future translate contract replay is not implemented; this RED-only checker must be extended by a Build card before green status")


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "fixture_kind": "self_test",
                "source_authority_refs": ["specs/microservices/translate.json"],
                "future_replay_artifacts": [f"specs/fixtures/translate/source-map/replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["translate_source_map_authority_lock_fixture"]["source_authority_refs"] = ["specs/microservices/translate.json", "specs/microservices/manifests-index.json#microservices[name=translate]", "oya/translate/manifest.json"]
    by_id["translate_source_map_authority_lock_fixture"]["must_assert"] = ["PRD-TRANSLATE-SOURCE-MAP", "current_authority", "inventory/provenance only", "ADR-0131/ADR-0512"]
    by_id["translate_domain_coverage_fixture"]["must_assert"] = sorted(EXPECTED_DOMAIN_IDS)
    by_id["translate_legacy_manifest_rejection_fixture"]["must_reject"] = ["microservices/translate/manifest.json restoration", "microservices/translate/** as live destination", "legacy"]
    by_id["translate_contract_replay_openapi_fixture"]["source_authority_refs"] = [CONTRACT_SOURCE_FILES["openapi"]]
    by_id["translate_contract_replay_asyncapi_fixture"]["source_authority_refs"] = [CONTRACT_SOURCE_FILES["asyncapi"]]
    by_id["translate_contract_replay_proto_fixture"]["source_authority_refs"] = [CONTRACT_SOURCE_FILES["proto"]]
    by_id["translate_browser_accessibility_gate_fixture"]["must_assert"] = ["WCAG 2.2 AA", "keyboard-only", "screen-reader", "N/A rationale"]
    by_id["translate_provider_residency_data_policy_fixture"]["must_assert"] = ["tenant/RBAC", "Cedar", "residency-aware provider routing", "cross-border consent", "human-review"]
    by_id["translate_inventory_runtime_readiness_rejection_fixture"]["must_reject"] = ["runtime readiness", "product readiness", "dirty oya/translate/** inventory"]
    by_id["translate_retired_foundry_intelligence_boundary_fixture"]["must_assert"] = ["retired/provenance", "intelligence"]
    by_id["translate_retired_foundry_intelligence_boundary_fixture"]["must_reject"] = ["Foundry", "foundry-runtime", "T2 auto-translation"]
    by_id["translate_build_parentage_guard_fixture"]["must_assert"] = ["t_ce0e47d2", "t_a966bab0", "allowed path", "generated-face"]
    by_id["translate_build_parentage_guard_fixture"]["must_reject"] = ["Build without source-lock review parent", "Build without RED fixture parent", "runtime handler in RED card"]
    return {
        "fixture_plan_id": "TRANSLATE-SOURCE-MAP-CONTRACT-REPLAY-RED-001",
        "kanban_task": "t_a966bab0",
        "review_fix_parent_task": "t_ce0e47d2",
        "source_lock_task": "t_558fedfa",
        "claim_boundary": "metadata/fixture-only; no runtime handlers, production readiness, or generated JSON hand edits",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "domain_coverage_expectations": sorted(EXPECTED_DOMAIN_IDS),
        "contract_replay_expectations": {
            "openapi": {"source_path": CONTRACT_SOURCE_FILES["openapi"], "legacy_manifest_pointer": LEGACY_CONTRACT_POINTERS["openapi"], "must_assert": ["tenant", "audit"], "must_reject": ["legacy", "residency"]},
            "asyncapi": {"source_path": CONTRACT_SOURCE_FILES["asyncapi"], "legacy_manifest_pointer": LEGACY_CONTRACT_POINTERS["asyncapi"], "must_assert": ["tenant", "audit"], "must_reject": ["legacy", "tenant"]},
            "proto": {"source_path": CONTRACT_SOURCE_FILES["proto"], "legacy_manifest_pointer": LEGACY_CONTRACT_POINTERS["proto"], "must_assert": ["tenant", "audit"], "must_reject": ["legacy", "tenant"]},
        },
        "future_replay_root": "specs/fixtures/translate/source-map/replay/",
        "browser_user_story_accessibility_gate": ["keyboard", "screen-reader", "N/A"],
        "provider_residency_data_policy_gate": ["Cedar", "residency", "cross-border", "human-review", "no plaintext"],
        "global_non_claims": ["microservices/translate/manifest.json restoration is not authorized", "active Foundry or foundry-runtime authority is not authorized", "green CI alone is insufficient"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    live_prd = load_json(PRD_PATH, "translate source map")
    validate_source_map(live_prd)
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "translate inventory manifest"))
    validate_contract_source_files()
    validate_candidate_cases()
    validate_manifest(live_manifest)
    valid = baseline_manifest()
    validate_manifest(valid)

    def expect_manifest_rejected(label: str, mutator: Callable[[dict[str, Any]], None]) -> None:
        candidate = copy.deepcopy(valid)
        mutator(candidate)
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                validate_manifest(candidate)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    def expect_source_map_rejected(label: str, mutator: Callable[[dict[str, Any]], None]) -> None:
        candidate = copy.deepcopy(live_prd)
        mutator(candidate)
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                validate_source_map(candidate)
        except SystemExit as exc:
            require(exc.code != 0, f"source-map self-test {label!r} exited successfully")
        else:
            fail(f"source-map self-test mutation was accepted: {label}")

    expect_manifest_rejected("missing fixture id", lambda data: data["fixtures"].pop())
    expect_manifest_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/translate/manifest.json"))
    expect_manifest_rejected("domain coverage gap", lambda data: data["domain_coverage_expectations"].remove("termbase"))
    expect_manifest_rejected("OpenAPI replay source missing", lambda data: data["fixtures"][3].update({"source_authority_refs": ["oya/translate/contracts/asyncapi/translate-events.yaml"]}))
    expect_manifest_rejected("browser accessibility gap", lambda data: data["fixtures"][6].update({"must_assert": ["keyboard-only"]}))
    expect_manifest_rejected("provider residency gap", lambda data: data["fixtures"][7].update({"must_assert": ["Cedar"]}))
    expect_manifest_rejected("retired Foundry boundary gap", lambda data: data["fixtures"][9].update({"must_reject": ["Foundry"]}))
    expect_manifest_rejected("stale fixture source authority ref", lambda data: data["fixtures"][9].update({"source_authority_refs": ["docs/decisions/ADR-0335-foundry-absorbed-by-intelligence.md"]}))
    expect_manifest_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/translate/source-map/replay/bad.generated.json"]}))
    expect_manifest_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    expect_source_map_rejected("missing source map", lambda data: data.update({"_meta": {}}))
    expect_source_map_rejected("stale ADR ref", lambda data: data["inherits"]["adrs"].append("$ref:docs/decisions/ADR-0335-foundry-absorbed-by-intelligence.md"))
    expect_source_map_rejected("domain omitted", lambda data: data["functional_domain_map"]["domains"].pop())
    expect_source_map_rejected("API replay surfaces omitted", lambda data: data["evidence_expectations"]["api_contract_replay"].update({"surfaces": []}))
    expect_source_map_rejected("browser evidence omitted", lambda data: data["evidence_expectations"].pop("browser_user_story_accessibility"))
    expect_source_map_rejected("security policy omitted", lambda data: data["evidence_expectations"].pop("security_data_policy"))
    expect_source_map_rejected("review gate omitted", lambda data: data["downstream_fanout_policy"].update({"review_required_before_red_or_build": False}))
    print("translate source-map RED fixture contract self-tests passed")


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
    validate_source_map(load_json(PRD_PATH, "translate source map"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_inventory_source_lock(load_json(INVENTORY_MANIFEST_PATH, "translate inventory manifest"))
    validate_contract_source_files()
    validate_candidate_cases()
    by_id = validate_manifest(manifest)
    validate_replay_artifacts(by_id, replay_root)


if __name__ == "__main__":
    main()
