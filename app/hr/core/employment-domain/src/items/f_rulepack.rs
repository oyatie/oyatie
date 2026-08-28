#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum HrRulepackSourceKind {
    LaborStandards,
    RulesOfEmployment,
    LaborManagementCouncil,
    LeaveAndHolidayStandards,
    WageHourRecordkeeping,
    EqualEmployment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrRulepackSourceInput {
    pub source_kind: HrRulepackSourceKind, // data_class: INTERNAL_ONLY
    pub source_ref: String,                // data_class: INTERNAL_ONLY
    pub official_url: String,              // data_class: PUBLIC
    pub version_label: String,             // data_class: INTERNAL_ONLY
    pub effective_date: String,            // data_class: INTERNAL_ONLY
    pub retrieved_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub evidence_ref: String,              // data_class: INTERNAL_ONLY
    pub digest: String,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrStatutoryRulepackManifestInput {
    pub rulepack_ref: String,                 // data_class: INTERNAL_ONLY
    pub jurisdiction: Jurisdiction,           // data_class: INTERNAL_ONLY
    pub source_version: String,               // data_class: INTERNAL_ONLY
    pub effective_date: String,               // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub sources: Vec<HrRulepackSourceInput>,  // data_class: INTERNAL_ONLY
    pub labor_workflow_engine_attached: bool, // data_class: PUBLIC
    pub payroll_calculation_attached: bool,   // data_class: PUBLIC
    pub filing_rail_attached: bool,           // data_class: PUBLIC
    pub cloud_deployment_attached: bool,      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrRulepackSource {
    pub source_kind: Classified<HrRulepackSourceKind>, // data_class: INTERNAL_ONLY
    pub source_ref: Classified<String>,                // data_class: INTERNAL_ONLY
    pub official_url: Classified<String>,              // data_class: PUBLIC
    pub version_label: Classified<String>,             // data_class: INTERNAL_ONLY
    pub effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub retrieved_at_epoch_seconds: Classified<u64>,   // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<AuditEvidenceRef>,    // data_class: INTERNAL_ONLY
    pub digest: Classified<RulepackSourceDigest>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrStatutoryRulepackManifest {
    pub rulepack_ref: Classified<RulepackRef>, // data_class: INTERNAL_ONLY
    pub jurisdiction: Classified<Jurisdiction>, // data_class: INTERNAL_ONLY
    pub source_version: Classified<String>,    // data_class: INTERNAL_ONLY
    pub effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub sources: Classified<Vec<HrRulepackSource>>, // data_class: INTERNAL_ONLY
    pub source_count: Classified<usize>,       // data_class: PUBLIC
    pub labor_workflow_engine_attached: Classified<bool>, // data_class: PUBLIC
    pub payroll_calculation_attached: Classified<bool>, // data_class: PUBLIC
    pub filing_rail_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

pub fn build_hr_statutory_rulepack_manifest(
    input: HrStatutoryRulepackManifestInput,
) -> Result<HrStatutoryRulepackManifest, HrDomainError> {
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        HrDomainError::InvalidRulepackRef,
    )?;
    validate_source_version(&input.source_version)?;
    validate_iso_date(&input.effective_date)?;
    validate_evidence_ref(&input.approval_evidence_ref)?;
    if input.sources.is_empty() {
        return Err(HrDomainError::RulepackSourcesRequired);
    }
    if input.labor_workflow_engine_attached
        || input.payroll_calculation_attached
        || input.filing_rail_attached
        || input.cloud_deployment_attached
    {
        return Err(HrDomainError::UnsupportedRulepackCapabilityClaim);
    }

    let source_count = input.sources.len();
    let mut sources = Vec::with_capacity(source_count);
    for source in input.sources {
        sources.push(build_hr_rulepack_source(source)?);
    }

    Ok(HrStatutoryRulepackManifest {
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        jurisdiction: internal(input.jurisdiction),
        source_version: internal(input.source_version),
        effective_date: internal(RulepackEffectiveDate {
            value: input.effective_date,
        }),
        approval_evidence_ref: internal(AuditEvidenceRef {
            value: input.approval_evidence_ref,
        }),
        sources: internal(sources),
        source_count: public(source_count),
        labor_workflow_engine_attached: public(false),
        payroll_calculation_attached: public(false),
        filing_rail_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(HR_STATUTORY_RULEPACK_SCHEMA_VERSION),
    })
}

fn build_hr_rulepack_source(
    source: HrRulepackSourceInput,
) -> Result<HrRulepackSource, HrDomainError> {
    validate_ref(
        &source.source_ref,
        HR_RULEPACK_SOURCE_REF_PREFIX,
        HrDomainError::InvalidRulepackSourceRef,
    )?;
    validate_official_source_url(&source.official_url)?;
    validate_source_version(&source.version_label)?;
    validate_iso_date(&source.effective_date)?;
    if source.retrieved_at_epoch_seconds == 0 {
        return Err(HrDomainError::InvalidRulepackSourceRetrievedAt);
    }
    validate_evidence_ref(&source.evidence_ref)?;
    validate_source_digest(&source.digest)?;

    Ok(HrRulepackSource {
        source_kind: internal(source.source_kind),
        source_ref: internal(source.source_ref),
        official_url: public(source.official_url),
        version_label: internal(source.version_label),
        effective_date: internal(RulepackEffectiveDate {
            value: source.effective_date,
        }),
        retrieved_at_epoch_seconds: internal(source.retrieved_at_epoch_seconds),
        evidence_ref: internal(AuditEvidenceRef {
            value: source.evidence_ref,
        }),
        digest: internal(RulepackSourceDigest {
            value: source.digest,
        }),
    })
}
