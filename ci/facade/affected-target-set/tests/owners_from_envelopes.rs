//! Integration tests for owners-from-envelopes against live envelopes SSOT.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use ci_affected_target_set::owners_from_envelopes::{
    EMIT_CODEOWNERS_RELPATH, EMIT_OWNERS_MAP_RELPATH, ENVELOPES_RELPATH, generate_owners,
    owners_map_json,
};

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(ENVELOPES_RELPATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (dir holding {ENVELOPES_RELPATH})");
}

#[test]
fn live_envelopes_generate_codeowners_and_owners_map() {
    let root = repo_root();
    let envelopes_path = root.join(ENVELOPES_RELPATH);
    if !envelopes_path.is_file() {
        eprintln!("skip: {ENVELOPES_RELPATH} absent on tip");
        return;
    }
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
