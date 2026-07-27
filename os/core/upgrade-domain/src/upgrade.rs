//! The OS upgrade sequence and `Upgrade` API flow.
//!
//! This is the top-level state machine that the `machined` `Upgrade` gRPC
//! method drives. The Talos upgrade sequence (`SequenceUpgrade` in
//! `v1alpha1_sequencer.go`) is roughly:
//!
//! 1. **Validate** the request (image ref, target version reachable).
//! 2. **Cordon & drain** the node (for control-plane/worker participation).
//! 3. **Etcd leave** — a control-plane node leaves the etcd membership before
//!    being taken down so quorum is preserved (the "leave before upgrade"
//!    behavior).
//! 4. **Stop services / unmount** the ephemeral state.
//! 5. **Install** the new system (immediately, or staged via META).
//! 6. **Reboot / kexec** into the new system.
//!
//! Each step is represented by [`UpgradeStep`]; the controller advances through
//! them, short-circuiting steps that don't apply (e.g. etcd-leave on a worker).
//! All OS boundaries are modeled by traits so the whole flow is unit-testable.

use crate::staged::validate_image_ref;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use os_kernel::Version;

/// The node's role in the cluster, which gates which upgrade steps run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    /// A control-plane node that is also an etcd member.
    ControlPlane,
    /// A plain worker node (no etcd, drain only).
    Worker,
    /// A single-node cluster (control-plane that must NOT etcd-leave, since it
    /// is the only member).
    SingleNode,
}

impl NodeRole {
    /// Whether this role participates in etcd membership.
    pub fn is_etcd_member(self) -> bool {
        matches!(self, NodeRole::ControlPlane)
    }
}

/// The discrete steps of the upgrade sequence, in execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpgradeStep {
    /// Validate the request.
    Validate,
    /// Cordon + drain the node.
    Drain,
    /// Leave etcd membership (control-plane only).
    EtcdLeave,
    /// Stop services and unmount ephemeral state.
    StopServices,
    /// Run the installer (write the new system).
    Install,
    /// Reboot / kexec into the new system.
    Reboot,
}

impl UpgradeStep {
    /// Stable name for logging.
    pub fn name(self) -> &'static str {
        match self {
            UpgradeStep::Validate => "validate",
            UpgradeStep::Drain => "drain",
            UpgradeStep::EtcdLeave => "etcd-leave",
            UpgradeStep::StopServices => "stop-services",
            UpgradeStep::Install => "install",
            UpgradeStep::Reboot => "reboot",
        }
    }
}

/// Coarse phase reported by the controller (maps the sequence onto the Talos
/// `MachineStatus` upgrade phases).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradePhase {
    /// Not yet started.
    Idle,
    /// Preflight (validate / drain / etcd-leave).
    Preflight,
    /// Installing the new system.
    Installing,
    /// About to reboot into the new system (terminal-success).
    Rebooting,
    /// The upgrade failed; the node remains on the current version.
    Failed,
}

/// Errors raised by the upgrade sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeError {
    /// The installer image reference was invalid.
    InvalidImageRef(String),
    /// The target version is not a permitted upgrade from the current version.
    UnsupportedTransition { from: Version, to: Version },
    /// Draining the node failed.
    DrainFailed(String),
    /// Leaving etcd failed.
    EtcdLeaveFailed(String),
    /// The installer failed.
    InstallFailed(String),
    /// A step was attempted out of order.
    OutOfOrder(UpgradeStep),
}

impl fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpgradeError::InvalidImageRef(m) => write!(f, "invalid installer image: {m}"),
            UpgradeError::UnsupportedTransition { from, to } => {
                write!(f, "upgrade from {from} to {to} is not permitted")
            }
            UpgradeError::DrainFailed(m) => write!(f, "drain failed: {m}"),
            UpgradeError::EtcdLeaveFailed(m) => write!(f, "etcd leave failed: {m}"),
            UpgradeError::InstallFailed(m) => write!(f, "install failed: {m}"),
            UpgradeError::OutOfOrder(s) => write!(f, "step {} attempted out of order", s.name()),
        }
    }
}

/// A validated upgrade request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeRequest {
    /// Current OS version.
    pub current: Version,
    /// Installer image reference.
    pub image_ref: String,
    /// Target OS version.
    pub target: Version,
    /// The node's role, gating etcd-leave/drain.
    pub role: NodeRole,
    /// Whether to perform a staged (META) install instead of immediate.
    pub staged: bool,
    /// Whether to preserve ephemeral data across the upgrade.
    pub preserve: bool,
}

impl UpgradeRequest {
    /// Build and validate a request (control-plane, immediate, no preserve by
    /// default). Validates the image ref and the version transition.
    pub fn new(current: Version, image_ref: &str, target: Version) -> Result<Self, UpgradeError> {
        validate_image_ref(image_ref).map_err(|e| UpgradeError::InvalidImageRef(e.to_string()))?;
        if !current.is_upgrade_allowed_to(&target) {
            return Err(UpgradeError::UnsupportedTransition {
                from: current,
                to: target,
            });
        }
        Ok(UpgradeRequest {
            current,
            image_ref: image_ref.to_string(),
            target,
            role: NodeRole::ControlPlane,
            staged: false,
            preserve: false,
        })
    }

    /// Set the node role.
    pub fn with_role(mut self, role: NodeRole) -> Self {
        self.role = role;
        self
    }

    /// Mark the upgrade as staged.
    pub fn staged(mut self) -> Self {
        self.staged = true;
        self
    }

    /// Preserve ephemeral data.
    pub fn preserve(mut self) -> Self {
        self.preserve = true;
        self
    }

    /// The ordered steps this request will execute, with role-gated steps
    /// dropped (workers and single-node clusters skip etcd-leave).
    pub fn plan(&self) -> Vec<UpgradeStep> {
        let mut steps = Vec::new();
        steps.push(UpgradeStep::Validate);
        if self.role != NodeRole::SingleNode {
            steps.push(UpgradeStep::Drain);
        }
        if self.role.is_etcd_member() {
            steps.push(UpgradeStep::EtcdLeave);
        }
        steps.push(UpgradeStep::StopServices);
        steps.push(UpgradeStep::Install);
        steps.push(UpgradeStep::Reboot);
        steps
    }
}

/// The etcd membership control boundary used to "leave before upgrade".
pub trait EtcdControl {
    /// The number of voting members currently in the cluster.
    fn member_count(&self) -> usize;

    /// Whether this node is currently a member.
    fn is_member(&self) -> bool;

    /// Leave the etcd cluster. Errors if doing so would lose quorum.
    fn leave(&mut self) -> Result<(), String>;
}

/// In-memory etcd membership for tests.
#[derive(Debug, Clone)]
pub struct InMemoryEtcd {
    members: usize,
    is_member: bool,
    /// If true, leaving is refused (models an unhealthy cluster).
    block_leave: bool,
}

impl InMemoryEtcd {
    /// A cluster with `members` members where this node is a member.
    pub fn new(members: usize) -> Self {
        InMemoryEtcd {
            members,
            is_member: true,
            block_leave: false,
        }
    }

    /// A node that is not part of etcd (worker).
    pub fn non_member() -> Self {
        InMemoryEtcd {
            members: 0,
            is_member: false,
            block_leave: false,
        }
    }

    /// Force `leave` to fail.
    pub fn block_leave(mut self) -> Self {
        self.block_leave = true;
        self
    }
}

impl EtcdControl for InMemoryEtcd {
    fn member_count(&self) -> usize {
        self.members
    }

    fn is_member(&self) -> bool {
        self.is_member
    }

    fn leave(&mut self) -> Result<(), String> {
        if !self.is_member {
            return Err("node is not an etcd member".to_string());
        }
        if self.block_leave {
            return Err("etcd cluster is unhealthy".to_string());
        }
        // Leaving a 2-member cluster would drop to 1 and risk quorum loss; a
        // single-member cluster cannot leave at all.
        if self.members <= 1 {
            return Err("cannot leave: would lose quorum".to_string());
        }
        self.members -= 1;
        self.is_member = false;
        Ok(())
    }
}

/// Drives the upgrade sequence step by step.
#[derive(Debug)]
pub struct UpgradeController {
    request: UpgradeRequest,
    phase: UpgradePhase,
    completed: Vec<UpgradeStep>,
    plan: Vec<UpgradeStep>,
    cursor: usize,
}

impl UpgradeController {
    /// Build a controller for a validated request.
    pub fn new(request: UpgradeRequest) -> Self {
        let plan = request.plan();
        UpgradeController {
            request,
            phase: UpgradePhase::Idle,
            completed: Vec::new(),
            plan,
            cursor: 0,
        }
    }

    /// The current coarse phase.
    pub fn phase(&self) -> UpgradePhase {
        self.phase
    }

    /// The steps completed so far, in order.
    pub fn completed_steps(&self) -> &[UpgradeStep] {
        &self.completed
    }

    /// The full planned step list.
    pub fn plan(&self) -> &[UpgradeStep] {
        &self.plan
    }

    /// The request being executed.
    pub fn request(&self) -> &UpgradeRequest {
        &self.request
    }

    /// The next step to run, if any remain.
    pub fn next_step(&self) -> Option<UpgradeStep> {
        self.plan.get(self.cursor).copied()
    }

    fn set_phase_for(&mut self, step: UpgradeStep) {
        self.phase = match step {
            UpgradeStep::Validate | UpgradeStep::Drain | UpgradeStep::EtcdLeave => {
                UpgradePhase::Preflight
            }
            UpgradeStep::StopServices | UpgradeStep::Install => UpgradePhase::Installing,
            UpgradeStep::Reboot => UpgradePhase::Rebooting,
        };
    }

    /// Run one step against the etcd control, advancing the cursor. Returns the
    /// step that ran. On error the phase becomes [`UpgradePhase::Failed`].
    pub fn step<E: EtcdControl>(&mut self, etcd: &mut E) -> Result<UpgradeStep, UpgradeError> {
        let step = match self.next_step() {
            Some(s) => s,
            None => return Err(UpgradeError::OutOfOrder(UpgradeStep::Reboot)),
        };
        self.set_phase_for(step);

        let result = match step {
            UpgradeStep::Validate => self.do_validate(),
            UpgradeStep::Drain => Ok(()), // drain handled by DrainController; modeled as a no-op success here
            UpgradeStep::EtcdLeave => self.do_etcd_leave(etcd),
            UpgradeStep::StopServices => Ok(()),
            UpgradeStep::Install => self.do_install(),
            UpgradeStep::Reboot => Ok(()),
        };

        match result {
            Ok(()) => {
                self.completed.push(step);
                self.cursor += 1;
                Ok(step)
            }
            Err(e) => {
                self.phase = UpgradePhase::Failed;
                Err(e)
            }
        }
    }

    fn do_validate(&self) -> Result<(), UpgradeError> {
        validate_image_ref(&self.request.image_ref)
            .map_err(|e| UpgradeError::InvalidImageRef(e.to_string()))?;
        if !self
            .request
            .current
            .is_upgrade_allowed_to(&self.request.target)
        {
            return Err(UpgradeError::UnsupportedTransition {
                from: self.request.current.clone(),
                to: self.request.target.clone(),
            });
        }
        Ok(())
    }

    fn do_etcd_leave<E: EtcdControl>(&self, etcd: &mut E) -> Result<(), UpgradeError> {
        if !etcd.is_member() {
            // Not a member -> nothing to do (defensive; planner usually gates).
            return Ok(());
        }
        etcd.leave().map_err(UpgradeError::EtcdLeaveFailed)
    }

    fn do_install(&self) -> Result<(), UpgradeError> {
        // The actual install is delegated to the installer image / staged flow;
        // here we model the success path. A real failure would surface from the
        // installer boundary.
        Ok(())
    }

    /// Run the whole sequence to completion (or first failure). Uses a default
    /// healthy etcd derived from the role.
    pub fn run_to_completion(&mut self) -> Result<UpgradePhase, UpgradeError> {
        let mut etcd = match self.request.role {
            NodeRole::ControlPlane => InMemoryEtcd::new(3),
            _ => InMemoryEtcd::non_member(),
        };
        self.run_with_etcd(&mut etcd)
    }

    /// Run the whole sequence against a provided etcd control.
    pub fn run_with_etcd<E: EtcdControl>(
        &mut self,
        etcd: &mut E,
    ) -> Result<UpgradePhase, UpgradeError> {
        while self.next_step().is_some() {
            self.step(etcd)?;
        }
        // Sanity: every planned step ran exactly once.
        let unique: BTreeSet<_> = self.completed.iter().copied().collect();
        debug_assert_eq!(unique.len(), self.completed.len());
        Ok(self.phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMAGE: &str = "ghcr.io/siderolabs/installer:v1.8.0";

    fn req() -> UpgradeRequest {
        UpgradeRequest::new(Version::new(1, 7, 0), IMAGE, Version::new(1, 8, 0)).unwrap()
    }

    #[test]
    fn request_validation() {
        assert!(
            UpgradeRequest::new(Version::new(1, 7, 0), "bad ref", Version::new(1, 8, 0)).is_err()
        );
        assert!(matches!(
            UpgradeRequest::new(Version::new(1, 7, 0), IMAGE, Version::new(1, 9, 0)),
            Err(UpgradeError::UnsupportedTransition { .. })
        ));
        assert!(req().current < req().target);
    }

    #[test]
    fn control_plane_plan_includes_etcd_leave() {
        let plan = req().plan();
        assert_eq!(
            plan,
            alloc::vec![
                UpgradeStep::Validate,
                UpgradeStep::Drain,
                UpgradeStep::EtcdLeave,
                UpgradeStep::StopServices,
                UpgradeStep::Install,
                UpgradeStep::Reboot,
            ]
        );
    }

    #[test]
    fn worker_plan_skips_etcd_leave() {
        let plan = req().with_role(NodeRole::Worker).plan();
        assert!(!plan.contains(&UpgradeStep::EtcdLeave));
        assert!(plan.contains(&UpgradeStep::Drain));
    }

    #[test]
    fn single_node_plan_skips_drain_and_etcd() {
        let plan = req().with_role(NodeRole::SingleNode).plan();
        assert!(!plan.contains(&UpgradeStep::EtcdLeave));
        assert!(!plan.contains(&UpgradeStep::Drain));
        assert!(plan.contains(&UpgradeStep::Install));
    }

    #[test]
    fn full_control_plane_sequence_reaches_reboot() {
        let mut ctrl = UpgradeController::new(req());
        assert_eq!(ctrl.phase(), UpgradePhase::Idle);
        let phase = ctrl.run_to_completion().unwrap();
        assert_eq!(phase, UpgradePhase::Rebooting);
        assert_eq!(ctrl.completed_steps().len(), 6);
        assert_eq!(ctrl.completed_steps().last(), Some(&UpgradeStep::Reboot));
    }

    #[test]
    fn etcd_leave_runs_and_decrements_membership() {
        let mut ctrl = UpgradeController::new(req());
        let mut etcd = InMemoryEtcd::new(3);
        ctrl.run_with_etcd(&mut etcd).unwrap();
        assert_eq!(etcd.member_count(), 2);
        assert!(!etcd.is_member());
        assert!(ctrl.completed_steps().contains(&UpgradeStep::EtcdLeave));
    }

    #[test]
    fn etcd_leave_blocked_fails_upgrade_at_preflight() {
        let mut ctrl = UpgradeController::new(req());
        let mut etcd = InMemoryEtcd::new(3).block_leave();
        let err = ctrl.run_with_etcd(&mut etcd).unwrap_err();
        assert!(matches!(err, UpgradeError::EtcdLeaveFailed(_)));
        assert_eq!(ctrl.phase(), UpgradePhase::Failed);
        // Install never ran.
        assert!(!ctrl.completed_steps().contains(&UpgradeStep::Install));
    }

    #[test]
    fn quorum_protection_blocks_leave_from_single_member_cluster() {
        // A lone etcd member cannot leave: doing so would destroy the cluster.
        let mut ctrl = UpgradeController::new(req());
        let mut etcd = InMemoryEtcd::new(1);
        let err = ctrl.run_with_etcd(&mut etcd).unwrap_err();
        assert!(matches!(err, UpgradeError::EtcdLeaveFailed(_)));
        // No member removed.
        assert_eq!(etcd.member_count(), 1);
        assert_eq!(ctrl.phase(), UpgradePhase::Failed);
    }

    #[test]
    fn leave_from_two_member_cluster_is_allowed() {
        // Talos permits leaving down to a single member; quorum of the
        // remaining one-node cluster is preserved.
        let mut ctrl = UpgradeController::new(req());
        let mut etcd = InMemoryEtcd::new(2);
        let phase = ctrl.run_with_etcd(&mut etcd).unwrap();
        assert_eq!(phase, UpgradePhase::Rebooting);
        assert_eq!(etcd.member_count(), 1);
    }

    #[test]
    fn worker_upgrade_does_not_touch_etcd() {
        let mut ctrl = UpgradeController::new(req().with_role(NodeRole::Worker));
        // A non-member etcd must remain untouched.
        let mut etcd = InMemoryEtcd::non_member();
        let phase = ctrl.run_with_etcd(&mut etcd).unwrap();
        assert_eq!(phase, UpgradePhase::Rebooting);
        assert!(!ctrl.completed_steps().contains(&UpgradeStep::EtcdLeave));
    }

    #[test]
    fn step_phase_progression() {
        let mut ctrl = UpgradeController::new(req());
        let mut etcd = InMemoryEtcd::new(3);

        ctrl.step(&mut etcd).unwrap(); // validate
        assert_eq!(ctrl.phase(), UpgradePhase::Preflight);

        ctrl.step(&mut etcd).unwrap(); // drain
        ctrl.step(&mut etcd).unwrap(); // etcd leave
        assert_eq!(ctrl.phase(), UpgradePhase::Preflight);

        ctrl.step(&mut etcd).unwrap(); // stop services
        assert_eq!(ctrl.phase(), UpgradePhase::Installing);

        ctrl.step(&mut etcd).unwrap(); // install
        assert_eq!(ctrl.phase(), UpgradePhase::Installing);

        ctrl.step(&mut etcd).unwrap(); // reboot
        assert_eq!(ctrl.phase(), UpgradePhase::Rebooting);

        // No more steps.
        assert!(ctrl.next_step().is_none());
        assert!(ctrl.step(&mut etcd).is_err());
    }

    #[test]
    fn staged_and_preserve_flags() {
        let r = req().staged().preserve();
        assert!(r.staged);
        assert!(r.preserve);
    }
}
