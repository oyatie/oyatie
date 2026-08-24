//! End-to-end completeness checks against committed Git trees.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

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
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers=[]\nresolver='2'\n",
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
        .env("OYATIE_LAYOUT_BASE", base)
        .env("OYATIE_LAYOUT_HEAD", head)
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
