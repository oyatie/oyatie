//! Enterprise Suite disbursement evidence contract foundation.
//!
//! This control-plane crate defines the pre-cloud disbursement evidence contract
//! that later Payroll, Accounting, Treasury, Procurement, Workflow, bank-file,
//! and Oyatie cloud adapters must satisfy before production payment or
//! disbursement evidence is claimed. It records official network/authority
//! surfaces, source rulepack or invoice evidence refs, payment-file schema refs,
//! digest obligations, beneficiary account privacy-boundary refs, approval
//! workflow prerequisites, reconciliation receipt obligations, rollback/reversal
//! runbooks, and segregation-of-duties controls. It deliberately does not move
//! money, attach bank credentials, connect to banks/PSPs, publish broker events,
//! persist a payment archive, deploy cloud resources, or emit runtime audit-chain
//! events.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIREMENT_COUNT: usize = 4;
const PLAN_NAME: &str = "enterprise-suite-disbursement-evidence-plan";
const SERVICE_NAME: &str = "enterprise-suite";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DisbursementJurisdiction {
    Korea,
    UnitedStates,
    Europe,
}

impl DisbursementJurisdiction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Korea => "KR",
            Self::UnitedStates => "US",
            Self::Europe => "EU",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DisbursementRailKind {
    PayrollAchCredit,
    TaxPaymentEftps,
    KoreanSocialInsuranceBankTransfer,
    SepaVendorCreditTransfer,
}

impl DisbursementRailKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PayrollAchCredit => "payroll_ach_credit",
            Self::TaxPaymentEftps => "tax_payment_eftps",
            Self::KoreanSocialInsuranceBankTransfer => "korean_social_insurance_bank_transfer",
            Self::SepaVendorCreditTransfer => "sepa_vendor_credit_transfer",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisbursementRequirement {
    pub requirement_id: &'static str,            // data_class: PUBLIC
    pub jurisdiction: DisbursementJurisdiction,  // data_class: PUBLIC
    pub rail_kind: DisbursementRailKind,         // data_class: PUBLIC
    pub network_or_authority_name: &'static str, // data_class: PUBLIC
    pub network_or_authority_url: &'static str,  // data_class: PUBLIC
    pub source_evidence_ref: &'static str,       // data_class: INTERNAL_ONLY
    pub payment_file_schema_ref: &'static str,   // data_class: INTERNAL_ONLY
    pub payment_digest_ref: &'static str,        // data_class: INTERNAL_ONLY
    pub beneficiary_account_tokenization_ref: &'static str, // data_class: INTERNAL_ONLY
    pub approval_workflow_ref: &'static str,     // data_class: INTERNAL_ONLY
    pub reconciliation_receipt_schema_ref: &'static str, // data_class: INTERNAL_ONLY
    pub rollback_or_reversal_runbook_ref: &'static str, // data_class: INTERNAL_ONLY
    pub legal_entity_scope_required: bool,       // data_class: PUBLIC
    pub segregation_of_duties_required: bool,    // data_class: PUBLIC
    pub dual_approval_required: bool,            // data_class: PUBLIC
    pub reconciliation_required: bool,           // data_class: PUBLIC
    pub schema_version: u32,                     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseDisbursementEvidencePlan {
    pub plan_name: &'static str,                    // data_class: PUBLIC
    pub service_name: &'static str,                 // data_class: PUBLIC
    pub requirements: Vec<DisbursementRequirement>, // data_class: PUBLIC
    pub source_rulepack_or_invoice_evidence_required: bool, // data_class: PUBLIC
    pub bank_network_registry_required: bool,       // data_class: PUBLIC
    pub payment_file_digest_required: bool,         // data_class: PUBLIC
    pub beneficiary_tokenization_required: bool,    // data_class: PUBLIC
    pub approval_workflow_required: bool,           // data_class: PUBLIC
    pub segregation_of_duties_required: bool,       // data_class: PUBLIC
    pub dual_approval_required: bool,               // data_class: PUBLIC
    pub reconciliation_receipt_required: bool,      // data_class: PUBLIC
    pub rollback_or_reversal_runbook_required: bool, // data_class: PUBLIC
    pub manual_bank_portal_workaround_allowed: bool, // data_class: PUBLIC
    pub runtime_payment_execution_attached: bool,   // data_class: INTERNAL_ONLY
    pub bank_credential_attached: bool,             // data_class: INTERNAL_ONLY
    pub bank_connection_attached: bool,             // data_class: INTERNAL_ONLY
    pub disbursement_rail_runtime_attached: bool,   // data_class: INTERNAL_ONLY
    pub tax_payment_execution_attached: bool,       // data_class: INTERNAL_ONLY
    pub durable_payment_archive_attached: bool,     // data_class: INTERNAL_ONLY
    pub cloud_deployment_attached: bool,            // data_class: INTERNAL_ONLY
    pub production_disbursement_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnterpriseDisbursementEvidenceError {
    InvalidPlanName,
    InvalidServiceName,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingJurisdiction(DisbursementJurisdiction),
    MissingRailKind(DisbursementRailKind),
    InvalidRequirementId,
    InvalidNetworkOrAuthorityName,
    InvalidOfficialNetworkOrAuthorityUrl,
    InvalidSourceEvidenceRef,
    InvalidPaymentFileSchemaRef,
    InvalidPaymentDigestRef,
    InvalidBeneficiaryAccountTokenizationRef,
    InvalidApprovalWorkflowRef,
    InvalidReconciliationReceiptSchemaRef,
    InvalidRollbackOrReversalRunbookRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn enterprise_suite_disbursement_evidence_plan() -> EnterpriseDisbursementEvidencePlan {
    EnterpriseDisbursementEvidencePlan {
        plan_name: PLAN_NAME,
        service_name: SERVICE_NAME,
        requirements: vec![
            DisbursementRequirement {
                requirement_id: "us-payroll-ach-direct-deposit",
                jurisdiction: DisbursementJurisdiction::UnitedStates,
                rail_kind: DisbursementRailKind::PayrollAchCredit,
                network_or_authority_name: "Nacha ACH Network",
                network_or_authority_url: "https://achdevguide.nacha.org/how-ach-works",
                source_evidence_ref: "evidence/multispectrum/cs-ent-payroll-statutory-rulepack-manifest-1779543600.json",
                payment_file_schema_ref: "schemas/disbursement/us/payroll-ach-credit.nacha.v1.txt",
                payment_digest_ref: "evidence/disbursement/us/payroll-ach-credit/payment-file-digest.jsonl",
                beneficiary_account_tokenization_ref: "privacy-boundary/disbursement/us/ach-beneficiary-account-alias",
                approval_workflow_ref: "workflow/disbursement/us/payroll-ach-dual-approval",
                reconciliation_receipt_schema_ref: "schemas/disbursement-reconciliation/us/ach-return-trace.v1.json",
                rollback_or_reversal_runbook_ref: "rollback/disbursement/us/ach-reversal-return-runbook",
                legal_entity_scope_required: true,
                segregation_of_duties_required: true,
                dual_approval_required: true,
                reconciliation_required: true,
                schema_version: SCHEMA_VERSION,
            },
            DisbursementRequirement {
                requirement_id: "us-federal-tax-eftps",
                jurisdiction: DisbursementJurisdiction::UnitedStates,
                rail_kind: DisbursementRailKind::TaxPaymentEftps,
                network_or_authority_name: "Electronic Federal Tax Payment System",
                network_or_authority_url: "https://www.irs.gov/eftps",
                source_evidence_ref: "evidence/multispectrum/cs-ent-accounting-statutory-rulepack-manifest-1779544800.json",
                payment_file_schema_ref: "schemas/disbursement/us/eftps-batch-provider.v1.json",
                payment_digest_ref: "evidence/disbursement/us/eftps/payment-instruction-digest.jsonl",
                beneficiary_account_tokenization_ref: "privacy-boundary/disbursement/us/eftps-originator-account-alias",
                approval_workflow_ref: "workflow/disbursement/us/tax-payment-dual-approval",
                reconciliation_receipt_schema_ref: "schemas/disbursement-reconciliation/us/eftps-acknowledgement.v1.json",
                rollback_or_reversal_runbook_ref: "rollback/disbursement/us/eftps-cancel-or-amend-runbook",
                legal_entity_scope_required: true,
                segregation_of_duties_required: true,
                dual_approval_required: true,
                reconciliation_required: true,
                schema_version: SCHEMA_VERSION,
            },
            DisbursementRequirement {
                requirement_id: "kr-social-insurance-bank-transfer",
                jurisdiction: DisbursementJurisdiction::Korea,
                rail_kind: DisbursementRailKind::KoreanSocialInsuranceBankTransfer,
                network_or_authority_name: "Korea Financial Telecommunications and Clearings Institute IFT Network",
                network_or_authority_url: "https://eng.kftc.or.kr/business",
                source_evidence_ref: "evidence/multispectrum/cs-ent-hr-statutory-rulepack-manifest-1779544200.json",
                payment_file_schema_ref: "schemas/disbursement/kr/social-insurance-bank-transfer.v1.json",
                payment_digest_ref: "evidence/disbursement/kr/social-insurance/payment-file-digest.jsonl",
                beneficiary_account_tokenization_ref: "privacy-boundary/disbursement/kr/social-insurance-beneficiary-account-alias",
                approval_workflow_ref: "workflow/disbursement/kr/social-insurance-dual-approval",
                reconciliation_receipt_schema_ref: "schemas/disbursement-reconciliation/kr/ift-receipt.v1.json",
                rollback_or_reversal_runbook_ref: "rollback/disbursement/kr/ift-correction-runbook",
                legal_entity_scope_required: true,
                segregation_of_duties_required: true,
                dual_approval_required: true,
                reconciliation_required: true,
                schema_version: SCHEMA_VERSION,
            },
            DisbursementRequirement {
                requirement_id: "eu-vendor-sepa-credit-transfer",
                jurisdiction: DisbursementJurisdiction::Europe,
                rail_kind: DisbursementRailKind::SepaVendorCreditTransfer,
                network_or_authority_name: "European Payments Council SEPA Credit Transfer",
                network_or_authority_url: "https://www.europeanpaymentscouncil.eu/what-we-do/epc-payment-scheme-management",
                source_evidence_ref: "evidence/multispectrum/cs-ent-procurement-source-to-pay-domain-1779545400.json",
                payment_file_schema_ref: "schemas/disbursement/eu/sepa-credit-transfer.iso20022-pain001.v1.xml",
                payment_digest_ref: "evidence/disbursement/eu/sepa-credit-transfer/payment-file-digest.jsonl",
                beneficiary_account_tokenization_ref: "privacy-boundary/disbursement/eu/sepa-beneficiary-iban-alias",
                approval_workflow_ref: "workflow/disbursement/eu/vendor-sepa-dual-approval",
                reconciliation_receipt_schema_ref: "schemas/disbursement-reconciliation/eu/sepa-camt054-receipt.v1.json",
                rollback_or_reversal_runbook_ref: "rollback/disbursement/eu/sepa-recall-investigation-runbook",
                legal_entity_scope_required: true,
                segregation_of_duties_required: true,
                dual_approval_required: true,
                reconciliation_required: true,
                schema_version: SCHEMA_VERSION,
            },
        ],
        source_rulepack_or_invoice_evidence_required: true,
        bank_network_registry_required: true,
        payment_file_digest_required: true,
        beneficiary_tokenization_required: true,
        approval_workflow_required: true,
        segregation_of_duties_required: true,
        dual_approval_required: true,
        reconciliation_receipt_required: true,
        rollback_or_reversal_runbook_required: true,
        manual_bank_portal_workaround_allowed: false,
        runtime_payment_execution_attached: false,
        bank_credential_attached: false,
        bank_connection_attached: false,
        disbursement_rail_runtime_attached: false,
        tax_payment_execution_attached: false,
        durable_payment_archive_attached: false,
        cloud_deployment_attached: false,
        production_disbursement_evidence_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

pub fn disbursement_network_or_authority_urls(
    plan: &EnterpriseDisbursementEvidencePlan,
) -> Vec<&'static str> {
    plan.requirements
        .iter()
        .map(|requirement| requirement.network_or_authority_url)
        .collect()
}

pub fn validate_enterprise_suite_disbursement_evidence_plan(
    plan: &EnterpriseDisbursementEvidencePlan,
) -> Result<(), EnterpriseDisbursementEvidenceError> {
    validate_slug(
        plan.plan_name,
        EnterpriseDisbursementEvidenceError::InvalidPlanName,
    )?;
    if plan.service_name != SERVICE_NAME {
        return Err(EnterpriseDisbursementEvidenceError::InvalidServiceName);
    }
    if plan.requirements.len() < MIN_REQUIREMENT_COUNT {
        return Err(EnterpriseDisbursementEvidenceError::MissingRequirements);
    }

    let mut seen = BTreeSet::new();
    let mut jurisdictions = BTreeSet::new();
    let mut rail_kinds = BTreeSet::new();
    for requirement in &plan.requirements {
        validate_requirement(requirement)?;
        if !seen.insert(requirement.requirement_id) {
            return Err(EnterpriseDisbursementEvidenceError::DuplicateRequirement(
                requirement.requirement_id.to_owned(),
            ));
        }
        jurisdictions.insert(requirement.jurisdiction);
        rail_kinds.insert(requirement.rail_kind);
    }

    for jurisdiction in [
        DisbursementJurisdiction::Korea,
        DisbursementJurisdiction::UnitedStates,
        DisbursementJurisdiction::Europe,
    ] {
        if !jurisdictions.contains(&jurisdiction) {
            return Err(EnterpriseDisbursementEvidenceError::MissingJurisdiction(
                jurisdiction,
            ));
        }
    }
    for rail_kind in [
        DisbursementRailKind::PayrollAchCredit,
        DisbursementRailKind::TaxPaymentEftps,
        DisbursementRailKind::KoreanSocialInsuranceBankTransfer,
        DisbursementRailKind::SepaVendorCreditTransfer,
    ] {
        if !rail_kinds.contains(&rail_kind) {
            return Err(EnterpriseDisbursementEvidenceError::MissingRailKind(
                rail_kind,
            ));
        }
    }

    require_control(
        plan.source_rulepack_or_invoice_evidence_required,
        "source_rulepack_or_invoice_evidence_required",
    )?;
    require_control(
        plan.bank_network_registry_required,
        "bank_network_registry_required",
    )?;
    require_control(
        plan.payment_file_digest_required,
        "payment_file_digest_required",
    )?;
    require_control(
        plan.beneficiary_tokenization_required,
        "beneficiary_tokenization_required",
    )?;
    require_control(
        plan.approval_workflow_required,
        "approval_workflow_required",
    )?;
    require_control(
        plan.segregation_of_duties_required,
        "segregation_of_duties_required",
    )?;
    require_control(plan.dual_approval_required, "dual_approval_required")?;
    require_control(
        plan.reconciliation_receipt_required,
        "reconciliation_receipt_required",
    )?;
    require_control(
        plan.rollback_or_reversal_runbook_required,
        "rollback_or_reversal_runbook_required",
    )?;
    if plan.manual_bank_portal_workaround_allowed
        || plan.runtime_payment_execution_attached
        || plan.bank_credential_attached
        || plan.bank_connection_attached
        || plan.disbursement_rail_runtime_attached
        || plan.tax_payment_execution_attached
        || plan.durable_payment_archive_attached
        || plan.cloud_deployment_attached
        || plan.production_disbursement_evidence_attached
        || plan.runtime_audit_chain_emission_attached
    {
        return Err(EnterpriseDisbursementEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_requirement(
    requirement: &DisbursementRequirement,
) -> Result<(), EnterpriseDisbursementEvidenceError> {
    validate_slug(
        requirement.requirement_id,
        EnterpriseDisbursementEvidenceError::InvalidRequirementId,
    )?;
    validate_label(
        requirement.network_or_authority_name,
        EnterpriseDisbursementEvidenceError::InvalidNetworkOrAuthorityName,
    )?;
    validate_network_or_authority_url(requirement.network_or_authority_url)?;
    validate_prefixed_ref(
        requirement.source_evidence_ref,
        "evidence/multispectrum/",
        EnterpriseDisbursementEvidenceError::InvalidSourceEvidenceRef,
    )?;
    validate_prefixed_ref(
        requirement.payment_file_schema_ref,
        "schemas/disbursement/",
        EnterpriseDisbursementEvidenceError::InvalidPaymentFileSchemaRef,
    )?;
    validate_prefixed_ref(
        requirement.payment_digest_ref,
        "evidence/disbursement/",
        EnterpriseDisbursementEvidenceError::InvalidPaymentDigestRef,
    )?;
    validate_prefixed_ref(
        requirement.beneficiary_account_tokenization_ref,
        "privacy-boundary/disbursement/",
        EnterpriseDisbursementEvidenceError::InvalidBeneficiaryAccountTokenizationRef,
    )?;
    validate_prefixed_ref(
        requirement.approval_workflow_ref,
        "workflow/disbursement/",
        EnterpriseDisbursementEvidenceError::InvalidApprovalWorkflowRef,
    )?;
    validate_prefixed_ref(
        requirement.reconciliation_receipt_schema_ref,
        "schemas/disbursement-reconciliation/",
        EnterpriseDisbursementEvidenceError::InvalidReconciliationReceiptSchemaRef,
    )?;
    validate_prefixed_ref(
        requirement.rollback_or_reversal_runbook_ref,
        "rollback/disbursement/",
        EnterpriseDisbursementEvidenceError::InvalidRollbackOrReversalRunbookRef,
    )?;
    require_control(
        requirement.legal_entity_scope_required,
        "legal_entity_scope_required",
    )?;
    require_control(
        requirement.segregation_of_duties_required,
        "requirement_segregation_of_duties_required",
    )?;
    require_control(
        requirement.dual_approval_required,
        "requirement_dual_approval_required",
    )?;
    require_control(
        requirement.reconciliation_required,
        "reconciliation_required",
    )?;
    Ok(())
}

fn validate_network_or_authority_url(url: &str) -> Result<(), EnterpriseDisbursementEvidenceError> {
    if is_unsafe_ref(url)
        || ![
            "https://www.nacha.org/",
            "https://achdevguide.nacha.org/",
            "https://www.irs.gov/",
            "https://fiscal.treasury.gov/",
            "https://www.eftps.gov/",
            "https://www.iso20022.org/",
            "https://www.europeanpaymentscouncil.eu/",
            "https://eng.kftc.or.kr/",
            "https://www.kftc.or.kr/",
        ]
        .iter()
        .any(|prefix| url.starts_with(prefix))
    {
        return Err(EnterpriseDisbursementEvidenceError::InvalidOfficialNetworkOrAuthorityUrl);
    }
    Ok(())
}

fn validate_label(
    value: &str,
    error: EnterpriseDisbursementEvidenceError,
) -> Result<(), EnterpriseDisbursementEvidenceError> {
    let lower = value.to_ascii_lowercase();
    if value.trim().is_empty()
        || value.trim() != value
        || value.contains("..")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("credential=")
        || lower.contains("private_key")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: EnterpriseDisbursementEvidenceError,
) -> Result<(), EnterpriseDisbursementEvidenceError> {
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
    error: EnterpriseDisbursementEvidenceError,
) -> Result<(), EnterpriseDisbursementEvidenceError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || is_unsafe_ref(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), EnterpriseDisbursementEvidenceError> {
    if value {
        Ok(())
    } else {
        Err(EnterpriseDisbursementEvidenceError::MissingRequiredControl(
            control,
        ))
    }
}

fn is_unsafe_ref(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.trim() != value
        || value.contains("..")
        || value.chars().any(char::is_whitespace)
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("credential=")
        || lower.contains("private_key")
        || lower.contains("api_key")
        || lower.contains("bearer")
}
