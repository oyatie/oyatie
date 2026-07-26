//! Service-state and service-health conditions.
//!
//! Mirrors Talos's `conditions.WaitForService` (see
//! `pkg/conditions/service.go` and the `system/health` event stream): the boot
//! sequencer's task dependencies are expressed as "wait for service `X` to be
//! `Running`/`Healthy`". A service goes through a lifecycle and reports health
//! independently of its run state (a process can be `Running` but not yet
//! `Healthy`).

use crate::condition::{Condition, Poll};
use std::collections::HashMap;

/// Run state of a Talos service, mirroring `system/events` `ServiceState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// Defined but not yet started.
    Initialized,
    /// Waiting on its own conditions/dependencies.
    Preparing,
    /// Process is up.
    Running,
    /// Stopped cleanly.
    Finished,
    /// Stopped with an error / crashed.
    Failed,
    /// Intentionally skipped (e.g. not applicable on this platform).
    Skipped,
}

impl ServiceState {
    /// Stable lowercase label, as Talos surfaces in events.
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceState::Initialized => "Initialized",
            ServiceState::Preparing => "Preparing",
            ServiceState::Running => "Running",
            ServiceState::Finished => "Finished",
            ServiceState::Failed => "Failed",
            ServiceState::Skipped => "Skipped",
        }
    }

    /// A terminal failure state that can never become `Running`.
    pub fn is_failed(&self) -> bool {
        matches!(self, ServiceState::Failed)
    }
}

/// Liveness/readiness of a service, decoupled from its [`ServiceState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    /// Health not yet evaluated.
    Unknown,
    /// Last health check passed.
    Healthy,
    /// Last health check failed.
    Unhealthy,
}

/// A snapshot of one service as the runtime would report it.
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    /// Service id, e.g. `"etcd"`, `"kubelet"`, `"cri"`.
    pub id: String,
    /// Current run state.
    pub state: ServiceState,
    /// Current health (only meaningful while `Running`).
    pub health: Health,
}

impl ServiceStatus {
    /// Convenience constructor.
    pub fn new(id: impl Into<String>, state: ServiceState, health: Health) -> Self {
        ServiceStatus {
            id: id.into(),
            state,
            health,
        }
    }
}

/// Read-only view of the service runtime: "what is the status of service X?".
///
/// The OS boundary for service conditions. Production would read the
/// `system.Services` registry / event stream; tests use [`ServiceRegistry`].
pub trait ServiceProbe {
    /// Current status of `id`, or `None` if the service is unknown.
    fn status(&self, id: &str) -> Option<ServiceStatus>;
}

/// In-memory [`ServiceProbe`].
#[derive(Debug, Default, Clone)]
pub struct ServiceRegistry {
    services: HashMap<String, ServiceStatus>,
}

impl ServiceRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        ServiceRegistry {
            services: HashMap::new(),
        }
    }

    /// Insert or replace a service status.
    pub fn set(&mut self, status: ServiceStatus) {
        self.services.insert(status.id.clone(), status);
    }

    /// Update just the run state of an existing service (no-op if unknown).
    pub fn set_state(&mut self, id: &str, state: ServiceState) {
        if let Some(s) = self.services.get_mut(id) {
            s.state = state;
        }
    }

    /// Update just the health of an existing service (no-op if unknown).
    pub fn set_health(&mut self, id: &str, health: Health) {
        if let Some(s) = self.services.get_mut(id) {
            s.health = health;
        }
    }
}

impl ServiceProbe for ServiceRegistry {
    fn status(&self, id: &str) -> Option<ServiceStatus> {
        self.services.get(id).cloned()
    }
}

/// Wait for a service to reach [`ServiceState::Running`].
///
/// Analogue of `conditions.WaitForService(ServiceStateRunning, id)`.
pub struct WaitForServiceState<'a, P: ServiceProbe> {
    probe: &'a P,
    id: String,
    target: ServiceState,
}

impl<'a, P: ServiceProbe> WaitForServiceState<'a, P> {
    /// Construct a condition waiting for `id` to reach `target`.
    pub fn new(probe: &'a P, id: impl Into<String>, target: ServiceState) -> Self {
        WaitForServiceState {
            probe,
            id: id.into(),
            target,
        }
    }

    /// Convenience: wait for the service to be `Running`.
    pub fn running(probe: &'a P, id: impl Into<String>) -> Self {
        Self::new(probe, id, ServiceState::Running)
    }
}

impl<P: ServiceProbe> Condition for WaitForServiceState<'_, P> {
    fn poll(&self) -> Poll {
        match self.probe.status(&self.id) {
            None => Poll::Pending(self.describe()),
            Some(s) if s.state == self.target => Poll::Ready,
            // A failed service can never become Running: surface a hard error
            // instead of spinning until timeout (matches retry's permanent err).
            Some(s) if s.state.is_failed() && self.target != ServiceState::Failed => {
                Poll::Failed(os_kernel::Error::invalid_state(format!(
                    "service {:?} failed while waiting for state {}",
                    self.id,
                    self.target.as_str()
                )))
            }
            Some(_) => Poll::Pending(self.describe()),
        }
    }

    fn describe(&self) -> String {
        format!(
            "service {:?} to be in state {}",
            self.id,
            self.target.as_str()
        )
    }
}

/// Wait for a service to be `Running` *and* [`Health::Healthy`].
///
/// Analogue of Talos's health-gated dependency (`service to be "up"`).
pub struct WaitForServiceHealthy<'a, P: ServiceProbe> {
    probe: &'a P,
    id: String,
}

impl<'a, P: ServiceProbe> WaitForServiceHealthy<'a, P> {
    /// Construct a health condition for `id`.
    pub fn new(probe: &'a P, id: impl Into<String>) -> Self {
        WaitForServiceHealthy {
            probe,
            id: id.into(),
        }
    }
}

impl<P: ServiceProbe> Condition for WaitForServiceHealthy<'_, P> {
    fn poll(&self) -> Poll {
        match self.probe.status(&self.id) {
            None => Poll::Pending(self.describe()),
            Some(s) if s.state.is_failed() => Poll::Failed(os_kernel::Error::invalid_state(
                format!("service {:?} failed", self.id),
            )),
            Some(s) if s.state == ServiceState::Running && s.health == Health::Healthy => {
                Poll::Ready
            }
            Some(_) => Poll::Pending(self.describe()),
        }
    }

    fn describe(&self) -> String {
        format!("service {:?} to be \"up\"", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{Poller, SimClock};

    #[test]
    fn unknown_service_is_pending() {
        let reg = ServiceRegistry::new();
        let cond = WaitForServiceState::running(&reg, "etcd");
        assert!(matches!(cond.poll(), Poll::Pending(_)));
    }

    #[test]
    fn running_state_is_ready() {
        let mut reg = ServiceRegistry::new();
        reg.set(ServiceStatus::new(
            "etcd",
            ServiceState::Running,
            Health::Unknown,
        ));
        let cond = WaitForServiceState::running(&reg, "etcd");
        assert_eq!(cond.poll(), Poll::Ready);
    }

    #[test]
    fn preparing_is_pending_then_running_ready() {
        let mut reg = ServiceRegistry::new();
        reg.set(ServiceStatus::new(
            "kubelet",
            ServiceState::Preparing,
            Health::Unknown,
        ));
        {
            let cond = WaitForServiceState::running(&reg, "kubelet");
            assert!(matches!(cond.poll(), Poll::Pending(_)));
        }
        reg.set_state("kubelet", ServiceState::Running);
        let cond = WaitForServiceState::running(&reg, "kubelet");
        assert_eq!(cond.poll(), Poll::Ready);
    }

    #[test]
    fn failed_service_fails_running_wait() {
        let mut reg = ServiceRegistry::new();
        reg.set(ServiceStatus::new(
            "cri",
            ServiceState::Failed,
            Health::Unhealthy,
        ));
        let cond = WaitForServiceState::running(&reg, "cri");
        match cond.poll() {
            Poll::Failed(e) => assert_eq!(e.kind(), "invalid_state"),
            other => panic!("expected failure, got {:?}", other),
        }
    }

    #[test]
    fn healthy_requires_running_and_healthy() {
        let mut reg = ServiceRegistry::new();
        reg.set(ServiceStatus::new(
            "etcd",
            ServiceState::Running,
            Health::Unhealthy,
        ));
        {
            let cond = WaitForServiceHealthy::new(&reg, "etcd");
            assert!(matches!(cond.poll(), Poll::Pending(_)));
            assert_eq!(cond.describe(), "service \"etcd\" to be \"up\"");
        }
        reg.set_health("etcd", Health::Healthy);
        let cond = WaitForServiceHealthy::new(&reg, "etcd");
        assert_eq!(cond.poll(), Poll::Ready);
    }

    #[test]
    fn healthy_wait_drives_to_ready() {
        let mut reg = ServiceRegistry::new();
        reg.set(ServiceStatus::new(
            "apid",
            ServiceState::Running,
            Health::Healthy,
        ));
        let clock = SimClock::new(0);
        let cond = WaitForServiceHealthy::new(&reg, "apid");
        let report = cond.wait(&clock, Poller::new(5, 10)).unwrap();
        assert_eq!(report.attempts, 1);
    }

    #[test]
    fn state_labels_are_stable() {
        assert_eq!(ServiceState::Running.as_str(), "Running");
        assert_eq!(ServiceState::Failed.as_str(), "Failed");
        assert!(ServiceState::Failed.is_failed());
        assert!(!ServiceState::Running.is_failed());
    }
}
