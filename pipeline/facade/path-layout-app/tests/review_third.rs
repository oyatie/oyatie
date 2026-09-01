//! Exact regressions from the third independent repository-layout review.
//! Provenance: ADR-0719.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use pipeline_admission::{WORKSPACE_EXCLUDES, WORKSPACE_MEMBER_GLOBS};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "path-layout-third-review-{}-{}",
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

fn assert_rejected(root: &Path, base: &str, head: &str, expected: &str) {
    let output = admit(root, base, head);
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains(expected), "{error}");
}

fn assert_admitted(root: &Path, base: &str, head: &str) {
    let output = admit(root, base, head);
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{error}");
}

fn write_owner(root: &Path, owner: &str, with_law: bool) {
    write(
        root,
        &format!("{owner}/core/engine/Cargo.toml"),
        &format!(
            "[package]\nname='{}-engine'\nversion='0.1.0'\nedition='2024'\n",
            owner.strip_prefix("app/").unwrap_or(owner)
        ),
    );
    write(
        root,
        &format!("{owner}/core/engine/src/lib.rs"),
        "pub fn engine() {}\n",
    );
    if with_law {
        for law in ["ADR.md", "PRD.md", "SPEC.md", "PLAN.md"] {
            write(root, &format!("{owner}/{law}"), "law\n");
        }
    }
}

#[test]
fn github_cannot_hide_a_workspace_crate() {
    let root = fixture();
    let base = commit(&root, "base");
    write(
        &root,
        ".github/core/shadow/Cargo.toml",
        "[package]\nname='shadow'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        ".github/core/shadow/src/lib.rs",
        "pub fn shadow() {}\n",
    );
    let head = commit(&root, "hidden crate");
    assert_rejected(&root, &base, &head, "`.github/` admits");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn existing_scaffold_is_admitted_by_its_first_core() {
    let root = fixture();
    write(&root, "app/calendar/README.md", "scaffold\n");
    let base = commit(&root, "scaffold");
    write_owner(&root, "app/calendar", false);
    let head = commit(&root, "first implementation");
    assert_admitted(&root, &base, &head);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ordinary_change_cannot_delete_frozen_owner_markdown() {
    let root = fixture();
    write_owner(&root, "network", true);
    let base = commit(&root, "implemented owner");
    git(&root, &["rm", "--quiet", "network/PRD.md"]);
    let head = commit(&root, "delete law");
    assert_rejected(&root, &base, &head, "non-root Markdown is frozen");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn impossible_target_dependencies_do_not_admit_base() {
    let root = fixture();
    for owner in ["network", "storage", "compute"] {
        write_owner(&root, owner, true);
    }
    let base = commit(&root, "three capabilities");
    write(
        &root,
        "base/core/bytes/Cargo.toml",
        "[package]\nname='base-bytes'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(&root, "base/core/bytes/src/lib.rs", "pub struct Bytes;\n");
    for owner in ["network", "storage", "compute"] {
        write(
            &root,
            &format!("{owner}/core/engine/Cargo.toml"),
            &format!(
                "[package]\nname='{owner}-engine'\nversion='0.1.0'\nedition='2024'\n[target.'cfg(any())'.dependencies]\nbase-bytes={{path='../../../base/core/bytes'}}\n"
            ),
        );
    }
    let head = commit(&root, "fake base quorum");
    assert_rejected(&root, &base, &head, "found 0");
    let _ = std::fs::remove_dir_all(root);
}
