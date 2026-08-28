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
            envelope.request_evidence_ref.value.value,
            "audit/hr/privacy/emp_001/request"
        );
        assert_eq!(
            envelope.read_log_evidence_ref.value.value,
            "audit/hr/privacy/emp_001/read-log"
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
        assert_eq!(
            envelope.topic.data_class.compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            envelope.data_kind.data_class.compatibility_data_class(),
            DataClass::SensitivePipaArticle23
        );
        assert_eq!(
            envelope.policy_ref.data_class.compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            envelope
                .basis_evidence_ref
                .data_class
                .compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            envelope
                .consent_evidence_ref
                .data_class
                .compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            envelope
                .request_evidence_ref
                .data_class
                .compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            envelope
                .read_log_evidence_ref
                .data_class
                .compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            envelope
                .decision_status
                .data_class
                .compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            envelope
                .idempotency_key
                .data_class
                .compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            envelope
                .payload_data_class
                .data_class
                .compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            envelope
                .schema_version
                .data_class
                .compatibility_data_class(),
            DataClass::Public
        );
    }

    #[test]
    fn every_sensitive_data_kind_constructs_a_public_envelope_with_its_payload_class() {
        // Catches a valid sensitive-data kind being rejected or mapped to another payload class.
        let accommodation = prepare_sensitive_hr_read_envelope(SensitiveHrReadInput {
            tenant_id: "ten_acme".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            actor_employee_id: "emp_admin_001".to_owned(),
            subject_employee_id: "emp_001".to_owned(),
            data_kind: SensitiveHrDataKind::DisabilityAccommodation,
            purpose: SensitiveReadPurpose::BenefitsAdministration,
            legal_basis: SensitiveReadLegalBasis::Consent,
            policy_ref: "policy/hr/sensitive-read/benefits-2026".to_owned(),
            basis_evidence_ref: "audit/hr/privacy/emp_001/basis".to_owned(),
            consent_evidence_ref: Some("audit/hr/privacy/emp_001/consent".to_owned()),
            request_evidence_ref: "audit/hr/privacy/emp_001/request".to_owned(),
            read_log_evidence_ref: "audit/hr/privacy/emp_001/read-log".to_owned(),
            evaluated_at_epoch_seconds: 1_779_533_400,
        })
        .expect("disability accommodation envelope is public");
        let accommodation_schema = &accommodation.audit_envelope.schema_version;
        assert_eq!(
            accommodation.audit_envelope.data_kind.value,
            SensitiveHrDataKind::DisabilityAccommodation
        );
        assert_eq!(
            accommodation.audit_envelope.payload_data_class.value,
            DataClass::Phi
        );
        assert_eq!(accommodation_schema.value, 1);
        assert_eq!(
            accommodation_schema.data_class.compatibility_data_class(),
            DataClass::Public
        );
        let compensation = prepare_sensitive_hr_read_envelope(SensitiveHrReadInput {
            tenant_id: "ten_acme".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            actor_employee_id: "emp_admin_001".to_owned(),
            subject_employee_id: "emp_001".to_owned(),
            data_kind: SensitiveHrDataKind::Compensation,
            purpose: SensitiveReadPurpose::BenefitsAdministration,
            legal_basis: SensitiveReadLegalBasis::Consent,
            policy_ref: "policy/hr/sensitive-read/benefits-2026".to_owned(),
            basis_evidence_ref: "audit/hr/privacy/emp_001/basis".to_owned(),
            consent_evidence_ref: Some("audit/hr/privacy/emp_001/consent".to_owned()),
            request_evidence_ref: "audit/hr/privacy/emp_001/request".to_owned(),
            read_log_evidence_ref: "audit/hr/privacy/emp_001/read-log".to_owned(),
            evaluated_at_epoch_seconds: 1_779_533_400,
        })
        .expect("compensation envelope is public");
        let compensation_schema = &compensation.audit_envelope.schema_version;
        assert_eq!(
            compensation.audit_envelope.data_kind.value,
            SensitiveHrDataKind::Compensation
        );
        assert_eq!(
            compensation.audit_envelope.payload_data_class.value,
            DataClass::Financial
        );
        assert_eq!(compensation_schema.value, 1);
        assert_eq!(
            compensation_schema.data_class.compatibility_data_class(),
            DataClass::Public
        );
        let government_identifier = prepare_sensitive_hr_read_envelope(SensitiveHrReadInput {
            tenant_id: "ten_acme".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            actor_employee_id: "emp_admin_001".to_owned(),
            subject_employee_id: "emp_001".to_owned(),
            data_kind: SensitiveHrDataKind::GovernmentIdentifier,
            purpose: SensitiveReadPurpose::BenefitsAdministration,
            legal_basis: SensitiveReadLegalBasis::Consent,
            policy_ref: "policy/hr/sensitive-read/benefits-2026".to_owned(),
            basis_evidence_ref: "audit/hr/privacy/emp_001/basis".to_owned(),
            consent_evidence_ref: Some("audit/hr/privacy/emp_001/consent".to_owned()),
            request_evidence_ref: "audit/hr/privacy/emp_001/request".to_owned(),
            read_log_evidence_ref: "audit/hr/privacy/emp_001/read-log".to_owned(),
            evaluated_at_epoch_seconds: 1_779_533_400,
        })
        .expect("government identifier envelope is public");
        let government_identifier_schema = &government_identifier.audit_envelope.schema_version;
        assert_eq!(
            government_identifier.audit_envelope.data_kind.value,
            SensitiveHrDataKind::GovernmentIdentifier
        );
        assert_eq!(
            government_identifier
                .audit_envelope
                .payload_data_class
                .value,
            DataClass::PiiIdentifying
        );
        assert_eq!(government_identifier_schema.value, 1);
        assert_eq!(
            government_identifier_schema
                .data_class
                .compatibility_data_class(),
            DataClass::Public
        );
        let disciplinary = prepare_sensitive_hr_read_envelope(SensitiveHrReadInput {
            tenant_id: "ten_acme".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            actor_employee_id: "emp_admin_001".to_owned(),
            subject_employee_id: "emp_001".to_owned(),
            data_kind: SensitiveHrDataKind::Disciplinary,
            purpose: SensitiveReadPurpose::BenefitsAdministration,
            legal_basis: SensitiveReadLegalBasis::Consent,
            policy_ref: "policy/hr/sensitive-read/benefits-2026".to_owned(),
            basis_evidence_ref: "audit/hr/privacy/emp_001/basis".to_owned(),
            consent_evidence_ref: Some("audit/hr/privacy/emp_001/consent".to_owned()),
            request_evidence_ref: "audit/hr/privacy/emp_001/request".to_owned(),
            read_log_evidence_ref: "audit/hr/privacy/emp_001/read-log".to_owned(),
            evaluated_at_epoch_seconds: 1_779_533_400,
        })
        .expect("disciplinary envelope is public");
        let disciplinary_schema = &disciplinary.audit_envelope.schema_version;
        assert_eq!(
            disciplinary.audit_envelope.data_kind.value,
            SensitiveHrDataKind::Disciplinary
        );
        assert_eq!(
            disciplinary.audit_envelope.payload_data_class.value,
            DataClass::SensitivePipaArticle23
        );
        assert_eq!(disciplinary_schema.value, 1);
        assert_eq!(
            disciplinary_schema.data_class.compatibility_data_class(),
            DataClass::Public
        );
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
