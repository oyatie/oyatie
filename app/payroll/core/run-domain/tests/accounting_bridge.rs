#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use payroll_run_domain::{
    PayrollDomainError, PayrollJournalInput, PayrollJournalLineInput, build_payroll_journal,
};

mod support;
use support::digest;

#[test]
fn test_payroll_posts_balanced_journals() {
    let draft = build_payroll_journal(journal_input()).expect("journal draft");

    assert_eq!(
        draft.total_debit_minor.value,
        draft.total_credit_minor.value
    );
    assert_eq!(draft.source_payroll_digest.value.value, digest());
    assert!(draft.reversal_required_ref.value.value.contains("reversal"));

    let mut unbalanced = journal_input();
    unbalanced.lines[1].credit_minor = 99;
    assert_eq!(
        build_payroll_journal(unbalanced),
        Err(PayrollDomainError::UnbalancedJournal)
    );

    let mut self_balancing_line = journal_input();
    self_balancing_line.lines = vec![PayrollJournalLineInput {
        account_code: "EXP-WAGES".to_owned(),
        debit_minor: 100,
        credit_minor: 100,
    }];
    assert_eq!(
        build_payroll_journal(self_balancing_line),
        Err(PayrollDomainError::InvalidMoney)
    );
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
