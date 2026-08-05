#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_storage_adapter_postgres::{
    POSTGRES_PAYROLL_RUN_ENVELOPE_DDL, POSTGRES_PAYROLL_RUN_ENVELOPE_RLS_SQL,
    POSTGRES_PAYROLL_RUN_ENVELOPE_ROLLBACK_SQL, PayrollPostgresPlanError, PayrollPostgresRunStore,
    PayrollPostgresStoredRecord, PayrollPostgresStoredRecordKind,
    payroll_postgres_storage_capabilities,
};

#[test]
fn postgres_contract_declares_rls_and_rollback_without_runtime_overclaims() {
    let capabilities = payroll_postgres_storage_capabilities();
    assert_eq!(capabilities.adapter, "postgres-payroll-rls-contract");
    assert!(capabilities.durable_backend_contract_declared);
    assert!(capabilities.postgres_rls_contract_declared);
    assert!(!capabilities.runtime_database_execution_attached);
    assert!(!capabilities.payroll_calculation_attached);
    assert!(!capabilities.statutory_filing_rails_attached);
    assert!(!capabilities.disbursement_rails_attached);
    assert!(!capabilities.workflow_dispatch_attached);
    assert!(!capabilities.hr_network_call_attached);
    assert!(!capabilities.accounting_network_call_attached);
    assert!(!capabilities.audit_chain_emission_attached);

    assert!(POSTGRES_PAYROLL_RUN_ENVELOPE_DDL.contains("payroll_run_envelopes"));
    assert!(POSTGRES_PAYROLL_RUN_ENVELOPE_DDL.contains("tenant_id TEXT NOT NULL"));
    assert!(POSTGRES_PAYROLL_RUN_ENVELOPE_DDL.contains("legal_entity_id TEXT NOT NULL"));
    assert!(POSTGRES_PAYROLL_RUN_ENVELOPE_DDL.contains("run_id TEXT NOT NULL"));
    assert!(POSTGRES_PAYROLL_RUN_ENVELOPE_DDL.contains("idempotency_key TEXT NOT NULL"));
    assert!(
        POSTGRES_PAYROLL_RUN_ENVELOPE_DDL
            .contains("UNIQUE (tenant_id, legal_entity_id, run_id, idempotency_key)")
    );
    assert!(POSTGRES_PAYROLL_RUN_ENVELOPE_DDL.contains("payroll_run_idempotency_reservations"));
    assert!(POSTGRES_PAYROLL_RUN_ENVELOPE_RLS_SQL.contains("ENABLE ROW LEVEL SECURITY"));
    assert!(POSTGRES_PAYROLL_RUN_ENVELOPE_RLS_SQL.contains("FORCE ROW LEVEL SECURITY"));
    assert!(
        POSTGRES_PAYROLL_RUN_ENVELOPE_RLS_SQL.contains("current_setting('oyatie.tenant_id', true)")
    );
    assert!(
        POSTGRES_PAYROLL_RUN_ENVELOPE_ROLLBACK_SQL
            .contains("DROP TABLE IF EXISTS payroll_run_envelopes")
    );
    assert!(
        POSTGRES_PAYROLL_RUN_ENVELOPE_ROLLBACK_SQL
            .contains("DROP TABLE IF EXISTS payroll_run_idempotency_reservations")
    );
}

#[test]
fn postgres_commit_plan_preserves_payroll_envelope_metadata_and_requires_reservation() {
    let record = trial_close_record();
    let plan = PayrollPostgresRunStore::commit_record_plan("ten_acme", &record).unwrap();

    assert_eq!(
        plan.statement_name,
        "payroll_postgres_commit_envelope_after_reservation"
    );
    assert!(plan.sql.contains("INSERT INTO payroll_run_envelopes"));
    assert!(plan.sql.contains("WHERE EXISTS"));
    assert!(plan.sql.contains("payroll_run_idempotency_reservations"));
    assert_eq!(plan.params[0], "ten_acme");
    assert_eq!(plan.params[1], "le_kr_001");
    assert_eq!(plan.params[2], "prun_kr_2026_01");
    assert_eq!(plan.params[3], "trial_close_audit");
    assert_eq!(plan.params[4], "audit.payroll.run.close");
    assert_eq!(plan.params[5], "prun_kr_2026_01");
    assert_eq!(plan.params[6], "prun_kr_2026_01:trial");
    assert_eq!(plan.params[7], "Financial");
    assert_eq!(plan.params[8], "1");
    assert_eq!(plan.params[9], "1");
    assert_eq!(plan.expected_idempotency_key, "prun_kr_2026_01:trial");

    let hr_plan = PayrollPostgresRunStore::commit_record_plan("ten_acme", &hr_leave_record())
        .expect("HR leave-impact envelope commit plan");
    assert_eq!(hr_plan.params[3], "hr_leave_impact_intake");
    assert_eq!(
        hr_plan.params[4],
        "integration.payroll.hr.leave-impact-intake"
    );
    assert_eq!(hr_plan.params[5], "leave_001");
    assert_eq!(hr_plan.params[6], "ten_acme:leave_001:payroll-intake");
    assert_eq!(hr_plan.params[8], "2");
}

#[test]
fn postgres_reservation_plan_and_commit_refuse_cross_tenant_or_unsafe_keys() {
    let mut store = PayrollPostgresRunStore::default();
    let reservation = store
        .reserve_idempotency_key_plan(
            "ten_acme",
            "le_kr_001",
            "prun_kr_2026_01",
            "prun_kr_2026_01:trial",
        )
        .unwrap();
    assert_eq!(
        reservation.statement_name,
        "payroll_postgres_reserve_idempotency_key"
    );
    assert!(
        reservation
            .sql
            .contains("INSERT INTO payroll_run_idempotency_reservations")
    );
    assert_eq!(store.generated_plans().len(), 1);

    let mut other_tenant_record = accounting_record();
    other_tenant_record.tenant_id = "ten_other".to_owned();
    assert_eq!(
        PayrollPostgresRunStore::commit_record_plan("ten_acme", &other_tenant_record).unwrap_err(),
        PayrollPostgresPlanError::TenantMismatch
    );
    assert_eq!(
        PayrollPostgresRunStore::reserve_idempotency_key_plan_static(
            "ten_acme",
            "le_kr_001",
            "prun_kr_2026_01",
            "../bad-key",
        )
        .unwrap_err(),
        PayrollPostgresPlanError::UnsafeMetadata
    );
}

fn trial_close_record() -> PayrollPostgresStoredRecord {
    PayrollPostgresStoredRecord {
        kind: PayrollPostgresStoredRecordKind::TrialCloseAudit,
        topic: "audit.payroll.run.close".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        run_id: "prun_kr_2026_01".to_owned(),
        primary_ref: "prun_kr_2026_01".to_owned(),
        idempotency_key: "prun_kr_2026_01:trial".to_owned(),
        payload_data_class: "Financial".to_owned(),
        evidence_ref_count: 1,
        schema_version: 1,
    }
}

fn accounting_record() -> PayrollPostgresStoredRecord {
    PayrollPostgresStoredRecord {
        kind: PayrollPostgresStoredRecordKind::AccountingJournalDispatch,
        topic: "tenant_rbac.payroll.accounting.journal_draft".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        run_id: "prun_kr_2026_01".to_owned(),
        primary_ref: "jrn_payroll_2026_01".to_owned(),
        idempotency_key: "prun_kr_2026_01:jrn_payroll_2026_01:accounting-dispatch".to_owned(),
        payload_data_class: "Financial".to_owned(),
        evidence_ref_count: 2,
        schema_version: 1,
    }
}

fn hr_leave_record() -> PayrollPostgresStoredRecord {
    PayrollPostgresStoredRecord {
        kind: PayrollPostgresStoredRecordKind::HrLeaveImpactIntake,
        topic: "integration.payroll.hr.leave-impact-intake".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        run_id: "prun_kr_2026_06".to_owned(),
        primary_ref: "leave_001".to_owned(),
        idempotency_key: "ten_acme:leave_001:payroll-intake".to_owned(),
        payload_data_class: "Financial".to_owned(),
        evidence_ref_count: 2,
        schema_version: 1,
    }
}
