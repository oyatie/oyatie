//! Extension services lifecycle.
//!
//! Some extensions (`kind: service`) ship a `spec.yaml` describing a long-running
//! process Talos supervises through its service runtime (see
//! `internal/app/machined/pkg/system/services/extension.go`). This module models
//! the service spec, its restart policy, a [`ServiceLauncher`] OS-boundary trait
//! (the thing that actually starts/stops processes), an in-memory launcher for
//! tests, and the [`ExtensionService`] state machine that wires them together
//! and implements [`os_kernel::traits::Runnable`].

use std::collections::HashMap;

use os_kernel::error::{Error, Result};
use os_kernel::traits::{RunState, Runnable};

use crate::config::ExtensionServiceConfig;

/// What Talos should do when a service process exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartPolicy {
    /// Never restart; a clean exit is success.
    Never,
    /// Restart only if the process exited non-zero.
    OnFailure,
    /// Always restart (the default for daemon-style extensions).
    Always,
}

impl RestartPolicy {
    /// Parse from the spec string.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim() {
            "never" => Ok(RestartPolicy::Never),
            "onFailure" | "on-failure" => Ok(RestartPolicy::OnFailure),
            "always" => Ok(RestartPolicy::Always),
            other => Err(Error::parse(format!("unknown restart policy '{other}'"))),
        }
    }

    /// Whether a process that exited with `code` should be restarted.
    pub fn should_restart(self, exit_code: i32) -> bool {
        match self {
            RestartPolicy::Never => false,
            RestartPolicy::OnFailure => exit_code != 0,
            RestartPolicy::Always => true,
        }
    }
}

/// The `spec.yaml` of a service extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSpec {
    /// Service name (used as the resource id, `ext-<name>`).
    pub name: String,
    /// Absolute path to the entrypoint binary inside the extension rootfs.
    pub entrypoint: String,
    /// Arguments passed to the entrypoint.
    pub args: Vec<String>,
    /// Restart policy.
    pub restart: RestartPolicy,
    /// Service names this one depends on (must be running first).
    pub depends_on: Vec<String>,
}

impl ServiceSpec {
    /// Construct a minimal spec.
    pub fn new(name: impl Into<String>, entrypoint: impl Into<String>) -> Self {
        ServiceSpec {
            name: name.into(),
            entrypoint: entrypoint.into(),
            args: Vec::new(),
            restart: RestartPolicy::Always,
            depends_on: Vec::new(),
        }
    }

    /// The supervised service id Talos exposes (`ext-<name>`).
    pub fn service_id(&self) -> String {
        format!("ext-{}", self.name)
    }

    /// Validate structural requirements.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::invalid("service name must not be empty"));
        }
        if !self.entrypoint.starts_with('/') {
            return Err(Error::invalid(format!(
                "service '{}' entrypoint '{}' must be absolute",
                self.name, self.entrypoint
            )));
        }
        Ok(())
    }
}

/// A handle a launcher returns for a started process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessHandle(pub u64);

/// OS boundary: the thing that actually launches and kills service processes.
///
/// Real Talos talks to the service runtime / containerd; here it's a trait so
/// tests can drive the lifecycle with an in-memory fake.
pub trait ServiceLauncher {
    /// Start `spec` with the given rendered environment. Returns a handle.
    fn launch(&mut self, spec: &ServiceSpec, env: &[String]) -> Result<ProcessHandle>;

    /// Stop the process identified by `handle`.
    fn terminate(&mut self, handle: ProcessHandle) -> Result<()>;

    /// Whether the process is still alive.
    fn is_alive(&self, handle: ProcessHandle) -> bool;
}

/// In-memory [`ServiceLauncher`] used by tests and the reconcile model. It
/// records every launch and lets tests simulate a process exiting.
#[derive(Debug, Default)]
pub struct InMemoryLauncher {
    next: u64,
    /// handle -> (alive, last env, exit_code if exited)
    procs: HashMap<u64, ProcState>,
    /// Ordered log of launched service names.
    pub launch_log: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProcState {
    alive: bool,
    env: Vec<String>,
}

impl InMemoryLauncher {
    /// A fresh launcher.
    pub fn new() -> Self {
        InMemoryLauncher {
            next: 1,
            procs: HashMap::new(),
            launch_log: Vec::new(),
        }
    }

    /// The environment a given handle was launched with.
    pub fn env_of(&self, handle: ProcessHandle) -> Option<&[String]> {
        self.procs.get(&handle.0).map(|p| p.env.as_slice())
    }

    /// Simulate the process exiting (e.g. crash). Marks it not-alive.
    pub fn simulate_exit(&mut self, handle: ProcessHandle) {
        if let Some(p) = self.procs.get_mut(&handle.0) {
            p.alive = false;
        }
    }
}

impl ServiceLauncher for InMemoryLauncher {
    fn launch(&mut self, spec: &ServiceSpec, env: &[String]) -> Result<ProcessHandle> {
        spec.validate()?;
        let id = self.next;
        self.next += 1;
        self.procs.insert(
            id,
            ProcState {
                alive: true,
                env: env.to_vec(),
            },
        );
        self.launch_log.push(spec.name.clone());
        Ok(ProcessHandle(id))
    }

    fn terminate(&mut self, handle: ProcessHandle) -> Result<()> {
        match self.procs.get_mut(&handle.0) {
            Some(p) => {
                p.alive = false;
                Ok(())
            }
            None => Err(Error::not_found(format!(
                "no process for handle {}",
                handle.0
            ))),
        }
    }

    fn is_alive(&self, handle: ProcessHandle) -> bool {
        self.procs.get(&handle.0).map(|p| p.alive).unwrap_or(false)
    }
}

/// A supervised extension service: the spec, optional user config, and current
/// lifecycle state. Generic over the launcher so tests inject the fake.
pub struct ExtensionService<L: ServiceLauncher> {
    spec: ServiceSpec,
    config: Option<ExtensionServiceConfig>,
    launcher: L,
    state: RunState,
    handle: Option<ProcessHandle>,
    id: String,
    /// Number of times the service has been (re)started.
    pub start_count: u32,
}

impl<L: ServiceLauncher> ExtensionService<L> {
    /// Wire a service to a launcher.
    pub fn new(spec: ServiceSpec, launcher: L) -> Self {
        let id = spec.service_id();
        ExtensionService {
            spec,
            config: None,
            launcher,
            state: RunState::Initialized,
            handle: None,
            id,
            start_count: 0,
        }
    }

    /// Attach a user `ExtensionServiceConfig`, validating it first.
    pub fn configure(&mut self, config: ExtensionServiceConfig) -> Result<()> {
        config.validate()?;
        if config.name != self.spec.name {
            return Err(Error::invalid(format!(
                "config for '{}' does not match service '{}'",
                config.name, self.spec.name
            )));
        }
        self.config = Some(config);
        Ok(())
    }

    /// The effective environment passed to the process.
    pub fn effective_env(&self) -> Vec<String> {
        self.config
            .as_ref()
            .map(|c| c.rendered_env())
            .unwrap_or_default()
    }

    /// Access the launcher (for assertions in tests).
    pub fn launcher(&self) -> &L {
        &self.launcher
    }

    /// The spec.
    pub fn spec(&self) -> &ServiceSpec {
        &self.spec
    }

    /// Reconcile health: if the process died but the restart policy says it
    /// should come back, mark the service Failed so the controller restarts it.
    /// Returns whether a restart is warranted.
    pub fn reconcile_health(&mut self) -> bool {
        if self.state == RunState::Running
            && let Some(h) = self.handle
                && !self.launcher.is_alive(h) {
                    self.state = RunState::Failed;
                    return self.spec.restart.should_restart(1);
                }
        false
    }
}

impl<L: ServiceLauncher> Runnable for ExtensionService<L> {
    fn id(&self) -> &str {
        &self.id
    }

    fn start(&mut self) -> Result<()> {
        if self.state == RunState::Running {
            return Ok(());
        }
        if !RunState::Initialized.can_transition_to(RunState::Preparing) {
            // Unreachable given the static rules, but keeps the model honest.
            return Err(Error::invalid_state("cannot prepare"));
        }
        self.spec.validate()?;
        let env = self.effective_env();
        let handle = self.launcher.launch(&self.spec, &env)?;
        self.handle = Some(handle);
        self.state = RunState::Running;
        self.start_count += 1;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if self.state != RunState::Running {
            return Ok(());
        }
        if let Some(h) = self.handle.take() {
            self.launcher.terminate(h)?;
        }
        self.state = RunState::Stopped;
        Ok(())
    }

    fn state(&self) -> RunState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> ServiceSpec {
        ServiceSpec::new("gvisor", "/usr/local/bin/runsc")
    }

    #[test]
    fn restart_policy_logic() {
        assert!(!RestartPolicy::Never.should_restart(1));
        assert!(RestartPolicy::OnFailure.should_restart(1));
        assert!(!RestartPolicy::OnFailure.should_restart(0));
        assert!(RestartPolicy::Always.should_restart(0));
        assert_eq!(
            RestartPolicy::parse("on-failure").unwrap(),
            RestartPolicy::OnFailure
        );
        assert!(RestartPolicy::parse("bogus").is_err());
    }

    #[test]
    fn spec_validation_and_id() {
        let s = spec();
        assert_eq!(s.service_id(), "ext-gvisor");
        assert!(s.validate().is_ok());
        let bad = ServiceSpec::new("x", "relative");
        assert!(bad.validate().is_err());
    }

    #[test]
    fn lifecycle_start_stop() {
        let mut svc = ExtensionService::new(spec(), InMemoryLauncher::new());
        assert_eq!(svc.state(), RunState::Initialized);
        assert!(!svc.is_healthy());
        svc.start().unwrap();
        assert_eq!(svc.state(), RunState::Running);
        assert!(svc.is_healthy());
        assert_eq!(svc.start_count, 1);
        assert_eq!(svc.launcher().launch_log, vec!["gvisor".to_string()]);
        // Idempotent start.
        svc.start().unwrap();
        assert_eq!(svc.start_count, 1);
        svc.stop().unwrap();
        assert_eq!(svc.state(), RunState::Stopped);
        // Idempotent stop.
        svc.stop().unwrap();
    }

    #[test]
    fn config_is_applied_to_env() {
        let mut svc = ExtensionService::new(spec(), InMemoryLauncher::new());
        let cfg = ExtensionServiceConfig::new("gvisor").with_env("LOG", "debug");
        svc.configure(cfg).unwrap();
        svc.start().unwrap();
        let h = svc.handle.unwrap();
        assert_eq!(
            svc.launcher().env_of(h).unwrap(),
            &["LOG=debug".to_string()]
        );
    }

    #[test]
    fn configure_rejects_mismatched_name() {
        let mut svc = ExtensionService::new(spec(), InMemoryLauncher::new());
        let cfg = ExtensionServiceConfig::new("other");
        assert!(svc.configure(cfg).is_err());
    }

    #[test]
    fn reconcile_detects_dead_process() {
        let mut svc = ExtensionService::new(spec(), InMemoryLauncher::new());
        svc.start().unwrap();
        let h = svc.handle.unwrap();
        svc.launcher.simulate_exit(h);
        let needs_restart = svc.reconcile_health();
        assert!(needs_restart);
        assert_eq!(svc.state(), RunState::Failed);
    }

    #[test]
    fn never_policy_does_not_request_restart() {
        let mut s = spec();
        s.restart = RestartPolicy::Never;
        let mut svc = ExtensionService::new(s, InMemoryLauncher::new());
        svc.start().unwrap();
        let h = svc.handle.unwrap();
        svc.launcher.simulate_exit(h);
        assert!(!svc.reconcile_health());
        assert_eq!(svc.state(), RunState::Failed);
    }
}
