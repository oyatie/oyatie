#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_hr_employment_domain::{
    BackendParityClaimStatus, HrBackendParityCapability, HrBackendParityProfileInput,
    HrDomainError, build_hr_backend_parity_profile,
};

#[test]
fn hr_backend_parity_profile_covers_hcm_capability_families_without_runtime_claims() {
    let profile = build_hr_backend_parity_profile(profile_input()).expect("profile");

    assert_eq!(profile.tenant_id.value.value, "ten_acme");
    assert_eq!(
        profile.profile_evidence_ref.value.value,
        "audit/hr/parity/profile"
    );
    assert_eq!(profile.schema_version.value, 1);
    assert_eq!(
        profile.schema_version.data_class.compatibility_data_class(),
        DataClass::Public
    );

    for expected in [
        HrBackendParityCapability::WorkforceCore,
        HrBackendParityCapability::OrganizationJobPosition,
        HrBackendParityCapability::LifecycleOnboardingOffboarding,
        HrBackendParityCapability::TimeAttendanceAbsence,
        HrBackendParityCapability::BenefitsCompensation,
        HrBackendParityCapability::TalentPerformanceLearning,
        HrBackendParityCapability::LaborStatutoryCompliance,
        HrBackendParityCapability::SensitiveHrPrivacy,
        HrBackendParityCapability::AnalyticsWorkforcePlanning,
        HrBackendParityCapability::IntegrationEvents,
        HrBackendParityCapability::CloudKubernetesReadiness,
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
        assert_eq!(capability.evidence_refs.value.len(), 4);
    }

    let cloud = profile
        .capabilities
        .value
        .iter()
        .find(|capability| {
            capability.capability.value == HrBackendParityCapability::CloudKubernetesReadiness
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
            .any(|claim| claim.contains("no shared cloud/Kubernetes deployment substrate"))
    );
}

#[test]
fn hr_backend_parity_profile_rejects_missing_or_unsafe_evidence() {
    let mut missing_sources = profile_input();
    missing_sources.source_evidence_refs.clear();
    assert_eq!(
        build_hr_backend_parity_profile(missing_sources),
        Err(HrDomainError::RulepackSourcesRequired)
    );

    let mut bad_profile_ref = profile_input();
    bad_profile_ref.profile_evidence_ref = "audit/hr/../profile".to_owned();
    assert_eq!(
        build_hr_backend_parity_profile(bad_profile_ref),
        Err(HrDomainError::InvalidAuditEvidenceRef)
    );
}

fn profile_input() -> HrBackendParityProfileInput {
    HrBackendParityProfileInput {
        tenant_id: "ten_acme".to_owned(),
        profile_evidence_ref: "audit/hr/parity/profile".to_owned(),
        source_evidence_refs: vec![
            "audit/hr/parity/source/sap-successfactors".to_owned(),
            "audit/hr/parity/source/oracle-hcm".to_owned(),
            "audit/hr/parity/source/workday-hcm".to_owned(),
            "audit/hr/parity/source/kubernetes".to_owned(),
        ],
    }
}
