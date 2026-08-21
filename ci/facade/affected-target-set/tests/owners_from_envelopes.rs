//! Integration tests for owners-from-envelopes against live envelopes SSOT.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use ci_affected_target_set::owners_from_envelopes::{
    EMIT_CODEOWNERS_RELPATH, EMIT_OWNERS_MAP_RELPATH, ENVELOPES_RELPATH, generate_owners,
    owners_map_json,
};

/// Anchor on this gate's OWN directory, never on the subject under test.
///
/// The previous anchor was [`ENVELOPES_RELPATH`] itself, which made the root walk and the
/// input check the same predicate — so the `if !envelopes_path.is_file()` skip below was
/// unreachable, and, worse, in a nested checkout (a git worktree under the primary clone) the
/// walk climbed straight out of the worktree and bound the PARENT clone's copy of the file.
/// The gate then reported green over a tree it was not asked to judge. Anchoring on a marker
/// this crate owns keeps the walk inside the checkout under test and makes the absence of the
/// subject a real, RED, named failure.
const GATE_DIR_MARKER: &str = "ci/facade/affected-target-set/Cargo.toml";

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(GATE_DIR_MARKER).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (dir holding {GATE_DIR_MARKER})");
}

#[test]
fn live_envelopes_generate_codeowners_and_owners_map() {
    let root = repo_root();
    let envelopes_path = root.join(ENVELOPES_RELPATH);
    assert!(
        envelopes_path.is_file(),
        "declared owners SSOT {ENVELOPES_RELPATH} does not resolve under {} — CODEOWNERS and \
         the owners map are derived from it, so an absent authority must be RED and name the \
         path. If the SSOT moved, repoint ENVELOPES_RELPATH in the same change that moves it",
        root.display()
    );
    let text = fs::read_to_string(&envelopes_path).expect("read envelopes");
    let envelopes: serde_json::Value = serde_json::from_str(&text).expect("parse envelopes");
    assert!(
        envelopes.get("roots").is_some(),
        "expected #roots on envelopes"
    );
    // #path_ownership may land on integ/specs before integ/ci restacks — generator
    // must work from #roots/#planes alone (path_ownership is additive law).
    let generated = generate_owners(&envelopes).expect("generate from live envelopes");
    assert!(
        generated
            .codeowners
            .contains("* @teams/council-architecture"),
        "catch-all missing"
    );
    assert!(
        generated.codeowners.contains("compute/ @teams/compute"),
        "compute ownership missing:\n{}",
        generated.codeowners
    );
    assert!(
        generated.owners_by_prefix.contains_key("messaging/"),
        "messaging/ OWNERS prefix missing"
    );
    assert!(
        generated.owners_by_prefix.len() >= 50,
        "expected most envelope dir prefixes; got {}",
        generated.owners_by_prefix.len()
    );
    let _map = owners_map_json(&generated.owners_by_prefix).expect("owners map json");
}

#[test]
fn emit_relpaths_are_under_ci_facade_package() {
    assert!(EMIT_CODEOWNERS_RELPATH.starts_with("ci/facade/affected-target-set/"));
    assert!(EMIT_OWNERS_MAP_RELPATH.starts_with("ci/facade/affected-target-set/"));
}
