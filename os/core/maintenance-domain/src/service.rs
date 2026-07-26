//! The maintenance-mode gRPC service surface and its state machine.
//!
//! In maintenance mode Talos exposes a deliberately small slice of
//! `machine.MachineService`. This module defines:
//!
//! - [`MaintenanceMethod`] — the methods the maintenance service answers, and
//!   the authorization/availability metadata for each.
//! - [`MaintenancePhase`] / [`MaintenanceState`] — the lifecycle of a node in
//!   maintenance, from booting through serving to leaving maintenance once a
//!   config is applied.
//! - [`MaintenanceService`] — the trait a maintenance backend implements.
//! - the request/response types for `Generate`, `Upgrade` and `Reset`.

use std::fmt;

use crate::config_apply::{ApplyConfigInput, ApplyConfigOutcome};

/// One RPC of the maintenance-mode service.
///
/// Maintenance mode answers only the methods needed to configure or recover a
/// node. Everything else returns "not available in maintenance mode", matching
/// the gate in `internal/app/maintenance/server`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaintenanceMethod {
    /// `Version` — report the maintenance image version. Read-only.
    Version,
    /// `Hostname` — report the node hostname. Read-only.
    Hostname,
    /// `Interfaces` / network introspection. Read-only.
    NetworkInfo,
    /// `GenerateConfiguration` — generate a machine config + talosconfig.
    GenerateConfiguration,
    /// `ApplyConfiguration` — accept, validate and persist a config. Mutating.
    ApplyConfiguration,
    /// `Upgrade` — upgrade the maintenance image. Mutating.
    Upgrade,
    /// `Reset` — wipe the machine and reboot. Mutating.
    Reset,
}

impl MaintenanceMethod {
    /// The fully-qualified gRPC method name.
    pub fn grpc_name(self) -> &'static str {
        match self {
            MaintenanceMethod::Version => "/machine.MachineService/Version",
            MaintenanceMethod::Hostname => "/machine.MachineService/Hostname",
            MaintenanceMethod::NetworkInfo => "/machine.MachineService/Interfaces",
            MaintenanceMethod::GenerateConfiguration => {
                "/machine.MachineService/GenerateConfiguration"
            }
            MaintenanceMethod::ApplyConfiguration => "/machine.MachineService/ApplyConfiguration",
            MaintenanceMethod::Upgrade => "/machine.MachineService/Upgrade",
            MaintenanceMethod::Reset => "/machine.MachineService/Reset",
        }
    }

    /// The short method name without the service prefix.
    pub fn short_name(self) -> &'static str {
        self.grpc_name().rsplit('/').next().unwrap_or("")
    }

    /// Whether the method mutates node state.
    pub fn is_mutating(self) -> bool {
        matches!(
            self,
            MaintenanceMethod::ApplyConfiguration
                | MaintenanceMethod::Upgrade
                | MaintenanceMethod::Reset
        )
    }

    /// Every method the maintenance service exposes.
    pub fn all() -> &'static [MaintenanceMethod] {
        &[
            MaintenanceMethod::Version,
            MaintenanceMethod::Hostname,
            MaintenanceMethod::NetworkInfo,
            MaintenanceMethod::GenerateConfiguration,
            MaintenanceMethod::ApplyConfiguration,
            MaintenanceMethod::Upgrade,
            MaintenanceMethod::Reset,
        ]
    }

    /// Parse a method from its fully-qualified or short name. Returns `None`
    /// for methods that exist on the full API but are **not** served in
    /// maintenance mode (e.g. `Bootstrap`, `EtcdMemberList`).
    pub fn parse(name: &str) -> Option<Self> {
        let short = name.rsplit('/').next().unwrap_or(name);
        let m = match short {
            "Version" => MaintenanceMethod::Version,
            "Hostname" => MaintenanceMethod::Hostname,
            "Interfaces" | "Network" => MaintenanceMethod::NetworkInfo,
            "GenerateConfiguration" => MaintenanceMethod::GenerateConfiguration,
            "ApplyConfiguration" => MaintenanceMethod::ApplyConfiguration,
            "Upgrade" => MaintenanceMethod::Upgrade,
            "Reset" => MaintenanceMethod::Reset,
            _ => return None,
        };
        Some(m)
    }
}

/// The lifecycle phase of a node while in (or leaving) maintenance mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenancePhase {
    /// Booting: deciding whether a config exists. No service yet.
    Booting,
    /// Generating self-signed PKI and binding the maintenance listener.
    Bootstrapping,
    /// Serving the maintenance API, waiting for a config.
    Serving,
    /// A valid config was applied; persisting and preparing to reboot.
    ConfigApplied,
    /// Rebooting / shutting the maintenance service down to enter normal boot.
    LeavingMaintenance,
    /// Maintenance was reset (node wiped); machine will reboot fresh.
    Reset,
}

impl MaintenancePhase {
    /// Whether the maintenance API is reachable in this phase.
    pub fn is_serving(self) -> bool {
        matches!(self, MaintenancePhase::Serving)
    }

    /// Whether this is a terminal phase (the maintenance service has handed off
    /// or is going away).
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            MaintenancePhase::LeavingMaintenance | MaintenancePhase::Reset
        )
    }

    /// Whether a transition from `self` to `next` is allowed.
    pub fn can_transition_to(self, next: MaintenancePhase) -> bool {
        use MaintenancePhase::*;
        matches!(
            (self, next),
            (Booting, Bootstrapping)
                | (Bootstrapping, Serving)
                | (Serving, ConfigApplied)
                | (Serving, Reset)
                | (ConfigApplied, LeavingMaintenance)
                | (Serving, LeavingMaintenance)
        )
    }
}

/// Errors returned by the maintenance service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaintenanceError {
    /// The requested method is not available in maintenance mode.
    MethodUnavailable(String),
    /// The node is not in a phase where this call is valid.
    WrongPhase {
        method: &'static str,
        phase: MaintenancePhase,
    },
    /// The apply-configuration flow failed; carries the display string.
    Apply(String),
    /// Generic invalid argument.
    InvalidArgument(String),
}

impl fmt::Display for MaintenanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaintenanceError::MethodUnavailable(m) => {
                write!(f, "method {m} is not available in maintenance mode")
            }
            MaintenanceError::WrongPhase { method, phase } => {
                write!(f, "method {method} not valid in phase {phase:?}")
            }
            MaintenanceError::Apply(s) => write!(f, "apply configuration failed: {s}"),
            MaintenanceError::InvalidArgument(s) => write!(f, "invalid argument: {s}"),
        }
    }
}

impl std::error::Error for MaintenanceError {}

/// The maintenance node state machine.
///
/// Tracks the current [`MaintenancePhase`], the node identity used to populate
/// SANs and config, and whether a config has been applied. Transitions are
/// validated so the server cannot, e.g., serve before bootstrapping PKI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceState {
    phase: MaintenancePhase,
    hostname: String,
    addresses: Vec<String>,
    version: String,
    config_applied: bool,
}

impl MaintenanceState {
    /// Create a freshly-booted maintenance node.
    pub fn new(hostname: impl Into<String>, version: impl Into<String>) -> Self {
        MaintenanceState {
            phase: MaintenancePhase::Booting,
            hostname: hostname.into(),
            addresses: Vec::new(),
            version: version.into(),
            config_applied: false,
        }
    }

    /// The current phase.
    pub fn phase(&self) -> MaintenancePhase {
        self.phase
    }

    /// The node hostname.
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// The maintenance image version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The node IP addresses (used to populate cert SANs).
    pub fn addresses(&self) -> &[String] {
        &self.addresses
    }

    /// Whether a config has been applied.
    pub fn config_applied(&self) -> bool {
        self.config_applied
    }

    /// Record a node address.
    pub fn add_address(&mut self, addr: impl Into<String>) {
        let addr = addr.into();
        if !addr.is_empty() && !self.addresses.iter().any(|a| a == &addr) {
            self.addresses.push(addr);
        }
    }

    /// Attempt a phase transition, returning an error if it is not permitted.
    pub fn transition_to(&mut self, next: MaintenancePhase) -> Result<(), MaintenanceError> {
        if self.phase == next {
            return Ok(());
        }
        if !self.phase.can_transition_to(next) {
            return Err(MaintenanceError::WrongPhase {
                method: "transition",
                phase: self.phase,
            });
        }
        if next == MaintenancePhase::ConfigApplied {
            self.config_applied = true;
        }
        self.phase = next;
        Ok(())
    }

    /// Whether the maintenance API is currently serving.
    pub fn is_serving(&self) -> bool {
        self.phase.is_serving()
    }
}

/// A `GenerateConfiguration` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateRequest {
    /// The cluster name to generate config for.
    pub cluster_name: String,
    /// The control-plane endpoint (`https://host:6443`).
    pub control_plane_endpoint: String,
    /// The machine type to generate (`controlplane`/`worker`/`init`).
    pub machine_type: String,
    /// The Kubernetes version, if requested.
    pub kubernetes_version: Option<String>,
}

impl GenerateRequest {
    /// Validate the request fields.
    pub fn validate(&self) -> Result<(), MaintenanceError> {
        if self.cluster_name.trim().is_empty() {
            return Err(MaintenanceError::InvalidArgument(
                "cluster name is required".into(),
            ));
        }
        if !self.control_plane_endpoint.starts_with("https://") {
            return Err(MaintenanceError::InvalidArgument(
                "control plane endpoint must be an https URL".into(),
            ));
        }
        if !matches!(
            self.machine_type.as_str(),
            "controlplane" | "worker" | "init"
        ) {
            return Err(MaintenanceError::InvalidArgument(format!(
                "invalid machine type '{}'",
                self.machine_type
            )));
        }
        Ok(())
    }
}

/// A `GenerateConfiguration` response: the machine config + a talosconfig.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateResponse {
    /// The generated machine configuration bytes.
    pub machine_config: Vec<u8>,
    /// The generated client talosconfig bytes.
    pub talosconfig: Vec<u8>,
}

/// An `Upgrade` request served in maintenance mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeRequest {
    /// The image reference to upgrade to.
    pub image: String,
    /// Whether to preserve data (no wipe).
    pub preserve: bool,
    /// Whether to stage the upgrade for the next reboot.
    pub stage: bool,
}

impl UpgradeRequest {
    /// Validate the request.
    pub fn validate(&self) -> Result<(), MaintenanceError> {
        if self.image.trim().is_empty() {
            return Err(MaintenanceError::InvalidArgument(
                "upgrade image is required".into(),
            ));
        }
        Ok(())
    }
}

/// An `Upgrade` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeResponse {
    /// The image that will be booted.
    pub image: String,
    /// Whether the node will reboot to apply.
    pub reboot: bool,
}

/// A `Reset` request served in maintenance mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetRequest {
    /// Whether to wipe all data (`graceful = false`, full wipe).
    pub wipe: bool,
    /// Whether to reboot after reset.
    pub reboot: bool,
}

/// The maintenance backend trait.
///
/// A real implementation wires into machined; the server in [`crate::server`]
/// provides an in-memory implementation over the state machine + boundaries.
pub trait MaintenanceService {
    /// Report the maintenance image version.
    fn version(&self) -> Result<String, MaintenanceError>;

    /// Report the node hostname.
    fn hostname(&self) -> Result<String, MaintenanceError>;

    /// Generate a machine config + talosconfig.
    fn generate_configuration(
        &self,
        req: &GenerateRequest,
    ) -> Result<GenerateResponse, MaintenanceError>;

    /// Apply (validate + persist) a configuration, returning the outcome.
    fn apply_configuration(
        &mut self,
        input: &ApplyConfigInput,
    ) -> Result<ApplyConfigOutcome, MaintenanceError>;

    /// Upgrade the maintenance image.
    fn upgrade(&mut self, req: &UpgradeRequest) -> Result<UpgradeResponse, MaintenanceError>;

    /// Reset (wipe) the machine.
    fn reset(&mut self, req: &ResetRequest) -> Result<(), MaintenanceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_parse_rejects_non_maintenance_methods() {
        assert_eq!(
            MaintenanceMethod::parse("ApplyConfiguration"),
            Some(MaintenanceMethod::ApplyConfiguration)
        );
        assert_eq!(
            MaintenanceMethod::parse("/machine.MachineService/Reset"),
            Some(MaintenanceMethod::Reset)
        );
        // Methods that exist on the full API but not in maintenance:
        assert_eq!(MaintenanceMethod::parse("Bootstrap"), None);
        assert_eq!(MaintenanceMethod::parse("EtcdMemberList"), None);
    }

    #[test]
    fn mutating_methods_classified() {
        assert!(MaintenanceMethod::ApplyConfiguration.is_mutating());
        assert!(MaintenanceMethod::Upgrade.is_mutating());
        assert!(MaintenanceMethod::Reset.is_mutating());
        assert!(!MaintenanceMethod::Version.is_mutating());
        assert!(!MaintenanceMethod::GenerateConfiguration.is_mutating());
    }

    #[test]
    fn all_methods_short_names_unique() {
        let mut names: Vec<&str> = MaintenanceMethod::all()
            .iter()
            .map(|m| m.short_name())
            .collect();
        names.sort_unstable();
        let len = names.len();
        names.dedup();
        assert_eq!(names.len(), len);
    }

    #[test]
    fn phase_transitions_follow_lifecycle() {
        use MaintenancePhase::*;
        assert!(Booting.can_transition_to(Bootstrapping));
        assert!(Bootstrapping.can_transition_to(Serving));
        assert!(Serving.can_transition_to(ConfigApplied));
        assert!(ConfigApplied.can_transition_to(LeavingMaintenance));
        assert!(Serving.can_transition_to(Reset));
        // illegal jumps:
        assert!(!Booting.can_transition_to(Serving));
        assert!(!Serving.can_transition_to(Booting));
        assert!(!ConfigApplied.can_transition_to(Serving));
    }

    #[test]
    fn state_machine_full_path_to_leaving() {
        let mut st = MaintenanceState::new("node-1", "v1.7.0");
        assert_eq!(st.phase(), MaintenancePhase::Booting);
        st.transition_to(MaintenancePhase::Bootstrapping).unwrap();
        st.transition_to(MaintenancePhase::Serving).unwrap();
        assert!(st.is_serving());
        assert!(!st.config_applied());
        st.transition_to(MaintenancePhase::ConfigApplied).unwrap();
        assert!(st.config_applied());
        st.transition_to(MaintenancePhase::LeavingMaintenance)
            .unwrap();
        assert!(st.phase().is_terminal());
    }

    #[test]
    fn state_machine_rejects_illegal_transition() {
        let mut st = MaintenanceState::new("node-1", "v1.7.0");
        let err = st.transition_to(MaintenancePhase::Serving).unwrap_err();
        assert!(matches!(err, MaintenanceError::WrongPhase { .. }));
        // unchanged
        assert_eq!(st.phase(), MaintenancePhase::Booting);
    }

    #[test]
    fn add_address_dedups_and_ignores_empty() {
        let mut st = MaintenanceState::new("node-1", "v1.7.0");
        st.add_address("10.0.0.5");
        st.add_address("10.0.0.5");
        st.add_address("");
        assert_eq!(st.addresses(), &["10.0.0.5".to_string()]);
    }

    #[test]
    fn generate_request_validation() {
        let ok = GenerateRequest {
            cluster_name: "test".into(),
            control_plane_endpoint: "https://10.0.0.1:6443".into(),
            machine_type: "controlplane".into(),
            kubernetes_version: None,
        };
        assert!(ok.validate().is_ok());

        let bad_endpoint = GenerateRequest {
            control_plane_endpoint: "http://10.0.0.1:6443".into(),
            ..ok.clone()
        };
        assert!(bad_endpoint.validate().is_err());

        let bad_type = GenerateRequest {
            machine_type: "router".into(),
            ..ok.clone()
        };
        assert!(bad_type.validate().is_err());

        let no_name = GenerateRequest {
            cluster_name: "  ".into(),
            ..ok
        };
        assert!(no_name.validate().is_err());
    }

    #[test]
    fn upgrade_request_requires_image() {
        assert!(
            UpgradeRequest {
                image: "".into(),
                preserve: false,
                stage: false
            }
            .validate()
            .is_err()
        );
        assert!(
            UpgradeRequest {
                image: "ghcr.io/talos:v1.7.1".into(),
                preserve: true,
                stage: false
            }
            .validate()
            .is_ok()
        );
    }
}
