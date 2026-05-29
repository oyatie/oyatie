#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_hr_employment_app::prepare_sensitive_hr_read_envelope;
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
