#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Rename detection must be a property of the TOOL, not of the machine it runs on.
//!
//! A capability move (ADR-0562 strangler, one move per PR) reaches the affected-set gate as a
//! rename. If git does not report it as one, the destination path arrives as an unowned `A`
//! (`Change::Present`) and [`resolve`] returns `RefuseUnowned` — which it does BEFORE reading
//! `full_reasons`, by design, so the paired `D` never gets to escalate the run to FULL. The move PR
//! then wedges with no in-band exit.
//!
//! Whether git reports a rename used to depend entirely on ambient config the PR author cannot see
//! (`diff.renames`, `diff.renameLimit`). These fixtures force the hostile config and prove the
//! explicit flags in [`merge_base_diff_args`] make the verdict independent of it.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use ci_affected_target_set::{
    Change, Decision, Policy, StructuralKind, merge_base_diff_args, parse_name_status_z,
    plan_changes, resolve,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// The argv as it stood before the fix: rename detection left to ambient config. Kept ONLY as the
/// negative control — a fixture that proves the new flags are load-bearing rather than decorative.
fn legacy_diff_args<'a>(merge_base: &'a str, head: &'a str) -> Vec<&'a str> {
    vec!["diff", "--name-status", "-z", merge_base, head]
}

fn fixture_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-affected-rename-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("git stdout utf8")
}

/// A repo whose HEAD renames one Rust source, with rename detection DISABLED in local config —
/// the exact environment in which the old argv silently degrades.
///
/// `diff.renames=false` is the reachable, reproducible half of the risk. The other half
/// (`diff.renameLimit` exceeded on a large move, where git degrades and only warns) is guarded by
/// the same explicit-flag fix via `-l0`, but needs a diff of thousands of paths to reproduce and is
/// not modelled here.
fn repo_with_hostile_rename_config() -> (PathBuf, String, String) {
    let root = fixture_root();
    git(&root, &["init"]);
    git(&root, &["config", "user.name", "Oyatie Test"]);
    git(&root, &["config", "user.email", "oyatie-test@example.com"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    // The whole point of the fixture.
    git(&root, &["config", "diff.renames", "false"]);

    std::fs::create_dir_all(root.join("legacy/core")).expect("create source dir");
    // Body must be long enough that git's similarity index would call it a rename if it were
    // looking — otherwise the control would pass for the wrong reason.
    let body: String = (0..40)
        .map(|i| format!("pub fn generated_symbol_{i}() -> usize {{ {i} }}\n"))
        .collect();
    std::fs::write(root.join("legacy/core/lib.rs"), &body).expect("write source");
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "base"]);
    let base = git(&root, &["rev-parse", "HEAD"]).trim().to_owned();

    std::fs::create_dir_all(root.join("capability/core")).expect("create dest dir");
    git(
        &root,
        &["mv", "legacy/core/lib.rs", "capability/core/lib.rs"],
    );
    git(&root, &["commit", "-m", "move capability"]);
    let head = git(&root, &["rev-parse", "HEAD"]).trim().to_owned();

    (root, base, head)
}

fn diff(root: &Path, args: Vec<&str>) -> Vec<Change> {
    let raw = git(root, &args);
    parse_name_status_z(&raw).expect("parse diff")
}

/// The one path in the fixture is owner-required and has NO buck2 owner — exactly a moved source
/// whose destination target does not exist yet at HEAD. `full_trigger_patterns` carries a single
/// entry that matches nothing in the fixture: `Policy` rejects an empty list, and an escape trigger
/// firing here would reach FULL for the wrong reason and hide the verdict under test.
fn policy() -> Policy {
    Policy::from_json(
        r#"{
            "gate_id": "cloud-ci-affected-set",
            "universe": "//...",
            "full_run_targets": ["//..."],
            "full_trigger_patterns": ["rust-toolchain.toml"],
            "require_owner_patterns": ["**/*.rs"],
            "package_definition_basenames": ["BUCK.v2", "BUCK"],
            "package_sibling_basenames": ["Cargo.toml", "build.rs"],
            "cell_roots": {"": "//"},
            "synthetic_dependencies": {},
            "inert_selection_classes": [],
            "default_base_ref": "origin/dev"
        }"#,
    )
    .expect("policy")
}

/// Phase A+B with every queried path unowned — the destination target does not exist at HEAD.
fn verdict(changes: &[Change]) -> Decision {
    let policy = policy();
    let plan = plan_changes(changes, &policy);
    let owners = plan
        .owner_paths
        .iter()
        .map(|p| (p.clone(), Vec::new()))
        .collect();
    resolve(&plan, &owners, &policy)
}

#[test]
fn capability_move_is_a_rename_even_when_ambient_config_disables_detection() {
    let (root, base, head) = repo_with_hostile_rename_config();
    let changes = diff(&root, merge_base_diff_args(&base, &head));

    assert_eq!(
        changes,
        vec![Change::Structural {
            path: "capability/core/lib.rs".into(),
            kind: StructuralKind::Rename,
        }],
        "explicit --find-renames must override diff.renames=false"
    );
}

/// NEGATIVE CONTROL. Without the flags, the same move under the same config is `A`+`D`. This is
/// what makes the assertion above meaningful rather than a restatement of git's defaults: if this
/// fixture ever starts agreeing with the one above, the flags have stopped being load-bearing and
/// the test above is no longer proving anything.
#[test]
fn without_the_explicit_flags_the_same_move_degrades_to_add_plus_delete() {
    let (root, base, head) = repo_with_hostile_rename_config();
    let changes = diff(&root, legacy_diff_args(&base, &head));

    assert_eq!(
        changes,
        vec![
            Change::Present("capability/core/lib.rs".into()),
            Change::Deleted("legacy/core/lib.rs".into()),
        ],
        "control: ambient diff.renames=false must degrade the move to add+delete"
    );
}

/// The consequence the flags actually buy. `RefuseUnowned` dominates `full_reasons` in [`resolve`]
/// by design, so the `D` half of a degraded rename CANNOT rescue the verdict — the move PR wedges.
#[test]
fn degraded_rename_wedges_the_move_while_a_detected_rename_escalates_to_full() {
    let (root, base, head) = repo_with_hostile_rename_config();

    match verdict(&diff(&root, legacy_diff_args(&base, &head))) {
        Decision::RefuseUnowned { paths } => assert_eq!(paths, vec!["capability/core/lib.rs"]),
        other => panic!("control must wedge on the unowned destination, got {other:?}"),
    }

    match verdict(&diff(&root, merge_base_diff_args(&base, &head))) {
        Decision::Full { reasons } => assert!(
            reasons.iter().any(|r| r.contains("rename")),
            "FULL must be justified by the rename, got {reasons:?}"
        ),
        other => panic!("a detected rename must escalate to FULL, got {other:?}"),
    }
}
