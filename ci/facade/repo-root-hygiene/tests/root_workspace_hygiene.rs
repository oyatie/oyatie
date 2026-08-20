// Born-blocking self-test over TODAY's real tracked-path corpus (ADR-0600).
//
// It loads the committed allowlist policy + the producer's git-ls-files snapshot
// (scm-facts.generated.json, a declared hermetic input that lists every tracked path), then:
//   1. asserts the gate is GREEN on the live, clean, allowlisted root tree (the make-impossible
//      guarantee holds today);
//   2. proves a synthetic tracked `foo.log` injected at the repo ROOT is born-blocking RED
//      (RED/GREEN evidence — the gate is non-inert).
//
// HERMETIC: the test reads the workflow-materialized scm-facts face when declared. In a direct
// Cargo workspace test it invokes the exact Cargo-built Rust emitter into temporary storage (no
// tracked generated face, network, or ambient binary). ADR-0083 Tier-3: integration tests use
// unwrap/expect/panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use ci_repo_root_hygiene::{
    Verdict, corpus_class_counts, corpus_class_raise_matches_reviewed_record, evaluate,
    evaluate_keyed, evaluate_talos_machine_config_documents,
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

/// Exact pre-existing public parser/differential fixtures that intentionally model Talos
/// credential topology with deterministic test-only values. This is a frozen exception, not a
/// directory exemption: a renamed, extensionless, or newly added sibling remains scanned. The two
/// invalid fixtures without sensitive topology (10/11) are not exceptions because they produce no
/// finding.
fn is_frozen_public_talos_parser_fixture(path: &str) -> bool {
    if path == "os/core/init-app/testdata/machine-config.yaml" {
        return true;
    }
    let fixture_name = [
        "os/core/machine-config-domain/testdata/configs/",
        "os/harness/difftest-app/configs/",
    ]
    .into_iter()
    .find_map(|prefix| path.strip_prefix(prefix));
    matches!(
        fixture_name,
        Some(
            "01-controlplane-full.yaml"
                | "02-controlplane-no-hostname-no-install.yaml"
                | "03-controlplane-hostname-no-install.yaml"
                | "04-worker-full.yaml"
                | "05-worker-no-hostname-no-install.yaml"
                | "06-worker-hostname-no-install.yaml"
                | "07-worker-install-no-hostname.yaml"
                | "08-controlplane-certsans.yaml"
                | "09-invalid-bad-type.yaml"
                | "12-invalid-no-endpoint.yaml"
                | "13-invalid-worker-ca-key.yaml"
                | "14-init-full.yaml"
                | "15-controlplane-rich-network.yaml"
                | "16-worker-rich.yaml"
                | "17-controlplane-install-image-only.yaml"
                | "18-worker-sysctls-env-only.yaml"
                | "19-init-cluster-network.yaml"
                | "20-worker-minimal-kubelet.yaml"
        )
    )
}

#[test]
fn bounded_tracked_utf8_corpus_has_no_generated_talos_machine_config_regardless_of_filename() {
    let root = repo_root();
    let observed = observed_from_scm_facts(&root);
    let mut documents = Vec::new();
    for row in observed["rows"].as_array().expect("rows") {
        let Some(path) = row["path"].as_str() else {
            continue;
        };
        let Ok(metadata) = fs::metadata(root.join(path)) else {
            continue;
        };
        if metadata.len() > 2 * 1024 * 1024 {
            continue;
        }
        let Ok(bytes) = fs::read(root.join(path)) else {
            continue;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        documents.push((path.to_owned(), text));
    }
    let borrowed = documents
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect::<Vec<_>>();

    let findings = evaluate_talos_machine_config_documents(borrowed);
    let unexpected = findings
        .iter()
        .filter(|finding| !is_frozen_public_talos_parser_fixture(&finding.key))
        .collect::<Vec<_>>();
    let frozen_count = findings.len() - unexpected.len();
    assert_eq!(
        frozen_count, 37,
        "the exact pre-existing public Talos fixture exception set must neither grow nor silently disappear"
    );
    assert!(
        unexpected.is_empty(),
        "outside the exact frozen public fixture set, bounded tracked UTF-8 blobs must not contain generated Talos machine-config credential topology regardless of filename; findings are value-redacted: {unexpected:#?}"
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
const CARGO_TEST_SCM_FACTS_EMITTER_ENV: &str = "OYA_CI_CARGO_TEST_SCM_FACTS_EMITTER_BIN";
static CARGO_SCM_FACTS: OnceLock<Value> = OnceLock::new();

/// Build the `{ "rows": [{"path": ...}] }` observed inventory from the materialized scm-facts face.
fn observed_from_scm_facts(root: &Path) -> Value {
    let scm = load_scm_facts(root);
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

/// Read the workflow-declared face when present. A direct Cargo workspace test has no workflow
/// pre-step, so it invokes the exact Cargo-built Rust emitter into temporary storage and parses
/// that equivalent input. The generated face is never written into the checkout.
fn load_scm_facts(root: &Path) -> Value {
    let declared = root.join(SCM_FACTS_REL);
    if declared.is_file() {
        // The face is gitignored, so a stale one is invisible to `git status` and would
        // silently supply the tracked-path set from before the last checkout or merge.
        if let Err(reason) = ci_path_resolver_ports::declared_face_is_fresh(root, &declared) {
            panic!("FAIL-CLOSED: {reason}");
        }
        return load_json(&declared);
    }
    CARGO_SCM_FACTS
        .get_or_init(|| {
            materialize_temporary_scm_facts(root)
                .unwrap_or_else(|error| panic!("FAIL-CLOSED: {error}"))
        })
        .clone()
}

fn materialize_temporary_scm_facts(root: &Path) -> Result<Value, String> {
    // The Cargo config binding is the out-of-graph capability signal. Buck does not expose it,
    // so a missing workflow-declared face inside a Buck action stays RED and cannot call Git.
    std::env::var_os(CARGO_TEST_SCM_FACTS_EMITTER_ENV)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing required {CARGO_TEST_SCM_FACTS_EMITTER_ENV}"))?;

    let temporary = tempfile::tempdir()
        .map_err(|error| format!("create temporary SCM facts directory: {error}"))?;
    let stable = temporary.path().join("scm-facts.generated.json");
    let volatile = temporary.path().join("scm-volatile-facts.generated.json");
    ci_scm_facts_snapshot::emit_candidate_scm_facts_out_of_graph(root, &stable, &volatile)?;
    let metadata = fs::symlink_metadata(&stable).map_err(|error| {
        format!(
            "inspect materialized SCM facts {}: {error}",
            stable.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "materialized SCM facts must be a regular non-symlink file: {}",
            stable.display()
        ));
    }
    let text = fs::read_to_string(&stable)
        .map_err(|error| format!("read {}: {error}", stable.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {}: {error}", stable.display()))
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

    let mut findings = evaluate_keyed(&policy, &observed);
    // Introduction grace (ADR-0717): while the merge-base policy has no corpus_budget block, the
    // corpus ceilings are advisory — the wave-2 cleanup PR may land after this one, so the
    // pre-cleanup tree legitimately exceeds the post-cleanup ceilings until then. Every PR after
    // the merge is bound by the ceilings (protected block present -> findings are blocking).
    let protected_policy_available = Command::new("git")
        .args([
            "show",
            "origin/dev:ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
        ])
        .current_dir(&root)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);
    let protected_has_budget = if protected_policy_available {
        let protected: serde_json::Value = serde_json::from_slice(
            &Command::new("git")
                .args([
                    "show",
                    "origin/dev:ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
                ])
                .current_dir(&root)
                .output()
                .expect("git show protected policy")
                .stdout,
        )
        .expect("parse protected policy");
        protected.get("corpus_budget").is_some()
    } else {
        false
    };
    if !protected_has_budget {
        findings.retain(|finding| !finding.code.starts_with("corpus_budget_"));
    }
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

/// ADR-0717: a reduction that lands WITHOUT lowering the frozen ceiling to the live
/// count would leave headroom for later growth back toward the original number,
/// breaking shrink-only. This live test loads the protected policy from the
/// merge-base (origin/dev) and fails when the tree has shrunk below the protected
/// ceiling while the candidate ceiling still sits above the live count (including
/// partial drops). A deliberate increase must equal the live count and carry an
/// exact, attributed candidate DATA record; that record is not external review
/// approval. Absent a protected corpus_budget block (this PR is the first to
/// introduce it), the check is a no-op.
#[test]
fn corpus_budget_reductions_must_lower_the_frozen_ceiling() {
    let root = repo_root();
    let policy = load_policy(&root);
    let Some(candidate_counts) = policy
        .get("corpus_budget")
        .and_then(|budget| budget.get("counts"))
        .and_then(serde_json::Value::as_object)
    else {
        panic!("candidate policy must carry corpus_budget.counts (fail closed)");
    };
    let protected_counts = {
        let output = Command::new("git")
            .args([
                "show",
                "origin/dev:ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
            ])
            .current_dir(&root)
            .output()
            .expect("run git show for the protected corpus budget");
        if !output.status.success() {
            // No merge-base policy (e.g. shallow/no origin ref): skip, the ceiling check itself
            // still runs through evaluate_keyed on the live tree.
            return;
        }
        let protected: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("parse protected policy");
        protected
            .get("corpus_budget")
            .and_then(|budget| budget.get("counts"))
            .and_then(serde_json::Value::as_object)
            .cloned()
    };
    let Some(protected_counts) = protected_counts else {
        return; // this PR introduces the block; nothing to compare against yet
    };

    // A ratchet must never be DELETED. The loop below walks candidate classes, so a class that
    // vanishes from the policy would otherwise be silently unenforced -- and pairing a deletion
    // with an addition is exactly how a ratchet gets laundered: re-freeze the same corpus under a
    // new key at a higher number. Removals are fatal on their own, which is what makes it safe to
    // admit newly-introduced classes below.
    let removed: Vec<&String> = protected_counts
        .keys()
        .filter(|class| !candidate_counts.contains_key(*class))
        .collect();
    assert!(
        removed.is_empty(),
        "corpus_budget.counts dropped {removed:?}; a frozen corpus class may shrink to zero but \
         must not be deleted -- deleting it retires the ratchet silently, and deleting one while \
         adding another launders it under a new key"
    );

    let observed = observed_from_scm_facts(&root);
    let observed_counts = ci_repo_root_hygiene::corpus_class_counts(&policy, &observed);
    for (class, frozen) in candidate_counts {
        let Some(protected) = protected_counts
            .get(class)
            .and_then(serde_json::Value::as_u64)
        else {
            // NEWLY INTRODUCED class. There is no prior ceiling to ratchet against, exactly as
            // when the whole block is introduced (see the early return above), so the raise and
            // reduction comparisons below have nothing to compare. The class is NOT unguarded:
            // evaluate_corpus_budget still enforces its ceiling against the live tree on this
            // very run, and from the next PR onward it has a protected value like any other.
            // Safe only because a deletion is fatal above, so this cannot be half of a rename.
            continue;
        };
        let candidate = frozen
            .as_u64()
            .expect("candidate ceiling must be an integer");
        let observed_count = observed_counts.get(class).copied().unwrap_or(0) as u64;
        if candidate > protected {
            assert!(
                corpus_class_raise_matches_reviewed_record(&policy, class, protected, candidate,),
                "corpus_budget.counts.{class} grew from {protected} to {candidate} without an exact, attributed corpus_budget.reviewed_raises.{class} DATA record"
            );
            assert_eq!(
                candidate, observed_count,
                "candidate corpus raise for {class} must equal the live count; pre-allocating growth headroom is forbidden"
            );
        }
        if ci_repo_root_hygiene::corpus_class_reduction_leaves_headroom(
            protected,
            candidate,
            observed_count,
        ) {
            panic!(
                "corpus class {class} shrank from {protected} to {observed_count} but the frozen ceiling is still {candidate}; \
                 lower corpus_budget.counts.{class} to {observed_count} in this same PR so the reduction is preserved"
            );
        }
    }
}
