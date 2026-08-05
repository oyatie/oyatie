#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_hr_employment_app::{
    HR_SENSITIVE_READ_AUDIT_EMISSION_CONTRACT_REF, HrAppError, SensitiveHrRuntimeReadBoundaryInput,
    authorize_sensitive_hr_runtime_read_boundary, prepare_sensitive_hr_read_envelope,
};
use oya_hr_employment_domain::{
    SensitiveHrDataKind, SensitiveHrReadInput, SensitiveReadDecisionStatus,
    SensitiveReadLegalBasis, SensitiveReadPurpose,
};

#[test]
fn sensitive_read_envelope_is_metadata_only() {
    let outcome =
        prepare_sensitive_hr_read_envelope(valid_input()).expect("sensitive read outcome");

    assert_eq!(
        outcome.audit_envelope.topic.value,
        "audit.hr.sensitive-read.policy"
    );
    assert_eq!(outcome.audit_envelope.tenant_id.value.value, "ten_acme");
    assert_eq!(
        outcome.audit_envelope.actor_employee_id.value.value,
        "emp_admin_001"
    );
    assert_eq!(
        outcome.audit_envelope.subject_employee_id.value.value,
        "emp_001"
    );
    assert_eq!(
        outcome.audit_envelope.data_kind.value,
        SensitiveHrDataKind::Medical
    );
    assert_eq!(
        outcome.audit_envelope.purpose.value,
        SensitiveReadPurpose::BenefitsAdministration
    );
    assert_eq!(
        outcome.audit_envelope.legal_basis.value,
        SensitiveReadLegalBasis::Consent
    );
    assert_eq!(
        outcome.audit_envelope.policy_ref.value.value,
        "policy/hr/sensitive-read/benefits-2026"
    );
    assert_eq!(
        outcome.audit_envelope.read_log_evidence_ref.value.value,
        "audit/hr/privacy/emp_001/read-log"
    );
    assert_eq!(
        outcome.audit_envelope.decision_status.value,
        SensitiveReadDecisionStatus::Allowed
    );
    assert_eq!(
        outcome.audit_envelope.payload_data_class.value,
        DataClass::Phi
    );
    assert_eq!(outcome.audit_envelope.schema_version.value, 1);
    assert_eq!(
        outcome.decision.idempotency_key.value,
        outcome.audit_envelope.idempotency_key.value
    );
}

#[test]
fn sensitive_read_runtime_boundary_fails_closed_without_tenant_rbac_scope() {
    let mut input = runtime_boundary_input();
    input.tenant_rbac_scope_evidence_ref = None;

    let error = authorize_sensitive_hr_runtime_read_boundary(input)
        .expect_err("runtime sensitive read must require tenant/RBAC scope evidence");

    assert_eq!(error, HrAppError::MissingTenantRbacScopeEvidence);
}

#[test]
fn sensitive_read_runtime_boundary_fails_closed_without_audit_contract() {
    let mut input = runtime_boundary_input();
    input.audit_emission_contract_ref = None;

    let error = authorize_sensitive_hr_runtime_read_boundary(input)
        .expect_err("runtime sensitive read must require audit emission contract evidence");

    assert_eq!(error, HrAppError::MissingSensitiveReadAuditContract);
}

#[test]
fn sensitive_read_runtime_boundary_rejects_invalid_tenant_rbac_scope_shape() {
    for invalid_scope_ref in [
        "audit/tenant-rbac/hr-sensitive-read/",
        "audit/tenant-rbac/hr-sensitive-read/../escape",
        "audit/tenant-rbac/hr-sensitive-read/scope token",
        "audit/tenant-rbac/hr-sensitive-read/secret-material",
        "audit/hr/privacy/emp_001/scope",
    ] {
        let mut input = runtime_boundary_input();
        input.tenant_rbac_scope_evidence_ref = Some(invalid_scope_ref.to_owned());

        let error = authorize_sensitive_hr_runtime_read_boundary(input)
            .expect_err("runtime sensitive read rejects malformed tenant/RBAC scope evidence");

        assert_eq!(error, HrAppError::InvalidTenantRbacScopeEvidence);
    }
}

#[test]
fn sensitive_read_runtime_boundary_preserves_metadata_without_sensitive_echo() {
    let outcome = authorize_sensitive_hr_runtime_read_boundary(runtime_boundary_input())
        .expect("runtime boundary accepts dependency evidence");

    assert_eq!(
        outcome.audit_envelope.topic.value,
        "audit.hr.sensitive-read.policy"
    );
    assert_eq!(
        outcome
            .audit_envelope
            .tenant_rbac_scope_evidence_ref
            .value
            .value,
        "audit/tenant-rbac/hr-sensitive-read/entset_hr_privacy_kr"
    );
    assert_eq!(
        outcome.audit_envelope.audit_emission_contract_ref.value,
        HR_SENSITIVE_READ_AUDIT_EMISSION_CONTRACT_REF
    );
    assert_eq!(
        outcome.audit_envelope.audit_event_class.value,
        "HrSensitiveReadPolicyEvaluated"
    );
    assert!(!outcome.audit_envelope.sensitive_data_fetch.value);
    assert!(!outcome.audit_envelope.raw_sensitive_data_echo.value);
    assert_eq!(
        outcome.audit_envelope.decision_status.value,
        SensitiveReadDecisionStatus::Allowed
    );
}

fn valid_input() -> SensitiveHrReadInput {
    SensitiveHrReadInput {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        actor_employee_id: "emp_admin_001".to_owned(),
        subject_employee_id: "emp_001".to_owned(),
        data_kind: SensitiveHrDataKind::Medical,
        purpose: SensitiveReadPurpose::BenefitsAdministration,
        legal_basis: SensitiveReadLegalBasis::Consent,
        policy_ref: "policy/hr/sensitive-read/benefits-2026".to_owned(),
        basis_evidence_ref: "audit/hr/privacy/emp_001/basis".to_owned(),
        consent_evidence_ref: Some("audit/hr/privacy/emp_001/consent".to_owned()),
        request_evidence_ref: "audit/hr/privacy/emp_001/request".to_owned(),
        read_log_evidence_ref: "audit/hr/privacy/emp_001/read-log".to_owned(),
        evaluated_at_epoch_seconds: 1_779_533_400,
    }
}

fn runtime_boundary_input() -> SensitiveHrRuntimeReadBoundaryInput {
    SensitiveHrRuntimeReadBoundaryInput {
        policy_input: valid_input(),
        tenant_rbac_scope_evidence_ref: Some(
            "audit/tenant-rbac/hr-sensitive-read/entset_hr_privacy_kr".to_owned(),
        ),
        audit_emission_contract_ref: Some(HR_SENSITIVE_READ_AUDIT_EMISSION_CONTRACT_REF.to_owned()),
    }
}
