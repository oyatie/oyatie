//! No-grouping fitness kernel (ADR-0362).
//!
//! Flat single-concern microservices are the only architecture unit. Product
//! grouping artifacts (`*-suite.json` / `*-family.json` / `*-bundle.json`
//! wrappers under `specs/microservices/`) are retired. This kernel fails on any
//! grouping-shaped spec wrapper unless it is one of the two known retiring
//! wrappers AND is marked `Deprecated` with a `retirement_ref` — a tombstone on
//! a tracked retirement path, not a live architecture artifact (ADR-0362).
//!
//! This closes the ADR-0132 aspirational gap: that ADR specified a
//! `no-new-suite-bundles` BLOCKER lane that was never implemented.
//!
//! The kernel does no I/O: the runner discovers grouping-shaped files under
//! `specs/microservices/`, reads each `_meta.status` / `_meta.retirement_ref`,
//! and passes typed [`GroupingArtifact`] values in.

#![forbid(unsafe_code)]
// ADR-0083 Tier 1 (kernel): no unwrap/expect/panic in non-test code.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// A grouping-shaped spec wrapper discovered under `specs/microservices/`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupingArtifact {
    /// File name only, e.g. `"connect-suite.json"`. data_class: INTERNAL_ONLY
    pub file_name: String,
    /// `_meta.status` if present. data_class: INTERNAL_ONLY
    pub status: Option<String>,
    /// `_meta.retirement_ref` present and non-empty. data_class: INTERNAL_ONLY
    pub has_retirement_ref: bool,
}

/// Summary returned on a clean validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoGroupingReport {
    /// Grouping-shaped artifacts inspected.
    pub artifacts_checked: usize,
    /// Allowlisted retiring wrappers confirmed as deprecated tombstones.
    pub retiring_wrappers: usize,
}

/// One violation of the flat-only / no-grouping doctrine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NoGroupingError {
    /// A grouping wrapper that is not on the bounded retirement allowlist —
    /// i.e. a NEW grouping artifact, forbidden by ADR-0362.
    NewGroupingArtifact { file_name: String },
    /// An allowlisted retiring wrapper that is still presented as live (not
    /// `Deprecated`, or missing a `retirement_ref`).
    LiveGroupingArtifact {
        file_name: String,
        reason: &'static str,
    },
}

/// The grouping wrappers grandfathered by ADR-0132 and demoted by ADR-0362.
/// Tolerated ONLY while `Deprecated` with a `retirement_ref`.
pub const RETIRING_WRAPPERS: &[&str] = &["connect-suite.json", "enterprise-suite.json"];

/// True if a file name is grouping-shaped (suite / family / bundle wrapper).
#[must_use]
pub fn is_grouping_artifact(file_name: &str) -> bool {
    file_name.ends_with("-suite.json")
        || file_name.ends_with("-family.json")
        || file_name.ends_with("-bundle.json")
}

/// Validate that no live grouping artifacts exist. Empty input (no grouping
/// wrappers at all) is the clean steady-state and passes.
///
/// # Errors
/// Returns the first [`NoGroupingError`] encountered.
pub fn validate_no_grouping<I>(artifacts: I) -> Result<NoGroupingReport, NoGroupingError>
where
    I: IntoIterator<Item = GroupingArtifact>,
{
    let mut artifacts_checked = 0usize;
    let mut retiring_wrappers = 0usize;
    for artifact in artifacts {
        artifacts_checked += 1;
        if !RETIRING_WRAPPERS.contains(&artifact.file_name.as_str()) {
            return Err(NoGroupingError::NewGroupingArtifact {
                file_name: artifact.file_name,
            });
        }
        if artifact.status.as_deref() != Some("Deprecated") {
            return Err(NoGroupingError::LiveGroupingArtifact {
                file_name: artifact.file_name,
                reason: "retiring grouping wrapper must set _meta.status to \"Deprecated\"",
            });
        }
        if !artifact.has_retirement_ref {
            return Err(NoGroupingError::LiveGroupingArtifact {
                file_name: artifact.file_name,
                reason: "retiring grouping wrapper must carry a _meta.retirement_ref",
            });
        }
        retiring_wrappers += 1;
    }
    Ok(NoGroupingReport {
        artifacts_checked,
        retiring_wrappers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deprecated(name: &str) -> GroupingArtifact {
        GroupingArtifact {
            file_name: name.to_string(),
            status: Some("Deprecated".to_string()),
            has_retirement_ref: true,
        }
    }

    #[test]
    fn passes_when_no_grouping_artifacts() {
        let report = validate_no_grouping(Vec::new()).expect("empty is clean");
        assert_eq!(report.artifacts_checked, 0);
        assert_eq!(report.retiring_wrappers, 0);
    }

    #[test]
    fn passes_when_both_wrappers_are_deprecated_tombstones() {
        let report = validate_no_grouping(vec![
            deprecated("connect-suite.json"),
            deprecated("enterprise-suite.json"),
        ])
        .expect("deprecated tombstones pass");
        assert_eq!(report.retiring_wrappers, 2);
    }

    #[test]
    fn fails_on_new_grouping_artifact() {
        let err = validate_no_grouping(vec![deprecated("healthcare-suite.json")])
            .expect_err("new grouping forbidden");
        assert_eq!(
            err,
            NoGroupingError::NewGroupingArtifact {
                file_name: "healthcare-suite.json".to_string()
            }
        );
    }

    #[test]
    fn fails_on_new_family_or_bundle_artifact() {
        assert!(matches!(
            validate_no_grouping(vec![deprecated("connect-family.json")]),
            Err(NoGroupingError::NewGroupingArtifact { .. })
        ));
        assert!(matches!(
            validate_no_grouping(vec![deprecated("office-bundle.json")]),
            Err(NoGroupingError::NewGroupingArtifact { .. })
        ));
    }

    #[test]
    fn fails_when_retiring_wrapper_not_deprecated() {
        let live = GroupingArtifact {
            file_name: "connect-suite.json".to_string(),
            status: Some("Accepted".to_string()),
            has_retirement_ref: false,
        };
        assert!(matches!(
            validate_no_grouping(vec![live]),
            Err(NoGroupingError::LiveGroupingArtifact { .. })
        ));
    }

    #[test]
    fn fails_when_retiring_wrapper_missing_retirement_ref() {
        let no_ref = GroupingArtifact {
            file_name: "enterprise-suite.json".to_string(),
            status: Some("Deprecated".to_string()),
            has_retirement_ref: false,
        };
        assert_eq!(
            validate_no_grouping(vec![no_ref]),
            Err(NoGroupingError::LiveGroupingArtifact {
                file_name: "enterprise-suite.json".to_string(),
                reason: "retiring grouping wrapper must carry a _meta.retirement_ref",
            })
        );
    }

    #[test]
    fn grouping_artifact_suffix_detection() {
        assert!(is_grouping_artifact("connect-suite.json"));
        assert!(is_grouping_artifact("x-family.json"));
        assert!(is_grouping_artifact("y-bundle.json"));
        assert!(!is_grouping_artifact("mail.json"));
        assert!(!is_grouping_artifact("manifest-schema.json"));
    }
}
