//! Runtime `MachineStatus` controller.
//!
//! Mirrors Talos `internal/app/machined/pkg/controllers/runtime/machine_status.go`:
//! it aggregates the observed boot stage and a set of "unmet conditions" into a
//! single `MachineStatus` resource with a derived readiness boolean. Talos uses
//! this resource to gate `talosctl` operations and to drive the API readiness
//! reported to the cluster.

use crate::reconcile::{
    Controller, Input, Output, ReconcileContext, ReconcileError, ReconcileResult,
};
use os_cosi_domain::resource::ResourceKind;
use os_cosi_domain::{Metadata, Resource};
use os_kernel::ResourceId;

/// The coarse boot stage a Talos machine progresses through.
///
/// Talos models this as `runtime.MachineStage`: the machine moves from
/// `Booting` through `Installing`/`Maintenance` to `Running`, and into
/// `Rebooting`/`Shutting down`/`Resetting`/`Upgrading` for lifecycle ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineStage {
    /// Unknown / not yet observed.
    Unknown,
    /// Early boot, before the machine config is applied.
    Booting,
    /// Installing Talos to disk.
    Installing,
    /// Running in maintenance mode (no machine config).
    Maintenance,
    /// Fully booted and reconciling normally.
    Running,
    /// Rebooting.
    Rebooting,
    /// Shutting down.
    ShuttingDown,
    /// Resetting (wiping) the machine.
    Resetting,
    /// Upgrading Talos.
    Upgrading,
}

impl MachineStage {
    /// Stable lowercase string for the stage.
    pub fn as_str(&self) -> &'static str {
        match self {
            MachineStage::Unknown => "unknown",
            MachineStage::Booting => "booting",
            MachineStage::Installing => "installing",
            MachineStage::Maintenance => "maintenance",
            MachineStage::Running => "running",
            MachineStage::Rebooting => "rebooting",
            MachineStage::ShuttingDown => "shuttingdown",
            MachineStage::Resetting => "resetting",
            MachineStage::Upgrading => "upgrading",
        }
    }

    /// Whether the machine is in a steady, reconciling stage. Only `Running`
    /// can ever be considered ready.
    pub fn can_be_ready(&self) -> bool {
        matches!(self, MachineStage::Running)
    }

    /// Whether the stage represents an in-progress lifecycle transition where
    /// readiness must be reported as `false` regardless of conditions.
    pub fn is_transitional(&self) -> bool {
        matches!(
            self,
            MachineStage::Rebooting
                | MachineStage::ShuttingDown
                | MachineStage::Resetting
                | MachineStage::Upgrading
                | MachineStage::Installing
        )
    }
}

/// An input resource reporting the current boot stage, produced upstream by the
/// machined boot sequencer. Modeled here as a controller input.
#[derive(Debug, Clone)]
pub struct StageReport {
    meta: Metadata,
    /// The observed stage.
    pub stage: MachineStage,
    /// Human-readable status text.
    pub status: String,
}

impl StageReport {
    /// The well-known singleton id Talos uses for machine-wide status.
    pub const ID: &'static str = "machine";

    /// Build a stage report singleton.
    pub fn new(stage: MachineStage, status: impl Into<String>) -> Self {
        StageReport {
            meta: Metadata::new("runtime", "StageReport", ResourceId::new(Self::ID).unwrap()),
            stage,
            status: status.into(),
        }
    }

    /// The resource kind for stage reports.
    pub fn kind() -> ResourceKind {
        ResourceKind::new("runtime", "StageReport")
    }
}

impl Resource for StageReport {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
    fn spec_fingerprint(&self) -> String {
        format!("stage={};status={}", self.stage.as_str(), self.status)
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// An unmet readiness condition reported by some subsystem (network, time,
/// etcd, ...). The presence of any such resource keeps the machine not-ready.
#[derive(Debug, Clone)]
pub struct UnmetCondition {
    meta: Metadata,
    /// Why the condition is unmet.
    pub reason: String,
}

impl UnmetCondition {
    /// Build an unmet condition with id `name`.
    pub fn new(name: &str, reason: impl Into<String>) -> Self {
        UnmetCondition {
            meta: Metadata::new("runtime", "UnmetCondition", ResourceId::new(name).unwrap()),
            reason: reason.into(),
        }
    }

    /// The resource kind for unmet conditions.
    pub fn kind() -> ResourceKind {
        ResourceKind::new("runtime", "UnmetCondition")
    }
}

impl Resource for UnmetCondition {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
    fn spec_fingerprint(&self) -> String {
        format!("reason={}", self.reason)
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// The aggregated machine status output.
#[derive(Debug, Clone)]
pub struct MachineStatus {
    meta: Metadata,
    /// The current stage.
    pub stage: MachineStage,
    /// Whether the machine is ready (running + no unmet conditions).
    pub ready: bool,
    /// The list of unmet condition reasons (sorted), empty when ready.
    pub unmet: Vec<String>,
    /// Status text mirrored from the stage report.
    pub status: String,
}

impl MachineStatus {
    /// The singleton id.
    pub const ID: &'static str = "machine";

    /// The resource kind for machine status.
    pub fn kind() -> ResourceKind {
        ResourceKind::new("runtime", "MachineStatus")
    }
}

impl Resource for MachineStatus {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
    fn spec_fingerprint(&self) -> String {
        format!(
            "stage={};ready={};unmet=[{}]",
            self.stage.as_str(),
            self.ready,
            self.unmet.join(",")
        )
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// The `MachineStatus` controller. Reads the [`StageReport`] singleton and all
/// [`UnmetCondition`]s and produces the single [`MachineStatus`] resource.
#[derive(Debug, Default)]
pub struct MachineStatusController;

impl MachineStatusController {
    /// Construct the controller.
    pub fn new() -> Self {
        MachineStatusController
    }

    /// Pure derivation used by the controller and exercised directly in tests:
    /// given a stage and a set of unmet reasons, compute the status.
    pub fn derive(stage: MachineStage, mut unmet: Vec<String>, status: String) -> MachineStatus {
        unmet.sort();
        unmet.dedup();
        let ready = stage.can_be_ready() && !stage.is_transitional() && unmet.is_empty();
        MachineStatus {
            meta: Metadata::new(
                "runtime",
                "MachineStatus",
                ResourceId::new(MachineStatus::ID).unwrap(),
            ),
            stage,
            ready,
            unmet,
            status,
        }
    }
}

impl Controller for MachineStatusController {
    fn name(&self) -> &str {
        "runtime.MachineStatusController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![
            Input::weak(StageReport::kind()),
            Input::weak(UnmetCondition::kind()),
        ]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(MachineStatus::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let report = ctx
            .get(&format!("runtime/StageReport/{}", StageReport::ID))
            .ok_or_else(|| ReconcileError::MissingInput("runtime/StageReport/machine".into()))?;

        // Parse the stage/status out of the fingerprint (the store hands back
        // type-erased resources; in real COSI we would downcast).
        let fp = report.spec_fingerprint();
        let (stage, status) = parse_stage_report(&fp);

        let unmet: Vec<String> = ctx
            .list(&UnmetCondition::kind())
            .iter()
            .map(|c| c.metadata().id().as_str().to_string())
            .collect();

        let status = MachineStatusController::derive(stage, unmet, status);
        ctx.write(Box::new(status))?;
        Ok(())
    }
}

fn parse_stage_report(fp: &str) -> (MachineStage, String) {
    let mut stage = MachineStage::Unknown;
    let mut status = String::new();
    for part in fp.split(';') {
        if let Some(v) = part.strip_prefix("stage=") {
            stage = match v {
                "booting" => MachineStage::Booting,
                "installing" => MachineStage::Installing,
                "maintenance" => MachineStage::Maintenance,
                "running" => MachineStage::Running,
                "rebooting" => MachineStage::Rebooting,
                "shuttingdown" => MachineStage::ShuttingDown,
                "resetting" => MachineStage::Resetting,
                "upgrading" => MachineStage::Upgrading,
                _ => MachineStage::Unknown,
            };
        } else if let Some(v) = part.strip_prefix("status=") {
            status = v.to_string();
        }
    }
    (stage, status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_cosi_domain::State;

    #[test]
    fn ready_only_when_running_and_no_conditions() {
        let s = MachineStatusController::derive(MachineStage::Running, vec![], "ok".into());
        assert!(s.ready);
        assert_eq!(s.stage.as_str(), "running");

        let s = MachineStatusController::derive(
            MachineStage::Running,
            vec!["network".into()],
            "waiting".into(),
        );
        assert!(!s.ready);
        assert_eq!(s.unmet, vec!["network".to_string()]);
    }

    #[test]
    fn transitional_stage_is_never_ready() {
        let s = MachineStatusController::derive(MachineStage::Upgrading, vec![], "x".into());
        assert!(!s.ready);
        assert!(s.stage.is_transitional());
        let s = MachineStatusController::derive(MachineStage::Maintenance, vec![], "x".into());
        assert!(!s.ready);
    }

    #[test]
    fn unmet_reasons_are_sorted_and_deduped() {
        let s = MachineStatusController::derive(
            MachineStage::Booting,
            vec!["b".into(), "a".into(), "a".into()],
            "x".into(),
        );
        assert_eq!(s.unmet, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn controller_requires_stage_report() {
        let mut state = State::new();
        let mut ctx = ReconcileContext::new(
            &mut state,
            "runtime.MachineStatusController",
            vec![MachineStatus::kind()],
        );
        let mut c = MachineStatusController::new();
        let err = c.reconcile(&mut ctx).unwrap_err();
        assert!(matches!(err, ReconcileError::MissingInput(_)));
    }

    #[test]
    fn controller_produces_ready_status_end_to_end() {
        let mut state = State::new();
        state
            .create(Box::new(StageReport::new(MachineStage::Running, "booted")))
            .unwrap();
        let mut c = MachineStatusController::new();
        {
            let mut ctx = ReconcileContext::new(
                &mut state,
                "runtime.MachineStatusController",
                vec![MachineStatus::kind()],
            );
            c.reconcile(&mut ctx).unwrap();
        }
        let out = state.get("runtime/MachineStatus/machine").unwrap();
        assert_eq!(out.spec_fingerprint(), "stage=running;ready=true;unmet=[]");
        assert_eq!(out.metadata().owner(), "runtime.MachineStatusController");
    }

    #[test]
    fn controller_reports_unmet_conditions() {
        let mut state = State::new();
        state
            .create(Box::new(StageReport::new(MachineStage::Running, "booted")))
            .unwrap();
        state
            .create(Box::new(UnmetCondition::new("time", "clock not synced")))
            .unwrap();
        let mut c = MachineStatusController::new();
        {
            let mut ctx = ReconcileContext::new(
                &mut state,
                "runtime.MachineStatusController",
                vec![MachineStatus::kind()],
            );
            c.reconcile(&mut ctx).unwrap();
        }
        let out = state.get("runtime/MachineStatus/machine").unwrap();
        assert_eq!(
            out.spec_fingerprint(),
            "stage=running;ready=false;unmet=[time]"
        );
    }
}
