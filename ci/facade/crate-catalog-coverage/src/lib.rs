//! cloud-ci-crate-catalog-coverage — catalog YAML is optional census metadata.
//!
//! ## Why this gate exists
//!
//! Membership is the Cargo workspace (and the closed capability registry for
//! capabilities). `registry/catalog/<package-name>.yaml` is NOT a per-crate census:
//! a missing row is not born-blocking. Hyperscaler does not require one YAML file
//! per package.
//!
//! Extra YAML (a row whose crate is gone) is ignored here. Unmarked stale rows
//! stay the catalog-liveness (`service-catalog-parity`) gate's job, which can read
//! explicit non-live markers. Filename-only comparison cannot tell a retired row
//! from a forgotten one.
//!
//! ## Shape
//!
//! The only RED this evaluator emits is an implausible crate corpus — a collection
//! bug must not present as a clean pass. Observed row counts are reported, never
//! required.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

/// Gate identity, as registered in the cloud-ci gate fleet.
pub const GATE_ID: &str = "cloud-ci-crate-catalog-coverage";

/// The observed corpus is implausibly small — a collection bug would otherwise
/// present as a clean pass. A gate that reports GREEN because it saw nothing is
/// the false-green this repo keeps re-learning.
pub const CODE_IMPLAUSIBLE_CORPUS: &str = "catalog_coverage_implausible_corpus";

/// Every code this gate can emit. Registered so the fleet meta-test can assert the
/// set is declared rather than discovered at runtime.
pub const VIOLATION_CODES: [&str; 1] = [CODE_IMPLAUSIBLE_CORPUS];

/// A single gate finding: the code, the subject it is about, and enough detail to
/// act without re-deriving anything.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Finding {
    pub code: String,
    pub subject: String,
    pub detail: String,
}

/// What the collector observed on the live tree. Pure DATA so the evaluator needs
/// no filesystem — the whole decision surface is testable without one.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Observed {
    /// package name -> the manifest path that declares it.
    pub crates: BTreeMap<String, String>,
    /// The `registry/catalog/*.yaml` stems present (optional census, not membership).
    pub catalog_rows: BTreeSet<String>,
}

/// Floor on the observed crate count. Collecting fewer than this means the
/// collector broke, not that the repo shrank by hundreds of crates.
///
/// `uncatalogued` is retained as policy DATA so the committed baseline file can
/// stay put; it is not a born-blocking set. Missing YAML is not a finding.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Baseline {
    pub uncatalogued: BTreeSet<String>,
    pub min_expected_crates: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub verdict: Verdict,
    pub findings: Vec<Finding>,
    pub crates_checked: usize,
    pub rows_checked: usize,
    /// Crates that now HAVE a row while still listed in the historical uncatalogued
    /// set — informational burn-down, never a verdict input.
    pub burned_down: BTreeSet<String>,
}

/// Evaluate coverage. Pure: no I/O, no clock, no environment.
///
/// Catalog YAML is optional. A crate without a row is GREEN. Extra YAML is ignored.
/// Only an implausibly small crate corpus is RED.
pub fn evaluate(observed: &Observed, baseline: &Baseline) -> Report {
    let mut findings: Vec<Finding> = Vec::new();

    // FALSE-GREEN FLOOR first: if the corpus is implausible, every other verdict
    // below is meaningless, so say so loudly rather than reporting a clean pass.
    if observed.crates.len() < baseline.min_expected_crates {
        findings.push(Finding {
            code: CODE_IMPLAUSIBLE_CORPUS.to_owned(),
            subject: format!("{} crates", observed.crates.len()),
            detail: format!(
                "collected {} crates, below the floor of {}; the collector is broken or the \
                 scan root moved — treat this as a gate failure, never as coverage",
                observed.crates.len(),
                baseline.min_expected_crates
            ),
        });
    }

    let mut burned_down: BTreeSet<String> = BTreeSet::new();
    for name in observed.crates.keys() {
        if observed.catalog_rows.contains(name) && baseline.uncatalogued.contains(name) {
            burned_down.insert(name.clone());
        }
    }

    findings.sort();
    let verdict = if findings.is_empty() {
        Verdict::Green
    } else {
        Verdict::Red
    };
    Report {
        verdict,
        findings,
        crates_checked: observed.crates.len(),
        rows_checked: observed.catalog_rows.len(),
        burned_down,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observed(crates: &[(&str, &str)], rows: &[&str]) -> Observed {
        Observed {
            crates: crates
                .iter()
                .map(|(n, p)| ((*n).to_owned(), (*p).to_owned()))
                .collect(),
            catalog_rows: rows.iter().map(|r| (*r).to_owned()).collect(),
        }
    }

    fn baseline(uncatalogued: &[&str]) -> Baseline {
        Baseline {
            uncatalogued: uncatalogued.iter().map(|s| (*s).to_owned()).collect(),
            min_expected_crates: 0,
        }
    }

    #[test]
    fn fully_covered_corpus_is_green() {
        let o = observed(&[("a", "a/Cargo.toml"), ("b", "b/Cargo.toml")], &["a", "b"]);
        let r = evaluate(&o, &baseline(&[]));
        assert_eq!(r.verdict, Verdict::Green);
        assert!(r.findings.is_empty());
        assert_eq!(r.crates_checked, 2);
    }

    #[test]
    fn uncatalogued_crate_is_not_born_blocking() {
        let o = observed(&[("a", "a/Cargo.toml"), ("b", "b/Cargo.toml")], &["a"]);
        let r = evaluate(&o, &baseline(&[]));
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
        assert!(r.findings.is_empty());
    }

    #[test]
    fn extra_yaml_for_a_gone_crate_is_ignored_here() {
        // Catalog-liveness owns unmarked stale rows (it can read markers).
        let o = observed(&[("a", "a/Cargo.toml")], &["a", "gone-crate"]);
        let r = evaluate(&o, &baseline(&[]));
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
        assert_eq!(r.rows_checked, 2);
    }

    #[test]
    fn moved_crate_without_a_row_is_green() {
        let o = observed(&[("new-name", "dest/Cargo.toml")], &["old-name"]);
        let r = evaluate(&o, &baseline(&["old-name"]));
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
    }

    #[test]
    fn move_with_the_row_co_moved_is_green() {
        let o = observed(&[("new-name", "dest/Cargo.toml")], &["new-name"]);
        let r = evaluate(&o, &baseline(&[]));
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
    }

    #[test]
    fn burn_down_is_reported_but_does_not_fail() {
        let o = observed(&[("a", "a/Cargo.toml"), ("b", "b/Cargo.toml")], &["a", "b"]);
        let r = evaluate(&o, &baseline(&["b"]));
        assert_eq!(r.burned_down, ["b".to_owned()].into_iter().collect());
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
    }

    #[test]
    fn baseline_entry_for_a_deleted_crate_is_not_a_finding() {
        let o = observed(&[("a", "a/Cargo.toml")], &["a"]);
        let r = evaluate(&o, &baseline(&["deleted-crate"]));
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
    }

    /// FALSE-GREEN FLOOR: an empty collection must not read as full coverage.
    #[test]
    fn implausible_corpus_fails_rather_than_reporting_clean() {
        let o = observed(&[], &[]);
        let b = Baseline {
            uncatalogued: BTreeSet::new(),
            min_expected_crates: 500,
        };
        let r = evaluate(&o, &b);
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_IMPLAUSIBLE_CORPUS);
    }

    #[test]
    fn every_emitted_code_is_registered() {
        let o = observed(&[], &[]);
        let b = Baseline {
            uncatalogued: BTreeSet::new(),
            min_expected_crates: 500,
        };
        let r = evaluate(&o, &b);
        for f in &r.findings {
            assert!(
                VIOLATION_CODES.contains(&f.code.as_str()),
                "unregistered code {}",
                f.code
            );
        }
        let codes: BTreeSet<&str> = r.findings.iter().map(|f| f.code.as_str()).collect();
        assert_eq!(codes.len(), VIOLATION_CODES.len());
    }
}
