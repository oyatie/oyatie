//! End-to-end regressions for independently reviewed admission bypasses.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use pipeline_admission::{WORKSPACE_EXCLUDES, WORKSPACE_MEMBER_GLOBS};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "path-layout-review-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture");
    git(&root, &["init", "--quiet"]);
    write(&root, "Cargo.toml", &workspace_manifest());
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
        .output()
        .expect("run path layout admission")
}

#[cfg(unix)]
#[test]
fn every_live_changed_file_must_be_a_regular_git_blob() {
    use std::os::unix::fs::symlink;

    let root = fixture();
    let base = commit(&root, "base");
    write(
        &root,
        "policy/core/evaluate/Cargo.toml",
        "[package]\nname='policy-evaluate'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        "policy/core/evaluate/src/lib.rs",
        "pub fn check() {}\n",
    );
    for law in ["PRD.md", "SPEC.md", "PLAN.md"] {
        write(&root, &format!("policy/{law}"), "law\n");
    }
    symlink("PRD.md", root.join("policy/ADR.md")).expect("create owner-law symlink");
    let head = commit(&root, "symlink owner law");

    let rejected = admit(&root, &base, &head);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(error.contains("policy/ADR.md"), "{error}");
    assert!(error.contains("regular Git blob"), "{error}");
    let _ = std::fs::remove_dir_all(root);
}

fn assert_config_rejected(config: &str, expected: &str) {
    let root = fixture();
    let base = commit(&root, "base");
    write(&root, ".cargo/config.toml", config);
    let head = commit(&root, "cargo substitution");
    let rejected = admit(&root, &base, &head);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(error.contains(expected), "{error}");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn repository_cargo_substitutions_are_forbidden() {
    for (config, expected) in [
        (
            "[patch.crates-io]\nblob = { path = 'storage/ports/draft/blob' }\n",
            "dependency override `patch`",
        ),
        (
            "[target.x86_64-unknown-linux-gnu]\nrunner = 'true'\n",
            "target configuration is forbidden",
        ),
        (
            "[alias]\nnextest = '!true'\n",
            "command aliases are forbidden",
        ),
        (
            "[build]\nrustc-wrapper = 'true'\n",
            "build configuration is forbidden",
        ),
        (
            "[env]\nRUSTC_WRAPPER = 'true'\n",
            "environment override `RUSTC_WRAPPER` is forbidden",
        ),
    ] {
        assert_config_rejected(config, expected);
    }
}

#[test]
fn pull_request_merge_tree_uses_current_dev_ownership() {
    let root = fixture();
    let initial = commit(&root, "initial");
    write(&root, "policy/OWNERS", "policy\n");
    let candidate = commit(&root, "stale candidate");
    git(&root, &["branch", "candidate"]);

    git(&root, &["checkout", "--quiet", "-b", "dev", &initial]);
    write(
        &root,
        "policy/core/evaluate/Cargo.toml",
        "[package]\nname='policy-evaluate'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        "policy/core/evaluate/src/lib.rs",
        "pub fn check() {}\n",
    );
    let dev = commit(&root, "dev creates policy");

    git(&root, &["checkout", "--quiet", "candidate"]);
    git(
        &root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "merge",
            "--quiet",
            "--no-ff",
            "-m",
            "synthetic merge",
            "dev",
        ],
    );
    let merged = git_text(&root, &["rev-parse", "HEAD"]);

    assert!(!admit(&root, &dev, &candidate).status.success());
    let admitted = admit(&root, &dev, &merged);
    assert!(
        admitted.status.success(),
        "{}",
        String::from_utf8_lossy(&admitted.stderr)
    );
    let _ = std::fs::remove_dir_all(root);
}
