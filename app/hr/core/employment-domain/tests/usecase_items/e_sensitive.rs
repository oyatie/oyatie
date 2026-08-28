mod sensitive_usecase_contract {
    use data_boundary_kernel::DataClass;
    use hr_employment_domain::{
        HrAppError, HrDomainError, SensitiveHrDataKind, SensitiveHrReadInput,
        SensitiveReadDecisionStatus, SensitiveReadLegalBasis, SensitiveReadPurpose,
        prepare_sensitive_hr_read_envelope,
    };

    #[test]
    fn accepted_sensitive_read_preserves_the_policy_audit_envelope() {
        // Catches sensitive-read actor, policy, or PHI payload metadata being altered.
        let outcome = prepare_sensitive_hr_read_envelope(SensitiveHrReadInput {
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
        })
        .expect("literal policy-bound sensitive read is accepted");

        let envelope = outcome.audit_envelope;
        assert_eq!(envelope.topic.value, "audit.hr.sensitive-read.policy");
        assert_eq!(envelope.tenant_id.value.value, "ten_acme");
        assert_eq!(envelope.legal_entity_id.value.value, "le_kr_001");
        assert_eq!(envelope.actor_employee_id.value.value, "emp_admin_001");
        assert_eq!(envelope.subject_employee_id.value.value, "emp_001");
        assert_eq!(envelope.data_kind.value, SensitiveHrDataKind::Medical);
        assert_eq!(
            envelope.purpose.value,
            SensitiveReadPurpose::BenefitsAdministration
        );
        assert_eq!(envelope.legal_basis.value, SensitiveReadLegalBasis::Consent);
        assert_eq!(
            envelope.policy_ref.value.value,
            "policy/hr/sensitive-read/benefits-2026"
        );
        assert_eq!(
            envelope.basis_evidence_ref.value.value,
            "audit/hr/privacy/emp_001/basis"
        );
        assert_eq!(
            envelope
                .consent_evidence_ref
                .value
                .expect("consent evidence")
                .value,
            "audit/hr/privacy/emp_001/consent"
        );
        assert_eq!(
            envelope.decision_status.value,
            SensitiveReadDecisionStatus::Allowed
        );
        assert_eq!(
            envelope.idempotency_key.value,
            "ten_acme:emp_001:Medical:BenefitsAdministration:1779533400"
        );
        assert_eq!(envelope.payload_data_class.value, DataClass::Phi);
        assert_eq!(envelope.schema_version.value, 1);
    }

    #[test]
    fn sensitive_read_returns_the_domain_error_when_consent_evidence_is_missing() {
        // Catches consent reads bypassing the mandatory evidence boundary.
        let error = prepare_sensitive_hr_read_envelope(SensitiveHrReadInput {
            tenant_id: "ten_acme".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            actor_employee_id: "emp_admin_001".to_owned(),
            subject_employee_id: "emp_001".to_owned(),
            data_kind: SensitiveHrDataKind::Medical,
            purpose: SensitiveReadPurpose::BenefitsAdministration,
            legal_basis: SensitiveReadLegalBasis::Consent,
            policy_ref: "policy/hr/sensitive-read/benefits-2026".to_owned(),
            basis_evidence_ref: "audit/hr/privacy/emp_001/basis".to_owned(),
            consent_evidence_ref: None,
            request_evidence_ref: "audit/hr/privacy/emp_001/request".to_owned(),
            read_log_evidence_ref: "audit/hr/privacy/emp_001/read-log".to_owned(),
            evaluated_at_epoch_seconds: 1_779_533_400,
        })
        .expect_err("consent without evidence is rejected");

        assert_eq!(
            error,
            HrAppError::Domain(HrDomainError::MissingConsentEvidence)
        );
    }
}
