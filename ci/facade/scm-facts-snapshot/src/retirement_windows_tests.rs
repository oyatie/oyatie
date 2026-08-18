//! Windows writer regressions for the canonical ignored generated faces.

use super::{
    CanonicalIgnoredGeneratedWriter, CanonicalRetirementFactsWriter, GENERATED_FACTS_PATH,
    NEXT_ATOMIC_WRITE_ID,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::Ordering;

const WINDOWS_IGNORED_RECEIPT: &str =
    "ci/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json";

fn windows_temp_git_repo(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "retirement-windows-{label}-{}-{}",
        std::process::id(),
        NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).expect("create windows writer repository");
    let status = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&root)
        .status()
        .expect("run git init");
    assert!(status.success(), "git init must succeed");
    root
}

fn write_windows_ignore(root: &Path) {
    std::fs::write(
        root.join(".gitignore"),
        format!("/{GENERATED_FACTS_PATH}\n/{WINDOWS_IGNORED_RECEIPT}\n"),
    )
    .expect("write gitignore");
}

fn create_windows_directory_reparse(link: &Path, target: &Path) {
    if std::os::windows::fs::symlink_dir(target, link).is_ok() {
        return;
    }
    let status = Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            &link.to_string_lossy(),
            &target.to_string_lossy(),
        ])
        .status()
        .expect("run mklink /J");
    assert!(
        status.success(),
        "must create a directory symlink or junction at {}",
        link.display()
    );
}

/// Both Windows writers succeed on an ignored, untracked temp git repo.
#[test]
fn windows_writers_materialize_ignored_untracked_faces() {
    let root = windows_temp_git_repo("writers");
    write_windows_ignore(&root);

    let writer = CanonicalRetirementFactsWriter::open(&root)
        .expect("windows retirement facts writer must open");
    writer
        .write(b"{\"facts\":true}")
        .expect("write retirement facts");
    let facts_path = root.join(GENERATED_FACTS_PATH);
    let facts = std::fs::read(&facts_path).expect("read materialized retirement facts");
    assert_eq!(facts, b"{\"facts\":true}");
    assert!(
        std::fs::symlink_metadata(&facts_path)
            .expect("metadata")
            .file_type()
            .is_file(),
        "retirement facts dest must be a regular file"
    );
    writer
        .write(b"{\"facts\":false}")
        .expect("replace retirement facts");
    assert_eq!(
        std::fs::read(&facts_path).expect("read replaced retirement facts"),
        b"{\"facts\":false}"
    );

    let ignored = CanonicalIgnoredGeneratedWriter::open(&root, Path::new(WINDOWS_IGNORED_RECEIPT))
        .expect("windows ignored generated writer must open");
    ignored
        .write(b"{}")
        .expect("write ignored generated receipt");
    let receipt_path = root.join(WINDOWS_IGNORED_RECEIPT);
    assert_eq!(
        std::fs::read(&receipt_path).expect("read materialized ignored receipt"),
        b"{}"
    );
    assert!(
        std::fs::symlink_metadata(&receipt_path)
            .expect("metadata")
            .file_type()
            .is_file(),
        "ignored generated dest must be a regular file"
    );
    ignored
        .write(b"{\"ok\":1}")
        .expect("replace ignored generated receipt");
    assert_eq!(
        std::fs::read(&receipt_path).expect("read replaced ignored receipt"),
        b"{\"ok\":1}"
    );

    std::fs::remove_dir_all(&root).expect("remove windows writer repository");
}

/// A symlink or junction parent component is rejected before any write.
#[test]
fn windows_writers_reject_symlink_or_junction_parent() {
    let root = windows_temp_git_repo("reparse");
    write_windows_ignore(&root);
    std::fs::create_dir(root.join("ci")).expect("create ci");
    let decoy = root.join("decoy-parent");
    std::fs::create_dir(&decoy).expect("create decoy");
    create_windows_directory_reparse(&root.join("ci").join("facade"), &decoy);

    let error = CanonicalRetirementFactsWriter::open(&root)
        .map(|_| ())
        .expect_err("junction parent must be rejected");
    assert!(
        error.contains("is not a real directory"),
        "unexpected junction error: {error}"
    );

    let error = CanonicalIgnoredGeneratedWriter::open(&root, Path::new(WINDOWS_IGNORED_RECEIPT))
        .map(|_| ())
        .expect_err("junction parent must be rejected for ignored writer");
    assert!(
        error.contains("is not a real directory"),
        "unexpected ignored-writer junction error: {error}"
    );

    std::fs::remove_dir_all(&root).expect("remove windows reparse repository");
}

/// Non-ignored and tracked outputs still fail closed at `open`.
#[test]
fn windows_writers_reject_non_ignored_or_tracked_path() {
    let root = windows_temp_git_repo("boundary");

    let error = CanonicalRetirementFactsWriter::open(&root)
        .map(|_| ())
        .expect_err("missing ignore must fail closed");
    assert!(
        error.contains("must be ignored and untracked"),
        "unexpected retirement boundary error: {error}"
    );

    let error = CanonicalIgnoredGeneratedWriter::open(&root, Path::new(WINDOWS_IGNORED_RECEIPT))
        .map(|_| ())
        .expect_err("missing ignore must fail closed for ignored writer");
    assert!(
        error.contains("must be ignored and untracked"),
        "unexpected ignored boundary error: {error}"
    );

    write_windows_ignore(&root);
    std::fs::create_dir_all(root.join("ci/facade/artifact-inventory-registry"))
        .expect("create receipt parent");
    std::fs::write(root.join(WINDOWS_IGNORED_RECEIPT), b"tracked").expect("write tracked receipt");
    let add = Command::new("git")
        .args(["add", "-f", "--", WINDOWS_IGNORED_RECEIPT])
        .current_dir(&root)
        .status()
        .expect("git add tracked receipt");
    assert!(add.success(), "git add -f must succeed");

    let error = CanonicalIgnoredGeneratedWriter::open(&root, Path::new(WINDOWS_IGNORED_RECEIPT))
        .map(|_| ())
        .expect_err("tracked ignored path must fail closed");
    assert!(
        error.contains("must be untracked"),
        "unexpected tracked-path error: {error}"
    );

    std::fs::remove_dir_all(&root).expect("remove windows boundary repository");
}
