//! Regional pack onboarding phase: lifecycle states and rollout gate for
//! onboarding a pack into a region.
//!
//! M03-P07 merge-variant delta-1.  No new crate, no new deps (std-only additions).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Ordered lifecycle phases for onboarding a regional pack.
///
/// A pack progresses strictly forward through these phases; no phase may be
/// skipped.  `Activated` is the terminal success state; `Withdrawn` is the
/// terminal failure state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PackOnboardingPhase {
    /// Pack registered in the registry; contracts not yet validated.
    Registered,
    /// Residency contracts under legal review.
    ContractReview,
    /// Technical controls verified in a staging environment.
    ControlsVerified,
    /// Compliance evidence reviewed and approved.
    ComplianceApproved,
    /// Pack live and serving tenant traffic.
    Activated,
    /// Onboarding halted; pack removed from the active registry.
    Withdrawn,
}

impl PackOnboardingPhase {
    /// Canonical kebab-case label used in audit events and registry keys.
    pub fn label(self) -> &'static str {
        match self {
            Self::Registered => "registered",
            Self::ContractReview => "contract-review",
            Self::ControlsVerified => "controls-verified",
            Self::ComplianceApproved => "compliance-approved",
            Self::Activated => "activated",
            Self::Withdrawn => "withdrawn",
        }
    }

    /// Returns `true` if this phase is a terminal state (no further progress).
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Activated | Self::Withdrawn)
    }

    /// Parse a canonical label back to a phase variant.
    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "registered" => Some(Self::Registered),
            "contract-review" => Some(Self::ContractReview),
            "controls-verified" => Some(Self::ControlsVerified),
            "compliance-approved" => Some(Self::ComplianceApproved),
            "activated" => Some(Self::Activated),
            "withdrawn" => Some(Self::Withdrawn),
            _ => None,
        }
    }

    /// Returns `true` if transitioning from `self` to `next` is a valid
    /// strict-forward step.
    ///
    /// Valid transitions advance exactly one position in declaration order,
    /// except that any non-terminal phase may transition directly to
    /// `Withdrawn` (emergency halt).  No transition is valid from a terminal
    /// phase (`Activated` or `Withdrawn`).
    pub fn is_valid_next(self, next: Self) -> bool {
        if self.is_terminal() {
            return false;
        }
        // Emergency withdrawal is always permitted from any non-terminal phase.
        if next == Self::Withdrawn {
            return true;
        }
        // Otherwise only a single forward step is allowed.
        let forward_next = match self {
            Self::Registered => Self::ContractReview,
            Self::ContractReview => Self::ControlsVerified,
            Self::ControlsVerified => Self::ComplianceApproved,
            Self::ComplianceApproved => Self::Activated,
            // Terminal phases handled above.
            Self::Activated | Self::Withdrawn => return false,
        };
        next == forward_next
    }
}

/// Installation status of a regional pack within a specific region slot.
///
/// Distinct from [`PackOnboardingPhase`]: this type tracks the per-region
/// runtime install state rather than the overall lifecycle phase.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PackInstallStatus {
    /// Pack is queued for installation but not yet started.
    Pending,
    /// Installation actively in progress.
    Installing,
    /// Pack installed and health checks passing.
    Healthy,
    /// Pack installed but health checks are failing.
    Degraded,
    /// Installation failed; rollback completed.
    Failed,
}

impl PackInstallStatus {
    /// Canonical kebab-case label used in metrics and install-event payloads.
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Installing => "installing",
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
        }
    }

    /// Returns `true` when this status means the pack is serving traffic.
    pub fn is_serving(self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    /// Parse a canonical label back to a status variant.
    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "installing" => Some(Self::Installing),
            "healthy" => Some(Self::Healthy),
            "degraded" => Some(Self::Degraded),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Gate that must pass before a pack rolls out to a new region.
///
/// Each field is a named prerequisite that must be satisfied.  The gate is
/// considered open only when all fields are `true`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalRolloutGate {
    /// All residency contracts for the target region are signed.
    pub contracts_signed: bool, // data_class: INTERNAL_ONLY
    /// Technical controls have been validated against the target region.
    pub controls_validated: bool, // data_class: INTERNAL_ONLY
    /// Compliance evidence package reviewed and accepted.
    pub compliance_evidence_accepted: bool, // data_class: INTERNAL_ONLY
    /// Capacity reservation for the target region is confirmed.
    pub capacity_reserved: bool, // data_class: INTERNAL_ONLY
}

/// Errors produced when constructing a [`RegionalRolloutGate`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegionalRolloutGateError {
    /// At least one prerequisite is not yet satisfied; gate remains closed.
    PrerequisiteNotMet { unmet: &'static str },
}

impl RegionalRolloutGate {
    /// Construct a fully satisfied (open) gate.
    pub fn open() -> Self {
        Self {
            contracts_signed: true,
            controls_validated: true,
            compliance_evidence_accepted: true,
            capacity_reserved: true,
        }
    }

    /// Returns `Ok(())` when all prerequisites are met, or the first unmet
    /// prerequisite as an error.
    pub fn check(&self) -> Result<(), RegionalRolloutGateError> {
        if !self.contracts_signed {
            return Err(RegionalRolloutGateError::PrerequisiteNotMet {
                unmet: "contracts_signed",
            });
        }
        if !self.controls_validated {
            return Err(RegionalRolloutGateError::PrerequisiteNotMet {
                unmet: "controls_validated",
            });
        }
        if !self.compliance_evidence_accepted {
            return Err(RegionalRolloutGateError::PrerequisiteNotMet {
                unmet: "compliance_evidence_accepted",
            });
        }
        if !self.capacity_reserved {
            return Err(RegionalRolloutGateError::PrerequisiteNotMet {
                unmet: "capacity_reserved",
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_next_allows_single_step_forward() {
        assert!(PackOnboardingPhase::Registered.is_valid_next(PackOnboardingPhase::ContractReview));
        assert!(
            PackOnboardingPhase::ContractReview
                .is_valid_next(PackOnboardingPhase::ControlsVerified)
        );
        assert!(
            PackOnboardingPhase::ControlsVerified
                .is_valid_next(PackOnboardingPhase::ComplianceApproved)
        );
        assert!(
            PackOnboardingPhase::ComplianceApproved.is_valid_next(PackOnboardingPhase::Activated)
        );
    }

    #[test]
    fn is_valid_next_allows_emergency_withdrawal_from_any_non_terminal() {
        assert!(PackOnboardingPhase::Registered.is_valid_next(PackOnboardingPhase::Withdrawn));
        assert!(PackOnboardingPhase::ContractReview.is_valid_next(PackOnboardingPhase::Withdrawn));
        assert!(
            PackOnboardingPhase::ControlsVerified.is_valid_next(PackOnboardingPhase::Withdrawn)
        );
        assert!(
            PackOnboardingPhase::ComplianceApproved.is_valid_next(PackOnboardingPhase::Withdrawn)
        );
    }

    #[test]
    fn is_valid_next_rejects_skips() {
        // Skip two steps.
        assert!(
            !PackOnboardingPhase::Registered.is_valid_next(PackOnboardingPhase::ControlsVerified)
        );
        assert!(
            !PackOnboardingPhase::Registered.is_valid_next(PackOnboardingPhase::ComplianceApproved)
        );
        assert!(!PackOnboardingPhase::Registered.is_valid_next(PackOnboardingPhase::Activated));
    }

    #[test]
    fn is_valid_next_rejects_transitions_from_terminal_phases() {
        assert!(!PackOnboardingPhase::Activated.is_valid_next(PackOnboardingPhase::Withdrawn));
        assert!(!PackOnboardingPhase::Withdrawn.is_valid_next(PackOnboardingPhase::Registered));
    }

    #[test]
    fn all_onboarding_phases_have_distinct_labels() {
        use std::collections::BTreeSet;
        let phases = [
            PackOnboardingPhase::Registered,
            PackOnboardingPhase::ContractReview,
            PackOnboardingPhase::ControlsVerified,
            PackOnboardingPhase::ComplianceApproved,
            PackOnboardingPhase::Activated,
            PackOnboardingPhase::Withdrawn,
        ];
        let labels: BTreeSet<_> = phases.iter().map(|p| p.label()).collect();
        assert_eq!(labels.len(), 6, "all 6 phases must have distinct labels");
    }

    #[test]
    fn terminal_phases_are_activated_and_withdrawn() {
        assert!(PackOnboardingPhase::Activated.is_terminal());
        assert!(PackOnboardingPhase::Withdrawn.is_terminal());
        assert!(!PackOnboardingPhase::Registered.is_terminal());
        assert!(!PackOnboardingPhase::ContractReview.is_terminal());
        assert!(!PackOnboardingPhase::ControlsVerified.is_terminal());
        assert!(!PackOnboardingPhase::ComplianceApproved.is_terminal());
    }

    #[test]
    fn pack_onboarding_phase_round_trips_via_from_wire() {
        let phases = [
            PackOnboardingPhase::Registered,
            PackOnboardingPhase::ContractReview,
            PackOnboardingPhase::ControlsVerified,
            PackOnboardingPhase::ComplianceApproved,
            PackOnboardingPhase::Activated,
            PackOnboardingPhase::Withdrawn,
        ];
        for phase in phases {
            assert_eq!(
                PackOnboardingPhase::from_wire(phase.label()),
                Some(phase),
                "round-trip must succeed for {}",
                phase.label()
            );
        }
        assert_eq!(PackOnboardingPhase::from_wire("unknown"), None);
    }

    #[test]
    fn all_install_statuses_have_distinct_labels() {
        use std::collections::BTreeSet;
        let statuses = [
            PackInstallStatus::Pending,
            PackInstallStatus::Installing,
            PackInstallStatus::Healthy,
            PackInstallStatus::Degraded,
            PackInstallStatus::Failed,
        ];
        let labels: BTreeSet<_> = statuses.iter().map(|s| s.label()).collect();
        assert_eq!(labels.len(), 5, "all 5 statuses must have distinct labels");
    }

    #[test]
    fn serving_statuses_are_healthy_and_degraded() {
        assert!(PackInstallStatus::Healthy.is_serving());
        assert!(PackInstallStatus::Degraded.is_serving());
        assert!(!PackInstallStatus::Pending.is_serving());
        assert!(!PackInstallStatus::Installing.is_serving());
        assert!(!PackInstallStatus::Failed.is_serving());
    }

    #[test]
    fn pack_install_status_round_trips_via_from_wire() {
        let statuses = [
            PackInstallStatus::Pending,
            PackInstallStatus::Installing,
            PackInstallStatus::Healthy,
            PackInstallStatus::Degraded,
            PackInstallStatus::Failed,
        ];
        for status in statuses {
            assert_eq!(
                PackInstallStatus::from_wire(status.label()),
                Some(status),
                "round-trip must succeed for {}",
                status.label()
            );
        }
        assert_eq!(PackInstallStatus::from_wire("unknown"), None);
    }

    #[test]
    fn regional_rollout_gate_open_passes_check() {
        assert_eq!(RegionalRolloutGate::open().check(), Ok(()));
    }

    #[test]
    fn regional_rollout_gate_fails_on_first_unmet_prerequisite() {
        let gate = RegionalRolloutGate {
            contracts_signed: false,
            controls_validated: true,
            compliance_evidence_accepted: true,
            capacity_reserved: true,
        };
        assert_eq!(
            gate.check(),
            Err(RegionalRolloutGateError::PrerequisiteNotMet {
                unmet: "contracts_signed"
            })
        );

        let gate2 = RegionalRolloutGate {
            contracts_signed: true,
            controls_validated: false,
            compliance_evidence_accepted: true,
            capacity_reserved: true,
        };
        assert_eq!(
            gate2.check(),
            Err(RegionalRolloutGateError::PrerequisiteNotMet {
                unmet: "controls_validated"
            })
        );
    }

    #[test]
    fn regional_rollout_gate_reports_capacity_unmet() {
        let gate = RegionalRolloutGate {
            contracts_signed: true,
            controls_validated: true,
            compliance_evidence_accepted: true,
            capacity_reserved: false,
        };
        assert_eq!(
            gate.check(),
            Err(RegionalRolloutGateError::PrerequisiteNotMet {
                unmet: "capacity_reserved"
            })
        );
    }
}
