//! Payroll run Postgres/RLS storage adapter contract.
//!
//! This crate is the Payroll-owned durable-storage seam for app-layer payroll
//! envelopes. It declares source-level Postgres DDL/RLS/rollback SQL and builds
//! tenant-scoped idempotency reservation/commit plans from the in-memory
//! reference record shape. It does not open database connections, run
//! migrations, execute Workflow, call HR or Accounting, emit audit-chain events,
//! calculate payroll, file with regulators, disburse funds, or deploy cloud I/O.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

const PAYROLL_POSTGRES_ADAPTER_LABEL: &str = "postgres-payroll-rls-contract";

pub const POSTGRES_PAYROLL_RUN_ENVELOPE_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS payroll_run_idempotency_reservations (
  tenant_id TEXT NOT NULL,
  legal_entity_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  reserved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, legal_entity_id, run_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS payroll_run_envelopes (
  tenant_id TEXT NOT NULL,
  legal_entity_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  record_kind TEXT NOT NULL,
  topic TEXT NOT NULL,
  primary_ref TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  payload_data_class TEXT NOT NULL,
  evidence_ref_count BIGINT NOT NULL,
  schema_version BIGINT NOT NULL,
  committed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, legal_entity_id, run_id, idempotency_key)
);
"#;

pub const POSTGRES_PAYROLL_RUN_ENVELOPE_RLS_SQL: &str = r#"
ALTER TABLE payroll_run_idempotency_reservations ENABLE ROW LEVEL SECURITY;
ALTER TABLE payroll_run_idempotency_reservations FORCE ROW LEVEL SECURITY;
CREATE POLICY payroll_run_idempotency_reservations_tenant_isolation
  ON payroll_run_idempotency_reservations
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE payroll_run_envelopes ENABLE ROW LEVEL SECURITY;
ALTER TABLE payroll_run_envelopes FORCE ROW LEVEL SECURITY;
CREATE POLICY payroll_run_envelopes_tenant_isolation
  ON payroll_run_envelopes
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
"#;

pub const POSTGRES_PAYROLL_RUN_ENVELOPE_ROLLBACK_SQL: &str = r#"
DROP TABLE IF EXISTS payroll_run_envelopes;
DROP TABLE IF EXISTS payroll_run_idempotency_reservations;
"#;

pub const POSTGRES_RESERVE_IDEMPOTENCY_KEY_SQL: &str = r#"
INSERT INTO payroll_run_idempotency_reservations (
  tenant_id, legal_entity_id, run_id, idempotency_key
)
VALUES ($1, $2, $3, $4)
ON CONFLICT (tenant_id, legal_entity_id, run_id, idempotency_key) DO NOTHING
"#;

pub const POSTGRES_COMMIT_PAYROLL_ENVELOPE_SQL: &str = r#"
INSERT INTO payroll_run_envelopes (
  tenant_id, legal_entity_id, run_id, record_kind, topic, primary_ref,
  idempotency_key, payload_data_class, evidence_ref_count, schema_version
)
SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9::BIGINT, $10::BIGINT
WHERE EXISTS (
  SELECT 1
  FROM payroll_run_idempotency_reservations
  WHERE tenant_id = $1
    AND legal_entity_id = $2
    AND run_id = $3
    AND idempotency_key = $7
)
ON CONFLICT (tenant_id, legal_entity_id, run_id, idempotency_key) DO NOTHING
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollPostgresStorageCapabilities {
    pub adapter: String,                           // data_class: PUBLIC
    pub durable_backend_contract_declared: bool,   // data_class: PUBLIC
    pub postgres_rls_contract_declared: bool,      // data_class: PUBLIC
    pub runtime_database_execution_attached: bool, // data_class: PUBLIC
    pub payroll_calculation_attached: bool,        // data_class: PUBLIC
    pub statutory_filing_rails_attached: bool,     // data_class: PUBLIC
    pub disbursement_rails_attached: bool,         // data_class: PUBLIC
    pub workflow_dispatch_attached: bool,          // data_class: PUBLIC
    pub hr_network_call_attached: bool,            // data_class: PUBLIC
    pub accounting_network_call_attached: bool,    // data_class: PUBLIC
    pub audit_chain_emission_attached: bool,       // data_class: PUBLIC
    pub schema_version: u32,                       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollPostgresQueryPlan {
    pub statement_name: String,           // data_class: INTERNAL_ONLY
    pub sql: String,                      // data_class: INTERNAL_ONLY
    pub params: Vec<String>,              // data_class: INTERNAL_ONLY
    pub expected_idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PayrollPostgresStoredRecordKind {
    TrialCloseAudit,
    AccountingJournalDispatch,
    HrLeaveImpactIntake,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollPostgresStoredRecord {
    pub kind: PayrollPostgresStoredRecordKind, // data_class: INTERNAL_ONLY
    pub topic: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,               // data_class: INTERNAL_ONLY
    pub run_id: String,                        // data_class: INTERNAL_ONLY
    pub primary_ref: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub payload_data_class: String,            // data_class: INTERNAL_ONLY
    pub evidence_ref_count: usize,             // data_class: INTERNAL_ONLY
    pub schema_version: u32,                   // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayrollPostgresPlanError {
    MissingTenantScope,
    TenantMismatch,
    UnsafeMetadata,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PayrollPostgresRunStore {
    generated_plans: Vec<PayrollPostgresQueryPlan>,
}

impl PayrollPostgresRunStore {
    pub fn reserve_idempotency_key_plan(
        &mut self,
        tenant_id: &str,
        legal_entity_id: &str,
        run_id: &str,
        idempotency_key: &str,
    ) -> Result<PayrollPostgresQueryPlan, PayrollPostgresPlanError> {
        let plan = Self::reserve_idempotency_key_plan_static(
            tenant_id,
            legal_entity_id,
            run_id,
            idempotency_key,
        )?;
        self.generated_plans.push(plan.clone());
        Ok(plan)
    }

    pub fn reserve_idempotency_key_plan_static(
        tenant_id: &str,
        legal_entity_id: &str,
        run_id: &str,
        idempotency_key: &str,
    ) -> Result<PayrollPostgresQueryPlan, PayrollPostgresPlanError> {
        validate_scope(tenant_id, legal_entity_id, run_id, idempotency_key)?;
        Ok(PayrollPostgresQueryPlan {
            statement_name: "payroll_postgres_reserve_idempotency_key".to_owned(),
            sql: POSTGRES_RESERVE_IDEMPOTENCY_KEY_SQL.to_owned(),
            params: vec![
                tenant_id.to_owned(),
                legal_entity_id.to_owned(),
                run_id.to_owned(),
                idempotency_key.to_owned(),
            ],
            expected_idempotency_key: idempotency_key.to_owned(),
        })
    }

    pub fn commit_record_plan(
        tenant_scope: &str,
        record: &PayrollPostgresStoredRecord,
    ) -> Result<PayrollPostgresQueryPlan, PayrollPostgresPlanError> {
        if tenant_scope.trim().is_empty() {
            return Err(PayrollPostgresPlanError::MissingTenantScope);
        }
        if tenant_scope != record.tenant_id {
            return Err(PayrollPostgresPlanError::TenantMismatch);
        }
        validate_scope(
            &record.tenant_id,
            &record.legal_entity_id,
            &record.run_id,
            &record.idempotency_key,
        )?;
        validate_metadata(&record.topic)?;
        validate_metadata(&record.primary_ref)?;
        validate_metadata(&record.payload_data_class)?;

        Ok(PayrollPostgresQueryPlan {
            statement_name: "payroll_postgres_commit_envelope_after_reservation".to_owned(),
            sql: POSTGRES_COMMIT_PAYROLL_ENVELOPE_SQL.to_owned(),
            params: vec![
                record.tenant_id.clone(),
                record.legal_entity_id.clone(),
                record.run_id.clone(),
                wire_kind(record.kind).to_owned(),
                record.topic.clone(),
                record.primary_ref.clone(),
                record.idempotency_key.clone(),
                record.payload_data_class.clone(),
                record.evidence_ref_count.to_string(),
                record.schema_version.to_string(),
            ],
            expected_idempotency_key: record.idempotency_key.clone(),
        })
    }

    #[must_use]
    pub fn generated_plans(&self) -> &[PayrollPostgresQueryPlan] {
        &self.generated_plans
    }
}

#[must_use]
pub fn payroll_postgres_storage_capabilities() -> PayrollPostgresStorageCapabilities {
    PayrollPostgresStorageCapabilities {
        adapter: PAYROLL_POSTGRES_ADAPTER_LABEL.to_owned(),
        durable_backend_contract_declared: true,
        postgres_rls_contract_declared: true,
        runtime_database_execution_attached: false,
        payroll_calculation_attached: false,
        statutory_filing_rails_attached: false,
        disbursement_rails_attached: false,
        workflow_dispatch_attached: false,
        hr_network_call_attached: false,
        accounting_network_call_attached: false,
        audit_chain_emission_attached: false,
        schema_version: 1,
    }
}

fn wire_kind(kind: PayrollPostgresStoredRecordKind) -> &'static str {
    match kind {
        PayrollPostgresStoredRecordKind::TrialCloseAudit => "trial_close_audit",
        PayrollPostgresStoredRecordKind::AccountingJournalDispatch => "accounting_journal_dispatch",
        PayrollPostgresStoredRecordKind::HrLeaveImpactIntake => "hr_leave_impact_intake",
    }
}

fn validate_scope(
    tenant_id: &str,
    legal_entity_id: &str,
    run_id: &str,
    idempotency_key: &str,
) -> Result<(), PayrollPostgresPlanError> {
    validate_metadata(tenant_id)?;
    validate_metadata(legal_entity_id)?;
    validate_metadata(run_id)?;
    validate_metadata(idempotency_key)
}

fn validate_metadata(value: &str) -> Result<(), PayrollPostgresPlanError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.contains("..")
        || value.chars().any(|value| {
            !(value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | ':' | '/' | '.'))
        })
    {
        return Err(PayrollPostgresPlanError::UnsafeMetadata);
    }
    Ok(())
}
