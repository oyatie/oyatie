#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use billing_accounting_journal::{
    AccountingDomainError, Jurisdiction, VatDeadlineInput, VatWorkflowStep, evaluate_vat_deadline,
};

mod support;
use support::digest;

#[test]
fn test_kr_vat_deadline_opens_workflow() {
    let workflow = evaluate_vat_deadline(vat_input())
        .expect("valid")
        .expect("opened");

    assert_eq!(workflow.return_id.value.value, "vat_2026_q1");
    assert_eq!(workflow.hometax_export_hash.value.value, digest());
    assert!(
        workflow
            .required_steps
            .value
            .contains(&VatWorkflowStep::HomeTaxExportHashAttached)
    );
    assert!(
        workflow
            .required_steps
            .value
            .contains(&VatWorkflowStep::EvidencePackAttached)
    );

    let mut early = vat_input();
    early.now_epoch_seconds = 1_700_000_000;
    assert_eq!(
        evaluate_vat_deadline(early),
        Err(AccountingDomainError::VatDeadlineNotReached)
    );

    let mut us = vat_input();
    us.jurisdiction = Jurisdiction::UnitedStates;
    assert_eq!(evaluate_vat_deadline(us).expect("valid"), None);
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
