//! End-to-end semantic-name regression for repository-layout refusal.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use pipeline_admission::{WORKSPACE_EXCLUDES, WORKSPACE_MEMBER_GLOBS};

fn fixture() -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("path-layout-semantic-names-{}", std::process::id()));
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
            "[workspace]\nmembers = [\n{members}]\nexclude = [\n{excludes}]\nresolver = '2'\n[workspace.package]\nrust-version = '1.98.0'\n"
        ),
    );
    write(
        &root,
        "rust-toolchain.toml",
        "[toolchain]\nchannel = '1.98.0'\ncomponents = ['rustfmt', 'clippy']\nprofile = 'minimal'\n",
    );
    root
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture path parent"))
        .expect("create fixture parent");
    std::fs::write(path, contents).expect("write fixture file");
}

fn git(root: &Path, arguments: &[&str]) -> Output {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()
        .expect("spawn git");
    assert!(
        output.status.success(),
        "git {}: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    output
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
    String::from_utf8(git(root, &["rev-parse", "HEAD"]).stdout)
        .expect("commit id")
        .trim()
        .to_owned()
}

fn has_decision_identifier(value: &str) -> bool {
    ["ADR-", "D-"].into_iter().any(|marker| {
        value.split(marker).skip(1).any(|tail| {
            tail.bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_digit())
        })
    })
}

#[test]
fn refusal_uses_a_semantic_primary_name() {
    let root = fixture();
    let base = commit(&root, "base");
    write(&root, "plan/legacy.md", "not an admitted root\n");
    let head = commit(&root, "invalid root");
    let output = Command::new(env!("CARGO_BIN_EXE_pipeline-path-layout-app"))
        .current_dir(&root)
        .args([base, head])
        .output()
        .expect("run repository-layout admission");
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).expect("utf-8 diagnostic");
    assert!(error.starts_with("repository layout refused:\n"), "{error}");
    assert!(
        !has_decision_identifier(&error),
        "refusal exposes decision provenance as its operational name: {error}"
    );
    let _ = std::fs::remove_dir_all(root);
}
