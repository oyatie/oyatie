//! SaaS workflow application — engine façade over the domain ledger.
//!
//! Exposes the `workflow.definition.publish` + `workflow.run.start` capability
//! surface required by M03-P04-IP-001. The app layer owns:
//! * input-shape validation that ties together kernel + domain types,
//! * the public `publish` / `start_run` / `record_step` / `complete_run` API,
//! * a per-tenant SLO counter for the preview observability lane.
//!
//! No external Rust deps — std + workspace path deps only per ADR-0015.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use oya_saas_workflow_domain::{WorkflowDomainError, WorkflowLedger, WorkflowRunSnapshot};
use oya_saas_workflow_kernel::{
    WorkflowDefinition, WorkflowDefinitionId, WorkflowEvent, WorkflowEventKind, WorkflowRunId,
    WorkflowRunState, WorkflowStep, WorkflowStepId, WorkflowStepKind,
};

const MAX_RETRY_ATTEMPTS: u32 = 1_000;

/// Errors returned by the workflow application façade.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowAppError {
    EmptyTenantId,
    EmptyRegionalPack,
    EmptyStepList,
    EmptyIdempotencyKey,
    EmptySideEffectRef,
    EmptyErrorClass,
    EmptyCancellationReasonRef,
    InvalidAttempt,
    InvalidRetryPolicy,
    StepAttemptNotStarted,
    StepAttemptStartMismatch,
    StepTimeoutNotElapsed,
    ReceiptRunMismatch,
    ReceiptStepMismatch,
    ReceiptIdempotencyMismatch,
    ReceiptEventKindMismatch,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecuteStepOnceInput {
    pub run_id: String,                 // data_class: INTERNAL_ONLY
    pub step_id: String,                // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub side_effect_ref: String,        // data_class: INTERNAL_ONLY; metadata ref only
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStepExecutionReceipt {
    pub run_id: WorkflowRunId,   // data_class: INTERNAL_ONLY
    pub step_id: WorkflowStepId, // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub side_effect_ref: String, // data_class: INTERNAL_ONLY; metadata ref only
    pub event: WorkflowEvent,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStepRetryPolicy {
    pub max_attempts: u32,                        // data_class: INTERNAL_ONLY
    pub initial_delay_seconds: u64,               // data_class: INTERNAL_ONLY
    pub max_delay_seconds: u64,                   // data_class: INTERNAL_ONLY
    pub backoff_multiplier: u32,                  // data_class: INTERNAL_ONLY
    pub retryable_error_classes: Vec<String>,     // data_class: INTERNAL_ONLY
    pub non_retryable_error_classes: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartStepAttemptInput {
    pub run_id: String,                // data_class: INTERNAL_ONLY
    pub step_id: String,               // data_class: INTERNAL_ONLY
    pub attempt: u32,                  // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStepAttemptStartReceipt {
    pub run_id: WorkflowRunId,         // data_class: INTERNAL_ONLY
    pub step_id: WorkflowStepId,       // data_class: INTERNAL_ONLY
    pub attempt: u32,                  // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub event: WorkflowEvent,          // data_class: INTERNAL_ONLY
    pub schema_version: u32,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordStepFailureInput {
    pub run_id: String,                                // data_class: INTERNAL_ONLY
    pub step_id: String,                               // data_class: INTERNAL_ONLY
    pub attempt: u32,                                  // data_class: INTERNAL_ONLY
    pub error_class: String,                           // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,                // data_class: INTERNAL_ONLY
    pub retry_policy: Option<WorkflowStepRetryPolicy>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordStepTimeoutInput {
    pub run_id: String,                                // data_class: INTERNAL_ONLY
    pub step_id: String,                               // data_class: INTERNAL_ONLY
    pub attempt: u32,                                  // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64,                 // data_class: INTERNAL_ONLY
    pub timeout_seconds: u64,                          // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,                // data_class: INTERNAL_ONLY
    pub retry_policy: Option<WorkflowStepRetryPolicy>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowStepRetryDecision {
    NoRetryPolicy,
    RetryScheduled,
    AttemptsExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStepFailureReceipt {
    pub run_id: WorkflowRunId,                    // data_class: INTERNAL_ONLY
    pub step_id: WorkflowStepId,                  // data_class: INTERNAL_ONLY
    pub attempt: u32,                             // data_class: INTERNAL_ONLY
    pub error_class: String,                      // data_class: INTERNAL_ONLY
    pub decision: WorkflowStepRetryDecision,      // data_class: INTERNAL_ONLY
    pub next_attempt: Option<u32>,                // data_class: INTERNAL_ONLY
    pub next_retry_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub failure_event: WorkflowEvent,             // data_class: INTERNAL_ONLY
    pub retry_event: Option<WorkflowEvent>,       // data_class: INTERNAL_ONLY
    pub schema_version: u32,                      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelRunInput {
    pub run_id: String,                 // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub reason_ref: String,             // data_class: INTERNAL_ONLY; metadata ref only
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRunCancellationReceipt {
    pub run_id: WorkflowRunId, // data_class: INTERNAL_ONLY
    pub reason_ref: String,    // data_class: INTERNAL_ONLY; metadata ref only
    pub event: WorkflowEvent,  // data_class: INTERNAL_ONLY
    pub schema_version: u32,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRunResumeReport {
    pub run_id: WorkflowRunId,                 // data_class: INTERNAL_ONLY
    pub restored_events: usize,                // data_class: INTERNAL_ONLY
    pub restored_step_receipts: usize,         // data_class: INTERNAL_ONLY
    pub restored_step_failure_receipts: usize, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                   // data_class: INTERNAL_ONLY
}

/// Per-tenant SLO counters surfaced to the preview observability lane.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowSloCounters {
    pub definitions_published: u64,  // data_class: INTERNAL_ONLY
    pub runs_started: u64,           // data_class: INTERNAL_ONLY
    pub runs_succeeded: u64,         // data_class: INTERNAL_ONLY
    pub runs_failed: u64,            // data_class: INTERNAL_ONLY
    pub runs_cancelled: u64,         // data_class: INTERNAL_ONLY
    pub step_failures: u64,          // data_class: INTERNAL_ONLY
    pub step_timeouts: u64,          // data_class: INTERNAL_ONLY
    pub step_retries_scheduled: u64, // data_class: INTERNAL_ONLY
}

/// Façade that owns the workflow ledger + tenant SLO counters.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowEngine {
    ledger: WorkflowLedger,
    slo: BTreeMap<String, WorkflowSloCounters>,
    step_receipts: BTreeMap<WorkflowStepExecutionKey, WorkflowStepExecutionReceipt>,
    step_attempt_starts: BTreeMap<WorkflowStepAttemptKey, WorkflowStepAttemptStartReceipt>,
    step_failure_receipts: BTreeMap<WorkflowStepFailureKey, WorkflowStepFailureReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkflowStepExecutionKey {
    run_id: WorkflowRunId,
    step_id: WorkflowStepId,
    idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkflowStepAttemptKey {
    run_id: WorkflowRunId,
    step_id: WorkflowStepId,
    attempt: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkflowStepFailureKey {
    run_id: WorkflowRunId,
    step_id: WorkflowStepId,
    attempt: u32,
    failure_kind: WorkflowEventKind,
}

impl WorkflowStepRetryPolicy {
    pub fn new(
        max_attempts: u32,
        initial_delay_seconds: u64,
        max_delay_seconds: u64,
        backoff_multiplier: u32,
    ) -> Result<Self, WorkflowAppError> {
        let policy = Self {
            max_attempts,
            initial_delay_seconds,
            max_delay_seconds,
            backoff_multiplier,
            retryable_error_classes: Vec::new(),
            non_retryable_error_classes: Vec::new(),
        };
        validate_retry_policy(&policy)?;
        Ok(policy)
    }

    pub fn with_retryable_error(mut self, error_class: impl Into<String>) -> Self {
        let error_class = error_class.into();
        let error_class = error_class.trim();
        if !error_class.is_empty() {
            self.retryable_error_classes.push(error_class.to_string());
        }
        self
    }

    pub fn with_non_retryable_error(mut self, error_class: impl Into<String>) -> Self {
        let error_class = error_class.into();
        let error_class = error_class.trim();
        if !error_class.is_empty() {
            self.non_retryable_error_classes
                .push(error_class.to_string());
        }
        self
    }
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

    pub fn execute_step_once(
        &mut self,
        input: ExecuteStepOnceInput,
    ) -> Result<WorkflowStepExecutionReceipt, WorkflowAppError> {
        if input.idempotency_key.trim().is_empty() {
            return Err(WorkflowAppError::EmptyIdempotencyKey);
        }
        if input.side_effect_ref.trim().is_empty() {
            return Err(WorkflowAppError::EmptySideEffectRef);
        }
        let run_id = WorkflowRunId::new(input.run_id).map_err(|_| WorkflowAppError::InvalidId)?;
        let step_id =
            WorkflowStepId::new(input.step_id).map_err(|_| WorkflowAppError::InvalidId)?;
        let key = WorkflowStepExecutionKey {
            run_id: run_id.clone(),
            step_id: step_id.clone(),
            idempotency_key: input.idempotency_key.clone(),
        };
        if let Some(receipt) = self.step_receipts.get(&key) {
            return Ok(receipt.clone());
        }
        let event = self.record_step(
            &run_id,
            &step_id,
            WorkflowEventKind::StepCompleted,
            input.occurred_at_epoch_seconds,
        )?;
        let receipt = WorkflowStepExecutionReceipt {
            run_id,
            step_id,
            idempotency_key: input.idempotency_key,
            side_effect_ref: input.side_effect_ref,
            event,
            schema_version: 1,
        };
        self.step_receipts.insert(key, receipt.clone());
        Ok(receipt)
    }

    pub fn start_step_attempt(
        &mut self,
        input: StartStepAttemptInput,
    ) -> Result<WorkflowStepAttemptStartReceipt, WorkflowAppError> {
        if input.attempt == 0 {
            return Err(WorkflowAppError::InvalidAttempt);
        }
        let run_id = WorkflowRunId::new(input.run_id).map_err(|_| WorkflowAppError::InvalidId)?;
        let step_id =
            WorkflowStepId::new(input.step_id).map_err(|_| WorkflowAppError::InvalidId)?;
        let key = WorkflowStepAttemptKey {
            run_id: run_id.clone(),
            step_id: step_id.clone(),
            attempt: input.attempt,
        };
        if let Some(receipt) = self.step_attempt_starts.get(&key) {
            return Ok(receipt.clone());
        }
        let event = self.record_step(
            &run_id,
            &step_id,
            WorkflowEventKind::StepStarted,
            input.started_at_epoch_seconds,
        )?;
        let receipt = WorkflowStepAttemptStartReceipt {
            run_id,
            step_id,
            attempt: input.attempt,
            started_at_epoch_seconds: input.started_at_epoch_seconds,
            event,
            schema_version: 1,
        };
        self.step_attempt_starts.insert(key, receipt.clone());
        Ok(receipt)
    }

    pub fn record_step_failure(
        &mut self,
        input: RecordStepFailureInput,
    ) -> Result<WorkflowStepFailureReceipt, WorkflowAppError> {
        self.record_step_failure_event(RecordStepFailureEventInput {
            run_id: input.run_id,
            step_id: input.step_id,
            attempt: input.attempt,
            error_class: input.error_class,
            occurred_at_epoch_seconds: input.occurred_at_epoch_seconds,
            retry_policy: input.retry_policy,
            failure_kind: WorkflowEventKind::StepFailed,
        })
    }

    pub fn record_step_timeout(
        &mut self,
        input: RecordStepTimeoutInput,
    ) -> Result<WorkflowStepFailureReceipt, WorkflowAppError> {
        if input.timeout_seconds == 0 {
            return Err(WorkflowAppError::InvalidRetryPolicy);
        }
        let run_id =
            WorkflowRunId::new(input.run_id.clone()).map_err(|_| WorkflowAppError::InvalidId)?;
        let step_id =
            WorkflowStepId::new(input.step_id.clone()).map_err(|_| WorkflowAppError::InvalidId)?;
        let start_key = WorkflowStepAttemptKey {
            run_id,
            step_id,
            attempt: input.attempt,
        };
        let started = self
            .step_attempt_starts
            .get(&start_key)
            .ok_or(WorkflowAppError::StepAttemptNotStarted)?;
        if started.started_at_epoch_seconds != input.started_at_epoch_seconds {
            return Err(WorkflowAppError::StepAttemptStartMismatch);
        }
        let deadline = started
            .started_at_epoch_seconds
            .saturating_add(input.timeout_seconds);
        if input.occurred_at_epoch_seconds < deadline {
            return Err(WorkflowAppError::StepTimeoutNotElapsed);
        }
        self.record_step_failure_event(RecordStepFailureEventInput {
            run_id: input.run_id,
            step_id: input.step_id,
            attempt: input.attempt,
            error_class: "StepTimeout".to_string(),
            occurred_at_epoch_seconds: input.occurred_at_epoch_seconds,
            retry_policy: input.retry_policy,
            failure_kind: WorkflowEventKind::StepTimedOut,
        })
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
                WorkflowRunState::Cancelled => bucket.runs_cancelled += 1,
                _ => {}
            }
        }
        Ok(event)
    }

    pub fn cancel_run(
        &mut self,
        input: CancelRunInput,
    ) -> Result<WorkflowRunCancellationReceipt, WorkflowAppError> {
        if input.reason_ref.trim().is_empty() {
            return Err(WorkflowAppError::EmptyCancellationReasonRef);
        }
        let run_id = WorkflowRunId::new(input.run_id).map_err(|_| WorkflowAppError::InvalidId)?;
        let event = self.complete_run(
            &run_id,
            WorkflowRunState::Cancelled,
            input.occurred_at_epoch_seconds,
        )?;
        Ok(WorkflowRunCancellationReceipt {
            run_id,
            reason_ref: input.reason_ref,
            event,
            schema_version: 1,
        })
    }

    pub fn restore_run_with_receipts(
        &mut self,
        snapshot: WorkflowRunSnapshot,
        receipts: Vec<WorkflowStepExecutionReceipt>,
    ) -> Result<WorkflowRunResumeReport, WorkflowAppError> {
        self.restore_run_with_lifecycle_receipts(snapshot, receipts, Vec::new())
    }

    pub fn restore_run_with_lifecycle_receipts(
        &mut self,
        snapshot: WorkflowRunSnapshot,
        step_receipts: Vec<WorkflowStepExecutionReceipt>,
        failure_receipts: Vec<WorkflowStepFailureReceipt>,
    ) -> Result<WorkflowRunResumeReport, WorkflowAppError> {
        for receipt in &step_receipts {
            validate_receipt_for_snapshot(&snapshot, receipt)?;
        }
        for receipt in &failure_receipts {
            validate_failure_receipt_for_snapshot(&snapshot, receipt)?;
        }
        let run_id = snapshot.run.id.clone();
        let restored_events = snapshot.events.len();
        let restored_step_receipts = step_receipts.len();
        let restored_step_failure_receipts = failure_receipts.len();
        self.restore_snapshot(snapshot)?;
        for receipt in step_receipts {
            self.restore_step_receipt(receipt)?;
        }
        for receipt in failure_receipts {
            self.restore_step_failure_receipt(receipt)?;
        }
        Ok(WorkflowRunResumeReport {
            run_id,
            restored_events,
            restored_step_receipts,
            restored_step_failure_receipts,
            schema_version: 1,
        })
    }

    pub fn snapshot(&self, run_id: &WorkflowRunId) -> Option<WorkflowRunSnapshot> {
        self.ledger.snapshot(run_id)
    }

    pub fn restore_snapshot(
        &mut self,
        snapshot: WorkflowRunSnapshot,
    ) -> Result<(), WorkflowAppError> {
        Ok(self.ledger.restore_snapshot(snapshot)?)
    }

    pub fn restore_step_receipt(
        &mut self,
        receipt: WorkflowStepExecutionReceipt,
    ) -> Result<(), WorkflowAppError> {
        if receipt.event.run_id != receipt.run_id {
            return Err(WorkflowAppError::ReceiptRunMismatch);
        }
        if receipt.event.step_id.as_ref() != Some(&receipt.step_id) {
            return Err(WorkflowAppError::ReceiptStepMismatch);
        }
        if receipt.idempotency_key.trim().is_empty() {
            return Err(WorkflowAppError::ReceiptIdempotencyMismatch);
        }
        if receipt.side_effect_ref.trim().is_empty() {
            return Err(WorkflowAppError::EmptySideEffectRef);
        }
        if receipt.event.kind != WorkflowEventKind::StepCompleted {
            return Err(WorkflowAppError::ReceiptEventKindMismatch);
        }
        self.validate_step_receipt_against_ledger(&receipt)?;
        let key = WorkflowStepExecutionKey {
            run_id: receipt.run_id.clone(),
            step_id: receipt.step_id.clone(),
            idempotency_key: receipt.idempotency_key.clone(),
        };
        self.step_receipts.insert(key, receipt);
        Ok(())
    }

    pub fn restore_step_failure_receipt(
        &mut self,
        receipt: WorkflowStepFailureReceipt,
    ) -> Result<(), WorkflowAppError> {
        self.validate_failure_receipt_against_ledger(&receipt)?;
        let key = WorkflowStepFailureKey {
            run_id: receipt.run_id.clone(),
            step_id: receipt.step_id.clone(),
            attempt: receipt.attempt,
            failure_kind: receipt.failure_event.kind,
        };
        self.step_failure_receipts.insert(key, receipt);
        Ok(())
    }

    pub fn step_failure_receipt(
        &self,
        run_id: &WorkflowRunId,
        step_id: &WorkflowStepId,
        attempt: u32,
        failure_kind: WorkflowEventKind,
    ) -> Option<&WorkflowStepFailureReceipt> {
        self.step_failure_receipts.get(&WorkflowStepFailureKey {
            run_id: run_id.clone(),
            step_id: step_id.clone(),
            attempt,
            failure_kind,
        })
    }

    pub fn counters(&self, tenant_id: &str) -> WorkflowSloCounters {
        self.slo.get(tenant_id).cloned().unwrap_or_default()
    }

    fn record_step_failure_event(
        &mut self,
        input: RecordStepFailureEventInput,
    ) -> Result<WorkflowStepFailureReceipt, WorkflowAppError> {
        if input.attempt == 0 {
            return Err(WorkflowAppError::InvalidAttempt);
        }
        if input.error_class.trim().is_empty() {
            return Err(WorkflowAppError::EmptyErrorClass);
        }
        let run_id = WorkflowRunId::new(input.run_id).map_err(|_| WorkflowAppError::InvalidId)?;
        let step_id =
            WorkflowStepId::new(input.step_id).map_err(|_| WorkflowAppError::InvalidId)?;
        let retry_plan = retry_plan(
            input.retry_policy.as_ref(),
            input.attempt,
            &input.error_class,
            input.occurred_at_epoch_seconds,
        )?;
        let failure_event = self.record_step(
            &run_id,
            &step_id,
            input.failure_kind,
            input.occurred_at_epoch_seconds,
        )?;
        self.increment_counter_for_run(&run_id, |counters| match input.failure_kind {
            WorkflowEventKind::StepTimedOut => counters.step_timeouts += 1,
            _ => counters.step_failures += 1,
        });

        let retry_event = if retry_plan.decision == WorkflowStepRetryDecision::RetryScheduled {
            self.increment_counter_for_run(&run_id, |counters| {
                counters.step_retries_scheduled += 1
            });
            Some(self.record_step(
                &run_id,
                &step_id,
                WorkflowEventKind::StepRetryScheduled,
                input.occurred_at_epoch_seconds,
            )?)
        } else {
            None
        };

        let receipt = WorkflowStepFailureReceipt {
            run_id: run_id.clone(),
            step_id: step_id.clone(),
            attempt: input.attempt,
            error_class: input.error_class,
            decision: retry_plan.decision,
            next_attempt: retry_plan.next_attempt,
            next_retry_at_epoch_seconds: retry_plan.next_retry_at_epoch_seconds,
            failure_event,
            retry_event,
            schema_version: 1,
        };
        let key = WorkflowStepFailureKey {
            run_id,
            step_id,
            attempt: receipt.attempt,
            failure_kind: receipt.failure_event.kind,
        };
        self.step_failure_receipts.insert(key, receipt.clone());
        Ok(receipt)
    }

    fn increment_counter_for_run(
        &mut self,
        run_id: &WorkflowRunId,
        update: impl FnOnce(&mut WorkflowSloCounters),
    ) {
        if let Some(run) = self.ledger.runs().find(|run| run.id == *run_id).cloned() {
            update(self.slo.entry(run.tenant_id).or_default());
        }
    }

    fn validate_step_receipt_against_ledger(
        &self,
        receipt: &WorkflowStepExecutionReceipt,
    ) -> Result<(), WorkflowAppError> {
        let run = self
            .ledger
            .runs()
            .find(|run| run.id == receipt.run_id)
            .ok_or(WorkflowAppError::Domain(WorkflowDomainError::UnknownRun))?;
        let definition = self
            .ledger
            .definitions()
            .find(|definition| definition.id == run.definition_id)
            .ok_or(WorkflowAppError::Domain(
                WorkflowDomainError::UnknownDefinition,
            ))?;
        if definition.step(&receipt.step_id).is_none() {
            return Err(WorkflowAppError::Domain(WorkflowDomainError::UnknownStep));
        }
        Ok(())
    }

    fn validate_failure_receipt_against_ledger(
        &self,
        receipt: &WorkflowStepFailureReceipt,
    ) -> Result<(), WorkflowAppError> {
        let run = self
            .ledger
            .runs()
            .find(|run| run.id == receipt.run_id)
            .ok_or(WorkflowAppError::Domain(WorkflowDomainError::UnknownRun))?;
        let definition = self
            .ledger
            .definitions()
            .find(|definition| definition.id == run.definition_id)
            .ok_or(WorkflowAppError::Domain(
                WorkflowDomainError::UnknownDefinition,
            ))?;
        if definition.step(&receipt.step_id).is_none() {
            return Err(WorkflowAppError::Domain(WorkflowDomainError::UnknownStep));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RecordStepFailureEventInput {
    run_id: String,
    step_id: String,
    attempt: u32,
    error_class: String,
    occurred_at_epoch_seconds: u64,
    retry_policy: Option<WorkflowStepRetryPolicy>,
    failure_kind: WorkflowEventKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkflowRetryPlan {
    decision: WorkflowStepRetryDecision,
    next_attempt: Option<u32>,
    next_retry_at_epoch_seconds: Option<u64>,
}

fn validate_receipt_for_snapshot(
    snapshot: &WorkflowRunSnapshot,
    receipt: &WorkflowStepExecutionReceipt,
) -> Result<(), WorkflowAppError> {
    if receipt.run_id != snapshot.run.id {
        return Err(WorkflowAppError::ReceiptRunMismatch);
    }
    if snapshot.definition.step(&receipt.step_id).is_none() {
        return Err(WorkflowAppError::ReceiptStepMismatch);
    }
    if receipt.event.run_id != receipt.run_id
        || receipt.event.step_id.as_ref() != Some(&receipt.step_id)
    {
        return Err(WorkflowAppError::ReceiptStepMismatch);
    }
    if receipt.idempotency_key.trim().is_empty() {
        return Err(WorkflowAppError::ReceiptIdempotencyMismatch);
    }
    if receipt.side_effect_ref.trim().is_empty() {
        return Err(WorkflowAppError::EmptySideEffectRef);
    }
    if receipt.event.kind != WorkflowEventKind::StepCompleted {
        return Err(WorkflowAppError::ReceiptEventKindMismatch);
    }
    Ok(())
}

fn validate_failure_receipt_for_snapshot(
    snapshot: &WorkflowRunSnapshot,
    receipt: &WorkflowStepFailureReceipt,
) -> Result<(), WorkflowAppError> {
    if receipt.run_id != snapshot.run.id {
        return Err(WorkflowAppError::ReceiptRunMismatch);
    }
    if snapshot.definition.step(&receipt.step_id).is_none() {
        return Err(WorkflowAppError::ReceiptStepMismatch);
    }
    if receipt.attempt == 0 {
        return Err(WorkflowAppError::InvalidAttempt);
    }
    if receipt.error_class.trim().is_empty() {
        return Err(WorkflowAppError::EmptyErrorClass);
    }
    let failure_kind = receipt.failure_event.kind;
    if !matches!(
        failure_kind,
        WorkflowEventKind::StepFailed | WorkflowEventKind::StepTimedOut
    ) {
        return Err(WorkflowAppError::ReceiptEventKindMismatch);
    }
    if receipt.failure_event.run_id != receipt.run_id
        || receipt.failure_event.step_id.as_ref() != Some(&receipt.step_id)
    {
        return Err(WorkflowAppError::ReceiptStepMismatch);
    }
    match receipt.decision {
        WorkflowStepRetryDecision::RetryScheduled => {
            let retry_event = receipt
                .retry_event
                .as_ref()
                .ok_or(WorkflowAppError::ReceiptEventKindMismatch)?;
            if retry_event.kind != WorkflowEventKind::StepRetryScheduled {
                return Err(WorkflowAppError::ReceiptEventKindMismatch);
            }
            if retry_event.run_id != receipt.run_id
                || retry_event.step_id.as_ref() != Some(&receipt.step_id)
                || receipt.next_attempt.is_none()
                || receipt.next_retry_at_epoch_seconds.is_none()
            {
                return Err(WorkflowAppError::ReceiptStepMismatch);
            }
        }
        WorkflowStepRetryDecision::NoRetryPolicy | WorkflowStepRetryDecision::AttemptsExhausted => {
            if receipt.retry_event.is_some()
                || receipt.next_attempt.is_some()
                || receipt.next_retry_at_epoch_seconds.is_some()
            {
                return Err(WorkflowAppError::ReceiptEventKindMismatch);
            }
        }
    }
    Ok(())
}

fn retry_plan(
    policy: Option<&WorkflowStepRetryPolicy>,
    attempt: u32,
    error_class: &str,
    occurred_at_epoch_seconds: u64,
) -> Result<WorkflowRetryPlan, WorkflowAppError> {
    let Some(policy) = policy else {
        return Ok(WorkflowRetryPlan {
            decision: WorkflowStepRetryDecision::NoRetryPolicy,
            next_attempt: None,
            next_retry_at_epoch_seconds: None,
        });
    };
    validate_retry_policy(policy)?;
    if !should_retry_error(policy, error_class) || attempt >= policy.max_attempts {
        return Ok(WorkflowRetryPlan {
            decision: WorkflowStepRetryDecision::AttemptsExhausted,
            next_attempt: None,
            next_retry_at_epoch_seconds: None,
        });
    }
    let delay = retry_delay_seconds(policy, attempt);
    Ok(WorkflowRetryPlan {
        decision: WorkflowStepRetryDecision::RetryScheduled,
        next_attempt: Some(attempt + 1),
        next_retry_at_epoch_seconds: Some(occurred_at_epoch_seconds.saturating_add(delay)),
    })
}

fn validate_retry_policy(policy: &WorkflowStepRetryPolicy) -> Result<(), WorkflowAppError> {
    if policy.max_attempts == 0
        || policy.max_attempts > MAX_RETRY_ATTEMPTS
        || policy.initial_delay_seconds == 0
        || policy.max_delay_seconds == 0
        || policy.backoff_multiplier == 0
        || policy.max_delay_seconds < policy.initial_delay_seconds
    {
        return Err(WorkflowAppError::InvalidRetryPolicy);
    }
    Ok(())
}

fn should_retry_error(policy: &WorkflowStepRetryPolicy, error_class: &str) -> bool {
    let error_class = error_class.trim();
    if policy
        .non_retryable_error_classes
        .iter()
        .any(|candidate| candidate == error_class)
    {
        return false;
    }
    policy.retryable_error_classes.is_empty()
        || policy
            .retryable_error_classes
            .iter()
            .any(|candidate| candidate == error_class)
}

fn retry_delay_seconds(policy: &WorkflowStepRetryPolicy, attempt: u32) -> u64 {
    let mut delay = policy.initial_delay_seconds;
    for _ in 1..attempt {
        delay = delay.saturating_mul(u64::from(policy.backoff_multiplier));
        if delay >= policy.max_delay_seconds {
            return policy.max_delay_seconds;
        }
    }
    delay.min(policy.max_delay_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn publish_fixture(definition_id: &str) -> PublishDefinitionInput {
        PublishDefinitionInput {
            definition_id: definition_id.to_string(),
            tenant_id: "ten_acme".to_string(),
            regional_pack: "oya-pack-alpha".to_string(),
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
    fn execute_step_once_returns_same_receipt_for_duplicate_idempotency_key() {
        let mut engine = WorkflowEngine::default();
        engine.publish(publish_fixture("wfd_idem")).unwrap();
        engine
            .start_run(StartRunInput {
                run_id: "wfr_idem".to_string(),
                definition_id: "wfd_idem".to_string(),
                started_at_epoch_seconds: 1_700_000_100,
            })
            .unwrap();
        let first = engine
            .execute_step_once(ExecuteStepOnceInput {
                run_id: "wfr_idem".to_string(),
                step_id: "wfs_extract".to_string(),
                idempotency_key: "idem-step-1".to_string(),
                side_effect_ref: "audit://workflow/wfr_idem/wfs_extract/1".to_string(),
                occurred_at_epoch_seconds: 1_700_000_110,
            })
            .unwrap();
        let duplicate = engine
            .execute_step_once(ExecuteStepOnceInput {
                run_id: "wfr_idem".to_string(),
                step_id: "wfs_extract".to_string(),
                idempotency_key: "idem-step-1".to_string(),
                side_effect_ref: "audit://workflow/wfr_idem/wfs_extract/retry".to_string(),
                occurred_at_epoch_seconds: 1_700_000_120,
            })
            .unwrap();
        assert_eq!(first, duplicate);
    }

    #[test]
    fn restore_snapshot_and_receipt_prevents_duplicate_side_effect_after_restart() {
        let mut engine = WorkflowEngine::default();
        engine.publish(publish_fixture("wfd_restart")).unwrap();
        engine
            .start_run(StartRunInput {
                run_id: "wfr_restart".to_string(),
                definition_id: "wfd_restart".to_string(),
                started_at_epoch_seconds: 1_700_000_100,
            })
            .unwrap();
        let receipt = engine
            .execute_step_once(ExecuteStepOnceInput {
                run_id: "wfr_restart".to_string(),
                step_id: "wfs_extract".to_string(),
                idempotency_key: "idem-restart-1".to_string(),
                side_effect_ref: "audit://workflow/wfr_restart/wfs_extract/1".to_string(),
                occurred_at_epoch_seconds: 1_700_000_110,
            })
            .unwrap();
        let snapshot = engine
            .snapshot(&WorkflowRunId::new("wfr_restart").unwrap())
            .unwrap();
        let mut restarted = WorkflowEngine::default();
        restarted.restore_snapshot(snapshot).unwrap();
        restarted.restore_step_receipt(receipt.clone()).unwrap();
        let duplicate = restarted
            .execute_step_once(ExecuteStepOnceInput {
                run_id: "wfr_restart".to_string(),
                step_id: "wfs_extract".to_string(),
                idempotency_key: "idem-restart-1".to_string(),
                side_effect_ref: "audit://workflow/wfr_restart/wfs_extract/retry".to_string(),
                occurred_at_epoch_seconds: 1_700_000_120,
            })
            .unwrap();
        assert_eq!(duplicate, receipt);
    }

    fn retry_policy() -> WorkflowStepRetryPolicy {
        WorkflowStepRetryPolicy::new(3, 10, 30, 3).unwrap()
    }

    fn publish_and_start(engine: &mut WorkflowEngine, definition_id: &str, run_id: &str) {
        engine.publish(publish_fixture(definition_id)).unwrap();
        engine
            .start_run(StartRunInput {
                run_id: run_id.to_string(),
                definition_id: definition_id.to_string(),
                started_at_epoch_seconds: 1_700_000_100,
            })
            .unwrap();
    }

    #[test]
    fn workflow_engine_schedules_retry_with_capped_backoff_for_retryable_failure() {
        let mut engine = WorkflowEngine::default();
        publish_and_start(&mut engine, "wfd_retry", "wfr_retry");

        let receipt = engine
            .record_step_failure(RecordStepFailureInput {
                run_id: "wfr_retry".to_string(),
                step_id: "wfs_extract".to_string(),
                attempt: 2,
                error_class: "TransientHttp503".to_string(),
                occurred_at_epoch_seconds: 1_700_000_120,
                retry_policy: Some(retry_policy()),
            })
            .unwrap();

        assert_eq!(receipt.failure_event.kind, WorkflowEventKind::StepFailed);
        assert_eq!(
            receipt.retry_event.unwrap().kind,
            WorkflowEventKind::StepRetryScheduled
        );
        assert_eq!(receipt.decision, WorkflowStepRetryDecision::RetryScheduled);
        assert_eq!(receipt.next_attempt, Some(3));
        assert_eq!(receipt.next_retry_at_epoch_seconds, Some(1_700_000_150));
        assert_eq!(engine.counters("ten_acme").step_retries_scheduled, 1);
    }

    #[test]
    fn workflow_engine_exhausts_non_retryable_failure_without_retry_event() {
        let mut engine = WorkflowEngine::default();
        publish_and_start(&mut engine, "wfd_no_retry", "wfr_no_retry");
        let policy = retry_policy().with_non_retryable_error("PermanentValidationError");

        let receipt = engine
            .record_step_failure(RecordStepFailureInput {
                run_id: "wfr_no_retry".to_string(),
                step_id: "wfs_extract".to_string(),
                attempt: 1,
                error_class: "PermanentValidationError".to_string(),
                occurred_at_epoch_seconds: 1_700_000_120,
                retry_policy: Some(policy),
            })
            .unwrap();

        assert_eq!(
            receipt.decision,
            WorkflowStepRetryDecision::AttemptsExhausted
        );
        assert!(receipt.retry_event.is_none());
        assert_eq!(engine.counters("ten_acme").step_failures, 1);
    }

    #[test]
    fn workflow_engine_records_timeout_only_after_deadline_and_can_retry() {
        let mut engine = WorkflowEngine::default();
        publish_and_start(&mut engine, "wfd_timeout", "wfr_timeout");

        let missing_start = engine
            .record_step_timeout(RecordStepTimeoutInput {
                run_id: "wfr_timeout".to_string(),
                step_id: "wfs_extract".to_string(),
                attempt: 1,
                started_at_epoch_seconds: 1_700_000_120,
                timeout_seconds: 30,
                occurred_at_epoch_seconds: 1_700_000_150,
                retry_policy: Some(retry_policy()),
            })
            .expect_err("timeout requires a recorded step-attempt start");
        assert_eq!(missing_start, WorkflowAppError::StepAttemptNotStarted);

        let started = engine
            .start_step_attempt(StartStepAttemptInput {
                run_id: "wfr_timeout".to_string(),
                step_id: "wfs_extract".to_string(),
                attempt: 1,
                started_at_epoch_seconds: 1_700_000_120,
            })
            .unwrap();
        assert_eq!(started.event.kind, WorkflowEventKind::StepStarted);

        let early = engine
            .record_step_timeout(RecordStepTimeoutInput {
                run_id: "wfr_timeout".to_string(),
                step_id: "wfs_extract".to_string(),
                attempt: 1,
                started_at_epoch_seconds: 1_700_000_120,
                timeout_seconds: 30,
                occurred_at_epoch_seconds: 1_700_000_149,
                retry_policy: Some(retry_policy()),
            })
            .expect_err("deadline has not elapsed");
        assert_eq!(early, WorkflowAppError::StepTimeoutNotElapsed);

        let receipt = engine
            .record_step_timeout(RecordStepTimeoutInput {
                run_id: "wfr_timeout".to_string(),
                step_id: "wfs_extract".to_string(),
                attempt: 1,
                started_at_epoch_seconds: 1_700_000_120,
                timeout_seconds: 30,
                occurred_at_epoch_seconds: 1_700_000_150,
                retry_policy: Some(retry_policy()),
            })
            .unwrap();

        assert_eq!(receipt.failure_event.kind, WorkflowEventKind::StepTimedOut);
        assert_eq!(receipt.decision, WorkflowStepRetryDecision::RetryScheduled);
        assert_eq!(receipt.next_attempt, Some(2));
        assert_eq!(receipt.next_retry_at_epoch_seconds, Some(1_700_000_160));
        assert_eq!(engine.counters("ten_acme").step_timeouts, 1);
    }

    #[test]
    fn workflow_engine_cancel_run_emits_receipt_and_blocks_later_steps() {
        let mut engine = WorkflowEngine::default();
        publish_and_start(&mut engine, "wfd_cancel", "wfr_cancel");

        let receipt = engine
            .cancel_run(CancelRunInput {
                run_id: "wfr_cancel".to_string(),
                occurred_at_epoch_seconds: 1_700_000_130,
                reason_ref: "audit://workflow/wfr_cancel/cancel-request".to_string(),
            })
            .unwrap();

        assert_eq!(receipt.event.kind, WorkflowEventKind::RunCancelled);
        assert_eq!(
            receipt.reason_ref,
            "audit://workflow/wfr_cancel/cancel-request"
        );
        assert_eq!(engine.counters("ten_acme").runs_cancelled, 1);
        let later = engine
            .execute_step_once(ExecuteStepOnceInput {
                run_id: "wfr_cancel".to_string(),
                step_id: "wfs_extract".to_string(),
                idempotency_key: "idem-cancelled".to_string(),
                side_effect_ref: "audit://workflow/wfr_cancel/wfs_extract/after-cancel".to_string(),
                occurred_at_epoch_seconds: 1_700_000_140,
            })
            .expect_err("cancelled run is terminal");
        assert_eq!(
            later,
            WorkflowAppError::Domain(WorkflowDomainError::RunNotRunning)
        );
    }

    #[test]
    fn workflow_engine_restores_snapshot_and_receipts_as_single_resume_report() {
        let mut engine = WorkflowEngine::default();
        publish_and_start(&mut engine, "wfd_resume", "wfr_resume");
        let receipt = engine
            .execute_step_once(ExecuteStepOnceInput {
                run_id: "wfr_resume".to_string(),
                step_id: "wfs_extract".to_string(),
                idempotency_key: "idem-resume".to_string(),
                side_effect_ref: "audit://workflow/wfr_resume/wfs_extract/1".to_string(),
                occurred_at_epoch_seconds: 1_700_000_120,
            })
            .unwrap();
        let snapshot = engine
            .snapshot(&WorkflowRunId::new("wfr_resume").unwrap())
            .unwrap();

        let mut restarted = WorkflowEngine::default();
        let report = restarted
            .restore_run_with_receipts(snapshot, vec![receipt.clone()])
            .unwrap();

        assert_eq!(report.run_id, WorkflowRunId::new("wfr_resume").unwrap());
        assert_eq!(report.restored_events, 2);
        assert_eq!(report.restored_step_receipts, 1);
        assert_eq!(report.restored_step_failure_receipts, 0);
        let duplicate = restarted
            .execute_step_once(ExecuteStepOnceInput {
                run_id: "wfr_resume".to_string(),
                step_id: "wfs_extract".to_string(),
                idempotency_key: "idem-resume".to_string(),
                side_effect_ref: "audit://workflow/wfr_resume/wfs_extract/retry".to_string(),
                occurred_at_epoch_seconds: 1_700_000_130,
            })
            .unwrap();
        assert_eq!(duplicate, receipt);
    }

    #[test]
    fn workflow_engine_restores_retry_schedule_receipts_for_resume() {
        let mut engine = WorkflowEngine::default();
        publish_and_start(&mut engine, "wfd_retry_resume", "wfr_retry_resume");
        let receipt = engine
            .record_step_failure(RecordStepFailureInput {
                run_id: "wfr_retry_resume".to_string(),
                step_id: "wfs_extract".to_string(),
                attempt: 2,
                error_class: "TransientHttp503".to_string(),
                occurred_at_epoch_seconds: 1_700_000_120,
                retry_policy: Some(retry_policy()),
            })
            .unwrap();
        let snapshot = engine
            .snapshot(&WorkflowRunId::new("wfr_retry_resume").unwrap())
            .unwrap();

        let mut restarted = WorkflowEngine::default();
        let report = restarted
            .restore_run_with_lifecycle_receipts(snapshot, Vec::new(), vec![receipt.clone()])
            .unwrap();

        assert_eq!(report.restored_events, 3);
        assert_eq!(report.restored_step_failure_receipts, 1);
        let restored = restarted
            .step_failure_receipt(
                &WorkflowRunId::new("wfr_retry_resume").unwrap(),
                &WorkflowStepId::new("wfs_extract").unwrap(),
                2,
                WorkflowEventKind::StepFailed,
            )
            .expect("retry receipt restored");
        assert_eq!(restored, &receipt);
    }

    #[test]
    fn workflow_engine_restore_step_receipt_rejects_missing_run() {
        let mut engine = WorkflowEngine::default();
        publish_and_start(&mut engine, "wfd_poison", "wfr_poison");
        let receipt = engine
            .execute_step_once(ExecuteStepOnceInput {
                run_id: "wfr_poison".to_string(),
                step_id: "wfs_extract".to_string(),
                idempotency_key: "idem-poison".to_string(),
                side_effect_ref: "audit://workflow/wfr_poison/wfs_extract/1".to_string(),
                occurred_at_epoch_seconds: 1_700_000_120,
            })
            .unwrap();

        let mut empty_engine = WorkflowEngine::default();
        let error = empty_engine
            .restore_step_receipt(receipt)
            .expect_err("receipt cannot be restored before run snapshot");

        assert_eq!(
            error,
            WorkflowAppError::Domain(WorkflowDomainError::UnknownRun)
        );
    }

    #[test]
    fn workflow_retry_policy_rejects_unbounded_attempt_counts() {
        let error = WorkflowStepRetryPolicy::new(MAX_RETRY_ATTEMPTS + 1, 1, 60, 2)
            .expect_err("retry budget must be bounded");

        assert_eq!(error, WorkflowAppError::InvalidRetryPolicy);
    }

    #[test]
    fn workflow_engine_restore_rejects_non_completed_receipt_before_mutation() {
        let mut engine = WorkflowEngine::default();
        publish_and_start(&mut engine, "wfd_bad_resume", "wfr_bad_resume");
        let mut receipt = engine
            .execute_step_once(ExecuteStepOnceInput {
                run_id: "wfr_bad_resume".to_string(),
                step_id: "wfs_extract".to_string(),
                idempotency_key: "idem-bad-resume".to_string(),
                side_effect_ref: "audit://workflow/wfr_bad_resume/wfs_extract/1".to_string(),
                occurred_at_epoch_seconds: 1_700_000_120,
            })
            .unwrap();
        let snapshot = engine
            .snapshot(&WorkflowRunId::new("wfr_bad_resume").unwrap())
            .unwrap();
        receipt.event.kind = WorkflowEventKind::StepFailed;

        let mut restarted = WorkflowEngine::default();
        let error = restarted
            .restore_run_with_receipts(snapshot.clone(), vec![receipt.clone()])
            .expect_err("non-completed receipt rejected before snapshot restore");

        assert_eq!(error, WorkflowAppError::ReceiptEventKindMismatch);
        restarted
            .restore_run_with_receipts(snapshot, Vec::new())
            .unwrap();
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
