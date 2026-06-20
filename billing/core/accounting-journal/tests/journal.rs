#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use billing_accounting_journal::{
    AccountingDomainError, JournalLineInput, PeriodState, post_journal,
};

mod support;
use support::journal_input;

#[test]
fn test_post_requires_balanced_open_period() {
    let journal = post_journal(journal_input()).expect("posted journal");

    assert_eq!(
        journal.total_debit_minor.value,
        journal.total_credit_minor.value
    );
    assert_eq!(journal.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(journal.source_documents.value.len(), 1);

    let mut unbalanced = journal_input();
    unbalanced.lines[1].credit_minor = 99;
    assert_eq!(
        post_journal(unbalanced),
        Err(AccountingDomainError::UnbalancedJournal)
    );

    let mut self_balancing_line = journal_input();
    self_balancing_line.lines = vec![JournalLineInput {
        account_code: "EXP-WAGES".to_owned(),
        debit_minor: 100,
        credit_minor: 100,
    }];
    assert_eq!(
        post_journal(self_balancing_line),
        Err(AccountingDomainError::InvalidMoney)
    );

    let mut closed_period = journal_input();
    closed_period.period_state = PeriodState::Closed;
    assert_eq!(
        post_journal(closed_period),
        Err(AccountingDomainError::PeriodNotOpen)
    );

    let mut missing_source = journal_input();
    missing_source.source_documents.clear();
    assert_eq!(
        post_journal(missing_source),
        Err(AccountingDomainError::InvalidSourceDocumentRef)
    );
}
