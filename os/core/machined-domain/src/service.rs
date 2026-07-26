//! Long-lived services and their lifecycle state, mirroring Talos
//! `internal/app/machined/pkg/system`.
//!
//! A [`Service`] is a named daemon (etcd, kubelet, apid, ...) with declared
//! dependency [`ServiceCondition`]s. Its lifecycle is tracked by a
//! [`ServiceState`] state machine; the [`crate::supervisor`] owns the actual
//! run/restart loop via a [`crate::supervisor::ServiceRunner`].

use crate::error::{MachinedError, Result};
use os_kernel::MachineType;

/// The lifecycle state of a service, mirroring Talos `events.ServiceState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceState {
    /// Registered but not yet started.
    Initialized,
    /// Waiting on a precondition (a dependency / a file / the network).
    Waiting,
    /// Preparing to launch (pulling images, writing config).
    Preparing,
    /// The process is up but has not yet reported healthy.
    Running,
    /// The process is up and has reported healthy.
    Healthy,
    /// The process exited or failed and may be restarted.
    Failed,
    /// The service was asked to stop and has stopped.
    Finished,
    /// The service is being shut down.
    Stopping,
}

impl ServiceState {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceState::Initialized => "initialized",
            ServiceState::Waiting => "waiting",
            ServiceState::Preparing => "preparing",
            ServiceState::Running => "running",
            ServiceState::Healthy => "healthy",
            ServiceState::Failed => "failed",
            ServiceState::Finished => "finished",
            ServiceState::Stopping => "stopping",
        }
    }

    /// Whether the process is currently executing (running or healthy).
    pub fn is_up(self) -> bool {
        matches!(self, ServiceState::Running | ServiceState::Healthy)
    }

    /// Whether the service has reached a terminal resting state.
    pub fn is_terminal(self) -> bool {
        matches!(self, ServiceState::Finished)
    }

    /// Legal transitions of the service state machine. Mirrors the allowed
    /// edges in the Talos service runner.
    pub fn can_transition_to(self, next: ServiceState) -> bool {
        use ServiceState::{
            Failed, Finished, Healthy, Initialized, Preparing, Running, Stopping, Waiting,
        };
        match self {
            Initialized => matches!(next, Waiting | Preparing | Finished),
            Waiting => matches!(next, Preparing | Failed | Finished | Stopping),
            Preparing => matches!(next, Running | Failed | Stopping),
            Running => matches!(next, Healthy | Failed | Stopping | Finished),
            Healthy => matches!(next, Failed | Stopping | Finished | Running),
            // A failed service may be restarted (back through waiting/preparing)
            // or finally given up on (finished).
            Failed => matches!(next, Waiting | Preparing | Finished),
            Stopping => matches!(next, Finished | Failed),
            Finished => false,
        }
    }
}

/// A precondition a service waits on before it may start.
///
/// Mirrors the `conditions` package: services declare dependencies on other
/// services being up, on machine config being present, or on the network being
/// ready.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceCondition {
    /// Another named service must be at least `Running`.
    ServiceUp(String),
    /// Another named service must be `Healthy`.
    ServiceHealthy(String),
    /// Machine configuration must have been applied.
    ConfigPresent,
    /// The node must be a control-plane node for the dependent service to run.
    ControlPlaneOnly,
    /// The network must be ready.
    NetworkReady,
}

impl ServiceCondition {
    /// A short label for the condition (used in logs).
    pub fn label(&self) -> String {
        match self {
            ServiceCondition::ServiceUp(s) => format!("service({s}).up"),
            ServiceCondition::ServiceHealthy(s) => format!("service({s}).healthy"),
            ServiceCondition::ConfigPresent => "config.present".to_string(),
            ServiceCondition::ControlPlaneOnly => "role.controlplane".to_string(),
            ServiceCondition::NetworkReady => "network.ready".to_string(),
        }
    }
}

/// A snapshot of the world a condition is evaluated against.
#[derive(Clone, Copy)]
pub struct ConditionEnv<'a> {
    /// Lookup of a service's current state by name.
    pub service_state: &'a dyn Fn(&str) -> Option<ServiceState>,
    /// Whether machine config has been applied.
    pub configured: bool,
    /// The machine role.
    pub machine_type: MachineType,
    /// Whether the network is ready.
    pub network_ready: bool,
}

impl ServiceCondition {
    /// Evaluate whether the condition is currently satisfied.
    pub fn is_satisfied(&self, env: &ConditionEnv<'_>) -> bool {
        match self {
            ServiceCondition::ServiceUp(name) => {
                (env.service_state)(name).is_some_and(ServiceState::is_up)
            }
            ServiceCondition::ServiceHealthy(name) => {
                (env.service_state)(name) == Some(ServiceState::Healthy)
            }
            ServiceCondition::ConfigPresent => env.configured,
            ServiceCondition::ControlPlaneOnly => env.machine_type.is_control_plane(),
            ServiceCondition::NetworkReady => env.network_ready,
        }
    }
}

/// A long-lived service definition plus its current lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    id: String,
    state: ServiceState,
    conditions: Vec<ServiceCondition>,
    restarts: u32,
    max_restarts: u32,
}

impl Service {
    /// Define a new service with the given id and dependency conditions.
    pub fn new(id: impl Into<String>, conditions: Vec<ServiceCondition>) -> Self {
        Service {
            id: id.into(),
            state: ServiceState::Initialized,
            conditions,
            restarts: 0,
            max_restarts: 3,
        }
    }

    /// Set the maximum number of automatic restarts before the service is
    /// considered permanently failed.
    pub fn with_max_restarts(mut self, max: u32) -> Self {
        self.max_restarts = max;
        self
    }

    /// The service id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The current lifecycle state.
    pub fn state(&self) -> ServiceState {
        self.state
    }

    /// The declared preconditions.
    pub fn conditions(&self) -> &[ServiceCondition] {
        &self.conditions
    }

    /// The number of times the service has restarted.
    pub fn restarts(&self) -> u32 {
        self.restarts
    }

    /// Whether every precondition is satisfied in the given environment.
    pub fn dependencies_met(&self, env: &ConditionEnv<'_>) -> bool {
        self.conditions.iter().all(|c| c.is_satisfied(env))
    }

    /// Attempt a state transition, validating it against the state machine.
    pub fn transition_to(&mut self, next: ServiceState) -> Result<()> {
        if !self.state.can_transition_to(next) {
            return Err(MachinedError::illegal_transition(
                self.state.as_str(),
                next.as_str(),
            ));
        }
        // Count restarts each time we re-enter the waiting/preparing path from
        // a failure.
        if self.state == ServiceState::Failed
            && matches!(next, ServiceState::Waiting | ServiceState::Preparing)
        {
            self.restarts += 1;
        }
        self.state = next;
        Ok(())
    }

    /// Whether the service still has restart budget left after a failure.
    pub fn can_restart(&self) -> bool {
        self.state == ServiceState::Failed && self.restarts < self.max_restarts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_with(configured: bool, mt: MachineType) -> (Vec<(String, ServiceState)>, bool) {
        let _ = (configured, mt);
        (Vec::new(), false)
    }

    #[test]
    fn legal_and_illegal_transitions() {
        let mut s = Service::new("etcd", vec![]);
        assert_eq!(s.state(), ServiceState::Initialized);
        s.transition_to(ServiceState::Preparing).unwrap();
        s.transition_to(ServiceState::Running).unwrap();
        s.transition_to(ServiceState::Healthy).unwrap();
        // Healthy -> Initialized is illegal.
        let err = s.transition_to(ServiceState::Initialized).unwrap_err();
        assert_eq!(err.kind(), "illegal_transition");
    }

    #[test]
    fn restart_budget_counts() {
        let mut s = Service::new("kubelet", vec![]).with_max_restarts(2);
        s.transition_to(ServiceState::Preparing).unwrap();
        s.transition_to(ServiceState::Running).unwrap();
        s.transition_to(ServiceState::Failed).unwrap();
        assert!(s.can_restart());
        s.transition_to(ServiceState::Preparing).unwrap();
        assert_eq!(s.restarts(), 1);
        s.transition_to(ServiceState::Failed).unwrap();
        s.transition_to(ServiceState::Preparing).unwrap();
        assert_eq!(s.restarts(), 2);
        s.transition_to(ServiceState::Failed).unwrap();
        assert!(!s.can_restart(), "budget exhausted");
    }

    #[test]
    fn conditions_gate_on_dependencies() {
        let states: Vec<(String, ServiceState)> = vec![("etcd".to_string(), ServiceState::Healthy)];
        let lookup = |name: &str| states.iter().find(|(n, _)| n == name).map(|(_, s)| *s);
        let env = ConditionEnv {
            service_state: &lookup,
            configured: true,
            machine_type: MachineType::ControlPlane,
            network_ready: true,
        };
        let apid = Service::new(
            "apid",
            vec![
                ServiceCondition::ConfigPresent,
                ServiceCondition::ServiceHealthy("etcd".to_string()),
                ServiceCondition::ControlPlaneOnly,
            ],
        );
        assert!(apid.dependencies_met(&env));
    }

    #[test]
    fn unmet_dependency_blocks() {
        let lookup = |_name: &str| None;
        let env = ConditionEnv {
            service_state: &lookup,
            configured: false,
            machine_type: MachineType::Worker,
            network_ready: false,
        };
        let svc = Service::new(
            "kube-apiserver",
            vec![
                ServiceCondition::ControlPlaneOnly,
                ServiceCondition::ConfigPresent,
            ],
        );
        assert!(!svc.dependencies_met(&env));
    }

    #[test]
    fn condition_labels() {
        assert_eq!(
            ServiceCondition::ServiceUp("etcd".to_string()).label(),
            "service(etcd).up"
        );
        assert_eq!(ServiceCondition::NetworkReady.label(), "network.ready");
    }

    #[test]
    fn helper_compiles() {
        let (v, b) = env_with(true, MachineType::Worker);
        assert!(v.is_empty());
        assert!(!b);
    }
}
