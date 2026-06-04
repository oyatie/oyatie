//! Validate the P0.0 auto-merge-after-CI contract is executable and closed.
//!
//! This check intentionally inspects the active scripts/docs/code surfaces that
//! arm Forgejo and GitHub auto-merge. It is not a live-green claim; it prevents
//! checked-in regressions to stale contexts, unpinned PR heads, missing conflict
//! guards, or Cargo/oya local authority language.

#[allow(dead_code)]
#[path = "../ci/assert-result-bundle-output.rs"]
mod json_support;

use json_support::Json;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SPEC: &str = "specs/phase0-auto-merge-after-ci.json";

fn repo_root() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::current_dir().unwrap_or_else(|error| panic!("current_dir failed: {error}"))
        })
}

fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn load_json(root: &Path, path: &str) -> Json {
    let text = read(root, path);
    json_support::parse_json(&text).unwrap_or_else(|error| panic!("parse {path}: {error}"))
}

fn object<'a>(
    value: &'a Json,
    label: &str,
    failures: &mut Vec<String>,
) -> &'a BTreeMap<String, Json> {
    value.as_object().unwrap_or_else(|| {
        failures.push(format!("{label} must be an object"));
        static EMPTY: std::sync::OnceLock<BTreeMap<String, Json>> = std::sync::OnceLock::new();
        EMPTY.get_or_init(BTreeMap::new)
    })
}

fn field<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Option<&'a Json> {
    object.get(key)
}

fn string_field(object: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    field(object, key)
        .and_then(Json::as_str)
        .map(str::to_string)
}

fn bool_field(object: &BTreeMap<String, Json>, key: &str) -> Option<bool> {
    field(object, key).and_then(Json::as_bool)
}

fn object_field<'a>(
    object: &'a BTreeMap<String, Json>,
    key: &str,
) -> Option<&'a BTreeMap<String, Json>> {
    field(object, key).and_then(Json::as_object)
}

fn string_array_field(object: &BTreeMap<String, Json>, key: &str) -> Vec<String> {
    field(object, key)
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Json::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn require_contains(text: &str, needle: &str, label: &str, failures: &mut Vec<String>) {
    if !text.contains(needle) {
        failures.push(format!("{label}: missing {needle:?}"));
    }
}

fn require_string(
    object: &BTreeMap<String, Json>,
    key: &str,
    expected: &str,
    message: &str,
    failures: &mut Vec<String>,
) {
    if string_field(object, key).as_deref() != Some(expected) {
        failures.push(message.to_string());
    }
}

fn require_bool_true(
    object: &BTreeMap<String, Json>,
    key: &str,
    message: &str,
    failures: &mut Vec<String>,
) {
    if bool_field(object, key) != Some(true) {
        failures.push(message.to_string());
    }
}

fn require_string_array_eq(
    object: &BTreeMap<String, Json>,
    key: &str,
    expected: &[&str],
    message: &str,
    failures: &mut Vec<String>,
) {
    let actual = string_array_field(object, key);
    let expected = expected
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    if actual != expected {
        failures.push(message.to_string());
    }
}

fn require_array_contains(
    object: &BTreeMap<String, Json>,
    key: &str,
    expected: &str,
    message: &str,
    failures: &mut Vec<String>,
) {
    if !string_array_field(object, key)
        .iter()
        .any(|item| item == expected)
    {
        failures.push(message.to_string());
    }
}

fn validate_spec(spec: &BTreeMap<String, Json>, failures: &mut Vec<String>) {
    require_string(
        spec,
        "required_context",
        "github-lane-unlocker-required",
        "spec.required_context must be github-lane-unlocker-required during the GitHub lane unlocker",
        failures,
    );
    if bool_field(spec, "p0_0_green") != Some(false)
        || bool_field(spec, "phase0_complete") != Some(false)
    {
        failures.push("spec must retain p0_0_green=false and phase0_complete=false".to_string());
    }

    let Some(github) = object_field(spec, "github") else {
        failures.push("spec.github must be an object".to_string());
        return;
    };
    let Some(forgejo) = object_field(spec, "forgejo") else {
        failures.push("spec.forgejo must be an object".to_string());
        return;
    };

    require_string(
        github,
        "auto_merge_flag",
        "--auto",
        "github.auto_merge_flag must be --auto",
        failures,
    );
    require_string(
        github,
        "head_pin_flag",
        "--match-head-commit",
        "github.head_pin_flag must be --match-head-commit",
        failures,
    );
    require_string_array_eq(
        github,
        "allowed_merge_methods",
        &["squash"],
        "github.allowed_merge_methods must be ['squash']",
        failures,
    );
    require_bool_true(
        github,
        "script_rejects_non_squash_merge_method",
        "github.script_rejects_non_squash_merge_method must be true",
        failures,
    );
    require_bool_true(
        github,
        "script_rejects_conflict_before_auto_merge",
        "github.script_rejects_conflict_before_auto_merge must be true",
        failures,
    );
    require_bool_true(
        github,
        "trigger_non_dry_run_merge_path_tested",
        "github.trigger_non_dry_run_merge_path_tested must be true",
        failures,
    );
    require_string(
        github,
        "trigger_non_dry_run_merge_path_evidence_scope",
        "local_sequencing_regression_guard_not_live_authority_proof",
        "github.trigger_non_dry_run_merge_path_evidence_scope must label local evidence scope",
        failures,
    );
    require_string(
        github,
        "trigger_conflict_guard_test",
        "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh",
        "github.trigger_conflict_guard_test must name the trigger-level conflict guard test",
        failures,
    );
    require_string(
        github,
        "conflict_guard_compatibility_entrypoint",
        "scripts/check-sequential-pr-merge-conflicts.sh",
        "github.conflict_guard_compatibility_entrypoint must preserve the legacy CLI path",
        failures,
    );
    require_string(
        github,
        "conflict_guard_implementation",
        "scripts/check-sequential-pr-merge-conflicts.rs",
        "github.conflict_guard_implementation must name the Rust implementation",
        failures,
    );
    require_string(
        github,
        "conflict_guard_fetch_remote_test",
        "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.rs",
        "github.conflict_guard_fetch_remote_test must name the Rust fetch-remote regression",
        failures,
    );
    require_string(
        github,
        "required_context_rollup_check",
        "scripts/ci/assert-pr-required-context.rs",
        "github.required_context_rollup_check must name the non-mutating rollup checker",
        failures,
    );
    require_string(
        github,
        "required_context_rollup_test",
        "scripts/tests/phase0_required_context_rollup_check.rs",
        "github.required_context_rollup_test must name the rollup fixture test",
        failures,
    );
    require_string(
        github,
        "trusted_required_context_producer",
        "github-lane-unlocker-ci-cd",
        "github.trusted_required_context_producer must be github-lane-unlocker-ci-cd",
        failures,
    );
    require_bool_true(
        github,
        "script_rejects_missing_required_context_producer",
        "github.script_rejects_missing_required_context_producer must be true",
        failures,
    );
    require_bool_true(
        github,
        "script_rejects_untrusted_required_context_producer",
        "github.script_rejects_untrusted_required_context_producer must be true",
        failures,
    );
    require_string(
        github,
        "live_no_checks_reported_failure_reason",
        "no_status_checks_reported",
        "github.live_no_checks_reported_failure_reason must be no_status_checks_reported",
        failures,
    );
    require_bool_true(
        github,
        "script_detects_missing_live_required_context",
        "github.script_detects_missing_live_required_context must be true",
        failures,
    );
    for fixture in [
        "specs/fixtures/phase0-required-context-rollup/good-nested-github-lane-unlocker-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-missing-producer.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-untrusted-producer.json",
    ] {
        require_array_contains(
            github,
            "required_context_rollup_fixtures",
            fixture,
            &format!("github.required_context_rollup_fixtures missing {fixture}"),
            failures,
        );
    }

    require_string(
        forgejo,
        "schedule_field",
        "merge_when_checks_succeed",
        "forgejo.schedule_field must be merge_when_checks_succeed",
        failures,
    );
    require_string(
        forgejo,
        "head_pin_field",
        "head_commit_id",
        "forgejo.head_pin_field must be head_commit_id",
        failures,
    );
    require_bool_true(
        forgejo,
        "script_requires_mergeability_guard",
        "forgejo.script_requires_mergeability_guard must be true",
        failures,
    );
    require_string_array_eq(
        forgejo,
        "allowed_merge_methods",
        &["squash"],
        "forgejo.allowed_merge_methods must be ['squash']",
        failures,
    );
    require_bool_true(
        forgejo,
        "delete_branch_after_merge_locked",
        "forgejo.delete_branch_after_merge_locked must be true",
        failures,
    );
    require_bool_true(
        forgejo,
        "tide_required_context_hard_pinned",
        "forgejo.tide_required_context_hard_pinned must be true",
        failures,
    );
    require_string(
        forgejo,
        "tide_merge_method_hard_pinned",
        "squash",
        "forgejo.tide_merge_method_hard_pinned must be squash",
        failures,
    );
    require_bool_true(
        forgejo,
        "tide_head_pin_full_sha_guard",
        "forgejo.tide_head_pin_full_sha_guard must be true",
        failures,
    );

    let Some(buck2_enforcement) = object_field(spec, "buck2_enforcement") else {
        failures.push("spec.buck2_enforcement must be an object".to_string());
        return;
    };
    require_string(
        buck2_enforcement,
        "github_bootstrap_test",
        "//:github-auto-merge-after-ci-check",
        "buck2_enforcement.github_bootstrap_test must be //:github-auto-merge-after-ci-check",
        failures,
    );
    require_string(
        buck2_enforcement,
        "github_required_context_rollup_test",
        "//:github-auto-merge-after-ci-check",
        "buck2_enforcement.github_required_context_rollup_test must be //:github-auto-merge-after-ci-check",
        failures,
    );
}

fn validate_text_surfaces(root: &Path, failures: &mut Vec<String>) {
    let trigger = read(root, "scripts/trigger-next-queue-automerge.sh");
    for needle in [
        "--auto --match-head-commit",
        "scripts/check-sequential-pr-merge-conflicts.sh",
        "live branch-protection required contexts drift",
        "--merge-method is fixed to squash",
        "--fetch-remote",
        "remote_url_contains_github \"github-mirror\"",
        "merge_flag=\"--squash\"",
        "gh pr merge \"$number\" \"$merge_flag\" --auto --match-head-commit \"$head_oid\"",
    ] {
        require_contains(&trigger, needle, "github trigger", failures);
    }

    let conflict_guard_shim = read(root, "scripts/check-sequential-pr-merge-conflicts.sh");
    for needle in [
        "Compatibility entrypoint only",
        "scripts/check-sequential-pr-merge-conflicts.rs",
        "rustc --edition=2021 -D warnings",
    ] {
        require_contains(
            &conflict_guard_shim,
            needle,
            "conflict guard compatibility shim",
            failures,
        );
    }

    let conflict_guard = read(root, "scripts/check-sequential-pr-merge-conflicts.rs");
    for needle in [
        "--fetch-remote <remote>",
        "git merge-tree",
        "fn run_git_fetch",
        "\"fetch\"",
        "pass --fetch-remote for the GitHub mirror when origin is Forgejo",
    ] {
        require_contains(&conflict_guard, needle, "conflict guard Rust", failures);
    }

    let forge_script = read(root, "scripts/ci/arm-auto-merge.sh");
    for needle in [
        "REQUIRED_CONTEXT=\"oya-ci-required\"",
        "REQUIRED_CONTEXT is fixed to oya-ci-required",
        "--merge-method is fixed to squash",
        "--delete-branch-after-merge is fixed to true",
        "merge_when_checks_succeed",
        "head_commit_id",
        "delete_branch_after_merge",
        "--head-commit is required with --pr-index",
        "--head-commit must be a full SHA-1 (40 hex) or SHA-256 (64 hex) commit id",
        "pulls/${PR_INDEX}/merge",
        "validate_pr_ready_for_auto_merge",
        "pr_resp=\"$(forge_api GET \"${PR_ITEM_ENDPOINT}\")\"",
        "head = d.get(\"head\") or {}",
        "head_sha = head.get(\"sha\") or \"\"",
        "mergeable = d.get(\"mergeable\", None)",
        "does not match expected",
        "PR is not mergeable according to Forgejo",
    ] {
        require_contains(&forge_script, needle, "forgejo script", failures);
    }

    let tide_adapter = read(
        root,
        "oya/ci-tide/crates/oya-ci-tide-forgejo-adapter/src/lib.rs",
    );
    for needle in [
        "merge_when_checks_succeed: true",
        "delete_branch_after_merge: true",
        "head_commit_id: head_sha.to_owned()",
        "P0.0 Tide auto-merge scheduling is squash-only",
        "is_full_hex_commit_id(head_sha)",
        "head_sha must be a full SHA-1 (40 hex) or SHA-256 (64 hex) commit id",
    ] {
        require_contains(&tide_adapter, needle, "tide adapter", failures);
    }

    let tide_kernel = read(root, "oya/ci-tide/crates/oya-ci-tide-kernel/src/lib.rs");
    for needle in [
        "let required_status_context = DEFAULT_REQUIRED_STATUS_CONTEXT.to_owned();",
        "let merge_method = MergeMethod::Squash;",
        "configured_required_status_context_cannot_override_phase0_default",
        "configured_merge_method_cannot_override_phase0_squash_default",
        "assert_eq!(MergeMethod::from_str(\"merge\"), MergeMethod::Squash);",
    ] {
        require_contains(&tide_kernel, needle, "tide kernel", failures);
    }

    let tide_app = read(root, "oya/ci-tide/crates/oya-ci-tide-app/src/lib.rs");
    require_contains(&tide_app, "&fresh_pr.head_sha", "tide app", failures);

    let github_test = read(
        root,
        "scripts/tests/trigger_next_queue_automerge_required_contexts.rs",
    );
    require_contains(
        &github_test,
        "--merge-method is fixed to squash",
        "github trigger test",
        failures,
    );

    let rollup_check = read(root, "scripts/ci/assert-pr-required-context.rs");
    for needle in [
        "no_status_checks_reported",
        "missing_required_context",
        "required_context_not_success",
        "github-lane-unlocker-required",
        "github-lane-unlocker-ci-cd",
        "missing_required_context_producer",
        "untrusted_required_context_producer",
        "required_context_trusted_producer",
        "status-rollup evidence only; this checker never posts statuses",
    ] {
        require_contains(
            &rollup_check,
            needle,
            "required context rollup check",
            failures,
        );
    }

    let rollup_test = read(
        root,
        "scripts/tests/phase0_required_context_rollup_check.rs",
    );
    for needle in [
        "bad-no-checks-reported.json",
        "no_status_checks_reported",
        "required_context_not_success",
        "good-nested-github-lane-unlocker-required-success.json",
        "missing_required_context_producer",
        "untrusted_required_context_producer",
    ] {
        require_contains(
            &rollup_test,
            needle,
            "required context rollup test",
            failures,
        );
    }

    let github_conflict_test = read(
        root,
        "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh",
    );
    for needle in [
        "sequential PR merge simulation passed: 1 PRs modeled",
        "dry-run: gh pr merge 455 --squash --auto --match-head-commit",
        "Local sequencing regression guard",
        "clean_real_work=",
        "guard_marker=guard passed:",
        "pr merge 455 --squash --auto --match-head-commit",
        "::error::sequential merge conflict at PR #455",
        "conflict scenario invoked gh pr merge",
    ] {
        require_contains(
            &github_conflict_test,
            needle,
            "github conflict guard test",
            failures,
        );
    }

    let conflict_test = read(
        root,
        "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.rs",
    );
    for needle in [
        "--fetch-remote",
        "github-mirror",
        "failed to fetch PR #455 head from remote origin",
        "default origin fetch should fail when origin is non-GitHub Forgejo remote",
    ] {
        require_contains(&conflict_test, needle, "conflict guard test", failures);
    }

    for path in ["docs/ci/auto-merge-flow.md", "docs/ci/forge-of-record.md"] {
        let text = read(root, path);
        for needle in [
            "github-lane-unlocker-required",
            "oya-ci-required",
            "--match-head-commit",
            "head_commit_id",
        ] {
            require_contains(&text, needle, path, failures);
        }
        if text.contains("oya-ci-gate") {
            failures.push(format!("{path}: must not reference stale oya-ci-gate"));
        }
    }
}

fn validate_policy(root: &Path, failures: &mut Vec<String>) {
    let policy = load_json(root, "specs/buck2-authority-policy.json");
    let policy = object(&policy, "buck2 policy", failures);
    let command_scan_files = string_array_field(policy, "command_scan_files");
    for required in [
        "scripts/ci/arm-auto-merge.sh",
        "scripts/trigger-next-queue-automerge.sh",
        "scripts/check-sequential-pr-merge-conflicts.sh",
        "scripts/check-sequential-pr-merge-conflicts.rs",
        "scripts/tests/forgejo_auto_merge_after_ci.test.sh",
        "scripts/tests/trigger_next_queue_automerge_required_contexts.rs",
        "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh",
        "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.rs",
        "scripts/tests/phase0_required_context_rollup_check.rs",
        "scripts/ci/assert-pr-required-context.rs",
        "scripts/tests/phase0_auto_merge_after_ci_contract_check.rs",
        "docs/ci/auto-merge-flow.md",
        "docs/ci/forge-of-record.md",
        "specs/phase0-auto-merge-after-ci.json",
        "oya/ci-tide/crates/oya-ci-tide-kernel/src/lib.rs",
        "oya/ci-tide/crates/oya-ci-tide-app/src/lib.rs",
        "oya/ci-tide/crates/oya-ci-tide-forgejo-adapter/src/lib.rs",
    ] {
        if !command_scan_files.iter().any(|item| item == required) {
            failures.push(format!(
                "buck2 policy command_scan_files missing {required}"
            ));
        }
    }
}

pub fn evaluate(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let spec = load_json(root, SPEC);
    let spec = object(&spec, "phase0 auto-merge spec", &mut failures);
    validate_spec(spec, &mut failures);
    validate_text_surfaces(root, &mut failures);
    validate_policy(root, &mut failures);
    failures
}

fn failures_json(failures: &[String]) -> String {
    format!(
        "[{}]",
        failures
            .iter()
            .map(|failure| json_string(failure))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn pass_json() -> String {
    concat!(
        "{\"checks\":{",
        "\"buck2_policy_scan_covered\":true,",
        "\"conflict_guard_declared\":true,",
        "\"conflict_guard_rust_implementation_declared\":true,",
        "\"conflict_guard_rust_fetch_remote_tested\":true,",
        "\"forgejo_auto_merge_after_ci_head_pinned\":true,",
        "\"forgejo_mergeability_guard_declared\":true,",
        "\"github_auto_merge_head_pinned\":true,",
        "\"p0_0_green\":false,",
        "\"phase0_complete\":false,",
        "\"required_context_rollup_check_tested\":true,",
        "\"tide_context_hard_pinned\":true,",
        "\"tide_full_sha_guard_declared\":true,",
        "\"tide_squash_only\":true,",
        "\"trigger_conflict_guard_tested\":true,",
        "\"trigger_non_dry_run_merge_path_scope_labeled\":true,",
        "\"trigger_non_dry_run_merge_path_tested\":true",
        "},",
        "\"required_context\":\"github-lane-unlocker-required\",",
        "\"spec\":\"specs/phase0-auto-merge-after-ci.json\",",
        "\"verdict\":\"PASS\"}"
    )
    .to_string()
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn main() {
    let root = repo_root();
    let failures = evaluate(&root);
    if failures.is_empty() {
        println!("{}", pass_json());
    } else {
        eprintln!("phase0-auto-merge-after-ci-contract: RED");
        for failure in &failures {
            eprintln!("- {failure}");
        }
        eprintln!(
            "{{\"failures\":{},\"required_context\":\"github-lane-unlocker-required\",\"spec\":\"{}\",\"verdict\":\"FAIL\"}}",
            failures_json(&failures),
            SPEC
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_contract_passes() {
        let failures = evaluate(&repo_root());
        assert!(failures.is_empty(), "{failures:?}");
        assert!(pass_json().contains("\"required_context\":\"github-lane-unlocker-required\""));
        assert!(pass_json().contains("\"p0_0_green\":false"));
        assert!(pass_json().contains("\"phase0_complete\":false"));
    }

    #[test]
    fn require_contains_records_missing_marker() {
        let mut failures = Vec::new();
        require_contains("abc", "missing", "sample", &mut failures);
        assert_eq!(failures, vec!["sample: missing \"missing\""]);
    }
}
