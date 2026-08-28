#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SensitiveHrDataKind {
    Medical,
    DisabilityAccommodation,
    Disciplinary,
    Compensation,
    GovernmentIdentifier,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SensitiveReadPurpose {
    BenefitsAdministration,
    AccommodationReview,
    PayrollAudit,
    LegalCompliance,
    GeneralBrowsing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SensitiveReadLegalBasis {
    Consent,
    EmploymentLawObligation,
    LegalClaim,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SensitiveReadDecisionStatus {
    Allowed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SensitiveHrReadInput {
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,              // data_class: INTERNAL_ONLY
    pub actor_employee_id: String,            // data_class: INTERNAL_ONLY
    pub subject_employee_id: String,          // data_class: INTERNAL_ONLY
    pub data_kind: SensitiveHrDataKind,       // data_class: SENSITIVE_PIPA_ART23
    pub purpose: SensitiveReadPurpose,        // data_class: INTERNAL_ONLY
    pub legal_basis: SensitiveReadLegalBasis, // data_class: INTERNAL_ONLY
    pub policy_ref: String,                   // data_class: INTERNAL_ONLY
    pub basis_evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub consent_evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,         // data_class: INTERNAL_ONLY
    pub read_log_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SensitiveHrReadDecision {
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub actor_employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub subject_employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub data_kind: Classified<SensitiveHrDataKind>, // data_class: SENSITIVE_PIPA_ART23
    pub purpose: Classified<SensitiveReadPurpose>, // data_class: INTERNAL_ONLY
    pub legal_basis: Classified<SensitiveReadLegalBasis>, // data_class: INTERNAL_ONLY
    pub policy_ref: Classified<PolicyRef>, // data_class: INTERNAL_ONLY
    pub basis_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub consent_evidence_ref: Classified<Option<AuditEvidenceRef>>, // data_class: INTERNAL_ONLY
    pub request_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub read_log_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub decision_status: Classified<SensitiveReadDecisionStatus>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

pub fn evaluate_sensitive_hr_read(
    input: SensitiveHrReadInput,
) -> Result<SensitiveHrReadDecision, HrDomainError> {
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        HrDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        HrDomainError::InvalidLegalEntityId,
    )?;
    validate_identifier(
        &input.actor_employee_id,
        EMPLOYEE_ID_PREFIX,
        HrDomainError::InvalidEmployeeId,
    )?;
    validate_identifier(
        &input.subject_employee_id,
        EMPLOYEE_ID_PREFIX,
        HrDomainError::InvalidEmployeeId,
    )?;
    if input.purpose == SensitiveReadPurpose::GeneralBrowsing {
        return Err(HrDomainError::DisallowedSensitiveReadPurpose);
    }
    if input.legal_basis == SensitiveReadLegalBasis::None {
        return Err(HrDomainError::MissingSensitiveReadLegalBasis);
    }
    validate_ref(
        &input.policy_ref,
        HR_POLICY_REF_PREFIX,
        HrDomainError::InvalidPolicyRef,
    )?;
    validate_evidence_ref(&input.basis_evidence_ref)?;
    let consent_evidence_ref = match input.consent_evidence_ref {
        Some(value) => {
            validate_evidence_ref(&value)?;
            Some(AuditEvidenceRef { value })
        }
        None if input.legal_basis == SensitiveReadLegalBasis::Consent => {
            return Err(HrDomainError::MissingConsentEvidence);
        }
        None => None,
    };
    validate_evidence_ref(&input.request_evidence_ref)?;
    validate_evidence_ref(&input.read_log_evidence_ref)?;
    if input.evaluated_at_epoch_seconds == 0 {
        return Err(HrDomainError::InvalidEvaluatedAt);
    }

    let idempotency_key = format!(
        "{}:{}:{:?}:{:?}:{}",
        input.tenant_id,
        input.subject_employee_id,
        input.data_kind,
        input.purpose,
        input.evaluated_at_epoch_seconds
    );

    Ok(SensitiveHrReadDecision {
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        actor_employee_id: internal(EmployeeId {
            value: input.actor_employee_id,
        }),
        subject_employee_id: internal(EmployeeId {
            value: input.subject_employee_id,
        }),
        data_kind: Classified::new(input.data_kind, DataClass::SensitivePipaArticle23),
        purpose: internal(input.purpose),
        legal_basis: internal(input.legal_basis),
        policy_ref: internal(PolicyRef {
            value: input.policy_ref,
        }),
        basis_evidence_ref: internal(AuditEvidenceRef {
            value: input.basis_evidence_ref,
        }),
        consent_evidence_ref: internal(consent_evidence_ref),
        request_evidence_ref: internal(AuditEvidenceRef {
            value: input.request_evidence_ref,
        }),
        read_log_evidence_ref: internal(AuditEvidenceRef {
            value: input.read_log_evidence_ref,
        }),
        idempotency_key: internal(idempotency_key),
        evaluated_at_epoch_seconds: internal(input.evaluated_at_epoch_seconds),
        decision_status: internal(SensitiveReadDecisionStatus::Allowed),
        schema_version: public(SENSITIVE_HR_READ_SCHEMA_VERSION),
    })
}
