#!/usr/bin/env python3
"""Validate P00 repo hygiene automation contracts.

This check is intentionally local/static. It proves the repo records automation
for git, branch, repository, disk, Kubernetes, and documentation-sprawl hygiene
without deleting files, mutating live branch protection, or scaling workloads.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(os.environ.get("OYA_REPO_ROOT", Path(__file__).resolve().parents[2])).resolve()
SPEC_PATH = REPO_ROOT / "specs/repo-hygiene-automation.json"
RETIRED_SUBSTRATE_PATH = REPO_ROOT / "specs/retired-external-substrate-registry.json"
ROOT_HUB_PATH = REPO_ROOT / "specs/root-hub-pointers.json"
GITHUB_BRIDGE_PATH = REPO_ROOT / "specs/github-lane-unlocker-bridge.json"
MASTERPLAN_PATH = REPO_ROOT / "specs/masterplan.json"
SEQUENCING_PATH = REPO_ROOT / "specs/master-plan-sequencing.json"
README_PATH = REPO_ROOT / "README.md"
AGENTS_PATH = REPO_ROOT / "AGENTS.md"
CLAUDE_PATH = REPO_ROOT / "CLAUDE.md"
DOC_AGENTS_PATH = REPO_ROOT / "docs/AGENTS.md"
DOC_CATALOG_PATH = REPO_ROOT / "docs/DOC-CATALOG.md"
LANE_UNLOCKER_PROCEDURE_PATH = REPO_ROOT / "docs/ci/github-actions-lane-unlocker.md"
WORKFLOW_PATH = REPO_ROOT / ".github/workflows/github-lane-unlocker-ci-cd.yml"
BUCK_PATH = REPO_ROOT / "BUCK"
REPORT_DOC_STALENESS_PATH = REPO_ROOT / "tools/oya-doc-staleness-inventory-app/src/main.rs"
STALE_DOC_INVENTORY_COMMAND = "buck2 build //tools/oya-doc-staleness-inventory-app:doc-staleness-inventory-json"
STALE_DOC_INVENTORY_TEST_COMMAND = "buck2 build //tools/oya-doc-staleness-inventory-app:doc-staleness-inventory-unit-tests"
ACTIVE_EXACT_NAME_SCAN_PATHS = [
    REPO_ROOT / "AGENTS.md",
    REPO_ROOT / "CLAUDE.md",
    REPO_ROOT / "README.md",
    REPO_ROOT / "docs/AGENTS.md",
    REPO_ROOT / "docs/DOC-CATALOG.md",
    REPO_ROOT / "docs/MASTERPLAN.md",
    REPO_ROOT / "docs/ci/github-actions-lane-unlocker.md",
    REPO_ROOT / "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
    REPO_ROOT / ".github/branch-protection.yaml",
    REPO_ROOT / "infra/branch-protection/dev.json",
    REPO_ROOT / "specs/root-hub-pointers.json",
    REPO_ROOT / "specs/masterplan.json",
    REPO_ROOT / "specs/master-plan-sequencing.json",
    REPO_ROOT / "specs/github-lane-unlocker-bridge.json",
    REPO_ROOT / "specs/repo-hygiene-automation.json",
    REPO_ROOT / "specs/retired-external-substrate-registry.json",
]
RETIRED_EXACT_NAME_PATTERNS = ("Jenkins", "Forgejo", "ArgoCD", "Argo CD", "Argo Workflows/Rollouts")
ALLOWED_EXACT_NAME_CONTEXTS = (
    "retired-external-substrate-registry.json",
    "ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate",
    "adr_0349_jenkins_argocd_ci_cd_substrate",
    "15-ZE-jenkins-argocd-self-hostable-ci-cd-substrate",
    "ADR-0363-retire-agentic-vcs-foundry-to-intelligence-forgejo-substrate",
)

REQUIRED_DOMAINS = {
    "git_worktree_hygiene",
    "branch_merge_hygiene",
    "repository_publication_hygiene",
    "disk_workspace_hygiene",
    "kubernetes_workload_hygiene",
    "documentation_sprawl_hygiene",
}

REQUIRED_AUTOMATION_COMMANDS = {
    "python3 scripts/ci/assert-repo-hygiene-automation.py --json",
    "buck2 build //:repo-hygiene-automation-check",
    "python3 scripts/ci/assert-github-lane-unlocker-bridge.py --json",
    "buck2 build //:github-lane-unlocker-bridge-check //:buck2-authority-policy-check //:repo-hygiene-automation-check",
    STALE_DOC_INVENTORY_COMMAND,
    STALE_DOC_INVENTORY_TEST_COMMAND,
}

REQUIRED_SOURCE_URLS = {
    "https://docs.github.com/en/repositories/creating-and-managing-repositories/about-repositories",
    "https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners",
    "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue",
    "https://docs.github.com/en/actions/sharing-automations/reusing-workflows",
    "https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs",
    "https://sapling-scm.com/docs/introduction/",
    "https://sapling-scm.com/docs/scale/overview/",
    "https://architecture.cncf.io/",
    "https://kubernetes.io/docs/tasks/run-application/scale-deployment/",
    "https://www.nist.gov/publications/zero-trust-architecture",
    "https://csrc.nist.gov/pubs/sp/800/162/upd2/final",
    "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
    "https://kubernetes.io/docs/concepts/security/pod-security-standards/",
    "https://kubernetes.io/docs/concepts/containers/runtime-class/",
    "https://kubernetes.io/docs/tasks/configure-pod-container/configure-service-account/",
    "https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/",
    "https://docs.github.com/en/actions/concepts/security/openid-connect",
    "https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions",
    "https://slsa.dev/spec/v1.2/requirements",
    "https://istio.io/latest/docs/concepts/security/",
    "https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_policies_scps.html",
}
REQUIRED_SECURITY_BACKLOG_IDS = {
    "zero_trust_architecture",
    "privileged_identity_management",
    "abac_beyond_rbac",
    "network_microsegmentation",
    "silo_apis_integrations",
    "silo_ai_automation",
    "encrypt_sensitive_data",
    "purge_redundant_obsolete_data",
    "data_layer_rate_limiting",
    "automated_session_revocation",
    "dual_attribution_audit_logging",
    "isolate_ci_cd_pipelines",
    "tightly_scope_pipeline_secrets",
    "pin_dependencies_private_registries",
    "honeytokens_tripwires",
    "host_based_microsegmentation",
    "hardware_enforced_isolation",
    "multi_account_cloud_strategy",
    "service_control_policies",
    "enforce_workload_identity",
    "block_node_metadata_access",
    "pod_level_runtime_isolation",
    "restrict_container_privileges",
    "immutable_container_filesystems",
    "drop_linux_capabilities",
    "default_deny_network_policies",
    "service_mesh_mtls",
    "disable_default_service_account_token_mounting",
    "cluster_architecture_blast_walls",
    "separate_control_planes",
    "sandboxed_runtimes",
}

ROOT_MARKDOWN_ALLOWLIST = {"README.md", "AGENTS.md", "CLAUDE.md"}
ACTIVE_CONTEXT_SCAN_TEXTS = {
    "AGENTS.md": AGENTS_PATH,
    "CLAUDE.md": CLAUDE_PATH,
    "README.md": README_PATH,
    "docs/AGENTS.md": DOC_AGENTS_PATH,
    "docs/ci/github-actions-lane-unlocker.md": LANE_UNLOCKER_PROCEDURE_PATH,
    "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md": REPO_ROOT / "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
    ".github/branch-protection.yaml": REPO_ROOT / ".github/branch-protection.yaml",
    "infra/branch-protection/dev.json": REPO_ROOT / "infra/branch-protection/dev.json",
}
FORBIDDEN_ACTIVE_DOC_PHRASES = [
    "cloud-ci/oya-ci required context + reviewer APPROVE gate merge readiness",
    "cloud-ci/oya-ci required context is merge authority",
    "GitHub Actions is retired",
    "Jenkins CI + oya gate run-all",
    "self-hosted Forgejo required-checks/auto-merge is the substrate target",
    "manual oya-ci-required success statuses to merge bridge PRs",
    "infra/ci/buck2-affected-gate.sh origin/dev HEAD",
]
REQUIRED_FORBIDDEN_PHRASE_IDS = {
    "cloud-ci/oya-ci required context + reviewer APPROVE gate merge readiness",
    "cloud-ci/oya-ci required context is merge authority",
    "GitHub Actions is retired",
    "legacy_ci_server CI + oya gate run-all",
    "legacy_self_hosted_git_forge required-checks/auto-merge is the substrate target",
    "manual oya-ci-required success statuses to merge bridge PRs",
    "infra/ci/buck2-affected-gate.sh origin/dev HEAD",
}


def load_json(path: Path, failures: list[str]) -> Any:
    if not path.exists():
        failures.append(f"{path.relative_to(REPO_ROOT)}: missing")
        return {}
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        failures.append(f"{path.relative_to(REPO_ROOT)}: invalid JSON: {exc}")
        return {}


def read_text(path: Path, failures: list[str]) -> str:
    if not path.exists():
        failures.append(f"{path.relative_to(REPO_ROOT)}: missing")
        return ""
    return path.read_text(errors="replace")


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
    retired = load_json(RETIRED_SUBSTRATE_PATH, failures)
    root_hub = load_json(ROOT_HUB_PATH, failures)
    github_bridge = load_json(GITHUB_BRIDGE_PATH, failures)
    masterplan = load_json(MASTERPLAN_PATH, failures)
    sequencing = load_json(SEQUENCING_PATH, failures)
    readme = read_text(README_PATH, failures)
    agents = read_text(AGENTS_PATH, failures)
    claude = read_text(CLAUDE_PATH, failures)
    doc_agents = read_text(DOC_AGENTS_PATH, failures)
    doc_catalog = read_text(DOC_CATALOG_PATH, failures)
    procedure = read_text(LANE_UNLOCKER_PROCEDURE_PATH, failures)
    workflow = read_text(WORKFLOW_PATH, failures)
    buck = read_text(BUCK_PATH, failures)

    require(spec.get("status") == "p00_active_automation_contract", failures, "repo hygiene spec status must be p00_active_automation_contract")
    require(spec.get("local_static_only") is True, failures, "repo hygiene checker must be local/static only")
    require(spec.get("live_mutation_performed") is False, failures, "repo hygiene checker must not mutate live systems")
    require(spec.get("buck2_authority") is True, failures, "repo hygiene automation must keep Buck2 authority")
    require(spec.get("github_bridge_temporary") is True, failures, "repo hygiene automation must keep GitHub temporary")
    require(spec.get("native_scm_requires_github_adapter") is True, failures, "native SCM must require a GitHub adapter")

    domains = {item.get("id") for item in spec.get("domains", []) if isinstance(item, dict)}
    missing_domains = sorted(REQUIRED_DOMAINS - domains)
    require(not missing_domains, failures, f"repo hygiene domains missing: {', '.join(missing_domains)}")

    for item in spec.get("domains", []):
        if not isinstance(item, dict):
            failures.append("repo hygiene domains entries must be objects")
            continue
        domain_id = item.get("id", "<missing-id>")
        for field in ["owner", "automation", "policy", "claim_boundary"]:
            value = item.get(field)
            if not isinstance(value, str) or not value.strip():
                failures.append(f"domain {domain_id}: {field} must be a non-empty string")
        if item.get("live_mutation") is not False:
            failures.append(f"domain {domain_id}: live_mutation must be false")

    commands = set(spec.get("automation_commands", []))
    missing_commands = sorted(REQUIRED_AUTOMATION_COMMANDS - commands)
    require(not missing_commands, failures, f"automation_commands missing: {', '.join(missing_commands)}")

    source_urls = {item.get("url") for item in spec.get("official_sources", []) if isinstance(item, dict)}
    missing_urls = sorted(REQUIRED_SOURCE_URLS - source_urls)
    require(not missing_urls, failures, f"official_sources missing: {', '.join(missing_urls)}")

    active_context_drift = spec.get("active_context_drift_scan", {})
    require(active_context_drift.get("required_interim_context") == "github-lane-unlocker-required", failures, "active context drift scan must require github-lane-unlocker-required")
    require(active_context_drift.get("native_cutover_context") == "oya-ci-required", failures, "active context drift scan must preserve oya-ci-required as native cutover only")
    required_tool_examples = set(active_context_drift.get("required_tool_examples", []))
    for tool_example in [
        "git worktree add",
        "gh pr create --base dev",
        "buck2 build //:repo-hygiene-automation-check",
        "infra/ci/buck2-affected-gate.sh github-mirror/dev HEAD",
    ]:
        require(tool_example in required_tool_examples, failures, f"active context drift scan missing required tool example {tool_example!r}")
    forbidden_active_phrase_ids = set(active_context_drift.get("forbidden_active_doc_phrases", []))
    missing_forbidden_phrase_ids = sorted(REQUIRED_FORBIDDEN_PHRASE_IDS - forbidden_active_phrase_ids)
    require(not missing_forbidden_phrase_ids, failures, f"active context drift scan missing forbidden phrase ids: {', '.join(missing_forbidden_phrase_ids)}")

    doc_sprawl = spec.get("documentation_sprawl", {})
    require(doc_sprawl.get("new_markdown_default") == "reject_unless_registered_or_lane_owned", failures, "documentation_sprawl.new_markdown_default must reject unregistered/laneless docs")
    require(doc_sprawl.get("root_markdown_allowlist") == sorted(ROOT_MARKDOWN_ALLOWLIST), failures, "documentation_sprawl.root_markdown_allowlist must be README/AGENTS/CLAUDE only")
    require(doc_sprawl.get("stale_doc_scan_cutoff_days") == 3, failures, "documentation_sprawl.stale_doc_scan_cutoff_days must be 3")
    require(doc_sprawl.get("archive_before_delete") is True, failures, "documentation_sprawl.archive_before_delete must be true")
    require(doc_sprawl.get("thin_pointer_shared_docs") is True, failures, "documentation_sprawl.thin_pointer_shared_docs must be true")
    stale_inventory = doc_sprawl.get("stale_doc_inventory", {})
    require(stale_inventory.get("command") == STALE_DOC_INVENTORY_COMMAND, failures, "documentation_sprawl.stale_doc_inventory.command must match the CI command")
    require(stale_inventory.get("cutoff_days") == 3, failures, "documentation_sprawl.stale_doc_inventory.cutoff_days must be 3")
    require(stale_inventory.get("live_mutation") is False, failures, "documentation_sprawl.stale_doc_inventory must not mutate files")
    require(stale_inventory.get("claim_boundary") == "inventory_only_no_deletion_no_archive_no_live_mutation", failures, "documentation_sprawl.stale_doc_inventory claim boundary must stay inventory-only")
    require("git-backed stale documentation inventory" in doc_sprawl.get("automation_targets", []), failures, "documentation_sprawl automation_targets must include git-backed stale documentation inventory")
    require(REPORT_DOC_STALENESS_PATH.exists(), failures, "tools/oya-doc-staleness-inventory-app/src/main.rs must exist")

    language_policy = spec.get("automation_language_policy", {})
    require(language_policy.get("new_parallel_fanout_automation") == "rust_buck2_first", failures, "automation_language_policy must make new fanout automation Rust/Buck2-first")
    require(language_policy.get("new_python_or_shell_gate_surface") == "deny_unless_explicit_bootstrap_exception", failures, "automation_language_policy must deny new Python/shell gate surfaces by default")
    require(language_policy.get("legacy_python_shell_migration_backlog") is True, failures, "automation_language_policy must record legacy Python/shell migration backlog")

    shared_surface = spec.get("shared_surface_mitigation", {})
    require("vertical" in str(shared_surface.get("decision", "")).lower(), failures, "shared_surface_mitigation.decision must include vertical ownership")
    require(shared_surface.get("mechanical_resolution", {}).get("required") is True, failures, "shared_surface_mitigation must require mechanical resolution")
    require("generated_consolidation" in json.dumps(shared_surface), failures, "shared_surface_mitigation must include generated consolidation")

    cleanup_ids = {item.get("id") for item in spec.get("cleanup_candidate_backlog", []) if isinstance(item, dict)}
    for cleanup_id in ["legacy_python_shell_gate_surfaces", "shared_ci_workflow_surface", "stale_doc_inventory_followups", "retired_external_substrate_residue"]:
        require(cleanup_id in cleanup_ids, failures, f"cleanup_candidate_backlog missing {cleanup_id}")

    security_ids = {item.get("id") for item in spec.get("security_hardening_backlog", []) if isinstance(item, dict)}
    missing_security_ids = sorted(REQUIRED_SECURITY_BACKLOG_IDS - security_ids)
    require(not missing_security_ids, failures, f"security_hardening_backlog missing: {', '.join(missing_security_ids)}")
    for item in spec.get("security_hardening_backlog", []):
        if not isinstance(item, dict):
            failures.append("security_hardening_backlog entries must be objects")
            continue
        require(item.get("valid") is True, failures, f"security_hardening_backlog.{item.get('id', '<missing-id>')}: valid must be true")
        require(bool(item.get("lane")), failures, f"security_hardening_backlog.{item.get('id', '<missing-id>')}: lane must be set")
        require(bool(item.get("backlog_action")), failures, f"security_hardening_backlog.{item.get('id', '<missing-id>')}: backlog_action must be set")

    root_markdown = {path.name for path in REPO_ROOT.glob("*.md") if path.is_file()}
    require(root_markdown <= ROOT_MARKDOWN_ALLOWLIST, failures, f"root markdown sprawl: {sorted(root_markdown - ROOT_MARKDOWN_ALLOWLIST)}")

    retired_items = {item.get("id") for item in retired.get("retired_external_substrates", []) if isinstance(item, dict)}
    for item_id in ["legacy_ci_server", "legacy_self_hosted_git_forge", "legacy_gitops_cd_runtime", "legacy_workflow_runtime"]:
        require(item_id in retired_items, failures, f"retired external substrate registry missing {item_id}")
    require(retired.get("active_docs_must_use_generic_term") == "retired external SCM/CI/CD substrates", failures, "retired substrate registry must require the generic active-doc term")
    require(retired.get("exact_names_allowed_only_in") == ["retired registry", "historical ADR provenance", "adapter code awaiting retirement", "archive manifests"], failures, "retired substrate exact-name allowlist must stay narrow")

    entry_points = root_hub.get("entry_points", {}) if isinstance(root_hub, dict) else {}
    pointers = root_hub.get("pointers", {}) if isinstance(root_hub, dict) else {}
    require(entry_points.get("repo_hygiene_automation", {}).get("current_path") == "/specs/repo-hygiene-automation.json", failures, "root hub must point repo_hygiene_automation to /specs/repo-hygiene-automation.json")
    require(entry_points.get("retired_external_substrate_registry", {}).get("current_path") == "/specs/retired-external-substrate-registry.json", failures, "root hub must point retired_external_substrate_registry to /specs/retired-external-substrate-registry.json")
    require(pointers.get("repo_hygiene_automation") == "specs/repo-hygiene-automation.json", failures, "root hub pointers.repo_hygiene_automation must point to specs/repo-hygiene-automation.json")
    require(pointers.get("retired_external_substrate_registry") == "specs/retired-external-substrate-registry.json", failures, "root hub pointers.retired_external_substrate_registry must point to specs/retired-external-substrate-registry.json")

    scm = github_bridge.get("native_destination_seams", {}).get("oyatie_scm", {}) if isinstance(github_bridge, dict) else {}
    adapters = scm.get("adapters", {}) if isinstance(scm, dict) else {}
    require(adapters.get("github_public_private_publication", {}).get("required") is True, failures, "github bridge must require native SCM GitHub publication adapter")
    require(adapters.get("git_protocol", {}).get("required") is True, failures, "github bridge must require native SCM git protocol adapter")
    require(adapters.get("github_actions_status_bridge", {}).get("temporary") is True, failures, "github bridge must keep GitHub Actions status adapter temporary")
    require(github_bridge.get("repo_hygiene_automation_ref") == "/specs/repo-hygiene-automation.json", failures, "github bridge must reference repo hygiene automation spec")

    p00 = masterplan.get("imminent_work_p00", {}) if isinstance(masterplan, dict) else {}
    require(p00.get("repo_hygiene_automation") is not None, failures, "masterplan.imminent_work_p00 must include repo_hygiene_automation")
    require("repo_hygiene_automation" in json.dumps(sequencing), failures, "master-plan sequencing must mention repo_hygiene_automation")

    for label, text in [
        ("README.md", readme),
        ("AGENTS.md", agents),
        ("CLAUDE.md", claude),
        ("docs/AGENTS.md", doc_agents),
        ("docs/DOC-CATALOG.md", doc_catalog),
        ("docs/ci/github-actions-lane-unlocker.md", procedure),
    ]:
        require_contains(text, "repo-hygiene-automation", failures, label)
        require_contains(text, "buck2 build //:repo-hygiene-automation-check", failures, label)

    for label, text in [("AGENTS.md", agents), ("CLAUDE.md", claude)]:
        for tool_example in [
            "git worktree add",
            "gh pr create --base dev",
            "infra/ci/buck2-affected-gate.sh github-mirror/dev HEAD",
        ]:
            require_contains(text, tool_example, failures, label)

    for label, text in [("README.md", readme), ("docs/AGENTS.md", doc_agents), ("docs/ci/github-actions-lane-unlocker.md", procedure)]:
        require_contains(text, "github-lane-unlocker-required", failures, label)

    for label, path in ACTIVE_CONTEXT_SCAN_TEXTS.items():
        text = read_text(path, failures)
        for phrase in FORBIDDEN_ACTIVE_DOC_PHRASES:
            if phrase in text:
                failures.append(f"{label}: stale active authority phrase present: {phrase!r}")

    for needle in [
        "repo-hygiene-automation-check",
        "assert-repo-hygiene-automation.py",
        "repo-hygiene-automation.json",
        "retired-external-substrate-registry.json",
        "oya-doc-staleness-inventory-app",
    ]:
        require_contains(buck, needle, failures, "BUCK")

    for scan_path in ACTIVE_EXACT_NAME_SCAN_PATHS:
        if not scan_path.exists():
            failures.append(f"{scan_path.relative_to(REPO_ROOT)}: missing from retired exact-name scan")
            continue
        rel = scan_path.relative_to(REPO_ROOT).as_posix()
        for line_no, line in enumerate(scan_path.read_text(errors="replace").splitlines(), start=1):
            lowered = line.lower()
            if not any(pattern.lower() in lowered for pattern in RETIRED_EXACT_NAME_PATTERNS):
                continue
            if any(allowed.lower() in lowered or allowed.lower() in rel.lower() for allowed in ALLOWED_EXACT_NAME_CONTEXTS):
                continue
            failures.append(f"{rel}:{line_no}: retired exact-name reference must use generic active-doc term")

    for needle in [
        "python3 scripts/ci/assert-repo-hygiene-automation.py --json",
        "buck2 build //:repo-hygiene-automation-check",
        STALE_DOC_INVENTORY_COMMAND,
        STALE_DOC_INVENTORY_TEST_COMMAND,
    ]:
        require_contains(workflow, needle, failures, "github-lane-unlocker workflow")

    result = {
        "verdict": "PASS" if not failures else "FAIL",
        "spec": "specs/repo-hygiene-automation.json",
        "domains": sorted(domains),
        "root_markdown": sorted(root_markdown),
        "local_static_only": True,
        "live_mutation_performed": False,
        "retired_exact_name_scan_files": len(ACTIVE_EXACT_NAME_SCAN_PATHS),
        "active_context_scan_files": len(ACTIVE_CONTEXT_SCAN_TEXTS),
        "stale_doc_inventory_command": STALE_DOC_INVENTORY_COMMAND,
        "stale_doc_inventory_test_command": STALE_DOC_INVENTORY_TEST_COMMAND,
        "security_hardening_backlog_count": len(security_ids),
        "failures": failures,
    }
    if args.json or not failures:
        print(json.dumps(result, sort_keys=True))
    if failures:
        if not args.json:
            print("repo-hygiene-automation: RED", file=sys.stderr)
            for failure in failures:
                print(f"- {failure}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
