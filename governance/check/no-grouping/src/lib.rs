//! No-grouping fitness kernel (ADR-0362).
//!
//! Flat single-concern microservices are the only architecture unit. Product,
//! module, family, suite, and bundle grouping artifacts under
//! `specs/microservices/` are retired. Packaging is a later tenant/RBAC
//! entitlement view, not a microservice boundary. This kernel fails on any
//! grouping-shaped spec wrapper.
//!
//! This closes the ADR-0132 aspirational gap: that ADR specified a
//! `no-grouping` BLOCKER lane that was never implemented.
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
    /// File name only, e.g. `"connect-module.json"`. data_class: INTERNAL_ONLY
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
    /// Retiring wrappers confirmed as deprecated tombstones. The steady-state
    /// value is zero because the retirement allowlist is closed.
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

/// The retirement allowlist is closed: Connect/Enterprise/Healthcare package
/// views are derived from tenant/RBAC entitlements outside `specs/microservices/`.
pub const RETIRING_WRAPPERS: &[&str] = &[];

/// True if a file name is grouping-shaped (suite / module / family / bundle wrapper).
#[must_use]
pub fn is_grouping_artifact(file_name: &str) -> bool {
    file_name.ends_with("-suite.json")
        || file_name.ends_with("-module.json")
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
    if let Some(artifact) = artifacts.into_iter().next() {
        return Err(NoGroupingError::NewGroupingArtifact {
            file_name: artifact.file_name,
        });
    }

    Ok(NoGroupingReport {
        artifacts_checked: 0,
        retiring_wrappers: 0,
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
    fn fails_even_when_former_wrappers_are_deprecated_tombstones() {
        let err = validate_no_grouping(vec![deprecated("connect-module.json")])
            .expect_err("former grouping wrappers are no longer allowlisted");
        assert_eq!(
            err,
            NoGroupingError::NewGroupingArtifact {
                file_name: "connect-module.json".to_string()
            }
        );
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
    fn fails_when_former_wrapper_not_deprecated() {
        let live = GroupingArtifact {
            file_name: "connect-module.json".to_string(),
            status: Some("Accepted".to_string()),
            has_retirement_ref: false,
        };
        assert!(matches!(
            validate_no_grouping(vec![live]),
            Err(NoGroupingError::NewGroupingArtifact { .. })
        ));
    }

    #[test]
    fn fails_when_former_wrapper_missing_retirement_ref() {
        let no_ref = GroupingArtifact {
            file_name: "enterprise-module.json".to_string(),
            status: Some("Deprecated".to_string()),
            has_retirement_ref: false,
        };
        assert_eq!(
            validate_no_grouping(vec![no_ref]),
            Err(NoGroupingError::NewGroupingArtifact {
                file_name: "enterprise-module.json".to_string(),
            })
        );
    }

    #[test]
    fn grouping_artifact_suffix_detection() {
        assert!(is_grouping_artifact("connect-module.json"));
        assert!(is_grouping_artifact("x-suite.json"));
        assert!(is_grouping_artifact("x-family.json"));
        assert!(is_grouping_artifact("y-bundle.json"));
        assert!(!is_grouping_artifact("mail.json"));
        assert!(!is_grouping_artifact("manifest-schema.json"));
    }
}
