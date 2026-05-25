#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_accounting_journal_domain::{ApApprovalCheck, ApInvoiceInput, evaluate_invoice_approval};

#[test]
fn test_invoice_approval_before_liability_post() {
    let route = evaluate_invoice_approval(invoice(false)).expect("route");

    assert!(!route.liability_post_allowed.value);
    assert!(!route.payment_request_allowed.value);
    assert_eq!(
        route.required_checks.value,
        vec![
            ApApprovalCheck::Policy,
            ApApprovalCheck::Budget,
            ApApprovalCheck::Vendor,
            ApApprovalCheck::Evidence,
        ]
    );

    let approved = evaluate_invoice_approval(invoice(true)).expect("approved route");
    assert!(approved.liability_post_allowed.value);
    assert!(approved.payment_request_allowed.value);
}

fn invoice(approved: bool) -> ApInvoiceInput {
    ApInvoiceInput {
        invoice_id: "apinv_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        vendor_ref: "src/vendor/acme".to_owned(),
        amount_minor: 2_500_000,
        policy_threshold_minor: 1_000_000,
        budget_ref: "src/budget/ops".to_owned(),
        evidence_ref: "audit/accounting/ap/evidence".to_owned(),
        approved,
    }
}
