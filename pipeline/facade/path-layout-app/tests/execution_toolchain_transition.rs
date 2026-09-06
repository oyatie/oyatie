mod support;

use support::*;

#[test]
fn candidate_toolchain_mutation_is_checked_before_execution() {
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
    let minor = commit(&root, "unqualified minor");
    let refused = admit(&root, &base, &minor);
    assert!(!refused.status.success());
    let error = String::from_utf8_lossy(&refused.stderr);
    assert!(
        error.contains("execution toolchain patch-only policy refused"),
        "{error}"
    );
    assert!(
        error.contains("ForwardMinor execution transition from 1.98.0 to 1.99.0"),
        "{error}"
    );
    let _ = std::fs::remove_dir_all(root);
}
