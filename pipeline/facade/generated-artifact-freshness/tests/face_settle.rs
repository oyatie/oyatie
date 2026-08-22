#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use ci_generated_artifact_freshness::{
    FACE_REMEDIATION_COMMAND, FACE_SETTLE_PROTOCOL, FACE_VERIFY_REMEDIATION_COMMAND,
    FaceSettleMode, LOCK_REMEDIATION_COMMAND, assert_committed_tree_clean,
    assert_non_face_tree_clean, generated_face_paths, parse_face_settle_args,
    settle_regenerated_faces, verify_committed_tree,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "face-settle-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn git(root: &Path, args: &[&str]) {
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
}

fn git_output(root: &Path, args: &[&str]) -> String {
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

fn init_repo_with_faces() -> PathBuf {
    let root = fixture_root();
    git(&root, &["init"]);
    git(&root, &["config", "user.name", "Oyatie Test"]);
    git(&root, &["config", "user.email", "oyatie-test@example.com"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    git(
        &root,
        &["config", "user.signingkey", "missing-local-test-key"],
    );
    std::fs::write(root.join("README.md"), "content v1\n").expect("write content");
    // Minimal workspace shape so the verify mode can run the freshness gate's FULL check
    // (lock member parity) against the fixture repo.
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"libs/*\"]\nexclude = []\n\n[workspace.package]\nversion = \"0.1.0\"\n",
    )
    .expect("write workspace manifest");
    std::fs::create_dir_all(root.join("libs/fixture-kernel")).expect("create member dir");
    write_member_manifest(&root, "0.1.0");
    write_lock(&root, "0.1.0");
    for path in generated_face_paths() {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().expect("face parent")).expect("create face parent");
        std::fs::write(path, "old face\n").expect("write face");
    }
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "seed"]);
    root
}

fn write_member_manifest(root: &Path, version: &str) {
    std::fs::write(
        root.join("libs/fixture-kernel/Cargo.toml"),
        format!("[package]\nname = \"fixture-kernel\"\nversion = \"{version}\"\n"),
    )
    .expect("write member manifest");
}

fn write_lock(root: &Path, version: &str) {
    std::fs::write(
        root.join("Cargo.lock"),
        format!(
            "version = 3\n\n[[package]]\nname = \"fixture-kernel\"\nversion = \"{version}\"\n"
        ),
    )
    .expect("write lock");
}

fn regenerated_faces() -> Vec<(String, String)> {
    generated_face_paths()
        .into_iter()
        .map(|path| {
            let name = Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .expect("face name")
                .to_owned();
            (name, format!("new bytes for {path}\n"))
        })
        .collect()
}

fn mark_gate_baseline_controller_owned(root: &Path) {
    std::fs::create_dir_all(root.join("registry")).expect("create registry dir");
    std::fs::write(
        root.join("registry/generated-artifact-control-plane.json"),
        r#"{
  "artifacts": [
    {
      "path": "ci/facade/artifact-inventory-registry/gate-baseline.generated.json",
      "materialization_mode": "main-branch-materialized"
    }
  ]
}"#,
    )
    .expect("write control-plane manifest");
    git(
        root,
        &["add", "registry/generated-artifact-control-plane.json"],
    );
    git(
        root,
        &["commit", "-m", "test: mark controller-owned baseline"],
    );
}

#[test]
fn dirty_tree_refusal_fixture_reports_settle_protocol() {
    let root = init_repo_with_faces();
    std::fs::write(root.join("README.md"), "content v2\n").expect("dirty tracked content");

    let error = assert_non_face_tree_clean(&root).expect_err("dirty content must be refused");
    let message = error.to_string();

    assert!(message.contains("README.md"));
    assert!(message.contains(FACE_SETTLE_PROTOCOL));
    assert!(message.contains("commit content changes first"));
    assert!(message.contains("faces regenerate from the TRACKED TREE STATE"));
    assert!(message.contains("never mix content and regenerated faces in one commit"));
}

#[test]
fn untracked_file_refusal_fixture_reports_settle_protocol() {
    let root = init_repo_with_faces();
    std::fs::write(root.join("new-policy.md"), "content v1\n").expect("write untracked file");

    let error = assert_non_face_tree_clean(&root).expect_err("untracked file must be refused");
    let message = error.to_string();

    assert!(message.contains("untracked files"));
    assert!(message.contains("new-policy.md"));
    assert!(message.contains(FACE_SETTLE_PROTOCOL));
    assert!(message.contains("faces regenerate from the TRACKED TREE STATE"));
}

#[test]
fn faces_only_staging_fixture_stages_only_generated_faces() {
    let root = init_repo_with_faces();

    let report = settle_regenerated_faces(&root, regenerated_faces(), FaceSettleMode::Settle)
        .expect("settle regenerated faces");

    assert!(report.message.contains(FACE_REMEDIATION_COMMAND));
    assert!(
        report
            .message
            .contains("git commit -S -m \"chore: settle generated cloud-ci faces\"")
    );
    let staged: BTreeSet<String> = git_output(&root, &["diff", "--cached", "--name-only"])
        .lines()
        .map(str::to_owned)
        .collect();
    let face_paths: BTreeSet<String> = generated_face_paths().into_iter().collect();
    assert_eq!(staged, face_paths);
    assert!(
        git_output(&root, &["diff", "--name-only"])
            .trim()
            .is_empty()
    );
}

#[test]
fn face_settle_does_not_stage_controller_owned_baseline() {
    let root = init_repo_with_faces();
    mark_gate_baseline_controller_owned(&root);

    let report = settle_regenerated_faces(&root, regenerated_faces(), FaceSettleMode::Settle)
        .expect("settle regenerated faces");

    assert!(
        !report
            .staged_faces
            .iter()
            .any(|path| path.ends_with("gate-baseline.generated.json"))
    );
    assert_eq!(
        std::fs::read_to_string(
            root.join("ci/facade/artifact-inventory-registry/gate-baseline.generated.json")
        )
        .expect("read baseline"),
        "old face\n"
    );
    let staged: BTreeSet<String> = git_output(&root, &["diff", "--cached", "--name-only"])
        .lines()
        .map(str::to_owned)
        .collect();
    let expected: BTreeSet<String> = generated_face_paths()
        .into_iter()
        .filter(|path| !path.ends_with("gate-baseline.generated.json"))
        .collect();
    assert_eq!(staged, expected);
}

#[test]
fn settle_and_commit_fixture_creates_exactly_one_faces_only_commit() {
    let root = init_repo_with_faces();

    let report =
        settle_regenerated_faces(&root, regenerated_faces(), FaceSettleMode::SettleAndCommit)
            .expect("settle and commit generated faces");

    assert!(report.committed);
    assert_eq!(
        git_output(&root, &["rev-list", "--count", "HEAD"]).trim(),
        "2"
    );
    assert_eq!(
        git_output(&root, &["log", "-1", "--pretty=%s"]).trim(),
        "chore: settle generated cloud-ci faces"
    );
    let committed_paths: BTreeSet<String> = git_output(
        &root,
        &["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"],
    )
    .lines()
    .map(str::to_owned)
    .collect();
    let face_paths: BTreeSet<String> = generated_face_paths().into_iter().collect();
    assert_eq!(committed_paths, face_paths);
    assert!(
        git_output(&root, &["status", "--porcelain=v1"])
            .trim()
            .is_empty()
    );
}

#[test]
fn parse_face_settle_args_maps_commit_to_settle_and_commit_mode() {
    let parsed = parse_face_settle_args(vec![
        "--repo-root".to_owned(),
        "/tmp/oyatie".to_owned(),
        "--settle".to_owned(),
        "--commit".to_owned(),
    ])
    .expect("parse settle commit args");

    assert_eq!(parsed.repo_root, PathBuf::from("/tmp/oyatie"));
    assert_eq!(parsed.mode, FaceSettleMode::SettleAndCommit);
}

#[test]
fn parse_face_settle_args_refuses_commit_without_settle() {
    let error = parse_face_settle_args(vec!["--commit".to_owned()])
        .expect_err("commit without settle must be refused");

    assert!(error.to_string().contains("--commit requires --settle"));
}

/// Mirror of the emitter's `is_generated_class` (cloud-ci-scm-facts-emitter-app
/// src/main.rs): the generated class is excluded from scm-facts last_touch so settle
/// (faces-only) and lock-refresh (Cargo.lock-only) commits are fixpoints.
fn is_generated_class(path: &str) -> bool {
    path.ends_with(".generated.json")
        || path.ends_with("Cargo.lock")
        || path.starts_with("docs/machine-readable/")
}

/// SYNTHETIC drift-bearing stand-in for the regenerated faces. It deliberately encodes
/// per-path last-touch metadata (the retired v1 scm-facts shape) so that commits touching
/// non-generated-class paths change the regenerated bytes while faces-only settle commits
/// and Cargo.lock-only refresh commits are fixpoints — the harshest drift profile the
/// settle MACHINERY must handle. The real v2 faces are tree-pure (ADR-0552: history-derived
/// facts live in the untracked volatile snapshot), so they drift strictly less than this
/// stand-in; the machinery contract being pinned here (any regenerated-byte drift after a
/// commit => verify fails; fixpoint commits => verify stays green) is shape-agnostic.
fn metadata_bearing_regenerated_faces(root: &Path) -> Vec<(String, String)> {
    let mut scm_facts = String::from(
        "scm-facts stand-in: per-path last_touch_commit over tracked non-generated-class paths\n",
    );
    for path in git_output(root, &["ls-files"]).lines() {
        if is_generated_class(path) {
            // Emitter fixpoint contract: generated-class paths are excluded from last_touch.
            continue;
        }
        let sha = git_output(root, &["log", "-1", "--format=%H", "--", path]);
        scm_facts.push_str(&format!("{path} {}\n", sha.trim()));
    }
    generated_face_paths()
        .into_iter()
        .map(|path| {
            let name = Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .expect("face name")
                .to_owned();
            let bytes = if name == "scm-facts.generated.json" {
                scm_facts.clone()
            } else {
                format!("stable producer bytes for {path}\n")
            };
            (name, bytes)
        })
        .collect()
}

fn settle_metadata_bearing_faces(root: &Path) {
    let report = settle_regenerated_faces(
        root,
        metadata_bearing_regenerated_faces(root),
        FaceSettleMode::SettleAndCommit,
    )
    .expect("settle and commit metadata-bearing faces");
    assert!(report.committed);
}

#[test]
fn verify_passes_on_properly_settled_tree() {
    let root = init_repo_with_faces();
    settle_metadata_bearing_faces(&root);

    let report = verify_committed_tree(&root, metadata_bearing_regenerated_faces(&root))
        .expect("verify settled tree");

    assert!(report.is_success());
    assert!(report.stale_faces.is_empty());
    assert!(report.lock_findings.is_empty());
    assert!(report.message.contains("settled at HEAD"));
}

#[test]
fn verify_fails_on_stale_lock_even_when_faces_are_settled() {
    // Review-blocker shape (pre-open review HIGH finding): the freshness gate checks BOTH
    // generated-face byte parity AND Cargo.lock member parity. A face-only verify would
    // exit 0 here and the push would still fail the gate — the exact wasted-CI-round-trip
    // class FRIC-1781250000 converts. Verify must run the gate's FULL check.
    let root = init_repo_with_faces();
    settle_metadata_bearing_faces(&root);
    // Bump the member version WITHOUT refreshing Cargo.lock; commit as content.
    write_member_manifest(&root, "0.2.0");
    git(&root, &["add", "libs/fixture-kernel/Cargo.toml"]);
    git(&root, &["commit", "-m", "feat: bump fixture kernel"]);
    // Re-settle so the FACE half is fresh again at HEAD; only the lock is stale now.
    settle_metadata_bearing_faces(&root);

    let report = verify_committed_tree(&root, metadata_bearing_regenerated_faces(&root))
        .expect("verify lock-stale tree");

    assert!(!report.is_success());
    assert!(
        report.stale_faces.is_empty(),
        "faces are settled; only the lock is stale"
    );
    assert_eq!(report.lock_findings.len(), 1);
    assert!(report.lock_findings[0].contains("lock_stale_member_version"));
    assert!(report.message.contains("STALE at HEAD"));
    assert!(report.message.contains(LOCK_REMEDIATION_COMMAND));
}

#[test]
fn verify_stays_green_after_lock_only_refresh_commit() {
    // Cargo.lock is generated-class: a lock-only refresh commit after settle does NOT
    // un-settle scm-facts (emitter excludes the generated class from last_touch), so the
    // lock remediation loop ends at verify-green without another settle round.
    let root = init_repo_with_faces();
    write_member_manifest(&root, "0.2.0");
    write_lock(&root, "0.1.0");
    git(&root, &["add", "."]);
    git(
        &root,
        &["commit", "-m", "feat: bump fixture kernel (lock stale)"],
    );
    settle_metadata_bearing_faces(&root);
    // Lock-only refresh commit AFTER the settle commit.
    write_lock(&root, "0.2.0");
    git(&root, &["add", "Cargo.lock"]);
    git(&root, &["commit", "-m", "chore: refresh Cargo.lock"]);

    let report = verify_committed_tree(&root, metadata_bearing_regenerated_faces(&root))
        .expect("verify after lock-only refresh");

    assert!(report.is_success(), "unexpected: {}", report.message);
    assert!(report.stale_faces.is_empty());
    assert!(report.lock_findings.is_empty());
}

#[test]
fn verify_fails_after_docs_only_commit_following_settle() {
    // Occurrence-3 shape (PR #695 round 2): the worker commits ANYTHING after the settle
    // commit — here a docs-only file the worker believes cannot affect the faces — and the
    // local "faces byte-identical" self-assessment is provably wrong because scm-facts
    // encodes per-path commit metadata.
    let root = init_repo_with_faces();
    settle_metadata_bearing_faces(&root);
    std::fs::write(root.join("docs-note.md"), "docs-only change\n").expect("write docs file");
    git(&root, &["add", "docs-note.md"]);
    git(&root, &["commit", "-m", "docs: add note after settle"]);

    let report = verify_committed_tree(&root, metadata_bearing_regenerated_faces(&root))
        .expect("verify stale tree");

    assert!(!report.is_success());
    assert_eq!(
        report.stale_faces,
        vec!["scm-facts.generated.json".to_owned()]
    );
    assert!(report.message.contains("STALE at HEAD"));
    assert!(report.message.contains("- scm-facts.generated.json"));
    assert!(report.message.contains(FACE_VERIFY_REMEDIATION_COMMAND));
    assert!(
        report
            .message
            .contains("cloud-ci-face-settle --settle --commit")
    );
}

#[test]
fn verify_fails_when_settle_commit_sits_mid_stack() {
    // Occurrence-1/2 shape (PR #690, PR #695): content commits land AFTER the settle commit
    // (leader scratch relocation / review-closure commits), leaving settle mid-stack.
    let root = init_repo_with_faces();
    settle_metadata_bearing_faces(&root);
    std::fs::write(root.join("README.md"), "content v2 after settle\n").expect("write content");
    git(&root, &["add", "README.md"]);
    git(&root, &["commit", "-m", "fix: review closure after settle"]);

    let report = verify_committed_tree(&root, metadata_bearing_regenerated_faces(&root))
        .expect("verify stale tree");

    assert!(!report.is_success());
    assert_eq!(
        report.stale_faces,
        vec!["scm-facts.generated.json".to_owned()]
    );
    assert!(report.message.contains(FACE_VERIFY_REMEDIATION_COMMAND));
}

#[test]
fn verify_never_mutates_the_tree() {
    // Pin byte-identity of the fixture tree across verify on BOTH the green (settled) and
    // red (stale) paths: HEAD must not move, no tracked path may change bytes, and no
    // untracked file may appear.
    let root = init_repo_with_faces();
    settle_metadata_bearing_faces(&root);

    let snapshot = |label: &str| {
        let head = git_output(&root, &["rev-parse", "HEAD"]);
        let status = git_output(
            &root,
            &["status", "--porcelain=v1", "--untracked-files=all"],
        );
        assert!(
            status.trim().is_empty(),
            "{label}: tree must be clean, got:\n{status}"
        );
        let faces: Vec<(String, String)> = generated_face_paths()
            .into_iter()
            .map(|path| {
                let bytes = std::fs::read_to_string(root.join(&path)).expect("read face");
                (path, bytes)
            })
            .collect();
        (head, faces)
    };

    let before_green = snapshot("before green verify");
    let green = verify_committed_tree(&root, metadata_bearing_regenerated_faces(&root))
        .expect("green verify");
    assert!(green.is_success());
    assert_eq!(before_green, snapshot("after green verify"));

    std::fs::write(root.join("docs-note.md"), "docs-only change\n").expect("write docs file");
    git(&root, &["add", "docs-note.md"]);
    git(&root, &["commit", "-m", "docs: add note after settle"]);

    let before_red = snapshot("before red verify");
    let red = verify_committed_tree(&root, metadata_bearing_regenerated_faces(&root))
        .expect("red verify");
    assert!(!red.is_success());
    assert_eq!(before_red, snapshot("after red verify"));
}

#[test]
fn verify_refuses_dirty_tracked_content() {
    let root = init_repo_with_faces();
    std::fs::write(root.join("README.md"), "content v2\n").expect("dirty tracked content");

    let error = assert_committed_tree_clean(&root).expect_err("dirty content must be refused");
    let message = error.to_string();

    assert!(message.contains("README.md"));
    assert!(message.contains("COMMITTED tree (HEAD)"));
    assert!(message.contains(FACE_SETTLE_PROTOCOL));
}

#[test]
fn verify_refuses_uncommitted_face_changes() {
    // The forgot-to-commit shape: --settle wrote the faces but the settle commit never
    // happened. Unlike the non-face cleanliness check, verify must refuse dirty FACE paths
    // too — the committed state cannot be certified while face edits sit uncommitted.
    let root = init_repo_with_faces();
    let face_path = generated_face_paths()
        .into_iter()
        .find(|path| path.ends_with("scm-facts.generated.json"))
        .expect("scm-facts face path");
    std::fs::write(root.join(&face_path), "regenerated but uncommitted\n")
        .expect("dirty face content");

    let error = assert_committed_tree_clean(&root).expect_err("dirty face must be refused");
    let message = error.to_string();

    assert!(message.contains("scm-facts.generated.json"));
    assert!(message.contains("--settle without --commit"));

    // The non-face assertion deliberately ignores face paths — verify must not.
    assert_non_face_tree_clean(&root).expect("non-face check ignores dirty faces");
}

#[test]
fn verify_refuses_untracked_files() {
    let root = init_repo_with_faces();
    std::fs::write(root.join("new-policy.md"), "content v1\n").expect("write untracked file");

    let error = assert_committed_tree_clean(&root).expect_err("untracked file must be refused");
    let message = error.to_string();

    assert!(message.contains("untracked files"));
    assert!(message.contains("new-policy.md"));
}

#[test]
fn parse_face_settle_args_maps_verify_mode() {
    let parsed = parse_face_settle_args(vec![
        "--repo-root".to_owned(),
        "/tmp/oyatie".to_owned(),
        "--verify".to_owned(),
    ])
    .expect("parse verify args");

    assert_eq!(parsed.repo_root, PathBuf::from("/tmp/oyatie"));
    assert_eq!(parsed.mode, FaceSettleMode::Verify);
}

#[test]
fn parse_face_settle_args_refuses_verify_combined_with_settle() {
    let error = parse_face_settle_args(vec!["--verify".to_owned(), "--settle".to_owned()])
        .expect_err("verify with settle must be refused");

    assert!(error.to_string().contains("read-only"));
}

#[test]
fn parse_face_settle_args_refuses_verify_combined_with_settle_and_commit() {
    let error = parse_face_settle_args(vec![
        "--settle".to_owned(),
        "--commit".to_owned(),
        "--verify".to_owned(),
    ])
    .expect_err("verify with settle/commit must be refused");

    assert!(error.to_string().contains("read-only"));
}
