//! The high-level machine state machine.
//!
//! Mirrors the lifecycle states a Talos node moves through as `machined`
//! sequences it: from `Booting` through `Running`, and the terminal
//! `Rebooting`/`ShuttingDown` states. The transitions here gate which
//! [`Sequence`] the [`crate::sequencer::Sequencer`] is allowed to begin.

use crate::error::{MachinedError, Result};
use crate::sequence::Sequence;

/// Coarse machine lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MachineState {
    /// Initial state before any sequence runs.
    Initializing,
    /// The Boot sequence is in progress.
    Booting,
    /// Boot completed; services are up and the node is operational.
    Running,
    /// An Install sequence is running (first boot on metal).
    Installing,
    /// An Upgrade sequence is running.
    Upgrading,
    /// A Reset sequence is running.
    Resetting,
    /// The machine is on its way down to reboot.
    Rebooting,
    /// The machine is on its way down to power off.
    ShuttingDown,
}

impl MachineState {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            MachineState::Initializing => "initializing",
            MachineState::Booting => "booting",
            MachineState::Running => "running",
            MachineState::Installing => "installing",
            MachineState::Upgrading => "upgrading",
            MachineState::Resetting => "resetting",
            MachineState::Rebooting => "rebooting",
            MachineState::ShuttingDown => "shuttingDown",
        }
    }

    /// Whether the machine is in a terminal (going-down) state from which no
    /// further sequence may start.
    pub fn is_terminal(self) -> bool {
        matches!(self, MachineState::Rebooting | MachineState::ShuttingDown)
    }

    /// Whether the node is fully operational.
    pub fn is_running(self) -> bool {
        self == MachineState::Running
    }

    /// The state the machine enters while running `seq`.
    fn running_state_for(seq: Sequence) -> Option<MachineState> {
        match seq {
            Sequence::Boot => Some(MachineState::Booting),
            Sequence::Install => Some(MachineState::Installing),
            Sequence::Upgrade => Some(MachineState::Upgrading),
            Sequence::Reset => Some(MachineState::Resetting),
            Sequence::Reboot => Some(MachineState::Rebooting),
            Sequence::Shutdown => Some(MachineState::ShuttingDown),
            // Staged/maintenance/noop don't change the coarse lifecycle state.
            Sequence::StageUpgrade | Sequence::MaintenanceUpgrade | Sequence::NoOp => None,
        }
    }
}

/// The machine lifecycle state machine driven by the sequencer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachine {
    state: MachineState,
    history: Vec<MachineState>,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    /// Create a state machine in [`MachineState::Initializing`].
    pub fn new() -> Self {
        StateMachine {
            state: MachineState::Initializing,
            history: vec![MachineState::Initializing],
        }
    }

    /// The current state.
    pub fn state(&self) -> MachineState {
        self.state
    }

    /// The ordered history of states the machine has occupied.
    pub fn history(&self) -> &[MachineState] {
        &self.history
    }

    /// Validate that a sequence may begin from the current state, returning the
    /// state the machine will occupy while it runs.
    ///
    /// Terminal states reject everything. A second `Boot` from `Running` is
    /// rejected (the machine has already booted).
    pub fn validate_start(&self, seq: Sequence) -> Result<MachineState> {
        if self.state.is_terminal() {
            return Err(MachinedError::sequence_not_allowed(format!(
                "machine is {}, cannot start {}",
                self.state.as_str(),
                seq.as_str()
            )));
        }
        if seq == Sequence::Boot && self.state == MachineState::Running {
            return Err(MachinedError::sequence_not_allowed(
                "machine already booted",
            ));
        }
        Ok(MachineState::running_state_for(seq).unwrap_or(self.state))
    }

    /// Transition into the running state for `seq` (after [`Self::validate_start`]).
    pub fn begin(&mut self, seq: Sequence) -> Result<()> {
        let next = self.validate_start(seq)?;
        self.transition(next);
        Ok(())
    }

    /// Record that the sequence `seq` finished successfully, moving the machine
    /// to its post-sequence resting state.
    pub fn complete(&mut self, seq: Sequence) -> Result<()> {
        let next = match seq {
            // A successful boot/upgrade/maintenance leaves the node running.
            Sequence::Boot
            | Sequence::Upgrade
            | Sequence::MaintenanceUpgrade
            | Sequence::Install => MachineState::Running,
            // Terminal sequences stay terminal.
            Sequence::Reboot => MachineState::Rebooting,
            Sequence::Shutdown => MachineState::ShuttingDown,
            // Reset returns to initializing (back to maintenance).
            Sequence::Reset => MachineState::Initializing,
            Sequence::StageUpgrade | Sequence::NoOp => self.state,
        };
        if self.state.is_terminal() && !next.is_terminal() {
            return Err(MachinedError::illegal_transition(
                self.state.as_str(),
                next.as_str(),
            ));
        }
        self.transition(next);
        Ok(())
    }

    fn transition(&mut self, next: MachineState) {
        if next != self.state {
            self.state = next;
        }
        self.history.push(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boots_to_running() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.state(), MachineState::Initializing);
        sm.begin(Sequence::Boot).unwrap();
        assert_eq!(sm.state(), MachineState::Booting);
        sm.complete(Sequence::Boot).unwrap();
        assert!(sm.state().is_running());
    }

    #[test]
    fn double_boot_rejected() {
        let mut sm = StateMachine::new();
        sm.begin(Sequence::Boot).unwrap();
        sm.complete(Sequence::Boot).unwrap();
        let err = sm.begin(Sequence::Boot).unwrap_err();
        assert_eq!(err.kind(), "sequence_not_allowed");
    }

    #[test]
    fn terminal_blocks_further_sequences() {
        let mut sm = StateMachine::new();
        sm.begin(Sequence::Boot).unwrap();
        sm.complete(Sequence::Boot).unwrap();
        sm.begin(Sequence::Reboot).unwrap();
        assert_eq!(sm.state(), MachineState::Rebooting);
        assert!(sm.state().is_terminal());
        assert!(sm.begin(Sequence::Upgrade).is_err());
    }

    #[test]
    fn reset_returns_to_initializing() {
        let mut sm = StateMachine::new();
        sm.begin(Sequence::Boot).unwrap();
        sm.complete(Sequence::Boot).unwrap();
        sm.begin(Sequence::Reset).unwrap();
        assert_eq!(sm.state(), MachineState::Resetting);
        sm.complete(Sequence::Reset).unwrap();
        assert_eq!(sm.state(), MachineState::Initializing);
    }

    #[test]
    fn history_is_recorded() {
        let mut sm = StateMachine::new();
        sm.begin(Sequence::Boot).unwrap();
        sm.complete(Sequence::Boot).unwrap();
        let h = sm.history();
        assert_eq!(h.first(), Some(&MachineState::Initializing));
        assert!(h.contains(&MachineState::Booting));
        assert_eq!(h.last(), Some(&MachineState::Running));
    }
}
