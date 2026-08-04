//! HR employment Postgres/RLS storage adapter contract.
//!
//! This crate is the HR-owned durable-storage seam for app-layer HR metadata
//! envelopes. It declares source-level Postgres DDL/RLS/rollback SQL and builds
//! tenant-scoped idempotency reservation/commit plans from the tested HR storage
//! record shape. It does not open database connections, run migrations, retrieve
//! sensitive HR data, execute Workflow, call Payroll, emit audit-chain events,
//! or deploy cloud I/O.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

const HR_POSTGRES_ADAPTER_LABEL: &str = "postgres-hr-rls-contract";

pub const POSTGRES_HR_EMPLOYMENT_ENVELOPE_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS hr_employment_idempotency_reservations (
  tenant_id TEXT NOT NULL,
  legal_entity_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  reserved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, legal_entity_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS hr_employment_envelopes (
  tenant_id TEXT NOT NULL,
  legal_entity_id TEXT NOT NULL,
  record_kind TEXT NOT NULL,
  topic TEXT NOT NULL,
  primary_ref TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  payload_data_class TEXT NOT NULL,
  evidence_ref_count BIGINT NOT NULL,
  schema_version BIGINT NOT NULL,
  committed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, legal_entity_id, idempotency_key)
);
"#;

pub const POSTGRES_HR_EMPLOYMENT_ENVELOPE_RLS_SQL: &str = r#"
ALTER TABLE hr_employment_idempotency_reservations ENABLE ROW LEVEL SECURITY;
ALTER TABLE hr_employment_idempotency_reservations FORCE ROW LEVEL SECURITY;
CREATE POLICY hr_employment_idempotency_reservations_tenant_isolation
  ON hr_employment_idempotency_reservations
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE hr_employment_envelopes ENABLE ROW LEVEL SECURITY;
ALTER TABLE hr_employment_envelopes FORCE ROW LEVEL SECURITY;
CREATE POLICY hr_employment_envelopes_tenant_isolation
  ON hr_employment_envelopes
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
"#;

pub const POSTGRES_HR_EMPLOYMENT_ENVELOPE_ROLLBACK_SQL: &str = r#"
DROP TABLE IF EXISTS hr_employment_envelopes;
DROP TABLE IF EXISTS hr_employment_idempotency_reservations;
"#;

pub const POSTGRES_RESERVE_HR_IDEMPOTENCY_KEY_SQL: &str = r#"
INSERT INTO hr_employment_idempotency_reservations (
  tenant_id, legal_entity_id, idempotency_key
)
VALUES ($1, $2, $3)
ON CONFLICT (tenant_id, legal_entity_id, idempotency_key) DO NOTHING
"#;

pub const POSTGRES_COMMIT_HR_ENVELOPE_SQL: &str = r#"
INSERT INTO hr_employment_envelopes (
  tenant_id, legal_entity_id, record_kind, topic, primary_ref,
  idempotency_key, payload_data_class, evidence_ref_count, schema_version
)
SELECT $1, $2, $3, $4, $5, $6, $7, $8::BIGINT, $9::BIGINT
WHERE EXISTS (
  SELECT 1
  FROM hr_employment_idempotency_reservations
  WHERE tenant_id = $1
    AND legal_entity_id = $2
    AND idempotency_key = $6
)
ON CONFLICT (tenant_id, legal_entity_id, idempotency_key) DO NOTHING
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrPostgresStorageCapabilities {
    pub adapter: String,                           // data_class: PUBLIC
    pub durable_backend_contract_declared: bool,   // data_class: PUBLIC
    pub postgres_rls_contract_declared: bool,      // data_class: PUBLIC
    pub runtime_database_execution_attached: bool, // data_class: PUBLIC
    pub sensitive_data_retrieval_attached: bool,   // data_class: PUBLIC
    pub workflow_execution_attached: bool,         // data_class: PUBLIC
    pub payroll_network_call_attached: bool,       // data_class: PUBLIC
    pub audit_chain_emission_attached: bool,       // data_class: PUBLIC
    pub cloud_io_attached: bool,                   // data_class: PUBLIC
    pub schema_version: u32,                       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrPostgresQueryPlan {
    pub statement_name: String,           // data_class: INTERNAL_ONLY
    pub sql: String,                      // data_class: INTERNAL_ONLY
    pub params: Vec<String>,              // data_class: INTERNAL_ONLY
    pub expected_idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HrPostgresStoredRecordKind {
    LifecycleAudit,
    LaborWorkflowDispatch,
    LeavePayrollImpact,
    SensitiveReadPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrPostgresStoredRecord {
    pub kind: HrPostgresStoredRecordKind, // data_class: INTERNAL_ONLY
    pub topic: String,                    // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,          // data_class: INTERNAL_ONLY
    pub primary_ref: String,              // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub payload_data_class: String,       // data_class: INTERNAL_ONLY
    pub evidence_ref_count: usize,        // data_class: INTERNAL_ONLY
    pub schema_version: u32,              // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum HrPostgresPlanError {
    MissingTenantScope,
    TenantMismatch,
    UnsafeMetadata,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HrPostgresStorageStore {
    generated_plans: Vec<HrPostgresQueryPlan>,
}

impl HrPostgresStorageStore {
    pub fn reserve_idempotency_key_plan(
        &mut self,
        tenant_id: &str,
        legal_entity_id: &str,
        idempotency_key: &str,
    ) -> Result<HrPostgresQueryPlan, HrPostgresPlanError> {
        let plan =
            Self::reserve_idempotency_key_plan_static(tenant_id, legal_entity_id, idempotency_key)?;
        self.generated_plans.push(plan.clone());
        Ok(plan)
    }

    pub fn reserve_idempotency_key_plan_static(
        tenant_id: &str,
        legal_entity_id: &str,
        idempotency_key: &str,
    ) -> Result<HrPostgresQueryPlan, HrPostgresPlanError> {
        validate_scope(tenant_id, legal_entity_id, idempotency_key)?;
        Ok(HrPostgresQueryPlan {
            statement_name: "hr_postgres_reserve_idempotency_key".to_owned(),
            sql: POSTGRES_RESERVE_HR_IDEMPOTENCY_KEY_SQL.to_owned(),
            params: vec![
                tenant_id.to_owned(),
                legal_entity_id.to_owned(),
                idempotency_key.to_owned(),
            ],
            expected_idempotency_key: idempotency_key.to_owned(),
        })
    }

    pub fn commit_record_plan(
        tenant_scope: &str,
        record: &HrPostgresStoredRecord,
    ) -> Result<HrPostgresQueryPlan, HrPostgresPlanError> {
        if tenant_scope.trim().is_empty() {
            return Err(HrPostgresPlanError::MissingTenantScope);
        }
        if tenant_scope != record.tenant_id {
            return Err(HrPostgresPlanError::TenantMismatch);
        }
        validate_scope(
            &record.tenant_id,
            &record.legal_entity_id,
            &record.idempotency_key,
        )?;
        validate_metadata(&record.topic)?;
        validate_metadata(&record.primary_ref)?;
        validate_metadata(&record.payload_data_class)?;

        Ok(HrPostgresQueryPlan {
            statement_name: "hr_postgres_commit_envelope_after_reservation".to_owned(),
            sql: POSTGRES_COMMIT_HR_ENVELOPE_SQL.to_owned(),
            params: vec![
                record.tenant_id.clone(),
                record.legal_entity_id.clone(),
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
    pub fn generated_plans(&self) -> &[HrPostgresQueryPlan] {
        &self.generated_plans
    }
}

#[must_use]
pub fn hr_postgres_storage_capabilities() -> HrPostgresStorageCapabilities {
    HrPostgresStorageCapabilities {
        adapter: HR_POSTGRES_ADAPTER_LABEL.to_owned(),
        durable_backend_contract_declared: true,
        postgres_rls_contract_declared: true,
        runtime_database_execution_attached: false,
        sensitive_data_retrieval_attached: false,
        workflow_execution_attached: false,
        payroll_network_call_attached: false,
        audit_chain_emission_attached: false,
        cloud_io_attached: false,
        schema_version: 1,
    }
}

fn wire_kind(kind: HrPostgresStoredRecordKind) -> &'static str {
    match kind {
        HrPostgresStoredRecordKind::LifecycleAudit => "lifecycle_audit",
        HrPostgresStoredRecordKind::LaborWorkflowDispatch => "labor_workflow_dispatch",
        HrPostgresStoredRecordKind::LeavePayrollImpact => "leave_payroll_impact",
        HrPostgresStoredRecordKind::SensitiveReadPolicy => "sensitive_read_policy",
    }
}

fn validate_scope(
    tenant_id: &str,
    legal_entity_id: &str,
    idempotency_key: &str,
) -> Result<(), HrPostgresPlanError> {
    validate_metadata(tenant_id)?;
    validate_metadata(legal_entity_id)?;
    validate_metadata(idempotency_key)?;
    Ok(())
}

fn validate_metadata(value: &str) -> Result<(), HrPostgresPlanError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.contains("..")
        || value.chars().any(|ch| !is_metadata_char(ch))
    {
        return Err(HrPostgresPlanError::UnsafeMetadata);
    }
    Ok(())
}

fn is_metadata_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':')
}
