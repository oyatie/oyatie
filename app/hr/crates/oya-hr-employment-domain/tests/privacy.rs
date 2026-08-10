#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_hr_employment_domain::{
    HrDomainError, SensitiveHrDataKind, SensitiveHrReadInput, SensitiveReadDecisionStatus,
    SensitiveReadLegalBasis, SensitiveReadPurpose, evaluate_sensitive_hr_read,
};

#[test]
fn test_sensitive_hr_read_requires_basis() {
    let decision = evaluate_sensitive_hr_read(valid_input()).expect("sensitive read decision");

    assert_eq!(decision.tenant_id.value.value, "ten_acme");
    assert_eq!(decision.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(decision.actor_employee_id.value.value, "emp_admin_001");
    assert_eq!(decision.subject_employee_id.value.value, "emp_001");
    assert_eq!(decision.data_kind.value, SensitiveHrDataKind::Medical);
    assert_eq!(
        decision.purpose.value,
        SensitiveReadPurpose::BenefitsAdministration
    );
    assert_eq!(decision.legal_basis.value, SensitiveReadLegalBasis::Consent);
    assert_eq!(
        decision.data_kind.data_class.compatibility_data_class(),
        DataClass::SensitivePipaArticle23
    );
    assert_eq!(
        decision.policy_ref.value.value,
        "policy/hr/sensitive-read/benefits-2026"
    );
    assert_eq!(
        decision.basis_evidence_ref.value.value,
        "audit/hr/privacy/emp_001/basis"
    );
    assert_eq!(
        decision
            .consent_evidence_ref
            .value
            .as_ref()
            .expect("consent evidence")
            .value,
        "audit/hr/privacy/emp_001/consent"
    );
    assert_eq!(
        decision.read_log_evidence_ref.value.value,
        "audit/hr/privacy/emp_001/read-log"
    );
    assert_eq!(
        decision.decision_status.value,
        SensitiveReadDecisionStatus::Allowed
    );
    assert_eq!(
        decision.idempotency_key.value,
        "ten_acme:emp_001:Medical:BenefitsAdministration:1779533400"
    );
    assert_eq!(decision.schema_version.value, 1);
}

#[test]
fn sensitive_hr_read_rejects_general_browsing() {
    let error = evaluate_sensitive_hr_read(SensitiveHrReadInput {
        purpose: SensitiveReadPurpose::GeneralBrowsing,
        ..valid_input()
    })
    .expect_err("general browsing is not purpose bound");

    assert_eq!(error, HrDomainError::DisallowedSensitiveReadPurpose);
}

#[test]
fn sensitive_hr_read_requires_legal_basis() {
    let error = evaluate_sensitive_hr_read(SensitiveHrReadInput {
        legal_basis: SensitiveReadLegalBasis::None,
        consent_evidence_ref: None,
        ..valid_input()
    })
    .expect_err("legal basis is required");

    assert_eq!(error, HrDomainError::MissingSensitiveReadLegalBasis);
}

#[test]
fn sensitive_hr_read_requires_consent_evidence_when_basis_is_consent() {
    let error = evaluate_sensitive_hr_read(SensitiveHrReadInput {
        consent_evidence_ref: None,
        ..valid_input()
    })
    .expect_err("consent basis requires consent evidence");

    assert_eq!(error, HrDomainError::MissingConsentEvidence);
}

#[test]
fn sensitive_hr_read_rejects_unsafe_read_log_evidence() {
    let error = evaluate_sensitive_hr_read(SensitiveHrReadInput {
        read_log_evidence_ref: "audit/hr/privacy/bearer-token".to_owned(),
        ..valid_input()
    })
    .expect_err("read log evidence cannot look like a credential");

    assert_eq!(error, HrDomainError::InvalidAuditEvidenceRef);
}

#[test]
fn sensitive_hr_read_requires_policy_ref_prefix() {
    let error = evaluate_sensitive_hr_read(SensitiveHrReadInput {
        policy_ref: "policy/hr/general-access/benefits-2026".to_owned(),
        ..valid_input()
    })
    .expect_err("sensitive read policy prefix is required");

    assert_eq!(error, HrDomainError::InvalidPolicyRef);
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
