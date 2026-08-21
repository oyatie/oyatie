//! cloud-ci-crate-catalog-coverage — every live first-party crate must carry a
//! `registry/catalog/<package-name>.yaml` row.
//!
//! ## Why this gate exists
//!
//! The service catalog is keyed by FILENAME, one file per crate. That single fact
//! makes it invisible to the search an agent naturally reaches for: the crate name
//! lives in the PATH, so `grep -r <crate-name>` over file CONTENTS never finds the
//! row. A crate move therefore strands its catalog row under the old package name
//! with nothing pointing at the problem until three unrelated born-blocking gates
//! fail downstream (`slo-coverage`, `catalog-liveness`, the ADR census receipt),
//! none of which names the missing row as the cause.
//!
//! That is exactly how PR #1437 went red: a crate was moved out of the legacy
//! `oya/` root, every code reference was repointed, the build and its consumer both
//! passed locally — and the move still failed CI because a YAML file 300 directories
//! away was named after the old crate.
//!
//! The existing catalog checks run the OTHER direction (a row whose crate is gone).
//! This one closes the loop: a crate with no row. Together they make the crate set
//! and the catalog set mutually total.
//!
//! ## Shape
//!
//! Born-blocking against a FROZEN, shrink-only baseline of the crates that lack a
//! row today. Pre-existing gaps are tolerated; a NEW uncatalogued crate fails. A
//! MOVED crate is a new package name, is therefore absent from the baseline, and
//! fails unless its catalog row moves in the same PR — which is the coupled edit
//! this gate exists to force.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

/// Gate identity, as registered in the cloud-ci gate fleet.
pub const GATE_ID: &str = "cloud-ci-crate-catalog-coverage";

/// A live crate has no `registry/catalog/<package-name>.yaml`.
pub const CODE_CRATE_WITHOUT_CATALOG_ROW: &str = "crate_without_catalog_row";
/// The frozen baseline names a crate that no longer lacks a row (or no longer
/// exists). Stale debt must shrink in the SAME change that fixes it, or the
/// baseline silently over-tolerates.
pub const CODE_STALE_BASELINE_ENTRY: &str = "catalog_coverage_stale_baseline_entry";
/// The observed corpus is implausibly small — a collection bug would otherwise
/// present as a clean pass. A gate that reports GREEN because it saw nothing is
/// the false-green this repo keeps re-learning.
pub const CODE_IMPLAUSIBLE_CORPUS: &str = "catalog_coverage_implausible_corpus";

/// Every code this gate can emit. Registered so the fleet meta-test can assert the
/// set is declared rather than discovered at runtime.
pub const VIOLATION_CODES: [&str; 3] = [
    CODE_CRATE_WITHOUT_CATALOG_ROW,
    CODE_STALE_BASELINE_ENTRY,
    CODE_IMPLAUSIBLE_CORPUS,
];

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
    /// The `registry/catalog/*.yaml` stems present.
    pub catalog_rows: BTreeSet<String>,
}

/// The frozen, shrink-only debt: crates known to lack a catalog row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Baseline {
    pub uncatalogued: BTreeSet<String>,
    /// Optional test-only floor. Live policy no longer carries a crate-count
    /// census; an empty scan still fails closed.
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
    /// Baselined crates that now HAVE a row — real burn-down, reported so the
    /// baseline can be shrunk deliberately rather than drifting.
    pub burned_down: BTreeSet<String>,
}

/// Evaluate coverage. Pure: no I/O, no clock, no environment.
///
/// A crate fails iff it has no catalog row AND is not in the frozen baseline.
/// A baseline entry that is no longer uncatalogued is stale and also fails, so
/// burn-down cannot be pocketed silently while the baseline keeps its slack.
pub fn evaluate(observed: &Observed, baseline: &Baseline) -> Report {
    let mut findings: Vec<Finding> = Vec::new();

    // FALSE-GREEN GUARD first: an empty collection must not read as full coverage.
    // A numeric crate-count census is not a coverage invariant.
    if observed.crates.is_empty() || observed.crates.len() < baseline.min_expected_crates {
        findings.push(Finding {
            code: CODE_IMPLAUSIBLE_CORPUS.to_owned(),
            subject: format!("{} crates", observed.crates.len()),
            detail: format!(
                "collected {} crates (empty scan, or below the optional test floor of {}); \
                 the collector is broken or the scan root moved — treat this as a gate \
                 failure, never as coverage",
                observed.crates.len(),
                baseline.min_expected_crates
            ),
        });
    }

    let mut burned_down: BTreeSet<String> = BTreeSet::new();

    for (name, manifest) in &observed.crates {
        if observed.catalog_rows.contains(name) {
            // Has a row. If it was baselined as missing one, that is burn-down.
            if baseline.uncatalogued.contains(name) {
                burned_down.insert(name.clone());
            }
            continue;
        }
        if baseline.uncatalogued.contains(name) {
            continue; // frozen, tolerated debt
        }
        findings.push(Finding {
            code: CODE_CRATE_WITHOUT_CATALOG_ROW.to_owned(),
            subject: name.clone(),
            detail: format!(
                "{manifest} declares package `{name}` but registry/catalog/{name}.yaml does not \
                 exist. The catalog is keyed by FILENAME, so a content search for the crate name \
                 will not find this. If you MOVED or RENAMED a crate, move its catalog row in the \
                 same change: `git mv registry/catalog/<old-name>.yaml registry/catalog/{name}.yaml` \
                 and update `role`/`capability` to match the destination."
            ),
        });
    }

    // Stale baseline entries: a crate that gained a row, or vanished entirely.
    for name in &baseline.uncatalogued {
        let still_missing =
            observed.crates.contains_key(name) && !observed.catalog_rows.contains(name);
        if still_missing {
            continue;
        }
        let reason = if observed.crates.contains_key(name) {
            "now HAS a catalog row (burn-down)"
        } else {
            "is no longer a live crate"
        };
        findings.push(Finding {
            code: CODE_STALE_BASELINE_ENTRY.to_owned(),
            subject: name.clone(),
            detail: format!(
                "the frozen baseline still lists `{name}`, but it {reason}. Remove the entry in \
                 this same change — a baseline that keeps slack it no longer needs silently \
                 tolerates the next regression that lands on that name."
            ),
        });
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
    fn uncatalogued_crate_fails_closed() {
        let o = observed(&[("a", "a/Cargo.toml"), ("b", "b/Cargo.toml")], &["a"]);
        let r = evaluate(&o, &baseline(&[]));
        assert_eq!(r.verdict, Verdict::Red);
        let f = &r.findings[0];
        assert_eq!(f.code, CODE_CRATE_WITHOUT_CATALOG_ROW);
        assert_eq!(f.subject, "b");
        // The remedy must be actionable without re-deriving the keying rule.
        assert!(f.detail.contains("keyed by FILENAME"));
        assert!(f.detail.contains("git mv registry/catalog/"));
    }

    #[test]
    fn baselined_gap_is_tolerated() {
        let o = observed(&[("a", "a/Cargo.toml"), ("b", "b/Cargo.toml")], &["a"]);
        let r = evaluate(&o, &baseline(&["b"]));
        assert_eq!(r.verdict, Verdict::Green);
    }

    /// THE MOVE CASE — the whole reason this gate exists. A crate renamed from
    /// `old-name` to `new-name` without moving its catalog row must fail, even
    /// though the OLD name was baselined debt. Baselined slack must not transfer
    /// to a new identity.
    #[test]
    fn moved_crate_cannot_inherit_the_old_names_baseline_slack() {
        let o = observed(&[("new-name", "dest/Cargo.toml")], &["old-name"]);
        let r = evaluate(&o, &baseline(&["old-name"]));
        assert_eq!(r.verdict, Verdict::Red);
        let codes: BTreeSet<&str> = r.findings.iter().map(|f| f.code.as_str()).collect();
        assert!(
            codes.contains(CODE_CRATE_WITHOUT_CATALOG_ROW),
            "the moved crate has no row of its own: {:?}",
            r.findings
        );
        assert!(
            codes.contains(CODE_STALE_BASELINE_ENTRY),
            "the old baseline entry is now stale: {:?}",
            r.findings
        );
    }

    /// The move done RIGHT is green: crate and row relocate together.
    #[test]
    fn move_with_the_row_co_moved_is_green() {
        let o = observed(&[("new-name", "dest/Cargo.toml")], &["new-name"]);
        let r = evaluate(&o, &baseline(&[]));
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
    }

    #[test]
    fn burn_down_is_reported_and_stale_entry_must_be_removed() {
        // `b` gained a row while still baselined: real progress, but the baseline
        // must shrink in the same change.
        let o = observed(&[("a", "a/Cargo.toml"), ("b", "b/Cargo.toml")], &["a", "b"]);
        let r = evaluate(&o, &baseline(&["b"]));
        assert_eq!(r.burned_down, ["b".to_owned()].into_iter().collect());
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_STALE_BASELINE_ENTRY);
    }

    #[test]
    fn baseline_entry_for_a_deleted_crate_is_stale() {
        let o = observed(&[("a", "a/Cargo.toml")], &["a"]);
        let r = evaluate(&o, &baseline(&["deleted-crate"]));
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_STALE_BASELINE_ENTRY);
        assert!(r.findings[0].detail.contains("no longer a live crate"));
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
    fn findings_are_deterministically_ordered() {
        let o = observed(
            &[
                ("z", "z/Cargo.toml"),
                ("a", "a/Cargo.toml"),
                ("m", "m/Cargo.toml"),
            ],
            &[],
        );
        let r = evaluate(&o, &baseline(&[]));
        let subjects: Vec<&str> = r.findings.iter().map(|f| f.subject.as_str()).collect();
        assert_eq!(subjects, vec!["a", "m", "z"], "output must be stable");
    }

    #[test]
    fn every_emitted_code_is_registered() {
        let o = observed(&[("uncovered", "u/Cargo.toml")], &[]);
        let b = Baseline {
            uncatalogued: ["gone".to_owned()].into_iter().collect(),
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
        // All three codes are reachable — no dead code in the registered set.
        let codes: BTreeSet<&str> = r.findings.iter().map(|f| f.code.as_str()).collect();
        assert_eq!(codes.len(), VIOLATION_CODES.len());
    }
}
