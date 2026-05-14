//! Doc-freshness fitness kernel — blocks PRs that change a
//! source-of-truth file without regenerating its dependent docs.
//!
//! I/O-free. Runners feed the kernel typed
//! [`SourceDependency`] mappings (e.g., "openapi.yaml drives
//! docs/reference/api.md") plus the set of files changed in the PR.
//! The kernel returns the set of dependent docs that should have been
//! re-generated but weren't touched.

/// Declared "if A changes, B must be regenerated" rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDependency {
    pub source: String,    // data_class: INTERNAL_ONLY
    pub dependent: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaleDoc {
    pub source: String,    // data_class: INTERNAL_ONLY
    pub dependent: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocFreshnessReport {
    pub sources_changed: usize,    // data_class: INTERNAL_ONLY
    pub stale_docs: Vec<StaleDoc>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocFreshnessError {
    EmptySource { dependent: String },
    EmptyDependent { source: String },
    SelfDependency { path: String },
}

impl DocFreshnessError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptySource { dependent } => format!("empty source for dependent {dependent}"),
            Self::EmptyDependent { source } => format!("empty dependent for source {source}"),
            Self::SelfDependency { path } => format!("self-dependency on {path}"),
        }
    }
}

/// Return the dependents that should have been touched in this changeset
/// but weren't. `changed_files` is the deduplicated set of files in the
/// PR diff.
pub fn check(
    dependencies: &[SourceDependency],
    changed_files: &[String],
) -> Result<DocFreshnessReport, DocFreshnessError> {
    let changed: std::collections::BTreeSet<&str> =
        changed_files.iter().map(String::as_str).collect();

    let mut stale = Vec::new();
    let mut sources_changed: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();

    for dep in dependencies {
        if dep.source.is_empty() {
            return Err(DocFreshnessError::EmptySource {
                dependent: dep.dependent.clone(),
            });
        }
        if dep.dependent.is_empty() {
            return Err(DocFreshnessError::EmptyDependent {
                source: dep.source.clone(),
            });
        }
        if dep.source == dep.dependent {
            return Err(DocFreshnessError::SelfDependency {
                path: dep.source.clone(),
            });
        }

        if changed.contains(dep.source.as_str()) {
            sources_changed.insert(dep.source.as_str());
            if !changed.contains(dep.dependent.as_str()) {
                stale.push(StaleDoc {
                    source: dep.source.clone(),
                    dependent: dep.dependent.clone(),
                });
            }
        }
    }

    stale.sort_by(|a, b| {
        (a.source.as_str(), a.dependent.as_str()).cmp(&(b.source.as_str(), b.dependent.as_str()))
    });

    Ok(DocFreshnessReport {
        sources_changed: sources_changed.len(),
        stale_docs: stale,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dep(s: &str, d: &str) -> SourceDependency {
        SourceDependency {
            source: s.into(),
            dependent: d.into(),
        }
    }

    #[test]
    fn empty_changes_no_stale() {
        let r = check(&[dep("a", "b")], &[]).unwrap();
        assert!(r.stale_docs.is_empty());
        assert_eq!(r.sources_changed, 0);
    }

    #[test]
    fn source_changed_dependent_untouched_is_stale() {
        let r = check(&[dep("a", "b")], &["a".into()]).unwrap();
        assert_eq!(r.stale_docs.len(), 1);
        assert_eq!(r.stale_docs[0].dependent, "b");
    }

    #[test]
    fn source_and_dependent_both_touched_passes() {
        let r = check(&[dep("a", "b")], &["a".into(), "b".into()]).unwrap();
        assert!(r.stale_docs.is_empty());
    }

    #[test]
    fn unrelated_change_is_ignored() {
        let r = check(&[dep("a", "b")], &["unrelated".into()]).unwrap();
        assert!(r.stale_docs.is_empty());
    }

    #[test]
    fn one_source_with_two_dependents_both_flagged() {
        let r = check(&[dep("a", "b"), dep("a", "c")], &["a".into()]).unwrap();
        assert_eq!(r.stale_docs.len(), 2);
    }

    #[test]
    fn stale_docs_sorted() {
        let r = check(&[dep("a", "z"), dep("a", "m")], &["a".into()]).unwrap();
        assert_eq!(r.stale_docs[0].dependent, "m");
        assert_eq!(r.stale_docs[1].dependent, "z");
    }

    #[test]
    fn sources_changed_counts_unique() {
        let r = check(
            &[dep("a", "b"), dep("a", "c"), dep("x", "y")],
            &["a".into(), "x".into()],
        )
        .unwrap();
        assert_eq!(r.sources_changed, 2);
    }

    #[test]
    fn empty_source_errors() {
        let err = check(&[dep("", "b")], &[]).unwrap_err();
        assert!(matches!(err, DocFreshnessError::EmptySource { .. }));
    }

    #[test]
    fn empty_dependent_errors() {
        let err = check(&[dep("a", "")], &[]).unwrap_err();
        assert!(matches!(err, DocFreshnessError::EmptyDependent { .. }));
    }

    #[test]
    fn self_dependency_errors() {
        let err = check(&[dep("a", "a")], &[]).unwrap_err();
        assert!(matches!(err, DocFreshnessError::SelfDependency { .. }));
    }

    #[test]
    fn no_dependencies_no_stale() {
        let r = check(&[], &["whatever".into()]).unwrap();
        assert!(r.stale_docs.is_empty());
    }
}
