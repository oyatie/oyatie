//! End-to-end completeness checks against committed Git trees.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use pipeline_admission::{WORKSPACE_EXCLUDES, WORKSPACE_MEMBER_GLOBS};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "path-layout-completeness-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture");
    git(&root, &["init", "--quiet"]);
    write(&root, "Cargo.toml", &workspace_manifest(&[]));
    root
}

fn workspace_manifest(extra_excludes: &[&str]) -> String {
    let members = WORKSPACE_MEMBER_GLOBS
        .iter()
        .map(|entry| format!("  {entry:?},\n"))
        .collect::<String>();
    let excludes = WORKSPACE_EXCLUDES
        .iter()
        .chain(extra_excludes)
        .map(|entry| format!("  {entry:?},\n"))
        .collect::<String>();
    format!("[workspace]\nmembers = [\n{members}]\nexclude = [\n{excludes}]\nresolver = '2'\n")
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture path parent"))
        .expect("create fixture parent");
    std::fs::write(path, contents).expect("write fixture file");
}

fn commit(root: &Path, message: &str) -> String {
    git(root, &["add", "-A"]);
    git(
        root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "-m",
            message,
        ],
    );
    git_text(root, &["rev-parse", "HEAD"])
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("spawn git");
    assert!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_text(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("spawn git");
    assert!(output.status.success(), "git {}", args.join(" "));
    String::from_utf8(output.stdout)
        .expect("git text")
        .trim()
        .to_owned()
}

fn admit(root: &Path, base: &str, head: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pipeline-path-layout-app"))
        .current_dir(root)
        .args([base, head])
        // A repository Cargo config can force these legacy variables. Conflicting values
        // prove that the trusted event SHAs now travel as process arguments instead.
        .env("OYATIE_LAYOUT_BASE", base)
        .env("OYATIE_LAYOUT_HEAD", base)
        .output()
        .expect("run path layout admission")
}

#[test]
fn touched_face_leaf_must_be_complete_unless_fully_deleted() {
    let root = fixture();
    let base = commit(&root, "base");
    write(
        &root,
        "network/ports/blob/tests/fixture.rs",
        "#[test] fn fixture() {}\n",
    );
    let incomplete = commit(&root, "incomplete leaf");
    let rejected = admit(&root, &base, &incomplete);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(error.contains("touched face leaf must contain `Cargo.toml`"));
    assert!(error.contains("canonical entry point"));

    write(
        &root,
        "network/ports/blob/Cargo.toml",
        "[package]\nname='network-blob'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(&root, "network/ports/blob/src/lib.rs", "pub fn blob() {}\n");
    let complete = commit(&root, "complete leaf");
    assert!(
        admit(&root, &base, &complete).status.success(),
        "a complete touched crate must admit"
    );

    std::fs::remove_dir_all(root.join("network/ports/blob")).expect("delete whole crate");
    let deleted = commit(&root, "delete complete leaf");
    assert!(
        admit(&root, &complete, &deleted).status.success(),
        "a fully removed leaf must not require ghost crate files"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn tracked_symlink_cannot_disguise_an_owner_draft_dependency() {
    use std::os::unix::fs::symlink;

    let root = fixture();
    let base = commit(&root, "base");
    write(
        &root,
        "storage/ports/draft/blob/Cargo.toml",
        "[package]\nname='storage-blob-draft'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        "storage/ports/draft/blob/src/lib.rs",
        "pub fn blob() {}\n",
    );
    write(
        &root,
        "network/core/route/Cargo.toml",
        "[package]\nname='network-route'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nblob={path='src/blob.rs'}\n",
    );
    write(
        &root,
        "network/core/route/src/lib.rs",
        "pub fn route() {}\n",
    );
    symlink(
        "../../../../storage/ports/draft/blob",
        root.join("network/core/route/src/blob.rs"),
    )
    .expect("create dependency-path symlink");
    let disguised = commit(&root, "disguised draft dependency");

    let rejected = admit(&root, &base, &disguised);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        error.contains("dependency `blob` has unsafe path `src/blob.rs`"),
        "{error}"
    );
    assert!(
        error.contains("tracked symlink component `network/core/route/src/blob.rs`"),
        "{error}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn trusted_sha_arguments_cannot_be_overridden_by_cargo_environment() {
    let root = fixture();
    let base = commit(&root, "base");
    write(&root, "plan/forbidden.md", "must not admit\n");
    let head = commit(&root, "forbidden path");

    let rejected = admit(&root, &base, &head);
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("forbidden root `plan`"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn workspace_excludes_cannot_hide_a_member_from_workspace_tests() {
    let root = fixture();
    let base = commit(&root, "base");
    write(
        &root,
        "Cargo.toml",
        &workspace_manifest(&["network/core/route"]),
    );
    let head = commit(&root, "hide workspace member");

    let rejected = admit(&root, &base, &head);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(error.contains("unexpected workspace exclude `network/core/route`"));
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn canonical_entrypoint_must_be_a_regular_git_blob() {
    use std::os::unix::fs::symlink;

    let root = fixture();
    let base = commit(&root, "base");
    write(
        &root,
        "storage/core/blob/Cargo.toml",
        "[package]\nname='storage-blob'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(&root, "storage/core/blob/src/lib.rs", "pub fn blob() {}\n");
    write(
        &root,
        "network/core/route/Cargo.toml",
        "[package]\nname='network-route'\nversion='0.1.0'\nedition='2024'\n",
    );
    std::fs::create_dir_all(root.join("network/core/route/src"))
        .expect("create consumer source directory");
    symlink(
        "../../../../storage/core/blob/src/lib.rs",
        root.join("network/core/route/src/lib.rs"),
    )
    .expect("create cross-owner entrypoint symlink");
    let head = commit(&root, "symlinked entrypoint");

    let rejected = admit(&root, &base, &head);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(error.contains("canonical entry point"), "{error}");
    assert!(error.contains("regular Git blob"), "{error}");
    let _ = std::fs::remove_dir_all(root);
}
