//! The service supervisor, mirroring Talos `internal/app/machined/pkg/system`.
//!
//! The [`Supervisor`] is the registry of [`Service`]s and the engine that
//! starts them in dependency order, restarting failed ones within their
//! restart budget. The actual process execution boundary is abstracted by the
//! [`ServiceLauncher`] trait so tests can drive it deterministically without
//! spawning real processes / containerd.

use crate::error::{MachinedError, Result};
use crate::events::{EventKind, EventStream};
use crate::service::{ConditionEnv, Service, ServiceState};
use os_kernel::MachineType;
use os_runtime_cri_domain::{
    REGISTRYD_SERVICE_ID, RegistrydContentResponse, RegistrydHealthProbe, RegistrydRuntimeService,
    RegistrydServiceError, RegistrydServiceManager, RegistrydState,
};
use std::collections::HashMap;

/// The boundary to the real process runtime (containerd / exec).
///
/// In production this would launch a container or fork a process; in tests an
/// in-memory implementation decides whether each launch succeeds and whether
/// the service becomes healthy.
pub trait ServiceLauncher {
    /// Launch the service. Returns `Ok(true)` if the process came up and is
    /// healthy, `Ok(false)` if it came up but is not yet healthy, or `Err` if
    /// the launch failed.
    fn launch(&mut self, id: &str) -> Result<bool>;

    /// Launch registryd with the runtime payload selected by the image-cache plan.
    ///
    /// Implementations that do not need registryd-specific payload access can
    /// use the generic launch path.
    fn launch_registryd_runtime_service(
        &mut self,
        id: &str,
        service: &RegistrydRuntimeService,
    ) -> Result<bool> {
        let _ = service;
        self.launch(id)
    }

    /// Stop the service.
    fn stop(&mut self, id: &str) -> Result<()> {
        let _ = id;
        Ok(())
    }
}

/// Drives a single [`Service`] through its lifecycle against a [`ServiceLauncher`].
///
/// Mirrors the Talos `ServiceRunner`: it owns the run/health/restart loop for
/// one service.
#[derive(Debug)]
pub struct ServiceRunner {
    service: Service,
}

impl ServiceRunner {
    /// Wrap a service definition in a runner.
    pub fn new(service: Service) -> Self {
        ServiceRunner { service }
    }

    /// The underlying service.
    pub fn service(&self) -> &Service {
        &self.service
    }

    /// The current state of the wrapped service.
    pub fn state(&self) -> ServiceState {
        self.service.state()
    }

    /// Attempt to start the service: check preconditions, then launch.
    ///
    /// If preconditions are unmet the service moves to `Waiting` and the call
    /// returns `Ok(false)` (the supervisor will retry later). On a successful
    /// launch it moves to `Running` then `Healthy` (if healthy immediately).
    pub fn start(
        &mut self,
        env: &ConditionEnv<'_>,
        launcher: &mut dyn ServiceLauncher,
    ) -> Result<bool> {
        self.start_with_launch(env, |id| launcher.launch(id))
    }

    /// Attempt to start registryd with its loaded runtime-service payload.
    pub fn start_registryd_runtime_service(
        &mut self,
        env: &ConditionEnv<'_>,
        launcher: &mut dyn ServiceLauncher,
        service: &RegistrydRuntimeService,
    ) -> Result<bool> {
        self.start_with_launch(env, |id| {
            launcher.launch_registryd_runtime_service(id, service)
        })
    }

    fn start_with_launch<F>(&mut self, env: &ConditionEnv<'_>, launch: F) -> Result<bool>
    where
        F: FnOnce(&str) -> Result<bool>,
    {
        if !self.service.dependencies_met(env) {
            // Only move to waiting if we can legally do so.
            if self.service.state() == ServiceState::Initialized {
                self.service.transition_to(ServiceState::Waiting)?;
            }
            return Ok(false);
        }

        // Move toward preparing from whatever non-running state we are in.
        match self.service.state() {
            ServiceState::Initialized | ServiceState::Waiting | ServiceState::Failed => {
                if self.service.state() == ServiceState::Initialized {
                    self.service.transition_to(ServiceState::Waiting)?;
                }
                self.service.transition_to(ServiceState::Preparing)?;
            }
            ServiceState::Preparing => {}
            other => {
                return Err(MachinedError::service_error(
                    self.service.id(),
                    format!("cannot start from state {}", other.as_str()),
                ));
            }
        }

        match launch(self.service.id()) {
            Ok(healthy) => {
                self.service.transition_to(ServiceState::Running)?;
                if healthy {
                    self.service.transition_to(ServiceState::Healthy)?;
                }
                Ok(true)
            }
            Err(e) => {
                self.service.transition_to(ServiceState::Failed)?;
                Err(e)
            }
        }
    }

    /// Mark the service as failed (e.g. its process exited).
    pub fn mark_failed(&mut self) -> Result<()> {
        self.service.transition_to(ServiceState::Failed)
    }

    /// Whether the service can be restarted within its budget.
    pub fn can_restart(&self) -> bool {
        self.service.can_restart()
    }

    /// Stop the service.
    pub fn stop(&mut self, launcher: &mut dyn ServiceLauncher) -> Result<()> {
        if self.service.state() == ServiceState::Finished {
            return Ok(());
        }
        // Move through Stopping where legal, then Finished.
        if self
            .service
            .state()
            .can_transition_to(ServiceState::Stopping)
        {
            self.service.transition_to(ServiceState::Stopping)?;
        }
        launcher.stop(self.service.id())?;
        self.service.transition_to(ServiceState::Finished)
    }
}

/// The registry + engine that supervises all node services.
pub struct Supervisor {
    runners: HashMap<String, ServiceRunner>,
    order: Vec<String>,
    configured: bool,
    machine_type: MachineType,
    network_ready: bool,
    events: EventStream,
    registryd_runtime_service: Option<RegistrydRuntimeService>,
}

impl Supervisor {
    /// Create a supervisor for a machine with the given role.
    pub fn new(machine_type: MachineType) -> Self {
        Supervisor {
            runners: HashMap::new(),
            order: Vec::new(),
            configured: false,
            machine_type,
            network_ready: false,
            events: EventStream::default(),
            registryd_runtime_service: None,
        }
    }

    /// Borrow the supervisor's event stream (service state-change events).
    pub fn events(&self) -> &EventStream {
        &self.events
    }

    /// Runtime registryd service payload loaded by the service manager, if any.
    pub fn registryd_runtime_service(&self) -> Option<&RegistrydRuntimeService> {
        self.registryd_runtime_service.as_ref()
    }

    /// Serve one host-safe registryd request through the loaded runtime service.
    ///
    /// Source Talos binds `registry.NewService(NewMultiPathFS(...))` inside the
    /// loaded `registryd` service. The supervisor stores that runtime payload,
    /// so this method exposes the same request delegation without opening the
    /// loopback socket in tests or host-side boot modeling.
    pub fn handle_registryd_request(
        &self,
        method: &str,
        target: &str,
    ) -> Option<RegistrydContentResponse> {
        self.registryd_runtime_service
            .as_ref()?
            .handle_request(method, target)
    }

    /// Emit a service state-change event and return the runner's new state.
    fn emit_state(&mut self, id: &str, state: ServiceState) {
        self.events.publish(EventKind::ServiceStateChange {
            service: id.to_string(),
            state,
        });
    }

    /// Mark machine config as applied (unblocks `ConfigPresent` conditions).
    pub fn set_configured(&mut self, configured: bool) {
        self.configured = configured;
    }

    /// Mark the network ready (unblocks `NetworkReady` conditions).
    pub fn set_network_ready(&mut self, ready: bool) {
        self.network_ready = ready;
    }

    /// Register a service. Preserves registration order for deterministic
    /// startup.
    pub fn register(&mut self, service: Service) {
        let id = service.id().to_string();
        if !self.runners.contains_key(&id) {
            self.order.push(id.clone());
        }
        self.runners.insert(id, ServiceRunner::new(service));
    }

    /// Number of registered services.
    pub fn len(&self) -> usize {
        self.runners.len()
    }

    /// Whether no services are registered.
    pub fn is_empty(&self) -> bool {
        self.runners.is_empty()
    }

    /// The current state of a service by id, if registered.
    pub fn state_of(&self, id: &str) -> Option<ServiceState> {
        self.runners.get(id).map(ServiceRunner::state)
    }

    /// Promote a running service to healthy after an explicit health observation.
    ///
    /// This keeps launch success distinct from readiness: services can remain
    /// `Running` until their source health check succeeds.
    pub fn mark_service_healthy(&mut self, id: &str) -> Result<bool> {
        let before = self
            .runners
            .get(id)
            .map(ServiceRunner::state)
            .ok_or_else(|| MachinedError::not_found(format!("service {id}")))?;

        match before {
            ServiceState::Healthy => return Ok(false),
            ServiceState::Running => {}
            other => {
                return Err(MachinedError::service_error(
                    id,
                    format!("cannot mark healthy from state {}", other.as_str()),
                ));
            }
        }

        let runner = self
            .runners
            .get_mut(id)
            .expect("runner exists after lookup");
        runner.service.transition_to(ServiceState::Healthy)?;
        self.emit_state(id, ServiceState::Healthy);
        Ok(true)
    }

    fn snapshot_states(&self) -> HashMap<String, ServiceState> {
        self.order
            .iter()
            .filter_map(|id| self.runners.get(id).map(|r| (id.clone(), r.state())))
            .collect()
    }

    /// Start one registered service against the current supervisor state.
    ///
    /// This is the narrow service-manager operation needed by source-guided
    /// runtime adapters such as registryd: the caller asks for one service id
    /// instead of reconciling every pending service in registration order.
    pub fn start_service(&mut self, id: &str, launcher: &mut dyn ServiceLauncher) -> Result<bool> {
        let states = self.snapshot_states();
        let lookup = |name: &str| states.get(name).copied();
        let env = ConditionEnv {
            service_state: &lookup,
            configured: self.configured,
            machine_type: self.machine_type,
            network_ready: self.network_ready,
        };

        let before = self
            .runners
            .get(id)
            .map(ServiceRunner::state)
            .ok_or_else(|| MachinedError::not_found(format!("service {id}")))?;

        match before {
            ServiceState::Running | ServiceState::Healthy => return Ok(false),
            ServiceState::Finished | ServiceState::Stopping => {
                return Err(MachinedError::service_error(
                    id,
                    format!("cannot start from state {}", before.as_str()),
                ));
            }
            ServiceState::Failed if self.runners.get(id).is_none_or(|r| !r.can_restart()) => {
                return Err(MachinedError::service_error(id, "restart budget exhausted"));
            }
            _ => {}
        }

        let result = {
            let runner = self
                .runners
                .get_mut(id)
                .expect("runner exists after lookup");
            runner.start(&env, launcher)
        };
        let after = self.runners.get(id).map(ServiceRunner::state);
        if let Some(after) = after
            && after != before
        {
            self.emit_state(id, after);
        }
        result
    }

    /// Start the loaded registryd service with its runtime payload.
    pub fn start_registryd_service(
        &mut self,
        id: &str,
        launcher: &mut dyn ServiceLauncher,
    ) -> Result<bool> {
        if id != REGISTRYD_SERVICE_ID {
            return Err(MachinedError::service_error(
                id,
                format!("unsupported registryd service id {id}"),
            ));
        }

        let service = self.registryd_runtime_service.clone().ok_or_else(|| {
            MachinedError::service_error(id, "registryd runtime service payload is not loaded")
        })?;

        let states = self.snapshot_states();
        let lookup = |name: &str| states.get(name).copied();
        let env = ConditionEnv {
            service_state: &lookup,
            configured: self.configured,
            machine_type: self.machine_type,
            network_ready: self.network_ready,
        };

        let before = self
            .runners
            .get(id)
            .map(ServiceRunner::state)
            .ok_or_else(|| MachinedError::not_found(format!("service {id}")))?;

        match before {
            ServiceState::Running | ServiceState::Healthy => return Ok(false),
            ServiceState::Finished | ServiceState::Stopping => {
                return Err(MachinedError::service_error(
                    id,
                    format!("cannot start from state {}", before.as_str()),
                ));
            }
            ServiceState::Failed if self.runners.get(id).is_none_or(|r| !r.can_restart()) => {
                return Err(MachinedError::service_error(id, "restart budget exhausted"));
            }
            _ => {}
        }

        let result = {
            let runner = self
                .runners
                .get_mut(id)
                .expect("runner exists after lookup");
            runner.start_registryd_runtime_service(&env, launcher, &service)
        };
        let after = self.runners.get(id).map(ServiceRunner::state);
        if let Some(after) = after
            && after != before
        {
            self.emit_state(id, after);
        }
        result
    }

    /// Run one supervision pass: attempt to start every not-yet-up service whose
    /// dependencies are met, and restart any failed service still in budget.
    ///
    /// Returns the number of services that successfully came up this pass.
    /// Iterating [`Self::reconcile`] to a fixed point starts the whole graph in
    /// dependency order.
    pub fn reconcile(&mut self, launcher: &mut dyn ServiceLauncher) -> Result<usize> {
        let mut started = 0usize;
        let order = self.order.clone();
        for id in order {
            // Take a snapshot of all states for condition evaluation, computed
            // fresh each iteration so just-started deps are visible.
            let states = self.snapshot_states();
            let lookup = |name: &str| states.get(name).copied();
            let env = ConditionEnv {
                service_state: &lookup,
                configured: self.configured,
                machine_type: self.machine_type,
                network_ready: self.network_ready,
            };

            let before = match self.runners.get(&id) {
                Some(r) => r.state(),
                None => continue,
            };

            match before {
                ServiceState::Running
                | ServiceState::Healthy
                | ServiceState::Finished
                | ServiceState::Stopping => continue,
                // A failed service is only retried while it still has budget.
                ServiceState::Failed if self.runners.get(&id).is_none_or(|r| !r.can_restart()) => {
                    continue;
                }
                _ => {}
            }

            let result = {
                let runner = self.runners.get_mut(&id).expect("runner present");
                runner.start(&env, launcher)
            };
            let after = self.runners.get(&id).map(ServiceRunner::state);
            if let Some(after) = after
                && after != before
            {
                self.emit_state(&id, after);
            }
            if result? {
                started += 1;
            }
        }
        Ok(started)
    }

    /// Drive [`Self::reconcile`] until no further progress is made, returning
    /// the total number of services that came up.
    pub fn start_all(&mut self, launcher: &mut dyn ServiceLauncher) -> Result<usize> {
        let mut total = 0usize;
        loop {
            let n = self.reconcile(launcher)?;
            total += n;
            if n == 0 {
                break;
            }
        }
        Ok(total)
    }

    /// How many services are currently up (running or healthy).
    pub fn up_count(&self) -> usize {
        self.runners.values().filter(|r| r.state().is_up()).count()
    }

    /// How many services are healthy (passed their health check).
    pub fn healthy_count(&self) -> usize {
        self.runners
            .values()
            .filter(|r| r.state() == ServiceState::Healthy)
            .count()
    }

    /// How many services have permanently failed (failed with no restart budget).
    pub fn failed_count(&self) -> usize {
        self.runners
            .values()
            .filter(|r| r.state() == ServiceState::Failed && !r.can_restart())
            .count()
    }

    /// The ids of every service still waiting on a dependency or not yet up.
    pub fn pending(&self) -> Vec<String> {
        self.order
            .iter()
            .filter(|id| {
                self.runners.get(*id).is_some_and(|r| {
                    !matches!(
                        r.state(),
                        ServiceState::Running | ServiceState::Healthy | ServiceState::Finished
                    )
                })
            })
            .cloned()
            .collect()
    }

    /// The registered service ids in registration order.
    pub fn service_ids(&self) -> Vec<String> {
        self.order.clone()
    }

    /// Whether every registered service is up (running or healthy). True for an
    /// empty supervisor (vacuously).
    pub fn all_up(&self) -> bool {
        self.up_count() == self.runners.len()
    }

    /// Stop every service (reverse registration order, as Talos tears down in
    /// the opposite order it brought services up).
    pub fn stop_all(&mut self, launcher: &mut dyn ServiceLauncher) -> Result<()> {
        let mut order = self.order.clone();
        order.reverse();
        for id in order {
            let before = self.runners.get(&id).map(ServiceRunner::state);
            if let Some(r) = self.runners.get_mut(&id) {
                r.stop(launcher)?;
            }
            let after = self.runners.get(&id).map(ServiceRunner::state);
            if before != after
                && let Some(after) = after
            {
                self.emit_state(&id, after);
            }
        }
        Ok(())
    }
}

/// Bridge a machined [`Supervisor`] into the CRI image-cache registryd adapter.
///
/// Source Talos calls `V1Alpha1ServiceManager.IsRunning`, loads
/// `services.NewRegistryD()` when the lookup fails, then starts
/// `services.RegistryID`. This wrapper keeps that service-manager authority in
/// machined/runtime code while the COSI image-cache controller remains
/// host-safe and effect-free.
pub struct SupervisorRegistrydServiceManager<'a> {
    supervisor: &'a mut Supervisor,
    launcher: &'a mut dyn ServiceLauncher,
}

impl<'a> SupervisorRegistrydServiceManager<'a> {
    /// Build a registryd manager over the live supervisor and launch boundary.
    pub fn new(supervisor: &'a mut Supervisor, launcher: &'a mut dyn ServiceLauncher) -> Self {
        SupervisorRegistrydServiceManager {
            supervisor,
            launcher,
        }
    }

    /// Apply one source-shaped registryd health probe response.
    ///
    /// Source Talos checks `http://127.0.0.1:3172/healthz` before treating
    /// registryd as healthy. A non-2xx status leaves the service `Running`; a
    /// 2xx status promotes it to `Healthy`.
    pub fn observe_registryd_health_status(
        &mut self,
        status_code: u16,
    ) -> std::result::Result<bool, RegistrydServiceError> {
        let probe = RegistrydHealthProbe::source();
        if !probe.accepts_status(status_code) {
            return Ok(false);
        }

        self.supervisor
            .mark_service_healthy(REGISTRYD_SERVICE_ID)
            .map_err(|err| RegistrydServiceError::Health {
                service_id: REGISTRYD_SERVICE_ID.to_string(),
                message: format!(
                    "{} accepted status {status_code} but supervisor update failed: {err}",
                    probe.url()
                ),
            })
    }

    /// Project the supervisor state into the source controller's registryd flags.
    ///
    /// Talos `ImageCacheConfigController` reads the `registryd` service resource
    /// and requires both `Running` and `Healthy` before declaring the image
    /// cache ready. The supervisor keeps those as lifecycle states, so this
    /// method exposes the same two booleans without making COSI reconciliation
    /// perform service-manager effects.
    pub fn registryd_state(&self) -> RegistrydState {
        match self.supervisor.state_of(REGISTRYD_SERVICE_ID) {
            Some(ServiceState::Healthy) => RegistrydState {
                running: true,
                healthy: true,
            },
            Some(ServiceState::Running) => RegistrydState {
                running: true,
                healthy: false,
            },
            _ => RegistrydState::default(),
        }
    }
}

impl RegistrydServiceManager for SupervisorRegistrydServiceManager<'_> {
    fn is_running(&mut self, service_id: &str) -> std::result::Result<bool, RegistrydServiceError> {
        self.supervisor
            .state_of(service_id)
            .map(ServiceState::is_up)
            .ok_or_else(|| RegistrydServiceError::IsRunning {
                service_id: service_id.to_string(),
                message: "service not registered".to_string(),
            })
    }

    fn load_registryd(&mut self, service: RegistrydRuntimeService) {
        self.supervisor.registryd_runtime_service = Some(service);
        if self.supervisor.state_of(REGISTRYD_SERVICE_ID).is_none() {
            // Source `services.NewRegistryD()` has no conditions/dependencies;
            // the runner and health check are supplied by the launch boundary.
            self.supervisor
                .register(Service::new(REGISTRYD_SERVICE_ID, Vec::new()));
        }
    }

    fn start(&mut self, service_id: &str) -> std::result::Result<(), RegistrydServiceError> {
        if service_id != REGISTRYD_SERVICE_ID {
            return Err(RegistrydServiceError::Start {
                service_id: service_id.to_string(),
                message: format!("unsupported registryd service id {service_id}"),
            });
        }

        self.supervisor
            .start_registryd_service(service_id, self.launcher)
            .map(|_| ())
            .map_err(|err| RegistrydServiceError::Start {
                service_id: service_id.to_string(),
                message: err.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::ServiceCondition;
    use os_runtime_cri_domain::{
        ImageCacheConfig, ImageCacheCopyStatus, ImageCacheRuntimePlan, ImageCacheStatus,
        REGISTRYD_SERVICE_ID, RegistrydAction, RegistrydRuntimeAdapter,
        RegistrydServiceExecutionStatus, RegistrydState,
    };
    use std::{
        collections::HashSet,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    /// A launcher that succeeds (and reports healthy) for everything except an
    /// explicit failure set.
    struct FakeLauncher {
        fail: HashSet<String>,
        unhealthy: HashSet<String>,
        launched: Vec<String>,
    }

    impl FakeLauncher {
        fn new() -> Self {
            FakeLauncher {
                fail: HashSet::new(),
                unhealthy: HashSet::new(),
                launched: Vec::new(),
            }
        }
        fn failing(ids: &[&str]) -> Self {
            FakeLauncher {
                fail: ids.iter().map(ToString::to_string).collect(),
                unhealthy: HashSet::new(),
                launched: Vec::new(),
            }
        }
        fn unhealthy(ids: &[&str]) -> Self {
            FakeLauncher {
                fail: HashSet::new(),
                unhealthy: ids.iter().map(ToString::to_string).collect(),
                launched: Vec::new(),
            }
        }
    }

    impl ServiceLauncher for FakeLauncher {
        fn launch(&mut self, id: &str) -> Result<bool> {
            self.launched.push(id.to_string());
            if self.fail.contains(id) {
                return Err(MachinedError::service_error(id, "launch failed"));
            }
            Ok(!self.unhealthy.contains(id))
        }
    }

    fn cp_supervisor() -> Supervisor {
        let mut s = Supervisor::new(MachineType::ControlPlane);
        s.set_configured(true);
        s.set_network_ready(true);
        s
    }

    #[test]
    fn starts_independent_services() {
        let mut sup = cp_supervisor();
        sup.register(Service::new("etcd", vec![ServiceCondition::ConfigPresent]));
        sup.register(Service::new(
            "kubelet",
            vec![ServiceCondition::ConfigPresent],
        ));
        let mut l = FakeLauncher::new();
        let started = sup.start_all(&mut l).unwrap();
        assert_eq!(started, 2);
        assert_eq!(sup.up_count(), 2);
        assert_eq!(sup.state_of("etcd"), Some(ServiceState::Healthy));
    }

    #[test]
    fn dependency_ordering_reaches_fixed_point() {
        // apid depends on etcd being healthy; etcd depends only on config.
        let mut sup = cp_supervisor();
        sup.register(Service::new(
            "apid",
            vec![ServiceCondition::ServiceHealthy("etcd".to_string())],
        ));
        sup.register(Service::new("etcd", vec![ServiceCondition::ConfigPresent]));
        let mut l = FakeLauncher::new();
        let started = sup.start_all(&mut l).unwrap();
        assert_eq!(started, 2);
        assert_eq!(sup.state_of("apid"), Some(ServiceState::Healthy));
        // etcd must have launched before apid.
        let ei = l.launched.iter().position(|x| x == "etcd").unwrap();
        let ai = l.launched.iter().position(|x| x == "apid").unwrap();
        assert!(ei < ai, "etcd should launch before apid");
    }

    #[test]
    fn unmet_dependency_keeps_service_waiting() {
        let mut sup = Supervisor::new(MachineType::Worker);
        sup.set_configured(true);
        // ControlPlaneOnly condition is never met on a worker.
        sup.register(Service::new(
            "kube-apiserver",
            vec![ServiceCondition::ControlPlaneOnly],
        ));
        let mut l = FakeLauncher::new();
        let started = sup.start_all(&mut l).unwrap();
        assert_eq!(started, 0);
        assert_eq!(sup.state_of("kube-apiserver"), Some(ServiceState::Waiting));
    }

    #[test]
    fn launch_failure_marks_failed() {
        let mut sup = cp_supervisor();
        sup.register(Service::new("etcd", vec![]));
        let mut l = FakeLauncher::failing(&["etcd"]);
        let err = sup.reconcile(&mut l).unwrap_err();
        assert_eq!(err.kind(), "service_error");
        assert_eq!(sup.state_of("etcd"), Some(ServiceState::Failed));
    }

    #[test]
    fn stop_all_finishes_services() {
        let mut sup = cp_supervisor();
        sup.register(Service::new("etcd", vec![]));
        let mut l = FakeLauncher::new();
        sup.start_all(&mut l).unwrap();
        sup.stop_all(&mut l).unwrap();
        assert_eq!(sup.state_of("etcd"), Some(ServiceState::Finished));
        assert_eq!(sup.up_count(), 0);
    }

    #[test]
    fn emits_service_state_events_on_start() {
        let mut sup = cp_supervisor();
        sup.register(Service::new("etcd", vec![]));
        let mut l = FakeLauncher::new();
        sup.start_all(&mut l).unwrap();
        // We should have seen at least a transition into Healthy for etcd.
        let healthy = sup.events().of_type("service.state").into_iter().any(|e| {
            matches!(
                &e.kind,
                crate::events::EventKind::ServiceStateChange { service, state }
                    if service == "etcd" && *state == ServiceState::Healthy
            )
        });
        assert!(healthy, "expected a healthy event for etcd");
    }

    #[test]
    fn introspection_counts() {
        let mut sup = cp_supervisor();
        sup.register(Service::new("etcd", vec![ServiceCondition::ConfigPresent]));
        sup.register(Service::new(
            "blocked",
            vec![ServiceCondition::ServiceHealthy("missing".to_string())],
        ));
        let mut l = FakeLauncher::new();
        sup.start_all(&mut l).unwrap();
        assert_eq!(sup.healthy_count(), 1);
        assert!(!sup.all_up());
        assert_eq!(sup.pending(), vec!["blocked".to_string()]);
        assert_eq!(
            sup.service_ids(),
            vec!["etcd".to_string(), "blocked".to_string()]
        );
    }

    #[test]
    fn failed_count_tracks_exhausted_budget() {
        let mut sup = cp_supervisor();
        sup.register(Service::new("etcd", vec![]).with_max_restarts(0));
        let mut l = FakeLauncher::failing(&["etcd"]);
        let _ = sup.reconcile(&mut l);
        assert_eq!(sup.failed_count(), 1);
        assert!(!sup.all_up());
    }

    fn registryd_runtime_plan() -> ImageCacheRuntimePlan {
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

    fn test_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "operating-system-wave113-{label}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn start_service_starts_only_requested_registered_service() {
        let mut sup = cp_supervisor();
        sup.register(Service::new("other", vec![]));
        sup.register(Service::new(REGISTRYD_SERVICE_ID, vec![]));
        let mut l = FakeLauncher::new();

        let started = sup.start_service(REGISTRYD_SERVICE_ID, &mut l).unwrap();

        assert!(started);
        assert_eq!(l.launched, vec![REGISTRYD_SERVICE_ID.to_string()]);
        assert_eq!(
            sup.state_of(REGISTRYD_SERVICE_ID),
            Some(ServiceState::Healthy)
        );
        assert_eq!(sup.state_of("other"), Some(ServiceState::Initialized));
    }

    #[test]
    fn registryd_service_manager_loads_and_starts_supervisor_service() {
        let mut sup = cp_supervisor();
        let mut l = FakeLauncher::new();
        let plan = registryd_runtime_plan();

        let report = {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .unwrap()
        };

        assert_eq!(
            report.status,
            RegistrydServiceExecutionStatus::LoadedAndStarted
        );
        assert!(report.loaded);
        assert!(report.started);
        assert_eq!(sup.service_ids(), vec![REGISTRYD_SERVICE_ID.to_string()]);
        assert_eq!(
            sup.state_of(REGISTRYD_SERVICE_ID),
            Some(ServiceState::Healthy)
        );
        assert_eq!(l.launched, vec![REGISTRYD_SERVICE_ID.to_string()]);
    }

    #[test]
    fn registryd_service_manager_launches_with_loaded_runtime_payload() {
        struct RuntimePayloadLauncher {
            launches: Vec<String>,
            runtime_roots: Vec<PathBuf>,
        }

        impl RuntimePayloadLauncher {
            fn new() -> Self {
                Self {
                    launches: Vec::new(),
                    runtime_roots: Vec::new(),
                }
            }
        }

        impl ServiceLauncher for RuntimePayloadLauncher {
            fn launch(&mut self, id: &str) -> Result<bool> {
                self.launches.push(format!("generic:{id}"));
                Ok(true)
            }

            fn launch_registryd_runtime_service(
                &mut self,
                id: &str,
                service: &os_runtime_cri_domain::RegistrydRuntimeService,
            ) -> Result<bool> {
                self.launches.push(format!("registryd:{id}"));
                self.runtime_roots = service.roots().roots().to_vec();
                Ok(true)
            }
        }

        let temp = test_temp_dir("registryd-runtime-payload-launcher");
        let root = temp.join("root");
        fs::create_dir_all(&root).unwrap();
        let mut sup = cp_supervisor();
        let mut launcher = RuntimePayloadLauncher::new();
        let mut plan = registryd_runtime_plan();
        plan.config.roots = vec![root.display().to_string()];

        let report = {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut launcher);
            RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .unwrap()
        };

        assert_eq!(
            report.status,
            RegistrydServiceExecutionStatus::LoadedAndStarted
        );
        assert_eq!(
            launcher.launches,
            vec![format!("registryd:{REGISTRYD_SERVICE_ID}")]
        );
        assert_eq!(launcher.runtime_roots, vec![root.clone()]);
        assert_eq!(
            sup.state_of(REGISTRYD_SERVICE_ID),
            Some(ServiceState::Healthy)
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn registryd_service_manager_stores_loaded_runtime_service_roots() {
        let temp = test_temp_dir("registryd-service-manager-roots");
        let missing_root = temp.join("missing-root");
        let root = temp.join("root");
        fs::create_dir_all(&root).unwrap();
        let mut sup = cp_supervisor();
        let mut l = FakeLauncher::new();
        let mut plan = registryd_runtime_plan();
        plan.config.roots = vec![
            missing_root.display().to_string(),
            root.display().to_string(),
        ];

        {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .unwrap();
        }

        let service = sup
            .registryd_runtime_service()
            .expect("registryd runtime service loaded");
        assert_eq!(service.roots().roots(), std::slice::from_ref(&root));
        assert_eq!(service.skipped_roots()[0].root, missing_root);
        assert_eq!(
            service.skipped_roots()[0].error_kind,
            std::io::ErrorKind::NotFound
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn registryd_service_manager_serves_loaded_runtime_service_request() {
        let temp = test_temp_dir("registryd-service-manager-request");
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "4".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "4".repeat(64)));
        let blob = b"supervisor runtime-service blob bytes";
        fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        fs::write(&blob_path, blob).unwrap();

        let mut sup = cp_supervisor();
        let mut l = FakeLauncher::new();
        let mut plan = registryd_runtime_plan();
        plan.config.roots = vec![root.display().to_string()];

        {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .unwrap();
        }

        let response = sup
            .handle_registryd_request(
                "GET",
                &format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io"),
            )
            .expect("loaded registryd runtime service handles blob route");

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, blob);
        assert_eq!(response.content_path, Some(blob_path));
        assert_eq!(
            response.docker_content_digest.as_deref(),
            Some(blob_digest.as_str())
        );

        let unloaded = cp_supervisor();
        assert!(
            unloaded
                .handle_registryd_request("GET", "/healthz")
                .is_none()
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn registryd_service_manager_start_can_leave_registryd_running_until_health_check() {
        let mut sup = cp_supervisor();
        let mut l = FakeLauncher::unhealthy(&[REGISTRYD_SERVICE_ID]);
        let plan = registryd_runtime_plan();

        let report = {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .unwrap()
        };

        assert_eq!(
            report.status,
            RegistrydServiceExecutionStatus::LoadedAndStarted
        );
        assert!(report.loaded);
        assert!(report.started);
        assert_eq!(sup.service_ids(), vec![REGISTRYD_SERVICE_ID.to_string()]);
        assert_eq!(
            sup.state_of(REGISTRYD_SERVICE_ID),
            Some(ServiceState::Running)
        );
        assert_eq!(l.launched, vec![REGISTRYD_SERVICE_ID.to_string()]);
    }

    #[test]
    fn registryd_service_manager_marks_running_registryd_healthy_from_source_probe_status() {
        let mut sup = cp_supervisor();
        let mut l = FakeLauncher::unhealthy(&[REGISTRYD_SERVICE_ID]);
        let plan = registryd_runtime_plan();

        {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .unwrap();
        }
        assert_eq!(
            sup.state_of(REGISTRYD_SERVICE_ID),
            Some(ServiceState::Running)
        );

        let promoted = {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            manager.observe_registryd_health_status(204).unwrap()
        };

        assert!(promoted);
        assert_eq!(
            sup.state_of(REGISTRYD_SERVICE_ID),
            Some(ServiceState::Healthy)
        );
    }

    #[test]
    fn registryd_service_manager_rejects_non_success_probe_without_marking_healthy() {
        let mut sup = cp_supervisor();
        let mut l = FakeLauncher::unhealthy(&[REGISTRYD_SERVICE_ID]);
        let plan = registryd_runtime_plan();

        {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .unwrap();
        }

        let promoted = {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            manager.observe_registryd_health_status(404).unwrap()
        };

        assert!(!promoted);
        assert_eq!(
            sup.state_of(REGISTRYD_SERVICE_ID),
            Some(ServiceState::Running)
        );
    }

    #[test]
    fn registryd_service_manager_projects_source_running_healthy_flags_from_supervisor_state() {
        let mut sup = cp_supervisor();
        let mut l = FakeLauncher::unhealthy(&[REGISTRYD_SERVICE_ID]);
        let plan = registryd_runtime_plan();

        {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            assert_eq!(manager.registryd_state(), RegistrydState::default());

            RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .unwrap();
            assert_eq!(
                manager.registryd_state(),
                RegistrydState {
                    running: true,
                    healthy: false,
                }
            );

            assert!(manager.observe_registryd_health_status(204).unwrap());
            assert_eq!(
                manager.registryd_state(),
                RegistrydState {
                    running: true,
                    healthy: true,
                }
            );
        }
    }

    #[test]
    fn registryd_service_manager_does_not_duplicate_running_registryd() {
        let mut sup = cp_supervisor();
        sup.register(Service::new(REGISTRYD_SERVICE_ID, vec![]));
        let mut l = FakeLauncher::new();
        sup.start_all(&mut l).unwrap();
        l.launched.clear();
        let plan = registryd_runtime_plan();

        let report = {
            let mut manager = SupervisorRegistrydServiceManager::new(&mut sup, &mut l);
            RegistrydRuntimeAdapter
                .execute(&plan, &mut manager)
                .unwrap()
        };

        assert_eq!(
            report.status,
            RegistrydServiceExecutionStatus::AlreadyRunning
        );
        assert!(!report.loaded);
        assert!(!report.started);
        assert!(l.launched.is_empty());
        assert_eq!(sup.service_ids(), vec![REGISTRYD_SERVICE_ID.to_string()]);
    }

    #[test]
    fn stop_all_tears_down_in_reverse_order() {
        let mut sup = cp_supervisor();
        sup.register(Service::new("etcd", vec![]));
        sup.register(Service::new("apid", vec![]));
        let mut l = FakeLauncher::new();
        sup.start_all(&mut l).unwrap();
        let before = sup.events().next_id();
        sup.stop_all(&mut l).unwrap();
        // Collect finished-service events emitted after stop began, in order.
        let order: Vec<String> = sup
            .events()
            .since(before - 1)
            .into_iter()
            .filter_map(|e| match &e.kind {
                crate::events::EventKind::ServiceStateChange { service, state }
                    if *state == ServiceState::Finished =>
                {
                    Some(service.clone())
                }
                _ => None,
            })
            .collect();
        assert_eq!(order, vec!["apid".to_string(), "etcd".to_string()]);
    }
}
