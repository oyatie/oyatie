#!/usr/bin/env python3
"""Fail-closed RED checker for the Anonymous PRD privacy fixture plan.

This is a metadata/source-lock guard for RED-ANON-001. It validates that the
anonymous RED fixture manifest is grounded in the PLAN/SPEC-ANON-001 source lock
and remains RED until future Build cards create source-backed contract replay
artifacts and runtime implementation under the locked community home.
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
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "anonymous-prd" / "red-fixtures.json"
DEFAULT_REPLAY_ROOT = REPO_ROOT / "specs" / "fixtures" / "anonymous-prd" / "replay"
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "anonymous.json"
MANIFEST_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
PARENT_INVENTORY_PATH = REPO_ROOT / "oya" / "community" / "manifest.json"
FORBIDDEN_STANDALONE_MANIFESTS = [
    REPO_ROOT / "microservices" / "anonymous" / "manifest.json",
    REPO_ROOT / "oya" / "anonymous" / "manifest.json",
]

EXPECTED_FIXTURE_IDS = [
    "anonymous_prd_authority_source_lock_fixture",
    "anonymous_prd_post_token_only_fixture",
    "anonymous_prd_identity_fk_leakage_rejection_fixture",
    "anonymous_prd_verification_vault_isolation_fixture",
    "anonymous_prd_hr_aggregate_k_anonymity_fixture",
    "anonymous_prd_moderator_token_only_fixture",
    "anonymous_prd_legal_hold_four_eyes_fixture",
    "anonymous_prd_work_personal_context_isolation_fixture",
    "anonymous_prd_retired_authority_rejection_fixture",
    "anonymous_prd_browser_accessibility_evidence_fixture",
    "anonymous_prd_build_parentage_fixture",
]

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/anonymous.json",
    "specs/microservices/anonymous.json#implementation_source_lock",
    "specs/microservices/manifests-index.json#microservices[name=anonymous]",
    "oya/community/manifest.json",
    "kanban:t_8da1d130#PLAN/SPEC-ANON-001",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/community/manifest.json",
    "microservices/anonymous/manifest.json",
    "oya/anonymous/manifest.json",
    "identity.crate_refs_planned layer-per-crate list",
    "Foundry/AI-substrate active authority",
}

REQUIRED_AC_IDS = ["AC-ANON-01", "AC-ANON-02", "AC-ANON-03", "AC-ANON-04", "AC-ANON-05", "AC-ANON-06"]
REQUIRED_PRODUCED_CONTRACTS = [
    "community.anonymous.post.v1",
    "community.anonymous.salary_benchmark.v1",
    "community.anonymous.topic_trend.v1",
    "community.anonymous.legal_hold_identity_reveal.v1",
    "audit.community.anonymous.policy.v1",
]
REQUIRED_PERSONAS = {"verified_employee", "moderator", "enterprise_hr_admin", "compliance_officer"}
EXPECTED_RED_STATUS = "RED_UNTIL_ANONYMOUS_REPLAY_ARTIFACT_EXISTS"
EXPECTED_REPLAY_STATUS = "green_after_build_slice_contract_replay"
GENERATED_SUFFIX = ".generated.json"


def is_explicit_non_rust_evidence(test_name: str) -> bool:
    lower = test_name.lower()
    return (
        "api/contract replay only" in lower
        or "explicit n/a" in lower
        or "replay checker" in lower
    )


def fail(message: str) -> NoReturn:
    print(f"anonymous PRD RED fixture contract check failed: {message}", file=sys.stderr)
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


def require_terms_in_list(items: object, required_terms: list[str], label: str) -> None:
    require(isinstance(items, list), f"{label} must be a list")
    haystack = text(items)
    missing = [term for term in required_terms if term.lower() not in haystack]
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
    require(isinstance(artifacts, list) and artifacts, f"{fixture.get('fixture_id')} must name future_replay_artifacts")
    for raw in artifacts:
        artifact = str(raw)
        require(
            artifact.startswith("specs/fixtures/anonymous-prd/replay/"),
            f"future replay artifact must stay under specs/fixtures/anonymous-prd/replay/: {artifact}",
        )
        require(not artifact.endswith(GENERATED_SUFFIX), f"future replay artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future replay artifact must not traverse directories: {artifact}")


def validate_prd_source_lock(prd: dict[str, Any]) -> None:
    meta = prd.get("_meta", {})
    require(isinstance(meta, dict), "anonymous PRD _meta must be an object")
    require(meta.get("spec_id") == "PRD-ANONYMOUS", "anonymous PRD spec_id must be PRD-ANONYMOUS")
    require(meta.get("status") == "Draft", "anonymous PRD status must remain Draft for RED planning")
    identity = prd.get("identity", {})
    require(isinstance(identity, dict), "anonymous PRD identity must be an object")
    require(identity.get("product_id") == "anonymous", "anonymous PRD identity.product_id must be anonymous")
    require(identity.get("owning_axis") == "community", "anonymous PRD owning_axis must be community")
    require(identity.get("context_model") == ["work"], "anonymous PRD context_model must be work-only")

    source_lock = prd.get("implementation_source_lock", {})
    require(isinstance(source_lock, dict), "anonymous PRD implementation_source_lock must be present")
    require(
        source_lock.get("status") == "implementation_ready_for_red_fixture_planning_only",
        "implementation_source_lock status must stay RED-planning-only",
    )
    source_authority = source_lock.get("source_authority", {})
    require(isinstance(source_authority, dict), "source_authority must be an object")
    require(source_authority.get("prd") == "specs/microservices/anonymous.json", "source_authority.prd drifted")
    require(source_authority.get("parent_inventory") == "oya/community/manifest.json", "source_authority parent inventory drifted")
    require("no standalone" in str(source_authority.get("boundary", "")).lower(), "source boundary must reject standalone anonymous service roots")
    require("oya/community/**" in str(source_lock.get("allowed_build_home", "")), "allowed build home must stay under oya/community/**")

    crate_shape = source_lock.get("crate_shape_decision", {})
    require(isinstance(crate_shape, dict), "crate_shape_decision must be an object")
    require(crate_shape.get("adr") == "ADR-0512", "crate shape decision must cite ADR-0512")
    require(crate_shape.get("default_crate_candidate") == "oya-community-anonymous", "default crate candidate drifted")
    require("layer-per-crate" in str(crate_shape.get("decision", "")).lower(), "crate decision must reject layer-per-crate scaffolding")

    retired = source_lock.get("retired_authority_mapping", {}).get("foundry", {})
    require(isinstance(retired, dict), "retired Foundry mapping must be present")
    require(retired.get("status") == "retired_historical_term", "Foundry mapping must stay retired historical term")
    require(retired.get("absorbed_by") == "intelligence", "Foundry successor must be intelligence")
    require("do not create active foundry" in str(retired.get("rule", "")).lower(), "Foundry rule must reject active backlog")

    red_fixture_list = source_lock.get("red_fixture_list")
    require(isinstance(red_fixture_list, list) and len(red_fixture_list) >= 7, "source lock must carry RED fixture list")
    red_text = text(red_fixture_list)
    for term in [
        "anonymous_author_token",
        "verification vault",
        "k>=10",
        "moderator queue",
        "two compliance officers",
        "work/personal context",
        "foundry/ai-substrate",
    ]:
        require(term.lower() in red_text, f"source-lock RED fixture list missing {term!r}")

    contracts = prd.get("contracts", {})
    require(isinstance(contracts, dict), "anonymous PRD contracts must be an object")
    require_contains_all(contracts.get("produces"), REQUIRED_PRODUCED_CONTRACTS, "anonymous PRD produced contracts")
    acs = prd.get("acceptance_criteria")
    require(isinstance(acs, list), "anonymous PRD acceptance_criteria must be a list")
    ac_ids = [str(item.get("id")) for item in acs if isinstance(item, dict)]
    require(ac_ids == REQUIRED_AC_IDS, f"anonymous PRD AC ids/order must be {REQUIRED_AC_IDS}; got {ac_ids}")
    personas = {str(item.get("persona")) for item in source_lock.get("user_story_browser_accessibility_matrix", []) if isinstance(item, dict)}
    require(REQUIRED_PERSONAS <= personas, f"source lock browser/accessibility matrix missing personas {sorted(REQUIRED_PERSONAS - personas)}")


def validate_manifest_index_source_lock(index: dict[str, Any]) -> None:
    rows = index.get("microservices")
    require(isinstance(rows, list), "manifests-index microservices must be a list")
    anonymous_rows = [row for row in rows if isinstance(row, dict) and row.get("name") == "anonymous"]
    require(len(anonymous_rows) == 1, f"manifests-index must contain exactly one anonymous row; got {len(anonymous_rows)}")
    row = anonymous_rows[0]
    require("manifest" not in row, "anonymous manifest-index row must not declare standalone manifest")
    require(row.get("prd") == "specs/microservices/anonymous.json", "anonymous manifest-index row must point to specs/microservices/anonymous.json")
    require(row.get("parent_inventory") == "oya/community/manifest.json", "anonymous parent_inventory drifted")
    require(row.get("subproduct_of") == "community", "anonymous must remain community subproduct")
    require(row.get("fd001_material") is False, "anonymous fd001_material must be false until source-locked Build changes it")
    require("no runtime/product-readiness claim" in str(row.get("authority_boundary", "")).lower(), "anonymous authority boundary must deny runtime/product-readiness claim")


def validate_parent_inventory(inventory: dict[str, Any]) -> None:
    require(inventory.get("microservice") == "community", "parent inventory manifest microservice must be community")
    pillars = inventory.get("product_pillars")
    require(isinstance(pillars, list), "community inventory product_pillars must be a list")
    pillar_text = text(pillars)
    require("teamblind-style-anonymous-workplace" in pillar_text, "community inventory must retain Teamblind-style anonymous workplace provenance")
    require("verified workplace affiliation" in pillar_text, "community inventory must retain verified workplace affiliation provenance")


def validate_forbidden_standalone_manifests_absent() -> None:
    for path in FORBIDDEN_STANDALONE_MANIFESTS:
        require(not path.exists(), f"standalone anonymous manifest must not exist: {rel(path)}")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "ANONYMOUS-PRD-RED-FIXTURE-CONTRACT-PLAN-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_fe98ab0a", "manifest must bind to kanban task t_fe98ab0a")
    require(manifest.get("parent_plan_spec_task") == "t_8da1d130", "manifest must bind to parent Plan/Spec task t_8da1d130")
    require("runtime" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny runtime claims")
    require("generated json hand edits" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must deny generated JSON hand edits")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")
    require_contains_all(manifest.get("required_prd_acceptance_criteria"), REQUIRED_AC_IDS, "required_prd_acceptance_criteria")
    require_contains_all(manifest.get("required_produced_contracts"), REQUIRED_PRODUCED_CONTRACTS, "required_produced_contracts")
    require(manifest.get("future_replay_root") == "specs/fixtures/anonymous-prd/replay/", "future_replay_root must be source-locked")
    require(manifest.get("default_future_build_home") == "oya/community/crates/oya-community-anonymous", "default future Build home must match source lock")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match anonymous RED plan; got {actual_ids}")
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        validate_future_replay_artifacts(fixture)

    require_fixture_sources(by_id["anonymous_prd_authority_source_lock_fixture"], ["anonymous.json#implementation_source_lock", "manifests-index", "oya/community/manifest.json", "kanban:t_8da1d130"], "authority source-lock fixture")
    require_fixture_terms(by_id["anonymous_prd_authority_source_lock_fixture"], "must_reject", ["standalone oya/anonymous", "microservices/anonymous", "runtime readiness from parent inventory"], "authority source-lock fixture")
    require_fixture_terms(by_id["anonymous_prd_post_token_only_fixture"], "must_assert", ["anonymous_author_token", "no user_id", "no identity FK", "no identity fields in search index"], "post token-only fixture")
    require_fixture_terms(by_id["anonymous_prd_identity_fk_leakage_rejection_fixture"], "must_reject", ["user_id", "identity_fk", "real identity", "salary records"], "identity/FK leakage fixture")
    require_fixture_terms(by_id["anonymous_prd_verification_vault_isolation_fixture"], "must_assert", ["one-time-use", "24h", "separate verification vault", "no FK to AnonPost"], "verification vault fixture")
    require_fixture_terms(by_id["anonymous_prd_hr_aggregate_k_anonymity_fixture"], "must_assert", ["aggregate-only", "k>=10", "403", "no individual attribution"], "HR aggregate fixture")
    require_fixture_terms(by_id["anonymous_prd_moderator_token_only_fixture"], "must_assert", ["moderator", "anonymous_author_token", "audit", "not real identity"], "moderator fixture")
    require_fixture_terms(by_id["anonymous_prd_legal_hold_four_eyes_fixture"], "must_assert", ["two compliance officers", "court-order", "HSM", "sealed audit package", "immutable audit-chain event"], "legal hold fixture")
    require_fixture_terms(by_id["anonymous_prd_work_personal_context_isolation_fixture"], "must_reject", ["personal anonymous boards", "personal feeds", "personal search", "personal suggestions", "work/personal context leakage"], "work/personal isolation fixture")
    require_fixture_terms(by_id["anonymous_prd_retired_authority_rejection_fixture"], "must_reject", ["Foundry", "AI-substrate", "active authority"], "retired authority fixture")
    require_fixture_terms(by_id["anonymous_prd_retired_authority_rejection_fixture"], "allowed_successors", ["intelligence", "Workflow", "Ontology"], "retired authority fixture")
    require_fixture_terms(by_id["anonymous_prd_browser_accessibility_evidence_fixture"], "must_assert", ["verified_employee", "moderator", "enterprise_hr_admin", "compliance_officer", "WCAG 2.2 AA", "keyboard", "screen-reader", "mobile", "desktop"], "browser/accessibility fixture")
    require_fixture_terms(by_id["anonymous_prd_build_parentage_fixture"], "must_assert", ["t_8da1d130", "t_fe98ab0a", "oya/community/**", "ADR-0512", "no generated JSON"], "build parentage fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    require("green ci alone" in lower_manifest, "manifest must reject green-CI-alone UI readiness")
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
            "RED: future anonymous PRD API/privacy/legal-hold replay artifacts are missing under "
            f"{rel(replay_root)}: {preview}{suffix}"
        )

    build_home = REPO_ROOT / "oya" / "community" / "crates" / "oya-community-anonymous"
    build_src = build_home / "src" / "lib.rs"
    require(build_src.exists(), "anonymous Build crate must exist under oya/community/crates/oya-community-anonymous")
    require((build_home / "Cargo.toml").exists(), "anonymous Build crate must include Cargo.toml")
    src_text = build_src.read_text(encoding="utf-8")
    for term in [
        "AnonymousPost",
        "AnonymousSalaryEntry",
        "VerificationToken",
        "LegalHoldIdentityRevealRequest",
        "anonymous_author_token",
        "IndividualAttributionForbidden",
        "PersonalAnonymousBoardForbidden",
    ]:
        require(term in src_text, f"anonymous Build crate missing implementation term {term!r}")

    for fixture_id, fixture in by_id.items():
        for raw in fixture["future_replay_artifacts"]:
            replay_path = REPO_ROOT / Path(str(raw))
            replay = load_json(replay_path, f"anonymous replay fixture {fixture_id}")
            require(replay.get("fixture_id") == fixture_id, f"{fixture_id} replay fixture_id drifted")
            require(replay.get("fixture_kind") == fixture.get("fixture_kind"), f"{fixture_id} replay fixture_kind drifted")
            require(replay.get("replay_status") == EXPECTED_REPLAY_STATUS, f"{fixture_id} replay_status drifted")
            require(replay.get("build_task") == "t_956bcd9c", f"{fixture_id} replay must bind BUILD task t_956bcd9c")
            require(replay.get("parent_red_task") == "t_fe98ab0a", f"{fixture_id} replay must bind RED parent t_fe98ab0a")
            require(replay.get("parent_plan_spec_task") == "t_8da1d130", f"{fixture_id} replay must bind PLAN/SPEC parent t_8da1d130")
            require(
                replay.get("implementation_home") == "oya/community/crates/oya-community-anonymous",
                f"{fixture_id} replay implementation_home must stay under oya/community/**",
            )
            claim_boundary = str(replay.get("claim_boundary", "")).lower()
            for term in ["no standalone anonymous service root", "no microservices/anonymous", "no generated json", "no production readiness claim"]:
                require(term in claim_boundary, f"{fixture_id} claim_boundary missing {term!r}")

            evidence = replay.get("implementation_evidence", {})
            require(isinstance(evidence, dict), f"{fixture_id} implementation_evidence must be an object")
            require(evidence.get("crate") == "oya-community-anonymous", f"{fixture_id} evidence crate drifted")
            require(evidence.get("source") == "oya/community/crates/oya-community-anonymous/src/lib.rs", f"{fixture_id} evidence source drifted")
            tests = evidence.get("tests")
            require(isinstance(tests, list) and tests, f"{fixture_id} must cite focused tests or explicit N/A evidence")
            for test_name in tests:
                test_label = str(test_name)
                require(
                    test_label in src_text or is_explicit_non_rust_evidence(test_label),
                    f"{fixture_id} cites evidence {test_label!r} that is neither a crate test nor explicit non-Rust evidence",
                )

            require_terms_in_list(replay.get("assertions_satisfied"), fixture.get("must_assert", []), f"{fixture_id} assertions_satisfied")
            require_terms_in_list(replay.get("negative_cases"), fixture.get("must_reject", []), f"{fixture_id} negative_cases")
            require_terms_in_list(replay.get("allowed_successors"), fixture.get("allowed_successors", []), f"{fixture_id} allowed_successors")

            replay_text = text(replay)
            for term in fixture.get("must_assert", []):
                require(str(term).lower() in replay_text, f"{fixture_id} replay missing asserted term {term!r}")
            for term in fixture.get("must_reject", []):
                require(str(term).lower() in replay_text, f"{fixture_id} replay missing rejection term {term!r}")
            for term in fixture.get("allowed_successors", []):
                require(str(term).lower() in replay_text, f"{fixture_id} replay missing allowed successor {term!r}")

    print("anonymous PRD Build replay contract passed")


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixtures.append(
            {
                "fixture_id": fixture_id,
                "fixture_kind": "self_test",
                "source_authority_refs": ["specs/microservices/anonymous.json#implementation_source_lock", "kanban:t_8da1d130"],
                "future_replay_artifacts": [f"specs/fixtures/anonymous-prd/replay/self-test/{fixture_id}.fixture.json"],
                "must_assert": ["self-test assertion"],
                "must_reject": ["self-test rejection"],
                "allowed_successors": ["intelligence", "Workflow", "Ontology"],
                "expected_red_status": EXPECTED_RED_STATUS,
            }
        )
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["anonymous_prd_authority_source_lock_fixture"]["source_authority_refs"] += ["specs/microservices/manifests-index.json#microservices[name=anonymous]", "oya/community/manifest.json"]
    by_id["anonymous_prd_authority_source_lock_fixture"]["must_reject"] = ["standalone oya/anonymous", "microservices/anonymous", "runtime readiness from parent inventory"]
    by_id["anonymous_prd_post_token_only_fixture"]["must_assert"] = ["anonymous_author_token", "no user_id", "no identity FK", "no identity fields in search index"]
    by_id["anonymous_prd_identity_fk_leakage_rejection_fixture"]["must_reject"] = ["user_id", "identity_fk", "real identity", "salary records"]
    by_id["anonymous_prd_verification_vault_isolation_fixture"]["must_assert"] = ["one-time-use", "24h", "separate verification vault", "no FK to AnonPost"]
    by_id["anonymous_prd_hr_aggregate_k_anonymity_fixture"]["must_assert"] = ["aggregate-only", "k>=10", "403", "no individual attribution"]
    by_id["anonymous_prd_moderator_token_only_fixture"]["must_assert"] = ["moderator", "anonymous_author_token", "audit", "not real identity"]
    by_id["anonymous_prd_legal_hold_four_eyes_fixture"]["must_assert"] = ["two compliance officers", "court-order", "HSM", "sealed audit package", "immutable audit-chain event"]
    by_id["anonymous_prd_work_personal_context_isolation_fixture"]["must_reject"] = ["personal anonymous boards", "personal feeds", "personal search", "personal suggestions", "work/personal context leakage"]
    by_id["anonymous_prd_retired_authority_rejection_fixture"]["must_reject"] = ["Foundry", "AI-substrate", "active authority"]
    by_id["anonymous_prd_browser_accessibility_evidence_fixture"]["must_assert"] = ["verified_employee", "moderator", "enterprise_hr_admin", "compliance_officer", "WCAG 2.2 AA", "keyboard", "screen-reader", "mobile", "desktop"]
    by_id["anonymous_prd_build_parentage_fixture"]["must_assert"] = ["t_8da1d130", "t_fe98ab0a", "oya/community/**", "ADR-0512", "no generated JSON"]
    return {
        "fixture_plan_id": "ANONYMOUS-PRD-RED-FIXTURE-CONTRACT-PLAN-001",
        "kanban_task": "t_fe98ab0a",
        "parent_plan_spec_task": "t_8da1d130",
        "claim_boundary": "metadata/fixture-only; no runtime handlers or production claim; no generated JSON hand edits",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "required_prd_acceptance_criteria": REQUIRED_AC_IDS[:],
        "required_produced_contracts": REQUIRED_PRODUCED_CONTRACTS[:],
        "future_replay_root": "specs/fixtures/anonymous-prd/replay/",
        "default_future_build_home": "oya/community/crates/oya-community-anonymous",
        "global_non_claims": ["green CI alone is insufficient"],
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
    validate_prd_source_lock(load_json(PRD_PATH, "anonymous PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_parent_inventory(load_json(PARENT_INVENTORY_PATH, "community parent inventory manifest"))
    validate_forbidden_standalone_manifests_absent()
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
    expect_rejected("missing AC id", lambda data: data.update({"required_prd_acceptance_criteria": ["AC-ANON-01"]}))
    expect_rejected("missing produced contract", lambda data: data.update({"required_produced_contracts": ["community.anonymous.post.v1"]}))
    expect_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/community/manifest.json"))
    expect_rejected("post token-only assertion gap", lambda data: data["fixtures"][1].update({"must_assert": ["anonymous_author_token"]}))
    expect_rejected("identity leakage rejection gap", lambda data: data["fixtures"][2].update({"must_reject": ["user_id"]}))
    expect_rejected("legal hold court order gap", lambda data: data["fixtures"][6].update({"must_assert": ["two compliance officers"]}))
    expect_rejected("retired authority successor gap", lambda data: data["fixtures"][8].update({"allowed_successors": ["intelligence"]}))
    expect_rejected("generated future replay artifact", lambda data: data["fixtures"][0].update({"future_replay_artifacts": ["specs/fixtures/anonymous-prd/replay/bad.generated.json"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("anonymous PRD RED fixture contract self-tests passed")


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
    validate_prd_source_lock(load_json(PRD_PATH, "anonymous PRD"))
    validate_manifest_index_source_lock(load_json(MANIFEST_INDEX_PATH, "manifests-index"))
    validate_parent_inventory(load_json(PARENT_INVENTORY_PATH, "community parent inventory manifest"))
    validate_forbidden_standalone_manifests_absent()
    by_id = validate_manifest(manifest)
    validate_replay_artifacts(by_id, replay_root)


if __name__ == "__main__":
    main()
