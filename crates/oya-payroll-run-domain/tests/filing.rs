#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_domain::{
    PayrollDomainError, StatutoryExportInput, StatutoryExportKind, statutory_export_evidence,
};

mod support;
use support::digest;

#[test]
fn test_kr_exports_require_hashes_and_receipts() {
    let evidence = statutory_export_evidence(export_input()).expect("export evidence");

    assert_eq!(evidence.export_hash.value.value, digest());
    assert!(evidence.receipt_ref.value.is_some());
    assert!(evidence.rejection_reason.value.is_none());

    let mut missing_receipt = export_input();
    missing_receipt.receipt_ref = None;
    assert_eq!(
        statutory_export_evidence(missing_receipt),
        Err(PayrollDomainError::MissingReceiptOrRejection)
    );

    let mut rejection = export_input();
    rejection.receipt_ref = None;
    rejection.rejection_reason = Some("HomeTax schema rejected line 4".to_owned());
    assert!(statutory_export_evidence(rejection).is_ok());
}

fn export_input() -> StatutoryExportInput {
    StatutoryExportInput {
        run_id: "prun_kr_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        export_kind: StatutoryExportKind::KoreaHomeTaxWithholding,
        export_hash: digest(),
        receipt_ref: Some("audit/payroll/kr/hometax/receipt".to_owned()),
        rejection_reason: None,
        rollback_plan_ref: "audit/payroll/kr/hometax/rollback-plan".to_owned(),
    }
}
