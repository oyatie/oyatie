//! The v1alpha1 runtime state machine.
//!
//! Mirrors `siderolabs/talos` `internal/app/machined/pkg/runtime/v1alpha1`: the
//! concrete `runtime.Runtime` implementation machined uses at boot. It tracks
//! the coarse boot phase (the "machine status" the apid surfaces) and which
//! config has been applied. It is distinct from the [`crate::state_machine`]:
//! that models the sequencer's view, this models the v1alpha1 runtime's own
//! status reporting.

use crate::error::{MachinedError, Result};
use crate::runtime::RuntimeMode;
use os_kernel::MachineType;

/// The v1alpha1 runtime status, as surfaced over the machine API.
///
/// Mirrors the `MachineStatus` boot stages in Talos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum V1Alpha1State {
    /// Process just started, nothing done yet.
    Unknown,
    /// Awaiting machine configuration (maintenance mode).
    Maintenance,
    /// Config applied, services booting.
    Booting,
    /// Booted; node is running normally.
    Running,
    /// Shutting down.
    Shutdown,
}

impl V1Alpha1State {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            V1Alpha1State::Unknown => "unknown",
            V1Alpha1State::Maintenance => "maintenance",
            V1Alpha1State::Booting => "booting",
            V1Alpha1State::Running => "running",
            V1Alpha1State::Shutdown => "shutdown",
        }
    }

    /// Whether the runtime is reporting ready to serve workloads.
    pub fn is_ready(self) -> bool {
        self == V1Alpha1State::Running
    }

    /// Legal forward transitions of the v1alpha1 runtime status.
    fn can_transition_to(self, next: V1Alpha1State) -> bool {
        use V1Alpha1State::{Booting, Maintenance, Running, Shutdown, Unknown};
        match self {
            Unknown => matches!(next, Maintenance | Booting | Shutdown),
            Maintenance => matches!(next, Booting | Shutdown),
            Booting => matches!(next, Running | Shutdown | Maintenance),
            Running => matches!(next, Shutdown | Maintenance),
            Shutdown => false,
        }
    }
}

/// The concrete v1alpha1 runtime: mode, role, applied config and current
/// status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1Alpha1Runtime {
    mode: RuntimeMode,
    machine_type: MachineType,
    state: V1Alpha1State,
    config: Option<String>,
}

impl V1Alpha1Runtime {
    /// Create a runtime in the `Unknown` state with no config.
    pub fn new(mode: RuntimeMode, machine_type: MachineType) -> Self {
        V1Alpha1Runtime {
            mode,
            machine_type,
            state: V1Alpha1State::Unknown,
            config: None,
        }
    }

    /// The platform mode.
    pub fn mode(&self) -> RuntimeMode {
        self.mode
    }

    /// The machine role.
    pub fn machine_type(&self) -> MachineType {
        self.machine_type
    }

    /// The current v1alpha1 status.
    pub fn state(&self) -> V1Alpha1State {
        self.state
    }

    /// Whether config has been applied.
    pub fn is_configured(&self) -> bool {
        self.config.is_some()
    }

    /// The applied config blob, if any.
    pub fn config(&self) -> Option<&str> {
        self.config.as_deref()
    }

    /// Enter maintenance mode (awaiting config). Valid from `Unknown`.
    pub fn enter_maintenance(&mut self) -> Result<()> {
        self.transition(V1Alpha1State::Maintenance)
    }

    /// Apply machine configuration. Non-empty config is required.
    ///
    /// Applying config does not by itself change the status, but it is a
    /// precondition for [`Self::begin_boot`].
    pub fn set_config(&mut self, config: impl Into<String>) -> Result<()> {
        let cfg = config.into();
        if cfg.trim().is_empty() {
            return Err(MachinedError::Core(os_kernel::Error::invalid(
                "machine config is empty",
            )));
        }
        self.config = Some(cfg);
        Ok(())
    }

    /// Begin booting. Requires config to have been applied.
    pub fn begin_boot(&mut self) -> Result<()> {
        if !self.is_configured() {
            return Err(MachinedError::sequence_not_allowed(
                "cannot boot before config is applied",
            ));
        }
        self.transition(V1Alpha1State::Booting)
    }

    /// Mark the runtime as fully running.
    pub fn mark_running(&mut self) -> Result<()> {
        self.transition(V1Alpha1State::Running)
    }

    /// Begin shutdown.
    pub fn shutdown(&mut self) -> Result<()> {
        self.transition(V1Alpha1State::Shutdown)
    }

    fn transition(&mut self, next: V1Alpha1State) -> Result<()> {
        if !self.state.can_transition_to(next) {
            return Err(MachinedError::illegal_transition(
                self.state.as_str(),
                next.as_str(),
            ));
        }
        self.state = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt() -> V1Alpha1Runtime {
        V1Alpha1Runtime::new(RuntimeMode::Metal, MachineType::ControlPlane)
    }

    #[test]
    fn happy_path_boot() {
        let mut r = rt();
        r.enter_maintenance().unwrap();
        r.set_config("version: v1alpha1").unwrap();
        r.begin_boot().unwrap();
        r.mark_running().unwrap();
        assert!(r.state().is_ready());
        assert!(r.is_configured());
    }

    #[test]
    fn cannot_boot_without_config() {
        let mut r = rt();
        r.enter_maintenance().unwrap();
        let err = r.begin_boot().unwrap_err();
        assert_eq!(err.kind(), "sequence_not_allowed");
    }

    #[test]
    fn empty_config_rejected() {
        let mut r = rt();
        let err = r.set_config("   ").unwrap_err();
        assert_eq!(err.kind(), "core");
        assert!(!r.is_configured());
    }

    #[test]
    fn illegal_transition_rejected() {
        let mut r = rt();
        // Unknown -> Running is illegal (must go through booting).
        let err = r.mark_running().unwrap_err();
        assert_eq!(err.kind(), "illegal_transition");
    }

    #[test]
    fn shutdown_is_terminal() {
        let mut r = rt();
        r.enter_maintenance().unwrap();
        r.shutdown().unwrap();
        assert_eq!(r.state(), V1Alpha1State::Shutdown);
        assert!(r.enter_maintenance().is_err());
    }
}
