//! The `MachineService` API surface.
//!
//! Mirrors `pkg/machinery/api/machine/machine.proto`: lifecycle operations
//! (`Bootstrap`, `Reset`, `Reboot`, `Shutdown`, `Upgrade`), config application
//! (`ApplyConfiguration`), introspection (`Version`, `ServiceList`), and the
//! streaming `Logs`/`Dmesg`/`Events` calls. OS effects are modeled behind the
//! [`MachineBackend`] trait with an in-memory implementation.

use std::collections::BTreeMap;

use os_kernel::machine_type::MachineType;
use os_kernel::role::Role;
use os_kernel::version::Version;

use crate::common::{ApiError, Code, Data, Envelope, RequestContext};

/// The reset graceful/wipe options, mirroring `machine.ResetRequest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetRequest {
    /// Whether to gracefully cordon/drain before resetting.
    pub graceful: bool,
    /// Whether to reboot (vs. shut down) after the reset.
    pub reboot: bool,
    /// Per-partition wipe specs (label -> wipe mode).
    pub system_partitions_to_wipe: Vec<PartitionWipeSpec>,
}

impl Default for ResetRequest {
    fn default() -> Self {
        ResetRequest {
            graceful: true,
            reboot: true,
            system_partitions_to_wipe: Vec::new(),
        }
    }
}

/// A single partition wipe spec for [`ResetRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionWipeSpec {
    /// The partition label (e.g. `EPHEMERAL`, `STATE`).
    pub label: String,
    /// Whether to perform a secure (full overwrite) wipe.
    pub wipe: bool,
}

/// The reboot mode, mirroring `machine.RebootRequest.Mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebootMode {
    /// Normal full reboot via firmware.
    Default,
    /// In-place kexec into the new kernel without a firmware cycle.
    Powercycle,
}

impl RebootMode {
    /// Numeric wire value.
    pub fn as_i32(self) -> i32 {
        match self {
            RebootMode::Default => 0,
            RebootMode::Powercycle => 1,
        }
    }
}

/// How an applied configuration should take effect, mirroring
/// `machine.ApplyConfigurationRequest.Mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyMode {
    /// Reboot to apply.
    Reboot,
    /// Apply without rebooting where possible.
    Auto,
    /// Apply only if no reboot is required, else fail.
    NoReboot,
    /// Stage the config to be applied on next boot.
    Staged,
    /// Apply immediately to the running config (live).
    Try,
}

impl ApplyMode {
    /// Numeric wire value matching the proto enum order.
    pub fn as_i32(self) -> i32 {
        match self {
            ApplyMode::Reboot => 0,
            ApplyMode::Auto => 1,
            ApplyMode::NoReboot => 2,
            ApplyMode::Staged => 3,
            ApplyMode::Try => 4,
        }
    }

    /// Parse a `talosctl --mode` value.
    pub fn parse(s: &str) -> Result<Self, ApiError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "reboot" => Ok(ApplyMode::Reboot),
            "auto" => Ok(ApplyMode::Auto),
            "no-reboot" | "noreboot" => Ok(ApplyMode::NoReboot),
            "staged" | "stage" => Ok(ApplyMode::Staged),
            "try" => Ok(ApplyMode::Try),
            other => Err(ApiError::new(
                Code::InvalidArgument,
                format!("unknown apply mode '{other}'"),
            )),
        }
    }

    /// Whether applying in this mode implies a reboot.
    pub fn requires_reboot(self) -> bool {
        matches!(self, ApplyMode::Reboot)
    }
}

/// An `ApplyConfiguration` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyConfigurationRequest {
    /// The raw machine-config document bytes.
    pub data: Vec<u8>,
    /// How to apply.
    pub mode: ApplyMode,
    /// Validate the incoming config without persisting it.
    pub dry_run: bool,
}

/// The result of an `ApplyConfiguration` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyConfigurationResponse {
    /// Whether a reboot is needed to finish applying.
    pub mode_applied: ApplyMode,
    /// A human-readable warning/notice list.
    pub warnings: Vec<String>,
}

/// An `Upgrade` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeRequest {
    /// The target installer image reference.
    pub image: String,
    /// Whether to preserve user data (`STATE`/`EPHEMERAL`) across the upgrade.
    pub preserve: bool,
    /// Whether to stage the upgrade for the next boot.
    pub stage: bool,
    /// Force the upgrade even if pre-checks fail.
    pub force: bool,
}

/// The result of an `Upgrade` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeResponse {
    /// The actor id assigned to this upgrade (for tracking via Events).
    pub actor_id: String,
}

/// A `Version` reply, mirroring `machine.VersionInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    /// The Talos version string.
    pub tag: String,
    /// The build SHA.
    pub sha: String,
    /// The OS/arch this node runs.
    pub arch: String,
}

/// A single service's runtime state, mirroring `machine.ServiceInfo` +
/// `machine.ServiceHealth`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceInfo {
    /// The service id (e.g. `apid`, `etcd`, `kubelet`).
    pub id: String,
    /// The current lifecycle state (`Running`, `Finished`, `Failed`, ...).
    pub state: ServiceState,
    /// Whether the service is healthy (its health check passes).
    pub healthy: Option<bool>,
}

/// Service lifecycle states, mirroring Talos `system/services` runner states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// Defined but not yet started.
    Initialized,
    /// Waiting on dependencies/conditions.
    Waiting,
    /// Up and running.
    Running,
    /// Exited successfully (one-shot).
    Finished,
    /// Exited with an error.
    Failed,
    /// In the process of stopping.
    Stopping,
    /// Skipped (condition not met).
    Skipped,
}

impl ServiceState {
    /// The lowercase string form used in `talosctl services`.
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceState::Initialized => "Initialized",
            ServiceState::Waiting => "Waiting",
            ServiceState::Running => "Running",
            ServiceState::Finished => "Finished",
            ServiceState::Failed => "Failed",
            ServiceState::Stopping => "Stopping",
            ServiceState::Skipped => "Skipped",
        }
    }

    /// Whether the service has reached a terminal state.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            ServiceState::Finished | ServiceState::Failed | ServiceState::Skipped
        )
    }
}

/// A `Logs`/`Dmesg` streaming request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogsRequest {
    /// The namespace (e.g. `system`, `k8s`).
    pub namespace: String,
    /// The service/container id whose logs to stream.
    pub id: String,
    /// Whether to follow (stream live) vs. dump existing.
    pub follow: bool,
    /// Tail this many lines from the end (0 = all).
    pub tail_lines: u32,
}

/// The recorded lifecycle of the machine, used to gate operations the way
/// `machined`'s sequencer does (you cannot bootstrap twice, cannot upgrade
/// before installed, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineStage {
    /// Just booted, no config yet.
    Booting,
    /// Config installed, services starting.
    Installing,
    /// Running normally.
    Running,
    /// Control-plane node has been bootstrapped (etcd initialized).
    Bootstrapped,
    /// Upgrade in progress.
    Upgrading,
    /// Reset/reboot in progress.
    Resetting,
}

impl MachineStage {
    /// Whether bootstrap is permitted from this stage.
    pub fn can_bootstrap(self) -> bool {
        matches!(self, MachineStage::Running)
    }
}

/// OS-level effects of the machine service, modeled as a trait so tests can use
/// an in-memory backend instead of touching real syscalls/etcd/containerd.
pub trait MachineBackend {
    /// The node's reported version info.
    fn version(&self) -> VersionInfo;

    /// The node's machine type.
    fn machine_type(&self) -> MachineType;

    /// The current lifecycle stage.
    fn stage(&self) -> MachineStage;

    /// List the running system/k8s services.
    fn services(&self) -> Vec<ServiceInfo>;

    /// Initialize the etcd cluster on a control-plane node.
    fn bootstrap(&mut self) -> Result<(), ApiError>;

    /// Apply a machine configuration.
    fn apply_configuration(
        &mut self,
        req: &ApplyConfigurationRequest,
    ) -> Result<ApplyConfigurationResponse, ApiError>;

    /// Begin an upgrade, returning an actor id.
    fn upgrade(&mut self, req: &UpgradeRequest) -> Result<UpgradeResponse, ApiError>;

    /// Reboot the node.
    fn reboot(&mut self, mode: RebootMode) -> Result<(), ApiError>;

    /// Reset/wipe the node.
    fn reset(&mut self, req: &ResetRequest) -> Result<(), ApiError>;

    /// Read a slice of logs for a given service id.
    fn read_logs(&self, req: &LogsRequest) -> Result<Vec<u8>, ApiError>;
}

/// The `MachineService` itself: validates RBAC and stage preconditions, then
/// delegates to a [`MachineBackend`]. Mirrors `internal/app/machined`'s gRPC
/// service which wraps the runtime controller.
pub struct MachineService<B: MachineBackend> {
    backend: B,
}

impl<B: MachineBackend> MachineService<B> {
    /// Wrap a backend.
    pub fn new(backend: B) -> Self {
        MachineService { backend }
    }

    /// Access the underlying backend.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// `Version` is readable by anyone with read access.
    pub fn version(&self, ctx: &RequestContext) -> Result<Envelope<VersionInfo>, ApiError> {
        ctx.authorize(Role::Reader)?;
        let mut env = Envelope::new();
        env.push_ok(self.backend.version());
        Ok(env)
    }

    /// `ServiceList`.
    pub fn service_list(&self, ctx: &RequestContext) -> Result<Envelope<ServiceInfo>, ApiError> {
        ctx.authorize(Role::Reader)?;
        let mut env = Envelope::new();
        for svc in self.backend.services() {
            env.push_ok(svc);
        }
        Ok(env)
    }

    /// `Bootstrap`: control-plane only, and only from the `Running` stage.
    pub fn bootstrap(&mut self, ctx: &RequestContext) -> Result<(), ApiError> {
        ctx.authorize(Role::Admin)?;
        if !self.backend.machine_type().is_control_plane() {
            return Err(ApiError::new(
                Code::FailedPrecondition,
                "bootstrap is only valid on control-plane nodes",
            ));
        }
        if !self.backend.stage().can_bootstrap() {
            return Err(ApiError::new(
                Code::FailedPrecondition,
                format!("cannot bootstrap from stage {:?}", self.backend.stage()),
            ));
        }
        self.backend.bootstrap()
    }

    /// `ApplyConfiguration`.
    pub fn apply_configuration(
        &mut self,
        ctx: &RequestContext,
        req: &ApplyConfigurationRequest,
    ) -> Result<ApplyConfigurationResponse, ApiError> {
        ctx.authorize(Role::Admin)?;
        if req.data.is_empty() {
            return Err(ApiError::new(Code::InvalidArgument, "empty configuration"));
        }
        self.backend.apply_configuration(req)
    }

    /// `Upgrade`.
    pub fn upgrade(
        &mut self,
        ctx: &RequestContext,
        req: &UpgradeRequest,
    ) -> Result<UpgradeResponse, ApiError> {
        ctx.authorize(Role::Admin)?;
        if req.image.trim().is_empty() {
            return Err(ApiError::new(
                Code::InvalidArgument,
                "upgrade image is required",
            ));
        }
        self.backend.upgrade(req)
    }

    /// `Reboot`.
    pub fn reboot(&mut self, ctx: &RequestContext, mode: RebootMode) -> Result<(), ApiError> {
        ctx.authorize(Role::Admin)?;
        self.backend.reboot(mode)
    }

    /// `Reset`.
    pub fn reset(&mut self, ctx: &RequestContext, req: &ResetRequest) -> Result<(), ApiError> {
        ctx.authorize(Role::Admin)?;
        self.backend.reset(req)
    }

    /// `Logs`/`Dmesg`: stream chunks as a [`Data`] envelope.
    pub fn logs(&self, ctx: &RequestContext, req: &LogsRequest) -> Result<Vec<Data>, ApiError> {
        ctx.authorize(Role::Reader)?;
        let bytes = self.backend.read_logs(req)?;
        let mut out = Vec::new();
        // Chunk at 64 KiB the way apid frames streaming data.
        for chunk in bytes.chunks(64 * 1024).map(|c| c.to_vec()) {
            out.push(Data::local(chunk));
        }
        Ok(out)
    }
}

/// An in-memory [`MachineBackend`] for tests and the modeled control loop.
#[derive(Debug, Clone)]
pub struct InMemoryMachine {
    /// The node's version.
    pub version: VersionInfo,
    /// The node's machine type.
    pub machine_type: MachineType,
    /// The current lifecycle stage.
    pub stage: MachineStage,
    /// The running services keyed by id.
    pub services: BTreeMap<String, ServiceInfo>,
    /// The currently-applied config bytes.
    pub config: Vec<u8>,
    /// Captured log buffers keyed by `namespace/id`.
    pub logs: BTreeMap<String, Vec<u8>>,
    /// Monotonic counter used to mint actor ids.
    actor_seq: u64,
    /// Whether the node has been bootstrapped.
    pub bootstrapped: bool,
}

impl InMemoryMachine {
    /// A control-plane node in the `Running` stage with apid+etcd up.
    pub fn control_plane(tag: &str) -> Self {
        let mut services = BTreeMap::new();
        for id in ["apid", "etcd", "kubelet"] {
            services.insert(
                id.to_string(),
                ServiceInfo {
                    id: id.to_string(),
                    state: ServiceState::Running,
                    healthy: Some(true),
                },
            );
        }
        InMemoryMachine {
            version: VersionInfo {
                tag: tag.to_string(),
                sha: "deadbeef".to_string(),
                arch: "amd64".to_string(),
            },
            machine_type: MachineType::ControlPlane,
            stage: MachineStage::Running,
            services,
            config: Vec::new(),
            logs: BTreeMap::new(),
            actor_seq: 0,
            bootstrapped: false,
        }
    }

    /// A worker node in the `Running` stage.
    pub fn worker(tag: &str) -> Self {
        let mut m = InMemoryMachine::control_plane(tag);
        m.machine_type = MachineType::Worker;
        m.services.remove("etcd");
        m
    }

    /// Seed a log buffer for a service.
    pub fn set_logs(&mut self, namespace: &str, id: &str, data: impl Into<Vec<u8>>) {
        self.logs.insert(format!("{namespace}/{id}"), data.into());
    }
}

impl MachineBackend for InMemoryMachine {
    fn version(&self) -> VersionInfo {
        self.version.clone()
    }

    fn machine_type(&self) -> MachineType {
        self.machine_type
    }

    fn stage(&self) -> MachineStage {
        self.stage
    }

    fn services(&self) -> Vec<ServiceInfo> {
        self.services.values().cloned().collect()
    }

    fn bootstrap(&mut self) -> Result<(), ApiError> {
        if self.bootstrapped {
            return Err(ApiError::new(
                Code::AlreadyExists,
                "etcd already bootstrapped",
            ));
        }
        self.bootstrapped = true;
        self.stage = MachineStage::Bootstrapped;
        Ok(())
    }

    fn apply_configuration(
        &mut self,
        req: &ApplyConfigurationRequest,
    ) -> Result<ApplyConfigurationResponse, ApiError> {
        // Model a trivial validation: config must start with the v1alpha1 marker.
        // Dry-run still validates; it only skips persistence.
        if !req.data.starts_with(b"version: v1alpha1") {
            return Err(ApiError::new(
                Code::InvalidArgument,
                "configuration must declare 'version: v1alpha1'",
            ));
        }
        let mut warnings = Vec::new();
        if matches!(req.mode, ApplyMode::Try) {
            warnings.push("config applied in try mode; will revert on timeout".to_string());
        }
        if !req.dry_run {
            self.config = req.data.clone();
        }
        Ok(ApplyConfigurationResponse {
            mode_applied: req.mode,
            warnings,
        })
    }

    fn upgrade(&mut self, req: &UpgradeRequest) -> Result<UpgradeResponse, ApiError> {
        if !req.force && self.bootstrapped && self.services.contains_key("etcd") {
            // Model the etcd-member pre-check: a lone control-plane node refuses
            // a non-staged, non-forced upgrade if it would lose quorum.
            if !req.stage && self.machine_type.is_control_plane() {
                // Single-member assumption here; require staging or force.
                return Err(ApiError::new(
                    Code::FailedPrecondition,
                    "refusing to upgrade sole etcd member without --stage or --force",
                ));
            }
        }
        self.actor_seq += 1;
        if !req.stage {
            self.stage = MachineStage::Upgrading;
        }
        Ok(UpgradeResponse {
            actor_id: format!("upgrade-{}", self.actor_seq),
        })
    }

    fn reboot(&mut self, _mode: RebootMode) -> Result<(), ApiError> {
        self.stage = MachineStage::Resetting;
        Ok(())
    }

    fn reset(&mut self, req: &ResetRequest) -> Result<(), ApiError> {
        self.stage = MachineStage::Resetting;
        if req
            .system_partitions_to_wipe
            .iter()
            .any(|p| p.label == "STATE")
        {
            self.config.clear();
            self.bootstrapped = false;
        }
        Ok(())
    }

    fn read_logs(&self, req: &LogsRequest) -> Result<Vec<u8>, ApiError> {
        let key = format!("{}/{}", req.namespace, req.id);
        let buf = self
            .logs
            .get(&key)
            .ok_or_else(|| ApiError::new(Code::NotFound, format!("no logs for {key}")))?;
        if req.tail_lines == 0 {
            return Ok(buf.clone());
        }
        // Tail the requested number of newline-delimited lines.
        let text = String::from_utf8_lossy(buf);
        let lines: Vec<&str> = text.lines().collect();
        let start = lines.len().saturating_sub(req.tail_lines as usize);
        let tail = lines[start..].join("\n");
        Ok(tail.into_bytes())
    }
}

/// Recommended installer-image upgrade validation, mirroring the version-skew
/// rule machined enforces before accepting an upgrade.
pub fn validate_upgrade_versions(current: &Version, target: &Version) -> Result<(), ApiError> {
    if !current.is_upgrade_allowed_to(target) {
        return Err(ApiError::new(
            Code::FailedPrecondition,
            format!("upgrade from {current} to {target} violates the version-skew policy"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::role::RoleSet;

    fn admin() -> RequestContext {
        RequestContext::admin_local()
    }

    #[test]
    fn apply_mode_parse_and_wire() {
        assert_eq!(ApplyMode::parse("no-reboot").unwrap(), ApplyMode::NoReboot);
        assert_eq!(ApplyMode::parse("STAGE").unwrap(), ApplyMode::Staged);
        assert!(ApplyMode::parse("bogus").is_err());
        assert_eq!(ApplyMode::Try.as_i32(), 4);
        assert!(ApplyMode::Reboot.requires_reboot());
        assert!(!ApplyMode::Auto.requires_reboot());
    }

    #[test]
    fn version_and_services_require_read() {
        let svc = MachineService::new(InMemoryMachine::control_plane("v1.8.0"));
        let v = svc.version(&admin()).unwrap();
        assert_eq!(v.ok_payloads().next().unwrap().tag, "v1.8.0");

        let services = svc.service_list(&admin()).unwrap();
        assert_eq!(services.len(), 3);

        // No-role caller is denied.
        let nobody = RequestContext::with_roles(RoleSet::new());
        assert_eq!(
            svc.version(&nobody).unwrap_err().code,
            Code::PermissionDenied
        );
    }

    #[test]
    fn bootstrap_is_control_plane_only_and_once() {
        let mut cp = MachineService::new(InMemoryMachine::control_plane("v1.8.0"));
        cp.bootstrap(&admin()).unwrap();
        assert_eq!(cp.backend().stage, MachineStage::Bootstrapped);
        // Cannot bootstrap again (stage no longer Running).
        assert_eq!(
            cp.bootstrap(&admin()).unwrap_err().code,
            Code::FailedPrecondition
        );

        let mut worker = MachineService::new(InMemoryMachine::worker("v1.8.0"));
        assert_eq!(
            worker.bootstrap(&admin()).unwrap_err().code,
            Code::FailedPrecondition
        );
    }

    #[test]
    fn bootstrap_requires_admin() {
        let mut cp = MachineService::new(InMemoryMachine::control_plane("v1.8.0"));
        let reader = RequestContext::with_roles(RoleSet::from_roles([Role::Reader]));
        assert_eq!(
            cp.bootstrap(&reader).unwrap_err().code,
            Code::PermissionDenied
        );
    }

    #[test]
    fn apply_configuration_validates_marker() {
        let mut svc = MachineService::new(InMemoryMachine::control_plane("v1.8.0"));
        let bad = ApplyConfigurationRequest {
            data: b"nonsense".to_vec(),
            mode: ApplyMode::Auto,
            dry_run: false,
        };
        assert_eq!(
            svc.apply_configuration(&admin(), &bad).unwrap_err().code,
            Code::InvalidArgument
        );

        let good = ApplyConfigurationRequest {
            data: b"version: v1alpha1\nmachine: {}".to_vec(),
            mode: ApplyMode::Try,
            dry_run: false,
        };
        let resp = svc.apply_configuration(&admin(), &good).unwrap();
        assert_eq!(resp.mode_applied, ApplyMode::Try);
        assert_eq!(resp.warnings.len(), 1);
        assert_eq!(svc.backend().config, good.data);

        // Empty config rejected at the service layer.
        let empty = ApplyConfigurationRequest {
            data: Vec::new(),
            mode: ApplyMode::Auto,
            dry_run: false,
        };
        assert_eq!(
            svc.apply_configuration(&admin(), &empty).unwrap_err().code,
            Code::InvalidArgument
        );
    }

    #[test]
    fn apply_configuration_dry_run_validates_without_persisting() {
        let mut svc = MachineService::new(InMemoryMachine::control_plane("v1.8.0"));
        let bad = ApplyConfigurationRequest {
            data: b"nonsense".to_vec(),
            mode: ApplyMode::Auto,
            dry_run: true,
        };
        assert_eq!(
            svc.apply_configuration(&admin(), &bad).unwrap_err().code,
            Code::InvalidArgument
        );

        let good = ApplyConfigurationRequest {
            data: b"version: v1alpha1\nmachine: {}".to_vec(),
            mode: ApplyMode::Auto,
            dry_run: true,
        };
        let resp = svc.apply_configuration(&admin(), &good).unwrap();
        assert_eq!(resp.mode_applied, ApplyMode::Auto);
        assert!(svc.backend().config.is_empty());
    }

    #[test]
    fn upgrade_etcd_member_precheck() {
        let mut svc = MachineService::new(InMemoryMachine::control_plane("v1.8.0"));
        svc.bootstrap(&admin()).unwrap();

        let unstaged = UpgradeRequest {
            image: "ghcr.io/siderolabs/installer:v1.9.0".to_string(),
            preserve: true,
            stage: false,
            force: false,
        };
        assert_eq!(
            svc.upgrade(&admin(), &unstaged).unwrap_err().code,
            Code::FailedPrecondition
        );

        let staged = UpgradeRequest {
            stage: true,
            ..unstaged.clone()
        };
        let resp = svc.upgrade(&admin(), &staged).unwrap();
        assert_eq!(resp.actor_id, "upgrade-1");

        // Empty image rejected.
        let no_image = UpgradeRequest {
            image: "  ".to_string(),
            ..unstaged
        };
        assert_eq!(
            svc.upgrade(&admin(), &no_image).unwrap_err().code,
            Code::InvalidArgument
        );
    }

    #[test]
    fn logs_chunking_and_tail() {
        let mut backend = InMemoryMachine::control_plane("v1.8.0");
        backend.set_logs("system", "apid", "l1\nl2\nl3\nl4\nl5");
        let svc = MachineService::new(backend);

        let req = LogsRequest {
            namespace: "system".to_string(),
            id: "apid".to_string(),
            follow: false,
            tail_lines: 2,
        };
        let chunks = svc.logs(&admin(), &req).unwrap();
        let joined: Vec<u8> = chunks.into_iter().flat_map(|d| d.bytes).collect();
        assert_eq!(String::from_utf8(joined).unwrap(), "l4\nl5");

        let missing = LogsRequest {
            namespace: "system".to_string(),
            id: "ghost".to_string(),
            follow: false,
            tail_lines: 0,
        };
        assert_eq!(
            svc.logs(&admin(), &missing).unwrap_err().code,
            Code::NotFound
        );
    }

    #[test]
    fn reset_wipes_state_partition() {
        let mut backend = InMemoryMachine::control_plane("v1.8.0");
        backend.bootstrapped = true;
        backend.config = b"version: v1alpha1".to_vec();
        let mut svc = MachineService::new(backend);

        let req = ResetRequest {
            graceful: true,
            reboot: false,
            system_partitions_to_wipe: vec![PartitionWipeSpec {
                label: "STATE".to_string(),
                wipe: true,
            }],
        };
        svc.reset(&admin(), &req).unwrap();
        assert!(svc.backend().config.is_empty());
        assert!(!svc.backend().bootstrapped);
        assert_eq!(svc.backend().stage, MachineStage::Resetting);
    }

    #[test]
    fn upgrade_version_skew_validation() {
        assert!(validate_upgrade_versions(&Version::new(1, 8, 0), &Version::new(1, 9, 5)).is_ok());
        let err =
            validate_upgrade_versions(&Version::new(1, 8, 0), &Version::new(1, 11, 0)).unwrap_err();
        assert_eq!(err.code, Code::FailedPrecondition);
    }

    #[test]
    fn service_state_helpers() {
        assert_eq!(ServiceState::Running.as_str(), "Running");
        assert!(ServiceState::Finished.is_terminal());
        assert!(!ServiceState::Running.is_terminal());
    }
}
