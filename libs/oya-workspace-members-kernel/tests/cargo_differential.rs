//! Hermetic differential fixtures for Cargo workspace-member expansion.
//!
//! These are integration tests because they exercise the external Cargo boundary. The kernel's
//! unit tests remain pure and cover its own error classification independently.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use oya_workspace_members_kernel::{ResolveError, resolve_member_dirs};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-workspace-members-cargo-differential-{tag}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture file has a parent"))
        .expect("create fixture parent");
    std::fs::write(path, content).expect("write fixture file");
}

fn crate_manifest(name: &str) -> String {
    format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
}

fn cargo_metadata(root: &Path) -> std::process::Output {
    Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args([
            "metadata",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(root)
        .output()
        .expect("spawn Cargo metadata for hermetic fixture")
}

#[cfg(windows)]
#[test]
fn cyclic_directory_symlink_inspection_error_matches_cargo_success() {
    use std::os::windows::fs::symlink_dir;

    let root = fixture_root("windows-cyclic-directory-symlink");
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"members/*\"]\nresolver = \"2\"\n",
    );
    std::fs::create_dir_all(root.join("members")).expect("create member root");
    symlink_dir("loop", root.join("members/loop")).expect("create cyclic directory symlink");

    let inspection_error = std::fs::metadata(root.join("members/loop"))
        .expect_err("the cyclic directory symlink must fail filesystem inspection");
    assert_eq!(
        inspection_error.raw_os_error(),
        Some(1921),
        "the fixture must specifically exercise ERROR_CANT_RESOLVE_FILENAME"
    );

    let cargo = cargo_metadata(&root);
    assert!(
        cargo.status.success(),
        "Cargo must skip ERROR_CANT_RESOLVE_FILENAME while expanding member globs: {}",
        String::from_utf8_lossy(&cargo.stderr)
    );
    assert_eq!(
        resolve_member_dirs(&root)
            .expect("owned resolver must match Cargo's skipped inspection error"),
        Vec::<String>::new()
    );

    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn directory_symlink_member_matches_cargo_success() {
    use std::os::unix::fs::symlink;

    let root = fixture_root("directory-symlink");
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"members/*\"]\nresolver = \"2\"\n",
    );
    write(&root, "real/Cargo.toml", &crate_manifest("real-member"));
    write(&root, "real/src/lib.rs", "pub fn member() {}\n");
    std::fs::create_dir_all(root.join("members")).expect("create member root");
    symlink("../real", root.join("members/link")).expect("create directory symlink");

    let cargo = cargo_metadata(&root);
    assert!(
        cargo.status.success(),
        "Cargo must resolve the directory symlink fixture: {}",
        String::from_utf8_lossy(&cargo.stderr)
    );
    assert_eq!(
        resolve_member_dirs(&root).expect("owned resolver must match Cargo success"),
        vec!["members/link".to_owned()]
    );

    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn symlink_to_directory_without_manifest_matches_cargo_failure() {
    use std::os::unix::fs::symlink;

    let root = fixture_root("missing-manifest-symlink");
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"members/*\"]\nresolver = \"2\"\n",
    );
    std::fs::create_dir_all(root.join("real")).expect("create non-member directory");
    std::fs::create_dir_all(root.join("members")).expect("create member root");
    symlink("../real", root.join("members/link")).expect("create directory symlink");

    let cargo = cargo_metadata(&root);
    assert!(
        !cargo.status.success(),
        "Cargo must reject a symlink member directory without Cargo.toml"
    );
    assert_eq!(
        resolve_member_dirs(&root),
        Err(ResolveError::MissingManifests(vec![
            "members/link".to_owned()
        ]))
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn unexcluded_missing_manifest_matches_cargo_failure() {
    let root = fixture_root("missing-manifest");
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"members/*\"]\nresolver = \"2\"\n",
    );
    std::fs::create_dir_all(root.join("members/not-a-crate")).expect("create invalid member");

    let cargo = cargo_metadata(&root);
    assert!(
        !cargo.status.success(),
        "Cargo must reject an unexcluded member directory without Cargo.toml"
    );
    assert_eq!(
        resolve_member_dirs(&root),
        Err(ResolveError::MissingManifests(vec![
            "members/not-a-crate".to_owned()
        ]))
    );

    let _ = std::fs::remove_dir_all(root);
}
