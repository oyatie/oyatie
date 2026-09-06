//! End-to-end regressions for the second independent admission review.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use pipeline_admission::{WORKSPACE_EXCLUDES, WORKSPACE_MEMBER_GLOBS};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "path-layout-followup-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture");
    git(&root, &["init", "--quiet"]);
    write(&root, "Cargo.toml", &workspace_manifest());
    write(&root, "rust-toolchain.toml", valid_toolchain());
    root
}

fn workspace_manifest() -> String {
    let members = WORKSPACE_MEMBER_GLOBS
        .iter()
        .map(|entry| format!("  {entry:?},\n"))
        .collect::<String>();
    let excludes = WORKSPACE_EXCLUDES
        .iter()
        .map(|entry| format!("  {entry:?},\n"))
        .collect::<String>();
    format!(
        "[workspace]\nmembers = [\n{members}]\nexclude = [\n{excludes}]\nresolver = '2'\n[workspace.package]\nrust-version = '1.98.0'\n"
    )
}

fn valid_toolchain() -> &'static str {
    "[toolchain]\nchannel = '1.98.0'\ncomponents = ['rustfmt', 'clippy']\nprofile = 'minimal'\n"
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

#[test]
fn cargo_include_and_non_virtual_workspace_are_rejected() {
    let root = fixture();
    let base = commit(&root, "base");
    write(&root, ".cargo/config.toml", "include = 'bypass.toml'\n");
    write(
        &root,
        ".cargo/bypass.toml",
        "[target.x86_64-unknown-linux-gnu]\nrunner = 'true'\n",
    );
    let included = commit(&root, "include cargo bypass");
    assert_rejected(&root, &base, &included, "includes are forbidden");

    git(&root, &["reset", "--hard", &base]);
    let package = format!(
        "{}\n[package]\nname='shadow-root'\nversion='0.1.0'\nedition='2024'\n[lib]\npath='README.md'\n",
        workspace_manifest()
    );
    write(&root, "Cargo.toml", &package);
    write(&root, "README.md", "shadow crate\n");
    let packaged = commit(&root, "make root a package");
    assert_rejected(&root, &base, &packaged, "must remain virtual");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn docs_base_and_pack_shadow_payloads_are_rejected() {
    for (path, contents, expected) in [
        (
            "docs/scratch/Cargo.toml",
            "[package]\n",
            "root docs are limited",
        ),
        (
            "base/ports/blob/Cargo.toml",
            "[package]\n",
            "base admits only",
        ),
        (
            "packs/eu/plan/todo.md",
            "later\n",
            "frozen non-root Markdown",
        ),
    ] {
        let root = fixture();
        let base = commit(&root, "base");
        write(&root, path, contents);
        let head = commit(&root, "add shadow payload");
        assert_rejected(&root, &base, &head, expected);
        let _ = std::fs::remove_dir_all(root);
    }
}

#[test]
fn a_complete_owner_cannot_decay_to_paperwork() {
    let root = fixture();
    write(
        &root,
        "network/core/dataplane/Cargo.toml",
        "[package]\nname='network-dataplane'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        "network/core/dataplane/src/lib.rs",
        "pub fn check() {}\n",
    );
    write(&root, "network/OWNERS", "network-owner\n");
    let base = commit(&root, "complete network owner");
    git(&root, &["rm", "-r", "network/core/dataplane"]);
    let head = commit(&root, "remove last core crate");
    assert_rejected(&root, &base, &head, "last complete core crate");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn touched_handwritten_files_obey_the_three_hundred_line_budget() {
    let root = fixture();
    let base = commit(&root, "base");
    write(&root, "README.md", &"line\n".repeat(301));
    let head = commit(&root, "oversized readme");
    assert_rejected(&root, &base, &head, "repository 300-line file budget");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn a_new_ordinary_capability_requires_complete_owner_shape() {
    let root = fixture();
    let base = commit(&root, "base");
    write(&root, "network/README.md", "paperwork only\n");
    let paperwork = commit(&root, "incomplete network owner");
    assert_rejected(
        &root,
        &base,
        &paperwork,
        "new capability owner requires one core crate",
    );

    git(&root, &["reset", "--hard", &base]);
    write(
        &root,
        "network/core/route/Cargo.toml",
        "[package]\nname='network-route'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        "network/core/route/src/lib.rs",
        "pub fn route() {}\n",
    );
    write(&root, "network/OWNERS", "network-owner\n");
    let complete = commit(&root, "complete network owner");
    let admitted = admit(&root, &base, &complete);
    assert!(
        admitted.status.success(),
        "{}",
        String::from_utf8_lossy(&admitted.stderr)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn loaded_test_data_can_change_inside_the_bounded_fixture_grammar() {
    let root = fixture();
    write(
        &root,
        "cell/core/regional-pack/Cargo.toml",
        "[package]\nname='cell-regional-pack'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        "cell/core/regional-pack/src/lib.rs",
        "pub fn pack() {}\n",
    );
    write(
        &root,
        "cell/core/regional-pack/tests/fixtures/kr/manifest.json",
        "{}\n",
    );
    let base = commit(&root, "fixture base");
    write(
        &root,
        "cell/core/regional-pack/tests/fixtures/kr/manifest.json",
        "{\"version\":1}\n",
    );
    let head = commit(&root, "refresh loaded fixture");
    let admitted = admit(&root, &base, &head);
    assert!(
        admitted.status.success(),
        "{}",
        String::from_utf8_lossy(&admitted.stderr)
    );
    let _ = std::fs::remove_dir_all(root);
}
