//! Git-fixture helpers shared by the path-layout admission suites.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use pipeline_admission::{WORKSPACE_EXCLUDES, WORKSPACE_MEMBER_GLOBS};

pub static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn fixture() -> PathBuf {
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

pub fn workspace_manifest(extra_excludes: &[&str]) -> String {
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

pub fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture path parent"))
        .expect("create fixture parent");
    std::fs::write(path, contents).expect("write fixture file");
}

pub fn commit(root: &Path, message: &str) -> String {
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

pub fn git(root: &Path, args: &[&str]) {
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

pub fn git_text(root: &Path, args: &[&str]) -> String {
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

pub fn admit(root: &Path, base: &str, head: &str) -> Output {
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
