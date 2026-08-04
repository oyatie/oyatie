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

use ci_repo_root_hygiene::{
    Verdict, evaluate, evaluate_keyed, evaluate_talos_machine_config_documents,
};
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

#[test]
fn generated_talos_root_outputs_are_ignored_by_exact_root_anchored_rules() {
    let root = repo_root();
    let gitignore = fs::read_to_string(root.join(".gitignore")).expect("read root .gitignore");
    for exact_rule in [
        "/controlplane.yaml",
        "/worker.yaml",
        "/talosconfig",
        "/secrets.yaml",
    ] {
        assert!(
            gitignore.lines().any(|line| line == exact_rule),
            "missing exact root-anchored Talos generated-output ignore rule: {exact_rule}"
        );
    }
}

#[test]
fn live_tracked_yaml_corpus_has_no_generated_talos_machine_config() {
    let root = repo_root();
    let observed = observed_from_scm_facts(&root);
    let mut documents = Vec::new();
    for row in observed["rows"].as_array().expect("rows") {
        let Some(path) = row["path"].as_str() else {
            continue;
        };
        if !(path.ends_with(".yaml") || path.ends_with(".yml")) {
            continue;
        }
        let text = fs::read_to_string(root.join(path))
            .unwrap_or_else(|error| panic!("read tracked YAML path {path}: {error}"));
        documents.push((path.to_owned(), text));
    }
    let borrowed = documents
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect::<Vec<_>>();

    let findings = evaluate_talos_machine_config_documents(borrowed);
    assert!(
        findings.is_empty(),
        "tracked YAML must not contain generated Talos machine-config credential topology; findings are value-redacted: {findings:#?}"
    );
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

/// The materialized tracked-path face this gate reads. ADR-0604 de-commit class: NOT tracked in
/// git, so in ANY clean worktree it is simply absent.
const SCM_FACTS_REL: &str = "ci/facade/artifact-inventory-registry/scm-facts.generated.json";

/// The exact command that materializes it locally — the same binary the CI "Materialize cloud-ci
/// generated faces" step runs, minus `--github-event` (which only reads `GITHUB_EVENT_PATH`).
const MATERIALIZE_CMD: &str = "buck2 run \
     //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin \
     -- --repo-root .";

/// Resolve the scm-facts face, or fail with an ACTIONABLE message.
///
/// Before this, a clean worktree got `read <abs path>: No such file or directory (os error 2)`.
/// That is a true statement and useless advice: the face is generated, its producer is not
/// discoverable from the path, and an author checking their change against this gate had no way
/// to know the gate was not evaluating anything. A gate that cannot be run locally gives no local
/// signal, which is how a change reaches CI unchecked.
fn require_scm_facts(root: &Path) -> PathBuf {
    let path = root.join(SCM_FACTS_REL);
    assert!(
        path.is_file(),
        "{SCM_FACTS_REL} is missing — this gate reads the materialized tracked-path face, which \
         is generated (ADR-0604 de-commit class) and therefore absent in a clean worktree.\n\
         \n\
         Materialize it, then re-run this gate:\n\
         \n    {MATERIALIZE_CMD}\n\
         \n\
         In CI this is the \"Materialize cloud-ci generated faces\" step; the faces are then \
         uploaded as the `generated-faces` artifact and downloaded by every gate leg."
    );
    path
}

/// Build the `{ "rows": [{"path": ...}] }` observed inventory from the materialized scm-facts face.
fn observed_from_scm_facts(root: &Path) -> Value {
    let scm = load_json(&require_scm_facts(root));
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
