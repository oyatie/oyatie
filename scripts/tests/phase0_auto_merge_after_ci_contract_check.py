#!/usr/bin/env python3
"""Validate the P0.0 auto-merge-after-CI contract is executable and closed.

This check intentionally inspects the active scripts/docs/code surfaces that arm
Forgejo and GitHub auto-merge. It is not a live-green claim; it prevents checked-
in regressions to stale contexts, unpinned PR heads, missing conflict guards, or
Cargo/oya local authority language.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(os.environ.get("OYA_REPO_ROOT", Path(__file__).resolve().parents[2])).resolve()
SPEC = REPO_ROOT / "specs/phase0-auto-merge-after-ci.json"


def read(path: str) -> str:
    return (REPO_ROOT / path).read_text()


def load_json(path: Path) -> Any:
    with path.open() as fh:
        return json.load(fh)


def require_contains(text: str, needle: str, label: str, failures: list[str]) -> None:
    if needle not in text:
        failures.append(f"{label}: missing {needle!r}")


def main() -> int:
    failures: list[str] = []
    spec = load_json(SPEC)

    if spec.get("required_context") != "github-lane-unlocker-required":
        failures.append("spec.required_context must be github-lane-unlocker-required during the GitHub lane unlocker")
    if spec.get("p0_0_green") is not False or spec.get("phase0_complete") is not False:
        failures.append("spec must retain p0_0_green=false and phase0_complete=false")

    github = spec.get("github", {})
    forgejo = spec.get("forgejo", {})
    if github.get("auto_merge_flag") != "--auto":
        failures.append("github.auto_merge_flag must be --auto")
    if github.get("head_pin_flag") != "--match-head-commit":
        failures.append("github.head_pin_flag must be --match-head-commit")
    if github.get("allowed_merge_methods") != ["squash"]:
        failures.append("github.allowed_merge_methods must be ['squash']")
    if github.get("script_rejects_non_squash_merge_method") is not True:
        failures.append("github.script_rejects_non_squash_merge_method must be true")
    if github.get("script_rejects_conflict_before_auto_merge") is not True:
        failures.append("github.script_rejects_conflict_before_auto_merge must be true")
    if github.get("trigger_non_dry_run_merge_path_tested") is not True:
        failures.append("github.trigger_non_dry_run_merge_path_tested must be true")
    if github.get("trigger_non_dry_run_merge_path_evidence_scope") != "local_sequencing_regression_guard_not_live_authority_proof":
        failures.append("github.trigger_non_dry_run_merge_path_evidence_scope must label local evidence scope")
    if github.get("trigger_conflict_guard_test") != "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh":
        failures.append("github.trigger_conflict_guard_test must name the trigger-level conflict guard test")
    if github.get("required_context_rollup_check") != "scripts/ci/assert-pr-required-context.rs":
        failures.append("github.required_context_rollup_check must name the non-mutating rollup checker")
    if github.get("required_context_rollup_test") != "scripts/tests/phase0_required_context_rollup_check.rs":
        failures.append("github.required_context_rollup_test must name the rollup fixture test")
    if github.get("trusted_required_context_producer") != "github-lane-unlocker-ci-cd":
        failures.append("github.trusted_required_context_producer must be github-lane-unlocker-ci-cd")
    if github.get("script_rejects_missing_required_context_producer") is not True:
        failures.append("github.script_rejects_missing_required_context_producer must be true")
    if github.get("script_rejects_untrusted_required_context_producer") is not True:
        failures.append("github.script_rejects_untrusted_required_context_producer must be true")
    if github.get("live_no_checks_reported_failure_reason") != "no_status_checks_reported":
        failures.append("github.live_no_checks_reported_failure_reason must be no_status_checks_reported")
    if github.get("script_detects_missing_live_required_context") is not True:
        failures.append("github.script_detects_missing_live_required_context must be true")
    for fixture in [
        "specs/fixtures/phase0-required-context-rollup/good-nested-github-lane-unlocker-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-missing-producer.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-untrusted-producer.json",
    ]:
        if fixture not in github.get("required_context_rollup_fixtures", []):
            failures.append(f"github.required_context_rollup_fixtures missing {fixture}")
    if forgejo.get("schedule_field") != "merge_when_checks_succeed":
        failures.append("forgejo.schedule_field must be merge_when_checks_succeed")
    if forgejo.get("head_pin_field") != "head_commit_id":
        failures.append("forgejo.head_pin_field must be head_commit_id")
    if forgejo.get("script_requires_mergeability_guard") is not True:
        failures.append("forgejo.script_requires_mergeability_guard must be true")
    if forgejo.get("allowed_merge_methods") != ["squash"]:
        failures.append("forgejo.allowed_merge_methods must be ['squash']")
    if forgejo.get("delete_branch_after_merge_locked") is not True:
        failures.append("forgejo.delete_branch_after_merge_locked must be true")
    if forgejo.get("tide_required_context_hard_pinned") is not True:
        failures.append("forgejo.tide_required_context_hard_pinned must be true")
    if forgejo.get("tide_merge_method_hard_pinned") != "squash":
        failures.append("forgejo.tide_merge_method_hard_pinned must be squash")
    if forgejo.get("tide_head_pin_full_sha_guard") is not True:
        failures.append("forgejo.tide_head_pin_full_sha_guard must be true")

    buck2_enforcement = spec.get("buck2_enforcement", {})
    if buck2_enforcement.get("github_bootstrap_test") != "//:github-auto-merge-after-ci-check":
        failures.append("buck2_enforcement.github_bootstrap_test must be //:github-auto-merge-after-ci-check")
    if buck2_enforcement.get("github_required_context_rollup_test") != "//:github-auto-merge-after-ci-check":
        failures.append("buck2_enforcement.github_required_context_rollup_test must be //:github-auto-merge-after-ci-check")

    trigger = read("scripts/trigger-next-queue-automerge.sh")
    require_contains(trigger, "--auto --match-head-commit", "github trigger", failures)
    require_contains(trigger, "scripts/check-sequential-pr-merge-conflicts.sh", "github trigger", failures)
    require_contains(trigger, "live branch-protection required contexts drift", "github trigger", failures)
    require_contains(trigger, "--merge-method is fixed to squash", "github trigger", failures)
    require_contains(trigger, "--fetch-remote", "github trigger", failures)
    require_contains(trigger, 'remote_url_contains_github "github-mirror"', "github trigger", failures)
    require_contains(trigger, 'merge_flag="--squash"', "github trigger", failures)
    require_contains(trigger, "gh pr merge \"$number\" \"$merge_flag\" --auto --match-head-commit \"$head_oid\"", "github trigger", failures)

    conflict_guard = read("scripts/check-sequential-pr-merge-conflicts.sh")
    require_contains(conflict_guard, "--fetch-remote <remote>", "conflict guard", failures)
    require_contains(conflict_guard, 'git fetch --no-tags "$fetch_remote"', "conflict guard", failures)
    require_contains(conflict_guard, "pass --fetch-remote for the GitHub mirror when origin is Forgejo", "conflict guard", failures)

    forge_script = read("scripts/ci/arm-auto-merge.sh")
    require_contains(forge_script, 'REQUIRED_CONTEXT="oya-ci-required"', "forgejo script", failures)
    require_contains(forge_script, "REQUIRED_CONTEXT is fixed to oya-ci-required", "forgejo script", failures)
    require_contains(forge_script, "--merge-method is fixed to squash", "forgejo script", failures)
    require_contains(forge_script, "--delete-branch-after-merge is fixed to true", "forgejo script", failures)
    require_contains(forge_script, "merge_when_checks_succeed", "forgejo script", failures)
    require_contains(forge_script, "head_commit_id", "forgejo script", failures)
    require_contains(forge_script, "delete_branch_after_merge", "forgejo script", failures)
    require_contains(forge_script, "--head-commit is required with --pr-index", "forgejo script", failures)
    require_contains(forge_script, "--head-commit must be a full SHA-1 (40 hex) or SHA-256 (64 hex) commit id", "forgejo script", failures)
    require_contains(forge_script, "pulls/${PR_INDEX}/merge", "forgejo script", failures)
    require_contains(forge_script, "validate_pr_ready_for_auto_merge", "forgejo script", failures)
    require_contains(forge_script, 'pr_resp="$(forge_api GET "${PR_ITEM_ENDPOINT}")"', "forgejo script", failures)
    require_contains(forge_script, 'head = d.get("head") or {}', "forgejo script", failures)
    require_contains(forge_script, 'head_sha = head.get("sha") or ""', "forgejo script", failures)
    require_contains(forge_script, 'mergeable = d.get("mergeable", None)', "forgejo script", failures)
    require_contains(forge_script, "does not match expected", "forgejo script", failures)
    require_contains(forge_script, "PR is not mergeable according to Forgejo", "forgejo script", failures)

    tide_adapter = read("oya/ci-tide/crates/oya-ci-tide-forgejo-adapter/src/lib.rs")
    require_contains(tide_adapter, "merge_when_checks_succeed: true", "tide adapter", failures)
    require_contains(tide_adapter, "delete_branch_after_merge: true", "tide adapter", failures)
    require_contains(tide_adapter, "head_commit_id: head_sha.to_owned()", "tide adapter", failures)
    require_contains(tide_adapter, "P0.0 Tide auto-merge scheduling is squash-only", "tide adapter", failures)
    require_contains(tide_adapter, "is_full_hex_commit_id(head_sha)", "tide adapter", failures)
    require_contains(tide_adapter, "head_sha must be a full SHA-1 (40 hex) or SHA-256 (64 hex) commit id", "tide adapter", failures)

    tide_kernel = read("oya/ci-tide/crates/oya-ci-tide-kernel/src/lib.rs")
    require_contains(tide_kernel, "let required_status_context = DEFAULT_REQUIRED_STATUS_CONTEXT.to_owned();", "tide kernel", failures)
    require_contains(tide_kernel, "let merge_method = MergeMethod::Squash;", "tide kernel", failures)
    require_contains(tide_kernel, "configured_required_status_context_cannot_override_phase0_default", "tide kernel", failures)
    require_contains(tide_kernel, "configured_merge_method_cannot_override_phase0_squash_default", "tide kernel", failures)
    require_contains(tide_kernel, 'assert_eq!(MergeMethod::from_str("merge"), MergeMethod::Squash);', "tide kernel", failures)

    tide_app = read("oya/ci-tide/crates/oya-ci-tide-app/src/lib.rs")
    require_contains(tide_app, "&fresh_pr.head_sha", "tide app", failures)

    github_test = read("scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh")
    require_contains(github_test, "--merge-method is fixed to squash", "github trigger test", failures)

    rollup_check = read("scripts/ci/assert-pr-required-context.rs")
    require_contains(rollup_check, "no_status_checks_reported", "required context rollup check", failures)
    require_contains(rollup_check, "missing_required_context", "required context rollup check", failures)
    require_contains(rollup_check, "required_context_not_success", "required context rollup check", failures)
    require_contains(rollup_check, "github-lane-unlocker-required", "required context rollup check", failures)
    require_contains(rollup_check, "github-lane-unlocker-ci-cd", "required context rollup check", failures)
    require_contains(rollup_check, "missing_required_context_producer", "required context rollup check", failures)
    require_contains(rollup_check, "untrusted_required_context_producer", "required context rollup check", failures)
    require_contains(rollup_check, "required_context_trusted_producer", "required context rollup check", failures)
    require_contains(rollup_check, "status-rollup evidence only; this checker never posts statuses", "required context rollup check", failures)

    rollup_test = read("scripts/tests/phase0_required_context_rollup_check.rs")
    require_contains(rollup_test, "bad-no-checks-reported.json", "required context rollup test", failures)
    require_contains(rollup_test, "no_status_checks_reported", "required context rollup test", failures)
    require_contains(rollup_test, "required_context_not_success", "required context rollup test", failures)
    require_contains(rollup_test, "good-nested-github-lane-unlocker-required-success.json", "required context rollup test", failures)
    require_contains(rollup_test, "missing_required_context_producer", "required context rollup test", failures)
    require_contains(rollup_test, "untrusted_required_context_producer", "required context rollup test", failures)

    github_conflict_test = read("scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh")
    require_contains(github_conflict_test, "sequential PR merge simulation passed: 1 PRs modeled", "github conflict guard test", failures)
    require_contains(github_conflict_test, "dry-run: gh pr merge 455 --squash --auto --match-head-commit", "github conflict guard test", failures)
    require_contains(github_conflict_test, "Local sequencing regression guard", "github conflict guard test", failures)
    require_contains(github_conflict_test, "clean_real_work=", "github conflict guard test", failures)
    require_contains(github_conflict_test, "guard_marker=guard passed:", "github conflict guard test", failures)
    require_contains(github_conflict_test, "pr merge 455 --squash --auto --match-head-commit", "github conflict guard test", failures)
    require_contains(github_conflict_test, "::error::sequential merge conflict at PR #455", "github conflict guard test", failures)
    require_contains(github_conflict_test, "conflict scenario invoked gh pr merge", "github conflict guard test", failures)

    conflict_test = read("scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh")
    require_contains(conflict_test, "--fetch-remote github-mirror", "conflict guard test", failures)
    require_contains(conflict_test, "failed to fetch PR #455 head from remote origin", "conflict guard test", failures)

    for path in ["docs/ci/auto-merge-flow.md", "docs/ci/forge-of-record.md"]:
        text = read(path)
        require_contains(text, "github-lane-unlocker-required", path, failures)
        require_contains(text, "oya-ci-required", path, failures)
        require_contains(text, "--match-head-commit", path, failures)
        require_contains(text, "head_commit_id", path, failures)
        if "oya-ci-gate" in text:
            failures.append(f"{path}: must not reference stale oya-ci-gate")

    policy = load_json(REPO_ROOT / "specs/buck2-authority-policy.json")
    for required in [
        "scripts/ci/arm-auto-merge.sh",
        "scripts/trigger-next-queue-automerge.sh",
        "scripts/check-sequential-pr-merge-conflicts.sh",
        "scripts/tests/forgejo_auto_merge_after_ci.test.sh",
        "scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh",
        "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh",
        "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh",
        "scripts/tests/phase0_required_context_rollup_check.rs",
        "scripts/ci/assert-pr-required-context.rs",
        "scripts/tests/phase0_auto_merge_after_ci_contract_check.py",
        "docs/ci/auto-merge-flow.md",
        "docs/ci/forge-of-record.md",
        "specs/phase0-auto-merge-after-ci.json",
        "oya/ci-tide/crates/oya-ci-tide-kernel/src/lib.rs",
        "oya/ci-tide/crates/oya-ci-tide-app/src/lib.rs",
        "oya/ci-tide/crates/oya-ci-tide-forgejo-adapter/src/lib.rs",
    ]:
        if required not in policy.get("command_scan_files", []):
            failures.append(f"buck2 policy command_scan_files missing {required}")

    if failures:
        print("phase0-auto-merge-after-ci-contract: RED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(json.dumps({
        "verdict": "PASS",
        "spec": "specs/phase0-auto-merge-after-ci.json",
        "required_context": "github-lane-unlocker-required",
        "checks": {
            "github_auto_merge_head_pinned": True,
            "forgejo_auto_merge_after_ci_head_pinned": True,
            "forgejo_mergeability_guard_declared": True,
            "tide_context_hard_pinned": True,
            "tide_squash_only": True,
            "tide_full_sha_guard_declared": True,
            "conflict_guard_declared": True,
            "trigger_conflict_guard_tested": True,
            "trigger_non_dry_run_merge_path_tested": True,
            "trigger_non_dry_run_merge_path_scope_labeled": True,
            "required_context_rollup_check_tested": True,
            "buck2_policy_scan_covered": True,
            "p0_0_green": False,
            "phase0_complete": False,
        },
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
