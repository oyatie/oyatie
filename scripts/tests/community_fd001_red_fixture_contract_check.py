#!/usr/bin/env python3
"""Fail-closed RED checker for the Community FD-001 fixture contract plan.

This is a metadata/fixture guard. It validates the RED fixture manifest derived
from the approved Community FD-001 Plan/Spec, then remains red until future Build
cards create source-backed API/data/event/schema contract artifacts.
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
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "fixtures" / "community-fd001" / "red-fixtures.json"
DEFAULT_CONTRACT_ROOT = REPO_ROOT / "contracts" / "community-fd001"

EXPECTED_FIXTURE_IDS = [
    "community_fd001_authority_boundary_fixture",
    "community_fd001_inventory_provenance_rejection_fixture",
    "community_fd001_mode_normalization_fixture",
    "community_fd001_retired_network_shorts_successor_fixture",
    "community_fd001_messenger_mail_separation_fixture",
    "community_fd001_workflow_ops_ontology_separation_fixture",
    "community_fd001_tenant_rbac_context_isolation_fixture",
    "community_fd001_data_class_retention_legal_hold_fixture",
    "community_fd001_anonymity_privacy_fixture",
    "community_fd001_social_media_privacy_fixture",
    "community_fd001_api_schema_event_contract_fixture",
    "community_fd001_observability_slo_runbook_fixture",
    "community_fd001_localization_kr_fixture",
    "community_fd001_ux_accessibility_evidence_fixture",
    "community_fd001_generated_face_no_hand_edit_fixture",
    "community_fd001_build_parentage_fixture",
]

REQUIRED_TOP_LEVEL_SOURCES = {
    "specs/microservices/community.json",
    "plan/community/community-fd001-service-boundary-plan-spec.md",
    "docs/decisions/ADR-0217-vertical-slice-rollout-order.md",
    "docs/decisions/ADR-0234-connect-social-expansion-planning-contract.md",
    "specs/microservices/social.json",
    "specs/microservices/anonymous.json",
}

INVENTORY_NOT_AUTHORITY_MARKERS = {
    "oya/community/manifest.json",
    "registry/catalog/oya-community-*.yaml",
    "tasks/community-*.md",
    "specs/proto/backbone/community/community_post_store.proto",
    "oya/community/contracts/*",
}

EXPECTED_RED_STATUS = "RED_UNTIL_CONTRACT_ARTIFACT_EXISTS"
GENERATED_SUFFIX = ".generated.json"


def fail(message: str) -> NoReturn:
    print(f"community FD-001 RED fixture contract check failed: {message}", file=sys.stderr)
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


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        candidate = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing manifest: {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON manifest {rel(path)}: {exc}")
    require(isinstance(candidate, dict), "manifest must be a JSON object")
    return candidate


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


def require_contains_all(haystack: object, needles: set[str], label: str) -> None:
    values = set(str(item) for item in haystack) if isinstance(haystack, list) else set()
    missing = sorted(needles - values)
    require(not missing, f"{label} missing {missing}")


def require_fixture_rejects(fixture: dict[str, Any], required_terms: list[str], label: str) -> None:
    haystack = text(fixture.get("must_reject", []))
    missing = [term for term in required_terms if term.lower() not in haystack]
    require(not missing, f"{label} must_reject missing terms {missing}")


def require_fixture_sources(fixture: dict[str, Any], required_sources: list[str], label: str) -> None:
    values = set(str(item) for item in fixture.get("source_authority_refs", []))
    missing = [source for source in required_sources if not any(source in value for value in values)]
    require(not missing, f"{label} source_authority_refs missing {missing}")


def validate_future_artifacts(fixture: dict[str, Any]) -> None:
    artifacts = fixture.get("future_contract_artifacts")
    require(isinstance(artifacts, list) and artifacts, f"{fixture.get('fixture_id')} must name future_contract_artifacts")
    for raw in artifacts:
        artifact = str(raw)
        require(artifact.startswith("contracts/community-fd001/"), f"future artifact must stay under contracts/community-fd001/: {artifact}")
        require(not artifact.endswith(GENERATED_SUFFIX), f"future artifact must not be a hand-edited generated face: {artifact}")
        require(".." not in Path(artifact).parts, f"future artifact must not traverse directories: {artifact}")


def validate_manifest(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    require(manifest.get("fixture_plan_id") == "COMMUNITY-FD001-RED-FIXTURE-CONTRACT-PLAN-001", "unexpected fixture_plan_id")
    require(manifest.get("kanban_task") == "t_f07b0559", "manifest must bind to kanban task t_f07b0559")
    require(manifest.get("parent_plan_spec_task") == "t_3321dc87", "manifest must bind to parent Plan/Spec task t_3321dc87")
    require("runtime" in str(manifest.get("claim_boundary", "")).lower(), "claim_boundary must explicitly deny runtime claims")
    require_contains_all(manifest.get("source_authority_refs"), REQUIRED_TOP_LEVEL_SOURCES, "source_authority_refs")
    require_contains_all(manifest.get("inventory_context_not_live_authority"), INVENTORY_NOT_AUTHORITY_MARKERS, "inventory_context_not_live_authority")

    by_id = fixture_by_id(manifest)
    actual_ids = list(by_id)
    require(actual_ids == EXPECTED_FIXTURE_IDS, f"fixture ids/order must exactly match parent Plan/Spec; got {actual_ids}")

    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = by_id[fixture_id]
        require(fixture.get("expected_red_status") == EXPECTED_RED_STATUS, f"{fixture_id} must remain {EXPECTED_RED_STATUS}")
        require(
            str(fixture.get("source_plan_spec_lines", "")).startswith(
                "plan/community/community-fd001-service-boundary-plan-spec.md:"
            ),
            f"{fixture_id} must cite parent Plan/Spec fixture-family lines",
        )
        validate_future_artifacts(fixture)

    require_fixture_sources(by_id["community_fd001_authority_boundary_fixture"], ["specs/microservices/community.json", "ADR-0217", "ADR-0234"], "authority boundary fixture")
    require_fixture_rejects(by_id["community_fd001_inventory_provenance_rejection_fixture"], ["oya/community/manifest.json", "registry/catalog", "tasks/community", "proto"], "inventory provenance rejection fixture")
    require_fixture_rejects(by_id["community_fd001_messenger_mail_separation_fixture"], ["messenger", "mail", "message delivery", "mail import"], "messenger/mail separation fixture")
    require_fixture_rejects(by_id["community_fd001_workflow_ops_ontology_separation_fixture"], ["workflow-engine", "workflow-studio", "ops-dashboard", "control-center", "ontology", "intelligence", "infra"], "workflow/ops/ontology separation fixture")
    require_fixture_rejects(by_id["community_fd001_retired_network_shorts_successor_fixture"], ["standalone network", "standalone shorts"], "retired network/shorts successor fixture")
    require_fixture_sources(by_id["community_fd001_retired_network_shorts_successor_fixture"], ["specs/microservices/community.json", "specs/microservices/social.json"], "retired network/shorts successor fixture")
    require_fixture_sources(by_id["community_fd001_social_media_privacy_fixture"], ["specs/microservices/social.json"], "social media privacy fixture")
    require_fixture_rejects(by_id["community_fd001_social_media_privacy_fixture"], ["feed blending", "ad-signal", "biometric", "shorts standalone"], "social media privacy fixture")
    require_fixture_sources(by_id["community_fd001_anonymity_privacy_fixture"], ["specs/microservices/anonymous.json"], "anonymity privacy fixture")
    require_fixture_rejects(by_id["community_fd001_anonymity_privacy_fixture"], ["real identity", "personal anonymous", "employer individual", "moderator real-identity", "four-eyes"], "anonymity privacy fixture")
    require_fixture_rejects(by_id["community_fd001_generated_face_no_hand_edit_fixture"], ["*.generated.json", "hand edit"], "generated face fixture")
    require_fixture_rejects(by_id["community_fd001_build_parentage_fixture"], ["t_3321dc87", "t_f07b0559", "allowed path", "generated-face"], "build parentage fixture")

    lower_manifest = text(manifest)
    require("passed_after_future_runtime_evidence" not in lower_manifest, "manifest must not fabricate a green future-runtime status")
    return by_id


def validate_contract_artifacts(by_id: dict[str, dict[str, Any]], contract_root: Path) -> None:
    missing: list[str] = []
    for fixture in by_id.values():
        for raw in fixture["future_contract_artifacts"]:
            rel_path = Path(str(raw))
            expected = REPO_ROOT / rel_path
            if not expected.exists():
                missing.append(str(rel_path))
    if missing:
        preview = ", ".join(missing[:8])
        suffix = "" if len(missing) <= 8 else f" ... (+{len(missing) - 8} more)"
        fail(
            "RED: future community FD-001 API/data/event/schema contract artifacts are missing under "
            f"{rel(contract_root)}: {preview}{suffix}"
        )
    fail("future contract replay is not implemented; this RED-only checker must be extended by a Build card before green status")


def baseline_manifest() -> dict[str, Any]:
    fixtures = []
    for fixture_id in EXPECTED_FIXTURE_IDS:
        fixture = {
            "fixture_id": fixture_id,
            "fixture_kind": "self_test",
            "source_plan_spec_lines": "plan/community/community-fd001-service-boundary-plan-spec.md:313-333",
            "source_authority_refs": ["specs/microservices/community.json", "docs/decisions/ADR-0217-vertical-slice-rollout-order.md", "docs/decisions/ADR-0234-connect-social-expansion-planning-contract.md"],
            "future_contract_artifacts": [f"contracts/community-fd001/self-test/{fixture_id}.fixture.json"],
            "must_reject": ["self-test forbidden pattern"],
            "expected_red_status": EXPECTED_RED_STATUS,
        }
        fixtures.append(fixture)
    by_id = {fixture["fixture_id"]: fixture for fixture in fixtures}
    by_id["community_fd001_inventory_provenance_rejection_fixture"]["must_reject"] = ["oya/community/manifest.json", "registry/catalog", "tasks/community", "proto"]
    by_id["community_fd001_messenger_mail_separation_fixture"]["must_reject"] = ["messenger", "mail", "message delivery", "mail import"]
    by_id["community_fd001_workflow_ops_ontology_separation_fixture"]["must_reject"] = ["workflow-engine", "workflow-studio", "ops-dashboard", "control-center", "ontology", "intelligence", "infra"]
    by_id["community_fd001_retired_network_shorts_successor_fixture"]["source_authority_refs"] += ["specs/microservices/social.json"]
    by_id["community_fd001_retired_network_shorts_successor_fixture"]["must_reject"] = ["standalone network", "standalone shorts"]
    by_id["community_fd001_social_media_privacy_fixture"]["source_authority_refs"] += ["specs/microservices/social.json"]
    by_id["community_fd001_social_media_privacy_fixture"]["must_reject"] = ["feed blending", "ad-signal", "biometric", "shorts standalone"]
    by_id["community_fd001_anonymity_privacy_fixture"]["source_authority_refs"] += ["specs/microservices/anonymous.json"]
    by_id["community_fd001_anonymity_privacy_fixture"]["must_reject"] = ["real identity", "personal anonymous", "employer individual", "moderator real-identity", "four-eyes"]
    by_id["community_fd001_generated_face_no_hand_edit_fixture"]["must_reject"] = ["*.generated.json", "hand edit"]
    by_id["community_fd001_build_parentage_fixture"]["must_reject"] = ["t_3321dc87", "t_f07b0559", "allowed path", "generated-face"]
    return {
        "fixture_plan_id": "COMMUNITY-FD001-RED-FIXTURE-CONTRACT-PLAN-001",
        "kanban_task": "t_f07b0559",
        "parent_plan_spec_task": "t_3321dc87",
        "claim_boundary": "metadata/fixture-only; no runtime handlers or production claim",
        "source_authority_refs": sorted(REQUIRED_TOP_LEVEL_SOURCES),
        "inventory_context_not_live_authority": sorted(INVENTORY_NOT_AUTHORITY_MARKERS),
        "fixtures": fixtures,
    }


def run_self_tests(live_manifest: dict[str, Any]) -> None:
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
    expect_rejected("inventory live-authority marker missing", lambda data: data["inventory_context_not_live_authority"].remove("oya/community/manifest.json"))
    expect_rejected("messenger/mail conflation gap", lambda data: data["fixtures"][4].update({"must_reject": ["messenger"]}))
    expect_rejected("workflow/ops conflation gap", lambda data: data["fixtures"][5].update({"must_reject": ["workflow-engine"]}))
    expect_rejected("social mode source missing", lambda data: data["fixtures"][9].update({"source_authority_refs": ["specs/microservices/community.json"]}))
    expect_rejected("anonymous mode source missing", lambda data: data["fixtures"][8].update({"source_authority_refs": ["specs/microservices/community.json"]}))
    expect_rejected("generated future artifact", lambda data: data["fixtures"][0].update({"future_contract_artifacts": ["contracts/community-fd001/bad.generated.json"]}))
    expect_rejected("fabricated green status", lambda data: data["fixtures"][0].update({"expected_red_status": "GREEN"}))
    print("community FD-001 RED fixture contract self-tests passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST), help="RED fixture manifest JSON path")
    parser.add_argument("--contract-root", default=str(DEFAULT_CONTRACT_ROOT), help="future contract artifact root")
    parser.add_argument("--self-test", action="store_true", help="run fail-closed validator self-tests")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    manifest_path = Path(args.manifest)
    if not manifest_path.is_absolute():
        manifest_path = REPO_ROOT / manifest_path
    contract_root = Path(args.contract_root)
    if not contract_root.is_absolute():
        contract_root = REPO_ROOT / contract_root
    manifest = load_manifest(manifest_path)
    if args.self_test:
        run_self_tests(manifest)
        return
    by_id = validate_manifest(manifest)
    validate_contract_artifacts(by_id, contract_root)


if __name__ == "__main__":
    main()
