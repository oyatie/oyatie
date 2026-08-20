//! Design-partner onboarding status for M04-P03.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Lifecycle status of a design-partner tenant through the M04-P03 onboarding
/// funnel.  Variants are ordered from earliest to latest stage; `Ord` reflects
/// that ordering so callers can assert forward-only progression.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DesignPartnerStatus {
    /// Tenant record created; KR pack bound; awaiting first workflow authoring.
    Provisioned,
    /// At least one tenant-specific workflow authored in Workflow Studio.
    WorkflowAuthored,
    /// Foundry agents activated under autonomy ceiling T1-T3.
    AgentsActive,
    /// All M04-P03 acceptance criteria met; partner considered live.
    Live,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignPartnerStatusError {
    /// Attempted to regress to an earlier or equal stage.
    IllegalRegression {
        from: DesignPartnerStatus,
        to: DesignPartnerStatus,
    },
    /// Attempted to jump over one or more required intermediate stages
    /// (e.g. `Provisioned -> Live` skipping `WorkflowAuthored`/`AgentsActive`).
    SkippedStage {
        from: DesignPartnerStatus,
        to: DesignPartnerStatus,
        expected_next: DesignPartnerStatus,
    },
    /// Attempted to advance past the terminal stage.
    AlreadyTerminal { from: DesignPartnerStatus },
}

impl DesignPartnerStatus {
    /// Returns the ADR-0049-style snake_case label for this status.
    pub fn label(self) -> &'static str {
        match self {
            Self::Provisioned => "provisioned",
            Self::WorkflowAuthored => "workflow_authored",
            Self::AgentsActive => "agents_active",
            Self::Live => "live",
        }
    }

    /// Returns the only legal next stage from `self`, or `None` when `self` is
    /// terminal (`Live`).
    pub fn expected_next(self) -> Option<Self> {
        match self {
            Self::Provisioned => Some(Self::WorkflowAuthored),
            Self::WorkflowAuthored => Some(Self::AgentsActive),
            Self::AgentsActive => Some(Self::Live),
            Self::Live => None,
        }
    }

    /// Attempt to advance to `next`. Only the immediately-adjacent stage is
    /// accepted; regressions and skipped-stage jumps are both rejected so that
    /// every required onboarding gate is observed.
    pub fn advance(self, next: Self) -> Result<Self, DesignPartnerStatusError> {
        if next <= self {
            return Err(DesignPartnerStatusError::IllegalRegression {
                from: self,
                to: next,
            });
        }
        match self.expected_next() {
            None => Err(DesignPartnerStatusError::AlreadyTerminal { from: self }),
            Some(expected) if next == expected => Ok(next),
            Some(expected) => Err(DesignPartnerStatusError::SkippedStage {
                from: self,
                to: next,
                expected_next: expected,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_snake_case() {
        assert_eq!(DesignPartnerStatus::Provisioned.label(), "provisioned");
        assert_eq!(
            DesignPartnerStatus::WorkflowAuthored.label(),
            "workflow_authored"
        );
        assert_eq!(DesignPartnerStatus::AgentsActive.label(), "agents_active");
        assert_eq!(DesignPartnerStatus::Live.label(), "live");
    }

    #[test]
    fn forward_advance_succeeds() {
        let s = DesignPartnerStatus::Provisioned
            .advance(DesignPartnerStatus::WorkflowAuthored)
            .expect("Provisioned -> WorkflowAuthored is valid");
        assert_eq!(s, DesignPartnerStatus::WorkflowAuthored);
    }

    #[test]
    fn full_happy_path() {
        let live = DesignPartnerStatus::Provisioned
            .advance(DesignPartnerStatus::WorkflowAuthored)
            .unwrap()
            .advance(DesignPartnerStatus::AgentsActive)
            .unwrap()
            .advance(DesignPartnerStatus::Live)
            .unwrap();
        assert_eq!(live.label(), "live");
    }

    #[test]
    fn regression_rejected() {
        let err = DesignPartnerStatus::AgentsActive
            .advance(DesignPartnerStatus::Provisioned)
            .expect_err("regression must be rejected");
        assert_eq!(
            err,
            DesignPartnerStatusError::IllegalRegression {
                from: DesignPartnerStatus::AgentsActive,
                to: DesignPartnerStatus::Provisioned,
            }
        );
    }

    #[test]
    fn same_stage_is_also_regression() {
        let err = DesignPartnerStatus::Live
            .advance(DesignPartnerStatus::Live)
            .expect_err("idempotent advance must be rejected");
        assert_eq!(
            err,
            DesignPartnerStatusError::IllegalRegression {
                from: DesignPartnerStatus::Live,
                to: DesignPartnerStatus::Live,
            }
        );
    }

    #[test]
    fn skipped_stage_provisioned_to_live_rejected() {
        let err = DesignPartnerStatus::Provisioned
            .advance(DesignPartnerStatus::Live)
            .expect_err("non-adjacent jump must be rejected");
        assert_eq!(
            err,
            DesignPartnerStatusError::SkippedStage {
                from: DesignPartnerStatus::Provisioned,
                to: DesignPartnerStatus::Live,
                expected_next: DesignPartnerStatus::WorkflowAuthored,
            }
        );
    }

    #[test]
    fn skipped_stage_provisioned_to_agents_active_rejected() {
        let err = DesignPartnerStatus::Provisioned
            .advance(DesignPartnerStatus::AgentsActive)
            .expect_err("non-adjacent jump must be rejected");
        assert!(matches!(err, DesignPartnerStatusError::SkippedStage { .. }));
    }

    #[test]
    fn advance_past_terminal_rejected() {
        let err = DesignPartnerStatus::Live
            .advance(DesignPartnerStatus::Provisioned)
            .expect_err("advance past terminal must be rejected");
        // Regression check fires first (Provisioned < Live), so this is
        // IllegalRegression rather than AlreadyTerminal. Confirms ordering of
        // guards.
        assert!(matches!(
            err,
            DesignPartnerStatusError::IllegalRegression { .. }
        ));
    }
}
