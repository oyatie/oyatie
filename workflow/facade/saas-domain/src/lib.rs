//! SaaS workflow domain — definition publishing + run-event recording.
//!
//! Wraps the kernel identity/contract types in a ledger that enforces:
//! * one-shot publish per definition id,
//! * monotonically increasing run start timestamps,
//! * `workflow.run.event` ordering tied to definition step order.
//!
//! Per ADR-0023 the engine never executes plugin code itself — plugin step
//! invocations are dispatched to [`saas_plugin_app`] from the app
//! layer; the domain only records audit events.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use workflow_saas_kernel::{
    WorkflowDefinition, WorkflowDefinitionId, WorkflowEvent, WorkflowEventId, WorkflowEventKind,
    WorkflowKernelError, WorkflowRun, WorkflowRunId, WorkflowRunState, WorkflowStepId,
};

/// Errors surfaced when the workflow domain rejects a state mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowDomainError {
    DuplicateDefinition,
    UnknownDefinition,
    DefinitionMismatch,
    DuplicateRun,
    UnknownRun,
    UnknownStep,
    RunNotRunning,
    TenantMismatch,
    Kernel(WorkflowKernelError),
}

impl From<WorkflowKernelError> for WorkflowDomainError {
    fn from(value: WorkflowKernelError) -> Self {
        Self::Kernel(value)
    }
}

/// In-process ledger backing the SaaS workflow engine preview.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowLedger {
    definitions: BTreeMap<WorkflowDefinitionId, WorkflowDefinition>,
    runs: BTreeMap<WorkflowRunId, WorkflowRun>,
    events_by_run: BTreeMap<WorkflowRunId, Vec<WorkflowEvent>>,
    next_event_seq: u64,
}

/// Snapshot of a run plus its emitted events (used by app layer + bench).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRunSnapshot {
    pub definition: WorkflowDefinition, // data_class: INTERNAL_ONLY
    pub run: WorkflowRun,               // data_class: INTERNAL_ONLY
    pub events: Vec<WorkflowEvent>,     // data_class: INTERNAL_ONLY
}

impl WorkflowLedger {
    /// Publish a definition. Returns the canonical `DefinitionPublished` event.
    pub fn publish(
        &mut self,
        definition: WorkflowDefinition,
    ) -> Result<WorkflowEvent, WorkflowDomainError> {
        if self.definitions.contains_key(&definition.id) {
            return Err(WorkflowDomainError::DuplicateDefinition);
        }
        let event = WorkflowEvent::new(
            self.next_event_id()?,
            // synthetic run id wrapping the definition id so the publish event
            // is greppable in the per-run audit channel later.
            WorkflowRunId::new(format!("wfr_pub_{}", &definition.id.value["wfd_".len()..]))?,
            None,
            WorkflowEventKind::DefinitionPublished,
            definition.published_at_epoch_seconds,
        );
        self.definitions.insert(definition.id.clone(), definition);
        Ok(event)
    }

    /// Start a run for a previously published definition.
    pub fn start_run(
        &mut self,
        run_id: WorkflowRunId,
        definition_id: &WorkflowDefinitionId,
        started_at_epoch_seconds: u64,
    ) -> Result<WorkflowEvent, WorkflowDomainError> {
        let definition = self
            .definitions
            .get(definition_id)
            .ok_or(WorkflowDomainError::UnknownDefinition)?;
        if self.runs.contains_key(&run_id) {
            return Err(WorkflowDomainError::DuplicateRun);
        }
        let run = WorkflowRun::start(run_id.clone(), definition, started_at_epoch_seconds)?;
        let event = WorkflowEvent::new(
            self.next_event_id()?,
            run.id.clone(),
            None,
            WorkflowEventKind::RunStarted,
            started_at_epoch_seconds,
        );
        self.runs.insert(run.id.clone(), run);
        self.events_by_run
            .entry(run_id.clone())
            .or_default()
            .push(event.clone());
        Ok(event)
    }

    /// Record a per-step audit event for a running run.
    pub fn record_step_event(
        &mut self,
        run_id: &WorkflowRunId,
        step_id: &WorkflowStepId,
        kind: WorkflowEventKind,
        occurred_at_epoch_seconds: u64,
    ) -> Result<WorkflowEvent, WorkflowDomainError> {
        let run = self
            .runs
            .get(run_id)
            .ok_or(WorkflowDomainError::UnknownRun)?;
        if run.state != WorkflowRunState::Running {
            return Err(WorkflowDomainError::RunNotRunning);
        }
        let definition = self
            .definitions
            .get(&run.definition_id)
            .ok_or(WorkflowDomainError::UnknownDefinition)?;
        if definition.step(step_id).is_none() {
            return Err(WorkflowDomainError::UnknownStep);
        }
        let event = WorkflowEvent::new(
            self.next_event_id()?,
            run_id.clone(),
            Some(step_id.clone()),
            kind,
            occurred_at_epoch_seconds,
        );
        self.events_by_run
            .entry(run_id.clone())
            .or_default()
            .push(event.clone());
        Ok(event)
    }

    /// Terminate a run with the given terminal kind.
    pub fn finish_run(
        &mut self,
        run_id: &WorkflowRunId,
        terminal: WorkflowRunState,
        occurred_at_epoch_seconds: u64,
    ) -> Result<WorkflowEvent, WorkflowDomainError> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or(WorkflowDomainError::UnknownRun)?;
        run.transition(terminal)?;
        let kind = match terminal {
            WorkflowRunState::Succeeded => WorkflowEventKind::RunCompleted,
            WorkflowRunState::Failed => WorkflowEventKind::RunFailed,
            WorkflowRunState::Cancelled => WorkflowEventKind::RunCancelled,
            _ => return Err(WorkflowDomainError::RunNotRunning),
        };
        let event = WorkflowEvent::new(
            self.next_event_id()?,
            run_id.clone(),
            None,
            kind,
            occurred_at_epoch_seconds,
        );
        self.events_by_run
            .entry(run_id.clone())
            .or_default()
            .push(event.clone());
        Ok(event)
    }

    pub fn snapshot(&self, run_id: &WorkflowRunId) -> Option<WorkflowRunSnapshot> {
        let run = self.runs.get(run_id)?.clone();
        let definition = self.definitions.get(&run.definition_id)?.clone();
        let events = self.events_by_run.get(run_id).cloned().unwrap_or_default();
        Some(WorkflowRunSnapshot {
            definition,
            run,
            events,
        })
    }

    pub fn restore_snapshot(
        &mut self,
        snapshot: WorkflowRunSnapshot,
    ) -> Result<(), WorkflowDomainError> {
        if self.runs.contains_key(&snapshot.run.id) {
            return Err(WorkflowDomainError::DuplicateRun);
        }
        if snapshot.run.definition_id != snapshot.definition.id
            || snapshot.run.tenant_id != snapshot.definition.tenant_id
            || snapshot.run.regional_pack != snapshot.definition.regional_pack
        {
            return Err(WorkflowDomainError::DefinitionMismatch);
        }
        if snapshot
            .events
            .iter()
            .any(|event| event.run_id != snapshot.run.id)
        {
            return Err(WorkflowDomainError::TenantMismatch);
        }
        if let Some(existing) = self.definitions.get(&snapshot.definition.id) {
            if existing != &snapshot.definition {
                return Err(WorkflowDomainError::DefinitionMismatch);
            }
        } else {
            self.definitions
                .insert(snapshot.definition.id.clone(), snapshot.definition);
        }
        if let Some(max_restored_seq) = snapshot.events.iter().filter_map(event_sequence).max() {
            self.next_event_seq = self.next_event_seq.max(max_restored_seq);
        }
        self.events_by_run
            .insert(snapshot.run.id.clone(), snapshot.events);
        self.runs.insert(snapshot.run.id.clone(), snapshot.run);
        Ok(())
    }

    pub fn definitions(&self) -> impl Iterator<Item = &WorkflowDefinition> {
        self.definitions.values()
    }

    pub fn runs(&self) -> impl Iterator<Item = &WorkflowRun> {
        self.runs.values()
    }

    fn next_event_id(&mut self) -> Result<WorkflowEventId, WorkflowKernelError> {
        self.next_event_seq += 1;
        WorkflowEventId::new(format!("wfe_{:020}", self.next_event_seq))
    }
}

fn event_sequence(event: &WorkflowEvent) -> Option<u64> {
    event.id.as_str().strip_prefix("wfe_")?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use workflow_saas_kernel::{WorkflowStep, WorkflowStepKind};

    fn fixture_definition(id: &str) -> WorkflowDefinition {
        WorkflowDefinition::new(
            WorkflowDefinitionId::new(id).unwrap(),
            "ten_acme",
            "oya-pack-alpha",
            vec![
                WorkflowStep::new(
                    WorkflowStepId::new("wfs_extract").unwrap(),
                    WorkflowStepKind::Plugin,
                    1,
                    "oya:plugin:extract@1",
                ),
                WorkflowStep::new(
                    WorkflowStepId::new("wfs_summarize").unwrap(),
                    WorkflowStepKind::Plugin,
                    2,
                    "oya:plugin:summarize@1",
                ),
            ],
            1_700_000_000,
        )
        .expect("valid fixture definition")
    }

    #[test]
    fn ledger_publishes_definitions_once() {
        let mut ledger = WorkflowLedger::default();
        let event = ledger
            .publish(fixture_definition("wfd_summarize_v1"))
            .expect("first publish");
        assert_eq!(event.kind, WorkflowEventKind::DefinitionPublished);

        let dup = ledger
            .publish(fixture_definition("wfd_summarize_v1"))
            .expect_err("duplicate publish rejected");
        assert_eq!(dup, WorkflowDomainError::DuplicateDefinition);
    }

    #[test]
    fn ledger_starts_runs_only_against_published_definitions() {
        let mut ledger = WorkflowLedger::default();
        let missing = ledger
            .start_run(
                WorkflowRunId::new("wfr_001").unwrap(),
                &WorkflowDefinitionId::new("wfd_missing").unwrap(),
                1_700_000_100,
            )
            .expect_err("must publish before run");
        assert_eq!(missing, WorkflowDomainError::UnknownDefinition);

        let _ = ledger
            .publish(fixture_definition("wfd_summarize_v1"))
            .unwrap();
        let started = ledger
            .start_run(
                WorkflowRunId::new("wfr_001").unwrap(),
                &WorkflowDefinitionId::new("wfd_summarize_v1").unwrap(),
                1_700_000_100,
            )
            .expect("run starts");
        assert_eq!(started.kind, WorkflowEventKind::RunStarted);
    }

    #[test]
    fn ledger_records_step_events_in_running_state() {
        let mut ledger = WorkflowLedger::default();
        let _ = ledger.publish(fixture_definition("wfd_a")).unwrap();
        let _ = ledger
            .start_run(
                WorkflowRunId::new("wfr_a").unwrap(),
                &WorkflowDefinitionId::new("wfd_a").unwrap(),
                1_700_000_100,
            )
            .unwrap();
        let started = ledger
            .record_step_event(
                &WorkflowRunId::new("wfr_a").unwrap(),
                &WorkflowStepId::new("wfs_extract").unwrap(),
                WorkflowEventKind::StepStarted,
                1_700_000_110,
            )
            .expect("step event recorded");
        assert_eq!(started.kind, WorkflowEventKind::StepStarted);

        let unknown = ledger
            .record_step_event(
                &WorkflowRunId::new("wfr_a").unwrap(),
                &WorkflowStepId::new("wfs_ghost").unwrap(),
                WorkflowEventKind::StepStarted,
                1_700_000_111,
            )
            .expect_err("unknown step rejected");
        assert_eq!(unknown, WorkflowDomainError::UnknownStep);
    }

    #[test]
    fn ledger_finishes_runs_with_terminal_event() {
        let mut ledger = WorkflowLedger::default();
        let _ = ledger.publish(fixture_definition("wfd_b")).unwrap();
        let _ = ledger
            .start_run(
                WorkflowRunId::new("wfr_b").unwrap(),
                &WorkflowDefinitionId::new("wfd_b").unwrap(),
                1_700_000_100,
            )
            .unwrap();
        let done = ledger
            .finish_run(
                &WorkflowRunId::new("wfr_b").unwrap(),
                WorkflowRunState::Succeeded,
                1_700_000_900,
            )
            .expect("terminal succeeded");
        assert_eq!(done.kind, WorkflowEventKind::RunCompleted);
        let again = ledger.finish_run(
            &WorkflowRunId::new("wfr_b").unwrap(),
            WorkflowRunState::Succeeded,
            1_700_000_999,
        );
        assert!(again.is_err(), "terminal state is immutable");
    }

    #[test]
    fn snapshot_includes_run_state_and_event_trail() {
        let mut ledger = WorkflowLedger::default();
        let _ = ledger.publish(fixture_definition("wfd_c")).unwrap();
        let run_id = WorkflowRunId::new("wfr_c").unwrap();
        let _ = ledger
            .start_run(
                run_id.clone(),
                &WorkflowDefinitionId::new("wfd_c").unwrap(),
                1_700_000_100,
            )
            .unwrap();
        let _ = ledger
            .record_step_event(
                &run_id,
                &WorkflowStepId::new("wfs_extract").unwrap(),
                WorkflowEventKind::StepStarted,
                1_700_000_110,
            )
            .unwrap();

        let snap = ledger.snapshot(&run_id).expect("snapshot exists");
        assert_eq!(snap.run.id, run_id);
        assert_eq!(snap.events.len(), 2);
        assert_eq!(snap.events[0].kind, WorkflowEventKind::RunStarted);
        assert_eq!(snap.events[1].kind, WorkflowEventKind::StepStarted);
    }

    #[test]
    fn restore_snapshot_recovers_run_history_without_reexecution() {
        let mut ledger = WorkflowLedger::default();
        let _ = ledger.publish(fixture_definition("wfd_restore")).unwrap();
        let run_id = WorkflowRunId::new("wfr_restore").unwrap();
        let _ = ledger
            .start_run(
                run_id.clone(),
                &WorkflowDefinitionId::new("wfd_restore").unwrap(),
                1_700_000_100,
            )
            .unwrap();
        let _ = ledger
            .record_step_event(
                &run_id,
                &WorkflowStepId::new("wfs_extract").unwrap(),
                WorkflowEventKind::StepCompleted,
                1_700_000_110,
            )
            .unwrap();
        let snap = ledger.snapshot(&run_id).unwrap();
        let mut restored = WorkflowLedger::default();
        restored.restore_snapshot(snap).unwrap();
        let next_event = restored
            .record_step_event(
                &run_id,
                &WorkflowStepId::new("wfs_summarize").unwrap(),
                WorkflowEventKind::StepStarted,
                1_700_000_120,
            )
            .unwrap();
        assert_eq!(next_event.id.as_str(), "wfe_00000000000000000004");
    }
}
