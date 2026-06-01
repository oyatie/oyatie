#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_payroll_run_domain::{
    BackendParityClaimStatus, PayrollBackendParityCapability, PayrollBackendParityProfileInput,
    PayrollDomainError, build_payroll_backend_parity_profile,
};

#[test]
fn payroll_backend_parity_profile_covers_leading_payroll_capabilities_without_live_rails_claims() {
    let profile = build_payroll_backend_parity_profile(profile_input()).expect("profile");

    assert_eq!(profile.tenant_id.value.value, "ten_acme");
    assert_eq!(
        profile.profile_evidence_ref.value.value,
        "audit/payroll/parity/profile"
    );
    assert_eq!(profile.schema_version.value, 1);
    assert_eq!(
        profile.schema_version.data_class.compatibility_data_class(),
        DataClass::Public
    );

    for expected in [
        PayrollBackendParityCapability::GrossToNetRunControls,
        PayrollBackendParityCapability::EarningsDeductionsTaxModel,
        PayrollBackendParityCapability::TimeLeavePayrollIntake,
        PayrollBackendParityCapability::RetroOffCycleReversal,
        PayrollBackendParityCapability::StatutoryExportEvidence,
        PayrollBackendParityCapability::PayslipDisbursementSeam,
        PayrollBackendParityCapability::AccountingGlExport,
        PayrollBackendParityCapability::GroupLegalEntityRollup,
        PayrollBackendParityCapability::VarianceAnomalyRollback,
        PayrollBackendParityCapability::AuditIdempotencyTenantResidency,
        PayrollBackendParityCapability::CloudKubernetesReadiness,
    ] {
        assert!(
            profile
                .capabilities
                .value
                .iter()
                .any(|capability| capability.capability.value == expected),
            "missing {expected:?}"
        );
    }

    for capability in &profile.capabilities.value {
        assert!(capability.tenant_scoped.value);
        assert!(capability.data_class_declared.value);
        assert!(capability.idempotency_contract.value);
        assert!(capability.audit_evidence_required.value);
        assert!(capability.residency_scope_declared.value);
        assert!(capability.observability_contract.value);
        assert!(!capability.production_runtime_claimed.value);
        assert!(!capability.live_money_movement_claimed.value);
        assert!(!capability.tax_filing_submission_claimed.value);
        assert_eq!(capability.evidence_refs.value.len(), 4);
    }

    let cloud = profile
        .capabilities
        .value
        .iter()
        .find(|capability| {
            capability.capability.value == PayrollBackendParityCapability::CloudKubernetesReadiness
        })
        .expect("cloud capability");
    assert_eq!(
        cloud.claim_status.value,
        BackendParityClaimStatus::ContractReady
    );
    assert!(cloud.kubernetes_native_contract_ready.value);
    assert!(
        profile
            .nonclaims
            .value
            .iter()
            .any(|claim| claim.contains("no bank/payment/disbursement rails"))
    );
}

#[test]
fn payroll_backend_parity_profile_rejects_missing_or_unsafe_evidence() {
    let mut missing_sources = profile_input();
    missing_sources.source_evidence_refs.clear();
    assert_eq!(
        build_payroll_backend_parity_profile(missing_sources),
        Err(PayrollDomainError::RulepackSourcesRequired)
    );

    let mut bad_profile_ref = profile_input();
    bad_profile_ref.profile_evidence_ref = "audit/payroll/../profile".to_owned();
    assert_eq!(
        build_payroll_backend_parity_profile(bad_profile_ref),
        Err(PayrollDomainError::InvalidEvidenceRef)
    );
}

fn profile_input() -> PayrollBackendParityProfileInput {
    PayrollBackendParityProfileInput {
        tenant_id: "ten_acme".to_owned(),
        profile_evidence_ref: "audit/payroll/parity/profile".to_owned(),
        source_evidence_refs: vec![
            "audit/payroll/parity/source/sap-successfactors-payroll".to_owned(),
            "audit/payroll/parity/source/oracle-payroll".to_owned(),
            "audit/payroll/parity/source/workday-payroll".to_owned(),
            "audit/payroll/parity/source/kubernetes".to_owned(),
        ],
    }
}
