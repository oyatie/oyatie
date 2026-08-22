//! Hermetic differential fixtures for Cargo workspace-member expansion.
//!
//! These are integration tests because they exercise the external Cargo boundary. The kernel's
//! unit tests remain pure and cover its own error classification independently.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use workspace_members_kernel::{ResolveError, resolve_member_dirs};
use serde_json::Value;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "workspace-members-cargo-differential-{tag}-{}-{}",
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

#[cfg(unix)]
fn failed_exit_status() -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;

    std::process::ExitStatus::from_raw(1 << 8)
}

#[cfg(windows)]
fn failed_exit_status() -> std::process::ExitStatus {
    use std::os::windows::process::ExitStatusExt;

    std::process::ExitStatus::from_raw(1)
}

#[test]
fn cargo_metadata_uses_workspace_root_prefix_and_preserves_member_identity() {
    let root = fixture_root("reported-workspace-root");
    let metadata = r#"{
        "workspace_root": "/cargo-reported-workspace-root",
        "workspace_members": ["link 0.1.0 (path+file:///cargo-reported-workspace-root/members/link)"],
        "packages": [{
            "id": "link 0.1.0 (path+file:///cargo-reported-workspace-root/members/link)",
            "manifest_path": "/cargo-reported-workspace-root/members/link/Cargo.toml"
        }]
    }"#;

    assert_eq!(
        cargo_workspace_member_dirs(metadata),
        BTreeSet::from(["members/link".to_owned()])
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn missing_member_manifest_accepts_windows_separators_without_cargo_wording() {
    let cargo = std::process::Output {
        status: failed_exit_status(),
        stdout: Vec::new(),
        stderr: b"error: could not read C:\\workspace\\members\\not-a-crate\\Cargo.toml".to_vec(),
    };

    assert_cargo_missing_member_manifest(&cargo, "members/not-a-crate/Cargo.toml");
}

fn cargo_workspace_member_dirs(metadata: &str) -> BTreeSet<String> {
    let metadata: Value = serde_json::from_str(metadata).expect("Cargo metadata must be JSON");
    let workspace_root = PathBuf::from(
        metadata["workspace_root"]
            .as_str()
            .expect("Cargo metadata workspace root"),
    );
    let workspace_members = metadata["workspace_members"]
        .as_array()
        .expect("Cargo metadata workspace_members array")
        .iter()
        .map(|id| id.as_str().expect("workspace member package ID"))
        .collect::<BTreeSet<_>>();
    metadata["packages"]
        .as_array()
        .expect("Cargo metadata packages array")
        .iter()
        .filter(|package| {
            package["id"]
                .as_str()
                .is_some_and(|id| workspace_members.contains(id))
        })
        .map(|package| {
            let manifest = PathBuf::from(
                package["manifest_path"]
                    .as_str()
                    .expect("workspace package manifest path"),
            );
            manifest
                .parent()
                .expect("manifest has parent")
                .strip_prefix(&workspace_root)
                .expect("Cargo metadata manifest lies under its workspace root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

fn assert_cargo_metadata_matches_owned_resolver(root: &Path, cargo: std::process::Output) {
    assert!(
        cargo.status.success(),
        "Cargo metadata must resolve fixture: {}",
        String::from_utf8_lossy(&cargo.stderr)
    );
    let cargo_members = cargo_workspace_member_dirs(
        &String::from_utf8(cargo.stdout).expect("Cargo metadata stdout UTF-8"),
    );
    let owned_members = resolve_member_dirs(root)
        .expect("owned resolver must resolve fixture")
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(owned_members, cargo_members);
}

fn assert_cargo_missing_member_manifest(cargo: &std::process::Output, member_manifest: &str) {
    assert!(
        !cargo.status.success(),
        "Cargo must reject missing member manifest"
    );
    let cargo_error = String::from_utf8_lossy(&cargo.stderr).replace('\\', "/");
    assert!(
        cargo_error.contains(member_manifest),
        "Cargo failure must name the missing member manifest: {cargo_error}"
    );
}

#[test]
fn cargo_metadata_workspace_members_match_owned_resolver() {
    let root = fixture_root("metadata-members");
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"members/*\"]\nexclude = [\"members/excluded\"]\nresolver = \"2\"\n",
    );
    write(&root, "members/one/Cargo.toml", &crate_manifest("one"));
    write(&root, "members/one/src/lib.rs", "pub fn one() {}\n");
    write(&root, "members/two/Cargo.toml", &crate_manifest("two"));
    write(&root, "members/two/src/lib.rs", "pub fn two() {}\n");
    write(
        &root,
        "members/excluded/Cargo.toml",
        &crate_manifest("excluded"),
    );

    assert_cargo_metadata_matches_owned_resolver(&root, cargo_metadata(&root));

    let _ = std::fs::remove_dir_all(root);
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

    assert_cargo_metadata_matches_owned_resolver(&root, cargo_metadata(&root));

    let _ = std::fs::remove_dir_all(root);
}

/// Unix arm of the cyclic-symlink differential above.
///
/// The PROPERTY under test is portable: when a `members = ["members/*"]` glob hits a directory
/// symlink that cannot be inspected, the owned resolver must agree with cargo. Only the errno
/// is platform-specific — Windows reports ERROR_CANT_RESOLVE_FILENAME (1921), Unix reports
/// ELOOP — so the Windows arm keeps the exact-errno assertion and this arm asserts the
/// portable part.
///
/// Why this exists: the differential was reachable ONLY through a `windows-latest` matrix leg,
/// which made a genuinely portable invariant depend on Windows CI capacity. The raw errno is
/// deliberately NOT asserted here: ELOOP is 40 on Linux and 62 on macOS/BSD, and pulling `libc`
/// into this crate to name it would put a transient dependency in a `*-kernel` crate, which the
/// ADR-0547 kernel-purity gate forbids. `expect_err` plus the resolver differential is the
/// portable contract; the platform-specific constant stays with the platform that defines it.
#[cfg(unix)]
#[test]
fn cyclic_directory_symlink_inspection_error_matches_cargo_success_unix() {
    use std::os::unix::fs::symlink;

    let root = fixture_root("unix-cyclic-directory-symlink");
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"members/*\"]\nresolver = \"2\"\n",
    );
    std::fs::create_dir_all(root.join("members")).expect("create member root");
    // Self-referential relative symlink: resolving `loop` requires resolving `loop`.
    symlink("loop", root.join("members/loop")).expect("create cyclic directory symlink");

    std::fs::metadata(root.join("members/loop"))
        .expect_err("the cyclic directory symlink must fail filesystem inspection");

    assert_cargo_metadata_matches_owned_resolver(&root, cargo_metadata(&root));

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

    assert_cargo_metadata_matches_owned_resolver(&root, cargo_metadata(&root));

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
    assert_cargo_missing_member_manifest(&cargo, "members/link/Cargo.toml");
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
    assert_cargo_missing_member_manifest(&cargo, "members/not-a-crate/Cargo.toml");
    assert_eq!(
        resolve_member_dirs(&root),
        Err(ResolveError::MissingManifests(vec![
            "members/not-a-crate".to_owned()
        ]))
    );

    let _ = std::fs::remove_dir_all(root);
}
