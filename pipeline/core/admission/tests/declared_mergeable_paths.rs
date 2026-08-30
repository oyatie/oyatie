//! The exemption set, checked against the repository's real `.gitattributes`.
//!
//! The unit tests for `declared_mergeable` use a synthetic fixture, which is
//! enough to pin the parser and nothing else. Two things it cannot catch: the
//! real file drifting away from the driver allowlist, and the whole feature
//! being inert. The first version of the trunk read shipped dead — it went
//! through an object-id validator that rejects any non-hex byte, so
//! `.gitattributes` failed at its first character and the exemption set was
//! always empty — while every unit test, the substring freeze on the call
//! site, clippy, fmt and the layout gate stayed green.

use std::path::{Path, PathBuf};

use pipeline_admission::declared_mergeable;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn attributes() -> String {
    std::fs::read_to_string(repo_root().join(".gitattributes")).expect("repository .gitattributes")
}

#[test]
fn the_real_gitattributes_declares_the_hub_files() {
    let declared = declared_mergeable(&attributes()).expect("the real file parses");
    assert!(
        declared.contains("Cargo.lock"),
        "the lockfile carries `merge=cargo-lock` and is the reason this rule \
         exists; if it stops being declared, every structural lane wedges again"
    );
    for ledger in [
        "evidence/audit-chain.jsonl",
        "registry/fixuptasks.jsonl",
        "ci/facade/action-item-accounting/friction-ledger.jsonl",
    ] {
        assert!(
            declared.contains(ledger),
            "{ledger} carries a merge driver and must be exempt"
        );
    }
}

#[test]
fn every_declared_driver_is_one_this_repository_wrote() {
    // Drift guard. A future `merge=` line naming a driver outside the allowlist
    // leaves its path occupancy-bearing, which is the safe direction but a
    // silent one — the author would see refusals and no explanation. Fail here
    // instead, at the place that decides.
    let text = attributes();
    let mut undeclared = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split_whitespace();
        let Some(pattern) = fields.next() else {
            continue;
        };
        if let Some(driver) = fields.find_map(|a| a.strip_prefix("merge=")) {
            let declared = declared_mergeable(&format!("{pattern} merge={driver}\n"));
            if declared.map(|set| set.is_empty()).unwrap_or(true) {
                undeclared.push(format!("{pattern} -> merge={driver}"));
            }
        }
    }
    assert!(
        undeclared.is_empty(),
        "`.gitattributes` names merge drivers occupancy does not recognise, so \
         these paths silently stay occupancy-bearing: {undeclared:?}"
    );
}
