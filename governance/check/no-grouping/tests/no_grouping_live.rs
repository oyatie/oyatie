//! Live-tree caller for the no-grouping fitness gate (ADR-0362, restated live by ADR-0709 as
//! the "ADR-362 residual"; closes the ADR-0132 aspirational gap CLAUDE.md names directly as
//! `no_grouping_policy`).
//!
//! The kernel is pure; this walks the REAL `specs/microservices/` corpus and hands it
//! observations as DATA. There is no frozen violation set to maintain: `RETIRING_WRAPPERS` is a
//! CLOSED, empty allowlist (per the kernel's own doc comment, "Connect/Enterprise/Healthcare
//! package views are derived from tenant/RBAC entitlements outside `specs/microservices/`"), so
//! the doctrine's steady state is exactly zero grouping-shaped files, always. A single match of
//! any kind is a violation -- there is nothing to two-sided-freeze because nothing is tolerated.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use check_no_grouping::{GroupingArtifact, is_grouping_artifact, validate_no_grouping};

const MARKER: &str = "specs/microservices/manifest-schema.json";
const SCOPE_DIR: &str = "specs/microservices";

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(MARKER).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (the dir holding {MARKER})");
}

/// Every tracked `*.json` directly under `specs/microservices/` (not `scorecards/` or other
/// subdirectories -- the doctrine's scope is product-grouping wrappers, which live flat in this
/// directory; a scorecard is a different artifact class entirely and never matches the suffix
/// predicate below anyway, so excluding the subdirectory is belt-and-suspenders, not load-bearing).
fn live_specs(root: &Path) -> Vec<PathBuf> {
    let dir = root.join(SCOPE_DIR);
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("read_dir {}: {error}", dir.display()));
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
}

fn to_artifact(path: &Path) -> GroupingArtifact {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .expect("utf8 file name")
        .to_owned();
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let value: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
    let status = value
        .pointer("/_meta/status")
        .and_then(|v| v.as_str())
        .map(str::to_owned);
    let has_retirement_ref = value
        .pointer("/_meta/retirement_ref")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty());
    GroupingArtifact {
        file_name,
        status,
        has_retirement_ref,
    }
}

#[test]
fn live_corpus_has_no_grouping_artifacts() {
    let root = repo_root();
    let specs = live_specs(&root);

    // Anti-vacuity floor. A collapsed walk (the scope dir emptied, renamed, or read_dir
    // silently returning nothing) must not read as "doctrine satisfied" -- the steady state
    // (zero grouping-shaped files) is indistinguishable from a broken scan unless the corpus
    // itself is proven non-empty first. 25, not the live 33: loose enough to survive one file
    // being added or retired without a re-freeze, tight enough to catch a real collapse.
    const MIN_EXPECTED_SPECS: usize = 25;
    assert!(
        specs.len() >= MIN_EXPECTED_SPECS,
        "observed {} tracked specs/microservices/*.json, below the floor of {MIN_EXPECTED_SPECS} \
         -- the walk collapsed or the scope directory moved",
        specs.len()
    );

    let grouping_shaped: Vec<GroupingArtifact> = specs
        .iter()
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_grouping_artifact)
        })
        .map(|path| to_artifact(path))
        .collect();

    if let Err(error) = validate_no_grouping(grouping_shaped) {
        panic!(
            "ADR-0362 violation: {error:?}\n\
             `specs/microservices/*-suite.json` / `*-module.json` / `*-family.json` / \
             `*-bundle.json` are retired as architecture artifacts; the RETIRING_WRAPPERS \
             allowlist is closed. A grouping-shaped file must either be renamed to a flat \
             single-concern spec, or its introduction reverted -- there is no deprecation path \
             left, per ADR-0362 Decision #2 as closed out by this kernel's own \
             RETIRING_WRAPPERS = &[]."
        );
    }
}

#[test]
fn the_two_named_retiring_wrappers_from_adr_0362_do_not_match_the_suffix_predicate() {
    // ADR-0362's text names `tenant-rbac.json` / `tenant-rbac-packaging.json` as the two known
    // wrappers a grandfather clause once covered. Neither matches this kernel's suffix
    // predicate (`-suite/-module/-family/-bundle.json`), so RETIRING_WRAPPERS being closed is
    // consistent, not a silent narrowing: the kernel was never going to see them. Regression
    // guard against a future edit accidentally widening the predicate to catch them by surprise.
    assert!(!is_grouping_artifact("tenant-rbac.json"));
    assert!(!is_grouping_artifact("tenant-rbac-packaging.json"));
}
