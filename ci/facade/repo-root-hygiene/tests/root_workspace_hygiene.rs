// Born-blocking self-test over TODAY's real tracked-path corpus (ADR-0600).
//
// It loads the committed allowlist policy + the producer's git-ls-files snapshot
// (scm-facts.generated.json, a declared hermetic input that lists every tracked path), then:
//   1. asserts the gate is GREEN on the live, clean, allowlisted root tree (the make-impossible
//      guarantee holds today);
//   2. proves a synthetic tracked `foo.log` injected at the repo ROOT is born-blocking RED
//      (RED/GREEN evidence — the gate is non-inert).
//
// HERMETIC: the test reads the MATERIALIZED scm-facts face from the source tree (no git, no
// network). scm-facts is the ADR-0604 de-commit class (NOT tracked in git): the CI producer-regen
// job materializes it and every gate matrix leg downloads it before `cargo test`, so the repo-root
// walk reaches it on disk. ADR-0083 Tier-3: integration tests use unwrap/expect/panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use ci_repo_root_hygiene::{Verdict, evaluate, evaluate_keyed};
use serde_json::{Value, json};

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Mirrors the helper used by the sibling gate tests.
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn gate_dir(root: &Path) -> PathBuf {
    root.join("ci/facade/repo-root-hygiene")
}

fn load_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn load_policy(root: &Path) -> Value {
    load_json(&gate_dir(root).join("root-workspace-hygiene-policy.json"))
}

const SYNTHETIC_RUNTIME_STATE_PATHS: [&str; 5] = [
    ".claude/worktrees/old-lane/marker",
    ".claude/settings.local.json",
    ".codex/.DS_Store",
    ".omc/state/team/mailbox.json",
    ".omx/state/team/mailbox.json",
];

/// Decode git's C-style path quoting: git surrounds a path containing special bytes with double
/// quotes and octal/`\`-escapes the inner bytes. The scm-facts snapshot carries those quoted forms
/// verbatim. For the purpose of TOP-LEVEL-segment + ROOT-file classification we only need the
/// quotes stripped (so `"oya/…µservice….md"` classifies under the real `oya` dir, not `"oya`).
/// We strip a surrounding pair of double-quotes if present; the inner escapes do not affect the
/// first path segment for any real corpus path.
fn unquote_git_path(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// Build the `{ "rows": [{"path": ...}] }` observed inventory from the committed scm-facts snapshot.
fn observed_from_scm_facts(root: &Path) -> Value {
    let scm =
        load_json(&root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json"));
    let paths = scm["tracked_paths"]
        .as_array()
        .expect("scm-facts.generated.json must carry a tracked_paths array");
    let rows: Vec<Value> = paths
        .iter()
        .filter_map(Value::as_str)
        .map(|p| json!({ "path": unquote_git_path(p) }))
        .collect();
    json!({ "rows": rows })
}

#[test]
fn live_tracked_root_tree_is_allowlist_clean_green() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed = observed_from_scm_facts(&root);

    let rows = observed["rows"].as_array().expect("rows");
    assert!(
        rows.len() > 1000,
        "expected the full tracked-path corpus, got {} rows",
        rows.len()
    );

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "root-workspace-hygiene gate found violations over the live tracked tree — the allowlist \
         either forbids a legitimate root surface or a scratch file is still tracked:\n{findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
}

#[test]
fn retired_consumers_are_absent_from_active_configuration() {
    let root = repo_root();
    for path in [
        ".gitattributes",
        "specs/capability-registry.json",
        "docs/oya-ci/gate-catalog.md",
        ".github/workflows/oya-ci-required.yml",
    ] {
        let contents = fs::read_to_string(root.join(path))
            .unwrap_or_else(|error| panic!("read {path}: {error}"));
        assert!(
            !contents.contains("oya-friction-ledger-merge-driver-app")
                && !contents.contains("cloud-ci-friction-accounting")
                && !contents.contains("action-item-accounting"),
            "retired consumer remains in {path}"
        );
    }
}

/// The live tracked agent/runtime surface must contain only explicit exceptions. Local worktrees,
/// tmux/team state, caches, and settings.local files must stay ignored and untracked.
#[test]
fn live_tracked_runtime_state_dirs_are_explicitly_allowlisted_green() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed = observed_from_scm_facts(&root);

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        !findings
            .iter()
            .any(|f| f.code == "root_workspace_restricted_dir_unallowlisted_path"),
        "live tracked runtime/provenance paths must all be explicit exceptions; got {findings:#?}"
    );
}

/// RED FIXTURE (mandatory, proves non-inert): injecting a tracked `foo.log` at the repo ROOT must
/// make the gate RED with the offending key surfaced under the unallowlisted-file code — this IS
/// the "committed root scratch is structurally impossible" guarantee.
#[test]
fn synthetic_tracked_root_log_is_born_blocking_red() {
    let root = repo_root();
    let policy = load_policy(&root);
    let mut observed = observed_from_scm_facts(&root);

    observed["rows"]
        .as_array_mut()
        .expect("rows array")
        .push(json!({ "path": "foo.log" }));

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == "foo.log"),
        "a tracked root `foo.log` must be born-blocking with its key surfaced; got {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

/// RED FIXTURE: local agent runtime state under `.claude/.codex/.omc/.omx` must be born-blocking
/// if it is ever forced into the tracked-path corpus. This prevents the four agent-state trees from
/// drifting into committed merge-conflict surfaces.
#[test]
fn synthetic_tracked_runtime_state_paths_are_born_blocking_red() {
    let root = repo_root();
    let policy = load_policy(&root);
    let mut observed = observed_from_scm_facts(&root);

    for path in SYNTHETIC_RUNTIME_STATE_PATHS {
        observed["rows"]
            .as_array_mut()
            .expect("rows array")
            .push(json!({ "path": path }));
    }

    let findings = evaluate_keyed(&policy, &observed);
    for path in SYNTHETIC_RUNTIME_STATE_PATHS {
        assert!(
            findings.iter().any(|f| {
                f.code == "root_workspace_restricted_dir_unallowlisted_path" && f.key == path
            }),
            "{path} must be born-blocking under restricted runtime/state roots; got {findings:#?}"
        );
    }
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

/// The exact root scratch this PR removes must each fail the live allowlist (regression guard:
/// the gate would have caught the original pollution).
#[test]
fn the_removed_root_scratch_shapes_are_red_against_the_live_policy() {
    let root = repo_root();
    let policy = load_policy(&root);
    for scratch in [
        "backfill-targets.txt",
        "branch-wired-members.txt",
        "final-targets.txt",
        "slice06-progress.log",
        "retest-targets.txt",
        "run-slice.sh",
        "premise.txt",
        "review-verdict.txt",
    ] {
        let observed = json!({ "rows": [{ "path": scratch }] });
        let findings = evaluate_keyed(&policy, &observed);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == scratch),
            "{scratch} must be born-blocking against the live allowlist policy"
        );
    }
}

/// RED FIXTURE: the retired direnv + bin/oya CLI shim must not return as tracked roots.
#[test]
fn retired_dev_env_cli_surfaces_are_born_blocking_red() {
    let root = repo_root();
    let policy = load_policy(&root);

    let root_file_findings = evaluate_keyed(&policy, &json!({ "rows": [{ "path": ".envrc" }] }));
    assert!(
        root_file_findings
            .iter()
            .any(|f| f.code == "root_workspace_unallowlisted_file" && f.key == ".envrc"),
        ".envrc must stay retired from the tracked repo root; got {root_file_findings:#?}"
    );

    let bin_dir_findings = evaluate_keyed(&policy, &json!({ "rows": [{ "path": "bin/oya" }] }));
    assert!(
        bin_dir_findings
            .iter()
            .any(|f| f.code == "root_workspace_unallowlisted_dir" && f.key == "bin"),
        "bin/oya must stay retired because top-level bin/ is no longer allowlisted; got {bin_dir_findings:#?}"
    );
}

/// Every offending finding must carry a concrete auto-fix remediation (relocate to `.omc/` or
/// `git rm`) — the gate is auto-fixing, not flag-only.
#[test]
fn live_policy_findings_carry_concrete_remediation() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed = json!({ "rows": [{ "path": "foo.log" }] });
    let findings = evaluate_keyed(&policy, &observed);
    let f = findings
        .iter()
        .find(|f| f.key == "foo.log")
        .expect("finding for foo.log");
    assert!(
        f.detail.contains("git rm") && f.detail.contains(".omc/"),
        "remediation must name the concrete auto-fix; got: {}",
        f.detail
    );
}
