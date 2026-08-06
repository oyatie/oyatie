//! Authoritative-tracked fitness kernel.
//!
//! The kernel is I/O-free. Runners enumerate authoritative artifact paths,
//! collect repository tracking state, and pass typed records into [`check`].
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeArtifact {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub tracked: bool,    // data_class: INTERNAL_ONLY
    pub on_disk: bool,    // data_class: INTERNAL_ONLY
    pub gitignored: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeTrackedFitnessReport {
    pub artifacts_checked: usize, // data_class: INTERNAL_ONLY
    pub tracked_artifacts: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthoritativeTrackedFitnessError {
    EmptyArtifactSet,
    Missing { path: String },
    Gitignored { path: String },
    Untracked { path: String },
}

impl AuthoritativeTrackedFitnessError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyArtifactSet => "authoritative artifact set is empty".into(),
            Self::Missing { path } => format!("authoritative artifact '{path}' is missing"),
            Self::Gitignored { path } => {
                format!("authoritative artifact '{path}' is matched by ignore rules")
            }
            Self::Untracked { path } => {
                format!("authoritative artifact '{path}' exists but is not tracked")
            }
        }
    }
}

pub fn check(
    artifacts: &[AuthoritativeArtifact],
) -> Result<AuthoritativeTrackedFitnessReport, AuthoritativeTrackedFitnessError> {
    if artifacts.is_empty() {
        return Err(AuthoritativeTrackedFitnessError::EmptyArtifactSet);
    }

    let mut tracked_artifacts = 0usize;
    for artifact in artifacts {
        if !artifact.on_disk {
            return Err(AuthoritativeTrackedFitnessError::Missing {
                path: artifact.path.clone(),
            });
        }
        if artifact.gitignored {
            return Err(AuthoritativeTrackedFitnessError::Gitignored {
                path: artifact.path.clone(),
            });
        }
        if !artifact.tracked {
            return Err(AuthoritativeTrackedFitnessError::Untracked {
                path: artifact.path.clone(),
            });
        }
        tracked_artifacts += 1;
    }

    Ok(AuthoritativeTrackedFitnessReport {
        artifacts_checked: artifacts.len(),
        tracked_artifacts,
    })
}

pub fn validate_authoritative_tracked_fitness(
    artifacts: &[AuthoritativeArtifact],
) -> Result<AuthoritativeTrackedFitnessReport, AuthoritativeTrackedFitnessError> {
    check(artifacts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_all_present_tracked_authority() {
        let report = check(&[
            artifact("docs/AGENTS.md", true, true, false),
            artifact(
                "docs/decisions/ADR-0709-general-live-apex.md",
                true,
                true,
                false,
            ),
        ])
        .expect("authoritative artifacts are tracked");

        assert_eq!(report.artifacts_checked, 2);
        assert_eq!(report.tracked_artifacts, 2);
    }

    #[test]
    fn rejects_empty_artifact_set() {
        assert_eq!(
            check(&[]),
            Err(AuthoritativeTrackedFitnessError::EmptyArtifactSet)
        );
    }

    #[test]
    fn rejects_missing_authority_before_tracking_checks() {
        assert_eq!(
            check(&[artifact("docs/MISSING.md", false, false, true)]),
            Err(AuthoritativeTrackedFitnessError::Missing {
                path: "docs/MISSING.md".into()
            })
        );
    }

    #[test]
    fn rejects_gitignored_authority() {
        assert_eq!(
            check(&[artifact("docs/SECRET.md", true, true, true)]),
            Err(AuthoritativeTrackedFitnessError::Gitignored {
                path: "docs/SECRET.md".into()
            })
        );
    }

    #[test]
    fn rejects_untracked_authority() {
        assert_eq!(
            check(&[artifact("docs/DESIGN.md", true, false, false)]),
            Err(AuthoritativeTrackedFitnessError::Untracked {
                path: "docs/DESIGN.md".into()
            })
        );
    }

    fn artifact(
        path: &str,
        on_disk: bool,
        tracked: bool,
        gitignored: bool,
    ) -> AuthoritativeArtifact {
        AuthoritativeArtifact {
            path: path.into(),
            tracked,
            on_disk,
            gitignored,
        }
    }
}
