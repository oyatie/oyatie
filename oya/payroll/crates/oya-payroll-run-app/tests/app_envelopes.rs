#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::DataClass;
use oya_payroll_run_app::{close_trial_run, prepare_accounting_dispatch};
use oya_payroll_run_domain::{
    MoneyAmount, PayeeClass, PayeeInput, PayrollJournalInput, PayrollJournalLineInput,
    PayrollRunState, PayrollTrialCloseInput, WageLedgerEntryInput, WageLineKind,
};

#[test]
fn trial_close_emits_audit_event() {
    let outcome = close_trial_run(trial_close_input()).expect("trial close outcome");

    assert_eq!(outcome.run.state.value, PayrollRunState::TrialClosed);
    assert_eq!(
        outcome.audit_envelope.topic.value,
        "audit.payroll.run.close"
    );
    assert_eq!(outcome.audit_envelope.run_id.value.value, "prun_kr_2026_01");
    assert_eq!(outcome.audit_envelope.evidence_digest.value.value, digest());
    assert_eq!(
        outcome
            .audit_envelope
            .evidence_digest
            .data_class
            .compatibility_data_class(),
        DataClass::Financial
    );
    assert_eq!(
        outcome.audit_envelope.payload_data_class.value,
        DataClass::Financial
    );
    assert_eq!(
        outcome
            .audit_envelope
            .schema_version
            .data_class
            .compatibility_data_class(),
        DataClass::Public
    );
}

#[test]
fn accounting_bridge_emits_integration_event() {
    let outcome = prepare_accounting_dispatch(journal_input()).expect("accounting dispatch");

    assert_eq!(
        outcome.journal.total_debit_minor.value,
        outcome.journal.total_credit_minor.value
    );
    assert_eq!(
        outcome.dispatch_envelope.topic.value,
        "tenant_rbac.payroll.accounting.journal_draft"
    );
    assert_eq!(
        outcome.dispatch_envelope.source_payroll_digest.value.value,
        digest()
    );
    assert_eq!(
        outcome
            .dispatch_envelope
            .source_payroll_digest
            .data_class
            .compatibility_data_class(),
        DataClass::Financial
    );
    assert_eq!(
        outcome.dispatch_envelope.idempotency_key.value,
        "prun_kr_2026_01:jrn_payroll_2026_01:accounting-dispatch"
    );
    assert_eq!(
        outcome.dispatch_envelope.payload_data_class.value,
        DataClass::Financial
    );
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
