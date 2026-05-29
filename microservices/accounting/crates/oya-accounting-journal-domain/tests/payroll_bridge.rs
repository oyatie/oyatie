#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_accounting_journal_domain::{AccountingDomainError, PayrollPostingInput, payroll_posting};

mod support;
use support::{digest, lines};

#[test]
fn test_payroll_digest_required_for_posting() {
    let evidence = payroll_posting(payroll_input()).expect("payroll posting");

    assert_eq!(evidence.source_payroll_digest.value.value, digest());
    assert_eq!(evidence.wage_ledger_refs.value.len(), 1);
    assert!(evidence.reversal_path_ref.value.value.contains("reversal"));

    let mut missing_digest = payroll_input();
    missing_digest.source_payroll_digest.clear();
    assert_eq!(
        payroll_posting(missing_digest),
        Err(AccountingDomainError::PayrollDigestRequired)
    );

    let mut missing_wage_refs = payroll_input();
    missing_wage_refs.wage_ledger_refs.clear();
    assert_eq!(
        payroll_posting(missing_wage_refs),
        Err(AccountingDomainError::WageLedgerRefsRequired)
    );
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
