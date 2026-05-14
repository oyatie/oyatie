//! SaaS workflow application — engine façade over the domain ledger.
//!
//! Exposes the `workflow.definition.publish` + `workflow.run.start` capability
//! surface required by M03-P04-IP-001. The app layer owns:
//! * input-shape validation that ties together kernel + domain types,
//! * the public `publish` / `start_run` / `record_step` / `complete_run` API,
//! * a per-tenant SLO counter for the preview observability lane.
//!
//! No external Rust deps — std + workspace path deps only per ADR-0015.

use std::collections::BTreeMap;

use oya_saas_workflow_domain::{WorkflowDomainError, WorkflowLedger, WorkflowRunSnapshot};
use oya_saas_workflow_kernel::{
    WorkflowDefinition, WorkflowDefinitionId, WorkflowEvent, WorkflowEventKind, WorkflowRunId,
    WorkflowRunState, WorkflowStep, WorkflowStepId, WorkflowStepKind,
};

/// Errors returned by the workflow application façade.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowAppError {
    EmptyTenantId,
    EmptyRegionalPack,
    EmptyStepList,
    InvalidId,
    Domain(WorkflowDomainError),
}

impl From<WorkflowDomainError> for WorkflowAppError {
    fn from(value: WorkflowDomainError) -> Self {
        Self::Domain(value)
    }
}

/// Shape used by the public REST API for publishing definitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishDefinitionInput {
    pub definition_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub regional_pack: String,           // data_class: INTERNAL_ONLY
    pub steps: Vec<PublishStepInput>,    // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

/// Shape of a workflow step inside [`PublishDefinitionInput`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishStepInput {
    pub step_id: String,         // data_class: INTERNAL_ONLY
    pub kind: WorkflowStepKind,  // data_class: INTERNAL_ONLY
    pub order: u32,              // data_class: INTERNAL_ONLY
    pub plugin_manifest: String, // data_class: INTERNAL_ONLY
}

/// Shape used by the public REST API for starting runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartRunInput {
    pub run_id: String,                // data_class: INTERNAL_ONLY
    pub definition_id: String,         // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

/// Per-tenant SLO counters surfaced to the preview observability lane.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowSloCounters {
    pub definitions_published: u64, // data_class: INTERNAL_ONLY
    pub runs_started: u64,          // data_class: INTERNAL_ONLY
    pub runs_succeeded: u64,        // data_class: INTERNAL_ONLY
    pub runs_failed: u64,           // data_class: INTERNAL_ONLY
}

/// Façade that owns the workflow ledger + tenant SLO counters.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowEngine {
    ledger: WorkflowLedger,
    slo: BTreeMap<String, WorkflowSloCounters>,
}

impl WorkflowEngine {
    /// `workflow.definition.publish` — validates input + delegates to ledger.
    pub fn publish(
        &mut self,
        input: PublishDefinitionInput,
    ) -> Result<WorkflowEvent, WorkflowAppError> {
        if input.tenant_id.is_empty() {
            return Err(WorkflowAppError::EmptyTenantId);
        }
        if input.regional_pack.is_empty() {
            return Err(WorkflowAppError::EmptyRegionalPack);
        }
        if input.steps.is_empty() {
            return Err(WorkflowAppError::EmptyStepList);
        }
        let mut steps = Vec::with_capacity(input.steps.len());
        for step in input.steps {
            steps.push(WorkflowStep::new(
                WorkflowStepId::new(step.step_id).map_err(|_| WorkflowAppError::InvalidId)?,
                step.kind,
                step.order,
                step.plugin_manifest,
            ));
        }
        let definition = WorkflowDefinition::new(
            WorkflowDefinitionId::new(input.definition_id)
                .map_err(|_| WorkflowAppError::InvalidId)?,
            input.tenant_id.clone(),
            input.regional_pack,
            steps,
            input.published_at_epoch_seconds,
        )
        .map_err(|err| WorkflowAppError::Domain(WorkflowDomainError::Kernel(err)))?;
        let event = self.ledger.publish(definition)?;
        self.slo
            .entry(input.tenant_id)
            .or_default()
            .definitions_published += 1;
        Ok(event)
    }

    /// `workflow.run.start` — instantiates a run from a published definition.
    pub fn start_run(&mut self, input: StartRunInput) -> Result<WorkflowEvent, WorkflowAppError> {
        let definition_id = WorkflowDefinitionId::new(input.definition_id)
            .map_err(|_| WorkflowAppError::InvalidId)?;
        let run_id = WorkflowRunId::new(input.run_id).map_err(|_| WorkflowAppError::InvalidId)?;
        let event =
            self.ledger
                .start_run(run_id, &definition_id, input.started_at_epoch_seconds)?;
        if let Some(run) = self.ledger.runs().find(|r| r.id == event.run_id).cloned() {
            self.slo.entry(run.tenant_id).or_default().runs_started += 1;
        }
        Ok(event)
    }

    pub fn record_step(
        &mut self,
        run_id: &WorkflowRunId,
        step_id: &WorkflowStepId,
        kind: WorkflowEventKind,
        occurred_at_epoch_seconds: u64,
    ) -> Result<WorkflowEvent, WorkflowAppError> {
        Ok(self
            .ledger
            .record_step_event(run_id, step_id, kind, occurred_at_epoch_seconds)?)
    }

    pub fn complete_run(
        &mut self,
        run_id: &WorkflowRunId,
        terminal: WorkflowRunState,
        occurred_at_epoch_seconds: u64,
    ) -> Result<WorkflowEvent, WorkflowAppError> {
        let event = self
            .ledger
            .finish_run(run_id, terminal, occurred_at_epoch_seconds)?;
        if let Some(run) = self.ledger.runs().find(|r| r.id == *run_id).cloned() {
            let bucket = self.slo.entry(run.tenant_id).or_default();
            match terminal {
                WorkflowRunState::Succeeded => bucket.runs_succeeded += 1,
                WorkflowRunState::Failed => bucket.runs_failed += 1,
                _ => {}
            }
        }
        Ok(event)
    }

    pub fn snapshot(&self, run_id: &WorkflowRunId) -> Option<WorkflowRunSnapshot> {
        self.ledger.snapshot(run_id)
    }

    pub fn counters(&self, tenant_id: &str) -> WorkflowSloCounters {
        self.slo.get(tenant_id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn publish_fixture(definition_id: &str) -> PublishDefinitionInput {
        PublishDefinitionInput {
            definition_id: definition_id.to_string(),
            tenant_id: "ten_acme".to_string(),
            regional_pack: "oya-pack-kr".to_string(),
            steps: vec![
                PublishStepInput {
                    step_id: "wfs_extract".to_string(),
                    kind: WorkflowStepKind::Plugin,
                    order: 1,
                    plugin_manifest: "oya:plugin:extract@1".to_string(),
                },
                PublishStepInput {
                    step_id: "wfs_summarize".to_string(),
                    kind: WorkflowStepKind::Plugin,
                    order: 2,
                    plugin_manifest: "oya:plugin:summarize@1".to_string(),
                },
            ],
            published_at_epoch_seconds: 1_700_000_000,
        }
    }

    #[test]
    fn publish_emits_event_and_increments_slo() {
        let mut engine = WorkflowEngine::default();
        let event = engine
            .publish(publish_fixture("wfd_v1"))
            .expect("publish ok");
        assert_eq!(event.kind, WorkflowEventKind::DefinitionPublished);
        assert_eq!(engine.counters("ten_acme").definitions_published, 1);
    }

    #[test]
    fn publish_rejects_empty_tenant_pack_or_steps() {
        let mut engine = WorkflowEngine::default();
        let no_tenant = engine
            .publish(PublishDefinitionInput {
                tenant_id: String::new(),
                ..publish_fixture("wfd_a")
            })
            .expect_err("empty tenant rejected");
        assert_eq!(no_tenant, WorkflowAppError::EmptyTenantId);

        let no_pack = engine
            .publish(PublishDefinitionInput {
                regional_pack: String::new(),
                ..publish_fixture("wfd_b")
            })
            .expect_err("empty pack rejected");
        assert_eq!(no_pack, WorkflowAppError::EmptyRegionalPack);

        let no_steps = engine
            .publish(PublishDefinitionInput {
                steps: vec![],
                ..publish_fixture("wfd_c")
            })
            .expect_err("empty steps rejected");
        assert_eq!(no_steps, WorkflowAppError::EmptyStepList);
    }

    #[test]
    fn start_run_then_record_step_then_complete() {
        let mut engine = WorkflowEngine::default();
        engine.publish(publish_fixture("wfd_v1")).unwrap();
        let started = engine
            .start_run(StartRunInput {
                run_id: "wfr_1".to_string(),
                definition_id: "wfd_v1".to_string(),
                started_at_epoch_seconds: 1_700_000_100,
            })
            .expect("run started");
        assert_eq!(started.kind, WorkflowEventKind::RunStarted);

        let step_event = engine
            .record_step(
                &WorkflowRunId::new("wfr_1").unwrap(),
                &WorkflowStepId::new("wfs_extract").unwrap(),
                WorkflowEventKind::StepCompleted,
                1_700_000_110,
            )
            .expect("step recorded");
        assert_eq!(step_event.kind, WorkflowEventKind::StepCompleted);

        let done = engine
            .complete_run(
                &WorkflowRunId::new("wfr_1").unwrap(),
                WorkflowRunState::Succeeded,
                1_700_000_900,
            )
            .expect("run completed");
        assert_eq!(done.kind, WorkflowEventKind::RunCompleted);
        let counters = engine.counters("ten_acme");
        assert_eq!(counters.runs_started, 1);
        assert_eq!(counters.runs_succeeded, 1);
        assert_eq!(counters.runs_failed, 0);
    }

    #[test]
    fn snapshot_returns_run_and_event_trail() {
        let mut engine = WorkflowEngine::default();
        engine.publish(publish_fixture("wfd_snap")).unwrap();
        engine
            .start_run(StartRunInput {
                run_id: "wfr_snap".to_string(),
                definition_id: "wfd_snap".to_string(),
                started_at_epoch_seconds: 1_700_000_100,
            })
            .unwrap();
        let snap = engine
            .snapshot(&WorkflowRunId::new("wfr_snap").unwrap())
            .expect("snapshot exists");
        assert_eq!(snap.events.len(), 1);
        assert_eq!(snap.run.state, WorkflowRunState::Running);
    }

    #[test]
    fn invalid_id_inputs_are_rejected_with_invalid_id() {
        let mut engine = WorkflowEngine::default();
        let bad_def_id = engine
            .publish(PublishDefinitionInput {
                definition_id: "nope".to_string(),
                ..publish_fixture("wfd_v1")
            })
            .expect_err("bad definition id");
        assert_eq!(bad_def_id, WorkflowAppError::InvalidId);

        let bad_step_id = engine
            .publish(PublishDefinitionInput {
                steps: vec![PublishStepInput {
                    step_id: "nope".to_string(),
                    kind: WorkflowStepKind::Plugin,
                    order: 1,
                    plugin_manifest: "oya:plugin:x@1".to_string(),
                }],
                ..publish_fixture("wfd_v2")
            })
            .expect_err("bad step id");
        assert_eq!(bad_step_id, WorkflowAppError::InvalidId);
    }
}
