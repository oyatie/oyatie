#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HrDomainError {
    InvalidEmployeeId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidPersonRef,
    InvalidManagerId,
    InvalidAuditEvidenceRef,
    InvalidHrEventId,
    InvalidLaborComplianceObligationId,
    InvalidRulepackRef,
    InvalidRulepackEffectiveDate,
    InvalidRulepackSourceRef,
    InvalidRulepackSourceVersion,
    InvalidRulepackSourceUrl,
    InvalidRulepackSourceDigest,
    InvalidRulepackSourceRetrievedAt,
    InvalidWorkflowRef,
    InvalidVersion,
    InvalidDataClass,
    InvalidEvaluatedAt,
    InvalidLeaveRequestId,
    InvalidApproverId,
    InvalidLeaveDate,
    InvalidPayrollPeriod,
    InvalidDecisionTimestamp,
    InvalidPolicyRef,
    DisallowedSensitiveReadPurpose,
    MissingSensitiveReadLegalBasis,
    MissingConsentEvidence,
    RulepackSourcesRequired,
    UnsupportedRulepackCapabilityClaim,
    InvalidAccrualUnits,
    NegativeLeaveBalance,
    CarryOverCapExceeded,
    OnboardingItemsRequired,
    DuplicateOnboardingItem,
    OnboardingItemNotCleared,
    CarryOverCapBelowFloor,
}

fn employee_id(value: &str) -> Result<EmployeeId, HrDomainError> {
    validate_identifier(value, EMPLOYEE_ID_PREFIX, HrDomainError::InvalidManagerId)?;
    Ok(EmployeeId {
        value: value.to_owned(),
    })
}

fn validate_identifier(
    value: &str,
    prefix: &str,
    error: HrDomainError,
) -> Result<(), HrDomainError> {
    let Some(suffix) = value.strip_prefix(prefix) else {
        return Err(error);
    };
    if suffix.is_empty()
        || has_unsafe_text(value)
        || suffix.contains("..")
        || !suffix
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(error);
    }
    Ok(())
}

fn validate_ref(value: &str, prefix: &str, error: HrDomainError) -> Result<(), HrDomainError> {
    let Some(suffix) = value.strip_prefix(prefix) else {
        return Err(error);
    };
    if suffix.is_empty() || has_unsafe_text(value) || value.contains('\\') {
        return Err(error);
    }
    if value
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(error);
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '.'))
    {
        return Err(error);
    }
    Ok(())
}

fn validate_evidence_ref(value: &str) -> Result<(), HrDomainError> {
    validate_ref(
        value,
        AUDIT_EVIDENCE_PREFIX,
        HrDomainError::InvalidAuditEvidenceRef,
    )?;
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("token")
        || lowered.contains("secret")
        || lowered.contains("bearer")
        || lowered.contains("password")
    {
        return Err(HrDomainError::InvalidAuditEvidenceRef);
    }
    Ok(())
}

fn validate_iso_date(value: &str) -> Result<(), HrDomainError> {
    if !is_valid_iso_date(value) {
        return Err(HrDomainError::InvalidRulepackEffectiveDate);
    }
    Ok(())
}

fn validate_source_version(value: &str) -> Result<(), HrDomainError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.len() > 96
        || has_unsafe_text(trimmed)
        || trimmed.contains("..")
        || !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
    {
        return Err(HrDomainError::InvalidRulepackSourceVersion);
    }
    Ok(())
}

fn validate_official_source_url(value: &str) -> Result<(), HrDomainError> {
    if has_unsafe_text(value) || !value.starts_with("https://") {
        return Err(HrDomainError::InvalidRulepackSourceUrl);
    }
    let allowed = [
        "https://www.moel.go.kr/",
        "https://moel.go.kr/",
        "https://law.go.kr/",
        "https://www.law.go.kr/",
        "https://www.dol.gov/",
        "https://www.eeoc.gov/",
    ];
    if !allowed.iter().any(|prefix| value.starts_with(prefix)) {
        return Err(HrDomainError::InvalidRulepackSourceUrl);
    }
    if value.contains("..") || value.contains('\\') {
        return Err(HrDomainError::InvalidRulepackSourceUrl);
    }
    Ok(())
}

fn validate_source_digest(value: &str) -> Result<(), HrDomainError> {
    let Some(hex) = value.strip_prefix(SOURCE_DIGEST_PREFIX) else {
        return Err(HrDomainError::InvalidRulepackSourceDigest);
    };
    if hex.len() != 64 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(HrDomainError::InvalidRulepackSourceDigest);
    }
    Ok(())
}

fn validate_leave_dates(start_date: &str, end_date: &str) -> Result<(), HrDomainError> {
    if !is_valid_iso_date(start_date) || !is_valid_iso_date(end_date) || start_date > end_date {
        return Err(HrDomainError::InvalidLeaveDate);
    }
    Ok(())
}

fn is_valid_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| matches!(idx, 4 | 7) || byte.is_ascii_digit())
    {
        return false;
    }
    let Ok(month) = value[5..7].parse::<u8>() else {
        return false;
    };
    let Ok(day) = value[8..10].parse::<u8>() else {
        return false;
    };
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn validate_payroll_period(value: &str) -> Result<(), HrDomainError> {
    let bytes = value.as_bytes();
    if bytes.len() != 7
        || bytes[4] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| idx == 4 || byte.is_ascii_digit())
    {
        return Err(HrDomainError::InvalidPayrollPeriod);
    }
    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| HrDomainError::InvalidPayrollPeriod)?;
    if !(1..=12).contains(&month) {
        return Err(HrDomainError::InvalidPayrollPeriod);
    }
    Ok(())
}

fn has_unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace) || value.chars().any(char::is_control)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, PrivacyDataClass::internal_only())
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}
