#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_hr_employment_storage_adapter_postgres::{
    HrPostgresPlanError, HrPostgresStorageStore, HrPostgresStoredRecord,
    HrPostgresStoredRecordKind, POSTGRES_HR_EMPLOYMENT_ENVELOPE_DDL,
    POSTGRES_HR_EMPLOYMENT_ENVELOPE_RLS_SQL, POSTGRES_HR_EMPLOYMENT_ENVELOPE_ROLLBACK_SQL,
    hr_postgres_storage_capabilities,
};

#[test]
fn postgres_contract_declares_rls_and_rollback_without_runtime_overclaims() {
    let capabilities = hr_postgres_storage_capabilities();
    assert_eq!(capabilities.adapter, "postgres-hr-rls-contract");
    assert!(capabilities.durable_backend_contract_declared);
    assert!(capabilities.postgres_rls_contract_declared);
    assert!(!capabilities.runtime_database_execution_attached);
    assert!(!capabilities.sensitive_data_retrieval_attached);
    assert!(!capabilities.workflow_execution_attached);
    assert!(!capabilities.payroll_network_call_attached);
    assert!(!capabilities.audit_chain_emission_attached);
    assert!(!capabilities.cloud_io_attached);

    assert!(POSTGRES_HR_EMPLOYMENT_ENVELOPE_DDL.contains("hr_employment_envelopes"));
    assert!(POSTGRES_HR_EMPLOYMENT_ENVELOPE_DDL.contains("tenant_id TEXT NOT NULL"));
    assert!(POSTGRES_HR_EMPLOYMENT_ENVELOPE_DDL.contains("legal_entity_id TEXT NOT NULL"));
    assert!(POSTGRES_HR_EMPLOYMENT_ENVELOPE_DDL.contains("idempotency_key TEXT NOT NULL"));
    assert!(
        POSTGRES_HR_EMPLOYMENT_ENVELOPE_DDL
            .contains("UNIQUE (tenant_id, legal_entity_id, idempotency_key)")
    );
    assert!(POSTGRES_HR_EMPLOYMENT_ENVELOPE_DDL.contains("hr_employment_idempotency_reservations"));
    assert!(POSTGRES_HR_EMPLOYMENT_ENVELOPE_RLS_SQL.contains("ENABLE ROW LEVEL SECURITY"));
    assert!(POSTGRES_HR_EMPLOYMENT_ENVELOPE_RLS_SQL.contains("FORCE ROW LEVEL SECURITY"));
    assert!(
        POSTGRES_HR_EMPLOYMENT_ENVELOPE_RLS_SQL
            .contains("current_setting('oyatie.tenant_id', true)")
    );
    assert!(
        POSTGRES_HR_EMPLOYMENT_ENVELOPE_ROLLBACK_SQL
            .contains("DROP TABLE IF EXISTS hr_employment_envelopes")
    );
    assert!(
        POSTGRES_HR_EMPLOYMENT_ENVELOPE_ROLLBACK_SQL
            .contains("DROP TABLE IF EXISTS hr_employment_idempotency_reservations")
    );
}

#[test]
fn postgres_commit_plan_preserves_hr_envelope_metadata_and_requires_reservation() {
    let lifecycle = lifecycle_record();
    let plan = HrPostgresStorageStore::commit_record_plan("ten_acme", &lifecycle).unwrap();

    assert_eq!(
        plan.statement_name,
        "hr_postgres_commit_envelope_after_reservation"
    );
    assert!(plan.sql.contains("INSERT INTO hr_employment_envelopes"));
    assert!(plan.sql.contains("WHERE EXISTS"));
    assert!(plan.sql.contains("hr_employment_idempotency_reservations"));
    assert_eq!(plan.params[0], "ten_acme");
    assert_eq!(plan.params[1], "le_kr_001");
    assert_eq!(plan.params[2], "lifecycle_audit");
    assert_eq!(plan.params[3], "audit.hr.employment.lifecycle");
    assert_eq!(plan.params[4], "hr/employee/emp_001");
    assert_eq!(plan.params[5], "emp_001:1");
    assert_eq!(plan.params[6], "PiiIdentifying");
    assert_eq!(plan.params[7], "1");
    assert_eq!(plan.params[8], "1");
    assert_eq!(plan.expected_idempotency_key, "emp_001:1");

    let workflow_plan = HrPostgresStorageStore::commit_record_plan("ten_acme", &workflow_record())
        .expect("workflow dispatch metadata commit plan");
    assert_eq!(workflow_plan.params[2], "labor_workflow_dispatch");
    assert_eq!(workflow_plan.params[3], "workflow.hr.compliance.dispatch");
    assert_eq!(workflow_plan.params[7], "2");

    let leave_plan = HrPostgresStorageStore::commit_record_plan("ten_acme", &leave_record())
        .expect("leave payroll-impact metadata commit plan");
    assert_eq!(leave_plan.params[2], "leave_payroll_impact");
    assert_eq!(leave_plan.params[3], "integration.hr.payroll.leave-impact");
    assert_eq!(leave_plan.params[6], "Financial");

    let sensitive_plan =
        HrPostgresStorageStore::commit_record_plan("ten_acme", &sensitive_record())
            .expect("sensitive-read policy metadata commit plan");
    assert_eq!(sensitive_plan.params[2], "sensitive_read_policy");
    assert_eq!(sensitive_plan.params[3], "audit.hr.sensitive-read.policy");
    assert_eq!(sensitive_plan.params[7], "4");
}

#[test]
fn postgres_reservation_plan_and_commit_refuse_cross_tenant_or_unsafe_keys() {
    let mut store = HrPostgresStorageStore::default();
    let reservation = store
        .reserve_idempotency_key_plan("ten_acme", "le_kr_001", "emp_001:1")
        .unwrap();
    assert_eq!(
        reservation.statement_name,
        "hr_postgres_reserve_idempotency_key"
    );
    assert!(
        reservation
            .sql
            .contains("INSERT INTO hr_employment_idempotency_reservations")
    );
    assert_eq!(
        reservation.params,
        vec!["ten_acme", "le_kr_001", "emp_001:1"]
    );
    assert_eq!(store.generated_plans().len(), 1);

    let mut other_tenant_record = lifecycle_record();
    other_tenant_record.tenant_id = "ten_other".to_owned();
    assert_eq!(
        HrPostgresStorageStore::commit_record_plan("ten_acme", &other_tenant_record).unwrap_err(),
        HrPostgresPlanError::TenantMismatch
    );
    assert_eq!(
        HrPostgresStorageStore::reserve_idempotency_key_plan_static(
            "ten_acme",
            "le_kr_001",
            "../bad-key",
        )
        .unwrap_err(),
        HrPostgresPlanError::UnsafeMetadata
    );

    let mut unsafe_topic_record = lifecycle_record();
    unsafe_topic_record.topic = "audit.hr.employment;drop".to_owned();
    assert_eq!(
        HrPostgresStorageStore::commit_record_plan("ten_acme", &unsafe_topic_record).unwrap_err(),
        HrPostgresPlanError::UnsafeMetadata
    );

    let mut unsafe_primary_ref_record = lifecycle_record();
    unsafe_primary_ref_record.primary_ref = "hr/employee emp_001".to_owned();
    assert_eq!(
        HrPostgresStorageStore::commit_record_plan("ten_acme", &unsafe_primary_ref_record)
            .unwrap_err(),
        HrPostgresPlanError::UnsafeMetadata
    );

    let mut unsafe_data_class_record = lifecycle_record();
    unsafe_data_class_record.payload_data_class = "../Phi".to_owned();
    assert_eq!(
        HrPostgresStorageStore::commit_record_plan("ten_acme", &unsafe_data_class_record)
            .unwrap_err(),
        HrPostgresPlanError::UnsafeMetadata
    );
}

fn lifecycle_record() -> HrPostgresStoredRecord {
    HrPostgresStoredRecord {
        kind: HrPostgresStoredRecordKind::LifecycleAudit,
        topic: "audit.hr.employment.lifecycle".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        primary_ref: "hr/employee/emp_001".to_owned(),
        idempotency_key: "emp_001:1".to_owned(),
        payload_data_class: "PiiIdentifying".to_owned(),
        evidence_ref_count: 1,
        schema_version: 1,
    }
}

fn workflow_record() -> HrPostgresStoredRecord {
    HrPostgresStoredRecord {
        kind: HrPostgresStoredRecordKind::LaborWorkflowDispatch,
        topic: "workflow.hr.compliance.dispatch".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        primary_ref: "workflow/hr-compliance/kr".to_owned(),
        idempotency_key: "le_kr_001:LaborComplianceFiling:2026-01-01".to_owned(),
        payload_data_class: "InternalOnly".to_owned(),
        evidence_ref_count: 2,
        schema_version: 1,
    }
}

fn leave_record() -> HrPostgresStoredRecord {
    HrPostgresStoredRecord {
        kind: HrPostgresStoredRecordKind::LeavePayrollImpact,
        topic: "integration.hr.payroll.leave-impact".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        primary_ref: "leave_001".to_owned(),
        idempotency_key: "leave_001:payroll-impact".to_owned(),
        payload_data_class: "Financial".to_owned(),
        evidence_ref_count: 3,
        schema_version: 1,
    }
}

fn sensitive_record() -> HrPostgresStoredRecord {
    HrPostgresStoredRecord {
        kind: HrPostgresStoredRecordKind::SensitiveReadPolicy,
        topic: "audit.hr.sensitive-read.policy".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        primary_ref: "emp_001".to_owned(),
        idempotency_key: "emp_001:Medical:BenefitsAdministration".to_owned(),
        payload_data_class: "Phi".to_owned(),
        evidence_ref_count: 4,
        schema_version: 1,
    }
}
