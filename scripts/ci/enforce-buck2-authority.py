#!/usr/bin/env python3
"""Fail closed when active CI/CD/build/script lanes regress from Buck2 to Cargo.

The scanner is intentionally policy-file driven so P0.0 additions must be
mapped into the automated chain instead of relying on operator memory.
Historical ADR prose is handled by explicit amendment markers; active lanes
are scanned for executable Cargo commands and legacy required contexts.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(
    __import__("os").environ.get("OYA_REPO_ROOT", Path(__file__).resolve().parents[2])
).resolve()
MATRIX_PATH = "specs/phase0-automation-matrix.json"
COVERAGE_REGISTRY_PATH = "specs/phase0-automation-coverage-registry.json"
PROW_PARITY_PATH = "specs/oya-ci-prow-capability-parity.json"
ROOT_HUB_PATH = "specs/root-hub-pointers.json"
BUCK2_AUTHORITY_ROW_ID = "AC-0.0-buck2-authority-no-cargo-regression"
BUCK2_AUTHORITY_VERIFICATION_COMMAND = "buck2 build //:buck2-authority-policy-check"
REQUIRED_PROW_BASELINE = {
    "repository": "https://github.com/kubernetes-sigs/prow",
    "docs": "https://docs.prow.k8s.io/docs/",
    "architecture_docs": "https://docs.prow.k8s.io/docs/overview/architecture/",
    "tide_docs": "https://docs.prow.k8s.io/docs/components/core/tide/",
}
REQUIRED_AUTHORITY_PRODUCER_TERMS = [
    "cloud-ci/oya-ci",
    "Rust Prow reimplementation",
    "trusted",
    "source-bound",
]
REQUIRED_PROW_CONTRACT_TERMS = [
    "Rust reimplementation",
    "improvement",
    "upstream Kubernetes Prow/Tide",
    "not a greenfield CI invention",
]
BUCK2_AUTHORITY_CLAIM_BOUNDARY_TERMS = [
    "not P0.0 green",
    "not protected-branch authority",
    "trusted cloud-ci/oya-ci",
]
REQUIRED_PROW_CAPABILITY_IDS = {
    "prow-hook-webhook-ingest",
    "prow-plugin-command-routing",
    "prow-prowjob-api-and-config",
    "prow-presubmit-jobs",
    "prow-postsubmit-jobs",
    "prow-periodic-jobs",
    "prow-batch-jobs",
    "prow-controller-manager-job-controller",
    "prow-crier-status-reporting",
    "prow-deck-web-ui",
    "prow-tide-merge-automation",
    "prow-sinker-gc",
    "prow-horologium-periodic-trigger",
    "prow-branchprotector",
    "prow-pod-utilities-clonerefs",
    "prow-pod-utilities-entrypoint-sidecar",
    "prow-artifact-storage",
    "prow-service-build-cluster-split",
    "prow-trusted-untrusted-execution",
    "prow-config-validation",
    "prow-label-approval-lgtm-policy",
    "prow-retest-and-trigger-policy",
    "prow-status-reconciliation",
    "prow-metrics-observability",
}
REQUIRED_IMPROVEMENT_IDS = {
    "rust-memory-safe-single-platform",
    "forgejo-native-no-github-gcs-coupling",
    "buck2-native-gate-execution",
    "self-hosted-artifact-storage",
    "tenant-isolated-trusted-controller",
    "source-bound-required-context",
    "buck2-native-llvm-coverage",
    "dual-cargo-buck2-mutation-testing-advisory",
}
ALLOWED_PARITY_SCOPES = {
    "direct_reimplementation",
    "equivalent_reimplementation",
    "improved_reimplementation",
    "superseded_by_improvement",
}
ALLOWED_PARITY_STATUSES = {
    "bridge_existing",
    "existing_partial",
    "phase0_contract",
    "phase1_target",
    "phase2_target",
    "phase3_target",
    "phase4_target",
}
REQUIRED_EXCLUDED_OR_SUPERSEDED_COMPONENT_IDS = {
    "prow-exporter",
    "prow-gcsupload",
    "prow-hmac",
    "prow-gerrit",
    "prow-tot",
    "prow-jenkins-operator",
}
ALLOWED_EXCLUDED_OR_SUPERSEDED_DISPOSITIONS = {
    "superseded_by_improvement",
    "out_of_scope_for_forgejo_native",
    "deferred_until_needed_with_waiver",
}


def load_policy(path: Path) -> dict:
    with path.open() as fh:
        return json.load(fh)


def load_json(path: str) -> dict:
    with (REPO_ROOT / path).open() as fh:
        return json.load(fh)


def find_by_id(items, item_id: str, field: str, failures: list[str]) -> dict | None:
    if not isinstance(items, list):
        failures.append(f"{field}: expected list")
        return None
    matches = [item for item in items if isinstance(item, dict) and item.get("id") == item_id]
    if len(matches) != 1:
        failures.append(f"{field}: expected exactly one {item_id} entry, found {len(matches)}")
        return None
    return matches[0]


def string_list(value) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, str)]


def object_list(value) -> list[dict]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, dict)]


def display_path(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def rel(path: str) -> Path:
    return REPO_ROOT / path


def iter_lines(path: Path):
    try:
        text = path.read_text(errors="replace")
    except FileNotFoundError:
        yield 0, "<missing>"
        return
    for index, line in enumerate(text.splitlines(), 1):
        yield index, line


def expand_policy_paths(policy: dict, file_key: str, glob_key: str) -> list[str]:
    paths: list[str] = []
    seen: set[str] = set()
    for file_name in policy.get(file_key, []):
        if file_name not in seen:
            paths.append(file_name)
            seen.add(file_name)
    for pattern in policy.get(glob_key, []):
        matches = sorted(
            path.as_posix()
            for path in REPO_ROOT.glob(pattern)
            if path.is_file()
        )
        if not matches:
            paths.append(f"<missing-glob:{pattern}>")
            continue
        for file_name in matches:
            if file_name not in seen:
                paths.append(file_name)
                seen.add(file_name)
    return paths


def forbidden_cargo_regex(subcommands: list[str]) -> re.Pattern[str]:
    joined = "|".join(re.escape(cmd) for cmd in subcommands)
    return re.compile(
        rf"(^|[;&|(`]|\s)cargo\s+(\+[^\s]+\s+)?({joined})(\s|$)",
        re.IGNORECASE,
    )


def validate_root_hub_pointer(root_hub: dict, failures: list[str]) -> None:
    entry_points = root_hub.get("entry_points")
    if not isinstance(entry_points, dict):
        failures.append(f"{ROOT_HUB_PATH}.entry_points: expected object")
        return
    entry = entry_points.get("oya_ci_prow_capability_parity")
    if not isinstance(entry, dict):
        failures.append(f"{ROOT_HUB_PATH}.entry_points must include oya_ci_prow_capability_parity")
    else:
        if entry.get("kind") != "spec":
            failures.append("root hub oya_ci_prow_capability_parity.kind must be spec")
        if entry.get("current_path") != f"/{PROW_PARITY_PATH}":
            failures.append(f"root hub oya_ci_prow_capability_parity.current_path must be /{PROW_PARITY_PATH}")
        purpose = entry.get("purpose")
        if not isinstance(purpose, str) or "Prow" not in purpose or "Rust" not in purpose:
            failures.append("root hub oya_ci_prow_capability_parity.purpose must describe Rust Prow parity")
    pointers = root_hub.get("pointers")
    if not isinstance(pointers, dict) or pointers.get("oya_ci_prow_capability_parity") != PROW_PARITY_PATH:
        failures.append(f"{ROOT_HUB_PATH}.pointers.oya_ci_prow_capability_parity must point to {PROW_PARITY_PATH}")


def validate_prow_parity_registry(registry: dict, failures: list[str]) -> None:
    claim_boundary = registry.get("claim_boundary")
    if not isinstance(claim_boundary, dict):
        failures.append(f"{PROW_PARITY_PATH}.claim_boundary: expected object")
    else:
        for key in [
            "p0_0_green",
            "phase0_complete",
            "live_full_parity_claimed",
            "protected_branch_authority_proven",
            "production_readiness",
            "hyperscaler_grade_readiness",
        ]:
            if claim_boundary.get(key) is not False:
                failures.append(f"{PROW_PARITY_PATH}.claim_boundary.{key} must be false")

    upstream_sources = registry.get("upstream_sources")
    if not isinstance(upstream_sources, dict):
        failures.append(f"{PROW_PARITY_PATH}.upstream_sources: expected object")
    else:
        source_key_map = {
            "repository": "repository",
            "docs": "documentation",
            "architecture_docs": "architecture",
            "tide_docs": "tide",
        }
        for key, expected in REQUIRED_PROW_BASELINE.items():
            source_key = source_key_map[key]
            if upstream_sources.get(source_key) != expected:
                failures.append(f"{PROW_PARITY_PATH}.upstream_sources.{source_key} must be {expected}")
        if upstream_sources.get("controller_manager") != "https://docs.prow.k8s.io/docs/components/":
            failures.append(f"{PROW_PARITY_PATH}.upstream_sources.controller_manager must cite the upstream components page")
        if upstream_sources.get("plank_deprecated") != "https://docs.prow.k8s.io/docs/components/deprecated/plank/":
            failures.append(f"{PROW_PARITY_PATH}.upstream_sources.plank_deprecated must cite deprecated Plank as legacy context only")

    required_capability_ids = set(string_list(registry.get("required_capability_ids")))
    missing_required_ids = sorted(REQUIRED_PROW_CAPABILITY_IDS - required_capability_ids)
    if missing_required_ids:
        failures.append(
            f"{PROW_PARITY_PATH}.required_capability_ids missing {', '.join(missing_required_ids)}"
        )

    capabilities = object_list(registry.get("capabilities"))
    capability_ids = {capability.get("id") for capability in capabilities if isinstance(capability.get("id"), str)}
    missing_capabilities = sorted(REQUIRED_PROW_CAPABILITY_IDS - capability_ids)
    if missing_capabilities:
        failures.append(f"{PROW_PARITY_PATH}.capabilities missing {', '.join(missing_capabilities)}")
    if len(capability_ids) != len(capabilities):
        failures.append(f"{PROW_PARITY_PATH}.capabilities must have unique string ids")

    for capability in capabilities:
        capability_id = capability.get("id")
        if not isinstance(capability_id, str):
            continue
        for field in [
            "upstream_feature",
            "upstream_source",
            "oya_ci_equivalent",
            "parity_requirement",
            "verification",
        ]:
            if not isinstance(capability.get(field), str) or not capability.get(field, "").strip():
                failures.append(f"{PROW_PARITY_PATH}.{capability_id}.{field}: expected non-empty string")
        if capability.get("parity_scope") not in ALLOWED_PARITY_SCOPES:
            failures.append(f"{PROW_PARITY_PATH}.{capability_id}.parity_scope is not allowed")
        if capability.get("current_status") not in ALLOWED_PARITY_STATUSES:
            failures.append(f"{PROW_PARITY_PATH}.{capability_id}.current_status is not allowed")
        if not string_list(capability.get("repo_artifacts")):
            failures.append(f"{PROW_PARITY_PATH}.{capability_id}.repo_artifacts must list local artifacts")
        if not string_list(capability.get("improvements_over_upstream")):
            failures.append(f"{PROW_PARITY_PATH}.{capability_id}.improvements_over_upstream must be non-empty")
        if capability.get("live_authority_claimed") is not False:
            failures.append(f"{PROW_PARITY_PATH}.{capability_id}.live_authority_claimed must be false")


    excluded = object_list(registry.get("excluded_or_superseded_upstream_components"))
    excluded_ids = {item.get("id") for item in excluded if isinstance(item.get("id"), str)}
    missing_excluded = sorted(REQUIRED_EXCLUDED_OR_SUPERSEDED_COMPONENT_IDS - excluded_ids)
    if missing_excluded:
        failures.append(
            f"{PROW_PARITY_PATH}.excluded_or_superseded_upstream_components missing {', '.join(missing_excluded)}"
        )
    for item in excluded:
        item_id = item.get("id")
        if not isinstance(item_id, str):
            failures.append(f"{PROW_PARITY_PATH}.excluded_or_superseded_upstream_components item missing string id")
            continue
        if item.get("disposition") not in ALLOWED_EXCLUDED_OR_SUPERSEDED_DISPOSITIONS:
            failures.append(f"{PROW_PARITY_PATH}.{item_id}.disposition is not allowed")
        for field in ["upstream_component", "upstream_source", "rationale", "replacement_or_reason"]:
            if not isinstance(item.get(field), str) or not item.get(field, "").strip():
                failures.append(f"{PROW_PARITY_PATH}.{item_id}.{field}: expected non-empty string")

    required_improvement_ids = set(string_list(registry.get("required_improvement_ids")))
    missing_improvement_ids = sorted(REQUIRED_IMPROVEMENT_IDS - required_improvement_ids)
    if missing_improvement_ids:
        failures.append(
            f"{PROW_PARITY_PATH}.required_improvement_ids missing {', '.join(missing_improvement_ids)}"
        )
    improvements = object_list(registry.get("improvements"))
    improvement_ids = {
        improvement.get("id") for improvement in improvements if isinstance(improvement.get("id"), str)
    }
    missing_improvements = sorted(REQUIRED_IMPROVEMENT_IDS - improvement_ids)
    if missing_improvements:
        failures.append(f"{PROW_PARITY_PATH}.improvements missing {', '.join(missing_improvements)}")
    for improvement in improvements:
        improvement_id = improvement.get("id")
        if not isinstance(improvement_id, str):
            continue
        if not isinstance(improvement.get("description"), str) or not improvement["description"].strip():
            failures.append(f"{PROW_PARITY_PATH}.{improvement_id}.description: expected non-empty string")
        mapped = set(string_list(improvement.get("capability_ids")))
        if not mapped:
            failures.append(f"{PROW_PARITY_PATH}.{improvement_id}.capability_ids must be non-empty")
        unknown = sorted(mapped - capability_ids)
        if unknown:
            failures.append(f"{PROW_PARITY_PATH}.{improvement_id}.capability_ids unknown: {', '.join(unknown)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--policy",
        default="specs/buck2-authority-policy.json",
        help="Path to the Buck2 authority policy JSON, relative to repo root by default.",
    )
    parser.add_argument("--matrix", default=MATRIX_PATH, help="Phase-0 automation matrix path.")
    parser.add_argument("--coverage-registry", default=COVERAGE_REGISTRY_PATH, help="Phase-0 coverage registry path.")
    parser.add_argument("--prow-parity-registry", default=PROW_PARITY_PATH, help="oya-ci Prow capability parity registry path.")
    parser.add_argument("--root-hub", default=ROOT_HUB_PATH, help="Root hub pointer registry path.")
    args = parser.parse_args()

    policy_path = Path(args.policy)
    if not policy_path.is_absolute():
        policy_path = REPO_ROOT / policy_path
    policy = load_policy(policy_path)

    failures: list[str] = []
    cargo_re = forbidden_cargo_regex(policy["forbidden_cargo_subcommands"])

    command_scan_files = expand_policy_paths(
        policy, "command_scan_files", "command_scan_globs"
    )
    for file_name in command_scan_files:
        if file_name.startswith("<missing-glob:"):
            failures.append(f"command-scan glob matched no files: {file_name[14:-1]}")
            continue
        file_path = rel(file_name)
        if not file_path.is_file():
            failures.append(f"missing command-scan file: {file_name}")
            continue
        for line_no, line in iter_lines(file_path):
            if cargo_re.search(line):
                failures.append(
                    f"{file_name}:{line_no}: forbidden Cargo executable lane: {line.strip()}"
                )

    forbidden_contexts = set(policy["forbidden_status_contexts"])
    for file_name in policy["status_context_scan_files"]:
        file_path = rel(file_name)
        if not file_path.is_file():
            failures.append(f"missing status-context-scan file: {file_name}")
            continue
        text = file_path.read_text(errors="replace")
        for context in sorted(forbidden_contexts):
            if context in text:
                failures.append(
                    f"{file_name}: forbidden legacy status context {context!r}; use oya-ci-required"
                )

    for file_name, anchors in policy["required_anchors"].items():
        file_path = rel(file_name)
        if not file_path.is_file():
            failures.append(f"missing required-anchor file: {file_name}")
            continue
        text = file_path.read_text(errors="replace")
        for anchor in anchors:
            if anchor not in text:
                failures.append(f"{file_name}: missing required Buck2 authority anchor {anchor!r}")

    for group in policy.get("required_glob_anchors", []):
        pattern = group["glob"]
        anchors = group["anchors"]
        matches = sorted(path for path in REPO_ROOT.glob(pattern) if path.is_file())
        if not matches:
            failures.append(f"required-anchor glob matched no files: {pattern}")
            continue
        for file_path in matches:
            file_name = file_path.relative_to(REPO_ROOT).as_posix()
            text = file_path.read_text(errors="replace")
            for anchor in anchors:
                if anchor not in text:
                    failures.append(
                        f"{file_name}: missing required Buck2 authority anchor {anchor!r}"
                    )

    amendment = policy["required_adr_amendment_text"]
    for file_name in policy["adr_amendment_files"]:
        file_path = rel(file_name)
        if not file_path.is_file():
            failures.append(f"missing ADR amendment file: {file_name}")
            continue
        text = file_path.read_text(errors="replace")
        if amendment not in text or "specs/buck2-authority-policy.json" not in text:
            failures.append(
                f"{file_name}: missing {amendment!r} and policy cross-reference"
            )

    release_exception_ids = {
        item["id"] for item in policy.get("allowed_cargo_exceptions", [])
    }
    if "production-release-image-binary-optimization" not in release_exception_ids:
        failures.append("policy lacks production release image/binary Cargo exception")
    if "buck2-graph-metadata-only" not in release_exception_ids:
        failures.append("policy lacks metadata-only Buck2 graph exception")

    upstream_prow_baseline = policy.get("upstream_prow_baseline")
    if not isinstance(upstream_prow_baseline, dict):
        failures.append("policy must record upstream Kubernetes Prow baseline for the Rust reimplementation")
    else:
        for key, expected in REQUIRED_PROW_BASELINE.items():
            if upstream_prow_baseline.get(key) != expected:
                failures.append(f"upstream_prow_baseline.{key} must be {expected}")
        contract = upstream_prow_baseline.get("contract")
        if not isinstance(contract, str):
            failures.append("upstream_prow_baseline.contract: expected string")
        else:
            for term in REQUIRED_PROW_CONTRACT_TERMS:
                if term not in contract:
                    failures.append(f"upstream_prow_baseline.contract missing {term!r}")
    target_authority = policy.get("target_authority")
    if not isinstance(target_authority, dict):
        failures.append("target_authority must be an object")
        producer = ""
    else:
        producer = target_authority.get("producer")
        if target_authority.get("required_context") != "oya-ci-required":
            failures.append("target_authority.required_context must be oya-ci-required")
    if not isinstance(producer, str):
        failures.append("target_authority.producer must be a string")
        producer = ""
    for term in REQUIRED_AUTHORITY_PRODUCER_TERMS:
        if term not in producer:
            failures.append(f"target_authority.producer must contain {term!r}")

    matrix = load_json(args.matrix)
    buck2_authority_row = find_by_id(
        matrix.get("seed_rows"),
        BUCK2_AUTHORITY_ROW_ID,
        f"{MATRIX_PATH}.seed_rows",
        failures,
    )
    if buck2_authority_row is not None:
        if buck2_authority_row.get("verification_command") != BUCK2_AUTHORITY_VERIFICATION_COMMAND:
            failures.append(
                f"{BUCK2_AUTHORITY_ROW_ID} row must record Buck2 authority verification command"
            )
        claim_boundary = buck2_authority_row.get("claim_boundary")
        if not isinstance(claim_boundary, str):
            failures.append(f"{BUCK2_AUTHORITY_ROW_ID}.claim_boundary: expected string")
        else:
            for term in BUCK2_AUTHORITY_CLAIM_BOUNDARY_TERMS:
                if term not in claim_boundary:
                    failures.append(f"{BUCK2_AUTHORITY_ROW_ID}.claim_boundary missing {term!r}")
        if buck2_authority_row.get("no_new_oya_cli_surface") is not True:
            failures.append(f"{BUCK2_AUTHORITY_ROW_ID}.no_new_oya_cli_surface must be true")

    prow_parity_path = rel(args.prow_parity_registry)
    if not prow_parity_path.is_file():
        failures.append(f"missing Prow parity registry: {args.prow_parity_registry}")
    else:
        validate_prow_parity_registry(load_json(args.prow_parity_registry), failures)

    validate_root_hub_pointer(load_json(args.root_hub), failures)

    coverage_registry = load_json(args.coverage_registry)
    ac_0_0_subject = find_by_id(
        coverage_registry.get("coverage_subjects"),
        "AC-0.0",
        f"{COVERAGE_REGISTRY_PATH}.coverage_subjects",
        failures,
    )
    if ac_0_0_subject is not None:
        mapped_rows = ac_0_0_subject.get("mapped_row_ids")
        if not isinstance(mapped_rows, list) or BUCK2_AUTHORITY_ROW_ID not in mapped_rows:
            failures.append(f"AC-0.0 coverage subject must map {BUCK2_AUTHORITY_ROW_ID}")
        verification_commands = ac_0_0_subject.get("verification_commands")
        if not isinstance(verification_commands, dict):
            failures.append("AC-0.0 coverage subject verification_commands must be an object")
        elif verification_commands.get(BUCK2_AUTHORITY_ROW_ID) != BUCK2_AUTHORITY_VERIFICATION_COMMAND:
            failures.append(
                f"AC-0.0 coverage subject must record Buck2 command for {BUCK2_AUTHORITY_ROW_ID}"
            )

    if failures:
        print("buck2-authority-policy: RED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(
        json.dumps(
            {
                "verdict": "PASS",
                "policy": display_path(policy_path),
                "command_scan_files": len(command_scan_files),
                "command_scan_globs": len(policy.get("command_scan_globs", [])),
                "status_context_scan_files": len(policy["status_context_scan_files"]),
                "adr_amendment_files": len(policy["adr_amendment_files"]),
                "authority_context": policy["target_authority"]["required_context"],
                "claim_boundary": policy["claim_boundary"],
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
