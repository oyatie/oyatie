//! The machine runtime abstraction, mirroring Talos `runtime.Runtime`.
//!
//! `RuntimeMode` distinguishes the environment the node boots into (mirrors
//! Talos `runtime.Mode`: metal, cloud, container, etc.). `MachineRuntime` is
//! the trait the sequencer queries to decide which sequences and phases are
//! valid, and to read machine identity (control-plane vs worker).

use os_kernel::{Error as CoreError, MachineType};
use std::str::FromStr;

/// The platform/runtime mode the machine is executing in.
///
/// Mirrors `siderolabs/talos` `runtime.Mode`. The mode constrains which
/// sequences are legal (for example, you cannot install or reset a
/// `Container` runtime — it has no disks).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMode {
    /// Bare-metal install (has block devices, can install/upgrade/reset).
    Metal,
    /// A cloud platform image (AWS, GCP, ...). Behaves like metal for disks.
    Cloud,
    /// Running inside a container (no disks, no reboot — used for tests/CI).
    Container,
    /// Metal agent mode.
    ///
    /// Source Talos `runtime.ModeMetalAgent` is not a container and is an
    /// agent mode. It retains host capabilities but does not require install.
    MetalAgent,
}

impl RuntimeMode {
    /// Canonical lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeMode::Metal => "metal",
            RuntimeMode::Cloud => "cloud",
            RuntimeMode::Container => "container",
            RuntimeMode::MetalAgent => "metal-agent",
        }
    }

    /// Whether the mode backs onto real block devices (install/reset apply).
    pub fn has_disks(self) -> bool {
        matches!(
            self,
            RuntimeMode::Metal | RuntimeMode::Cloud | RuntimeMode::MetalAgent
        )
    }

    /// Whether the runtime can actually reboot/poweroff the host.
    pub fn can_reboot(self) -> bool {
        // Containers can't reboot a kernel; they exit instead.
        !matches!(self, RuntimeMode::Container)
    }

    /// Whether this runtime is Talos container mode.
    pub fn in_container(self) -> bool {
        matches!(self, RuntimeMode::Container)
    }

    /// Whether this runtime is a Talos agent mode.
    pub fn is_agent(self) -> bool {
        matches!(self, RuntimeMode::MetalAgent)
    }

    /// Whether the node requires installation to persist (true on metal only).
    pub fn requires_install(self) -> bool {
        matches!(self, RuntimeMode::Metal)
    }
}

impl FromStr for RuntimeMode {
    type Err = CoreError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "metal" => Ok(RuntimeMode::Metal),
            "cloud" => Ok(RuntimeMode::Cloud),
            "container" => Ok(RuntimeMode::Container),
            "metal-agent" => Ok(RuntimeMode::MetalAgent),
            other => Err(CoreError::parse(format!("unknown runtime mode '{other}'"))),
        }
    }
}

/// The information the sequencer/controllers need from the runtime.
///
/// Mirrors the subset of Talos `runtime.Runtime` consulted by the sequencer:
/// the platform mode, the machine role, and whether a machine config has been
/// applied yet. Implementations back this with platform metadata, COSI
/// resources, etc.
pub trait MachineRuntime {
    /// The platform/runtime mode.
    fn mode(&self) -> RuntimeMode;

    /// The machine's role/type (controlplane, worker, init).
    fn machine_type(&self) -> MachineType;

    /// Whether a machine configuration has been applied. Many tasks are gated
    /// on this (e.g. you cannot bootstrap etcd before config is present).
    fn is_configured(&self) -> bool;

    /// Hostname, if known.
    fn hostname(&self) -> Option<&str>;

    /// Convenience: whether this is a control-plane node.
    fn is_control_plane(&self) -> bool {
        self.machine_type().is_control_plane()
    }
}

/// A simple in-memory [`MachineRuntime`] used by the sequencer in tests and as
/// a default driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryRuntime {
    mode: RuntimeMode,
    machine_type: MachineType,
    configured: bool,
    hostname: Option<String>,
}

impl InMemoryRuntime {
    /// Build a new runtime with no config applied yet.
    pub fn new(mode: RuntimeMode, machine_type: MachineType) -> Self {
        InMemoryRuntime {
            mode,
            machine_type,
            configured: false,
            hostname: None,
        }
    }

    /// Mark the machine as configured and set its hostname.
    pub fn with_config(mut self, hostname: impl Into<String>) -> Self {
        self.configured = true;
        self.hostname = Some(hostname.into());
        self
    }

    /// Mutate the configured flag at runtime (e.g. after applying config).
    pub fn set_configured(&mut self, configured: bool) {
        self.configured = configured;
    }
}

impl MachineRuntime for InMemoryRuntime {
    fn mode(&self) -> RuntimeMode {
        self.mode
    }
    fn machine_type(&self) -> MachineType {
        self.machine_type
    }
    fn is_configured(&self) -> bool {
        self.configured
    }
    fn hostname(&self) -> Option<&str> {
        self.hostname.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_capabilities() {
        assert!(RuntimeMode::Metal.has_disks());
        assert!(RuntimeMode::Metal.requires_install());
        assert!(!RuntimeMode::Metal.in_container());
        assert!(!RuntimeMode::Metal.is_agent());
        assert!(!RuntimeMode::Container.has_disks());
        assert!(!RuntimeMode::Container.can_reboot());
        assert!(RuntimeMode::Container.in_container());
        assert!(!RuntimeMode::Container.is_agent());
        assert!(!RuntimeMode::Cloud.requires_install());
        assert!(RuntimeMode::Cloud.can_reboot());
        assert!(RuntimeMode::MetalAgent.has_disks());
        assert!(RuntimeMode::MetalAgent.can_reboot());
        assert!(!RuntimeMode::MetalAgent.requires_install());
        assert!(!RuntimeMode::MetalAgent.in_container());
        assert!(RuntimeMode::MetalAgent.is_agent());
    }

    #[test]
    fn mode_parses() {
        assert_eq!("metal".parse::<RuntimeMode>().unwrap(), RuntimeMode::Metal);
        assert_eq!(
            " Container ".parse::<RuntimeMode>().unwrap(),
            RuntimeMode::Container
        );
        assert_eq!(
            "metal-agent".parse::<RuntimeMode>().unwrap(),
            RuntimeMode::MetalAgent
        );
        assert_eq!(RuntimeMode::MetalAgent.as_str(), "metal-agent");
        assert!("vm".parse::<RuntimeMode>().is_err());
    }

    #[test]
    fn runtime_reports_role_and_config() {
        let rt =
            InMemoryRuntime::new(RuntimeMode::Metal, MachineType::ControlPlane).with_config("cp-1");
        assert!(rt.is_control_plane());
        assert!(rt.is_configured());
        assert_eq!(rt.hostname(), Some("cp-1"));

        let mut worker = InMemoryRuntime::new(RuntimeMode::Cloud, MachineType::Worker);
        assert!(!worker.is_control_plane());
        assert!(!worker.is_configured());
        worker.set_configured(true);
        assert!(worker.is_configured());
    }
}
