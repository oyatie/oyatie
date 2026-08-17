//! Workflow-engine state-machine Postgres adapter foundation.
//!
//! This crate provides source-level, plan-only Postgres checkpoint semantics for
//! the state-machine usecase. It defines tenant-scoped load SQL, optimistic
//! append SQL, row mapping, and append-result mapping without opening database
//! connections, executing SQL, performing network I/O, or claiming durable
//! runtime behavior.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_state_machine_kernel::{StateCheckpoint, StepStatus, WorkflowRunStatus};
pub use workflow_state_machine_usecase::{
    StateCheckpointAppendFailure, StateCheckpointStoreFailure, StateCheckpointStorePort,
};

pub const POSTGRES_CHECKPOINT_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS workflow_state_checkpoints (
  tenant_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  spec_id TEXT NOT NULL,
  version_sha TEXT NOT NULL,
  checkpoint_seq BIGINT NOT NULL,
  run_status TEXT NOT NULL,
  current_step_index BIGINT NULL,
  step_status TEXT NULL,
  last_event_id TEXT NOT NULL,
  last_event_type TEXT NOT NULL,
  evidence_refs TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, run_id, checkpoint_seq)
)
"#;

pub const POSTGRES_LOAD_CURRENT_SQL: &str = r#"
SELECT tenant_id, run_id, spec_id, version_sha, checkpoint_seq, run_status,
       current_step_index, step_status, last_event_id, last_event_type,
       evidence_refs
FROM workflow_state_checkpoints
WHERE tenant_id = $1 AND run_id = $2
ORDER BY checkpoint_seq DESC
LIMIT 1
"#;

pub const POSTGRES_APPEND_CHECKPOINT_SQL: &str = r#"
INSERT INTO workflow_state_checkpoints (
  tenant_id, run_id, spec_id, version_sha, checkpoint_seq, run_status,
  current_step_index, step_status, last_event_id, last_event_type, evidence_refs
)
SELECT $1, $2, $3, $4, $5, $6, NULLIF($7, '')::BIGINT, NULLIF($8, ''), $9, $10, $11
WHERE COALESCE((
  SELECT MAX(checkpoint_seq)
  FROM workflow_state_checkpoints
  WHERE tenant_id = $1 AND run_id = $2
), 0) = $12
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresCheckpointQueryPlan {
    pub statement_name: String,       // data_class: INTERNAL_ONLY
    pub sql: String,                  // data_class: INTERNAL_ONLY
    pub params: Vec<String>,          // data_class: INTERNAL_ONLY
    pub expected_checkpoint_seq: u64, // data_class: INTERNAL_ONLY
}

pub type PostgresSqlPlan = PostgresCheckpointQueryPlan;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PostgresCheckpointPlanError {
    InvalidStatus,
    UnsafeMetadata,
}

#[derive(Clone, Eq, PartialEq)]
pub struct PostgresCheckpointRow {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub spec_id: String,                 // data_class: INTERNAL_ONLY
    pub version_sha: String,             // data_class: INTERNAL_ONLY
    pub checkpoint_seq: u64,             // data_class: INTERNAL_ONLY
    pub run_status: String,              // data_class: PUBLIC
    pub current_step_index: Option<u32>, // data_class: INTERNAL_ONLY
    pub step_status: Option<String>,     // data_class: PUBLIC
    pub last_event_id: String,           // data_class: INTERNAL_ONLY
    pub last_event_type: String,         // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

impl std::fmt::Debug for PostgresCheckpointRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresCheckpointRow")
            .field("checkpoint_seq", &self.checkpoint_seq)
            .field("run_status", &self.run_status)
            .field("current_step_index", &self.current_step_index)
            .field("step_status", &self.step_status)
            .field("evidence_ref_count", &self.evidence_refs.len())
            .finish()
    }
}

impl PostgresCheckpointRow {
    pub fn from_checkpoint(checkpoint: &StateCheckpoint) -> Self {
        Self {
            tenant_id: checkpoint.tenant_id.clone(),
            run_id: checkpoint.run_id.clone(),
            spec_id: checkpoint.spec_id.clone(),
            version_sha: checkpoint.version_sha.clone(),
            checkpoint_seq: checkpoint.checkpoint_seq,
            run_status: checkpoint.run_status.as_wire().to_owned(),
            current_step_index: checkpoint.current_step_index,
            step_status: checkpoint
                .step_status
                .map(|status| status.as_wire().to_owned()),
            last_event_id: checkpoint.last_event_id.clone(),
            last_event_type: checkpoint.last_event_type.clone(),
            evidence_refs: sorted_unique(checkpoint.evidence_refs.clone()),
        }
    }

    pub fn to_checkpoint(&self) -> Result<StateCheckpoint, PostgresCheckpointPlanError> {
        validate_row_metadata(self)?;
        Ok(StateCheckpoint {
            tenant_id: self.tenant_id.clone(),
            run_id: self.run_id.clone(),
            spec_id: self.spec_id.clone(),
            version_sha: self.version_sha.clone(),
            checkpoint_seq: self.checkpoint_seq,
            run_status: WorkflowRunStatus::from_wire(&self.run_status)
                .ok_or(PostgresCheckpointPlanError::InvalidStatus)?,
            current_step_index: self.current_step_index,
            step_status: match &self.step_status {
                Some(value) => Some(
                    StepStatus::from_wire(value)
                        .ok_or(PostgresCheckpointPlanError::InvalidStatus)?,
                ),
                None => None,
            },
            last_event_id: self.last_event_id.clone(),
            last_event_type: self.last_event_type.clone(),
            evidence_refs: sorted_unique(self.evidence_refs.clone()),
        })
    }
}

#[derive(Default)]
pub struct PostgresStateCheckpointAdapter {
    generated_plans: Vec<PostgresCheckpointQueryPlan>,
}

impl PostgresStateCheckpointAdapter {
    pub fn load_current_plan(
        tenant_id: &str,
        run_id: &str,
    ) -> Result<PostgresCheckpointQueryPlan, PostgresCheckpointPlanError> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(run_id) {
            return Err(PostgresCheckpointPlanError::UnsafeMetadata);
        }
        Ok(PostgresCheckpointQueryPlan {
            statement_name: "workflow_state_checkpoints_load_current".to_owned(),
            sql: POSTGRES_LOAD_CURRENT_SQL.to_owned(),
            params: vec![tenant_id.to_owned(), run_id.to_owned()],
            expected_checkpoint_seq: 0,
        })
    }

    pub fn append_plan(
        expected_checkpoint_seq: u64,
        checkpoint: &StateCheckpoint,
    ) -> Result<PostgresCheckpointQueryPlan, PostgresCheckpointPlanError> {
        validate_checkpoint_metadata(checkpoint)?;
        let expected_previous = expected_checkpoint_seq.saturating_sub(1);
        let row = PostgresCheckpointRow::from_checkpoint(checkpoint);
        Ok(PostgresCheckpointQueryPlan {
            statement_name: "workflow_state_checkpoints_append_expected_seq".to_owned(),
            sql: POSTGRES_APPEND_CHECKPOINT_SQL.to_owned(),
            params: vec![
                row.tenant_id,
                row.run_id,
                row.spec_id,
                row.version_sha,
                row.checkpoint_seq.to_string(),
                row.run_status,
                row.current_step_index
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                row.step_status.unwrap_or_default(),
                row.last_event_id,
                row.last_event_type,
                row.evidence_refs.join("|"),
                expected_previous.to_string(),
            ],
            expected_checkpoint_seq,
        })
    }

    pub fn map_append_result(
        expected_checkpoint_seq: u64,
        affected_rows: u64,
        evidence_ref: &str,
    ) -> Result<(), StateCheckpointAppendFailure> {
        let safe_evidence = if is_safe_ref(evidence_ref) {
            evidence_ref.to_owned()
        } else {
            "workflow-state-machine-postgres-adapter:append-result".to_owned()
        };
        match affected_rows {
            1 => Ok(()),
            0 => Err(StateCheckpointAppendFailure::Conflict {
                expected_checkpoint_seq,
                observed_checkpoint_seq: expected_checkpoint_seq.saturating_sub(1),
                evidence_ref: safe_evidence,
            }),
            _ => Err(StateCheckpointAppendFailure::Unavailable {
                evidence_ref: safe_evidence,
            }),
        }
    }

    pub fn generated_plans(&self) -> &[PostgresCheckpointQueryPlan] {
        &self.generated_plans
    }
}

impl StateCheckpointStorePort for PostgresStateCheckpointAdapter {
    fn load_current(
        &mut self,
        tenant_id: &str,
        run_id: &str,
    ) -> Result<Option<StateCheckpoint>, StateCheckpointStoreFailure> {
        match Self::load_current_plan(tenant_id, run_id) {
            Ok(plan) => self.generated_plans.push(plan),
            Err(_) => {
                return Err(StateCheckpointStoreFailure::Unavailable {
                    evidence_ref: "workflow-state-machine-postgres-adapter:plan-invalid-load"
                        .to_owned(),
                });
            }
        }
        Err(StateCheckpointStoreFailure::Unavailable {
            evidence_ref: "workflow-state-machine-postgres-adapter:plan-only-load".to_owned(),
        })
    }

    fn append_checkpoint(
        &mut self,
        expected_checkpoint_seq: u64,
        checkpoint: StateCheckpoint,
    ) -> Result<(), StateCheckpointAppendFailure> {
        match Self::append_plan(expected_checkpoint_seq, &checkpoint) {
            Ok(plan) => self.generated_plans.push(plan),
            Err(_) => {
                return Err(StateCheckpointAppendFailure::Unavailable {
                    evidence_ref: "workflow-state-machine-postgres-adapter:plan-invalid-append"
                        .to_owned(),
                });
            }
        }
        Err(StateCheckpointAppendFailure::Unavailable {
            evidence_ref: "workflow-state-machine-postgres-adapter:plan-only-append".to_owned(),
        })
    }
}

fn validate_checkpoint_metadata(
    checkpoint: &StateCheckpoint,
) -> Result<(), PostgresCheckpointPlanError> {
    if is_safe_tenant(&checkpoint.tenant_id)
        && is_safe_ref(&checkpoint.run_id)
        && is_safe_ref(&checkpoint.spec_id)
        && is_safe_ref(&checkpoint.version_sha)
        && is_safe_ref(&checkpoint.last_event_id)
        && is_safe_metadata(&checkpoint.last_event_type)
        && checkpoint
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(PostgresCheckpointPlanError::UnsafeMetadata)
    }
}

fn validate_row_metadata(row: &PostgresCheckpointRow) -> Result<(), PostgresCheckpointPlanError> {
    if is_safe_tenant(&row.tenant_id)
        && is_safe_ref(&row.run_id)
        && is_safe_ref(&row.spec_id)
        && is_safe_ref(&row.version_sha)
        && is_safe_ref(&row.last_event_id)
        && is_safe_metadata(&row.last_event_type)
        && row.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(PostgresCheckpointPlanError::UnsafeMetadata)
    }
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(trimmed)
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

    fn checkpoint(seq: u64) -> StateCheckpoint {
        StateCheckpoint {
            tenant_id: "ten_a".to_owned(),
            run_id: "run:workflow:pg:1".to_owned(),
            spec_id: "workflow-spec:invoice-approval".to_owned(),
            version_sha: "sha256:spec-v1".to_owned(),
            checkpoint_seq: seq,
            run_status: WorkflowRunStatus::Running,
            current_step_index: Some(0),
            step_status: Some(StepStatus::Running),
            last_event_id: format!("evt:pg:{seq}"),
            last_event_type: "StepStarted".to_owned(),
            evidence_refs: vec!["workflow-event:pg".to_owned()],
        }
    }

    #[test]
    fn sql_templates_include_tenant_predicate_and_unique_checkpoint_key() {
        assert!(POSTGRES_CHECKPOINT_DDL.contains("UNIQUE (tenant_id, run_id, checkpoint_seq)"));
        assert!(POSTGRES_LOAD_CURRENT_SQL.contains("WHERE tenant_id = $1 AND run_id = $2"));
        assert!(POSTGRES_APPEND_CHECKPOINT_SQL.contains("WHERE tenant_id = $1 AND run_id = $2"));
        assert!(POSTGRES_APPEND_CHECKPOINT_SQL.contains("= $12"));
        assert!(POSTGRES_APPEND_CHECKPOINT_SQL.contains("tenant_id"));
    }

    #[test]
    fn append_plan_uses_one_insert_and_expected_sequence_metadata() {
        let plan = PostgresStateCheckpointAdapter::append_plan(2, &checkpoint(2)).unwrap();

        assert_eq!(
            plan.statement_name,
            "workflow_state_checkpoints_append_expected_seq"
        );
        assert_eq!(plan.expected_checkpoint_seq, 2);
        assert_eq!(plan.params.len(), 12);
        assert_eq!(plan.params[0], "ten_a");
        assert_eq!(plan.params[11], "1");
        assert!(plan.sql.contains("INSERT INTO workflow_state_checkpoints"));
        assert!(!plan.sql.contains("UPDATE"));
    }

    #[test]
    fn load_plan_requires_tenant_and_run_predicate() {
        let plan = PostgresStateCheckpointAdapter::load_current_plan("ten_a", "run:workflow:pg:1")
            .unwrap();

        assert_eq!(
            plan.statement_name,
            "workflow_state_checkpoints_load_current"
        );
        assert_eq!(
            plan.params,
            vec!["ten_a".to_owned(), "run:workflow:pg:1".to_owned()]
        );
        assert!(plan.sql.contains("WHERE tenant_id = $1 AND run_id = $2"));
        assert!(plan.sql.contains("ORDER BY checkpoint_seq DESC"));
        assert!(plan.sql.contains("LIMIT 1"));
    }

    #[test]
    fn unsafe_identifiers_and_secret_shaped_refs_are_rejected_before_sql_plan() {
        let mut unsafe_checkpoint = checkpoint(1);
        unsafe_checkpoint.run_id = "run raw prompt Authorization: Bearer sk-test".to_owned();

        let err = PostgresStateCheckpointAdapter::append_plan(1, &unsafe_checkpoint).unwrap_err();

        assert_eq!(err, PostgresCheckpointPlanError::UnsafeMetadata);
        let rendered = format!("{err:?}").to_ascii_lowercase();
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw prompt"));
    }

    #[test]
    fn row_mapping_round_trips_checkpoint_without_raw_payload_fields() {
        let row = PostgresCheckpointRow::from_checkpoint(&checkpoint(7));
        let mapped = row.to_checkpoint().unwrap();

        assert_eq!(mapped, checkpoint(7));
        assert!(!format!("{row:?}").to_ascii_lowercase().contains("payload"));
        assert!(!format!("{row:?}").to_ascii_lowercase().contains("secret"));
    }

    #[test]
    fn affected_rows_zero_maps_to_append_conflict() {
        let failure = PostgresStateCheckpointAdapter::map_append_result(2, 0, "pg:append:conflict")
            .unwrap_err();

        assert_eq!(
            failure,
            StateCheckpointAppendFailure::Conflict {
                expected_checkpoint_seq: 2,
                observed_checkpoint_seq: 1,
                evidence_ref: "pg:append:conflict".to_owned(),
            }
        );
    }

    #[test]
    fn affected_rows_one_is_success_and_more_than_one_is_unavailable() {
        assert_eq!(
            PostgresStateCheckpointAdapter::map_append_result(2, 1, "pg:append:ok"),
            Ok(())
        );
        assert_eq!(
            PostgresStateCheckpointAdapter::map_append_result(2, 2, "pg:append:too-many")
                .unwrap_err(),
            StateCheckpointAppendFailure::Unavailable {
                evidence_ref: "pg:append:too-many".to_owned(),
            }
        );
    }

    #[test]
    fn port_impl_is_plan_only_and_never_claims_database_execution() {
        let mut adapter = PostgresStateCheckpointAdapter::default();

        assert_eq!(
            adapter.load_current("ten_a", "run:workflow:pg:1"),
            Err(StateCheckpointStoreFailure::Unavailable {
                evidence_ref: "workflow-state-machine-postgres-adapter:plan-only-load".to_owned(),
            })
        );
        assert_eq!(
            adapter.append_checkpoint(1, checkpoint(1)),
            Err(StateCheckpointAppendFailure::Unavailable {
                evidence_ref: "workflow-state-machine-postgres-adapter:plan-only-append".to_owned(),
            })
        );
        assert_eq!(adapter.generated_plans().len(), 2);
    }
}
