#!/usr/bin/env python3
"""Validate the temporary GitHub/GitHub Actions lane-unlocker bridge contract.

This checker intentionally separates the short-lived GitHub/GitHub Actions
SCM/CI/CD bridge from the destination Oyatie-native SCM/CI/CD seams. It is
local/static evidence only: it does not mutate branch protection, does not post
statuses, and does not claim P0.0 green.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(os.environ.get("OYA_REPO_ROOT", Path(__file__).resolve().parents[2])).resolve()
SPEC_PATH = REPO_ROOT / "specs/github-lane-unlocker-bridge.json"
WORKFLOW_PATH = REPO_ROOT / ".github/workflows/github-lane-unlocker-ci-cd.yml"
BOOTSTRAP_PATH = REPO_ROOT / "scripts/ci/github-actions-lane-unlocker-bootstrap.sh"
RUST_TOOLCHAIN_PATH = REPO_ROOT / "rust-toolchain.toml"
BRANCH_PROTECTION_JSON = REPO_ROOT / "infra/branch-protection/dev.json"
BRANCH_PROTECTION_YAML = REPO_ROOT / ".github/branch-protection.yaml"
ROOT_HUB = REPO_ROOT / "specs/root-hub-pointers.json"
BUCK2_POLICY = REPO_ROOT / "specs/buck2-authority-policy.json"
ADR_PATH = REPO_ROOT / "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md"
PROCEDURE_PATH = REPO_ROOT / "docs/ci/github-actions-lane-unlocker.md"


def load_json(path: Path, failures: list[str]) -> Any:
    if not path.exists():
        failures.append(f"{path.relative_to(REPO_ROOT)}: missing")
        return {}
    try:
        with path.open() as fh:
            return json.load(fh)
    except json.JSONDecodeError as exc:
        failures.append(f"{path.relative_to(REPO_ROOT)}: invalid JSON: {exc}")
        return {}


def read_text(path: Path, failures: list[str]) -> str:
    if not path.exists():
        failures.append(f"{path.relative_to(REPO_ROOT)}: missing")
        return ""
    return path.read_text()


def require(condition: bool, failures: list[str], message: str) -> None:
    if not condition:
        failures.append(message)


def require_contains(text: str, needle: str, failures: list[str], label: str) -> None:
    if needle not in text:
        failures.append(f"{label}: missing {needle!r}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true", help="print machine-readable result")
    args = parser.parse_args()

    failures: list[str] = []
    spec = load_json(SPEC_PATH, failures)
    workflow = read_text(WORKFLOW_PATH, failures)
    bootstrap = read_text(BOOTSTRAP_PATH, failures)
    rust_toolchain = read_text(RUST_TOOLCHAIN_PATH, failures)
    branch_json = load_json(BRANCH_PROTECTION_JSON, failures)
    branch_yaml = read_text(BRANCH_PROTECTION_YAML, failures)
    root_hub = load_json(ROOT_HUB, failures)
    buck2_policy = load_json(BUCK2_POLICY, failures)
    adr = read_text(ADR_PATH, failures)
    procedure = read_text(PROCEDURE_PATH, failures)

    bridge = spec.get("github_bridge", {}) if isinstance(spec, dict) else {}
    native = spec.get("native_destination_seams", {}) if isinstance(spec, dict) else {}
    pattern_strategy = spec.get("pattern_adoption_strategy", {}) if isinstance(spec, dict) else {}
    hyperscaler_fit = spec.get("cloud_native_hyperscaler_fit", {}) if isinstance(spec, dict) else {}
    alternatives = spec.get("alternatives_and_counterarguments", []) if isinstance(spec, dict) else []
    substrate_decoupling = spec.get("auth_shared_substrate_decoupling", {}) if isinstance(spec, dict) else {}
    boundary = spec.get("claim_boundary", {}) if isinstance(spec, dict) else {}
    lane_graph = spec.get("lane_graph", {}) if isinstance(spec, dict) else {}
    official_sources = spec.get("official_sources", []) if isinstance(spec, dict) else []
    non_interim = spec.get("not_interim_authorities", {}) if isinstance(spec, dict) else {}
    cd_bridge = spec.get("github_actions_cd_bridge", {}) if isinstance(spec, dict) else {}

    required_context = bridge.get("required_context")
    workflow_path = bridge.get("workflow_path")

    require(spec.get("bridge_status") == "temporary_github_actions_scm_cicd_lane_unlocker_not_native_authority", failures, "spec.bridge_status must be temporary_github_actions_scm_cicd_lane_unlocker_not_native_authority")
    require(bridge.get("temporary") is True, failures, "github_bridge.temporary must be true")
    require(bridge.get("permanent_first_class") is False, failures, "github_bridge.permanent_first_class must be false")
    require(bridge.get("interim_scm") == "github", failures, "github_bridge.interim_scm must be github")
    require(bridge.get("interim_ci") == "github_actions", failures, "github_bridge.interim_ci must be github_actions")
    require(bridge.get("interim_cd") == "github_actions", failures, "github_bridge.interim_cd must be github_actions")
    require(bridge.get("is_destination_scm") is False, failures, "github_bridge.is_destination_scm must be false")
    require(bridge.get("is_destination_ci") is False, failures, "github_bridge.is_destination_ci must be false")
    require(bridge.get("is_destination_cd") is False, failures, "github_bridge.is_destination_cd must be false")
    require(required_context == "github-lane-unlocker-required", failures, "github_bridge.required_context must be github-lane-unlocker-required")
    require(required_context != "oya-ci-required", failures, "temporary GitHub bridge must not reuse destination oya-ci-required context")
    require(workflow_path == ".github/workflows/github-lane-unlocker-ci-cd.yml", failures, "github_bridge.workflow_path must point to .github/workflows/github-lane-unlocker-ci-cd.yml")
    require(bridge.get("branch_protection_application") == "live_dev_required_context_converged_to_github_lane_unlocker_required_until_native_cutover", failures, "github_bridge.branch_protection_application must mark the live temporary required context convergence")
    require(bridge.get("manual_oya_ci_required_bridge_allowed") is False, failures, "manual oya-ci-required bridge must be disabled during the GitHub lane unlocker")
    require(bridge.get("native_cutover_target_context") == "oya-ci-required", failures, "github_bridge.native_cutover_target_context must remain oya-ci-required")

    js_runtime = bridge.get("javascript_action_runtime", {}) if isinstance(bridge.get("javascript_action_runtime"), dict) else {}
    require(js_runtime.get("checkout_action_ref") == "actions/checkout@v6", failures, "github_bridge.javascript_action_runtime.checkout_action_ref must use actions/checkout@v6")
    require(js_runtime.get("latest_checkout_release_verified") == "v6.0.3", failures, "github_bridge.javascript_action_runtime.latest_checkout_release_verified must record the verified v6 latest release")
    require(js_runtime.get("force_node24_runtime") is True, failures, "github_bridge.javascript_action_runtime.force_node24_runtime must be true")
    require(js_runtime.get("force_node24_env") == "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24", failures, "github_bridge.javascript_action_runtime.force_node24_env must name the GitHub Actions Node24 opt-in")
    require(js_runtime.get("node26_action_runtime_used") is False, failures, "github_bridge.javascript_action_runtime.node26_action_runtime_used must remain false for JavaScript action runtime")
    require(js_runtime.get("unsecure_node20_opt_out_allowed") is False, failures, "github_bridge.javascript_action_runtime.unsecure_node20_opt_out_allowed must be false")
    require(cd_bridge.get("mode") == "github_actions_cd_bridge_until_release_conveyor_cutover", failures, "github_actions_cd_bridge.mode must be temporary GitHub Actions CD bridge")
    require(cd_bridge.get("live_deployments_enabled") is False, failures, "github_actions_cd_bridge.live_deployments_enabled must be false")
    require(cd_bridge.get("deploys_via_retired_external_substrate") is False or (cd_bridge.get("deploys_via_jenkins") is False and cd_bridge.get("deploys_via_argocd") is False), failures, "github_actions_cd_bridge must not deploy through retired external SCM/CI/CD substrates")
    require(cd_bridge.get("cutover_destination") == "release_conveyor_cd", failures, "github_actions_cd_bridge.cutover_destination must be release_conveyor_cd")

    for forbidden, value in non_interim.items():
        require(value is False, failures, f"not_interim_authorities.{forbidden} must be false")
    require(non_interim.get("retired_external_scm_ci_cd_substrates") is False or all(non_interim.get(required_key) is False for required_key in ["jenkins", "forgejo", "argocd"]), failures, "not_interim_authorities must reject retired external SCM/CI/CD substrates")

    for seam in [
        "oyatie_scm",
        "cloud_workspace_service",
        "rust_prow_oya_ci",
        "buck2_execution",
        "llvm_source_based_coverage",
        "release_conveyor_cd",
    ]:
        require(seam in native, failures, f"native_destination_seams missing {seam}")
    scm = native.get("oyatie_scm", {}) if isinstance(native.get("oyatie_scm"), dict) else {}
    require(scm.get("decision") == "pure_rust_sapling_compatible_native_scm", failures, "native_destination_seams.oyatie_scm.decision must be pure_rust_sapling_compatible_native_scm")
    require(scm.get("implementation_strategy") == "adopt_existing_hyperscaler_patterns_not_wholesale_reimplementation", failures, "native_destination_seams.oyatie_scm must adopt existing patterns instead of reinventing the wheel")
    require(scm.get("durable_language") == "rust", failures, "native_destination_seams.oyatie_scm.durable_language must be rust")
    require(scm.get("upstream_sapling_role") == "behavioral_reference_not_permanent_fork_authority", failures, "native_destination_seams.oyatie_scm.upstream_sapling_role must be behavioral reference, not fork authority")
    require(scm.get("python_cpp_in_durable_path") is False, failures, "native_destination_seams.oyatie_scm.python_cpp_in_durable_path must be false")

    require(pattern_strategy.get("mode") == "best_of_existing_systems_not_wholesale_reimplementation", failures, "pattern_adoption_strategy.mode must reject wholesale reinvention")
    for source in ["prow", "sapling", "piper", "citc", "github_actions", "buck2", "kubernetes"]:
        require(source in pattern_strategy.get("source_systems", []), failures, f"pattern_adoption_strategy.source_systems missing {source}")
    for required in ["disjoint_merge_pools", "stacked_changes", "cloud_workspaces", "required_status_rollup", "affected_builds", "kubernetes_native_job_execution", "source_based_coverage"]:
        require(required in pattern_strategy.get("adopted_patterns", []), failures, f"pattern_adoption_strategy.adopted_patterns missing {required}")
    require(pattern_strategy.get("wholesale_clone_or_reimplementation") is False, failures, "pattern_adoption_strategy.wholesale_clone_or_reimplementation must be false")

    require(hyperscaler_fit.get("cloud_native") is True, failures, "cloud_native_hyperscaler_fit.cloud_native must be true")
    require(hyperscaler_fit.get("kubernetes_native") is True, failures, "cloud_native_hyperscaler_fit.kubernetes_native must be true")
    require(hyperscaler_fit.get("hyperscaler_native") is True, failures, "cloud_native_hyperscaler_fit.hyperscaler_native must be true")
    require(hyperscaler_fit.get("parallel_lane_ready") is True, failures, "cloud_native_hyperscaler_fit.parallel_lane_ready must be true")
    require(hyperscaler_fit.get("pod_shutdown_policy") == "scale_controllers_to_zero_not_delete_pods_blindly", failures, "cloud_native_hyperscaler_fit must prefer controller scale-to-zero over blind pod deletion")
    cncf = hyperscaler_fit.get("cncf_cloud_native", {}) if isinstance(hyperscaler_fit.get("cncf_cloud_native"), dict) else {}
    require(cncf.get("loosely_coupled_microservices") is True, failures, "cloud_native_hyperscaler_fit.cncf_cloud_native.loose microservice coupling must be true")
    require(cncf.get("interoperate_secure_resilient_manageable_observable") is True, failures, "cloud_native_hyperscaler_fit.cncf_cloud_native must preserve CNCF interoperability properties")
    require(cncf.get("separation_of_concerns") is True, failures, "cloud_native_hyperscaler_fit.cncf_cloud_native.separation_of_concerns must be true")
    for property_name in ["distributable", "observable", "portable", "interoperable", "available"]:
        require(cncf.get(property_name) is True, failures, f"cloud_native_hyperscaler_fit.cncf_cloud_native.{property_name} must be true")

    alt_ids = {item.get("id") for item in alternatives if isinstance(item, dict)}
    for alt in ["github_only_permanent", "upstream_prow_as_is", "sapling_fork_without_rust_boundary", "build_from_scratch_ignoring_proven_patterns"]:
        require(alt in alt_ids, failures, f"alternatives_and_counterarguments missing {alt}")


    require(substrate_decoupling.get("status") == "cloud_and_oyatie_auth_shared_substrates_decoupled_now", failures, "auth_shared_substrate_decoupling.status must decouple cloud and Oyatie auth/shared substrates now")
    require(substrate_decoupling.get("no_shared_contract_or_surface_now") is True, failures, "auth_shared_substrate_decoupling.no_shared_contract_or_surface_now must be true")
    require(substrate_decoupling.get("higher_concurrency_expected") is True, failures, "auth_shared_substrate_decoupling.higher_concurrency_expected must be true")
    require(substrate_decoupling.get("conflict_avoidance") == "separate_contract_files_schemas_and_runtime_surfaces", failures, "auth_shared_substrate_decoupling.conflict_avoidance must separate contract files, schemas, and runtime surfaces")
    require(substrate_decoupling.get("future_integration") == "rewrite_and_rewire_oyatie_products_to_consume_cloud_idp_after_cloud_substrate_stabilizes", failures, "auth_shared_substrate_decoupling.future_integration must record later rewrite/rewire through Cloud IdP")
    boundaries = substrate_decoupling.get("boundaries", {}) if isinstance(substrate_decoupling.get("boundaries"), dict) else {}
    for boundary_name in ["cloud_identity_shared", "oyatie_product_identity_shared"]:
        boundary_cfg = boundaries.get(boundary_name, {}) if isinstance(boundaries.get(boundary_name), dict) else {}
        require(boundary_cfg.get("own_contracts") is True, failures, f"auth_shared_substrate_decoupling.boundaries.{boundary_name}.own_contracts must be true")
        require(boundary_cfg.get("own_runtime_surface") is True, failures, f"auth_shared_substrate_decoupling.boundaries.{boundary_name}.own_runtime_surface must be true")
        require(boundary_cfg.get("no_cross_lane_shared_schema") is True, failures, f"auth_shared_substrate_decoupling.boundaries.{boundary_name}.no_cross_lane_shared_schema must be true")

    for family in ["product", "cloud_platform", "scm_ci_cd", "governance"]:
        require(family in lane_graph.get("families", {}), failures, f"lane_graph.families missing {family}")
    require(lane_graph.get("product_lanes_must_not_depend_on") == ["github temporary bridge internals", "retired external SCM bridge internals", "cloud-ci internals", "workspace substrate internals"], failures, "lane_graph.product_lanes_must_not_depend_on must keep product lanes platform-agnostic")

    require(boundary.get("p0_0_green") is False, failures, "claim_boundary.p0_0_green must be false")
    require(boundary.get("phase0_complete") is False, failures, "claim_boundary.phase0_complete must be false")
    require(boundary.get("github_is_permanent") is False, failures, "claim_boundary.github_is_permanent must be false")
    require(boundary.get("native_ci_authority_proven") is False, failures, "claim_boundary.native_ci_authority_proven must be false")
    forbidden = set(boundary.get("forbidden_claims", []))
    for claim in [
        "GitHub is the permanent SCM",
        "GitHub Actions is the permanent CI authority",
        "GitHub Actions is the permanent CD authority",
        "retired external SCM/CI/CD substrates are interim authorities",
        "P0.0 green",
        "Phase 0 complete",
        "cloud-ci/oya-ci live authority proven",
    ]:
        require(claim in forbidden, failures, f"claim_boundary.forbidden_claims missing {claim!r}")

    source_urls = {src.get("url") for src in official_sources if isinstance(src, dict)}
    for url in [
        "https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax",
        "https://docs.github.com/en/actions/reference/github-hosted-runners-reference",
        "https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/run-job-variations",
        "https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency",
        "https://docs.github.com/en/actions/reference/workflows-and-actions/reusing-workflow-configurations",
        "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches",
        "https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/merging-a-pull-request-with-a-merge-queue",
        "https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/automatically-merging-a-pull-request",
        "https://docs.prow.k8s.io/docs/overview/",
        "https://buck2.build/docs/users/commands/build/",
        "https://doc.rust-lang.org/rustc/instrument-coverage.html",
        "https://sapling-scm.com/docs/introduction/",
        "https://sapling-scm.com/docs/scale/overview/",
        "https://cacm.acm.org/research/why-google-stores-billions-of-lines-of-code-in-a-single-repository/",
        "https://kubernetes.io/docs/tasks/run-application/scale-deployment/",
        "https://github.com/cncf/toc/blob/main/DEFINITION.md",
        "https://architecture.cncf.io/",
        "https://github.com/actions/checkout/releases/tag/v6.0.3",
        "https://nodejs.org/en/about/previous-releases",
    ]:
        require(url in source_urls, failures, f"official_sources missing {url}")

    if workflow:
        for needle in [
            "name: github-lane-unlocker-ci-cd",
            "pull_request:",
            "branches: [dev]",
            "push:",
            "permissions:",
            "contents: read",
            "pull-requests: read",
            "BUCK2_RELEASE: \"2026-06-01\"",
            "runs-on: ubuntu-24.04-arm",
            "Bootstrap Rust and Buck2 toolchains",
            "scripts/ci/github-actions-lane-unlocker-bootstrap.sh",
            "concurrency:",
            "github.event.pull_request.number || github.head_ref || github.run_id",
            "cancel-in-progress: true",
            "strategy:",
            "fail-fast: false",
            "max-parallel: 4",
            "matrix:",
            "lane: [governance, buck2-authority, rust-llvm-coverage, affected-build]",
            "name: github-lane-unlocker-required",
            "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: \"true\"",
            "uses: actions/checkout@v6",
            "fetch-depth: 0",
            "python3 scripts/ci/assert-github-lane-unlocker-bridge.py --json",
            "python3 scripts/ci/enforce-buck2-authority.py --policy specs/buck2-authority-policy.json",
            "buck2 build //:github-lane-unlocker-bridge-check //:buck2-authority-policy-check",
            "buck2 build //:rust-llvm-coverage-runner-contract-check //:rust-llvm-coverage-smoke-check",
            "infra/ci/buck2-affected-gate.sh origin/dev HEAD",
            "name: github-lane-unlocker-cd-dry-run",
            "buck2 build //:github-lane-unlocker-bridge-check",
        ]:
            require_contains(workflow, needle, failures, "lane unlocker workflow")

        legacy_checkout = "actions/checkout@" + "v4"
        insecure_node_runtime_env = "ACTIONS_ALLOW_USE_UNSECURE_NODE" + "_VERSION"
        require(legacy_checkout not in workflow, failures, "lane unlocker workflow must not use the legacy checkout v4 action")
        require(insecure_node_runtime_env not in workflow, failures, "lane unlocker workflow must not opt out to an unsecure JavaScript action runtime")
        require("FORCE_JAVASCRIPT_ACTIONS_TO_NODE24" in workflow, failures, "lane unlocker workflow must opt into the GitHub Actions Node24 JavaScript runtime")
        require("cargo tarpaulin" not in workflow.lower(), failures, "lane unlocker workflow must not use Tarpaulin")
        require("github-lane-unlocker-required" in workflow, failures, "lane unlocker workflow must expose the temporary required context")
        require(workflow.count("scripts/ci/github-actions-lane-unlocker-bootstrap.sh") == 3, failures, "lane unlocker workflow must bootstrap Rust and Buck2 in fanout, aggregator, and dry-run jobs")
        require(workflow.count("runs-on: ubuntu-24.04-arm") == 3, failures, "lane unlocker workflow must use arm64 Ubuntu runners for the repo default aarch64 Buck2 Rust toolchain")
        require("oya-ci-required" not in workflow, failures, "lane unlocker workflow must not impersonate oya-ci-required")
        for forbidden in ["jenkins", "forgejo", "argocd"]:
            require(forbidden not in workflow.lower(), failures, f"lane unlocker workflow must not invoke or describe {forbidden} as interim authority")


    if bootstrap:
        for needle in [
            "Deterministic GitHub Actions bootstrap",
            "RUSTUP_CONCURRENT_DOWNLOADS",
            "rustup toolchain install",
            "llvm-tools-preview",
            "x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu",
            "rustup component add",
            "rustup target add",
            "llvm-profdata",
            "llvm-cov",
            "rustc --print=cfg --target=aarch64-unknown-linux-gnu",
            "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/buck2-${buck2_arch}.zst",
            "sudo install -m 0755 /tmp/buck2 /usr/local/bin/buck2",
            "buck2 --version",
        ]:
            require_contains(bootstrap, needle, failures, "lane unlocker bootstrap")
        for forbidden in ["jenkins", "forgejo", "argocd"]:
            require(forbidden not in bootstrap.lower(), failures, f"lane unlocker bootstrap must not invoke or describe {forbidden} as interim authority")

    if rust_toolchain:
        for needle in [
            'channel = "1.95.0"',
            'components = ["rustfmt", "clippy", "llvm-tools-preview"]',
            'targets = ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"]',
            'profile = "minimal"',
        ]:
            require_contains(rust_toolchain, needle, failures, "rust-toolchain.toml")

    temp_bridge = branch_json.get("temporary_github_lane_unlocker_bridge", {}) if isinstance(branch_json, dict) else {}
    require(temp_bridge.get("status") == "temporary_bridge_not_destination_authority", failures, "infra/branch-protection/dev.json must declare temporary bridge status")
    require(temp_bridge.get("required_context") == "github-lane-unlocker-required", failures, "infra/branch-protection/dev.json temporary bridge context must be github-lane-unlocker-required")
    require(branch_json.get("required_status_checks", {}).get("contexts") == ["github-lane-unlocker-required"], failures, "infra/branch-protection/dev.json must require automated github-lane-unlocker-required during the temporary bridge")
    require(temp_bridge.get("native_cutover_target_context") == "oya-ci-required", failures, "infra/branch-protection/dev.json must preserve native cutover context")
    require(temp_bridge.get("live_mutation_performed_by_this_file") is False, failures, "infra/branch-protection/dev.json must not claim live mutation")
    require(temp_bridge.get("retired_external_scm_ci_cd_substrates_interim") is False or (temp_bridge.get("jenkins_interim") is False and temp_bridge.get("forgejo_interim") is False and temp_bridge.get("argocd_interim") is False), failures, "infra/branch-protection/dev.json must reject retired external SCM/CI/CD substrates as interim")

    require_contains(branch_yaml, "github-lane-unlocker-required", failures, ".github/branch-protection.yaml")
    require_contains(branch_yaml, "temporary GitHub/GitHub Actions lane-unlocker", failures, ".github/branch-protection.yaml")
    require_contains(branch_yaml, "not the permanent CI/SCM authority", failures, ".github/branch-protection.yaml")
    require_contains(branch_yaml, "not interim retired external SCM/CI/CD substrate authority", failures, ".github/branch-protection.yaml")

    root_entry = root_hub.get("entry_points", {}).get("github_lane_unlocker_bridge", {}) if isinstance(root_hub, dict) else {}
    require(root_entry.get("current_path") == "/specs/github-lane-unlocker-bridge.json", failures, "root hub must point github_lane_unlocker_bridge to /specs/github-lane-unlocker-bridge.json")

    for required in [
        ".github/workflows/github-lane-unlocker-ci-cd.yml",
        "scripts/ci/assert-github-lane-unlocker-bridge.py",
        "scripts/ci/github-actions-lane-unlocker-bootstrap.sh",
        "rust-toolchain.toml",
        "specs/github-lane-unlocker-bridge.json",
        "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
        "docs/ci/github-actions-lane-unlocker.md",
        "infra/branch-protection/dev.json",
        ".github/branch-protection.yaml",
    ]:
        require(required in buck2_policy.get("command_scan_files", []), failures, f"buck2 policy command_scan_files missing {required}")

    for label, text in [("ADR-0516", adr), ("GitHub lane unlocker procedure", procedure)]:
        for needle in [
            "GitHub/GitHub Actions",
            "temporary lane-unlocker",
            "no retired external SCM/CI/CD substrates",
            "retired-external-substrate-registry.json",
            "pure-Rust Sapling-compatible native SCM",
            "best-of-existing hyperscaler patterns",
            "not a wholesale reimplementation",
            "cloud native",
            "Kubernetes-native",
            "hyperscaler native",
            "loosely coupled microservices",
            "secure, resilient, manageable, sustainable, and observable",
            "distributable, observable, portable, interoperable, and available",
            "Cloud auth/shared substrate and Oyatie product auth/shared substrate are decoupled now",
            "no shared contract or shared surface",
            "rewrite and rewire Oyatie products to consume the Cloud IdP",
            "Buck2",
            "LLVM source-based coverage",
            "not P0.0 green",
        ]:
            require_contains(text, needle, failures, label)

    result = {
        "verdict": "PASS" if not failures else "FAIL",
        "spec": "specs/github-lane-unlocker-bridge.json",
        "workflow": ".github/workflows/github-lane-unlocker-ci-cd.yml",
        "temporary_required_context": "github-lane-unlocker-required",
        "native_cutover_target_context": "oya-ci-required",
        "p0_0_green": False,
        "phase0_complete": False,
        "failures": failures,
    }

    if args.json or not failures:
        print(json.dumps(result, sort_keys=True))
    if failures:
        if not args.json:
            print("github-lane-unlocker-bridge: RED", file=sys.stderr)
            for failure in failures:
                print(f"- {failure}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
