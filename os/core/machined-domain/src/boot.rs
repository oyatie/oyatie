//! A real, callable boot **sequencer** that a PID1 process (talos-init) can
//! drive, mirroring `siderolabs/talos`'s machined sequencer.
//!
//! Unlike the generic [`crate::sequencer::Sequencer`] (which is query-only and
//! takes caller-supplied phases), this module ships a *concrete* boot sequence:
//! an ordered list of [`BootPhase`]s, each holding ordered [`BootTask`]s, that
//! is executed for real against a [`Runtime`] / [`PlatformOps`] trait object.
//!
//! The `Runtime` trait is the OS boundary. It is deliberately **object-safe**
//! (`&mut dyn Runtime`) so:
//!
//! - talos-init implements it for real with `libc` (`mount(2)`, `sethostname(2)`,
//!   writing `/proc/sys/...`, `fork`/`exec` of services), and
//! - this crate implements an in-memory [`FakeRuntime`] for tests.
//!
//! The sequencer reports progress through a [`ProgressLogger`] trait so the
//! caller can print each phase/task as it runs (talos-init wires this to its
//! kmsg/console logger; tests capture it into a vector).
//!
//! ## What talos-init calls
//!
//! ```ignore
//! let mut rt = LibcRuntime::new(/* platform, machine type */);
//! let mut log = KmsgLogger;
//! let seq = BootSequencer::new();
//! seq.run_boot(&mut rt, &mut log)?;   // drives every phase in order
//! ```
//!
//! The modeled boot phases (a simplified-but-real subset of Talos's Boot
//! sequence) are, in order:
//!
//! 1. [`BootPhaseId::MountPseudoFs`]     — mount `proc`/`sys`/`dev`/`run`/…
//!    This MUST come first: upstream Talos mounts the kernel pseudo-filesystems
//!    in PID 1 (`internal/app/init/main.go`) before machined runs, so that
//!    `/proc/sys/...` exists by the time KSPP sysctls are enforced.
//! 2. [`BootPhaseId::SystemDirectories`] — create `/system`, `/var`, … and
//!    enforce KSPP sysctls (writes `/proc/sys/...`, which only exists once the
//!    pseudo-filesystems above are mounted).
//! 3. [`BootPhaseId::LoadConfig`]        — load + validate the machine config.
//! 4. [`BootPhaseId::SetHostname`]       — `sethostname(2)` from config.
//! 5. [`BootPhaseId::Network`]           — bring loopback up, links up.
//! 6. [`BootPhaseId::StartServices`]     — spawn the node services.
//! 7. [`BootPhaseId::Running`]           — mark the machine running.
//!
//! Historical bug: `SystemDirectories` (which runs `enforceKSPP`) used to be
//! phase 1, *before* `MountPseudoFs`. On a real kernel `/proc/sys/...` did not
//! yet exist, so every sysctl write failed with `ENOENT` and was silently
//! skipped (best effort) — KSPP hardening was never actually applied. Mounting
//! the pseudo-filesystems first fixes this.

use crate::error::{MachinedError, Result};
use crate::supervisor::{ServiceLauncher, Supervisor, SupervisorRegistrydServiceManager};
use std::collections::BTreeMap;
use os_kernel::MachineType;
use os_runtime_cri_domain::{
    ImageCacheRuntimePlan, RegistrydAction, RegistrydRuntimeAdapter,
    RegistrydServiceExecutionStatus,
};

// ---------------------------------------------------------------------------
// Platform value types (dep-free models of the syscall arguments).
// ---------------------------------------------------------------------------

/// A pseudo-filesystem mount request, mirroring the args to `mount(2)`.
///
/// The real [`Runtime`] turns this into a `mount(2)` call; the fake records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountRequest {
    /// Source (e.g. `"proc"`, `"sysfs"`, `"devtmpfs"`).
    pub source: String,
    /// Target mount point (e.g. `"/proc"`).
    pub target: String,
    /// Filesystem type (e.g. `"proc"`, `"sysfs"`, `"tmpfs"`).
    pub fstype: String,
    /// Comma-separated mount flags (e.g. `"nosuid,nodev,noexec"`); modeled as a
    /// string to stay crate-free and to match what the init logs.
    pub flags: String,
}

impl MountRequest {
    /// Build a mount request.
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        fstype: impl Into<String>,
        flags: impl Into<String>,
    ) -> Self {
        MountRequest {
            source: source.into(),
            target: target.into(),
            fstype: fstype.into(),
            flags: flags.into(),
        }
    }
}

/// The restart policy for a [`BootService`], mirroring Talos service restart
/// behavior (`system/runner`): run once, always restart, or restart only on
/// failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartPolicy {
    /// Never restart; run to completion once (one-shot, e.g. a setup job).
    Never,
    /// Always restart when the process exits, up to the restart budget.
    Always,
    /// Restart only if the process exited non-zero / failed.
    OnFailure,
}

impl RestartPolicy {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            RestartPolicy::Never => "never",
            RestartPolicy::Always => "always",
            RestartPolicy::OnFailure => "on-failure",
        }
    }

    /// Whether a service with this policy should be restarted after a failure
    /// (assuming restart budget remains).
    pub fn restart_on_failure(self) -> bool {
        matches!(self, RestartPolicy::Always | RestartPolicy::OnFailure)
    }

    /// Whether a service with this policy should be restarted after a clean
    /// (exit-zero) finish.
    pub fn restart_on_success(self) -> bool {
        matches!(self, RestartPolicy::Always)
    }
}

/// The lifecycle state of a [`BootService`].
///
/// Mirrors the Talos service runner state machine, simplified to the states the
/// boot sequencer drives: `Preparing -> Running -> Finished | Failed`, with a
/// restart edge back to `Preparing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceStatus {
    /// Defined but not yet started.
    Initialized,
    /// Being launched (image pull / config write / exec in flight).
    Preparing,
    /// The process is up.
    Running,
    /// The process exited cleanly (or was stopped) and will not restart.
    Finished,
    /// The process failed; it may restart if it has budget.
    Failed,
}

impl ServiceStatus {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceStatus::Initialized => "initialized",
            ServiceStatus::Preparing => "preparing",
            ServiceStatus::Running => "running",
            ServiceStatus::Finished => "finished",
            ServiceStatus::Failed => "failed",
        }
    }

    /// Whether the process is currently executing.
    pub fn is_running(self) -> bool {
        self == ServiceStatus::Running
    }

    /// Whether this is a terminal resting state.
    pub fn is_terminal(self) -> bool {
        matches!(self, ServiceStatus::Finished)
    }

    /// The legal transition edges of the service state machine.
    pub fn can_transition_to(self, next: ServiceStatus) -> bool {
        use ServiceStatus::{Failed, Finished, Initialized, Preparing, Running};
        match self {
            Initialized => matches!(next, Preparing | Finished),
            Preparing => matches!(next, Running | Failed),
            Running => matches!(next, Finished | Failed),
            // A failed service may be restarted (back to preparing) or given up
            // on (finished).
            Failed => matches!(next, Preparing | Finished),
            Finished => false,
        }
    }
}

/// A long-lived service definition plus its live lifecycle state, as driven by
/// the boot sequencer.
///
/// Mirrors `internal/app/machined/pkg/system`'s service: a `name`, the
/// `command` (argv) used to spawn it, a [`RestartPolicy`], and a restart budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootService {
    name: String,
    command: Vec<String>,
    policy: RestartPolicy,
    status: ServiceStatus,
    pid: Option<u32>,
    restarts: u32,
    max_restarts: u32,
}

impl BootService {
    /// Define a service: a name, an argv command, and a restart policy.
    pub fn new(
        name: impl Into<String>,
        command: impl IntoIterator<Item = impl Into<String>>,
        policy: RestartPolicy,
    ) -> Self {
        BootService {
            name: name.into(),
            command: command.into_iter().map(Into::into).collect(),
            policy,
            status: ServiceStatus::Initialized,
            pid: None,
            restarts: 0,
            max_restarts: 3,
        }
    }

    /// Set the maximum number of restarts before the service is permanently
    /// failed.
    pub fn with_max_restarts(mut self, max: u32) -> Self {
        self.max_restarts = max;
        self
    }

    /// The service name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The argv used to launch the service.
    pub fn command(&self) -> &[String] {
        &self.command
    }

    /// The restart policy.
    pub fn policy(&self) -> RestartPolicy {
        self.policy
    }

    /// The current lifecycle state.
    pub fn status(&self) -> ServiceStatus {
        self.status
    }

    /// The OS pid, once spawned.
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// How many times the service has restarted.
    pub fn restarts(&self) -> u32 {
        self.restarts
    }

    /// Attempt a validated state transition. Errors on an illegal edge.
    pub fn transition_to(&mut self, next: ServiceStatus) -> Result<()> {
        if !self.status.can_transition_to(next) {
            return Err(MachinedError::illegal_transition(
                self.status.as_str(),
                next.as_str(),
            ));
        }
        if self.status == ServiceStatus::Failed && next == ServiceStatus::Preparing {
            self.restarts += 1;
        }
        if next != ServiceStatus::Running {
            // Leaving / not-yet a running process: clear the recorded pid.
            self.pid = None;
        }
        self.status = next;
        Ok(())
    }

    /// Record that the service is now running under the given pid (sets state to
    /// [`ServiceStatus::Running`]).
    pub fn mark_running(&mut self, pid: u32) -> Result<()> {
        self.transition_to(ServiceStatus::Running)?;
        self.pid = Some(pid);
        Ok(())
    }

    /// Whether the service may be restarted given its policy and remaining
    /// budget. `failed` indicates whether the last exit was a failure.
    pub fn should_restart(&self, failed: bool) -> bool {
        if self.restarts >= self.max_restarts {
            return false;
        }
        if failed {
            self.policy.restart_on_failure()
        } else {
            self.policy.restart_on_success()
        }
    }
}

// ---------------------------------------------------------------------------
// The Runtime / PlatformOps boundary.
// ---------------------------------------------------------------------------

/// The OS / platform operations boot tasks perform, mirroring the syscall
/// boundary Talos machined uses.
///
/// **This is the trait talos-init implements with `libc`.** It is object-safe
/// (every method takes `&mut self`, no generics, no `Self` by value) so the
/// sequencer can drive it as `&mut dyn Runtime`.
///
/// Tasks never perform syscalls directly: they call these methods, so the same
/// task list runs against a real `libc` runtime in production and against
/// [`FakeRuntime`] in tests.
///
/// `PlatformOps` is a type alias for `Runtime`; both names are exported because
/// Talos splits "platform" reads from "runtime" mutations, but here they are the
/// same trait object.
pub trait Runtime {
    /// The platform name (e.g. `"metal"`, `"aws"`, `"container"`). Used by tasks
    /// to gate platform-specific behavior and by the logger.
    fn platform(&self) -> &str;

    /// Whether the runtime backs onto real block devices (affects disk mounts).
    fn has_disks(&self) -> bool {
        true
    }

    /// Create a system directory (e.g. `/system`, `/var`), like `mkdir -p`.
    fn make_directory(&mut self, path: &str) -> Result<()>;

    /// Write a sysctl / KSPP knob, e.g. `("kernel.kptr_restrict", "1")`. The
    /// real runtime writes `/proc/sys/<dotted-as-path>`.
    ///
    /// This is **best effort** (see [`SysctlOutcome`]): on a privileged kernel
    /// it returns `Ok(SysctlOutcome::Applied)`; under a sandbox / unprivileged
    /// container / read-only `/proc` it returns `Ok(SysctlOutcome::Skipped(_))`
    /// so the boot continues. Only genuinely unexpected errors return `Err`.
    fn write_sysctl(&mut self, key: &str, value: &str) -> Result<SysctlOutcome>;

    /// Mount a (pseudo) filesystem. The real runtime calls `mount(2)`.
    fn mount(&mut self, req: &MountRequest) -> Result<()>;

    /// Load the machine configuration from its platform source, returning the
    /// raw config bytes/string. The real runtime reads disk / metadata service;
    /// the fake returns canned config.
    fn load_config(&mut self) -> Result<String>;

    /// Bring up any platform networking needed before config-source fetches.
    ///
    /// Most platforms can skip this. AWS needs a pre-IMDS bootstrap that brings
    /// the primary link up and starts DHCP operators before metadata is read, so
    /// IPv6-only instances can reach the IPv6 IMDS endpoint.
    fn bootstrap_network_for_config(&mut self) -> Result<bool> {
        Ok(false)
    }

    /// Apply / record the loaded machine config (after validation). Returns the
    /// machine hostname declared by the config, if any.
    fn apply_config(&mut self, raw: &str) -> Result<Option<String>>;

    /// Set the system hostname via `sethostname(2)`.
    fn set_hostname(&mut self, hostname: &str) -> Result<()>;

    /// Bring a network link up (e.g. `"lo"`, `"eth0"`). The real runtime issues
    /// the rtnetlink `RTM_NEWLINK`/`IFF_UP`.
    fn link_up(&mut self, iface: &str) -> Result<()>;

    /// Pump the boot-owned COSI network bridge after the primary links are up.
    ///
    /// The default keeps third-party/test runtimes source-compatible; talos-init
    /// overrides this to instantiate the Rust controller graph and publish live
    /// `network/LinkStatuses.net.talos.dev/*` resources before services start.
    fn boot_cosi_network_bridge(&mut self) -> Result<()> {
        Ok(())
    }

    /// Execute boot/runtime adapters for image-cache plans after the COSI graph
    /// has projected source state and before normal node services are spawned.
    ///
    /// The default is intentionally inert so third-party runtimes keep their
    /// current behavior. Runtimes that own service-manager authority can
    /// override this and call adapters such as [`RegistrydRuntimeAdapter`]
    /// through [`SupervisorRegistrydServiceManager`].
    fn run_image_cache_runtime_adapters(&mut self) -> Result<()> {
        Ok(())
    }

    /// Spawn a service process, returning its pid. The real runtime
    /// `fork`+`exec`s `service.command()`; the fake assigns a synthetic pid.
    fn spawn_service(&mut self, service: &BootService) -> Result<u32>;

    /// Emit a free-form log/console line (used by tasks for progress notes).
    fn log(&mut self, _line: &str) {}
}

/// Alias: Talos separates platform reads from runtime mutations; here both are
/// the same object-safe trait. talos-init may implement `Runtime` and refer to
/// it as `PlatformOps` interchangeably.
pub type PlatformOps = dyn Runtime;

// ---------------------------------------------------------------------------
// Progress logging.
// ---------------------------------------------------------------------------

/// Receives progress callbacks as the sequencer runs, so the caller can print
/// each phase/task. Mirrors how Talos machined logs `task X (Y/Z)`.
pub trait ProgressLogger {
    /// A phase is about to run.
    fn phase_start(&mut self, _index: usize, _total: usize, _phase: BootPhaseId) {}
    /// A task within a phase is about to run.
    fn task_start(&mut self, _phase: BootPhaseId, _task: &str) {}
    /// A task finished with the given outcome.
    fn task_done(&mut self, _phase: BootPhaseId, _task: &str, _outcome: TaskOutcome) {}
    /// A phase finished.
    fn phase_done(&mut self, _phase: BootPhaseId) {}
    /// The whole boot finished successfully.
    fn boot_done(&mut self) {}
    /// A task failed; the boot is aborting.
    fn task_failed(&mut self, _phase: BootPhaseId, _task: &str, _err: &MachinedError) {}
}

/// A [`ProgressLogger`] that discards everything (default for callers that don't
/// care).
#[derive(Debug, Default)]
pub struct NullLogger;
impl ProgressLogger for NullLogger {}

/// A [`ProgressLogger`] that records a human-readable transcript line per event,
/// for tests and for `talosctl dmesg`-style replay.
#[derive(Debug, Default)]
pub struct RecordingLogger {
    /// The recorded transcript, in order.
    pub lines: Vec<String>,
}

impl ProgressLogger for RecordingLogger {
    fn phase_start(&mut self, index: usize, total: usize, phase: BootPhaseId) {
        self.lines
            .push(format!("phase {}/{}: {}", index + 1, total, phase.as_str()));
    }
    fn task_start(&mut self, phase: BootPhaseId, task: &str) {
        self.lines
            .push(format!("  task {}:{} start", phase.as_str(), task));
    }
    fn task_done(&mut self, phase: BootPhaseId, task: &str, outcome: TaskOutcome) {
        self.lines.push(format!(
            "  task {}:{} {}",
            phase.as_str(),
            task,
            outcome.as_str()
        ));
    }
    fn phase_done(&mut self, phase: BootPhaseId) {
        self.lines.push(format!("phase {} done", phase.as_str()));
    }
    fn boot_done(&mut self) {
        self.lines.push("boot complete".to_string());
    }
    fn task_failed(&mut self, phase: BootPhaseId, task: &str, err: &MachinedError) {
        self.lines.push(format!(
            "  task {}:{} FAILED: {}",
            phase.as_str(),
            task,
            err
        ));
    }
}

// ---------------------------------------------------------------------------
// Tasks and phases.
// ---------------------------------------------------------------------------

/// The outcome of running a [`BootTask`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskOutcome {
    /// The task did work.
    Done,
    /// The task's postcondition already held; nothing was done.
    Skipped,
}

impl TaskOutcome {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            TaskOutcome::Done => "done",
            TaskOutcome::Skipped => "skipped",
        }
    }
}

/// The result of attempting a single best-effort sysctl write.
///
/// KSPP sysctl hardening is *best effort* (matching upstream Talos): on a real
/// privileged kernel every knob is `Applied`, but under a sandbox (gVisor /
/// runsc), an unprivileged container, or a read-only `/proc`, individual knobs
/// may be unwritable. Those cases are reported as [`SysctlOutcome::Skipped`]
/// (carrying the errno name) rather than aborting the boot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SysctlOutcome {
    /// The knob was written successfully.
    Applied,
    /// The knob could not be written for a benign, expected reason
    /// (sandbox / unprivileged / read-only / absent). The string is the
    /// classified errno name, e.g. `"EPERM"`.
    Skipped(&'static str),
}

impl SysctlOutcome {
    /// Whether the write actually happened.
    pub fn is_applied(&self) -> bool {
        matches!(self, SysctlOutcome::Applied)
    }

    /// Whether the write was skipped (best effort).
    pub fn is_skipped(&self) -> bool {
        matches!(self, SysctlOutcome::Skipped(_))
    }
}

// Linux errno values that mean a sysctl write should be treated as
// *best effort* and skipped rather than aborting the boot. These are the
// sandbox / unprivileged / read-only / absent cases. The numeric values are
// fixed by the Linux ABI (talos-machined intentionally carries no libc
// dependency, so we encode them directly).
const EPERM: i32 = 1; // operation not permitted (unprivileged)
const ENOENT: i32 = 2; // knob absent on this kernel
const ENODEV: i32 = 19; // no such device / fstype unsupported (sandbox)
const EBUSY: i32 = 16; // resource busy — target already mounted
const EACCES: i32 = 13; // permission denied
const EROFS: i32 = 30; // read-only filesystem (e.g. gVisor read-only /proc)
const ENOSYS: i32 = 38; // syscall not implemented (Sentry stubs it out)
const EOPNOTSUPP: i32 = 95; // operation not supported on this object (sandbox)
const ENOTTY: i32 = 25; // inappropriate ioctl — Sentry doesn't back this ioctl

/// Best-effort sysctl classifier.
///
/// Given the raw OS errno of a failed sysctl write, decide whether the failure
/// is one of the benign "skip and continue" cases (sandbox / unprivileged /
/// read-only / absent) or a genuinely unexpected error that must propagate.
///
/// Returns `Some(errno_name)` if the write should be **skipped** (best effort),
/// or `None` if the error is unexpected and should **fail** the task/boot.
///
/// This is pure and host-testable: it never touches the filesystem and takes
/// the errno as a plain integer so it can be exercised on any platform.
pub fn sysctl_skip_reason(errno: i32) -> Option<&'static str> {
    match errno {
        EPERM => Some("EPERM"),
        EACCES => Some("EACCES"),
        EROFS => Some("EROFS"),
        ENOENT => Some("ENOENT"),
        _ => None,
    }
}

/// Best-effort pseudo-filesystem mount classifier (mirrors
/// [`sysctl_skip_reason`] for `mount(2)`).
///
/// Under a sandbox the kernel pseudo-filesystems are frequently *already
/// provided* and re-mounting them over the top is forbidden: gVisor's Sentry
/// presents its own `/proc` and `/sys` and rejects a mount on top of them, a
/// kernel-supplied `devtmpfs` is already on `/dev`, and an unprivileged
/// container cannot mount at all. Those cases come back as a small set of
/// expected errnos and must be treated as a benign skip (logged, boot
/// continues) rather than a fatal error. On a real privileged kernel the mount
/// simply succeeds and this classifier is never consulted.
///
/// Given the raw OS errno of a failed `mount(2)`, returns `Some(errno_name)` if
/// the failure is one of the benign "skip and continue" cases, or `None` if the
/// error is genuinely unexpected and should **fail** the task/boot.
///
/// Pure and host-testable: it never touches the filesystem and takes the errno
/// as a plain integer so it can be exercised on any platform.
pub fn mount_skip_reason(errno: i32) -> Option<&'static str> {
    match errno {
        EPERM => Some("EPERM"),   // unprivileged container cannot mount
        EACCES => Some("EACCES"), // mount denied by sandbox policy
        EBUSY => Some("EBUSY"),   // target already mounted (kernel devtmpfs, etc.)
        ENODEV => Some("ENODEV"), // fstype unsupported / Sentry already provides it
        _ => None,
    }
}

/// Best-effort classifier for the privileged machine-bring-up syscalls:
/// `sethostname(2)`, network link-up (rtnetlink `RTM_NEWLINK`/`IFF_UP`),
/// address assignment (`add_ipv4` / `RTM_NEWADDR`), and kernel-module load
/// (`finit_module(2)`).
///
/// On a real privileged kernel (metal) every one of these succeeds and this
/// classifier is never consulted, so metal stays full-fidelity. Under a sandbox
/// (gVisor's Sentry, an unprivileged container) the host kernel already owns the
/// hostname/network/namespace or stubs the syscall out, so the operation comes
/// back with one of a small set of "already provided / forbidden by sandbox"
/// errnos: `EPERM` / `EACCES` (not permitted), `ENOSYS` (Sentry never
/// implemented the syscall), `EOPNOTSUPP` (object does not support it), or
/// `ENOTTY` (the Sentry does not back the device ioctl used by the
/// `SIOCSIFFLAGS` link-up fallback — gVisor returns "inappropriate ioctl for
/// device" instead of EPERM for an unsupported `AF_INET` socket ioctl). Those
/// must be treated as a benign skip (logged `[seq] <task>: skipped (<errno>,
/// sandbox)`, boot continues) rather than aborting the node bring-up.
///
/// Given the raw OS errno, returns `Some(errno_name)` if the failure is one of
/// the benign sandbox cases (skip and continue), or `None` if the error is
/// genuinely unexpected and should **fail** the task/boot.
///
/// Pure and host-testable: it never performs any syscall and takes the errno as
/// a plain integer so it can be exercised on any platform.
pub fn privileged_op_skip_reason(errno: i32) -> Option<&'static str> {
    match errno {
        EPERM => Some("EPERM"),           // unprivileged: kernel owns the resource
        EACCES => Some("EACCES"),         // denied by sandbox policy
        ENOSYS => Some("ENOSYS"),         // syscall not implemented by the Sentry
        EOPNOTSUPP => Some("EOPNOTSUPP"), // object does not support the op (sandbox)
        ENOTTY => Some("ENOTTY"),         // Sentry doesn't back the link-up ioctl
        _ => None,
    }
}

/// The runtime mode the node booted into, distinguishing a real privileged
/// kernel (metal) from a sandbox (gVisor's Sentry / an unprivileged container).
///
/// Mirrors Talos's `RuntimeMetal` vs `RuntimeContainer` split. It is detected
/// **once** at boot and logged as `[seq] runtime mode: metal|container` for
/// operator clarity. Crucially it is *informational, not load-bearing*: the
/// actual best-effort behavior is driven entirely by the errno tolerance in
/// [`privileged_op_skip_reason`] (a metal kernel that somehow denies an op is
/// still tolerated; a sandbox that somehow permits one still succeeds). This
/// keeps detection from ever weakening or strengthening real behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootRuntimeMode {
    /// A real, privileged kernel: every machine-bring-up syscall is expected to
    /// succeed.
    Metal,
    /// A sandbox (gVisor `runsc` Sentry, or an unprivileged container): the host
    /// already provides / forbids the privileged ops, so they are best-effort.
    Container,
}

impl BootRuntimeMode {
    /// Stable lowercase name (`"metal"` / `"container"`), matching the
    /// `[seq] runtime mode: <name>` log line.
    pub fn as_str(self) -> &'static str {
        match self {
            BootRuntimeMode::Metal => "metal",
            BootRuntimeMode::Container => "container",
        }
    }

    /// Whether this is a sandboxed runtime where privileged bring-up ops are
    /// expected to be best-effort.
    pub fn is_container(self) -> bool {
        matches!(self, BootRuntimeMode::Container)
    }
}

/// Detect the [`BootRuntimeMode`] from the kernel's `/proc/version` string
/// (`None` if `/proc/version` was unreadable).
///
/// gVisor's Sentry advertises itself in `/proc/version` (the real banner is
/// `Linux version 4.19.0-gvisor ...`, i.e. it contains the substring `gvisor`),
/// which is the canonical, cheap signal Talos-style detection keys on. The match
/// is ASCII-case-insensitive so both the real lowercase `-gvisor` release tag and
/// any `gVisor`-cased banner are recognized. Anything else — including an
/// unreadable `/proc/version`
/// (`None`) — is treated as [`BootRuntimeMode::Metal`], the safe full-fidelity
/// default: detection only ever *relaxes* logging, never behavior, so a
/// false "metal" is harmless (the errno tolerance still kicks in if a privileged
/// op is denied).
///
/// Pure and host-testable: it inspects only the supplied string.
pub fn detect_runtime_mode(proc_version: Option<&str>) -> BootRuntimeMode {
    match proc_version {
        Some(v) if v.to_ascii_lowercase().contains("gvisor") => BootRuntimeMode::Container,
        _ => BootRuntimeMode::Metal,
    }
}

/// How a phase treats a failing task.
///
/// Mirrors Talos's per-phase fail policy: most boot phases abort the whole
/// sequence on the first failure, but some "best effort" phases log and
/// continue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailPolicy {
    /// The first failing task aborts the phase and the whole boot.
    Abort,
    /// A failing task is logged; remaining tasks in the phase still run.
    Continue,
}

/// A single unit of boot work, run against a `&mut dyn Runtime`.
///
/// This is the trait-object form: a [`BootPhase`] owns `Box<dyn BootTask>`s.
/// Any `fn`/closure of the right shape becomes a task via [`FnTask`].
pub trait BootTask {
    /// Stable task name (for logs / error reporting).
    fn name(&self) -> &str;
    /// Execute the task against the runtime.
    fn run(&self, rt: &mut dyn Runtime) -> Result<TaskOutcome>;
}

/// Adapts a closure `Fn(&mut dyn Runtime) -> Result<TaskOutcome>` into a
/// [`BootTask`]. This is the "Task is a fn that takes a mutable runtime/state
/// and returns Result" form the sequencer executes.
pub struct FnTask {
    name: String,
    #[allow(clippy::type_complexity)]
    f: Box<dyn Fn(&mut dyn Runtime) -> Result<TaskOutcome>>,
}

impl FnTask {
    /// Build a named task from a closure.
    pub fn new(
        name: impl Into<String>,
        f: impl Fn(&mut dyn Runtime) -> Result<TaskOutcome> + 'static,
    ) -> Self {
        FnTask {
            name: name.into(),
            f: Box::new(f),
        }
    }

    /// Box this task as a trait object (convenience for phase construction).
    pub fn boxed(self) -> Box<dyn BootTask> {
        Box::new(self)
    }
}

impl BootTask for FnTask {
    fn name(&self) -> &str {
        &self.name
    }
    fn run(&self, rt: &mut dyn Runtime) -> Result<TaskOutcome> {
        (self.f)(rt)
    }
}

/// The identity of each modeled boot phase, in execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BootPhaseId {
    /// Mount the kernel pseudo-filesystems (proc/sys/dev/run/...). Runs first so
    /// that `/proc/sys/...` exists before KSPP sysctls are enforced.
    MountPseudoFs,
    /// Create the system directory tree and enforce KSPP sysctls (writes to
    /// `/proc/sys/...`, which requires the pseudo-filesystems above).
    SystemDirectories,
    /// Load and validate the machine configuration.
    LoadConfig,
    /// Set the system hostname.
    SetHostname,
    /// Bring the network up (loopback + links).
    Network,
    /// Spawn the node services.
    StartServices,
    /// Mark the machine fully running.
    Running,
}

impl BootPhaseId {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            BootPhaseId::SystemDirectories => "systemDirectories",
            BootPhaseId::MountPseudoFs => "mountPseudoFs",
            BootPhaseId::LoadConfig => "loadConfig",
            BootPhaseId::SetHostname => "setHostname",
            BootPhaseId::Network => "network",
            BootPhaseId::StartServices => "startServices",
            BootPhaseId::Running => "running",
        }
    }

    /// The canonical ordered list of boot phases.
    pub fn order() -> [BootPhaseId; 7] {
        [
            BootPhaseId::MountPseudoFs,
            BootPhaseId::SystemDirectories,
            BootPhaseId::LoadConfig,
            BootPhaseId::SetHostname,
            BootPhaseId::Network,
            BootPhaseId::StartServices,
            BootPhaseId::Running,
        ]
    }
}

/// An ordered, named group of boot tasks with a [`FailPolicy`].
pub struct BootPhase {
    id: BootPhaseId,
    tasks: Vec<Box<dyn BootTask>>,
    fail_policy: FailPolicy,
}

impl BootPhase {
    /// Create an empty phase (defaults to [`FailPolicy::Abort`]).
    pub fn new(id: BootPhaseId) -> Self {
        BootPhase {
            id,
            tasks: Vec::new(),
            fail_policy: FailPolicy::Abort,
        }
    }

    /// Set the phase's failure policy.
    pub fn with_fail_policy(mut self, policy: FailPolicy) -> Self {
        self.fail_policy = policy;
        self
    }

    /// Append a task (builder form).
    pub fn with_task(mut self, task: Box<dyn BootTask>) -> Self {
        self.tasks.push(task);
        self
    }

    /// Append a task in place.
    pub fn push(&mut self, task: Box<dyn BootTask>) {
        self.tasks.push(task);
    }

    /// The phase id.
    pub fn id(&self) -> BootPhaseId {
        self.id
    }

    /// The failure policy.
    pub fn fail_policy(&self) -> FailPolicy {
        self.fail_policy
    }

    /// Number of tasks.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Whether the phase has no tasks.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Run the phase's tasks in order against `rt`, reporting through `log`.
    ///
    /// Under [`FailPolicy::Abort`] the first failing task propagates its error.
    /// Under [`FailPolicy::Continue`] failures are reported to the logger and
    /// the phase keeps going; the phase still returns `Ok` but the failure count
    /// is reflected in the returned tally.
    fn run(&self, rt: &mut dyn Runtime, log: &mut dyn ProgressLogger) -> Result<PhaseTally> {
        let mut tally = PhaseTally::default();
        for task in &self.tasks {
            log.task_start(self.id, task.name());
            match task.run(rt) {
                Ok(outcome) => {
                    match outcome {
                        TaskOutcome::Done => tally.done += 1,
                        TaskOutcome::Skipped => tally.skipped += 1,
                    }
                    log.task_done(self.id, task.name(), outcome);
                }
                Err(e) => {
                    log.task_failed(self.id, task.name(), &e);
                    tally.failed += 1;
                    match self.fail_policy {
                        FailPolicy::Abort => return Err(e),
                        FailPolicy::Continue => continue,
                    }
                }
            }
        }
        Ok(tally)
    }
}

impl core::fmt::Debug for BootPhase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootPhase")
            .field("id", &self.id)
            .field("tasks", &self.tasks.len())
            .field("fail_policy", &self.fail_policy)
            .finish()
    }
}

/// Per-phase task tally (how many tasks did work / were skipped / failed).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhaseTally {
    /// Tasks that did work.
    pub done: usize,
    /// Tasks that were already satisfied.
    pub skipped: usize,
    /// Tasks that failed (only > 0 under [`FailPolicy::Continue`]).
    pub failed: usize,
}

/// A summary of a completed boot run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootReport {
    /// The per-phase tally, in execution order.
    pub phases: Vec<(BootPhaseId, PhaseTally)>,
}

impl BootReport {
    /// Number of phases that ran.
    pub fn phase_count(&self) -> usize {
        self.phases.len()
    }

    /// Total tasks that did work across all phases.
    pub fn total_done(&self) -> usize {
        self.phases.iter().map(|(_, t)| t.done).sum()
    }

    /// Total tasks that failed (best-effort phases) across all phases.
    pub fn total_failed(&self) -> usize {
        self.phases.iter().map(|(_, t)| t.failed).sum()
    }
}

// ---------------------------------------------------------------------------
// The sequencer.
// ---------------------------------------------------------------------------

/// The concrete, callable boot sequencer.
///
/// Holds the ordered [`BootPhase`] list and the list of [`BootService`]s the
/// `StartServices` phase spawns. The default ([`BootSequencer::new`]) installs
/// the standard boot phase list mirroring Talos; callers may build a custom one
/// with [`BootSequencer::with_phases`].
///
/// The entry point talos-init calls is [`BootSequencer::run_boot`].
pub struct BootSequencer {
    phases: Vec<BootPhase>,
}

impl Default for BootSequencer {
    fn default() -> Self {
        Self::new()
    }
}

impl BootSequencer {
    /// Build a sequencer with the standard boot phase list and the standard
    /// node service set (kubelet + containerd/etcd as appropriate at runtime).
    pub fn new() -> Self {
        BootSequencer {
            phases: standard_boot_phases(standard_services()),
        }
    }

    /// Build a sequencer with a custom phase list (used by tests).
    pub fn with_phases(phases: Vec<BootPhase>) -> Self {
        BootSequencer { phases }
    }

    /// Build a sequencer with the standard phases but a custom service list for
    /// the `StartServices` phase.
    pub fn with_services(services: Vec<BootService>) -> Self {
        BootSequencer {
            phases: standard_boot_phases(services),
        }
    }

    /// The ordered phase ids this sequencer will run.
    pub fn phase_ids(&self) -> Vec<BootPhaseId> {
        self.phases.iter().map(BootPhase::id).collect()
    }

    /// Number of phases.
    pub fn len(&self) -> usize {
        self.phases.len()
    }

    /// Whether there are no phases.
    pub fn is_empty(&self) -> bool {
        self.phases.is_empty()
    }

    /// **The entry point a PID1 process calls.** Run every boot phase in order
    /// against `rt`, reporting progress through `log`.
    ///
    /// On the first aborting task failure the boot stops and the error is
    /// returned (talos-init then triggers emergency poweroff). On success a
    /// [`BootReport`] summarizes what ran.
    pub fn run_boot(
        &self,
        rt: &mut dyn Runtime,
        log: &mut dyn ProgressLogger,
    ) -> Result<BootReport> {
        let total = self.phases.len();
        let mut phases = Vec::with_capacity(total);
        for (i, phase) in self.phases.iter().enumerate() {
            log.phase_start(i, total, phase.id());
            let tally = phase.run(rt, log)?;
            log.phase_done(phase.id());
            phases.push((phase.id(), tally));
        }
        log.boot_done();
        Ok(BootReport { phases })
    }
}

// ---------------------------------------------------------------------------
// The standard task catalog (mirrors Talos's Boot task list, simplified).
// ---------------------------------------------------------------------------

/// The KSPP sysctls the `SystemDirectories` phase enforces (key, value).
fn kspp_sysctls() -> &'static [(&'static str, &'static str)] {
    &[
        ("kernel.kptr_restrict", "1"),
        ("kernel.dmesg_restrict", "1"),
        ("kernel.kexec_load_disabled", "1"),
        ("kernel.unprivileged_bpf_disabled", "1"),
    ]
}

/// The system directories the `SystemDirectories` phase creates.
fn system_directories() -> &'static [&'static str] {
    &[
        "/system",
        "/system/state",
        "/system/run",
        "/var",
        "/run",
        "/tmp",
    ]
}

/// The pseudo-filesystems the `MountPseudoFs` phase mounts, in order.
fn pseudo_mounts() -> Vec<MountRequest> {
    vec![
        MountRequest::new("proc", "/proc", "proc", "nosuid,nodev,noexec"),
        MountRequest::new("sysfs", "/sys", "sysfs", "nosuid,nodev,noexec"),
        MountRequest::new("devtmpfs", "/dev", "devtmpfs", "nosuid"),
        MountRequest::new("tmpfs", "/run", "tmpfs", "nosuid,nodev"),
        MountRequest::new("devpts", "/dev/pts", "devpts", "nosuid,noexec"),
        MountRequest::new("tmpfs", "/dev/shm", "tmpfs", "nosuid,nodev"),
    ]
}

/// The standard node services spawned by the `StartServices` phase.
pub fn standard_services() -> Vec<BootService> {
    vec![
        BootService::new("containerd", ["/bin/containerd"], RestartPolicy::Always),
        BootService::new("udevd", ["/sbin/udevd"], RestartPolicy::Always),
        BootService::new("kubelet", ["/usr/bin/kubelet"], RestartPolicy::Always),
    ]
}

/// Build the standard ordered boot phase list, wiring each phase's tasks to the
/// [`Runtime`]. The `StartServices` phase spawns `services`.
pub fn standard_boot_phases(services: Vec<BootService>) -> Vec<BootPhase> {
    vec![
        // 1. Pseudo filesystems. Mounted FIRST so `/proc` (and `/sys`) exist
        //    before the KSPP sysctls in the next phase write `/proc/sys/...`.
        //    Mirrors upstream Talos, which mounts pseudo-fs in PID 1 before
        //    machined enforces KSPP.
        BootPhase::new(BootPhaseId::MountPseudoFs).with_task(
            FnTask::new("mountPseudoFilesystems", |rt: &mut dyn Runtime| {
                for m in pseudo_mounts() {
                    rt.mount(&m)?;
                }
                Ok(TaskOutcome::Done)
            })
            .boxed(),
        ),
        // 2. System directories + KSPP. Runs AFTER MountPseudoFs so the sysctl
        //    writes in enforceKSPP land on a real, mounted `/proc/sys`.
        BootPhase::new(BootPhaseId::SystemDirectories)
            .with_task(
                FnTask::new("setupSystemDirectory", |rt: &mut dyn Runtime| {
                    for dir in system_directories() {
                        rt.make_directory(dir)?;
                    }
                    Ok(TaskOutcome::Done)
                })
                .boxed(),
            )
            .with_task(
                FnTask::new("enforceKSPP", |rt: &mut dyn Runtime| {
                    // KSPP sysctl hardening is best effort: each knob is
                    // attempted; benign failures (sandbox / unprivileged /
                    // read-only / absent) are skipped-and-logged, and only a
                    // genuinely unexpected error aborts the boot. We keep an
                    // explicit applied-vs-skipped tally in the log.
                    //
                    // Because MountPseudoFs already ran, `/proc/sys/...` exists
                    // on a real kernel and these writes actually apply (rather
                    // than failing ENOENT and being silently skipped).
                    let mut applied: Vec<&str> = Vec::new();
                    let mut skipped: Vec<&str> = Vec::new();
                    for (seq, (k, v)) in kspp_sysctls().iter().enumerate() {
                        match rt.write_sysctl(k, v)? {
                            SysctlOutcome::Applied => {
                                rt.log(&format!("[{seq}] sysctl {k}: applied"));
                                applied.push(k);
                            }
                            SysctlOutcome::Skipped(errno) => {
                                rt.log(&format!("[{seq}] sysctl {k}: skipped ({errno})"));
                                skipped.push(k);
                            }
                        }
                    }
                    rt.log(&format!(
                        "[seq] enforceKSPP: applied {applied:?}, skipped {skipped:?}"
                    ));
                    Ok(TaskOutcome::Done)
                })
                .boxed(),
            ),
        // 3. Load + validate config. Records the hostname for the next phase by
        //    re-deriving it there from apply_config (kept simple/stateless here).
        BootPhase::new(BootPhaseId::LoadConfig)
            .with_task(
                FnTask::new("bootstrapNetworkForConfig", |rt: &mut dyn Runtime| {
                    if rt.bootstrap_network_for_config()? {
                        Ok(TaskOutcome::Done)
                    } else {
                        Ok(TaskOutcome::Skipped)
                    }
                })
                .boxed(),
            )
            .with_task(
                FnTask::new("loadAndValidateConfig", |rt: &mut dyn Runtime| {
                    let raw = rt.load_config()?;
                    if raw.trim().is_empty() {
                        return Err(MachinedError::task_failed(
                            "loadAndValidateConfig",
                            "machine config is empty",
                        ));
                    }
                    rt.apply_config(&raw)?;
                    Ok(TaskOutcome::Done)
                })
                .boxed(),
            ),
        // 4. Hostname.
        BootPhase::new(BootPhaseId::SetHostname).with_task(
            FnTask::new("setHostname", |rt: &mut dyn Runtime| {
                let raw = rt.load_config()?;
                let hostname = rt.apply_config(&raw)?;
                match hostname {
                    Some(h) if !h.is_empty() => {
                        rt.set_hostname(&h)?;
                        Ok(TaskOutcome::Done)
                    }
                    // No hostname in config: leave the kernel default in place.
                    _ => Ok(TaskOutcome::Skipped),
                }
            })
            .boxed(),
        ),
        // 5. Network: loopback first, then any configured links.
        BootPhase::new(BootPhaseId::Network).with_task(
            FnTask::new("bringNetworkUp", |rt: &mut dyn Runtime| {
                rt.link_up("lo")?;
                rt.link_up("eth0")?;
                rt.boot_cosi_network_bridge()?;
                rt.run_image_cache_runtime_adapters()?;
                Ok(TaskOutcome::Done)
            })
            .boxed(),
        ),
        // 6. Start services.
        BootPhase::new(BootPhaseId::StartServices).with_task(start_services_task(services).boxed()),
        // 7. Running.
        BootPhase::new(BootPhaseId::Running).with_task(
            FnTask::new("markRunning", |rt: &mut dyn Runtime| {
                rt.log("machine is running");
                Ok(TaskOutcome::Done)
            })
            .boxed(),
        ),
    ]
}

/// Build the `StartServices` task: drive each [`BootService`] through
/// `Preparing -> Running`, asking the runtime to spawn it and recording the pid.
fn start_services_task(services: Vec<BootService>) -> FnTask {
    FnTask::new("startServices", move |rt: &mut dyn Runtime| {
        // Work on a local clone so the task is `Fn` (re-runnable); the runtime
        // records the spawned pids, which is the source of truth the caller
        // inspects.
        let mut local = services.clone();
        for svc in &mut local {
            svc.transition_to(ServiceStatus::Preparing)?;
            rt.log(&service_start_preparing_line(svc));
            match rt.spawn_service(svc) {
                Ok(pid) => {
                    svc.mark_running(pid)?;
                    rt.log(&service_start_running_line(svc, pid));
                }
                Err(e) => {
                    svc.transition_to(ServiceStatus::Failed)?;
                    rt.log(&service_start_failed_line(svc));
                    return Err(e);
                }
            }
        }
        Ok(TaskOutcome::Done)
    })
}

fn service_start_preparing_line(service: &BootService) -> String {
    format!(
        "service-start: name={} status={} command={} restart={}",
        service.name(),
        service.status().as_str(),
        service.command().join(" "),
        service.policy().as_str()
    )
}

fn service_start_running_line(service: &BootService, pid: u32) -> String {
    format!(
        "service-start: name={} status={} pid={pid}",
        service.name(),
        service.status().as_str()
    )
}

fn service_start_failed_line(service: &BootService) -> String {
    format!(
        "service-start: name={} status={}",
        service.name(),
        service.status().as_str()
    )
}

// ---------------------------------------------------------------------------
// In-memory fake Runtime for tests.
// ---------------------------------------------------------------------------

/// An in-memory [`Runtime`] that records every operation instead of touching
/// the OS, and lets tests inject canned config and failures.
///
/// This is the test double for the real `libc`-backed runtime talos-init ships.
#[derive(Debug)]
pub struct FakeRuntime {
    platform: String,
    has_disks: bool,
    config: String,
    config_hostname: Option<String>,
    next_pid: u32,
    /// Operations that should fail, keyed by a short op tag (e.g. `"mount:/proc"`,
    /// `"spawn:kubelet"`, `"sethostname"`).
    fail: BTreeMap<String, String>,

    /// Recorded directories created.
    pub directories: Vec<String>,
    /// Recorded sysctls written, in order.
    pub sysctls: Vec<(String, String)>,
    /// Recorded sysctls skipped (best effort), as `(key, errno_name)`, in order.
    pub skipped_sysctls: Vec<(String, &'static str)>,
    /// Recorded mounts, in order.
    pub mounts: Vec<MountRequest>,
    /// Recorded pre-config network bootstrap interfaces.
    pub pre_config_network: Vec<String>,
    /// Number of machine-config loads.
    pub config_loads: usize,
    /// Hostname set, if any.
    pub hostname: Option<String>,
    /// Links brought up, in order.
    pub links_up: Vec<String>,
    /// Network-phase operations, in order, including the COSI bridge boundary.
    pub network_ops: Vec<String>,
    /// Privileged bring-up ops skipped best-effort under a simulated sandbox,
    /// as `(op, errno_name)`, in order. Mirrors `skipped_sysctls` for the
    /// `set_hostname` / `link_up` / `add_ipv4` / module-load family.
    pub skipped_privileged: Vec<(String, &'static str)>,
    /// Services spawned: (name, assigned pid), in order.
    pub spawned: Vec<(String, u32)>,
    /// Free-form log lines emitted via [`Runtime::log`].
    pub logs: Vec<String>,
    /// Optional host-safe image-cache runtime plan to exercise boot-owned
    /// adapters after the COSI bridge. Absent means no image-cache runtime work.
    image_cache_runtime_plan: Option<ImageCacheRuntimePlan>,
    /// Registryd service ids launched by the image-cache runtime adapter.
    pub image_cache_registryd_launches: Vec<String>,
}

impl FakeRuntime {
    /// Build a fake runtime for a platform with a canned machine config and an
    /// optional hostname the config "declares".
    pub fn new(platform: impl Into<String>) -> Self {
        FakeRuntime {
            platform: platform.into(),
            has_disks: true,
            config: "version: v1alpha1\nmachine:\n  type: worker\n".to_string(),
            config_hostname: Some("talos-node".to_string()),
            next_pid: 1000,
            fail: BTreeMap::new(),
            directories: Vec::new(),
            sysctls: Vec::new(),
            skipped_sysctls: Vec::new(),
            mounts: Vec::new(),
            pre_config_network: Vec::new(),
            config_loads: 0,
            hostname: None,
            links_up: Vec::new(),
            network_ops: Vec::new(),
            skipped_privileged: Vec::new(),
            spawned: Vec::new(),
            logs: Vec::new(),
            image_cache_runtime_plan: None,
            image_cache_registryd_launches: Vec::new(),
        }
    }

    /// Set the canned config the runtime "loads".
    pub fn with_config(mut self, raw: impl Into<String>, hostname: Option<&str>) -> Self {
        self.config = raw.into();
        self.config_hostname = hostname.map(std::string::ToString::to_string);
        self
    }

    /// Mark the runtime as diskless (e.g. a container).
    pub fn diskless(mut self) -> Self {
        self.has_disks = false;
        self
    }

    /// Inject a failure for a given op tag.
    pub fn fail_op(mut self, tag: impl Into<String>, reason: impl Into<String>) -> Self {
        self.fail.insert(tag.into(), reason.into());
        self
    }

    /// Attach a host-safe image-cache runtime plan that the fake boot runtime
    /// will execute through the same registryd adapter/supervisor boundary as
    /// live machined-owned service effects.
    pub fn with_image_cache_runtime_plan(mut self, plan: ImageCacheRuntimePlan) -> Self {
        self.image_cache_runtime_plan = Some(plan);
        self
    }

    fn check(&self, tag: &str) -> Result<()> {
        if let Some(reason) = self.fail.get(tag) {
            return Err(MachinedError::task_failed(tag, reason.clone()));
        }
        Ok(())
    }

    /// If a `skip_priv:<op>` tag was injected (carrying a sandbox errno), return
    /// its classified errno name, modeling the runtime tolerating the privileged
    /// bring-up op best-effort. The tag must carry one of the
    /// [`privileged_op_skip_reason`] errnos.
    fn skip_priv(&self, op: &str) -> Option<&'static str> {
        let errno_str = self.fail.get(&format!("skip_priv:{op}"))?;
        let errno: i32 = errno_str.parse().unwrap_or(EPERM);
        Some(privileged_op_skip_reason(errno).expect("skip_priv tag must use a skippable errno"))
    }
}

struct RecordingRegistrydLauncher<'a> {
    launches: &'a mut Vec<String>,
}

impl ServiceLauncher for RecordingRegistrydLauncher<'_> {
    fn launch(&mut self, id: &str) -> Result<bool> {
        self.launches.push(id.to_string());
        Ok(true)
    }
}

fn registryd_execution_status_label(status: RegistrydServiceExecutionStatus) -> &'static str {
    match status {
        RegistrydServiceExecutionStatus::NoAction => "no-action",
        RegistrydServiceExecutionStatus::AlreadyRunning => "already-running",
        RegistrydServiceExecutionStatus::Started => "started",
        RegistrydServiceExecutionStatus::LoadedAndStarted => "loaded-and-started",
    }
}

fn registryd_action_label(action: RegistrydAction) -> &'static str {
    match action {
        RegistrydAction::None => "none",
        RegistrydAction::Start => "start",
    }
}

impl Runtime for FakeRuntime {
    fn platform(&self) -> &str {
        &self.platform
    }
    fn has_disks(&self) -> bool {
        self.has_disks
    }
    fn make_directory(&mut self, path: &str) -> Result<()> {
        self.check(&format!("mkdir:{path}"))?;
        self.directories.push(path.to_string());
        Ok(())
    }
    fn write_sysctl(&mut self, key: &str, value: &str) -> Result<SysctlOutcome> {
        // A `skip_sysctl:<key>` tag with an errno number simulates a benign
        // best-effort skip (e.g. gVisor making the knob read-only); a plain
        // `sysctl:<key>` fail tag simulates a genuinely unexpected error that
        // must abort the boot.
        if let Some(errno_str) = self.fail.get(&format!("skip_sysctl:{key}")) {
            let errno: i32 = errno_str.parse().unwrap_or(EPERM);
            let reason =
                sysctl_skip_reason(errno).expect("skip_sysctl tag must use a skippable errno");
            self.skipped_sysctls.push((key.to_string(), reason));
            return Ok(SysctlOutcome::Skipped(reason));
        }
        self.check(&format!("sysctl:{key}"))?;
        self.sysctls.push((key.to_string(), value.to_string()));
        Ok(SysctlOutcome::Applied)
    }
    fn mount(&mut self, req: &MountRequest) -> Result<()> {
        self.check(&format!("mount:{}", req.target))?;
        self.mounts.push(req.clone());
        Ok(())
    }
    fn load_config(&mut self) -> Result<String> {
        self.check("load_config")?;
        self.config_loads += 1;
        Ok(self.config.clone())
    }
    fn bootstrap_network_for_config(&mut self) -> Result<bool> {
        self.check("bootstrap_network_for_config")?;
        if self.platform != "aws" {
            return Ok(false);
        }

        self.pre_config_network
            .push("eth0:dhcp4,dhcp6,metric=1024".to_string());
        self.links_up.push("eth0".to_string());
        self.logs
            .push("pre-config aws network bootstrap: eth0 dhcp4+dhcp6".to_string());
        Ok(true)
    }
    fn apply_config(&mut self, _raw: &str) -> Result<Option<String>> {
        self.check("apply_config")?;
        Ok(self.config_hostname.clone())
    }
    fn set_hostname(&mut self, hostname: &str) -> Result<()> {
        // A `skip_priv:set_hostname` tag carrying an errno simulates a benign
        // best-effort skip under a sandbox (the host owns the UTS namespace);
        // the runtime logs and continues. A plain `sethostname` fail tag
        // simulates a genuinely unexpected error that aborts the boot.
        if let Some(reason) = self.skip_priv("set_hostname") {
            self.skipped_privileged
                .push(("set_hostname".to_string(), reason));
            return Ok(());
        }
        self.check("sethostname")?;
        self.hostname = Some(hostname.to_string());
        Ok(())
    }
    fn link_up(&mut self, iface: &str) -> Result<()> {
        // `skip_priv:link_up:<iface>` simulates a sandbox best-effort skip;
        // a plain `link_up:<iface>` fail tag is a genuinely unexpected abort.
        if let Some(reason) = self.skip_priv(&format!("link_up:{iface}")) {
            self.skipped_privileged
                .push((format!("link_up:{iface}"), reason));
            return Ok(());
        }
        self.check(&format!("link_up:{iface}"))?;
        self.links_up.push(iface.to_string());
        self.network_ops.push(format!("link_up:{iface}"));
        Ok(())
    }
    fn boot_cosi_network_bridge(&mut self) -> Result<()> {
        self.check("boot_cosi_network_bridge")?;
        self.network_ops
            .push("boot_cosi_network_bridge".to_string());
        self.logs
            .push("boot-cosi-network-bridge: stable ticks=1".to_string());
        Ok(())
    }
    fn run_image_cache_runtime_adapters(&mut self) -> Result<()> {
        self.check("image_cache_runtime_adapters")?;
        let Some(plan) = self.image_cache_runtime_plan.clone() else {
            return Ok(());
        };

        // Registryd has no source conditions/dependencies, so the worker role
        // here is only a deterministic fake-supervisor host for the same
        // service-manager adapter boundary used by machined.
        let mut supervisor = Supervisor::new(MachineType::Worker);
        let mut launcher = RecordingRegistrydLauncher {
            launches: &mut self.image_cache_registryd_launches,
        };
        let (report, registryd_state) = {
            let mut manager =
                SupervisorRegistrydServiceManager::new(&mut supervisor, &mut launcher);
            let report = RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .map_err(|err| {
                    MachinedError::task_failed("imageCacheRuntimeAdapters", err.to_string())
                })?;
            (report, manager.registryd_state())
        };
        let status = registryd_execution_status_label(report.status);
        let observed_plan = plan.reproject_after_registryd_observation(registryd_state);
        self.image_cache_runtime_plan = Some(observed_plan.clone());
        self.network_ops
            .push(format!("image_cache_runtime:registryd:{status}"));
        self.logs.push(format!(
            "image-cache-runtime: registryd status={status} loaded={} started={} running={} healthy={} observedStatus={} observedAction={}",
            report.loaded,
            report.started,
            registryd_state.running,
            registryd_state.healthy,
            observed_plan.config.status.as_str(),
            registryd_action_label(observed_plan.registryd_action),
        ));
        Ok(())
    }
    fn spawn_service(&mut self, service: &BootService) -> Result<u32> {
        self.check(&format!("spawn:{}", service.name()))?;
        let pid = self.next_pid;
        self.next_pid += 1;
        self.spawned.push((service.name().to_string(), pid));
        Ok(pid)
    }
    fn log(&mut self, line: &str) {
        self.logs.push(line.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_runtime_cri_domain::{
        ImageCacheConfig, ImageCacheCopyStatus, ImageCacheRuntimePlan, ImageCacheStatus,
        REGISTRYD_SERVICE_ID, RegistrydAction,
    };

    fn log_position(lines: &[String], needle: &str) -> usize {
        lines
            .iter()
            .position(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("missing log line containing {needle:?}; logs={lines:?}"))
    }

    fn registryd_start_plan() -> ImageCacheRuntimePlan {
        ImageCacheRuntimePlan {
            config: ImageCacheConfig {
                status: ImageCacheStatus::Preparing,
                copy_status: ImageCacheCopyStatus::Skipped,
                roots: vec!["/system/imagecache/disk".to_string()],
            },
            registryd_action: RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        }
    }

    // ----- service state machine -----

    #[test]
    fn service_state_machine_happy_path() {
        let mut s = BootService::new("etcd", ["/usr/bin/etcd"], RestartPolicy::Always);
        assert_eq!(s.status(), ServiceStatus::Initialized);
        s.transition_to(ServiceStatus::Preparing).unwrap();
        s.mark_running(4242).unwrap();
        assert_eq!(s.status(), ServiceStatus::Running);
        assert_eq!(s.pid(), Some(4242));
        s.transition_to(ServiceStatus::Finished).unwrap();
        assert!(s.status().is_terminal());
        assert_eq!(s.pid(), None);
    }

    #[test]
    fn service_illegal_transition_rejected() {
        let mut s = BootService::new("kubelet", ["/k"], RestartPolicy::Always);
        // Initialized -> Running is illegal (must go through Preparing).
        let err = s.mark_running(1).unwrap_err();
        assert_eq!(err.kind(), "illegal_transition");
    }

    #[test]
    fn service_restart_counts_and_budget() {
        let mut s = BootService::new("kubelet", ["/k"], RestartPolicy::Always).with_max_restarts(2);
        s.transition_to(ServiceStatus::Preparing).unwrap();
        s.mark_running(1).unwrap();
        s.transition_to(ServiceStatus::Failed).unwrap();
        assert!(s.should_restart(true));
        s.transition_to(ServiceStatus::Preparing).unwrap();
        assert_eq!(s.restarts(), 1);
        s.mark_running(2).unwrap();
        s.transition_to(ServiceStatus::Failed).unwrap();
        s.transition_to(ServiceStatus::Preparing).unwrap();
        assert_eq!(s.restarts(), 2);
        s.mark_running(3).unwrap();
        s.transition_to(ServiceStatus::Failed).unwrap();
        // Budget exhausted.
        assert!(!s.should_restart(true));
    }

    #[test]
    fn restart_policy_semantics() {
        let never = BootService::new("oneshot", ["/x"], RestartPolicy::Never);
        let onfail = BootService::new("svc", ["/x"], RestartPolicy::OnFailure);
        let always = BootService::new("svc", ["/x"], RestartPolicy::Always);
        // After a failure:
        assert!(!never.should_restart(true));
        assert!(onfail.should_restart(true));
        assert!(always.should_restart(true));
        // After a clean exit:
        assert!(!never.should_restart(false));
        assert!(!onfail.should_restart(false));
        assert!(always.should_restart(false));
    }

    // ----- phase ordering -----

    #[test]
    fn standard_phase_order_matches_spec() {
        let seq = BootSequencer::new();
        assert_eq!(seq.phase_ids(), BootPhaseId::order().to_vec());
        assert_eq!(seq.len(), 7);
    }

    #[test]
    fn containerd_kubelet_standard_services_start_with_udevd_between() {
        let services = standard_services();
        let names: Vec<&str> = services.iter().map(BootService::name).collect();
        assert_eq!(names, vec!["containerd", "udevd", "kubelet"]);

        let containerd = &services[0];
        assert_eq!(containerd.command(), &["/bin/containerd".to_string()]);
        assert_eq!(containerd.policy(), RestartPolicy::Always);

        let kubelet = &services[2];
        assert_eq!(kubelet.command(), &["/usr/bin/kubelet".to_string()]);
        assert_eq!(kubelet.policy(), RestartPolicy::Always);
    }

    // ----- full boot drives the fake runtime through all phases -----

    #[test]
    fn run_boot_drives_all_phases() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal");
        let mut log = RecordingLogger::default();
        let report = seq.run_boot(&mut rt, &mut log).unwrap();

        // Every phase ran, in order.
        assert_eq!(report.phase_count(), 7);
        let ran: Vec<BootPhaseId> = report.phases.iter().map(|(id, _)| *id).collect();
        assert_eq!(ran, BootPhaseId::order().to_vec());

        // Real ops happened on the runtime.
        assert!(rt.directories.contains(&"/system".to_string()));
        assert_eq!(rt.sysctls.len(), kspp_sysctls().len());
        assert_eq!(rt.mounts.len(), pseudo_mounts().len());
        assert_eq!(rt.mounts[0].target, "/proc");
        assert_eq!(rt.hostname.as_deref(), Some("talos-node"));
        assert_eq!(rt.links_up, vec!["lo".to_string(), "eth0".to_string()]);
        assert_eq!(
            rt.network_ops,
            vec![
                "link_up:lo".to_string(),
                "link_up:eth0".to_string(),
                "boot_cosi_network_bridge".to_string()
            ]
        );
        assert_eq!(rt.spawned.len(), 3);
        assert_eq!(rt.spawned[0].0, "containerd");
        // Pids are assigned and recorded.
        assert!(rt.spawned.iter().all(|(_, pid)| *pid >= 1000));

        // Logger captured the transcript with the boot-complete line last.
        assert_eq!(log.lines.last().map(String::as_str), Some("boot complete"));
        assert!(log.lines.iter().any(|l| l.contains("systemDirectories")));
    }

    #[test]
    fn loopback_comes_up_before_other_links() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;
        seq.run_boot(&mut rt, &mut log).unwrap();
        let lo = rt.links_up.iter().position(|l| l == "lo").unwrap();
        let eth = rt.links_up.iter().position(|l| l == "eth0").unwrap();
        assert!(lo < eth);
    }

    #[test]
    fn boot_cosi_network_bridge_runs_after_primary_links_before_services() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal");
        let mut log = RecordingLogger::default();
        seq.run_boot(&mut rt, &mut log).unwrap();

        assert_eq!(
            rt.network_ops,
            vec![
                "link_up:lo".to_string(),
                "link_up:eth0".to_string(),
                "boot_cosi_network_bridge".to_string()
            ]
        );
        assert_eq!(rt.spawned[0].0, "containerd");

        let bridge_log = rt
            .logs
            .iter()
            .position(|line| line.contains("boot-cosi-network-bridge"))
            .unwrap();
        let running_log = rt
            .logs
            .iter()
            .position(|line| line == "machine is running")
            .unwrap();
        assert!(bridge_log < running_log);
    }

    #[test]
    fn image_cache_registryd_runtime_adapter_runs_after_cosi_bridge_before_services() {
        let seq = BootSequencer::with_services(vec![BootService::new(
            "svc",
            ["/usr/bin/svc"],
            RestartPolicy::Never,
        )]);
        let mut rt =
            FakeRuntime::new("metal").with_image_cache_runtime_plan(registryd_start_plan());
        let mut log = NullLogger;

        seq.run_boot(&mut rt, &mut log).unwrap();

        assert_eq!(
            rt.network_ops,
            vec![
                "link_up:lo".to_string(),
                "link_up:eth0".to_string(),
                "boot_cosi_network_bridge".to_string(),
                "image_cache_runtime:registryd:loaded-and-started".to_string(),
            ]
        );
        assert_eq!(
            rt.image_cache_registryd_launches,
            vec![REGISTRYD_SERVICE_ID.to_string()]
        );
        assert_eq!(
            rt.image_cache_runtime_plan.as_ref().unwrap().config.status,
            ImageCacheStatus::Ready
        );
        assert_eq!(rt.spawned, vec![("svc".to_string(), 1000)]);

        let bridge = log_position(&rt.logs, "boot-cosi-network-bridge: stable");
        let registryd = log_position(
            &rt.logs,
            "image-cache-runtime: registryd status=loaded-and-started loaded=true started=true running=true healthy=true observedStatus=ready observedAction=none",
        );
        let service = log_position(&rt.logs, "service-start: name=svc status=preparing");
        assert!(
            bridge < registryd && registryd < service,
            "registryd adapter must run after COSI bridge and before service start: {:?}",
            rt.logs
        );
    }

    #[test]
    fn image_cache_registryd_runtime_hook_is_inert_without_runtime_plan() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;

        seq.run_boot(&mut rt, &mut log).unwrap();

        assert_eq!(
            rt.network_ops,
            vec![
                "link_up:lo".to_string(),
                "link_up:eth0".to_string(),
                "boot_cosi_network_bridge".to_string(),
            ]
        );
        assert!(rt.image_cache_registryd_launches.is_empty());
        assert!(
            !rt.logs
                .iter()
                .any(|line| line.contains("image-cache-runtime: registryd"))
        );
    }

    #[test]
    fn containerd_kubelet_service_start_markers_follow_network_bridge_and_spawn_order() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;
        seq.run_boot(&mut rt, &mut log).unwrap();

        let spawned: Vec<&str> = rt.spawned.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(spawned, vec!["containerd", "udevd", "kubelet"]);

        let bridge = log_position(&rt.logs, "boot-cosi-network-bridge: stable");
        let containerd_preparing = log_position(
            &rt.logs,
            "service-start: name=containerd status=preparing command=/bin/containerd restart=always",
        );
        let containerd_running = log_position(
            &rt.logs,
            "service-start: name=containerd status=running pid=1000",
        );
        let udevd_preparing = log_position(
            &rt.logs,
            "service-start: name=udevd status=preparing command=/sbin/udevd restart=always",
        );
        let kubelet_preparing = log_position(
            &rt.logs,
            "service-start: name=kubelet status=preparing command=/usr/bin/kubelet restart=always",
        );
        let kubelet_running = log_position(
            &rt.logs,
            "service-start: name=kubelet status=running pid=1002",
        );

        assert!(
            bridge < containerd_preparing,
            "boot-owned COSI network bridge must publish before services start"
        );
        assert!(
            containerd_preparing < containerd_running
                && containerd_running < udevd_preparing
                && udevd_preparing < kubelet_preparing
                && kubelet_preparing < kubelet_running,
            "service markers must preserve sequential boot order: {:?}",
            rt.logs
        );
    }

    #[test]
    fn pseudo_fs_mounted_before_directories_and_kspp() {
        // Phase ordering guarantee (the KSPP bug fix): the kernel
        // pseudo-filesystems are mounted BEFORE the systemDirectories phase
        // (which runs enforceKSPP). This is what makes `/proc/sys/...` exist by
        // the time the sysctls are written, so KSPP hardening actually applies
        // on a real kernel. We assert via the recording logger transcript order.
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal");
        let mut log = RecordingLogger::default();
        seq.run_boot(&mut rt, &mut log).unwrap();
        let mounts = log
            .lines
            .iter()
            .position(|l| l.contains("mountPseudoFs"))
            .unwrap();
        let dirs = log
            .lines
            .iter()
            .position(|l| l.contains("systemDirectories"))
            .unwrap();
        assert!(
            mounts < dirs,
            "pseudo-fs must mount before systemDirectories/KSPP"
        );
    }

    #[test]
    fn aws_bootstraps_network_before_loading_config() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("aws");
        let mut log = RecordingLogger::default();
        seq.run_boot(&mut rt, &mut log).unwrap();

        assert_eq!(
            rt.pre_config_network,
            vec!["eth0:dhcp4,dhcp6,metric=1024".to_string()]
        );
        assert!(rt.config_loads > 0);
        assert_eq!(rt.links_up.first().map(String::as_str), Some("eth0"));

        let bootstrap = log
            .lines
            .iter()
            .position(|line| line.contains("bootstrapNetworkForConfig"))
            .unwrap();
        let load = log
            .lines
            .iter()
            .position(|line| line.contains("loadAndValidateConfig"))
            .unwrap();
        assert!(
            bootstrap < load,
            "AWS network bootstrap must precede config load"
        );
    }

    // ----- task failure aborts the boot -----

    #[test]
    fn mount_failure_aborts_boot() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal").fail_op("mount:/sys", "EBUSY");
        let mut log = RecordingLogger::default();
        let err = seq.run_boot(&mut rt, &mut log).unwrap_err();
        assert_eq!(err.kind(), "task_failed");
        // /proc mounted before the failure; /run never reached.
        assert!(rt.mounts.iter().any(|m| m.target == "/proc"));
        assert!(!rt.mounts.iter().any(|m| m.target == "/run"));
        // No service was spawned because boot aborted in an earlier phase.
        assert!(rt.spawned.is_empty());
        // The failure was logged.
        assert!(log.lines.iter().any(|l| l.contains("FAILED")));
    }

    #[test]
    fn empty_config_aborts_boot() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal").with_config("", None);
        let mut log = NullLogger;
        let err = seq.run_boot(&mut rt, &mut log).unwrap_err();
        assert_eq!(err.kind(), "task_failed");
        // Hostname was never set; no services spawned.
        assert!(rt.hostname.is_none());
        assert!(rt.spawned.is_empty());
    }

    #[test]
    fn spawn_failure_aborts_boot() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal").fail_op("spawn:udevd", "exec failed");
        let mut log = NullLogger;
        let err = seq.run_boot(&mut rt, &mut log).unwrap_err();
        assert_eq!(err.kind(), "task_failed");
        // containerd spawned before udevd failed; kubelet never reached.
        assert_eq!(rt.spawned.len(), 1);
        assert_eq!(rt.spawned[0].0, "containerd");
    }

    #[test]
    fn containerd_kubelet_containerd_spawn_failure_prevents_kubelet_spawn_and_logs_failed_marker() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal").fail_op("spawn:containerd", "exec failed");
        let mut log = NullLogger;
        let err = seq.run_boot(&mut rt, &mut log).unwrap_err();
        assert_eq!(err.kind(), "task_failed");

        assert!(rt.spawned.is_empty());
        assert!(rt.logs.iter().any(|line| {
            line == "service-start: name=containerd status=preparing command=/bin/containerd restart=always"
        }));
        assert!(
            rt.logs
                .iter()
                .any(|line| line == "service-start: name=containerd status=failed")
        );
        assert!(
            !rt.logs
                .iter()
                .any(|line| line.contains("service-start: name=kubelet"))
        );
    }

    #[test]
    fn containerd_kubelet_marker_absent_after_udevd_spawn_failure() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal").fail_op("spawn:udevd", "exec failed");
        let mut log = NullLogger;
        let err = seq.run_boot(&mut rt, &mut log).unwrap_err();
        assert_eq!(err.kind(), "task_failed");

        assert_eq!(rt.spawned, vec![("containerd".to_string(), 1000)]);
        assert!(
            rt.logs
                .iter()
                .any(|line| line == "service-start: name=udevd status=failed")
        );
        assert!(
            !rt.logs
                .iter()
                .any(|line| line.contains("service-start: name=kubelet"))
        );
    }

    // ----- fail policy: continue -----

    #[test]
    fn continue_policy_runs_remaining_tasks_after_failure() {
        let phase = BootPhase::new(BootPhaseId::Network)
            .with_fail_policy(FailPolicy::Continue)
            .with_task(
                FnTask::new("boom", |_| Err(MachinedError::task_failed("boom", "nope"))).boxed(),
            )
            .with_task(
                FnTask::new("after", |rt: &mut dyn Runtime| {
                    rt.link_up("lo")?;
                    Ok(TaskOutcome::Done)
                })
                .boxed(),
            );
        let seq = BootSequencer::with_phases(vec![phase]);
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;
        let report = seq.run_boot(&mut rt, &mut log).unwrap();
        // The phase did not abort: the later task ran.
        assert_eq!(rt.links_up, vec!["lo".to_string()]);
        assert_eq!(report.total_failed(), 1);
        assert_eq!(report.total_done(), 1);
    }

    #[test]
    fn abort_policy_stops_at_first_failure() {
        let phase = BootPhase::new(BootPhaseId::Network)
            .with_task(
                FnTask::new("boom", |_| Err(MachinedError::task_failed("boom", "nope"))).boxed(),
            )
            .with_task(
                FnTask::new("after", |rt: &mut dyn Runtime| {
                    rt.link_up("lo")?;
                    Ok(TaskOutcome::Done)
                })
                .boxed(),
            );
        let seq = BootSequencer::with_phases(vec![phase]);
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;
        let err = seq.run_boot(&mut rt, &mut log).unwrap_err();
        assert_eq!(err.kind(), "task_failed");
        // The later task must not have run.
        assert!(rt.links_up.is_empty());
    }

    // ----- custom services / hostname-less config -----

    #[test]
    fn custom_service_set_is_spawned() {
        let services = vec![
            BootService::new("apid", ["/apid"], RestartPolicy::Always),
            BootService::new("trustd", ["/trustd"], RestartPolicy::OnFailure),
        ];
        let seq = BootSequencer::with_services(services);
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;
        seq.run_boot(&mut rt, &mut log).unwrap();
        let names: Vec<String> = rt.spawned.iter().map(|(n, _)| n.clone()).collect();
        assert_eq!(names, vec!["apid".to_string(), "trustd".to_string()]);
    }

    #[test]
    fn missing_hostname_is_skipped_not_failed() {
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal")
            .with_config("version: v1alpha1\nmachine:\n  type: worker\n", None);
        let mut log = RecordingLogger::default();
        let report = seq.run_boot(&mut rt, &mut log).unwrap();
        assert!(rt.hostname.is_none());
        // The SetHostname phase still completed (task skipped), boot succeeded.
        assert!(
            report
                .phases
                .iter()
                .any(|(id, _)| *id == BootPhaseId::SetHostname)
        );
        assert!(
            log.lines
                .iter()
                .any(|l| l.contains("setHostname") && l.contains("skipped"))
        );
    }

    // ----- platform / runtime introspection -----

    #[test]
    fn runtime_reports_platform_and_disks() {
        let rt = FakeRuntime::new("aws");
        assert_eq!(rt.platform(), "aws");
        assert!(rt.has_disks());
        let c = FakeRuntime::new("container").diskless();
        assert!(!c.has_disks());
    }

    #[test]
    fn fail_policy_and_outcome_labels_are_stable() {
        assert_eq!(TaskOutcome::Done.as_str(), "done");
        assert_eq!(TaskOutcome::Skipped.as_str(), "skipped");
        assert_eq!(RestartPolicy::OnFailure.as_str(), "on-failure");
        assert_eq!(ServiceStatus::Running.as_str(), "running");
        assert_eq!(BootPhaseId::StartServices.as_str(), "startServices");
    }

    #[test]
    fn boxed_dyn_runtime_is_object_safe() {
        // Compile-time proof that the trait is object-safe and drivable as a
        // boxed trait object (the form talos-init passes in).
        let mut rt: Box<dyn Runtime> = Box::new(FakeRuntime::new("metal"));
        let seq = BootSequencer::new();
        let mut log = NullLogger;
        seq.run_boot(rt.as_mut(), &mut log).unwrap();
    }

    // ----- best-effort sysctl classifier -----

    #[test]
    fn sysctl_skip_reason_skips_sandbox_and_readonly_errnos() {
        // The sandbox / unprivileged / read-only / absent cases are skipped.
        assert_eq!(sysctl_skip_reason(EPERM), Some("EPERM"));
        assert_eq!(sysctl_skip_reason(EACCES), Some("EACCES"));
        assert_eq!(sysctl_skip_reason(EROFS), Some("EROFS"));
        assert_eq!(sysctl_skip_reason(ENOENT), Some("ENOENT"));
        // Literal Linux ABI values, in case the named consts ever drift.
        assert_eq!(sysctl_skip_reason(1), Some("EPERM"));
        assert_eq!(sysctl_skip_reason(2), Some("ENOENT"));
        assert_eq!(sysctl_skip_reason(13), Some("EACCES"));
        assert_eq!(sysctl_skip_reason(30), Some("EROFS"));
    }

    #[test]
    fn sysctl_skip_reason_fails_unexpected_errnos() {
        // Genuinely unexpected errors must propagate (None => fail the boot).
        assert_eq!(sysctl_skip_reason(0), None); // success-as-error sentinel
        assert_eq!(sysctl_skip_reason(-1), None); // no errno available
        assert_eq!(sysctl_skip_reason(5), None); // EIO
        assert_eq!(sysctl_skip_reason(22), None); // EINVAL (bad value)
        assert_eq!(sysctl_skip_reason(28), None); // ENOSPC
    }

    // ----- best-effort pseudo-fs mount classifier -----

    #[test]
    fn mount_skip_reason_skips_sandbox_already_provided_errnos() {
        // Sandbox / unprivileged / already-mounted cases are skipped.
        assert_eq!(mount_skip_reason(EPERM), Some("EPERM"));
        assert_eq!(mount_skip_reason(EACCES), Some("EACCES"));
        assert_eq!(mount_skip_reason(EBUSY), Some("EBUSY"));
        assert_eq!(mount_skip_reason(ENODEV), Some("ENODEV"));
        // Literal Linux ABI values, in case the named consts ever drift.
        assert_eq!(mount_skip_reason(1), Some("EPERM"));
        assert_eq!(mount_skip_reason(13), Some("EACCES"));
        assert_eq!(mount_skip_reason(16), Some("EBUSY"));
        assert_eq!(mount_skip_reason(19), Some("ENODEV"));
    }

    #[test]
    fn mount_skip_reason_fails_unexpected_errnos() {
        // Genuinely unexpected mount errors must propagate (None => fail boot).
        assert_eq!(mount_skip_reason(0), None); // success-as-error sentinel
        assert_eq!(mount_skip_reason(-1), None); // no errno available
        assert_eq!(mount_skip_reason(2), None); // ENOENT (missing source/target)
        assert_eq!(mount_skip_reason(5), None); // EIO
        assert_eq!(mount_skip_reason(22), None); // EINVAL (bad flags/fstype)
        assert_eq!(mount_skip_reason(28), None); // ENOSPC
    }

    // ----- best-effort privileged machine-bring-up classifier -----

    #[test]
    fn privileged_op_skip_reason_skips_sandbox_errnos() {
        // The exact "sandbox already provides / forbids" set the task specifies:
        // EPERM, EACCES, ENOSYS, EOPNOTSUPP.
        assert_eq!(privileged_op_skip_reason(EPERM), Some("EPERM"));
        assert_eq!(privileged_op_skip_reason(EACCES), Some("EACCES"));
        assert_eq!(privileged_op_skip_reason(ENOSYS), Some("ENOSYS"));
        assert_eq!(privileged_op_skip_reason(EOPNOTSUPP), Some("EOPNOTSUPP"));
        // gVisor returns ENOTTY for the AF_INET SIOCSIFFLAGS link-up ioctl it
        // does not back; treat it as a sandbox skip so `lo` bring-up is best
        // effort rather than aborting the boot under runsc.
        assert_eq!(privileged_op_skip_reason(ENOTTY), Some("ENOTTY"));
        // Literal Linux ABI values, in case the named consts ever drift.
        assert_eq!(privileged_op_skip_reason(1), Some("EPERM"));
        assert_eq!(privileged_op_skip_reason(13), Some("EACCES"));
        assert_eq!(privileged_op_skip_reason(38), Some("ENOSYS"));
        assert_eq!(privileged_op_skip_reason(95), Some("EOPNOTSUPP"));
        assert_eq!(privileged_op_skip_reason(25), Some("ENOTTY"));
    }

    #[test]
    fn privileged_op_skip_reason_fails_unexpected_errnos() {
        // Genuinely unexpected errors must propagate (None => fail the boot), so
        // metal stays full-fidelity and real bugs are not masked.
        assert_eq!(privileged_op_skip_reason(0), None); // success-as-error sentinel
        assert_eq!(privileged_op_skip_reason(-1), None); // no errno available
        assert_eq!(privileged_op_skip_reason(2), None); // ENOENT (missing iface)
        assert_eq!(privileged_op_skip_reason(5), None); // EIO
        assert_eq!(privileged_op_skip_reason(16), None); // EBUSY
        assert_eq!(privileged_op_skip_reason(22), None); // EINVAL (bad address)
    }

    // ----- runtime mode detection (informational, not load-bearing) -----

    #[test]
    fn detect_runtime_mode_flags_gvisor_as_container() {
        // gVisor's Sentry advertises itself in /proc/version. The REAL banner is
        // lowercase `-gvisor` (verified: `Linux version 4.19.0-gvisor #1 SMP`),
        // so detection must match it case-insensitively.
        let real_banner = "Linux version 4.19.0-gvisor #1 SMP Sun Jan 10 15:06:54 PST 2016";
        assert_eq!(
            detect_runtime_mode(Some(real_banner)),
            BootRuntimeMode::Container
        );
        assert!(detect_runtime_mode(Some(real_banner)).is_container());
        // A `gVisor`-cased banner must also be recognized.
        let cased = "Linux version 4.4.0 #1 SMP gVisor (compatible) ...";
        assert_eq!(detect_runtime_mode(Some(cased)), BootRuntimeMode::Container);
    }

    #[test]
    fn detect_runtime_mode_defaults_to_metal() {
        // A real kernel banner has no "gVisor" marker => metal.
        let real = "Linux version 6.6.0-0-virt (alpine) #1 SMP ...";
        assert_eq!(detect_runtime_mode(Some(real)), BootRuntimeMode::Metal);
        // An unreadable /proc/version is the safe full-fidelity default: metal.
        assert_eq!(detect_runtime_mode(None), BootRuntimeMode::Metal);
        assert!(!detect_runtime_mode(None).is_container());
    }

    #[test]
    fn boot_runtime_mode_labels_are_stable() {
        assert_eq!(BootRuntimeMode::Metal.as_str(), "metal");
        assert_eq!(BootRuntimeMode::Container.as_str(), "container");
    }

    #[test]
    fn sysctl_outcome_predicates() {
        assert!(SysctlOutcome::Applied.is_applied());
        assert!(!SysctlOutcome::Applied.is_skipped());
        assert!(SysctlOutcome::Skipped("EPERM").is_skipped());
        assert!(!SysctlOutcome::Skipped("EPERM").is_applied());
    }

    // ----- best-effort KSPP enforcement under a sandbox -----

    #[test]
    fn enforce_kspp_skips_readonly_knob_and_continues_boot() {
        // Simulate gVisor making kernel.kptr_restrict read-only (EROFS).
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("container")
            .fail_op("skip_sysctl:kernel.kptr_restrict", EROFS.to_string());
        let mut log = NullLogger;
        let report = seq.run_boot(&mut rt, &mut log).unwrap();

        // Boot completed end to end despite the unwritable knob.
        assert_eq!(report.phase_count(), 7);
        // The read-only knob was recorded as skipped, the rest applied.
        assert_eq!(
            rt.skipped_sysctls,
            vec![("kernel.kptr_restrict".to_string(), "EROFS")]
        );
        assert_eq!(rt.sysctls.len(), kspp_sysctls().len() - 1);
        assert!(rt.sysctls.iter().all(|(k, _)| k != "kernel.kptr_restrict"));
        // The skip is logged in the `[seq] sysctl <key>: skipped (<errno>)` form.
        assert!(
            rt.logs
                .iter()
                .any(|l| l.contains("sysctl kernel.kptr_restrict: skipped (EROFS)"))
        );
        // And the applied-vs-skipped summary is present.
        assert!(rt.logs.iter().any(|l| l.contains("enforceKSPP: applied")
            && l.contains("skipped")
            && l.contains("kernel.kptr_restrict")));
    }

    #[test]
    fn enforce_kspp_eperm_is_best_effort() {
        // EPERM (unprivileged) is also best effort, not a boot abort.
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("container")
            .fail_op("skip_sysctl:kernel.dmesg_restrict", EPERM.to_string());
        let mut log = NullLogger;
        seq.run_boot(&mut rt, &mut log).unwrap();
        assert_eq!(
            rt.skipped_sysctls,
            vec![("kernel.dmesg_restrict".to_string(), "EPERM")]
        );
    }

    #[test]
    fn enforce_kspp_applies_all_on_privileged_kernel() {
        // On a real privileged kernel every knob is applied; nothing skipped.
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;
        seq.run_boot(&mut rt, &mut log).unwrap();
        assert_eq!(rt.sysctls.len(), kspp_sysctls().len());
        assert!(rt.skipped_sysctls.is_empty());
        assert!(
            rt.logs
                .iter()
                .any(|l| l.contains("sysctl kernel.kptr_restrict: applied"))
        );
    }

    // ----- best-effort privileged bring-up under a sandbox (end to end) -----

    #[test]
    fn set_hostname_skipped_under_sandbox_continues_boot() {
        // Simulate gVisor/an unprivileged container denying sethostname (EPERM).
        let seq = BootSequencer::new();
        let mut rt =
            FakeRuntime::new("container").fail_op("skip_priv:set_hostname", EPERM.to_string());
        let mut log = NullLogger;
        let report = seq.run_boot(&mut rt, &mut log).unwrap();
        // Boot completed end to end despite the unwritable hostname.
        assert_eq!(report.phase_count(), 7);
        // The hostname was NOT applied, but recorded as a best-effort skip.
        assert!(rt.hostname.is_none());
        assert_eq!(
            rt.skipped_privileged,
            vec![("set_hostname".to_string(), "EPERM")]
        );
        // Services still started.
        assert_eq!(rt.spawned.len(), 3);
    }

    #[test]
    fn link_up_skipped_under_sandbox_continues_boot() {
        // Simulate the sandbox stubbing rtnetlink link-up out (ENOSYS) for lo.
        let seq = BootSequencer::new();
        let mut rt =
            FakeRuntime::new("container").fail_op("skip_priv:link_up:lo", ENOSYS.to_string());
        let mut log = NullLogger;
        let report = seq.run_boot(&mut rt, &mut log).unwrap();
        assert_eq!(report.phase_count(), 7);
        // lo was skipped; eth0 still came up normally.
        assert!(!rt.links_up.contains(&"lo".to_string()));
        assert!(rt.links_up.contains(&"eth0".to_string()));
        assert_eq!(
            rt.skipped_privileged,
            vec![("link_up:lo".to_string(), "ENOSYS")]
        );
    }

    #[test]
    fn privileged_ops_applied_on_privileged_kernel() {
        // On a real privileged kernel (metal) nothing is skipped: hostname is
        // set and every link comes up. Metal stays full-fidelity.
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;
        seq.run_boot(&mut rt, &mut log).unwrap();
        assert_eq!(rt.hostname.as_deref(), Some("talos-node"));
        assert_eq!(rt.links_up, vec!["lo".to_string(), "eth0".to_string()]);
        assert!(rt.skipped_privileged.is_empty());
    }

    #[test]
    fn set_hostname_unexpected_error_aborts_boot() {
        // A genuinely unexpected sethostname error (plain `sethostname` fail tag,
        // not a skip_priv sandbox errno) still aborts: tolerance is not blanket.
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal").fail_op("sethostname", "EFAULT");
        let mut log = NullLogger;
        let err = seq.run_boot(&mut rt, &mut log).unwrap_err();
        assert_eq!(err.kind(), "task_failed");
        // Aborted in SetHostname: no services spawned.
        assert!(rt.spawned.is_empty());
    }

    #[test]
    fn enforce_kspp_unexpected_errno_aborts_boot() {
        // A genuinely unexpected sysctl error (e.g. EINVAL/EIO via the plain
        // `sysctl:` fail tag) still aborts the boot — hardening is not weakened.
        let seq = BootSequencer::new();
        let mut rt = FakeRuntime::new("metal").fail_op("sysctl:kernel.kptr_restrict", "EINVAL");
        let mut log = NullLogger;
        let err = seq.run_boot(&mut rt, &mut log).unwrap_err();
        assert_eq!(err.kind(), "task_failed");
        // Boot aborted in the systemDirectories phase (enforceKSPP), which now
        // runs after MountPseudoFs: no services spawned.
        assert!(rt.spawned.is_empty());
        // The pseudo-filesystems were already mounted before KSPP aborted.
        assert!(rt.mounts.iter().any(|m| m.target == "/proc"));
    }
}
