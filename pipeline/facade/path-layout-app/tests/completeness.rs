//! End-to-end completeness checks against committed Git trees.

mod support;

use support::*;

#[test]
fn touched_face_leaf_must_be_complete_unless_fully_deleted() {
    let root = fixture();
    write(
        &root,
        "network/core/existing/Cargo.toml",
        "[package]\nname='network-existing'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        "network/core/existing/src/lib.rs",
        "pub fn existing() {}\n",
    );
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

#[cfg(unix)]
#[test]
fn tracked_symlink_cannot_disguise_an_owner_draft_dependency() {
    use std::os::unix::fs::symlink;

    let root = fixture();
    let base = commit(&root, "base");
    write(
        &root,
        "storage/ports/draft/blob/Cargo.toml",
        "[package]\nname='storage-blob-draft'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        "storage/ports/draft/blob/src/lib.rs",
        "pub fn blob() {}\n",
    );
    write(
        &root,
        "network/core/route/Cargo.toml",
        "[package]\nname='network-route'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nblob={path='src/blob.rs'}\n",
    );
    write(
        &root,
        "network/core/route/src/lib.rs",
        "pub fn route() {}\n",
    );
    symlink(
        "../../../../storage/ports/draft/blob",
        root.join("network/core/route/src/blob.rs"),
    )
    .expect("create dependency-path symlink");
    let disguised = commit(&root, "disguised draft dependency");

    let rejected = admit(&root, &base, &disguised);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        error.contains("dependency `blob` has unsafe path `src/blob.rs`"),
        "{error}"
    );
    assert!(
        error.contains("tracked symlink component `network/core/route/src/blob.rs`"),
        "{error}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn trusted_sha_arguments_cannot_be_overridden_by_cargo_environment() {
    let root = fixture();
    let base = commit(&root, "base");
    write(&root, "plan/forbidden.md", "must not admit\n");
    let head = commit(&root, "forbidden path");

    let rejected = admit(&root, &base, &head);
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("frozen non-root Markdown"),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn modified_and_type_changed_frozen_markdown_refuse_through_the_facade() {
    use std::os::unix::fs::symlink;

    let root = fixture();
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
    write(&root, "policy/legacy.md", "legacy\n");
    write(&root, "policy/blob-to-link.md", "legacy.md");
    symlink("legacy.md", root.join("policy/link-to-blob.md")).expect("create base symlink");
    let base = commit(&root, "base with frozen Markdown modes");

    write(&root, "policy/legacy.md", "modified\n");
    std::fs::remove_file(root.join("policy/blob-to-link.md")).expect("remove base blob");
    symlink("legacy.md", root.join("policy/blob-to-link.md")).expect("replace blob with symlink");
    std::fs::remove_file(root.join("policy/link-to-blob.md")).expect("remove base symlink");
    write(&root, "policy/link-to-blob.md", "legacy.md");
    let head = commit(&root, "modify and change frozen Markdown modes");

    let rejected = admit(&root, &base, &head);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    for path in [
        "policy/legacy.md",
        "policy/blob-to-link.md",
        "policy/link-to-blob.md",
    ] {
        assert!(error.contains(path), "{path}: {error}");
    }
    assert!(
        error.contains("frozen non-root Markdown cannot be changed or used as a copy source"),
        "{error}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn exact_root_markdown_exceptions_admit_through_the_facade() {
    let root = fixture();
    for path in ["README.md", "AGENTS.md", "CLAUDE.md"] {
        write(&root, path, "base\n");
    }
    let base = commit(&root, "base with root Markdown");

    for path in ["README.md", "AGENTS.md", "CLAUDE.md"] {
        write(&root, path, "modified\n");
    }
    let head = commit(&root, "modify root Markdown");

    let admitted = admit(&root, &base, &head);
    assert!(
        admitted.status.success(),
        "{}",
        String::from_utf8_lossy(&admitted.stderr)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn workspace_excludes_cannot_hide_a_member_from_workspace_tests() {
    let root = fixture();
    let base = commit(&root, "base");
    write(
        &root,
        "Cargo.toml",
        &workspace_manifest(&["network/core/route"]),
    );
    let head = commit(&root, "hide workspace member");

    let rejected = admit(&root, &base, &head);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(error.contains("unexpected workspace exclude `network/core/route`"));
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn canonical_entrypoint_must_be_a_regular_git_blob() {
    use std::os::unix::fs::symlink;

    let root = fixture();
    let base = commit(&root, "base");
    write(
        &root,
        "storage/core/blob/Cargo.toml",
        "[package]\nname='storage-blob'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(&root, "storage/core/blob/src/lib.rs", "pub fn blob() {}\n");
    write(
        &root,
        "network/core/route/Cargo.toml",
        "[package]\nname='network-route'\nversion='0.1.0'\nedition='2024'\n",
    );
    std::fs::create_dir_all(root.join("network/core/route/src"))
        .expect("create consumer source directory");
    symlink(
        "../../../../storage/core/blob/src/lib.rs",
        root.join("network/core/route/src/lib.rs"),
    )
    .expect("create cross-owner entrypoint symlink");
    let head = commit(&root, "symlinked entrypoint");

    let rejected = admit(&root, &base, &head);
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(error.contains("canonical entry point"), "{error}");
    assert!(error.contains("regular Git blob"), "{error}");
    let _ = std::fs::remove_dir_all(root);
}
