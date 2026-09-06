mod support;

use support::*;

#[test]
fn minimum_version_changes_require_compatible_execution_not_patch_only_policy() {
    let root = fixture();
    let base = commit(&root, "protected declarations");
    write(
        &root,
        "Cargo.toml",
        &workspace_manifest(&[]).replace("1.98.0", "1.99.0"),
    );
    write(
        &root,
        "rust-toolchain.toml",
        &valid_toolchain().replace("1.98.0", "1.99.0"),
    );
    let candidate = commit(&root, "compatible minimum update");
    let result = admit(&root, &base, &candidate);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn malformed_declarations_still_fail_compatibility_admission() {
    let root = fixture();
    let base = commit(&root, "protected declarations");
    write(&root, "rust-toolchain.toml", "[toolchain\n");
    let candidate = commit(&root, "malformed declaration");
    let result = admit(&root, &base, &candidate);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("execution toolchain analysis refused")
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn declaration_compatibility_does_not_impose_patch_only_promotion() {
    let root = fixture();
    let base = commit(&root, "protected toolchain");

    write(
        &root,
        "rust-toolchain.toml",
        "[toolchain]\nchannel = '1.98.1'\ncomponents = ['rustfmt', 'clippy']\nprofile = 'minimal'\n",
    );
    let patch = commit(&root, "forward patch");
    let admitted = admit(&root, &base, &patch);
    assert!(
        admitted.status.success(),
        "{}",
        String::from_utf8_lossy(&admitted.stderr)
    );

    git(&root, &["reset", "--hard", &base]);
    write(
        &root,
        "rust-toolchain.toml",
        "[toolchain]\nchannel = '1.99.0'\ncomponents = ['rustfmt', 'clippy']\nprofile = 'minimal'\n",
    );
    let minor = commit(&root, "minor declaration");
    let admitted = admit(&root, &base, &minor);
    assert!(
        admitted.status.success(),
        "{}",
        String::from_utf8_lossy(&admitted.stderr)
    );

    write(
        &root,
        "rust-toolchain.toml",
        "[toolchain]\nchannel = '1.97.0'\ncomponents = ['rustfmt', 'clippy']\nprofile = 'minimal'\n",
    );
    let incompatible = commit(&root, "execution below minimum");
    let refused = admit(&root, &base, &incompatible);
    assert!(!refused.status.success());
    let error = String::from_utf8_lossy(&refused.stderr);
    assert!(
        error.contains("execution toolchain 1.97.0 is below MSRV 1.98.0"),
        "{error}"
    );
    let _ = std::fs::remove_dir_all(root);
}
