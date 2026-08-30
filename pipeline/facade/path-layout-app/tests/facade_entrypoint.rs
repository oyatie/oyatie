//! The facade entry-point rule and its one-way ratchet (ADR-0719 D-30 amendment).

mod support;

use support::*;

#[test]
fn a_facade_may_be_staged_but_a_running_one_cannot_be_demoted() {
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

    // A surface whose listener is not attached roots at `src/lib.rs`. Before
    // the D-30 amendment this was refused, which made every staged facade in
    // the repository unedittable.
    write(
        &root,
        "network/facade/edge-app/Cargo.toml",
        "[package]\nname='network-edge-app'\nversion='0.1.0'\nedition='2024'\n",
    );
    write(
        &root,
        "network/facade/edge-app/src/lib.rs",
        "pub fn edge() {}\n",
    );
    let staged = commit(&root, "staged facade");
    assert!(
        admit(&root, &base, &staged).status.success(),
        "a facade rooted at src/lib.rs must admit"
    );

    // Attach the listener.
    write(
        &root,
        "network/facade/edge-app/src/main.rs",
        "fn main() {}\n",
    );
    let running = commit(&root, "attach listener");
    assert!(
        admit(&root, &staged, &running).status.success(),
        "a facade that gains src/main.rs must admit"
    );

    // The relaxation is one-way. Deleting the binary while lib.rs answers in
    // its place must NOT pass: that is a running service being demoted, not a
    // surface waiting to be composed.
    std::fs::remove_file(root.join("network/facade/edge-app/src/main.rs"))
        .expect("delete the entry point");
    let demoted = commit(&root, "delete the listener");
    let rejected = admit(&root, &running, &demoted);
    assert!(
        !rejected.status.success(),
        "deleting a facade's existing src/main.rs must be refused"
    );
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        error.contains("existed at the merge base and is absent at the head"),
        "expected the demotion ratchet, got: {error}"
    );
    let _ = std::fs::remove_dir_all(root);
}
