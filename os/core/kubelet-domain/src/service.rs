//! Kubelet service lifecycle and CSR-approval interaction.
//!
//! Mirrors Talos's kubelet `system.Service` plus the bootstrap flow: the kubelet
//! starts with a bootstrap kubeconfig, issues a node CSR, and once that CSR is
//! approved (by the Talos CSR-approver controller or kube-controller-manager) it
//! writes its client cert and transitions to fully running. We model the OS
//! boundaries (process supervision and the CSR API) as traits with in-memory
//! implementations the tests drive.

use os_kernel::error::{Error, Result};
use os_kernel::traits::{RunState, Runnable};

use crate::spec::KubeletSpec;

/// State of a node's client-certificate signing request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsrState {
    /// No CSR has been issued yet.
    None,
    /// CSR submitted, awaiting approval.
    Pending,
    /// CSR approved and certificate issued.
    Approved,
    /// CSR denied.
    Denied,
}

/// Abstracts the kubernetes CSR API the kubelet bootstrap interacts with.
///
/// In production this is the apiserver; here it is a trait so the lifecycle can
/// be tested deterministically.
pub trait CsrApprover {
    /// Submit a node CSR for the given node name and return the CSR id.
    fn submit(&mut self, node_name: &str) -> Result<String>;
    /// The current state of a previously-submitted CSR.
    fn state(&self, csr_id: &str) -> CsrState;
}

/// Abstracts process supervision (the Talos service runner / containerd shim).
pub trait ProcessSupervisor {
    /// Launch the process described by the spec, returning a pid-like handle.
    fn launch(&mut self, spec: &KubeletSpec) -> Result<u32>;
    /// Terminate a running process.
    fn terminate(&mut self, pid: u32) -> Result<()>;
    /// Whether the process is currently alive.
    fn is_alive(&self, pid: u32) -> bool;
}

/// An in-memory CSR approver. By default it auto-approves; set
/// [`auto_approve`](Self::auto_approve) to `false` to leave CSRs pending and
/// drive approval manually with [`approve`](Self::approve).
#[derive(Debug, Default)]
pub struct InMemoryCsrApprover {
    /// Whether to immediately approve submitted CSRs.
    pub auto_approve: bool,
    csrs: std::collections::BTreeMap<String, CsrState>,
    next_id: u32,
}

impl InMemoryCsrApprover {
    /// A new approver with the given auto-approve policy.
    pub fn new(auto_approve: bool) -> Self {
        InMemoryCsrApprover {
            auto_approve,
            csrs: Default::default(),
            next_id: 0,
        }
    }

    /// Approve a pending CSR by id.
    pub fn approve(&mut self, csr_id: &str) -> Result<()> {
        match self.csrs.get_mut(csr_id) {
            Some(s @ CsrState::Pending) => {
                *s = CsrState::Approved;
                Ok(())
            }
            Some(_) => Err(Error::invalid_state("CSR not pending")),
            None => Err(Error::not_found("CSR not found")),
        }
    }

    /// Deny a pending CSR by id.
    pub fn deny(&mut self, csr_id: &str) -> Result<()> {
        match self.csrs.get_mut(csr_id) {
            Some(s @ CsrState::Pending) => {
                *s = CsrState::Denied;
                Ok(())
            }
            Some(_) => Err(Error::invalid_state("CSR not pending")),
            None => Err(Error::not_found("CSR not found")),
        }
    }
}

impl CsrApprover for InMemoryCsrApprover {
    fn submit(&mut self, node_name: &str) -> Result<String> {
        if node_name.is_empty() {
            return Err(Error::invalid("CSR requires a node name"));
        }
        let id = format!("csr-{}-{}", node_name, self.next_id);
        self.next_id += 1;
        let state = if self.auto_approve {
            CsrState::Approved
        } else {
            CsrState::Pending
        };
        self.csrs.insert(id.clone(), state);
        Ok(id)
    }

    fn state(&self, csr_id: &str) -> CsrState {
        self.csrs.get(csr_id).copied().unwrap_or(CsrState::None)
    }
}

/// An in-memory process supervisor that tracks live pids.
#[derive(Debug, Default)]
pub struct InMemorySupervisor {
    next_pid: u32,
    alive: std::collections::BTreeSet<u32>,
}

impl ProcessSupervisor for InMemorySupervisor {
    fn launch(&mut self, spec: &KubeletSpec) -> Result<u32> {
        if spec.command.is_empty() {
            return Err(Error::invalid("kubelet spec has no command"));
        }
        self.next_pid += 1;
        let pid = self.next_pid;
        self.alive.insert(pid);
        Ok(pid)
    }

    fn terminate(&mut self, pid: u32) -> Result<()> {
        if self.alive.remove(&pid) {
            Ok(())
        } else {
            Err(Error::not_found("no such process"))
        }
    }

    fn is_alive(&self, pid: u32) -> bool {
        self.alive.contains(&pid)
    }
}

/// The phase the kubelet bootstrap is in. Sits alongside the generic
/// [`RunState`] from talos-core: `RunState` tracks the supervised process,
/// `BootstrapPhase` tracks the certificate handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapPhase {
    /// Not yet started.
    Idle,
    /// Process launched with the bootstrap kubeconfig, CSR submitted.
    AwaitingApproval,
    /// CSR approved; serving with the issued client cert.
    Bootstrapped,
    /// CSR denied; bootstrap failed.
    Failed,
}

/// The kubelet service: owns the rendered spec, the supervisor, and the CSR
/// approver, and drives the bootstrap + run lifecycle.
pub struct KubeletService<S: ProcessSupervisor, C: CsrApprover> {
    spec: KubeletSpec,
    supervisor: S,
    approver: C,
    state: RunState,
    phase: BootstrapPhase,
    pid: Option<u32>,
    csr_id: Option<String>,
}

impl<S: ProcessSupervisor, C: CsrApprover> KubeletService<S, C> {
    /// Construct a service in the `Initialized` / `Idle` state.
    pub fn new(spec: KubeletSpec, supervisor: S, approver: C) -> Self {
        KubeletService {
            spec,
            supervisor,
            approver,
            state: RunState::Initialized,
            phase: BootstrapPhase::Idle,
            pid: None,
            csr_id: None,
        }
    }

    /// Current bootstrap phase.
    pub fn phase(&self) -> BootstrapPhase {
        self.phase
    }

    /// The CSR id, once submitted.
    pub fn csr_id(&self) -> Option<&str> {
        self.csr_id.as_deref()
    }

    /// The supervised pid, once launched.
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// Mutable access to the CSR approver (used to drive approval/denial).
    pub fn approver_mut(&mut self) -> &mut C {
        &mut self.approver
    }

    /// Mutable access to the process supervisor.
    pub fn supervisor_mut(&mut self) -> &mut S {
        &mut self.supervisor
    }

    /// Begin the bootstrap: launch the process and submit the node CSR.
    ///
    /// Transitions `Initialized -> Preparing` and `Idle -> AwaitingApproval`.
    pub fn bootstrap(&mut self) -> Result<()> {
        if self.state != RunState::Initialized {
            return Err(Error::invalid_state("kubelet already bootstrapped"));
        }
        let pid = self.supervisor.launch(&self.spec)?;
        let csr_id = self.approver.submit(&self.spec.node_name)?;
        self.pid = Some(pid);
        self.csr_id = Some(csr_id);
        self.state = RunState::Preparing;
        self.phase = BootstrapPhase::AwaitingApproval;
        self.reconcile_bootstrap();
        Ok(())
    }

    /// Re-check the CSR state and advance the phase if it has been decided.
    ///
    /// Mirrors the controller reconcile loop polling the CSR: once approved the
    /// kubelet transitions to `Running`; if denied, to `Failed`.
    pub fn reconcile_bootstrap(&mut self) -> BootstrapPhase {
        if self.phase != BootstrapPhase::AwaitingApproval {
            return self.phase;
        }
        let Some(csr_id) = &self.csr_id else {
            return self.phase;
        };
        match self.approver.state(csr_id) {
            CsrState::Approved => {
                self.phase = BootstrapPhase::Bootstrapped;
                self.state = RunState::Running;
            }
            CsrState::Denied => {
                self.phase = BootstrapPhase::Failed;
                self.state = RunState::Failed;
            }
            _ => {}
        }
        self.phase
    }

    /// Whether the kubelet has finished bootstrapping and is serving.
    pub fn is_ready(&self) -> bool {
        self.phase == BootstrapPhase::Bootstrapped && self.state == RunState::Running
    }
}

impl<S: ProcessSupervisor, C: CsrApprover> Runnable for KubeletService<S, C> {
    fn id(&self) -> &str {
        "kubelet"
    }

    fn start(&mut self) -> Result<()> {
        if self.state == RunState::Initialized {
            self.bootstrap()?;
        }
        self.reconcile_bootstrap();
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(pid) = self.pid.take() {
            // Ignore "no such process" so stop is idempotent.
            let _ = self.supervisor.terminate(pid);
        }
        self.state = RunState::Stopped;
        self.phase = BootstrapPhase::Idle;
        Ok(())
    }

    fn state(&self) -> RunState {
        self.state
    }

    fn is_healthy(&self) -> bool {
        self.is_ready()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::address::NodeAddress;
    use os_kernel::traits::Runnable;

    use crate::config::KubeletConfig;
    use crate::nodename::Nodename;

    fn make_spec() -> KubeletSpec {
        let cfg = KubeletConfig::with_dns_from_service_cidr("10.96.0.0/12").unwrap();
        KubeletSpec::render(
            &cfg,
            &Nodename::new("worker-1").unwrap(),
            &[NodeAddress::parse_v4("10.0.0.5").unwrap()],
            &[],
        )
        .unwrap()
    }

    #[test]
    fn csr_auto_approve_flow() {
        let svc_spec = make_spec();
        let mut svc = KubeletService::new(
            svc_spec,
            InMemorySupervisor::default(),
            InMemoryCsrApprover::new(true),
        );
        assert_eq!(svc.phase(), BootstrapPhase::Idle);
        svc.bootstrap().unwrap();
        assert!(svc.is_ready());
        assert_eq!(svc.state(), RunState::Running);
        assert!(svc.pid().is_some());
        assert!(svc.is_healthy());
    }

    #[test]
    fn csr_manual_approval_advances_phase() {
        // Manual approval mode: a submitted CSR stays pending until approved.
        let approver = InMemoryCsrApprover::new(false);
        let mut svc = KubeletService::new(make_spec(), InMemorySupervisor::default(), approver);
        svc.bootstrap().unwrap();
        assert_eq!(svc.phase(), BootstrapPhase::AwaitingApproval);
        assert!(!svc.is_ready());

        let csr = svc.csr_id().unwrap().to_string();
        svc.approver.approve(&csr).unwrap();
        assert_eq!(svc.reconcile_bootstrap(), BootstrapPhase::Bootstrapped);
        assert!(svc.is_ready());
    }

    #[test]
    fn csr_denial_fails_bootstrap() {
        let mut svc = KubeletService::new(
            make_spec(),
            InMemorySupervisor::default(),
            InMemoryCsrApprover::new(false),
        );
        svc.bootstrap().unwrap();
        let csr = svc.csr_id().unwrap().to_string();
        svc.approver.deny(&csr).unwrap();
        assert_eq!(svc.reconcile_bootstrap(), BootstrapPhase::Failed);
        assert_eq!(svc.state(), RunState::Failed);
        assert!(!svc.is_healthy());
    }

    #[test]
    fn double_bootstrap_rejected() {
        let mut svc = KubeletService::new(
            make_spec(),
            InMemorySupervisor::default(),
            InMemoryCsrApprover::new(true),
        );
        svc.bootstrap().unwrap();
        assert_eq!(svc.bootstrap().unwrap_err().kind(), "invalid_state");
    }

    #[test]
    fn stop_terminates_process_and_is_idempotent() {
        let mut svc = KubeletService::new(
            make_spec(),
            InMemorySupervisor::default(),
            InMemoryCsrApprover::new(true),
        );
        svc.start().unwrap();
        let pid = svc.pid().unwrap();
        assert!(svc.supervisor.is_alive(pid));
        svc.stop().unwrap();
        assert!(!svc.supervisor.is_alive(pid));
        assert_eq!(svc.state(), RunState::Stopped);
        // Second stop must not error.
        svc.stop().unwrap();
    }

    #[test]
    fn supervisor_rejects_empty_command() {
        let mut sup = InMemorySupervisor::default();
        let mut spec = make_spec();
        spec.command.clear();
        assert!(sup.launch(&spec).is_err());
    }

    #[test]
    fn approver_state_unknown_is_none() {
        let approver = InMemoryCsrApprover::new(true);
        assert_eq!(approver.state("nope"), CsrState::None);
    }
}
