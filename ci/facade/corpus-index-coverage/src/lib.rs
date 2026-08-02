//! # cloud-ci-corpus-index-coverage — the northstar as a burn-down invariant
//!
//! "Everything in the code graph and the build graph for full visibility" is a slogan until a
//! number is attached to it and that number is only allowed to move one way. This gate attaches the
//! number.
//!
//! ## What it measures
//! A buck2 package (a directory holding a `BUCK` file) that OWNS at least one YAML file is either
//! INDEXED — it declares a `corpus-yaml-facts` extraction target, so its YAML is a build-graph
//! input whose facts are a build output — or UNCOVERED.
//!
//! `coverage = indexed / total`, COMPUTED from the observed corpus. Nothing here is asserted: the
//! caller supplies observations, the kernel counts them.
//!
//! ## Why it ratchets rather than blocks
//! Born-ADVISORY, shrink-only. Today almost every YAML-owning package is uncovered, so a blocking
//! gate would be permanently red and would be switched off within a week. Instead the current
//! uncovered count is frozen as a ceiling: existing debt is reported, and a NEW uncovered package
//! is a REGRESSION that fails closed. The ceiling is lowered as extraction targets land, so the
//! slogan becomes a burn-down.
//!
//! ## The anti-vacuity rule, which is the important one
//! The dangerous failure of a coverage gate is not a false red, it is a walk that silently sees
//! nothing: zero YAML packages observed means zero uncovered, which reads as PERFECT COVERAGE. A
//! suspiciously total number means the probe is broken until proven otherwise, so
//! `min_expected_yaml_packages` fails the gate closed when the observed corpus collapses.
//!
//! PURE: no I/O, no clock, no rand. The caller walks the tree and passes observations as DATA.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// The gate's stable identifier.
pub const GATE_ID: &str = "cloud-ci-corpus-index-coverage";

/// A NEW YAML-owning package that declares no extraction target, pushing the uncovered count above
/// the frozen ceiling. Blocking.
pub const CODE_COVERAGE_REGRESSION: &str = "corpus_index_coverage_regression";

/// The observed corpus collapsed below the expected floor — the walk is broken, and its "no
/// uncovered packages" result is meaningless. Blocking.
pub const CODE_VACUOUS_SCAN: &str = "corpus_index_scan_vacuous";

/// A YAML-owning package with no extraction target, within the frozen ceiling. Advisory: this is
/// the debt being burned down.
pub const CODE_UNCOVERED_PACKAGE: &str = "corpus_index_uncovered_package";

/// MORE YAML files now live outside every buck2 package than the frozen ceiling allows. Blocking.
///
/// This is the northstar ratchet. A file in no package cannot be indexed at all, and the fix is to
/// pull it INTO the build graph — never to index it through a side channel, which would let it stay
/// outside forever while the coverage number improved.
pub const CODE_UNPACKAGED_REGRESSION: &str = "corpus_index_unpackaged_regression";

/// The frozen ceiling is higher than the observed uncovered count — the ratchet has slack and
/// should be lowered so it keeps biting. Advisory.
pub const CODE_STALE_CEILING: &str = "corpus_index_stale_ceiling";

/// One observed buck2 package that owns at least one YAML file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageObservation {
    /// Repo-relative directory of the package (the dir holding its `BUCK` file).
    pub package: String,
    /// How many YAML files this package owns (files whose NEAREST ancestor `BUCK` is this one).
    pub yaml_files: usize,
    /// Does the package declare a `corpus-yaml-facts` extraction target?
    pub indexed: bool,
}

/// The frozen policy. All repo-specifics are DATA: another repo adopts this gate by repointing it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    /// The shrink-only ceiling: observed uncovered packages may not exceed this.
    pub baseline_uncovered_packages: usize,
    /// The northstar ceiling: YAML files outside every buck2 package may not exceed this.
    pub baseline_unpackaged_yaml_files: usize,
    /// Anti-vacuity floor: fewer observed YAML-owning packages than this means a broken walk.
    pub min_expected_yaml_packages: usize,
    /// Anti-vacuity floor on FILES. A walk that finds packages but no files is equally broken, and
    /// would report a shrinking unpackaged count that is pure measurement collapse.
    pub min_expected_yaml_files: usize,
}

/// Computed coverage over the observed corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    /// YAML-owning packages observed.
    pub total_packages: usize,
    /// Of those, how many declare an extraction target.
    pub indexed_packages: usize,
    /// `total_packages - indexed_packages`. The number the ratchet drives to zero.
    pub uncovered_packages: usize,
    /// EVERY tracked YAML file in the repo.
    ///
    /// The denominator deliberately includes files that belong to no buck2 package. Counting only
    /// in-package files would let the gate report flawless coverage while most of the corpus sat
    /// outside the build graph — the exact false green this gate exists to prevent.
    pub total_yaml_files: usize,
    /// YAML files owned by INDEXED packages — the files actually reaching the graph.
    pub indexed_yaml_files: usize,
    /// YAML files that belong to NO buck2 package, and so cannot be indexed at all today.
    /// Structurally the largest term, and the one the northstar ratchet drives down.
    pub unpackaged_yaml_files: usize,
}

impl Coverage {
    /// Indexed packages per ten-thousand, as an INTEGER.
    ///
    /// Basis points, never a float: a float in a gate verdict is a formatting hazard in every
    /// serialized artifact downstream, and integer bps carries all the precision anyone reads.
    /// Returns 0 when nothing was observed — a vacuous scan reports no coverage, never 100%.
    #[must_use]
    pub const fn package_coverage_bps(&self) -> u32 {
        if self.total_packages == 0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            ((self.indexed_packages * 10_000) / self.total_packages) as u32
        }
    }

    /// Indexed YAML FILES per ten-thousand, as an integer. Packages vary wildly in how much YAML
    /// they own, so the file-level number is the honest view of how much corpus is really visible.
    #[must_use]
    pub const fn file_coverage_bps(&self) -> u32 {
        if self.total_yaml_files == 0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            ((self.indexed_yaml_files * 10_000) / self.total_yaml_files) as u32
        }
    }
}

/// One gate finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// The violation code.
    pub code: String,
    /// The package the finding is about (empty for corpus-wide findings).
    pub package: String,
    /// Human-readable detail.
    pub detail: String,
    /// Does this finding fail the gate?
    pub blocking: bool,
}

/// The gate verdict: the computed coverage plus every finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    /// Computed coverage over the observations.
    pub coverage: Coverage,
    /// All findings, blocking and advisory.
    pub findings: Vec<Finding>,
}

impl Verdict {
    /// Does the gate fail? True iff any finding is blocking.
    #[must_use]
    pub fn failed(&self) -> bool {
        self.findings.iter().any(|finding| finding.blocking)
    }

    /// Only the blocking findings.
    #[must_use]
    pub fn blocking(&self) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.blocking).collect()
    }
}

/// Compute coverage over the observed packages. Pure counting — no policy involved.
///
/// `unpackaged_yaml_files` is the count of tracked YAML files belonging to NO buck2 package. It is
/// a separate argument rather than derivable from `observations` precisely because those files have
/// no package to be observed under, which is the whole problem.
#[must_use]
pub fn coverage(observations: &[PackageObservation], unpackaged_yaml_files: usize) -> Coverage {
    let total_packages = observations.len();
    let indexed_packages = observations.iter().filter(|o| o.indexed).count();
    let packaged_yaml_files: usize = observations.iter().map(|o| o.yaml_files).sum();
    Coverage {
        total_packages,
        indexed_packages,
        uncovered_packages: total_packages - indexed_packages,
        total_yaml_files: packaged_yaml_files + unpackaged_yaml_files,
        indexed_yaml_files: observations
            .iter()
            .filter(|o| o.indexed)
            .map(|o| o.yaml_files)
            .sum(),
        unpackaged_yaml_files,
    }
}

/// Evaluate the observed corpus against the frozen policy.
#[must_use]
pub fn evaluate(
    observations: &[PackageObservation],
    unpackaged_yaml_files: usize,
    policy: &Policy,
) -> Verdict {
    let coverage = coverage(observations, unpackaged_yaml_files);
    let mut findings = Vec::new();

    // Anti-vacuity FIRST: every other verdict below is meaningless if the walk saw nothing, and a
    // broken walk otherwise presents as flawless coverage.
    if coverage.total_packages < policy.min_expected_yaml_packages {
        findings.push(Finding {
            code: CODE_VACUOUS_SCAN.to_owned(),
            package: String::new(),
            detail: format!(
                "observed only {} YAML-owning packages, expected at least {} — the walk is broken, \
                 so its coverage result is not evidence",
                coverage.total_packages, policy.min_expected_yaml_packages
            ),
            blocking: true,
        });
    }
    if coverage.total_yaml_files < policy.min_expected_yaml_files {
        findings.push(Finding {
            code: CODE_VACUOUS_SCAN.to_owned(),
            package: String::new(),
            detail: format!(
                "observed only {} YAML files, expected at least {} — a collapsed file census makes \
                 the unpackaged count shrink for the wrong reason",
                coverage.total_yaml_files, policy.min_expected_yaml_files
            ),
            blocking: true,
        });
    }

    // The northstar ratchet: artifacts must move INTO the build graph, never around it.
    if coverage.unpackaged_yaml_files > policy.baseline_unpackaged_yaml_files {
        findings.push(Finding {
            code: CODE_UNPACKAGED_REGRESSION.to_owned(),
            package: String::new(),
            detail: format!(
                "{} YAML files belong to no buck2 package, above the frozen ceiling of {}. New YAML \
                 must land inside a buck2 package so it is a build-graph input.",
                coverage.unpackaged_yaml_files, policy.baseline_unpackaged_yaml_files
            ),
            blocking: true,
        });
    }

    if coverage.uncovered_packages > policy.baseline_uncovered_packages {
        findings.push(Finding {
            code: CODE_COVERAGE_REGRESSION.to_owned(),
            package: String::new(),
            detail: format!(
                "{} YAML-owning packages declare no corpus-yaml-facts target, above the frozen \
                 ceiling of {}. A new YAML-owning package must either declare an extraction target \
                 or lower the ceiling in the same change.",
                coverage.uncovered_packages, policy.baseline_uncovered_packages
            ),
            blocking: true,
        });
    } else if coverage.uncovered_packages < policy.baseline_uncovered_packages {
        // Slack means the ratchet has stopped biting: coverage improved but the ceiling was not
        // lowered, so a regression back to the old ceiling would pass unnoticed.
        findings.push(Finding {
            code: CODE_STALE_CEILING.to_owned(),
            package: String::new(),
            detail: format!(
                "uncovered is {} but the ceiling is {} — lower baseline_uncovered_packages to {} so \
                 the ratchet keeps biting",
                coverage.uncovered_packages,
                policy.baseline_uncovered_packages,
                coverage.uncovered_packages
            ),
            blocking: false,
        });
    }

    for observation in observations.iter().filter(|o| !o.indexed) {
        findings.push(Finding {
            code: CODE_UNCOVERED_PACKAGE.to_owned(),
            package: observation.package.clone(),
            detail: format!(
                "{} YAML file(s) outside the corpus graph",
                observation.yaml_files
            ),
            blocking: false,
        });
    }

    Verdict { coverage, findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str, files: usize, indexed: bool) -> PackageObservation {
        PackageObservation {
            package: name.to_owned(),
            yaml_files: files,
            indexed,
        }
    }

    fn policy(ceiling: usize, unpackaged: usize) -> Policy {
        Policy {
            baseline_uncovered_packages: ceiling,
            baseline_unpackaged_yaml_files: unpackaged,
            min_expected_yaml_packages: 1,
            min_expected_yaml_files: 1,
        }
    }

    #[test]
    fn coverage_is_computed_not_asserted() {
        let observed = [pkg("a", 3, true), pkg("b", 7, false), pkg("c", 5, true)];
        let computed = coverage(&observed, 0);
        assert_eq!(computed.total_packages, 3);
        assert_eq!(computed.indexed_packages, 2);
        assert_eq!(computed.uncovered_packages, 1);
        assert_eq!(computed.total_yaml_files, 15);
        assert_eq!(computed.indexed_yaml_files, 8);
        assert_eq!(computed.package_coverage_bps(), 6_666);
        assert_eq!(computed.file_coverage_bps(), 5_333);
    }

    // The false green this gate was REBUILT to prevent: an earlier version counted only in-package
    // YAML, so indexing every package reported 100% coverage while most of the corpus sat outside
    // the build graph entirely. Unpackaged files belong in the DENOMINATOR.
    #[test]
    fn unpackaged_files_stay_in_the_denominator() {
        let computed = coverage(&[pkg("a", 100, true)], 900);
        assert_eq!(computed.total_yaml_files, 1_000);
        assert_eq!(computed.indexed_yaml_files, 100);
        assert_eq!(
            computed.file_coverage_bps(),
            1_000,
            "every package indexed must NOT read as full coverage while 900 files are unpackaged"
        );
        assert_eq!(computed.package_coverage_bps(), 10_000);
    }

    #[test]
    fn at_the_ceiling_is_green() {
        let observed = [pkg("a", 1, false), pkg("b", 1, true)];
        assert!(!evaluate(&observed, 5, &policy(1, 5)).failed());
    }

    // The ratchet: one more uncovered package than the ceiling must FAIL.
    #[test]
    fn a_new_uncovered_package_regresses_and_blocks() {
        let observed = [pkg("a", 1, false), pkg("b", 1, false)];
        let verdict = evaluate(&observed, 0, &policy(1, 0));
        assert!(verdict.failed());
        assert!(
            verdict
                .blocking()
                .iter()
                .any(|f| f.code == CODE_COVERAGE_REGRESSION)
        );
    }

    // The northstar ratchet: new YAML outside every buck2 package must FAIL, so the fix is to pull
    // artifacts INTO the build graph rather than index them through a side channel.
    #[test]
    fn new_unpackaged_yaml_regresses_and_blocks() {
        let verdict = evaluate(&[pkg("a", 1, true)], 11, &policy(0, 10));
        assert!(verdict.failed());
        assert!(
            verdict
                .blocking()
                .iter()
                .any(|f| f.code == CODE_UNPACKAGED_REGRESSION)
        );
    }

    // Landing an extraction target must move the number and never fail the gate.
    #[test]
    fn indexing_a_package_burns_the_number_down() {
        let before = coverage(&[pkg("a", 4, false), pkg("b", 6, false)], 0);
        let after = coverage(&[pkg("a", 4, true), pkg("b", 6, false)], 0);
        assert_eq!(before.file_coverage_bps(), 0);
        assert_eq!(after.file_coverage_bps(), 4_000);
        assert!(!evaluate(&[pkg("a", 4, true), pkg("b", 6, false)], 0, &policy(2, 0)).failed());
    }

    // The failure that matters: a broken walk sees nothing, so uncovered is 0, which would read as
    // FLAWLESS COVERAGE without the floor.
    #[test]
    fn an_empty_scan_fails_closed_instead_of_reporting_perfection() {
        let verdict = evaluate(&[], 0, &policy(10, 10));
        assert_eq!(verdict.coverage.uncovered_packages, 0);
        assert_eq!(verdict.coverage.package_coverage_bps(), 0);
        assert_eq!(verdict.coverage.file_coverage_bps(), 0);
        assert!(verdict.failed(), "a vacuous scan must not pass");
        assert!(verdict.blocking().iter().any(|f| f.code == CODE_VACUOUS_SCAN));
    }

    // A collapsed file census would shrink the unpackaged count for the wrong reason, which would
    // read as northstar progress.
    #[test]
    fn a_collapsed_file_census_fails_closed() {
        let strict = Policy {
            baseline_uncovered_packages: 10,
            baseline_unpackaged_yaml_files: 5_000,
            min_expected_yaml_packages: 1,
            min_expected_yaml_files: 5_000,
        };
        let verdict = evaluate(&[pkg("a", 1, true)], 0, &strict);
        assert!(verdict.failed(), "a collapsed census must not read as progress");
        assert!(verdict.blocking().iter().any(|f| f.code == CODE_VACUOUS_SCAN));
    }

    #[test]
    fn slack_in_the_ceiling_is_reported_so_the_ratchet_keeps_biting() {
        let observed = [pkg("a", 1, true), pkg("b", 1, true)];
        let verdict = evaluate(&observed, 0, &policy(5, 0));
        assert!(!verdict.failed());
        assert!(verdict.findings.iter().any(|f| f.code == CODE_STALE_CEILING));
    }

    #[test]
    fn uncovered_packages_are_reported_individually_as_advisory_debt() {
        let observed = [pkg("a", 2, false), pkg("b", 3, false)];
        let verdict = evaluate(&observed, 0, &policy(2, 0));
        let advisory: Vec<&Finding> = verdict
            .findings
            .iter()
            .filter(|f| f.code == CODE_UNCOVERED_PACKAGE)
            .collect();
        assert_eq!(advisory.len(), 2);
        assert!(advisory.iter().all(|f| !f.blocking));
    }
}
