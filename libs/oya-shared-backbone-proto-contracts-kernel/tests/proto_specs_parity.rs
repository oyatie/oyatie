//! Drift guard: the crate-local proto copies (required for hermetic buck2
//! `include_str!`) must stay byte-identical to the canonical sources under
//! `specs/proto/backbone/`. Canonical home is `specs/proto/`; the crate-local
//! files are hermeticity copies until the api-contract-ssot emitters own both
//! as generated projections (ADR-0536 §gateway/SSOT).
//!
//! Runs under the cargo CI lane (repo checkout present). Inside the buck2
//! hermetic sandbox the canonical tree is unreachable; pairs are then skipped
//! loudly rather than passed silently — enforcement comes from the cargo lane.

use std::path::{Path, PathBuf};

const PAIRS: &[(&str, &str)] = &[
    (
        "contracts/backbone/community/community_post_store.proto",
        "specs/proto/backbone/community/community_post_store.proto",
    ),
    (
        "contracts/backbone/mail/mail.proto",
        "specs/proto/backbone/mail/mail.proto",
    ),
    (
        "contracts/backbone/messenger/message_stream.proto",
        "specs/proto/backbone/messenger/message_stream.proto",
    ),
    (
        "contracts/backbone/social/social_post_composition.proto",
        "specs/proto/backbone/social/social_post_composition.proto",
    ),
];

/// Walk upward from the crate manifest dir to the repo root, identified by the
/// root-hub marker (marker-file discovery, not a fixed parent count — see
/// FRIC-009: fixed-depth `repo_root()` helpers rot when trees move).
fn repo_root() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[test]
fn crate_local_protos_match_canonical_specs_proto() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(root) = repo_root() else {
        eprintln!(
            "proto_specs_parity: repo root marker not reachable (hermetic sandbox); \
             skipped {} pairs — cargo CI lane enforces parity",
            PAIRS.len()
        );
        return;
    };
    let mut mismatches = Vec::new();
    for (local, canonical) in PAIRS {
        let local_bytes = std::fs::read(crate_dir.join(local))
            .unwrap_or_else(|e| panic!("crate-local proto missing: {local}: {e}"));
        let canonical_path = root.join(canonical);
        let canonical_bytes = std::fs::read(&canonical_path).unwrap_or_else(|e| {
            panic!("canonical proto missing: {}: {e}", canonical_path.display())
        });
        if local_bytes != canonical_bytes {
            mismatches.push(format!("{local} != {canonical}"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "crate-local proto copies drifted from canonical specs/proto sources \
         (canonical wins; sync the crate copy in the same change):\n  {}",
        mismatches.join("\n  ")
    );
}
