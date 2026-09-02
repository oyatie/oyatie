//! Qualification of the PINNED Reindeer source, run only inside the
//! presubmit job that acquires it.
//!
//! The job checks out `facebookincubator/reindeer` at a pinned commit,
//! validates its commit and tree hashes, and hands this crate three
//! paths through the environment. Those hashes prove the checkout is the
//! revision we pinned; they say nothing about whether the source still
//! has the SHAPE this adapter will consume. That is what qualification
//! means before the adapter has behaviour, and it is what these assert.
//!
//! Ignored by default: without the job's checkout there is no pinned
//! source to qualify, and a test that silently passed on its absence
//! would be worse than no test. The job runs
//! `--run-ignored only --no-tests=fail`, so an empty selection is a
//! failure by design — the gate is fail-closed and this is its floor.
//!
//! FLOOR, NOT CEILING. This crate is Wave S — module discovery only,
//! with process behaviour deferred to the separately reviewed adapter
//! wave. When that wave lands, the qualification it needs is a
//! comparison against upstream's real generation behaviour, and these
//! assertions should grow into it rather than stay as they are.

use std::path::PathBuf;

fn required(variable: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(variable).unwrap_or_else(|| {
        panic!("{variable} is set by the qualification job; run this test through it")
    }))
}

#[test]
#[ignore = "requires the pinned Reindeer checkout the qualification job provides"]
fn the_pinned_reindeer_source_has_the_shape_this_adapter_consumes() {
    let root = required("REINDEER_PINNED_SOURCE_ROOT");
    assert!(
        root.is_dir(),
        "the pinned source root is a directory: {}",
        root.display()
    );

    // The generation surface this adapter is pinned against. A pin bump
    // that moved or renamed it would otherwise be discovered by the
    // adapter wave, at which point the pin is already merged.
    let buck = required("REINDEER_PINNED_BUCK_RS");
    let generation = std::fs::read_to_string(&buck)
        .unwrap_or_else(|error| panic!("read {}: {error}", buck.display()));
    assert!(
        !generation.trim().is_empty(),
        "the pinned generation surface is non-empty: {}",
        buck.display()
    );

    // The checkout is Reindeer itself, not merely some repository at the
    // pinned path — the tree hash proves WHICH revision, not WHAT it is.
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap_or_else(|error| panic!("read the pinned manifest: {error}"));
    assert!(
        manifest.contains("name = \"reindeer\""),
        "the pinned source is Reindeer: {manifest:.200}"
    );

    // The job isolates upstream compilation from the workspace target
    // dir; without this the qualification would silently share it.
    let target = required("REINDEER_QUALIFICATION_TARGET_DIR");
    assert!(
        target != root,
        "upstream compilation is isolated from the source root"
    );
}
