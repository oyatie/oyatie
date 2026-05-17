//! Cloud cell lifecycle state machine (P18-cloud-tenancy merge-variant delta-1).
//!
//! Implements the `CellState` FSM from IP-001-cloud-tenancy-kernel-scaffold:
//! `Creating → Active → Draining → Decommissioned`.
//! Decommissioned is a terminal state — re-activation is not permitted.
//! No I/O; no framework deps; pure kernel primitives.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub const CELL_LIFECYCLE_SCHEMA_VERSION: u32 = 1;

/// Lifecycle state of an infrastructure cell.
///
/// Transitions allowed:
/// - `Creating  → Active`
/// - `Active    → Draining`
/// - `Draining  → Decommissioned`
///
/// `Decommissioned` is **terminal**: no further transitions are accepted.
/// Any other pair is rejected with [`CellLifecycleError::InvalidTransition`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CellState {
    /// Cell is being provisioned; no tenants may be placed yet.
    Creating,
    /// Cell is fully operational and accepts new tenant placements.
    Active,
    /// Cell is winding down; existing tenants drain, no new placements allowed.
    Draining,
    /// Cell has been permanently decommissioned. Terminal state.
    Decommissioned,
}

impl CellState {
    /// Machine-readable label used in DDL CHECK constraints and audit rows.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Creating => "creating",
            Self::Active => "active",
            Self::Draining => "draining",
            Self::Decommissioned => "decommissioned",
        }
    }

    /// Returns `true` if new tenant placements are permitted in this state.
    pub fn accepts_new_tenants(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// Command that drives a cell through its lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellLifecycleCommand {
    /// Confirm that provisioning completed; move `Creating → Active`.
    Activate,
    /// Begin draining; move `Active → Draining`.
    Drain,
    /// Confirm decommission; move `Draining → Decommissioned`.
    Decommission,
}

/// Errors produced by lifecycle guard logic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellLifecycleError {
    /// The requested transition is not permitted from the current state.
    InvalidTransition {
        from: CellState,
        command: CellLifecycleCommand,
    },
    /// Attempted to activate a cell that has already been decommissioned.
    /// Halt condition per IP-001 §Halt Conditions #2.
    ReactivationOfDecommissionedCell,
}

impl CellLifecycleError {
    /// Human-readable description for logs and audit rows.
    pub fn message(&self) -> String {
        match self {
            Self::InvalidTransition { from, command } => format!(
                "invalid cell lifecycle transition: cannot apply {:?} in state {}",
                command,
                from.as_str()
            ),
            Self::ReactivationOfDecommissionedCell => {
                "cell is decommissioned — re-activation is permanently forbidden".to_owned()
            }
        }
    }
}

/// Apply `command` to `current` state, returning the next state on success.
///
/// Enforces halt condition: `Decommissioned` cells can never be reactivated.
pub fn apply_lifecycle_command(
    current: CellState,
    command: CellLifecycleCommand,
) -> Result<CellState, CellLifecycleError> {
    // Halt condition #2: decommissioned is terminal regardless of command.
    if current == CellState::Decommissioned {
        return Err(CellLifecycleError::ReactivationOfDecommissionedCell);
    }

    match (current, command) {
        (CellState::Creating, CellLifecycleCommand::Activate) => Ok(CellState::Active),
        (CellState::Active, CellLifecycleCommand::Drain) => Ok(CellState::Draining),
        (CellState::Draining, CellLifecycleCommand::Decommission) => Ok(CellState::Decommissioned),
        _ => Err(CellLifecycleError::InvalidTransition {
            from: current,
            command,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- happy-path transitions ---

    #[test]
    fn creating_to_active_via_activate() {
        assert_eq!(
            apply_lifecycle_command(CellState::Creating, CellLifecycleCommand::Activate),
            Ok(CellState::Active)
        );
    }

    #[test]
    fn active_to_draining_via_drain() {
        assert_eq!(
            apply_lifecycle_command(CellState::Active, CellLifecycleCommand::Drain),
            Ok(CellState::Draining)
        );
    }

    #[test]
    fn draining_to_decommissioned_via_decommission() {
        assert_eq!(
            apply_lifecycle_command(CellState::Draining, CellLifecycleCommand::Decommission),
            Ok(CellState::Decommissioned)
        );
    }

    #[test]
    fn full_happy_path_sequence() {
        let s0 = CellState::Creating;
        let s1 = apply_lifecycle_command(s0, CellLifecycleCommand::Activate).unwrap();
        assert_eq!(s1, CellState::Active);
        let s2 = apply_lifecycle_command(s1, CellLifecycleCommand::Drain).unwrap();
        assert_eq!(s2, CellState::Draining);
        let s3 = apply_lifecycle_command(s2, CellLifecycleCommand::Decommission).unwrap();
        assert_eq!(s3, CellState::Decommissioned);
    }

    // --- terminal-state guard (halt condition #2) ---

    #[test]
    fn decommissioned_rejects_activate() {
        assert_eq!(
            apply_lifecycle_command(CellState::Decommissioned, CellLifecycleCommand::Activate),
            Err(CellLifecycleError::ReactivationOfDecommissionedCell)
        );
    }

    #[test]
    fn decommissioned_rejects_drain() {
        assert_eq!(
            apply_lifecycle_command(CellState::Decommissioned, CellLifecycleCommand::Drain),
            Err(CellLifecycleError::ReactivationOfDecommissionedCell)
        );
    }

    #[test]
    fn decommissioned_rejects_decommission() {
        assert_eq!(
            apply_lifecycle_command(
                CellState::Decommissioned,
                CellLifecycleCommand::Decommission
            ),
            Err(CellLifecycleError::ReactivationOfDecommissionedCell)
        );
    }

    // --- invalid forward/skip transitions ---

    #[test]
    fn creating_rejects_drain() {
        assert!(matches!(
            apply_lifecycle_command(CellState::Creating, CellLifecycleCommand::Drain),
            Err(CellLifecycleError::InvalidTransition {
                from: CellState::Creating,
                ..
            })
        ));
    }

    #[test]
    fn creating_rejects_decommission() {
        assert!(matches!(
            apply_lifecycle_command(CellState::Creating, CellLifecycleCommand::Decommission),
            Err(CellLifecycleError::InvalidTransition {
                from: CellState::Creating,
                ..
            })
        ));
    }

    #[test]
    fn active_rejects_activate() {
        assert!(matches!(
            apply_lifecycle_command(CellState::Active, CellLifecycleCommand::Activate),
            Err(CellLifecycleError::InvalidTransition {
                from: CellState::Active,
                ..
            })
        ));
    }

    #[test]
    fn active_rejects_decommission() {
        assert!(matches!(
            apply_lifecycle_command(CellState::Active, CellLifecycleCommand::Decommission),
            Err(CellLifecycleError::InvalidTransition {
                from: CellState::Active,
                ..
            })
        ));
    }

    #[test]
    fn draining_rejects_activate() {
        assert!(matches!(
            apply_lifecycle_command(CellState::Draining, CellLifecycleCommand::Activate),
            Err(CellLifecycleError::InvalidTransition {
                from: CellState::Draining,
                ..
            })
        ));
    }

    #[test]
    fn draining_rejects_drain() {
        assert!(matches!(
            apply_lifecycle_command(CellState::Draining, CellLifecycleCommand::Drain),
            Err(CellLifecycleError::InvalidTransition {
                from: CellState::Draining,
                ..
            })
        ));
    }

    // --- semantic guards ---

    #[test]
    fn only_active_accepts_new_tenants() {
        assert!(!CellState::Creating.accepts_new_tenants());
        assert!(CellState::Active.accepts_new_tenants());
        assert!(!CellState::Draining.accepts_new_tenants());
        assert!(!CellState::Decommissioned.accepts_new_tenants());
    }

    #[test]
    fn cell_state_labels_are_distinct() {
        use std::collections::HashSet;
        let labels: HashSet<_> = [
            CellState::Creating,
            CellState::Active,
            CellState::Draining,
            CellState::Decommissioned,
        ]
        .iter()
        .map(|s| s.as_str())
        .collect();
        assert_eq!(labels.len(), 4);
    }

    #[test]
    fn error_message_non_empty() {
        let err = CellLifecycleError::InvalidTransition {
            from: CellState::Creating,
            command: CellLifecycleCommand::Drain,
        };
        assert!(!err.message().is_empty());

        let err2 = CellLifecycleError::ReactivationOfDecommissionedCell;
        assert!(!err2.message().is_empty());
    }
}
