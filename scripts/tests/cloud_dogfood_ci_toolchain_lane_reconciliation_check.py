#!/usr/bin/env python3
"""Validate G007 dogfood CI/toolchain lane contract and evidence snapshot."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "specs" / "cloud-dogfood-ci-toolchain-lane-reconciliation.json"
SNAPSHOT_PATH = REPO_ROOT / "specs" / "cloud-dogfood-ci-toolchain-lane-evidence-snapshot.json"
DEV_BRANCH_PROTECTION_PATH = REPO_ROOT / "infra" / "branch-protection" / "dev.json"
GATEWAY_CONFIG_PATH = REPO_ROOT / "oya" / "ci-webhook-gateway" / "src" / "config.rs"
GATEWAY_DEPLOYMENT_PATH = REPO_ROOT / "oya" / "ci-webhook-gateway" / "iac" / "k8s" / "helm" / "templates" / "deployment.yaml"
GATEWAY_VALUES_PATH = REPO_ROOT / "oya" / "ci-webhook-gateway" / "iac" / "k8s" / "helm" / "values.yaml"
JENKINS_VALUES_PATH = REPO_ROOT / "infra" / "ci" / "jenkins" / "values-local.yaml"
JENKINS_GATE_PATH = REPO_ROOT / "infra" / "ci" / "jenkins" / "Jenkinsfile-oya-ci-gate"
BUCK2_GATE_PATH = REPO_ROOT / "infra" / "ci" / "buck2-affected-gate.sh"
CONTROLLER_CHART_PATH = REPO_ROOT / "oya" / "ci-controller" / "iac" / "k8s" / "helm" / "Chart.yaml"
CONTROLLER_APP_PATH = REPO_ROOT / "oya" / "ci-controller" / "crates" / "oya-ci-controller-app" / "src" / "lib.rs"
CONTROLLER_KERNEL_PATH = REPO_ROOT / "oya" / "ci-controller" / "crates" / "oya-ci-controller-kernel" / "src" / "lib.rs"
ADR_0513_PATH = REPO_ROOT / "docs" / "decisions" / "ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md"
ADR_0511_PATH = REPO_ROOT / "docs" / "decisions" / "ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md"
REPORTED_CONTEXTS_PATH = REPO_ROOT / "infra" / "ci" / "jenkins" / "reported-status-contexts.json"
BRANCH_PROTECTION_APPLY_PATH = REPO_ROOT / "scripts" / "branch-protection-apply.sh"
GITHUB_BRANCH_PROTECTION_YAML_PATH = REPO_ROOT / ".github" / "branch-protection.yaml"

EXPECTED_CONTEXTS = [
    "cargo-fmt",
    "cargo-check",
    "cargo-clippy",
    "cargo-nextest",
    "cargo-deny",
    "oya-verify",
]
REQUIRED_CONTROL_FLAGS = {
    "strict_separation",
    "pure_dogfood",
    "self_hosted_only",
    "no_external_saas_ci",
    "no_github_actions_fallback",
    "no_public_cloud_ci",
    "safe_local_evidence_only",
    "no_live_cluster_claim",
    "no_production_green_claim",
    "no_jenkins_deleted_claim",
    "no_controller_cutover_claim",
    "no_argo_live_claim",
    "no_bespoke_ci_live_default_claim",
    "no_delete_jenkins_now",
}
REQUIRED_CANNOT_CLAIM_PHRASES = {
    "Jenkins is retired or safe to delete.",
    "The bespoke oya-ci controller is the default live CI dispatcher.",
    "Argo Workflows is the live CI orchestrator.",
    "The PR-sourced gate security hole is fixed in the live path.",
    "Production CI is green for Oyatie Cloud tenant workloads.",
    "GitHub Actions, external SaaS CI, or public-cloud CI is an allowed fallback.",
}
REQUIRED_BLOCKED_FAMILIES = {
    "jenkins_retired",
    "jenkins_safe_to_delete",
    "custom_ci_live_default",
    "controller_cutover_complete",
    "argo_workflows_live",
    "github_actions_fallback",
    "external_saas_ci_fallback",
    "public_cloud_ci_fallback",
    "production_ci_green",
    "tenant_workload_ready",
}
REQUIRED_SOURCE_IDS = {
    "adr-0363-current-substrate",
    "adr-0511-argo-proposed",
    "adr-0513-accepted-bespoke-oya-ci",
    "adr-0514-target-architecture-open-p0s",
    "jenkins-jcasc-bridge-job",
    "jenkins-bridge-pipeline-body",
    "buck2-affected-gate",
    "gateway-default-dispatcher",
    "gateway-helm-deployment-defaults",
    "gateway-helm-values-jenkins-url",
    "bespoke-controller-chart",
    "bespoke-controller-app",
    "bespoke-controller-kernel",
    "dev-branch-protection-contexts",
    "github-branch-protection-commentary",
    "jenkins-reported-context-drift-source",
    "root-jenkinsfile-hygiene-surface",
    "branch-protection-apply-github-bootstrap",
}
REQUIRED_CONTRADICTION_IDS = {
    "jenkins_bridge_vs_bespoke_destination",
    "argo_workflows_proposed_vs_bespoke_accepted",
    "controller_opt_in_not_default",
    "pr_sourced_gate_security_open",
    "bridge_status_context_not_required_or_reported",
    "required_context_surface_drift",
    "reported_status_metadata_legacy_github_producer_drift",
    "branch_protection_github_bootstrap_vs_forgejo_status_sink",
    "root_jenkinsfile_path_comment_drift",
}
VOLATILE_FIELDS_FORBIDDEN_IN_CONTRACT = {
    "source_evidence",
    "current_live_path",
    "bespoke_destination",
    "contradictions_and_resolution",
    "jenkins_retirement_gate",
}
FORBIDDEN_CAN_CLAIM_PATTERNS = [
    re.compile(pattern)
    for pattern in [
        r"\bjenkins\b.{0,40}\b(retired|removed|deleted|gone|obsolete)\b",
        r"\b(retired|removed|deleted|obsolete)\b.{0,40}\bjenkins\b",
        r"\bjenkins\b.{0,50}\b(safe\s+to\s+delete|can\s+be\s+deleted|removable|deletion\s+(is\s+)?allowed)\b",
        r"\b(safe\s+to\s+delete|safe\s+deletion\s+of|deletion\s+of|delete)\b.{0,50}\bjenkins\b",
        r"\b(custom|bespoke|controller)\b.{0,60}\b(live|default|cutover\s+complete|cutover\s+is\s+complete|cutover\s+completed|has\s+cut\s+over|in\s+production)\b",
        r"\b(live|default|cutover\s+complete|cutover\s+is\s+complete|cutover\s+completed|has\s+cut\s+over|in\s+production)\b.{0,60}\b(custom|bespoke|controller)\b",
        r"\blive\s+default\b.{0,60}\b(custom\s+ci|bespoke\s+ci|ci\s+controller|controller)\b",
        r"\bbranch\s+protection\b.{0,60}\b(cut\s+over|cutover|controller\s+contexts?)\b",
        r"\bargo(\s+workflows?)?\b.{0,50}\b(live|default|in\s+production|cutover\s+complete)\b",
        r"\bargo\b.{0,30}\bhas\s+cutover\s+complete\b",
        r"\bgithub\s+actions\b.{0,40}\b(fallback|allowed|accepted|enabled)\b",
        r"\bexternal\s+saas\s+ci\b.{0,40}\b(fallback|allowed|accepted|enabled)\b",
        r"\bpublic\s+cloud\s+ci\b.{0,40}\b(fallback|allowed|accepted|enabled)\b",
        r"\bproduction\s+(ci|gate)\b.{0,40}\b(green|passed|ready|live)\b",
        r"\btenant\s+workloads?\b.{0,40}\b(ready|safe|supported|enabled)\b",
        r"\bpr[- ]sourced\s+gate\b.{0,50}\b(fixed|closed|resolved|remediated)\b",
    ]
]

Mutator = Callable[[dict[str, Any], dict[str, Any]], None]


def fail(message: str) -> NoReturn:
    print(f"cloud dogfood CI/toolchain lane reconciliation check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def load_json(file_path: Path) -> dict[str, Any]:
    try:
        data = json.loads(file_path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {file_path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {file_path.relative_to(REPO_ROOT)}: {exc}")
    require(isinstance(data, dict), f"{file_path.relative_to(REPO_ROOT)} must contain a JSON object")
    return data if isinstance(data, dict) else {}


def read_text(file_path: Path) -> str:
    try:
        return file_path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing {file_path.relative_to(REPO_ROOT)}")


def as_list(value: object, label: str) -> list[Any]:
    require(isinstance(value, list), f"{label} must be a list")
    return list(value) if isinstance(value, list) else []


def normalized(value: object) -> str:
    if isinstance(value, dict):
        raw = " ".join(normalized(item) for item in value.values())
    elif isinstance(value, (list, tuple, set)):
        raw = " ".join(normalized(item) for item in value)
    else:
        raw = str(value).lower()
    return re.sub(r"[^a-z0-9]+", " ", raw).strip()


def contains_forbidden_can_claim(value: object) -> bool:
    haystack = f" {normalized(value)} "
    return any(pattern.search(haystack) for pattern in FORBIDDEN_CAN_CLAIM_PATTERNS)


def require_snippets(file_path: Path, snippets: list[Any], evidence_id: str) -> None:
    content = read_text(file_path)
    require(snippets, f"{evidence_id}: required_snippets must not be empty")
    for snippet in snippets:
        require(isinstance(snippet, str) and snippet, f"{evidence_id}: snippet values must be non-empty strings")
        require(snippet in content, f"{evidence_id}: missing snippet {snippet!r} in {file_path.relative_to(REPO_ROOT)}")


def validate_policy(contract: dict[str, Any]) -> None:
    policy = contract.get("validation_policy")
    require(isinstance(policy, dict), "validation_policy must be an object")
    if not isinstance(policy, dict):
        return
    require(policy.get("stable_contract_only") is True, "validation_policy.stable_contract_only must be true")
    require(policy.get("snapshot_artifact") == str(SNAPSHOT_PATH.relative_to(REPO_ROOT)), "validation_policy.snapshot_artifact mismatch")
    require(policy.get("snapshot_boundary") == "volatile_current_repository_topology_not_stable_contract", "validation_policy.snapshot_boundary mismatch")
    require(policy.get("safe_local_evidence_only") is True, "validation_policy.safe_local_evidence_only must be true")
    for key in ["network_required", "cluster_required", "external_provider_required"]:
        require(policy.get(key) is False, f"validation_policy.{key} must be false")
    require(policy.get("validator_command") == "python3 scripts/tests/cloud_dogfood_ci_toolchain_lane_reconciliation_check.py", "validator command mismatch")


def validate_claim_controls(contract: dict[str, Any]) -> None:
    controls = contract.get("claim_controls")
    require(isinstance(controls, dict), "claim_controls must be an object")
    if not isinstance(controls, dict):
        return
    for key in REQUIRED_CONTROL_FLAGS:
        require(controls.get(key) is True, f"claim_controls.{key} must be true")
    cannot_claim = set(as_list(controls.get("cannot_claim_yet"), "claim_controls.cannot_claim_yet"))
    require(cannot_claim >= REQUIRED_CANNOT_CLAIM_PHRASES, "claim_controls.cannot_claim_yet missing required negative claims")
    blocked_families = set(as_list(controls.get("blocked_claim_families"), "claim_controls.blocked_claim_families"))
    require(blocked_families >= REQUIRED_BLOCKED_FAMILIES, "claim_controls.blocked_claim_families missing required families")
    require(not contains_forbidden_can_claim(controls.get("can_claim_now", [])), "claim_controls.can_claim_now contains forbidden live/default/delete/fallback claim")


def validate_stable_contract(contract: dict[str, Any], snapshot: dict[str, Any]) -> None:
    for field in ["spec_id", "story_id", "title", "status", "purpose", "validation_policy", "claim_controls", "stable_contract", "machine_check_surfaces", "nonclaims", "next_goal_links"]:
        require(field in contract, f"missing top-level contract field {field!r}")
    require(not (VOLATILE_FIELDS_FORBIDDEN_IN_CONTRACT & set(contract)), "stable contract must not contain volatile snapshot fields at top level")
    require(contract.get("spec_id") == "EXE-CLOUD-DOGFOOD-CI-TOOLCHAIN-LANE-RECONCILIATION", "unexpected spec_id")
    require(contract.get("story_id") == "G007-dogfood-ci-toolchain-lane-reconcilia", "unexpected story_id")
    require(contract.get("status") == "Proposed-target", "status must remain Proposed-target")
    stable = contract.get("stable_contract")
    require(isinstance(stable, dict), "stable_contract must be an object")
    if not isinstance(stable, dict):
        return
    require(stable.get("contract_id") == "G007-DOGFOOD-CI-TOOLCHAIN-STABLE-CONTRACT", "stable_contract.contract_id mismatch")
    boundary = normalized(stable.get("contract_boundary", ""))
    for phrase in ["stable claim semantics", "volatile current repository topology", "snapshot artifact"]:
        require(phrase in boundary, f"stable contract boundary missing {phrase!r}")
    require(stable.get("snapshot_artifact") == str(SNAPSHOT_PATH.relative_to(REPO_ROOT)), "stable_contract.snapshot_artifact mismatch")
    volatile_fields = set(as_list(stable.get("volatile_snapshot_fields_not_allowed_at_contract_top_level"), "stable_contract.volatile_snapshot_fields_not_allowed_at_contract_top_level"))
    require(volatile_fields == VOLATILE_FIELDS_FORBIDDEN_IN_CONTRACT, "stable contract must enumerate volatile fields excluded from top level")
    required_classes = set(as_list(stable.get("required_reconciliation_classes"), "stable_contract.required_reconciliation_classes"))
    require(required_classes == REQUIRED_CONTRADICTION_IDS, "stable contract reconciliation classes must match required contradiction ids")
    snapshot_ids = {row.get("id") for row in snapshot.get("contradictions_and_resolution", []) if isinstance(row, dict)}
    require(snapshot_ids == required_classes, "snapshot contradiction ids must satisfy stable contract classes")
    semantic_text = normalized(stable.get("semantic_requirements", []))
    for phrase in ["root jenkinsfile hygiene", "active bridge", "jenkins is retired", "live default", "snapshot freshness checks"]:
        require(phrase in semantic_text, f"semantic_requirements missing {phrase!r}")
    require("delete allowed now" in normalized(stable.get("retirement_gate_derivation_rule", "")), "retirement gate derivation rule required")


def validate_snapshot_header(snapshot: dict[str, Any]) -> None:
    for field in ["snapshot_id", "story_id", "title", "status", "purpose", "snapshot_boundary", "source_evidence", "current_live_path", "bespoke_destination", "contradictions_and_resolution", "jenkins_retirement_gate"]:
        require(field in snapshot, f"missing snapshot field {field!r}")
    require(snapshot.get("snapshot_id") == "EXE-CLOUD-DOGFOOD-CI-TOOLCHAIN-LANE-EVIDENCE-SNAPSHOT", "unexpected snapshot_id")
    require(snapshot.get("story_id") == "G007-dogfood-ci-toolchain-lane-reconcilia", "unexpected snapshot story_id")
    require(snapshot.get("status") == "Current-repo-snapshot", "snapshot status must be Current-repo-snapshot")
    require(snapshot.get("snapshot_boundary") == "volatile_current_repository_topology_not_stable_contract", "snapshot boundary mismatch")
    purpose = normalized(snapshot.get("purpose", ""))
    for phrase in ["volatile current repository topology", "exact paths", "update this snapshot"]:
        require(phrase in purpose, f"snapshot purpose missing {phrase!r}")


def validate_source_evidence(snapshot: dict[str, Any]) -> None:
    rows = as_list(snapshot.get("source_evidence"), "snapshot.source_evidence")
    source_ids: set[str] = {row["id"] for row in rows if isinstance(row, dict) and isinstance(row.get("id"), str)}
    require(source_ids == REQUIRED_SOURCE_IDS, f"source_evidence ids mismatch: missing {sorted(REQUIRED_SOURCE_IDS - source_ids)} extra {sorted(source_ids - REQUIRED_SOURCE_IDS)}")
    for row in rows:
        require(isinstance(row, dict), "source_evidence rows must be objects")
        evidence_id = str(row.get("id", "<missing>"))
        if evidence_id == "root-jenkinsfile-hygiene-surface":
            require(row.get("status") == "hygiene-debt-source", "root Jenkinsfile evidence must be hygiene-debt-source")
            require("not source evidence" in str(row.get("role", "")), "root Jenkinsfile evidence must not be described as active bridge proof")
        rel_path = row.get("path")
        require(isinstance(rel_path, str) and rel_path, f"{evidence_id}: path required")
        require(not rel_path.startswith("/"), f"{evidence_id}: path must be repo-relative")
        require(".." not in Path(rel_path).parts, f"{evidence_id}: path must not escape repo")
        evidence_path = REPO_ROOT / rel_path
        require(evidence_path.exists(), f"{evidence_id}: missing evidence file {rel_path}")
        require_snippets(evidence_path, as_list(row.get("required_snippets"), f"{evidence_id}.required_snippets"), evidence_id)


def validate_current_live_path(snapshot: dict[str, Any]) -> None:
    live_path = snapshot.get("current_live_path")
    require(isinstance(live_path, dict), "current_live_path must be an object")
    if not isinstance(live_path, dict):
        return
    require(live_path.get("classification") == "jenkins_bridge_active_bespoke_controller_opt_in", "current_live_path.classification mismatch")
    require(live_path.get("active_orchestrator") == "jenkins_bridge", "active_orchestrator must remain jenkins_bridge")
    require(live_path.get("status_sink") == "Forgejo Commit Status API", "status sink must remain Forgejo Commit Status API")
    trigger_chain = as_list(live_path.get("trigger_chain"), "current_live_path.trigger_chain")
    for required in ["Forgejo PR webhook", "oya-ci-webhook-gateway", "infra/ci/jenkins/Jenkinsfile-oya-ci-gate", "infra/ci/buck2-affected-gate.sh origin/dev"]:
        require(required in trigger_chain, f"trigger_chain missing {required}")
    dispatcher = live_path.get("default_dispatcher_evidence")
    require(isinstance(dispatcher, dict), "default_dispatcher_evidence must be an object")
    if isinstance(dispatcher, dict):
        require(dispatcher.get("dispatcher_default") == "jenkins", "dispatcher_default must be jenkins")
        require(dispatcher.get("controller_dispatcher") == "available_opt_in_only", "controller_dispatcher must be available_opt_in_only")
        require(dispatcher.get("helm_sets_jenkins_url") is True, "helm_sets_jenkins_url must be true")
        require(dispatcher.get("helm_sets_controller_dispatcher") is False, "helm_sets_controller_dispatcher must be false")
        require(dispatcher.get("helm_sets_controller_url") is False, "helm_sets_controller_url must be false")
    require(live_path.get("branch_protection_required_contexts") == EXPECTED_CONTEXTS, "current_live_path branch-protection contexts mismatch")
    validate_current_live_path_drift_blocks(live_path)


def validate_current_live_path_drift_blocks(live_path: dict[str, Any]) -> None:
    host_drift = live_path.get("branch_protection_host_drift")
    require(isinstance(host_drift, dict), "current_live_path.branch_protection_host_drift must be an object")
    if isinstance(host_drift, dict):
        require(host_drift.get("branch_protection_spec_host") == "github_bootstrap_host", "branch-protection host drift must record GitHub bootstrap host")
        require("gh api" in str(host_drift.get("branch_protection_apply_surface", "")), "branch-protection host drift must record gh api apply surface")
        require(host_drift.get("status_sink_claim_for_bridge") == "Forgejo Commit Status API", "branch-protection host drift must preserve Forgejo status sink claim")
        require("GitHub remains bootstrap host" in str(host_drift.get("forgejo_target_host_source", "")), "branch-protection host drift must preserve ADR-0363 bootstrap nuance")
        require(host_drift.get("reconciliation_status") == "open_bootstrap_host_split", "branch-protection host drift status mismatch")
    metadata_drift = live_path.get("reported_status_metadata_drift")
    require(isinstance(metadata_drift, dict), "current_live_path.reported_status_metadata_drift must be an object")
    if isinstance(metadata_drift, dict):
        require(metadata_drift.get("metadata_file") == str(REPORTED_CONTEXTS_PATH.relative_to(REPO_ROOT)), "reported metadata drift file mismatch")
        require(metadata_drift.get("legacy_sink_wording_present") is True, "reported metadata drift must record legacy sink wording")
        require(metadata_drift.get("declared_missing_producer") == "infra/ci/Jenkinsfile", "reported metadata drift must record missing declared producer")
        require(metadata_drift.get("declared_missing_producer_exists") is False, "reported metadata declared producer must remain marked missing")
        require(metadata_drift.get("actual_bridge_producer") == str(JENKINS_GATE_PATH.relative_to(REPO_ROOT)), "reported metadata drift actual bridge producer mismatch")
        require(metadata_drift.get("reconciliation_status") == "open_metadata_hygiene_blocker", "reported metadata drift status mismatch")
    require(live_path.get("bridge_posted_status_contexts") == ["oya-ci-gate"], "current_live_path must record bridge-only oya-ci-gate context")
    require(live_path.get("bridge_context_required_by_dev_branch_protection") is False, "bridge context must remain marked not required by dev branch protection")
    require(live_path.get("bridge_context_declared_in_reported_status_metadata") is False, "bridge context must remain marked absent from reported-status metadata")
    status_note = normalized(live_path.get("status_context_reconciliation_note", ""))
    for phrase in ["oya ci gate", "cargo fmt", "cargo check", "cargo clippy", "cargo nextest", "cargo deny", "oya verify", "reported status metadata"]:
        require(phrase in status_note, f"status_context_reconciliation_note missing {phrase!r}")
    safe_answer = normalized(live_path.get("safe_answer_to_jenkinsfile_question", ""))
    for phrase in ["infra ci jenkins jenkinsfile oya ci gate", "jcasc", "gateway defaults", "root jenkinsfile", "hygiene"]:
        require(phrase in safe_answer, f"safe_answer_to_jenkinsfile_question missing {phrase!r}")
    require("root jenkinsfile is a separate stale" in safe_answer, "safe answer must not treat root Jenkinsfile as active bridge proof")
    require(not contains_forbidden_can_claim(live_path.get("safe_answer_to_jenkinsfile_question", "")), "safe_answer_to_jenkinsfile_question contains forbidden claim")


def validate_bespoke_destination(snapshot: dict[str, Any]) -> None:
    destination = snapshot.get("bespoke_destination")
    require(isinstance(destination, dict), "bespoke_destination must be an object")
    if not isinstance(destination, dict):
        return
    require(destination.get("platform") == "oya-ci bespoke Rust Prow-shaped platform", "bespoke_destination.platform mismatch")
    require(destination.get("accepted_adr") == str(ADR_0513_PATH.relative_to(REPO_ROOT)), "bespoke_destination.accepted_adr must point to ADR-0513")
    require(destination.get("phase_1_controller") == "oya-ci-controller", "bespoke_destination.phase_1_controller mismatch")
    require(destination.get("cutover_status") == "pending_parallel_green_and_delete_jenkins_gate_path", "cutover_status must remain pending")
    not_live_reason = normalized(destination.get("not_live_default_reason", ""))
    for phrase in ["dispatcherkind jenkins", "oya jenkins dispatch url", "oya ci dispatcher controller", "oya ci controller url"]:
        require(phrase in not_live_reason, f"not_live_default_reason must mention {phrase!r}")
    must_not_claim = {normalized(item) for item in as_list(destination.get("must_not_claim"), "bespoke_destination.must_not_claim")}
    for phrase in ["controller is live default", "jenkins has been retired", "argo workflows is live", "production gate is green"]:
        require(normalized(phrase) in must_not_claim, f"bespoke_destination.must_not_claim missing {phrase!r}")


def validate_contradictions(snapshot: dict[str, Any]) -> None:
    rows = as_list(snapshot.get("contradictions_and_resolution"), "contradictions_and_resolution")
    ids: set[str] = {row["id"] for row in rows if isinstance(row, dict) and isinstance(row.get("id"), str)}
    require(ids == REQUIRED_CONTRADICTION_IDS, f"contradiction ids mismatch: missing {sorted(REQUIRED_CONTRADICTION_IDS - ids)} extra {sorted(ids - REQUIRED_CONTRADICTION_IDS)}")
    for row in rows:
        require(isinstance(row, dict), "contradiction rows must be objects")
        row_id = str(row.get("id", "<missing>"))
        for field in ["classification", "conflict", "resolution", "status"]:
            require(isinstance(row.get(field), str) and row.get(field), f"{row_id}: {field} required")
        if row_id in {"controller_opt_in_not_default", "pr_sourced_gate_security_open", "bridge_status_context_not_required_or_reported", "required_context_surface_drift", "reported_status_metadata_legacy_github_producer_drift", "branch_protection_github_bootstrap_vs_forgejo_status_sink"}:
            require(str(row.get("status", "")).startswith("open"), f"{row_id}: must remain open until cutover proof exists")
        validate_contradiction_row_text(row_id, row)


def validate_contradiction_row_text(row_id: str, row: dict[str, Any]) -> None:
    row_text = normalized(row)
    if row_id == "bridge_status_context_not_required_or_reported":
        for phrase in ["oya ci gate", "cargo fmt", "cargo check", "cargo clippy", "cargo nextest", "cargo deny", "oya verify", "reported status metadata"]:
            require(phrase in row_text, f"{row_id}: missing {phrase!r}")
    if row_id == "reported_status_metadata_legacy_github_producer_drift":
        for phrase in ["github status check contexts", "jenkins github app", "infra ci jenkinsfile", "absent", "forgejo", "infra ci jenkins jenkinsfile oya ci gate"]:
            require(phrase in row_text, f"{row_id}: missing {phrase!r}")
    if row_id == "branch_protection_github_bootstrap_vs_forgejo_status_sink":
        for phrase in ["forgejo", "github branch protection rest api", "github com jason931225 oyatie", "gh api", "bootstrap"]:
            require(phrase in row_text, f"{row_id}: missing {phrase!r}")
    if row_id == "root_jenkinsfile_path_comment_drift":
        for phrase in ["root jenkinsfile", "hygiene", "infra ci jenkins jenkinsfile oya ci gate", "active bridge"]:
            require(phrase in row_text, f"{row_id}: missing {phrase!r}")


def validate_retirement_gate(snapshot: dict[str, Any]) -> None:
    gate = snapshot.get("jenkins_retirement_gate")
    require(isinstance(gate, dict), "jenkins_retirement_gate must be an object")
    if not isinstance(gate, dict):
        return
    require(gate.get("current_verdict") == "retain_as_bridge", "Jenkins retirement verdict must be retain_as_bridge")
    require(gate.get("delete_allowed_now") is False, "jenkins_retirement_gate.delete_allowed_now must be false")
    requirements = as_list(gate.get("required_before_deletion"), "jenkins_retirement_gate.required_before_deletion")
    require(len(requirements) >= 11, "Jenkins deletion gate must have at least eleven prerequisites")
    combined = normalized(requirements)
    for phrase in [
        "oya ci dispatcher controller",
        "oya ci controller url",
        "trusted dev source",
        "pr ref as data",
        "required forgejo status context",
        "bridge only oya ci gate context",
        "producer declaration",
        "parallel run evidence",
        "pr sourced changes cannot weaken",
        "structured failure summary",
        "reported status metadata",
        "infra ci jenkinsfile",
        "github bootstrap branch protection",
        "forgejo status sink",
        "status context drift",
    ]:
        require(phrase in combined, f"jenkins_retirement_gate.required_before_deletion missing {phrase!r}")


def validate_machine_surfaces(contract: dict[str, Any]) -> None:
    surfaces = {row.get("id"): row for row in as_list(contract.get("machine_check_surfaces"), "machine_check_surfaces") if isinstance(row, dict)}
    for surface_id in ["g007-contract-validator", "g007-snapshot-freshness-validator", "g007-self-test"]:
        require(surface_id in surfaces, f"machine_check_surfaces must include {surface_id}")
    require(surfaces["g007-contract-validator"].get("command") == "python3 scripts/tests/cloud_dogfood_ci_toolchain_lane_reconciliation_check.py", "g007-contract-validator command mismatch")
    require(surfaces["g007-snapshot-freshness-validator"].get("command") == "python3 scripts/tests/cloud_dogfood_ci_toolchain_lane_reconciliation_check.py", "g007-snapshot-freshness-validator command mismatch")
    require(surfaces["g007-self-test"].get("command") == "python3 scripts/tests/cloud_dogfood_ci_toolchain_lane_reconciliation_check.py --self-test", "g007-self-test command mismatch")


def validate_nonclaims(contract: dict[str, Any]) -> None:
    nonclaims = {row.get("id") for row in as_list(contract.get("nonclaims"), "nonclaims") if isinstance(row, dict)}
    required = {"custom_ci_live_default", "jenkins_retired", "argo_workflows_live", "external_ci_fallback", "live_cluster_evidence", "production_gate_green"}
    require(required <= nonclaims, f"missing nonclaims {sorted(required - nonclaims)}")


def validate_live_repo_cross_checks() -> None:
    branch_protection = load_json(DEV_BRANCH_PROTECTION_PATH)
    branch_protection_text = read_text(DEV_BRANCH_PROTECTION_PATH)
    require("GitHub branch-protection REST API" in branch_protection_text, "dev branch-protection spec no longer names GitHub REST API; update G007 host drift")
    github_branch_yaml = read_text(GITHUB_BRANCH_PROTECTION_YAML_PATH)
    require("github.com/jason931225/oyatie" in github_branch_yaml, "GitHub branch-protection YAML no longer names deployed GitHub ruleset; update G007 host drift")
    apply_script = read_text(BRANCH_PROTECTION_APPLY_PATH)
    for snippet in ["Synchronize GitHub required status-check protection", "gh api", "protection/required_status_checks"]:
        require(snippet in apply_script, f"branch-protection apply script missing {snippet!r}; update G007 host drift")
    contexts = branch_protection.get("required_status_checks", {}).get("contexts")
    require(contexts == EXPECTED_CONTEXTS, "infra/branch-protection/dev.json contexts changed; update G007 deliberately")

    gateway_config = read_text(GATEWAY_CONFIG_PATH)
    for snippet in ["Default is `Jenkins`", "OYA_CI_DISPATCHER=controller", "unwrap_or(DispatcherKind::Jenkins)"]:
        require(snippet in gateway_config, f"gateway config missing {snippet!r}")

    deployment = read_text(GATEWAY_DEPLOYMENT_PATH)
    require("OYA_JENKINS_DISPATCH_URL" in deployment, "gateway Helm deployment must still set OYA_JENKINS_DISPATCH_URL for bridge evidence")
    require("OYA_CI_DISPATCHER" not in deployment, "gateway Helm deployment now selects dispatcher; update G007 cutover status")
    require("OYA_CI_CONTROLLER_URL" not in deployment, "gateway Helm deployment now sets controller URL; update G007 cutover status")

    values = read_text(GATEWAY_VALUES_PATH)
    require("jenkins:" in values and "dispatchUrl:" in values, "gateway Helm values must still contain Jenkins dispatch URL")
    require("oya-jenkins.oya-ci-jenkins.svc.cluster.local" in values, "gateway Helm values no longer point at in-cluster Jenkins; update G007")

    jenkins_values = read_text(JENKINS_VALUES_PATH)
    require("pipelineJob('oya-ci-gate')" in jenkins_values, "JCasC no longer wires oya-ci-gate; update G007")
    require("scriptPath('infra/ci/jenkins/Jenkinsfile-oya-ci-gate')" in jenkins_values, "JCasC no longer points at bridge Jenkinsfile")

    jenkins_gate = read_text(JENKINS_GATE_PATH)
    for snippet in ["infra/ci/buck2-affected-gate.sh origin/dev", "postForgejoStatus('success'", "postForgejoStatus('failure'", '"context":"oya-ci-gate"']:
        require(snippet in jenkins_gate, f"bridge Jenkinsfile missing {snippet!r}")
    bridge_contexts = set(re.findall(r'"context":"([^"]+)"', jenkins_gate))
    require(bridge_contexts == {"oya-ci-gate"}, f"bridge Jenkinsfile contexts changed: {sorted(bridge_contexts)}")

    buck2_gate = read_text(BUCK2_GATE_PATH)
    require('"$BUCK2" build' in buck2_gate and '"$BUCK2" test' in buck2_gate, "Buck2 affected gate must still build and test")

    chart = read_text(CONTROLLER_CHART_PATH)
    require("bespoke-Prow plank+crier" in chart and "spawns K8s gate Jobs" in chart, "controller chart evidence changed")

    app = read_text(CONTROLLER_APP_PATH)
    for snippet in ["POST /gate-run", "Job name is deterministic", "409 create-conflict no-op", "posts Forgejo statuses"]:
        require(snippet in app, f"controller app evidence missing {snippet!r}")

    kernel = read_text(CONTROLLER_KERNEL_PATH)
    for snippet in ["No I/O, no async, no kube, no tokio", "#![forbid(unsafe_code)]", "pub enum ForgejoState", "pub struct GateRun"]:
        require(snippet in kernel, f"controller kernel evidence missing {snippet!r}")

    adr_0513 = read_text(ADR_0513_PATH)
    adr_0511 = read_text(ADR_0511_PATH)
    require("status: Accepted" in adr_0513, "ADR-0513 must remain Accepted for G007 precedence")
    require("status: Proposed" in adr_0511, "ADR-0511 must remain Proposed for G007 precedence")

    reported = load_json(REPORTED_CONTEXTS_PATH)
    meta = reported.get("_meta", {})
    require(isinstance(meta, dict), "reported-status metadata _meta must be an object")
    if isinstance(meta, dict):
        meta_text = normalized(meta)
        require("github status check contexts" in meta_text, "reported-status metadata legacy GitHub wording disappeared; update G007")
        require("jenkins github app" in meta_text, "reported-status metadata legacy Jenkins GitHub App wording disappeared; update G007")
        require("infra ci jenkinsfile" in meta_text, "reported-status metadata no longer names infra/ci/Jenkinsfile; update G007")
    require(not (REPO_ROOT / "infra" / "ci" / "Jenkinsfile").exists(), "infra/ci/Jenkinsfile now exists; update reported metadata drift in G007")
    reported_contexts = set(as_list(reported.get("reported_status_contexts"), "reported_status_contexts"))
    require(set(EXPECTED_CONTEXTS) <= reported_contexts, "reported-status contexts must cover required dev contexts")
    require("oya-pr-review" in reported_contexts, "reported-status context drift source no longer includes oya-pr-review; update G007")
    require("oya-ci-gate" not in reported_contexts, "reported-status metadata now declares oya-ci-gate; update G007 context-drift resolution")
    require("oya-ci-gate" not in set(EXPECTED_CONTEXTS), "dev branch-protection expected contexts unexpectedly include oya-ci-gate; update G007")
    require(reported_contexts != set(EXPECTED_CONTEXTS), "producer context drift disappeared; update G007")


def validate(contract: dict[str, Any], snapshot: dict[str, Any]) -> None:
    validate_snapshot_header(snapshot)
    validate_stable_contract(contract, snapshot)
    validate_policy(contract)
    validate_claim_controls(contract)
    validate_source_evidence(snapshot)
    validate_current_live_path(snapshot)
    validate_bespoke_destination(snapshot)
    validate_contradictions(snapshot)
    validate_retirement_gate(snapshot)
    validate_machine_surfaces(contract)
    validate_nonclaims(contract)
    require(contract.get("next_goal_links", {}).get("final_quality_gate") == "G008", "next_goal_links.final_quality_gate must be G008")
    validate_live_repo_cross_checks()


def run_self_tests() -> None:
    baseline_contract = load_json(SPEC_PATH)
    baseline_snapshot = load_json(SNAPSHOT_PATH)

    def expect_rejected(label: str, mutator: Mutator) -> None:
        contract = json.loads(json.dumps(baseline_contract))
        snapshot = json.loads(json.dumps(baseline_snapshot))
        mutator(contract, snapshot)
        try:
            validate(contract, snapshot)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    def remove_contradiction(_contract: dict[str, Any], snapshot: dict[str, Any]) -> None:
        snapshot["contradictions_and_resolution"] = snapshot["contradictions_and_resolution"][1:]

    def source_snippet_drift(_contract: dict[str, Any], snapshot: dict[str, Any]) -> None:
        snapshot["source_evidence"][0]["required_snippets"] = ["not present in ADR"]

    def remove_reported_metadata_drift(_contract: dict[str, Any], snapshot: dict[str, Any]) -> None:
        del snapshot["current_live_path"]["reported_status_metadata_drift"]

    def remove_branch_host_drift(_contract: dict[str, Any], snapshot: dict[str, Any]) -> None:
        del snapshot["current_live_path"]["branch_protection_host_drift"]

    def overclaim_root_jenkinsfile(_contract: dict[str, Any], snapshot: dict[str, Any]) -> None:
        for row in snapshot["source_evidence"]:
            if row["id"] == "root-jenkinsfile-hygiene-surface":
                row.update({"status": "wired-source", "role": "root Jenkinsfile has active bridge role plus stale path comments"})

    def fuse_snapshot_into_contract(contract: dict[str, Any], snapshot: dict[str, Any]) -> None:
        contract["current_live_path"] = snapshot["current_live_path"]

    def remove_snapshot_artifact(contract: dict[str, Any], _snapshot: dict[str, Any]) -> None:
        contract["stable_contract"].pop("snapshot_artifact", None)

    def missing_required_class(contract: dict[str, Any], _snapshot: dict[str, Any]) -> None:
        contract["stable_contract"]["required_reconciliation_classes"] = contract["stable_contract"]["required_reconciliation_classes"][1:]

    expect_rejected("custom CI live default claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["custom CI is live default"]}))
    expect_rejected("Jenkins retired claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["Jenkins retired and removable"]}))
    expect_rejected("Argo live claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["Argo Workflows is live"]}))
    expect_rejected("GitHub Actions fallback claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["GitHub Actions fallback allowed"]}))
    expect_rejected("production CI green claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["production CI is green"]}))
    expect_rejected("safe deletion of Jenkins claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["safe deletion of Jenkins is allowed"]}))
    expect_rejected("now safe to delete Jenkins claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["it is now safe to delete Jenkins"]}))
    expect_rejected("short Argo live claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["Argo is live"]}))
    expect_rejected("Argo cutover claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["Argo has cutover complete"]}))
    expect_rejected("reversed custom live default claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["the live default is the custom CI controller"]}))
    expect_rejected("production gate green claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["production gate is green"]}))
    expect_rejected("PR-sourced remediated claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["PR-sourced gate has been remediated"]}))
    expect_rejected("controller cutover complete claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["controller cutover is complete"]}))
    expect_rejected("controller cutover completed claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["controller cutover completed"]}))
    expect_rejected("controller has cut over claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["controller has cut over"]}))
    expect_rejected("branch protection cutover claim", lambda contract, _snapshot: contract["claim_controls"].update({"can_claim_now": ["branch protection has cut over to controller contexts"]}))
    expect_rejected("controller active orchestrator", lambda _contract, snapshot: snapshot["current_live_path"].update({"active_orchestrator": "controller_live"}))
    expect_rejected("dispatcher no longer Jenkins", lambda _contract, snapshot: snapshot["current_live_path"]["default_dispatcher_evidence"].update({"dispatcher_default": "controller"}))
    expect_rejected("controller dispatcher selected", lambda _contract, snapshot: snapshot["current_live_path"]["default_dispatcher_evidence"].update({"helm_sets_controller_dispatcher": True}))
    expect_rejected("allow delete Jenkins", lambda _contract, snapshot: snapshot["jenkins_retirement_gate"].update({"delete_allowed_now": True}))
    expect_rejected("delete no-delete control", lambda contract, _snapshot: contract["claim_controls"].update({"no_delete_jenkins_now": False}))
    expect_rejected("missing contradiction", remove_contradiction)
    expect_rejected("branch contexts changed", lambda _contract, snapshot: snapshot["current_live_path"].update({"branch_protection_required_contexts": ["oya-verify"]}))
    expect_rejected("source snippet drift", source_snippet_drift)
    expect_rejected("bridge context marked required", lambda _contract, snapshot: snapshot["current_live_path"].update({"bridge_context_required_by_dev_branch_protection": True}))
    expect_rejected("bridge context marked reported", lambda _contract, snapshot: snapshot["current_live_path"].update({"bridge_context_declared_in_reported_status_metadata": True}))
    expect_rejected("missing bridge context note", lambda _contract, snapshot: snapshot["current_live_path"].update({"status_context_reconciliation_note": "Jenkins bridge exists"}))
    expect_rejected("reported metadata drift missing", remove_reported_metadata_drift)
    expect_rejected("reported metadata producer marked present", lambda _contract, snapshot: snapshot["current_live_path"]["reported_status_metadata_drift"].update({"declared_missing_producer_exists": True}))
    expect_rejected("branch-protection host drift missing", remove_branch_host_drift)
    expect_rejected("branch-protection host marked Forgejo", lambda _contract, snapshot: snapshot["current_live_path"]["branch_protection_host_drift"].update({"branch_protection_spec_host": "forgejo"}))
    expect_rejected("root Jenkinsfile overclaimed as active bridge", overclaim_root_jenkinsfile)
    expect_rejected("safe answer omits root hygiene split", lambda _contract, snapshot: snapshot["current_live_path"].update({"safe_answer_to_jenkinsfile_question": "The root Jenkinsfile and Jenkins gate files are migration debt with active bridge value."}))
    expect_rejected("requires network", lambda contract, _snapshot: contract["validation_policy"].update({"network_required": True}))
    expect_rejected("requires cluster", lambda contract, _snapshot: contract["validation_policy"].update({"cluster_required": True}))
    expect_rejected("status changed", lambda contract, _snapshot: contract.update({"status": "Implemented"}))
    expect_rejected("destination accepted ADR changed", lambda _contract, snapshot: snapshot["bespoke_destination"].update({"accepted_adr": "docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md"}))
    expect_rejected("missing G008 link", lambda contract, _snapshot: contract.update({"next_goal_links": {}}))
    expect_rejected("contract fused with snapshot field", fuse_snapshot_into_contract)
    expect_rejected("contract missing snapshot artifact", remove_snapshot_artifact)
    expect_rejected("contract missing required reconciliation class", missing_required_class)
    expect_rejected("snapshot id changed", lambda _contract, snapshot: snapshot.update({"snapshot_id": "wrong"}))
    print("cloud dogfood CI/toolchain lane reconciliation self-tests passed")


def main() -> None:
    contract = load_json(SPEC_PATH)
    snapshot = load_json(SNAPSHOT_PATH)
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    validate(contract, snapshot)
    print(f"cloud dogfood CI/toolchain lane reconciliation check passed: {SPEC_PATH.relative_to(REPO_ROOT)} + {SNAPSHOT_PATH.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
