//! The reporter must actually RUN on the committed corpus, inside the required test job.
//!
//! The #2100 review was right that a gate which is built but never executed is not a gate. My first
//! answer was a GitHub Actions workflow, and that was wrong twice over: it put the execution outside
//! the required packet, and it introduced two inline-shell steps into workflow YAML — which the
//! `rust_first_automation_unbaselined_workflow_inline_shell` ratchet correctly rejected as new debt
//! ("productize it as a Rust/Buck2 step"). This is that step.
//!
//! Running it as a workspace test means `cargo test --locked --workspace` — the required
//! `test (workspace + gates)` job — exercises the reporter against the real mirror on every merge,
//! with no shell and no new workflow surface.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use ci_dep_freshness::{Policy, manifest, mirror, owner_index, stale_entries, verify};

/// Walk up from this crate to the repository root, identified by its root-hub marker.
fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        assert!(
            dir.pop(),
            "repository root marker not found above the crate"
        );
    }
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn the_reporter_runs_against_the_committed_mirror() {
    let policy = Policy::from_toml(&read("deps.toml")).expect("[freshness] policy parses");
    let manifest = manifest(&read(&policy.manifest)).expect("mirror manifest parses");
    let releases = mirror(&read(&policy.mirror)).expect("mirror parses");

    // The committed pair must agree. This is the check that makes a truncated or separately
    // regenerated mirror fail the required job rather than read as a clean corpus.
    verify(&releases, &manifest).expect("committed mirror matches its manifest");

    let owners = owner_index(&read("specs/oss-stewardship-registry.json"));
    let stale = stale_entries(&releases, &policy, &manifest.snapshot_date, &owners);

    assert!(
        !releases.is_empty(),
        "the committed mirror describes no crates"
    );
    assert!(
        stale.len() <= releases.len(),
        "more stale entries ({}) than crates in the corpus ({})",
        stale.len(),
        releases.len()
    );
    // Ordering is part of the contract: the report leads with the quietest dependency.
    for pair in stale.windows(2) {
        assert!(
            pair[0].days_since_release >= pair[1].days_since_release,
            "stale entries must be ordered quietest-first"
        );
    }
    println!(
        "DEP-FRESHNESS-STALE (advisory) — {} of {} direct dependencies quiet for over {} days, as of {}",
        stale.len(),
        releases.len(),
        policy.stale_after_days,
        manifest.snapshot_date
    );
}

#[test]
fn the_committed_policy_is_advisory() {
    // A blocking [freshness] policy would be refused by the gate rather than silently downgraded;
    // this pins the committed declaration so the refusal path is never reached by accident.
    let policy = Policy::from_toml(&read("deps.toml")).expect("policy parses");
    assert_eq!(policy.enforcement, "advisory");
    assert!(policy.stale_after_days > 0);
}

#[test]
fn every_mirror_entry_has_a_usable_release_date() {
    let policy = Policy::from_toml(&read("deps.toml")).expect("policy parses");
    let releases = mirror(&read(&policy.mirror)).expect("mirror parses");
    let owners = BTreeMap::new();
    // A record whose date cannot be parsed silently drops out of the staleness computation, which
    // would read as "not stale". Assert every committed date is well formed.
    for release in &releases {
        assert!(
            ci_dep_freshness::kernel::days_between(&release.last_release_date, "2026-08-17")
                .is_some(),
            "{} has an unparseable last_release_date {:?}",
            release.name,
            release.last_release_date
        );
    }
    let _ = stale_entries(&releases, &policy, "2026-08-17", &owners);
}
