#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use payroll_run_app::{
    close_trial_run, prepare_accounting_dispatch, prepare_hr_leave_impact_intake,
};
use payroll_run_domain::{
    HrLeaveImpactIntakeInput, HrLeaveImpactKind, MoneyAmount, PayeeClass, PayeeInput,
    PayrollJournalInput, PayrollJournalLineInput, PayrollTrialCloseInput, WageLedgerEntryInput,
    WageLineKind,
};
use payroll_run_storage_inmemory::{
    InMemoryPayrollRunStore, PayrollRunStoragePort, PayrollStorageError, PayrollStoredRecordKind,
    payroll_storage_capabilities,
};

#[test]
fn payroll_storage_records_metadata_without_durable_backend_claim() {
    let mut store = InMemoryPayrollRunStore::new();

    let close = close_trial_run(trial_close_input()).expect("trial close outcome");
    let close_record = store
        .persist_trial_close_audit(&close.audit_envelope)
        .expect("persist trial close audit");
    assert_eq!(close_record.kind, PayrollStoredRecordKind::TrialCloseAudit);
    assert_eq!(close_record.topic, "audit.payroll.run.close");
    assert_eq!(close_record.run_id, "prun_kr_2026_01");
    assert_eq!(close_record.primary_ref, "prun_kr_2026_01");
    assert_eq!(close_record.evidence_ref_count, 1);
    assert_eq!(close_record.storage_backend, "in-memory-payroll-reference");

    let accounting = prepare_accounting_dispatch(journal_input()).expect("accounting dispatch");
    let accounting_record = store
        .persist_accounting_dispatch(&accounting.dispatch_envelope)
        .expect("persist accounting dispatch");
    assert_eq!(
        accounting_record.kind,
        PayrollStoredRecordKind::AccountingJournalDispatch
    );
    assert_eq!(
        accounting_record.topic,
        "tenant_rbac.payroll.accounting.journal_draft"
    );
    assert_eq!(accounting_record.primary_ref, "jrn_payroll_2026_01");
    assert_eq!(accounting_record.evidence_ref_count, 2);

    let leave = prepare_hr_leave_impact_intake(hr_leave_input()).expect("HR leave intake");
    let leave_record = store
        .persist_hr_leave_impact_intake(&leave.intake_envelope)
        .expect("persist HR leave impact intake");
    assert_eq!(
        leave_record.kind,
        PayrollStoredRecordKind::HrLeaveImpactIntake
    );
    assert_eq!(
        leave_record.topic,
        "integration.payroll.hr.leave-impact-intake"
    );
    assert_eq!(leave_record.primary_ref, "leave_001");
    assert_eq!(leave_record.payload_data_class, "Financial");

    assert_eq!(store.len(), 3);
    assert!(store.require_record(&close_record.idempotency_key).is_ok());

    let capabilities = payroll_storage_capabilities();
    assert_eq!(capabilities.adapter, "in-memory-payroll-reference");
    assert!(!capabilities.durable_backend_attached);
    assert!(!capabilities.postgres_rls_attached);
    assert!(!capabilities.payroll_calculation_attached);
    assert!(!capabilities.statutory_filing_rails_attached);
    assert!(!capabilities.disbursement_rails_attached);
    assert!(!capabilities.workflow_dispatch_attached);
    assert!(!capabilities.hr_network_call_attached);
    assert!(!capabilities.accounting_network_call_attached);
    assert!(!capabilities.audit_chain_emission_attached);
}

#[test]
fn payroll_storage_refuses_duplicate_idempotency_keys() {
    let mut store = InMemoryPayrollRunStore::new();
    let close = close_trial_run(trial_close_input()).expect("trial close outcome");
    store
        .persist_trial_close_audit(&close.audit_envelope)
        .expect("first persist");

    let error = store
        .persist_trial_close_audit(&close.audit_envelope)
        .expect_err("duplicate idempotency key must be refused");
    assert_eq!(
        error,
        PayrollStorageError::DuplicateIdempotencyKey("prun_kr_2026_01:2026-01-01:trial".to_owned())
    );
}

#[test]
fn payroll_storage_reservation_validates_key_shape_and_allows_commit() {
    let mut store = InMemoryPayrollRunStore::new();
    assert_eq!(
        store.reserve_idempotency_key("../bad-key"),
        Err(PayrollStorageError::InvalidIdempotencyKey(
            "../bad-key".to_owned()
        ))
    );

    let leave = prepare_hr_leave_impact_intake(hr_leave_input()).expect("HR leave intake");
    let key = leave.intake_envelope.idempotency_key.value.clone();
    store.reserve_idempotency_key(&key).expect("reserve key");
    store
        .persist_hr_leave_impact_intake(&leave.intake_envelope)
        .expect("reserved key can be committed once");
    assert_eq!(store.len(), 1);
}

fn digest() -> String {
    format!("sha256:{}", "a".repeat(64))
}

fn trial_close_input() -> PayrollTrialCloseInput {
    PayrollTrialCloseInput {
        run_id: "prun_kr_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        evidence_digest: digest(),
        approval_evidence_ref: "audit/payroll/trial-close/approval".to_owned(),
        payees: vec![PayeeInput {
            payee_id: "payee_001".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            payee_class: PayeeClass::Employee,
            person_or_vendor_ref: "person/acme/001".to_owned(),
            tax_profile_ref: "tax/kr/employee/001".to_owned(),
            wage_ledger: vec![
                WageLedgerEntryInput {
                    entry_id: "wage_001_gross".to_owned(),
                    payee_id: "payee_001".to_owned(),
                    line_kind: WageLineKind::GrossEarnings,
                    amount: MoneyAmount {
                        amount_minor: 1_000_000,
                        currency: "KRW".to_owned(),
                    },
                    source_ref: "audit/hr/time/001".to_owned(),
                },
                WageLedgerEntryInput {
                    entry_id: "wage_001_net".to_owned(),
                    payee_id: "payee_001".to_owned(),
                    line_kind: WageLineKind::NetPay,
                    amount: MoneyAmount {
                        amount_minor: -800_000,
                        currency: "KRW".to_owned(),
                    },
                    source_ref: "audit/payroll/net/001".to_owned(),
                },
            ],
        }],
    }
}

fn journal_input() -> PayrollJournalInput {
    PayrollJournalInput {
        journal_id: "jrn_payroll_2026_01".to_owned(),
        run_id: "prun_kr_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        source_payroll_digest: digest(),
        approval_evidence_ref: "audit/payroll/approval/cfo".to_owned(),
        lines: vec![
            PayrollJournalLineInput {
                account_code: "EXP-WAGES".to_owned(),
                debit_minor: 1_000_000,
                credit_minor: 0,
            },
            PayrollJournalLineInput {
                account_code: "LIAB-NETPAY".to_owned(),
                debit_minor: 0,
                credit_minor: 1_000_000,
            },
        ],
    }
}

fn hr_leave_input() -> HrLeaveImpactIntakeInput {
    HrLeaveImpactIntakeInput {
        run_id: "prun_kr_2026_06".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payroll_period: "2026-06".to_owned(),
        payee_id: "payee_001".to_owned(),
        employee_id: "emp_001".to_owned(),
        leave_request_id: "leave_001".to_owned(),
        impact_kind: HrLeaveImpactKind::UnpaidLeaveDeduction,
        source_topic: "integration.hr.payroll.leave-impact".to_owned(),
        source_hr_idempotency_key: "ten_acme:leave_001:Approved:2026-06".to_owned(),
        decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
        routing_evidence_ref: "audit/hr/leave/leave_001/escalation".to_owned(),
        payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
        payroll_intake_evidence_ref: "audit/payroll/hr-leave/leave_001/intake".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        received_at_epoch_seconds: 1_779_535_200,
    }
}
