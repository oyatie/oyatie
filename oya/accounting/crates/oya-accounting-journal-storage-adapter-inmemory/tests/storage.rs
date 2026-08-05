#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_accounting_journal_app::{
    plan_vat_workflow, post_journal_with_audit, record_payroll_posting,
    record_payroll_posting_for_period,
};
use oya_accounting_journal_domain::{
    JournalLineInput, JournalPostInput, Jurisdiction, PayrollPostingInput, PeriodState,
    VatDeadlineInput,
};
use oya_accounting_journal_storage_adapter_inmemory::{
    AccountingJournalStoragePort, AccountingStorageError, AccountingStoredRecordKind,
    InMemoryAccountingJournalStore, accounting_storage_capabilities,
};

#[test]
fn accounting_storage_records_metadata_without_durable_backend_claim() {
    let mut store = InMemoryAccountingJournalStore::new();

    let journal = post_journal_with_audit(journal_input()).expect("journal outcome");
    let journal_record = store
        .persist_journal_post_audit(&journal.audit_envelope)
        .expect("persist journal audit");
    assert_eq!(
        journal_record.kind,
        AccountingStoredRecordKind::JournalPostAudit
    );
    assert_eq!(journal_record.topic, "audit.accounting.journal.posted");
    assert_eq!(journal_record.primary_ref, "jrn_2026_01");
    assert_eq!(journal_record.evidence_ref_count, 1);
    assert_eq!(
        journal_record.storage_backend,
        "in-memory-accounting-reference"
    );

    let payroll = record_payroll_posting(payroll_input()).expect("payroll posting outcome");
    let payroll_record = store
        .persist_payroll_posting_audit(&payroll.audit_envelope)
        .expect("persist payroll posting audit");
    assert_eq!(
        payroll_record.kind,
        AccountingStoredRecordKind::PayrollPostingAudit
    );
    assert_eq!(payroll_record.topic, "audit.accounting.payroll.posted");
    assert_eq!(payroll_record.primary_ref, "jrn_payroll_2026_01");
    assert_eq!(payroll_record.evidence_ref_count, 3);

    let vat = plan_vat_workflow(vat_input()).expect("VAT workflow outcome");
    let dispatch = vat.dispatch_envelope.expect("dispatch envelope");
    let vat_record = store
        .persist_vat_workflow_dispatch(&dispatch)
        .expect("persist VAT workflow dispatch");
    assert_eq!(
        vat_record.kind,
        AccountingStoredRecordKind::VatWorkflowDispatch
    );
    assert_eq!(vat_record.topic, "workflow.accounting.vat.dispatch");
    assert_eq!(vat_record.primary_ref, "vat_2026_q1");
    assert_eq!(vat_record.payload_data_class, "Financial");

    assert_eq!(store.len(), 3);
    assert!(
        store
            .require_record(&journal_record.idempotency_key)
            .is_ok()
    );

    let capabilities = accounting_storage_capabilities();
    assert_eq!(capabilities.adapter, "in-memory-accounting-reference");
    assert!(!capabilities.durable_ledger_backend_attached);
    assert!(!capabilities.postgres_rls_attached);
    assert!(!capabilities.workflow_execution_attached);
    assert!(!capabilities.statutory_filing_rails_attached);
    assert!(!capabilities.payment_execution_attached);
    assert!(!capabilities.payroll_network_call_attached);
    assert!(!capabilities.audit_chain_emission_attached);
}

#[test]
fn accounting_storage_refuses_duplicate_idempotency_keys() {
    let mut store = InMemoryAccountingJournalStore::new();
    let journal = post_journal_with_audit(journal_input()).expect("journal outcome");
    store
        .persist_journal_post_audit(&journal.audit_envelope)
        .expect("first persist");

    let error = store
        .persist_journal_post_audit(&journal.audit_envelope)
        .expect_err("duplicate idempotency key must be refused");
    assert_eq!(
        error,
        AccountingStorageError::DuplicateIdempotencyKey("jrn_2026_01:1:posted".to_owned())
    );
}

#[test]
fn payroll_gl_bridge_posting_preserves_accounting_control_refs() {
    let mut store = InMemoryAccountingJournalStore::new();
    let payroll = record_payroll_posting(payroll_input()).expect("payroll posting outcome");

    assert_eq!(
        payroll.evidence.journal.legal_entity_id.value.value,
        "le_kr_001"
    );
    assert_eq!(
        payroll.evidence.journal.approval_evidence_ref.value.value,
        "audit/accounting/payroll/approval"
    );
    assert_eq!(payroll.evidence.source_payroll_digest.value.value, digest());
    assert_eq!(
        payroll.evidence.wage_ledger_refs.value[0].value,
        "audit/payroll/wage-ledger/001"
    );
    assert_eq!(
        payroll.evidence.reversal_path_ref.value.value,
        "audit/accounting/payroll/reversal"
    );

    let record = store
        .persist_payroll_posting_audit(&payroll.audit_envelope)
        .expect("persist payroll posting audit");
    assert_eq!(
        record.idempotency_key,
        "ten_acme:jrn_payroll_2026_01:payroll-posted"
    );
    assert_eq!(record.evidence_ref_count, 3);
}

#[test]
fn payroll_gl_bridge_posting_refuses_closed_period() {
    assert!(record_payroll_posting_for_period(payroll_input(), PeriodState::Closed).is_err());
}

#[test]
fn payroll_gl_bridge_posting_refuses_duplicate_idempotency_key() {
    let mut store = InMemoryAccountingJournalStore::new();
    let payroll = record_payroll_posting(payroll_input()).expect("payroll posting outcome");
    store
        .persist_payroll_posting_audit(&payroll.audit_envelope)
        .expect("first payroll posting audit");

    let error = store
        .persist_payroll_posting_audit(&payroll.audit_envelope)
        .expect_err("duplicate payroll posting must be refused");

    assert_eq!(
        error,
        AccountingStorageError::DuplicateIdempotencyKey(
            "ten_acme:jrn_payroll_2026_01:payroll-posted".to_owned()
        )
    );
}

#[test]
fn accounting_storage_reservation_validates_key_shape_and_allows_commit() {
    let mut store = InMemoryAccountingJournalStore::new();
    assert_eq!(
        store.reserve_idempotency_key("bad key"),
        Err(AccountingStorageError::InvalidIdempotencyKey(
            "bad key".to_owned()
        ))
    );

    let vat = plan_vat_workflow(vat_input()).expect("VAT workflow outcome");
    let dispatch = vat.dispatch_envelope.expect("dispatch envelope");
    let key = dispatch.idempotency_key.value.clone();
    store.reserve_idempotency_key(&key).expect("reserve key");
    store
        .persist_vat_workflow_dispatch(&dispatch)
        .expect("reserved key can be committed once");
    assert_eq!(store.len(), 1);
}

fn digest() -> String {
    format!("sha256:{}", "b".repeat(64))
}

fn lines() -> Vec<JournalLineInput> {
    vec![
        JournalLineInput {
            account_code: "EXP-WAGES".to_owned(),
            debit_minor: 1_000_000,
            credit_minor: 0,
        },
        JournalLineInput {
            account_code: "LIAB-NETPAY".to_owned(),
            debit_minor: 0,
            credit_minor: 1_000_000,
        },
    ]
}

fn journal_input() -> JournalPostInput {
    JournalPostInput {
        journal_id: "jrn_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        period_state: PeriodState::Open,
        source_documents: vec!["src/payroll/run/prun_kr_2026_01".to_owned()],
        approval_evidence_ref: "audit/accounting/journal/approval".to_owned(),
        lines: lines(),
    }
}

fn vat_input() -> VatDeadlineInput {
    VatDeadlineInput {
        return_id: "vat_2026_q1".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        jurisdiction: Jurisdiction::Korea,
        period: "2026-01".to_owned(),
        deadline_epoch_seconds: 1_779_519_600,
        now_epoch_seconds: 1_779_519_601,
        workflow_ref: "workflow/accounting/vat/kr".to_owned(),
        hometax_export_hash: digest(),
        evidence_ref: "audit/accounting/vat/evidence".to_owned(),
    }
}

fn payroll_input() -> PayrollPostingInput {
    PayrollPostingInput {
        journal_id: "jrn_payroll_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        source_payroll_digest: digest(),
        wage_ledger_refs: vec!["audit/payroll/wage-ledger/001".to_owned()],
        approval_evidence_ref: "audit/accounting/payroll/approval".to_owned(),
        reversal_path_ref: "audit/accounting/payroll/reversal".to_owned(),
        lines: lines(),
    }
}
