//! Tenant RBAC statutory filing evidence contract foundation.
//!
//! This control-plane crate defines the pre-cloud statutory filing rail evidence
//! contract that later Payroll, Accounting, Workflow, broker, and Oyatie cloud
//! adapters must satisfy before production filing evidence is claimed. It records
//! official authority surfaces, source rulepack evidence refs, payload digest
//! obligations, agency receipt obligations, credential-boundary prerequisites,
//! and legal-entity isolation controls. It deliberately does not submit filings,
//! attach agency credentials, execute payments, persist a statutory archive,
//! publish broker events, deploy cloud resources, or emit runtime audit-chain
//! events.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIREMENT_COUNT: usize = 4;
const PLAN_NAME: &str = "tenant-rbac-statutory-filing-evidence-plan";
const SERVICE_NAME: &str = "tenant-rbac";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StatutoryFilingJurisdiction {
    Korea,
    UnitedStates,
}

impl StatutoryFilingJurisdiction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Korea => "KR",
            Self::UnitedStates => "US",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StatutoryFilingRailKind {
    PayrollWithholding,
    SocialInsurance,
    ValueAddedTax,
    CorporateIncomeTax,
}

impl StatutoryFilingRailKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PayrollWithholding => "payroll_withholding",
            Self::SocialInsurance => "social_insurance",
            Self::ValueAddedTax => "value_added_tax",
            Self::CorporateIncomeTax => "corporate_income_tax",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatutoryFilingRequirement {
    pub requirement_id: &'static str,               // data_class: PUBLIC
    pub jurisdiction: StatutoryFilingJurisdiction,  // data_class: PUBLIC
    pub rail_kind: StatutoryFilingRailKind,         // data_class: PUBLIC
    pub authority_name: &'static str,               // data_class: PUBLIC
    pub authority_url: &'static str,                // data_class: PUBLIC
    pub source_rulepack_evidence_ref: &'static str, // data_class: INTERNAL_ONLY
    pub payload_schema_ref: &'static str,           // data_class: INTERNAL_ONLY
    pub payload_digest_ref: &'static str,           // data_class: INTERNAL_ONLY
    pub submission_window_ref: &'static str,        // data_class: INTERNAL_ONLY
    pub required_receipt_schema_ref: &'static str,  // data_class: INTERNAL_ONLY
    pub rollback_evidence_ref: &'static str,        // data_class: INTERNAL_ONLY
    pub credential_boundary_ref: &'static str,      // data_class: INTERNAL_ONLY
    pub legal_entity_scope_required: bool,          // data_class: PUBLIC
    pub human_approval_required: bool,              // data_class: PUBLIC
    pub agency_acceptance_receipt_required: bool,   // data_class: PUBLIC
    pub schema_version: u32,                        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacStatutoryFilingEvidencePlan {
    pub plan_name: &'static str,                       // data_class: PUBLIC
    pub service_name: &'static str,                    // data_class: PUBLIC
    pub requirements: Vec<StatutoryFilingRequirement>, // data_class: PUBLIC
    pub source_rulepack_manifests_required: bool,      // data_class: PUBLIC
    pub authority_endpoint_registry_required: bool,    // data_class: PUBLIC
    pub payload_digest_required: bool,                 // data_class: PUBLIC
    pub agency_receipt_required: bool,                 // data_class: PUBLIC
    pub legal_entity_isolation_required: bool,         // data_class: PUBLIC
    pub credential_attestation_required: bool,         // data_class: PUBLIC
    pub human_approval_required: bool,                 // data_class: PUBLIC
    pub manual_submission_workaround_allowed: bool,    // data_class: PUBLIC
    pub runtime_submission_attached: bool,             // data_class: INTERNAL_ONLY
    pub agency_credential_attached: bool,              // data_class: INTERNAL_ONLY
    pub agency_connection_attached: bool,              // data_class: INTERNAL_ONLY
    pub filing_rail_runtime_attached: bool,            // data_class: INTERNAL_ONLY
    pub disbursement_rail_attached: bool,              // data_class: INTERNAL_ONLY
    pub tax_payment_execution_attached: bool,          // data_class: INTERNAL_ONLY
    pub durable_statutory_archive_attached: bool,      // data_class: INTERNAL_ONLY
    pub cloud_deployment_attached: bool,               // data_class: INTERNAL_ONLY
    pub production_filing_evidence_attached: bool,     // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool,   // data_class: INTERNAL_ONLY
    pub schema_version: u32,                           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacStatutoryFilingEvidenceError {
    InvalidPlanName,
    InvalidServiceName,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingJurisdiction(StatutoryFilingJurisdiction),
    MissingRailKind(StatutoryFilingRailKind),
    InvalidRequirementId,
    InvalidAuthorityName,
    InvalidOfficialAuthorityUrl,
    InvalidRulepackEvidenceRef,
    InvalidPayloadSchemaRef,
    InvalidPayloadDigestRef,
    InvalidSubmissionWindowRef,
    InvalidReceiptSchemaRef,
    InvalidRollbackEvidenceRef,
    InvalidCredentialBoundaryRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_statutory_filing_evidence_plan() -> TenantRbacStatutoryFilingEvidencePlan {
    TenantRbacStatutoryFilingEvidencePlan {
        plan_name: PLAN_NAME,
        service_name: SERVICE_NAME,
        requirements: vec![
            StatutoryFilingRequirement {
                requirement_id: "kr-payroll-withholding-hometax",
                jurisdiction: StatutoryFilingJurisdiction::Korea,
                rail_kind: StatutoryFilingRailKind::PayrollWithholding,
                authority_name: "National Tax Service HomeTax",
                authority_url: "https://s.nts.go.kr/nts/cm/cntnts/cntntsView.do?cntntsId=238910&mi=40296",
                source_rulepack_evidence_ref: "evidence/multispectrum/cs-ent-payroll-statutory-rulepack-manifest-1779543600.json",
                payload_schema_ref: "schemas/statutory-filing/kr/payroll-withholding.v1.json",
                payload_digest_ref: "evidence/statutory/kr/payroll-withholding/payload-digest.jsonl",
                submission_window_ref: "calendars/statutory/kr/payroll-withholding-window",
                required_receipt_schema_ref: "schemas/statutory-filing/kr/hometax-receipt.v1.json",
                rollback_evidence_ref: "rollback/statutory/kr/payroll-withholding-filing",
                credential_boundary_ref: "secrets-boundary/statutory/kr/hometax-certificate",
                legal_entity_scope_required: true,
                human_approval_required: true,
                agency_acceptance_receipt_required: true,
                schema_version: SCHEMA_VERSION,
            },
            StatutoryFilingRequirement {
                requirement_id: "kr-social-insurance-nps-edi",
                jurisdiction: StatutoryFilingJurisdiction::Korea,
                rail_kind: StatutoryFilingRailKind::SocialInsurance,
                authority_name: "National Pension Service EDI",
                authority_url: "https://edi.nps.or.kr/",
                source_rulepack_evidence_ref: "evidence/multispectrum/cs-ent-payroll-statutory-rulepack-manifest-1779543600.json",
                payload_schema_ref: "schemas/statutory-filing/kr/social-insurance.v1.json",
                payload_digest_ref: "evidence/statutory/kr/social-insurance/payload-digest.jsonl",
                submission_window_ref: "calendars/statutory/kr/social-insurance-window",
                required_receipt_schema_ref: "schemas/statutory-filing/kr/nps-edi-receipt.v1.json",
                rollback_evidence_ref: "rollback/statutory/kr/social-insurance-filing",
                credential_boundary_ref: "secrets-boundary/statutory/kr/nps-edi-certificate",
                legal_entity_scope_required: true,
                human_approval_required: true,
                agency_acceptance_receipt_required: true,
                schema_version: SCHEMA_VERSION,
            },
            StatutoryFilingRequirement {
                requirement_id: "kr-vat-hometax",
                jurisdiction: StatutoryFilingJurisdiction::Korea,
                rail_kind: StatutoryFilingRailKind::ValueAddedTax,
                authority_name: "National Tax Service HomeTax",
                authority_url: "https://www.hometax.go.kr/",
                source_rulepack_evidence_ref: "evidence/multispectrum/cs-ent-accounting-statutory-rulepack-manifest-1779544800.json",
                payload_schema_ref: "schemas/statutory-filing/kr/vat.v1.json",
                payload_digest_ref: "evidence/statutory/kr/vat/payload-digest.jsonl",
                submission_window_ref: "calendars/statutory/kr/vat-window",
                required_receipt_schema_ref: "schemas/statutory-filing/kr/hometax-vat-receipt.v1.json",
                rollback_evidence_ref: "rollback/statutory/kr/vat-filing",
                credential_boundary_ref: "secrets-boundary/statutory/kr/hometax-vat-certificate",
                legal_entity_scope_required: true,
                human_approval_required: true,
                agency_acceptance_receipt_required: true,
                schema_version: SCHEMA_VERSION,
            },
            StatutoryFilingRequirement {
                requirement_id: "us-corporate-income-tax-mef",
                jurisdiction: StatutoryFilingJurisdiction::UnitedStates,
                rail_kind: StatutoryFilingRailKind::CorporateIncomeTax,
                authority_name: "Internal Revenue Service Modernized e-File",
                authority_url: "https://www.irs.gov/e-file-providers/modernized-e-file-overview",
                source_rulepack_evidence_ref: "evidence/multispectrum/cs-ent-accounting-statutory-rulepack-manifest-1779544800.json",
                payload_schema_ref: "schemas/statutory-filing/us/corporate-income-tax-mef.v1.json",
                payload_digest_ref: "evidence/statutory/us/corporate-income-tax/payload-digest.jsonl",
                submission_window_ref: "calendars/statutory/us/corporate-income-tax-window",
                required_receipt_schema_ref: "schemas/statutory-filing/us/mef-acknowledgement.v1.json",
                rollback_evidence_ref: "rollback/statutory/us/corporate-income-tax-filing",
                credential_boundary_ref: "secrets-boundary/statutory/us/irs-efile-provider",
                legal_entity_scope_required: true,
                human_approval_required: true,
                agency_acceptance_receipt_required: true,
                schema_version: SCHEMA_VERSION,
            },
        ],
        source_rulepack_manifests_required: true,
        authority_endpoint_registry_required: true,
        payload_digest_required: true,
        agency_receipt_required: true,
        legal_entity_isolation_required: true,
        credential_attestation_required: true,
        human_approval_required: true,
        manual_submission_workaround_allowed: false,
        runtime_submission_attached: false,
        agency_credential_attached: false,
        agency_connection_attached: false,
        filing_rail_runtime_attached: false,
        disbursement_rail_attached: false,
        tax_payment_execution_attached: false,
        durable_statutory_archive_attached: false,
        cloud_deployment_attached: false,
        production_filing_evidence_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

pub fn statutory_authority_urls(plan: &TenantRbacStatutoryFilingEvidencePlan) -> Vec<&'static str> {
    plan.requirements
        .iter()
        .map(|requirement| requirement.authority_url)
        .collect()
}

pub fn validate_tenant_rbac_statutory_filing_evidence_plan(
    plan: &TenantRbacStatutoryFilingEvidencePlan,
) -> Result<(), TenantRbacStatutoryFilingEvidenceError> {
    validate_slug(
        plan.plan_name,
        TenantRbacStatutoryFilingEvidenceError::InvalidPlanName,
    )?;
    if plan.service_name != SERVICE_NAME {
        return Err(TenantRbacStatutoryFilingEvidenceError::InvalidServiceName);
    }
    if plan.requirements.len() < MIN_REQUIREMENT_COUNT {
        return Err(TenantRbacStatutoryFilingEvidenceError::MissingRequirements);
    }

    let mut seen = BTreeSet::new();
    let mut jurisdictions = BTreeSet::new();
    let mut rail_kinds = BTreeSet::new();
    for requirement in &plan.requirements {
        validate_requirement(requirement)?;
        if !seen.insert(requirement.requirement_id) {
            return Err(
                TenantRbacStatutoryFilingEvidenceError::DuplicateRequirement(
                    requirement.requirement_id.to_owned(),
                ),
            );
        }
        jurisdictions.insert(requirement.jurisdiction);
        rail_kinds.insert(requirement.rail_kind);
    }

    for jurisdiction in [
        StatutoryFilingJurisdiction::Korea,
        StatutoryFilingJurisdiction::UnitedStates,
    ] {
        if !jurisdictions.contains(&jurisdiction) {
            return Err(TenantRbacStatutoryFilingEvidenceError::MissingJurisdiction(
                jurisdiction,
            ));
        }
    }
    for rail_kind in [
        StatutoryFilingRailKind::PayrollWithholding,
        StatutoryFilingRailKind::SocialInsurance,
        StatutoryFilingRailKind::ValueAddedTax,
        StatutoryFilingRailKind::CorporateIncomeTax,
    ] {
        if !rail_kinds.contains(&rail_kind) {
            return Err(TenantRbacStatutoryFilingEvidenceError::MissingRailKind(
                rail_kind,
            ));
        }
    }

    require_control(
        plan.source_rulepack_manifests_required,
        "source_rulepack_manifests_required",
    )?;
    require_control(
        plan.authority_endpoint_registry_required,
        "authority_endpoint_registry_required",
    )?;
    require_control(plan.payload_digest_required, "payload_digest_required")?;
    require_control(plan.agency_receipt_required, "agency_receipt_required")?;
    require_control(
        plan.legal_entity_isolation_required,
        "legal_entity_isolation_required",
    )?;
    require_control(
        plan.credential_attestation_required,
        "credential_attestation_required",
    )?;
    require_control(plan.human_approval_required, "human_approval_required")?;
    if plan.manual_submission_workaround_allowed
        || plan.runtime_submission_attached
        || plan.agency_credential_attached
        || plan.agency_connection_attached
        || plan.filing_rail_runtime_attached
        || plan.disbursement_rail_attached
        || plan.tax_payment_execution_attached
        || plan.durable_statutory_archive_attached
        || plan.cloud_deployment_attached
        || plan.production_filing_evidence_attached
        || plan.runtime_audit_chain_emission_attached
    {
        return Err(TenantRbacStatutoryFilingEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirement(
    requirement: &StatutoryFilingRequirement,
) -> Result<(), TenantRbacStatutoryFilingEvidenceError> {
    validate_slug(
        requirement.requirement_id,
        TenantRbacStatutoryFilingEvidenceError::InvalidRequirementId,
    )?;
    validate_label(
        requirement.authority_name,
        TenantRbacStatutoryFilingEvidenceError::InvalidAuthorityName,
    )?;
    validate_authority_url(requirement.authority_url)?;
    validate_prefixed_ref(
        requirement.source_rulepack_evidence_ref,
        "evidence/multispectrum/",
        TenantRbacStatutoryFilingEvidenceError::InvalidRulepackEvidenceRef,
    )?;
    validate_prefixed_ref(
        requirement.payload_schema_ref,
        "schemas/statutory-filing/",
        TenantRbacStatutoryFilingEvidenceError::InvalidPayloadSchemaRef,
    )?;
    validate_prefixed_ref(
        requirement.payload_digest_ref,
        "evidence/statutory/",
        TenantRbacStatutoryFilingEvidenceError::InvalidPayloadDigestRef,
    )?;
    validate_prefixed_ref(
        requirement.submission_window_ref,
        "calendars/statutory/",
        TenantRbacStatutoryFilingEvidenceError::InvalidSubmissionWindowRef,
    )?;
    validate_prefixed_ref(
        requirement.required_receipt_schema_ref,
        "schemas/statutory-filing/",
        TenantRbacStatutoryFilingEvidenceError::InvalidReceiptSchemaRef,
    )?;
    validate_prefixed_ref(
        requirement.rollback_evidence_ref,
        "rollback/statutory/",
        TenantRbacStatutoryFilingEvidenceError::InvalidRollbackEvidenceRef,
    )?;
    validate_prefixed_ref(
        requirement.credential_boundary_ref,
        "secrets-boundary/statutory/",
        TenantRbacStatutoryFilingEvidenceError::InvalidCredentialBoundaryRef,
    )?;
    require_control(
        requirement.legal_entity_scope_required,
        "legal_entity_scope_required",
    )?;
    require_control(
        requirement.human_approval_required,
        "requirement_human_approval_required",
    )?;
    require_control(
        requirement.agency_acceptance_receipt_required,
        "agency_acceptance_receipt_required",
    )?;
    Ok(())
}

fn validate_authority_url(url: &str) -> Result<(), TenantRbacStatutoryFilingEvidenceError> {
    if is_unsafe_ref(url)
        || ![
            "https://s.nts.go.kr/",
            "https://www.hometax.go.kr/",
            "https://edi.nps.or.kr/",
            "https://www.irs.gov/",
            "https://www.eitc.irs.gov/",
        ]
        .iter()
        .any(|prefix| url.starts_with(prefix))
    {
        return Err(TenantRbacStatutoryFilingEvidenceError::InvalidOfficialAuthorityUrl);
    }
    Ok(())
}

fn validate_label(
    value: &str,
    error: TenantRbacStatutoryFilingEvidenceError,
) -> Result<(), TenantRbacStatutoryFilingEvidenceError> {
    let lower = value.to_ascii_lowercase();
    if value.trim().is_empty()
        || value.trim() != value
        || value.contains("..")
        || lower.contains("password")
        || lower.contains("token")
        || lower.contains("private_key")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: TenantRbacStatutoryFilingEvidenceError,
) -> Result<(), TenantRbacStatutoryFilingEvidenceError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.contains("--")
        || value
            .chars()
            .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'))
    {
        return Err(error);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: TenantRbacStatutoryFilingEvidenceError,
) -> Result<(), TenantRbacStatutoryFilingEvidenceError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || is_unsafe_ref(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), TenantRbacStatutoryFilingEvidenceError> {
    if value {
        Ok(())
    } else {
        Err(TenantRbacStatutoryFilingEvidenceError::MissingRequiredControl(control))
    }
}

fn is_unsafe_ref(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.trim() != value
        || value.contains("..")
        || value.chars().any(char::is_whitespace)
        || lower.contains("password")
        || lower.contains("token")
        || lower.contains("credential=")
        || lower.contains("private_key")
}
