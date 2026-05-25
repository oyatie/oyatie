//! Workflow-engine execution-engine Postgres adapter foundation.
//!
//! This crate provides source-level, plan-only Postgres semantics for the
//! execution-engine usecase ports. It defines tenant-scoped run, step,
//! dispatcher-outbox, and SLA-timer SQL plans plus redaction-safe row mapping
//! without opening database connections, executing SQL, performing network I/O,
//! filesystem access, queue processing, signing, or claiming durable runtime
//! behavior.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_execution_engine_usecase::{
    ExecutionDispatchError, ExecutionDomainCommandKind, ExecutionEngineKernelError,
    ExecutionStoreError, RetryAttempt, RetryPolicyEvaluator, SlaTimer, SlaTimerStore,
    StepDispatcher, StepExecution, StepExecutionStatus, WorkflowExecutionStatus, WorkflowRun,
    WorkflowRunStore,
};

pub const POSTGRES_EXECUTION_ENGINE_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS workflow_execution_runs (
  tenant_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  spec_id TEXT NOT NULL,
  version_sha TEXT NOT NULL,
  active_cell_id TEXT NOT NULL,
  status TEXT NOT NULL,
  version BIGINT NOT NULL,
  current_step_index BIGINT NULL,
  evidence_refs TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, run_id)
);

CREATE TABLE IF NOT EXISTS workflow_execution_steps (
  tenant_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  step_id TEXT NOT NULL,
  step_index BIGINT NOT NULL,
  attempt BIGINT NOT NULL,
  status TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  lease_owner_ref TEXT NULL,
  side_effect_ref TEXT NULL,
  last_error_ref TEXT NULL,
  evidence_refs TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, run_id, step_id, attempt)
);

CREATE TABLE IF NOT EXISTS workflow_execution_dispatch_outbox (
  tenant_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  step_index BIGINT NOT NULL,
  evidence_ref TEXT NOT NULL,
  enqueued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, run_id, step_index, evidence_ref)
);

CREATE TABLE IF NOT EXISTS workflow_execution_sla_timers (
  tenant_id TEXT NOT NULL,
  timer_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  step_index BIGINT NULL,
  armed_at_epoch_seconds BIGINT NOT NULL,
  deadline_epoch_seconds BIGINT NOT NULL,
  evidence_refs TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, timer_id)
);
"#;

pub const POSTGRES_EXECUTION_LOAD_RUN_SQL: &str = r#"
SELECT tenant_id, run_id, spec_id, version_sha, active_cell_id, status,
       version, current_step_index, evidence_refs
FROM workflow_execution_runs
WHERE tenant_id = $1 AND run_id = $2
LIMIT 1
"#;

pub const POSTGRES_EXECUTION_CREATE_RUN_SQL: &str = r#"
INSERT INTO workflow_execution_runs (
  tenant_id, run_id, spec_id, version_sha, active_cell_id, status,
  version, current_step_index, evidence_refs
)
VALUES ($1, $2, $3, $4, $5, $6, $7, NULLIF($8, '')::BIGINT, $9)
ON CONFLICT (tenant_id, run_id) DO NOTHING
"#;

pub const POSTGRES_EXECUTION_UPDATE_RUN_STATUS_SQL: &str = r#"
UPDATE workflow_execution_runs
SET status = $4, version = version + 1, updated_at = now()
WHERE tenant_id = $1 AND run_id = $2 AND version = $3
RETURNING version
"#;

pub const POSTGRES_EXECUTION_SAVE_STEP_SQL: &str = r#"
INSERT INTO workflow_execution_steps (
  tenant_id, run_id, step_id, step_index, attempt, status, idempotency_key,
  lease_owner_ref, side_effect_ref, last_error_ref, evidence_refs
)
VALUES ($1, $2, $3, $4, $5, $6, $7, NULLIF($8, ''), NULLIF($9, ''), NULLIF($10, ''), $11)
ON CONFLICT (tenant_id, run_id, step_id, attempt) DO UPDATE
SET status = EXCLUDED.status,
    lease_owner_ref = EXCLUDED.lease_owner_ref,
    side_effect_ref = EXCLUDED.side_effect_ref,
    last_error_ref = EXCLUDED.last_error_ref,
    evidence_refs = EXCLUDED.evidence_refs,
    updated_at = now()
"#;

pub const POSTGRES_EXECUTION_DISPATCH_STEP_SQL: &str = r#"
INSERT INTO workflow_execution_dispatch_outbox (
  tenant_id, run_id, step_index, evidence_ref
)
VALUES ($1, $2, $3, $4)
ON CONFLICT (tenant_id, run_id, step_index, evidence_ref) DO NOTHING
"#;

pub const POSTGRES_EXECUTION_ARM_TIMER_SQL: &str = r#"
INSERT INTO workflow_execution_sla_timers (
  tenant_id, timer_id, run_id, step_index, armed_at_epoch_seconds,
  deadline_epoch_seconds, evidence_refs
)
VALUES ($1, $2, $3, NULLIF($4, '')::BIGINT, $5, $6, $7)
ON CONFLICT (tenant_id, timer_id) DO UPDATE
SET run_id = EXCLUDED.run_id,
    step_index = EXCLUDED.step_index,
    armed_at_epoch_seconds = EXCLUDED.armed_at_epoch_seconds,
    deadline_epoch_seconds = EXCLUDED.deadline_epoch_seconds,
    evidence_refs = EXCLUDED.evidence_refs,
    updated_at = now()
"#;

pub const POSTGRES_EXECUTION_FIRE_EXPIRED_TIMERS_SQL: &str = r#"
SELECT tenant_id, timer_id, run_id, step_index, armed_at_epoch_seconds,
       deadline_epoch_seconds, evidence_refs
FROM workflow_execution_sla_timers
WHERE tenant_id = $1 AND deadline_epoch_seconds <= $2
ORDER BY deadline_epoch_seconds ASC, timer_id ASC
FOR UPDATE SKIP LOCKED
"#;

pub const POSTGRES_EXECUTION_CANCEL_TIMER_SQL: &str = r#"
DELETE FROM workflow_execution_sla_timers
WHERE tenant_id = $1 AND timer_id = $2
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresExecutionQueryPlan {
    pub statement_name: String,        // data_class: INTERNAL_ONLY
    pub sql: String,                   // data_class: INTERNAL_ONLY
    pub params: Vec<String>,           // data_class: INTERNAL_ONLY
    pub expected_version: Option<u64>, // data_class: INTERNAL_ONLY
}

pub type PostgresSqlPlan = PostgresExecutionQueryPlan;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PostgresExecutionPlanError {
    InvalidStatus,
    UnsafeMetadata,
}

#[derive(Clone, Eq, PartialEq)]
pub struct PostgresExecutionRunRow {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub spec_id: String,                 // data_class: INTERNAL_ONLY
    pub version_sha: String,             // data_class: INTERNAL_ONLY
    pub active_cell_id: String,          // data_class: INTERNAL_ONLY
    pub status: String,                  // data_class: PUBLIC
    pub version: u64,                    // data_class: INTERNAL_ONLY
    pub current_step_index: Option<u32>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

impl std::fmt::Debug for PostgresExecutionRunRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresExecutionRunRow")
            .field("run_id", &self.run_id)
            .field("status", &self.status)
            .field("version", &self.version)
            .field("current_step_index", &self.current_step_index)
            .field("evidence_ref_count", &self.evidence_refs.len())
            .finish()
    }
}

impl PostgresExecutionRunRow {
    pub fn from_run(run: &WorkflowRun) -> Self {
        Self {
            tenant_id: run.tenant_id.clone(),
            run_id: run.run_id.clone(),
            spec_id: run.spec_id.clone(),
            version_sha: run.version_sha.clone(),
            active_cell_id: run.active_cell_id.clone(),
            status: run.status.as_wire().to_owned(),
            version: run.version,
            current_step_index: run.current_step_index,
            evidence_refs: sorted_unique(run.evidence_refs.clone()),
        }
    }

    pub fn to_run(&self) -> Result<WorkflowRun, PostgresExecutionPlanError> {
        validate_run_row(self)?;
        Ok(WorkflowRun {
            tenant_id: self.tenant_id.clone(),
            run_id: self.run_id.clone(),
            spec_id: self.spec_id.clone(),
            version_sha: self.version_sha.clone(),
            active_cell_id: self.active_cell_id.clone(),
            status: WorkflowExecutionStatus::from_wire(&self.status)
                .ok_or(PostgresExecutionPlanError::InvalidStatus)?,
            version: self.version,
            current_step_index: self.current_step_index,
            evidence_refs: sorted_unique(self.evidence_refs.clone()),
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PostgresExecutionStepRow {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub step_id: String,                 // data_class: INTERNAL_ONLY
    pub step_index: u32,                 // data_class: INTERNAL_ONLY
    pub attempt: u32,                    // data_class: INTERNAL_ONLY
    pub status: String,                  // data_class: PUBLIC
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub lease_owner_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub side_effect_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub last_error_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

impl std::fmt::Debug for PostgresExecutionStepRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresExecutionStepRow")
            .field("step_id", &self.step_id)
            .field("step_index", &self.step_index)
            .field("attempt", &self.attempt)
            .field("status", &self.status)
            .field("evidence_ref_count", &self.evidence_refs.len())
            .finish()
    }
}

impl PostgresExecutionStepRow {
    pub fn from_step(step: &StepExecution) -> Self {
        Self {
            tenant_id: step.tenant_id.clone(),
            run_id: step.run_id.clone(),
            step_id: step.step_id.clone(),
            step_index: step.step_index,
            attempt: step.attempt,
            status: step.status.as_wire().to_owned(),
            idempotency_key: step.idempotency_key.clone(),
            lease_owner_ref: step.lease_owner_ref.clone(),
            side_effect_ref: step.side_effect_ref.clone(),
            last_error_ref: step.last_error_ref.clone(),
            evidence_refs: sorted_unique(step.evidence_refs.clone()),
        }
    }

    pub fn to_step(&self) -> Result<StepExecution, PostgresExecutionPlanError> {
        validate_step_row(self)?;
        Ok(StepExecution {
            tenant_id: self.tenant_id.clone(),
            run_id: self.run_id.clone(),
            step_id: self.step_id.clone(),
            step_index: self.step_index,
            attempt: self.attempt,
            status: StepExecutionStatus::from_wire(&self.status)
                .ok_or(PostgresExecutionPlanError::InvalidStatus)?,
            idempotency_key: self.idempotency_key.clone(),
            lease_owner_ref: self.lease_owner_ref.clone(),
            side_effect_ref: self.side_effect_ref.clone(),
            last_error_ref: self.last_error_ref.clone(),
            evidence_refs: sorted_unique(self.evidence_refs.clone()),
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PostgresExecutionTimerRow {
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub timer_id: String,            // data_class: INTERNAL_ONLY
    pub run_id: String,              // data_class: INTERNAL_ONLY
    pub step_index: Option<u32>,     // data_class: INTERNAL_ONLY
    pub armed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,  // data_class: INTERNAL_ONLY
}

impl std::fmt::Debug for PostgresExecutionTimerRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresExecutionTimerRow")
            .field("timer_id", &self.timer_id)
            .field("run_id", &self.run_id)
            .field("step_index", &self.step_index)
            .field("deadline_epoch_seconds", &self.deadline_epoch_seconds)
            .field("evidence_ref_count", &self.evidence_refs.len())
            .finish()
    }
}

impl PostgresExecutionTimerRow {
    pub fn from_timer(timer: &SlaTimer) -> Self {
        Self {
            tenant_id: timer.tenant_id.clone(),
            timer_id: timer.timer_id.clone(),
            run_id: timer.run_id.clone(),
            step_index: timer.step_index,
            armed_at_epoch_seconds: timer.armed_at_epoch_seconds,
            deadline_epoch_seconds: timer.deadline_epoch_seconds,
            evidence_refs: sorted_unique(timer.evidence_refs.clone()),
        }
    }

    pub fn to_timer(&self) -> Result<SlaTimer, PostgresExecutionPlanError> {
        validate_timer_row(self)?;
        Ok(SlaTimer {
            timer_id: self.timer_id.clone(),
            tenant_id: self.tenant_id.clone(),
            run_id: self.run_id.clone(),
            step_index: self.step_index,
            armed_at_epoch_seconds: self.armed_at_epoch_seconds,
            deadline_epoch_seconds: self.deadline_epoch_seconds,
            evidence_refs: sorted_unique(self.evidence_refs.clone()),
        })
    }
}

#[derive(Default)]
pub struct PostgresExecutionEngineAdapter {
    generated_plans: Vec<PostgresExecutionQueryPlan>,
}

impl PostgresExecutionEngineAdapter {
    pub fn load_run_plan(
        tenant_id: &str,
        run_id: &str,
    ) -> Result<PostgresExecutionQueryPlan, PostgresExecutionPlanError> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(run_id) {
            return Err(PostgresExecutionPlanError::UnsafeMetadata);
        }
        Ok(PostgresExecutionQueryPlan {
            statement_name: "workflow_execution_runs_load".to_owned(),
            sql: POSTGRES_EXECUTION_LOAD_RUN_SQL.to_owned(),
            params: vec![tenant_id.to_owned(), run_id.to_owned()],
            expected_version: None,
        })
    }

    pub fn create_run_plan(
        run: &WorkflowRun,
    ) -> Result<PostgresExecutionQueryPlan, PostgresExecutionPlanError> {
        validate_run_metadata(run)?;
        let row = PostgresExecutionRunRow::from_run(run);
        Ok(PostgresExecutionQueryPlan {
            statement_name: "workflow_execution_runs_create".to_owned(),
            sql: POSTGRES_EXECUTION_CREATE_RUN_SQL.to_owned(),
            params: vec![
                row.tenant_id,
                row.run_id,
                row.spec_id,
                row.version_sha,
                row.active_cell_id,
                row.status,
                row.version.to_string(),
                row.current_step_index
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                row.evidence_refs.join("|"),
            ],
            expected_version: Some(run.version),
        })
    }

    pub fn update_run_status_plan(
        tenant_id: &str,
        run_id: &str,
        expected_version: u64,
        status: WorkflowExecutionStatus,
        evidence_ref: &str,
    ) -> Result<PostgresExecutionQueryPlan, PostgresExecutionPlanError> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(run_id) || !is_safe_ref(evidence_ref) {
            return Err(PostgresExecutionPlanError::UnsafeMetadata);
        }
        Ok(PostgresExecutionQueryPlan {
            statement_name: "workflow_execution_runs_update_status_expected_version".to_owned(),
            sql: POSTGRES_EXECUTION_UPDATE_RUN_STATUS_SQL.to_owned(),
            params: vec![
                tenant_id.to_owned(),
                run_id.to_owned(),
                expected_version.to_string(),
                status.as_wire().to_owned(),
                evidence_ref.to_owned(),
            ],
            expected_version: Some(expected_version),
        })
    }

    pub fn save_step_plan(
        step: &StepExecution,
    ) -> Result<PostgresExecutionQueryPlan, PostgresExecutionPlanError> {
        validate_step_metadata(step)?;
        let row = PostgresExecutionStepRow::from_step(step);
        Ok(PostgresExecutionQueryPlan {
            statement_name: "workflow_execution_steps_upsert".to_owned(),
            sql: POSTGRES_EXECUTION_SAVE_STEP_SQL.to_owned(),
            params: vec![
                row.tenant_id,
                row.run_id,
                row.step_id,
                row.step_index.to_string(),
                row.attempt.to_string(),
                row.status,
                row.idempotency_key,
                row.lease_owner_ref.unwrap_or_default(),
                row.side_effect_ref.unwrap_or_default(),
                row.last_error_ref.unwrap_or_default(),
                row.evidence_refs.join("|"),
            ],
            expected_version: None,
        })
    }

    pub fn dispatch_step_plan(
        tenant_id: &str,
        run_id: &str,
        step_index: u32,
        evidence_ref: &str,
    ) -> Result<PostgresExecutionQueryPlan, PostgresExecutionPlanError> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(run_id) || !is_safe_ref(evidence_ref) {
            return Err(PostgresExecutionPlanError::UnsafeMetadata);
        }
        Ok(PostgresExecutionQueryPlan {
            statement_name: "workflow_execution_dispatch_outbox_enqueue".to_owned(),
            sql: POSTGRES_EXECUTION_DISPATCH_STEP_SQL.to_owned(),
            params: vec![
                tenant_id.to_owned(),
                run_id.to_owned(),
                step_index.to_string(),
                evidence_ref.to_owned(),
            ],
            expected_version: None,
        })
    }

    pub fn arm_timer_plan(
        timer: &SlaTimer,
    ) -> Result<PostgresExecutionQueryPlan, PostgresExecutionPlanError> {
        validate_timer_metadata(timer)?;
        let row = PostgresExecutionTimerRow::from_timer(timer);
        Ok(PostgresExecutionQueryPlan {
            statement_name: "workflow_execution_sla_timers_upsert".to_owned(),
            sql: POSTGRES_EXECUTION_ARM_TIMER_SQL.to_owned(),
            params: vec![
                row.tenant_id,
                row.timer_id,
                row.run_id,
                row.step_index
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                row.armed_at_epoch_seconds.to_string(),
                row.deadline_epoch_seconds.to_string(),
                row.evidence_refs.join("|"),
            ],
            expected_version: None,
        })
    }

    pub fn fire_expired_timers_plan(
        tenant_id: &str,
        now_epoch_seconds: u64,
    ) -> Result<PostgresExecutionQueryPlan, PostgresExecutionPlanError> {
        if !is_safe_tenant(tenant_id) {
            return Err(PostgresExecutionPlanError::UnsafeMetadata);
        }
        Ok(PostgresExecutionQueryPlan {
            statement_name: "workflow_execution_sla_timers_fire_expired".to_owned(),
            sql: POSTGRES_EXECUTION_FIRE_EXPIRED_TIMERS_SQL.to_owned(),
            params: vec![tenant_id.to_owned(), now_epoch_seconds.to_string()],
            expected_version: None,
        })
    }

    pub fn cancel_timer_plan(
        tenant_id: &str,
        timer_id: &str,
    ) -> Result<PostgresExecutionQueryPlan, PostgresExecutionPlanError> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(timer_id) {
            return Err(PostgresExecutionPlanError::UnsafeMetadata);
        }
        Ok(PostgresExecutionQueryPlan {
            statement_name: "workflow_execution_sla_timers_cancel".to_owned(),
            sql: POSTGRES_EXECUTION_CANCEL_TIMER_SQL.to_owned(),
            params: vec![tenant_id.to_owned(), timer_id.to_owned()],
            expected_version: None,
        })
    }

    pub fn map_update_result(
        expected_version: u64,
        affected_rows: u64,
        evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError> {
        let safe_evidence = safe_evidence_ref(
            evidence_ref,
            "workflow-execution-postgres-adapter:update-result",
        );
        match affected_rows {
            1 => Ok(()),
            0 => Err(ExecutionStoreError::Conflict {
                expected_version,
                observed_version: expected_version.saturating_add(1),
                evidence_ref: safe_evidence,
            }),
            _ => Err(ExecutionStoreError::Unavailable {
                evidence_ref: safe_evidence,
            }),
        }
    }

    pub fn map_single_row_result(
        affected_rows: u64,
        evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError> {
        let safe_evidence = safe_evidence_ref(
            evidence_ref,
            "workflow-execution-postgres-adapter:single-row-result",
        );
        match affected_rows {
            1 => Ok(()),
            _ => Err(ExecutionStoreError::Unavailable {
                evidence_ref: safe_evidence,
            }),
        }
    }

    pub fn map_dispatch_result(
        affected_rows: u64,
        evidence_ref: &str,
    ) -> Result<(), ExecutionDispatchError> {
        let safe_evidence = safe_evidence_ref(
            evidence_ref,
            "workflow-execution-postgres-adapter:dispatch-result",
        );
        match affected_rows {
            1 => Ok(()),
            0 => Err(ExecutionDispatchError::Denied {
                evidence_ref: safe_evidence,
            }),
            _ => Err(ExecutionDispatchError::Unavailable {
                evidence_ref: safe_evidence,
            }),
        }
    }

    pub fn generated_plans(&self) -> &[PostgresExecutionQueryPlan] {
        &self.generated_plans
    }
}

impl WorkflowRunStore for PostgresExecutionEngineAdapter {
    fn create_run(&mut self, run: WorkflowRun) -> Result<(), ExecutionStoreError> {
        match Self::create_run_plan(&run) {
            Ok(plan) => self.generated_plans.push(plan),
            Err(_) => {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "workflow-execution-postgres-adapter:plan-invalid-create-run"
                        .to_owned(),
                });
            }
        }
        Err(ExecutionStoreError::Unavailable {
            evidence_ref: "workflow-execution-postgres-adapter:plan-only-create-run".to_owned(),
        })
    }

    fn load_run(
        &self,
        tenant_id: &str,
        run_id: &str,
    ) -> Result<Option<WorkflowRun>, ExecutionStoreError> {
        Self::load_run_plan(tenant_id, run_id).map_err(|_| ExecutionStoreError::Unavailable {
            evidence_ref: "workflow-execution-postgres-adapter:plan-invalid-load-run".to_owned(),
        })?;
        Err(ExecutionStoreError::Unavailable {
            evidence_ref: "workflow-execution-postgres-adapter:plan-only-load-run".to_owned(),
        })
    }

    fn update_run_status(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        expected_version: u64,
        status: WorkflowExecutionStatus,
        evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError> {
        match Self::update_run_status_plan(
            tenant_id,
            run_id,
            expected_version,
            status,
            evidence_ref,
        ) {
            Ok(plan) => self.generated_plans.push(plan),
            Err(_) => {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "workflow-execution-postgres-adapter:plan-invalid-update-run"
                        .to_owned(),
                });
            }
        }
        Err(ExecutionStoreError::Unavailable {
            evidence_ref: "workflow-execution-postgres-adapter:plan-only-update-run".to_owned(),
        })
    }

    fn save_step(&mut self, step: StepExecution) -> Result<(), ExecutionStoreError> {
        match Self::save_step_plan(&step) {
            Ok(plan) => self.generated_plans.push(plan),
            Err(_) => {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "workflow-execution-postgres-adapter:plan-invalid-save-step"
                        .to_owned(),
                });
            }
        }
        Err(ExecutionStoreError::Unavailable {
            evidence_ref: "workflow-execution-postgres-adapter:plan-only-save-step".to_owned(),
        })
    }
}

impl StepDispatcher for PostgresExecutionEngineAdapter {
    fn dispatch_step(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        step_index: u32,
        evidence_ref: &str,
    ) -> Result<(), ExecutionDispatchError> {
        match Self::dispatch_step_plan(tenant_id, run_id, step_index, evidence_ref) {
            Ok(plan) => self.generated_plans.push(plan),
            Err(_) => {
                return Err(ExecutionDispatchError::Denied {
                    evidence_ref: "workflow-execution-postgres-adapter:plan-invalid-dispatch"
                        .to_owned(),
                });
            }
        }
        Err(ExecutionDispatchError::Unavailable {
            evidence_ref: "workflow-execution-postgres-adapter:plan-only-dispatch".to_owned(),
        })
    }
}

impl RetryPolicyEvaluator for PostgresExecutionEngineAdapter {
    fn next_delay_seconds(
        &self,
        attempt: &RetryAttempt,
    ) -> Result<Option<u64>, ExecutionEngineKernelError> {
        if is_safe_retry(attempt) {
            Ok(None)
        } else {
            Err(ExecutionEngineKernelError::UnsafeMetadata)
        }
    }
}

impl SlaTimerStore for PostgresExecutionEngineAdapter {
    fn arm_timer(&mut self, timer: SlaTimer) -> Result<(), ExecutionStoreError> {
        match Self::arm_timer_plan(&timer) {
            Ok(plan) => self.generated_plans.push(plan),
            Err(_) => {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "workflow-execution-postgres-adapter:plan-invalid-arm-timer"
                        .to_owned(),
                });
            }
        }
        Err(ExecutionStoreError::Unavailable {
            evidence_ref: "workflow-execution-postgres-adapter:plan-only-arm-timer".to_owned(),
        })
    }

    fn cancel_timer(&mut self, tenant_id: &str, timer_id: &str) -> Result<(), ExecutionStoreError> {
        match Self::cancel_timer_plan(tenant_id, timer_id) {
            Ok(plan) => self.generated_plans.push(plan),
            Err(_) => {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "workflow-execution-postgres-adapter:plan-invalid-cancel-timer"
                        .to_owned(),
                });
            }
        }
        Err(ExecutionStoreError::Unavailable {
            evidence_ref: "workflow-execution-postgres-adapter:plan-only-cancel-timer".to_owned(),
        })
    }

    fn fire_expired(
        &mut self,
        tenant_id: &str,
        now_epoch_seconds: u64,
    ) -> Result<Vec<SlaTimer>, ExecutionStoreError> {
        match Self::fire_expired_timers_plan(tenant_id, now_epoch_seconds) {
            Ok(plan) => self.generated_plans.push(plan),
            Err(_) => {
                return Err(ExecutionStoreError::Unavailable {
                    evidence_ref: "workflow-execution-postgres-adapter:plan-invalid-fire-timers"
                        .to_owned(),
                });
            }
        }
        Err(ExecutionStoreError::Unavailable {
            evidence_ref: "workflow-execution-postgres-adapter:plan-only-fire-timers".to_owned(),
        })
    }
}

fn validate_run_metadata(run: &WorkflowRun) -> Result<(), PostgresExecutionPlanError> {
    if is_safe_run(run) {
        Ok(())
    } else {
        Err(PostgresExecutionPlanError::UnsafeMetadata)
    }
}

fn validate_step_metadata(step: &StepExecution) -> Result<(), PostgresExecutionPlanError> {
    if is_safe_step(step) {
        Ok(())
    } else {
        Err(PostgresExecutionPlanError::UnsafeMetadata)
    }
}

fn validate_timer_metadata(timer: &SlaTimer) -> Result<(), PostgresExecutionPlanError> {
    if is_safe_timer(timer) {
        Ok(())
    } else {
        Err(PostgresExecutionPlanError::UnsafeMetadata)
    }
}

fn validate_run_row(row: &PostgresExecutionRunRow) -> Result<(), PostgresExecutionPlanError> {
    if is_safe_tenant(&row.tenant_id)
        && is_safe_ref(&row.run_id)
        && is_safe_ref(&row.spec_id)
        && is_safe_ref(&row.version_sha)
        && is_safe_ref(&row.active_cell_id)
        && row.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(PostgresExecutionPlanError::UnsafeMetadata)
    }
}

fn validate_step_row(row: &PostgresExecutionStepRow) -> Result<(), PostgresExecutionPlanError> {
    if is_safe_tenant(&row.tenant_id)
        && is_safe_ref(&row.run_id)
        && is_safe_ref(&row.step_id)
        && is_safe_ref(&row.idempotency_key)
        && row
            .lease_owner_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && row
            .side_effect_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && row
            .last_error_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && row.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(PostgresExecutionPlanError::UnsafeMetadata)
    }
}

fn validate_timer_row(row: &PostgresExecutionTimerRow) -> Result<(), PostgresExecutionPlanError> {
    if is_safe_ref(&row.timer_id)
        && is_safe_tenant(&row.tenant_id)
        && is_safe_ref(&row.run_id)
        && row.deadline_epoch_seconds > row.armed_at_epoch_seconds
        && row.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(PostgresExecutionPlanError::UnsafeMetadata)
    }
}

fn is_safe_run(run: &WorkflowRun) -> bool {
    is_safe_tenant(&run.tenant_id)
        && is_safe_ref(&run.run_id)
        && is_safe_ref(&run.spec_id)
        && is_safe_ref(&run.version_sha)
        && is_safe_ref(&run.active_cell_id)
        && run.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn is_safe_step(step: &StepExecution) -> bool {
    is_safe_tenant(&step.tenant_id)
        && is_safe_ref(&step.run_id)
        && is_safe_ref(&step.step_id)
        && is_safe_ref(&step.idempotency_key)
        && step
            .lease_owner_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && step
            .side_effect_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && step
            .last_error_ref
            .as_ref()
            .is_none_or(|value| is_safe_ref(value))
        && step.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn is_safe_retry(retry: &RetryAttempt) -> bool {
    is_safe_tenant(&retry.tenant_id)
        && is_safe_ref(&retry.run_id)
        && is_safe_ref(&retry.step_id)
        && is_safe_ref(&retry.error_class_ref)
        && is_safe_ref(&retry.retry_policy_ref)
        && retry.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn is_safe_timer(timer: &SlaTimer) -> bool {
    is_safe_ref(&timer.timer_id)
        && is_safe_tenant(&timer.tenant_id)
        && is_safe_ref(&timer.run_id)
        && timer.deadline_epoch_seconds > timer.armed_at_epoch_seconds
        && timer.evidence_refs.iter().all(|value| is_safe_ref(value))
}

fn safe_evidence_ref(value: &str, fallback: &str) -> String {
    if is_safe_ref(value) {
        value.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && !value.chars().any(char::is_whitespace)
        && !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("private key")
        || lower.contains("-----begin")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
        || lower.contains("payload")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_with_status(status: WorkflowExecutionStatus, version: u64) -> WorkflowRun {
        let mut run = WorkflowRun::new(
            "ten_a",
            "run:execution-pg:1",
            "workflow-spec:invoice-approval",
            "sha256:spec-v1",
            "cell:use1:a",
            vec![
                "workflow-execution-pg:run".to_owned(),
                "workflow-execution-pg:run".to_owned(),
            ],
        )
        .unwrap();
        run.status = status;
        run.version = version;
        run.current_step_index = Some(0);
        run
    }

    fn step_with_status(status: StepExecutionStatus, attempt: u32) -> StepExecution {
        let mut step = StepExecution::new(
            "ten_a",
            "run:execution-pg:1",
            "step:approve",
            0,
            attempt,
            "idempotency:step:approve:pg:1",
            vec!["workflow-execution-pg:step".to_owned()],
        )
        .unwrap();
        step.status = status;
        step.lease_owner_ref = Some("worker:execution-pg:1".to_owned());
        step
    }

    fn timer(deadline_epoch_seconds: u64) -> SlaTimer {
        SlaTimer::new(
            "timer:execution-pg:1",
            "ten_a",
            "run:execution-pg:1",
            Some(0),
            100,
            deadline_epoch_seconds,
            vec!["workflow-execution-pg:sla".to_owned()],
        )
        .unwrap()
    }

    #[test]
    fn sql_templates_include_tenant_predicates_and_unique_run_step_timer_keys() {
        assert!(POSTGRES_EXECUTION_ENGINE_DDL.contains("PRIMARY KEY (tenant_id, run_id)"));
        assert!(
            POSTGRES_EXECUTION_ENGINE_DDL
                .contains("PRIMARY KEY (tenant_id, run_id, step_id, attempt)")
        );
        assert!(POSTGRES_EXECUTION_ENGINE_DDL.contains("PRIMARY KEY (tenant_id, timer_id)"));
        assert!(POSTGRES_EXECUTION_LOAD_RUN_SQL.contains("WHERE tenant_id = $1 AND run_id = $2"));
        assert!(POSTGRES_EXECUTION_UPDATE_RUN_STATUS_SQL.contains("version = $3"));
        assert!(POSTGRES_EXECUTION_FIRE_EXPIRED_TIMERS_SQL.contains("FOR UPDATE SKIP LOCKED"));
        assert!(
            POSTGRES_EXECUTION_CREATE_RUN_SQL
                .contains("ON CONFLICT (tenant_id, run_id) DO NOTHING")
        );
    }

    #[test]
    fn create_update_step_dispatch_and_timer_plans_are_parameterized_and_plan_only() {
        let create = PostgresExecutionEngineAdapter::create_run_plan(&run_with_status(
            WorkflowExecutionStatus::Pending,
            1,
        ))
        .unwrap();
        assert_eq!(create.statement_name, "workflow_execution_runs_create");
        assert_eq!(create.params[0], "ten_a");
        assert_eq!(create.params.len(), 9);

        let update = PostgresExecutionEngineAdapter::update_run_status_plan(
            "ten_a",
            "run:execution-pg:1",
            7,
            WorkflowExecutionStatus::Running,
            "workflow-execution-pg:update",
        )
        .unwrap();
        assert_eq!(update.expected_version, Some(7));
        assert_eq!(update.params[3], "running");

        let step = PostgresExecutionEngineAdapter::save_step_plan(&step_with_status(
            StepExecutionStatus::Leased,
            1,
        ))
        .unwrap();
        assert_eq!(step.statement_name, "workflow_execution_steps_upsert");
        assert_eq!(step.params[5], "leased");

        let dispatch = PostgresExecutionEngineAdapter::dispatch_step_plan(
            "ten_a",
            "run:execution-pg:1",
            0,
            "workflow-execution-pg:dispatch",
        )
        .unwrap();
        assert_eq!(dispatch.params[2], "0");

        let timer_plan = PostgresExecutionEngineAdapter::arm_timer_plan(&timer(130)).unwrap();
        assert_eq!(timer_plan.params[5], "130");
    }

    #[test]
    fn row_mapping_round_trips_run_step_and_timer_without_payload_debug() {
        let run_row = PostgresExecutionRunRow::from_run(&run_with_status(
            WorkflowExecutionStatus::Running,
            7,
        ));
        assert_eq!(
            run_row.to_run().unwrap().status,
            WorkflowExecutionStatus::Running
        );
        assert!(
            !format!("{run_row:?}")
                .to_ascii_lowercase()
                .contains("secret")
        );

        let step_row =
            PostgresExecutionStepRow::from_step(&step_with_status(StepExecutionStatus::Leased, 1));
        assert_eq!(
            step_row.to_step().unwrap().status,
            StepExecutionStatus::Leased
        );
        assert!(
            !format!("{step_row:?}")
                .to_ascii_lowercase()
                .contains("payload")
        );

        let timer_row = PostgresExecutionTimerRow::from_timer(&timer(160));
        assert_eq!(timer_row.to_timer().unwrap().deadline_epoch_seconds, 160);
        assert!(
            !format!("{timer_row:?}")
                .to_ascii_lowercase()
                .contains("secret")
        );
    }

    #[test]
    fn unsafe_identifiers_and_secret_shaped_refs_are_rejected_before_sql_plan() {
        let mut unsafe_run = run_with_status(WorkflowExecutionStatus::Pending, 1);
        unsafe_run.run_id = "run raw prompt Authorization: Bearer sk-test".to_owned();

        let err = PostgresExecutionEngineAdapter::create_run_plan(&unsafe_run).unwrap_err();

        assert_eq!(err, PostgresExecutionPlanError::UnsafeMetadata);
        let rendered = format!("{err:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
    }

    #[test]
    fn affected_rows_mapping_distinguishes_success_conflict_and_unavailable() {
        assert_eq!(
            PostgresExecutionEngineAdapter::map_update_result(7, 1, "pg:run:update"),
            Ok(())
        );
        assert_eq!(
            PostgresExecutionEngineAdapter::map_update_result(7, 0, "pg:run:conflict").unwrap_err(),
            ExecutionStoreError::Conflict {
                expected_version: 7,
                observed_version: 8,
                evidence_ref: "pg:run:conflict".to_owned(),
            }
        );
        assert_eq!(
            PostgresExecutionEngineAdapter::map_update_result(7, 2, "pg:run:too-many").unwrap_err(),
            ExecutionStoreError::Unavailable {
                evidence_ref: "pg:run:too-many".to_owned(),
            }
        );
        assert_eq!(
            PostgresExecutionEngineAdapter::map_dispatch_result(0, "pg:dispatch:duplicate")
                .unwrap_err(),
            ExecutionDispatchError::Denied {
                evidence_ref: "pg:dispatch:duplicate".to_owned(),
            }
        );
    }

    #[test]
    fn port_impl_is_plan_only_and_records_plans_without_database_execution() {
        let mut adapter = PostgresExecutionEngineAdapter::default();

        assert_eq!(
            adapter.create_run(run_with_status(WorkflowExecutionStatus::Pending, 1)),
            Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-postgres-adapter:plan-only-create-run".to_owned(),
            })
        );
        assert_eq!(
            adapter.update_run_status(
                "ten_a",
                "run:execution-pg:1",
                1,
                WorkflowExecutionStatus::Running,
                "workflow-execution-pg:update",
            ),
            Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-postgres-adapter:plan-only-update-run".to_owned(),
            })
        );
        assert_eq!(adapter.generated_plans().len(), 2);
    }

    #[test]
    fn dispatcher_and_timer_ports_are_plan_only_and_validate_metadata() {
        let mut adapter = PostgresExecutionEngineAdapter::default();
        assert_eq!(
            adapter.dispatch_step(
                "ten_a",
                "run:execution-pg:1",
                0,
                "workflow-execution-pg:dispatch",
            ),
            Err(ExecutionDispatchError::Unavailable {
                evidence_ref: "workflow-execution-postgres-adapter:plan-only-dispatch".to_owned(),
            })
        );
        assert_eq!(
            adapter.arm_timer(timer(140)),
            Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-postgres-adapter:plan-only-arm-timer".to_owned(),
            })
        );
        assert_eq!(
            adapter.fire_expired("ten_a", 140),
            Err(ExecutionStoreError::Unavailable {
                evidence_ref: "workflow-execution-postgres-adapter:plan-only-fire-timers"
                    .to_owned(),
            })
        );
        assert_eq!(adapter.generated_plans().len(), 3);
    }
}
