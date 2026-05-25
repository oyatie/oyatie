#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_accounting_journal_domain::{AccountingDomainError, ClosePromotionInput, promote_close};

#[test]
fn test_close_refuses_missing_evidence() {
    let promotion = promote_close(close_input(vec![
        "audit/accounting/close/reconciliation".to_owned(),
        "audit/accounting/close/controller-approval".to_owned(),
    ]))
    .expect("promotion");

    assert!(promotion.promoted.value);
    assert_eq!(promotion.evidence_refs.value.len(), 2);

    assert_eq!(
        promote_close(close_input(Vec::new())),
        Err(AccountingDomainError::MissingCloseEvidence)
    );

    let mut manual = close_input(vec!["audit/accounting/close/evidence".to_owned()]);
    manual.manual_shell_workaround_requested = true;
    assert_eq!(
        promote_close(manual),
        Err(AccountingDomainError::ManualShellWorkaroundRefused)
    );
}

fn close_input(required_evidence_refs: Vec<String>) -> ClosePromotionInput {
    ClosePromotionInput {
        close_id: "close_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        required_evidence_refs,
        manual_shell_workaround_requested: false,
    }
}
