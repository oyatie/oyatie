#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use oya_cloud_ci_freshness_app::{
    FACE_REMEDIATION_COMMAND, FACE_SETTLE_PROTOCOL, FaceSettleMode, assert_non_face_tree_clean,
    generated_face_paths, parse_face_settle_args, settle_regenerated_faces,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-face-settle-{}-{}",
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
    for path in generated_face_paths() {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().expect("face parent")).expect("create face parent");
        std::fs::write(path, "old face\n").expect("write face");
    }
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "seed"]);
    root
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

#[test]
fn dirty_tree_refusal_fixture_reports_settle_protocol() {
    let root = init_repo_with_faces();
    std::fs::write(root.join("README.md"), "content v2\n").expect("dirty tracked content");

    let error = assert_non_face_tree_clean(&root).expect_err("dirty content must be refused");
    let message = error.to_string();

    assert!(message.contains("README.md"));
    assert!(message.contains(FACE_SETTLE_PROTOCOL));
    assert!(message.contains("commit content changes first"));
    assert!(message.contains("faces regenerate from TRACKED paths"));
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
    assert!(message.contains("faces regenerate from TRACKED paths"));
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
