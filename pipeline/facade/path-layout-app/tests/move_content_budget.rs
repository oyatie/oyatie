//! End-to-end boundaries for content-budget handling during path moves.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use pipeline_admission::{WORKSPACE_EXCLUDES, WORKSPACE_MEMBER_GLOBS};

static COUNTER: AtomicU64 = AtomicU64::new(0);
const LEGACY_SOURCE: &str = "network/core/engine/src/legacy.rs";
const MOVED_SOURCE: &str = "network/core/engine/src/moved.rs";

fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "path-layout-move-budget-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture");
    git(&root, &["init", "--quiet"]);
    let members = WORKSPACE_MEMBER_GLOBS
        .iter()
        .map(|entry| format!("  {entry:?},\n"))
        .collect::<String>();
    let excludes = WORKSPACE_EXCLUDES
        .iter()
        .map(|entry| format!("  {entry:?},\n"))
        .collect::<String>();
    write(
        &root,
        "Cargo.toml",
        &format!(
            "[workspace]\nmembers = [\n{members}]\nexclude = [\n{excludes}]\nresolver = '2'\n"
        ),
    );
    root
}

fn oversized_text() -> String {
    (0..301).map(|line| format!("line {line:03}\n")).collect()
}

fn write_complete_owner(root: &Path) {
    write(
        root,
        "network/core/engine/Cargo.toml",
        "[package]\nname='network-engine'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        root,
        "network/core/engine/src/lib.rs",
        "pub fn engine() {}\n",
    );
    for law in ["ADR.md", "PRD.md", "SPEC.md", "PLAN.md"] {
        write(root, &format!("network/{law}"), "law\n");
    }
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
        .output()
        .expect("run path layout admission")
}

fn assert_rejected(root: &Path, base: &str, head: &str, expected: &str) -> String {
    let output = admit(root, base, head);
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).expect("admission error text");
    assert!(error.contains(expected), "{error}");
    error
}

#[test]
fn byte_identical_move_does_not_reinspect_existing_content_debt() {
    let root = fixture();
    write_complete_owner(&root);
    write(&root, LEGACY_SOURCE, &oversized_text());
    let base = commit(&root, "existing content debt");
    git(&root, &["mv", LEGACY_SOURCE, MOVED_SOURCE]);
    let head = commit(&root, "move existing content");

    let output = admit(&root, &base, &head);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn edited_move_remains_content_budgeted() {
    let root = fixture();
    write_complete_owner(&root);
    write(&root, LEGACY_SOURCE, &oversized_text());
    let base = commit(&root, "existing content debt");
    git(&root, &["mv", LEGACY_SOURCE, MOVED_SOURCE]);
    let mut changed = oversized_text();
    changed.replace_range(..8, "changed ");
    write(&root, MOVED_SOURCE, &changed);
    let head = commit(&root, "move and edit content");
    assert_rejected(&root, &base, &head, "repository 300-line file budget");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn unchanged_copy_remains_content_budgeted() {
    let root = fixture();
    write_complete_owner(&root);
    write(&root, LEGACY_SOURCE, &oversized_text());
    let base = commit(&root, "existing content debt");
    std::fs::copy(root.join(LEGACY_SOURCE), root.join(MOVED_SOURCE)).expect("copy fixture content");
    let head = commit(&root, "copy existing content");
    assert_rejected(&root, &base, &head, "repository 300-line file budget");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn byte_identical_move_still_obeys_path_admission() {
    let root = fixture();
    write_complete_owner(&root);
    write(&root, LEGACY_SOURCE, &oversized_text());
    let base = commit(&root, "existing content debt");
    git(&root, &["mv", LEGACY_SOURCE, "unexpected.rs"]);
    let head = commit(&root, "move content outside admitted layout");

    let error = assert_rejected(&root, &base, &head, "unknown root file `unexpected.rs`");
    assert!(
        !error.contains("repository 300-line file budget"),
        "{error}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn byte_identical_symlink_move_still_requires_a_regular_blob() {
    use std::os::unix::fs::symlink;

    let root = fixture();
    write_complete_owner(&root);
    symlink("lib.rs", root.join(LEGACY_SOURCE)).expect("create fixture symlink");
    let base = commit(&root, "existing symlink");
    git(&root, &["mv", LEGACY_SOURCE, MOVED_SOURCE]);
    let head = commit(&root, "move symlink");

    assert_rejected(&root, &base, &head, "regular Git blob");
    let _ = std::fs::remove_dir_all(root);
}
