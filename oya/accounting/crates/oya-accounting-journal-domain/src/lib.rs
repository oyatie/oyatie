//! Accounting journal domain foundation.
//!
//! This crate owns pure accounting invariants for balanced journal posting,
//! payroll source-digest intake, VAT evidence workflow detection, AP approval
//! gates, and close-evidence refusal. It does not perform persistence, tax
//! filing, payment execution, workflow dispatch, or report-generation I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const JOURNAL_ID_PREFIX: &str = "jrn_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const HASH_PREFIX: &str = "sha256:";
const VAT_RETURN_ID_PREFIX: &str = "vat_";
const INVOICE_ID_PREFIX: &str = "apinv_";
const CLOSE_ID_PREFIX: &str = "close_";
const WORKFLOW_REF_PREFIX: &str = "workflow/";
const RULEPACK_REF_PREFIX: &str = "rulepack/";
const ACCOUNTING_RULEPACK_SOURCE_REF_PREFIX: &str = "accounting-rulepack-source/";
const ACCOUNTING_STATUTORY_RULEPACK_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct JournalId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TenantId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LegalEntityId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceDigest {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceDocumentRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VatReturnId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InvoiceId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloseId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorkflowRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RulepackRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RulepackEffectiveDate {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PeriodState {
    Open,
    SoftClosed,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum JournalState {
    Draft,
    Posted,
    Reversed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Jurisdiction {
    Korea,
    UnitedStates,
    EuropeanUnion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VatWorkflowStep {
    HomeTaxExportHashAttached,
    ReviewerAssigned,
    EvidencePackAttached,
    ReadyForFiling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AccountingRulepackSourceKind {
    VatFilingDeadline,
    ElectronicTaxFiling,
    CorporateIncomeTax,
    BusinessTaxReturn,
    TaxRecordkeeping,
    StatutoryFormSchema,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApApprovalCheck {
    Policy,
    Budget,
    Vendor,
    Evidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalLineInput {
    pub account_code: String, // data_class: INTERNAL_ONLY
    pub debit_minor: i64,     // data_class: INTERNAL_ONLY
    pub credit_minor: i64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalPostInput {
    pub journal_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub period: String,                // data_class: INTERNAL_ONLY
    pub period_state: PeriodState,     // data_class: INTERNAL_ONLY
    pub source_documents: Vec<String>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub lines: Vec<JournalLineInput>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalLine {
    pub account_code: Classified<String>, // data_class: INTERNAL_ONLY
    pub debit_minor: Classified<i64>,     // data_class: INTERNAL_ONLY
    pub credit_minor: Classified<i64>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalVoucher {
    pub journal_id: Classified<JournalId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,   // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub period: Classified<String>,        // data_class: INTERNAL_ONLY
    pub state: Classified<JournalState>,   // data_class: INTERNAL_ONLY
    pub source_documents: Classified<Vec<SourceDocumentRef>>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub lines: Classified<Vec<JournalLine>>, // data_class: INTERNAL_ONLY
    pub total_debit_minor: Classified<i64>, // data_class: INTERNAL_ONLY
    pub total_credit_minor: Classified<i64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollPostingInput {
    pub journal_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub period: String,                // data_class: INTERNAL_ONLY
    pub source_payroll_digest: String, // data_class: INTERNAL_ONLY
    pub wage_ledger_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub reversal_path_ref: String,     // data_class: INTERNAL_ONLY
    pub lines: Vec<JournalLineInput>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollPostingEvidence {
    pub journal: JournalVoucher, // data_class: INTERNAL_ONLY
    pub source_payroll_digest: Classified<EvidenceDigest>, // data_class: INTERNAL_ONLY
    pub wage_ledger_refs: Classified<Vec<EvidenceRef>>, // data_class: INTERNAL_ONLY
    pub reversal_path_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VatDeadlineInput {
    pub return_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,     // data_class: INTERNAL_ONLY
    pub jurisdiction: Jurisdiction,  // data_class: INTERNAL_ONLY
    pub period: String,              // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub workflow_ref: String,        // data_class: INTERNAL_ONLY
    pub hometax_export_hash: String, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VatReturnWorkflow {
    pub return_id: Classified<VatReturnId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,    // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub period: Classified<String>,         // data_class: INTERNAL_ONLY
    pub workflow_ref: Classified<WorkflowRef>, // data_class: INTERNAL_ONLY
    pub hometax_export_hash: Classified<EvidenceDigest>, // data_class: INTERNAL_ONLY
    pub evidence_paths: Classified<Vec<EvidenceRef>>, // data_class: INTERNAL_ONLY
    pub required_steps: Classified<Vec<VatWorkflowStep>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApInvoiceInput {
    pub invoice_id: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,     // data_class: INTERNAL_ONLY
    pub vendor_ref: String,          // data_class: INTERNAL_ONLY
    pub amount_minor: i64,           // data_class: INTERNAL_ONLY
    pub policy_threshold_minor: i64, // data_class: INTERNAL_ONLY
    pub budget_ref: String,          // data_class: INTERNAL_ONLY
    pub evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub approved: bool,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApApprovalRoute {
    pub invoice_id: Classified<InvoiceId>, // data_class: INTERNAL_ONLY
    pub required_checks: Classified<Vec<ApApprovalCheck>>, // data_class: INTERNAL_ONLY
    pub liability_post_allowed: Classified<bool>, // data_class: INTERNAL_ONLY
    pub payment_request_allowed: Classified<bool>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosePromotionInput {
    pub close_id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                 // data_class: INTERNAL_ONLY
    pub period: String,                          // data_class: INTERNAL_ONLY
    pub required_evidence_refs: Vec<String>,     // data_class: INTERNAL_ONLY
    pub manual_shell_workaround_requested: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosePromotion {
    pub close_id: Classified<CloseId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub promoted: Classified<bool>,      // data_class: INTERNAL_ONLY
    pub evidence_refs: Classified<Vec<EvidenceRef>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingRulepackSourceInput {
    pub source_kind: AccountingRulepackSourceKind, // data_class: INTERNAL_ONLY
    pub source_ref: String,                        // data_class: INTERNAL_ONLY
    pub official_url: String,                      // data_class: PUBLIC
    pub version_label: String,                     // data_class: INTERNAL_ONLY
    pub effective_date: String,                    // data_class: INTERNAL_ONLY
    pub retrieved_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                      // data_class: INTERNAL_ONLY
    pub digest: String,                            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingStatutoryRulepackManifestInput {
    pub rulepack_ref: String,                        // data_class: INTERNAL_ONLY
    pub jurisdiction: Jurisdiction,                  // data_class: INTERNAL_ONLY
    pub accounting_period: String,                   // data_class: FINANCIAL
    pub source_version: String,                      // data_class: INTERNAL_ONLY
    pub effective_date: String,                      // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub sources: Vec<AccountingRulepackSourceInput>, // data_class: INTERNAL_ONLY
    pub ledger_persistence_attached: bool,           // data_class: PUBLIC
    pub workflow_engine_attached: bool,              // data_class: PUBLIC
    pub statutory_filing_rail_attached: bool,        // data_class: PUBLIC
    pub payment_execution_attached: bool,            // data_class: PUBLIC
    pub cloud_deployment_attached: bool,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingRulepackSource {
    pub source_kind: Classified<AccountingRulepackSourceKind>, // data_class: INTERNAL_ONLY
    pub source_ref: Classified<String>,                        // data_class: INTERNAL_ONLY
    pub official_url: Classified<String>,                      // data_class: PUBLIC
    pub version_label: Classified<String>,                     // data_class: INTERNAL_ONLY
    pub effective_date: Classified<RulepackEffectiveDate>,     // data_class: INTERNAL_ONLY
    pub retrieved_at_epoch_seconds: Classified<u64>,           // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<EvidenceRef>,                 // data_class: INTERNAL_ONLY
    pub digest: Classified<EvidenceDigest>,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingStatutoryRulepackManifest {
    pub rulepack_ref: Classified<RulepackRef>, // data_class: INTERNAL_ONLY
    pub jurisdiction: Classified<Jurisdiction>, // data_class: INTERNAL_ONLY
    pub accounting_period: Classified<String>, // data_class: FINANCIAL
    pub source_version: Classified<String>,    // data_class: INTERNAL_ONLY
    pub effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub sources: Classified<Vec<AccountingRulepackSource>>, // data_class: INTERNAL_ONLY
    pub source_count: Classified<usize>,       // data_class: PUBLIC
    pub ledger_persistence_attached: Classified<bool>, // data_class: PUBLIC
    pub workflow_engine_attached: Classified<bool>, // data_class: PUBLIC
    pub statutory_filing_rail_attached: Classified<bool>, // data_class: PUBLIC
    pub payment_execution_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountingDomainError {
    InvalidJournalId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidPeriod,
    PeriodNotOpen,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidEvidenceDigest,
    InvalidAccountCode,
    InvalidMoney,
    JournalLinesRequired,
    UnbalancedJournal,
    PayrollDigestRequired,
    WageLedgerRefsRequired,
    InvalidVatReturnId,
    VatDeadlineNotReached,
    UnsupportedJurisdiction,
    InvalidWorkflowRef,
    InvalidRulepackRef,
    InvalidRulepackEffectiveDate,
    InvalidRulepackSourceRef,
    InvalidRulepackSourceVersion,
    InvalidRulepackSourceUrl,
    InvalidRulepackSourceRetrievedAt,
    RulepackSourcesRequired,
    UnsupportedRulepackCapabilityClaim,
    InvalidInvoiceId,
    InvoiceApprovalRequired,
    InvalidCloseId,
    MissingCloseEvidence,
    ManualShellWorkaroundRefused,
}

pub fn post_journal(input: JournalPostInput) -> Result<JournalVoucher, AccountingDomainError> {
    validate_journal_header(
        &input.journal_id,
        &input.tenant_id,
        &input.legal_entity_id,
        &input.period,
    )?;
    if input.period_state != PeriodState::Open {
        return Err(AccountingDomainError::PeriodNotOpen);
    }
    if input.source_documents.is_empty() {
        return Err(AccountingDomainError::InvalidSourceDocumentRef);
    }
    let source_documents = input
        .source_documents
        .into_iter()
        .map(|source| {
            validate_ref(
                &source,
                SOURCE_REF_PREFIX,
                AccountingDomainError::InvalidSourceDocumentRef,
            )?;
            Ok(SourceDocumentRef { value: source })
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_ref(
        &input.approval_evidence_ref,
        AUDIT_REF_PREFIX,
        AccountingDomainError::InvalidEvidenceRef,
    )?;
    let (lines, debit, credit) = build_lines(input.lines)?;
    Ok(JournalVoucher {
        journal_id: internal(JournalId {
            value: input.journal_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        period: internal(input.period),
        state: internal(JournalState::Posted),
        source_documents: internal(source_documents),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        lines: internal(lines),
        total_debit_minor: internal(debit),
        total_credit_minor: internal(credit),
        schema_version: public(1),
    })
}

pub fn payroll_posting(
    input: PayrollPostingInput,
) -> Result<PayrollPostingEvidence, AccountingDomainError> {
    payroll_posting_for_period(input, PeriodState::Open)
}

pub fn payroll_posting_for_period(
    input: PayrollPostingInput,
    period_state: PeriodState,
) -> Result<PayrollPostingEvidence, AccountingDomainError> {
    if input.source_payroll_digest.trim().is_empty() {
        return Err(AccountingDomainError::PayrollDigestRequired);
    }
    validate_digest(&input.source_payroll_digest)?;
    if input.wage_ledger_refs.is_empty() {
        return Err(AccountingDomainError::WageLedgerRefsRequired);
    }
    let wage_ledger_refs = input
        .wage_ledger_refs
        .iter()
        .map(|source| {
            validate_ref(
                source,
                AUDIT_REF_PREFIX,
                AccountingDomainError::InvalidEvidenceRef,
            )?;
            Ok(EvidenceRef {
                value: source.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_ref(
        &input.reversal_path_ref,
        AUDIT_REF_PREFIX,
        AccountingDomainError::InvalidEvidenceRef,
    )?;
    let journal = post_journal(JournalPostInput {
        journal_id: input.journal_id,
        tenant_id: input.tenant_id,
        legal_entity_id: input.legal_entity_id,
        period: input.period,
        period_state,
        source_documents: vec!["src/payroll/run".to_owned()],
        approval_evidence_ref: input.approval_evidence_ref,
        lines: input.lines,
    })?;
    Ok(PayrollPostingEvidence {
        journal,
        source_payroll_digest: internal(EvidenceDigest {
            value: input.source_payroll_digest,
        }),
        wage_ledger_refs: internal(wage_ledger_refs),
        reversal_path_ref: internal(EvidenceRef {
            value: input.reversal_path_ref,
        }),
    })
}

pub fn evaluate_vat_deadline(
    input: VatDeadlineInput,
) -> Result<Option<VatReturnWorkflow>, AccountingDomainError> {
    validate_identifier(
        &input.return_id,
        VAT_RETURN_ID_PREFIX,
        AccountingDomainError::InvalidVatReturnId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        AccountingDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        AccountingDomainError::InvalidLegalEntityId,
    )?;
    validate_period(&input.period)?;
    validate_ref(
        &input.workflow_ref,
        WORKFLOW_REF_PREFIX,
        AccountingDomainError::InvalidWorkflowRef,
    )?;
    validate_digest(&input.hometax_export_hash)?;
    validate_ref(
        &input.evidence_ref,
        AUDIT_REF_PREFIX,
        AccountingDomainError::InvalidEvidenceRef,
    )?;
    if input.jurisdiction != Jurisdiction::Korea {
        return Ok(None);
    }
    if input.now_epoch_seconds < input.deadline_epoch_seconds {
        return Err(AccountingDomainError::VatDeadlineNotReached);
    }
    Ok(Some(VatReturnWorkflow {
        return_id: internal(VatReturnId {
            value: input.return_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        period: internal(input.period),
        workflow_ref: internal(WorkflowRef {
            value: input.workflow_ref,
        }),
        hometax_export_hash: internal(EvidenceDigest {
            value: input.hometax_export_hash,
        }),
        evidence_paths: internal(vec![EvidenceRef {
            value: input.evidence_ref,
        }]),
        required_steps: internal(vec![
            VatWorkflowStep::HomeTaxExportHashAttached,
            VatWorkflowStep::ReviewerAssigned,
            VatWorkflowStep::EvidencePackAttached,
            VatWorkflowStep::ReadyForFiling,
        ]),
    }))
}

pub fn evaluate_invoice_approval(
    input: ApInvoiceInput,
) -> Result<ApApprovalRoute, AccountingDomainError> {
    validate_identifier(
        &input.invoice_id,
        INVOICE_ID_PREFIX,
        AccountingDomainError::InvalidInvoiceId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        AccountingDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        AccountingDomainError::InvalidLegalEntityId,
    )?;
    validate_ref(
        &input.vendor_ref,
        SOURCE_REF_PREFIX,
        AccountingDomainError::InvalidSourceDocumentRef,
    )?;
    validate_ref(
        &input.budget_ref,
        SOURCE_REF_PREFIX,
        AccountingDomainError::InvalidSourceDocumentRef,
    )?;
    validate_ref(
        &input.evidence_ref,
        AUDIT_REF_PREFIX,
        AccountingDomainError::InvalidEvidenceRef,
    )?;
    if input.amount_minor <= 0 || input.policy_threshold_minor <= 0 {
        return Err(AccountingDomainError::InvalidMoney);
    }
    let required_checks = vec![
        ApApprovalCheck::Policy,
        ApApprovalCheck::Budget,
        ApApprovalCheck::Vendor,
        ApApprovalCheck::Evidence,
    ];
    if input.amount_minor >= input.policy_threshold_minor && !input.approved {
        return Ok(ApApprovalRoute {
            invoice_id: internal(InvoiceId {
                value: input.invoice_id,
            }),
            required_checks: internal(required_checks),
            liability_post_allowed: internal(false),
            payment_request_allowed: internal(false),
        });
    }
    Ok(ApApprovalRoute {
        invoice_id: internal(InvoiceId {
            value: input.invoice_id,
        }),
        required_checks: internal(required_checks),
        liability_post_allowed: internal(true),
        payment_request_allowed: internal(true),
    })
}

pub fn promote_close(input: ClosePromotionInput) -> Result<ClosePromotion, AccountingDomainError> {
    validate_identifier(
        &input.close_id,
        CLOSE_ID_PREFIX,
        AccountingDomainError::InvalidCloseId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        AccountingDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        AccountingDomainError::InvalidLegalEntityId,
    )?;
    validate_period(&input.period)?;
    if input.manual_shell_workaround_requested {
        return Err(AccountingDomainError::ManualShellWorkaroundRefused);
    }
    if input.required_evidence_refs.is_empty() {
        return Err(AccountingDomainError::MissingCloseEvidence);
    }
    let evidence_refs = input
        .required_evidence_refs
        .into_iter()
        .map(|evidence| {
            validate_ref(
                &evidence,
                AUDIT_REF_PREFIX,
                AccountingDomainError::InvalidEvidenceRef,
            )?;
            Ok(EvidenceRef { value: evidence })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ClosePromotion {
        close_id: internal(CloseId {
            value: input.close_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        promoted: internal(true),
        evidence_refs: internal(evidence_refs),
    })
}

pub fn build_accounting_statutory_rulepack_manifest(
    input: AccountingStatutoryRulepackManifestInput,
) -> Result<AccountingStatutoryRulepackManifest, AccountingDomainError> {
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        AccountingDomainError::InvalidRulepackRef,
    )?;
    validate_period(&input.accounting_period)?;
    validate_source_version(&input.source_version)?;
    validate_iso_date(&input.effective_date)?;
    validate_ref(
        &input.approval_evidence_ref,
        AUDIT_REF_PREFIX,
        AccountingDomainError::InvalidEvidenceRef,
    )?;
    if input.sources.is_empty() {
        return Err(AccountingDomainError::RulepackSourcesRequired);
    }
    if input.ledger_persistence_attached
        || input.workflow_engine_attached
        || input.statutory_filing_rail_attached
        || input.payment_execution_attached
        || input.cloud_deployment_attached
    {
        return Err(AccountingDomainError::UnsupportedRulepackCapabilityClaim);
    }

    let source_count = input.sources.len();
    let mut sources = Vec::with_capacity(source_count);
    for source in input.sources {
        sources.push(build_accounting_rulepack_source(source)?);
    }

    Ok(AccountingStatutoryRulepackManifest {
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        jurisdiction: internal(input.jurisdiction),
        accounting_period: financial(input.accounting_period),
        source_version: internal(input.source_version),
        effective_date: internal(RulepackEffectiveDate {
            value: input.effective_date,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        sources: internal(sources),
        source_count: public(source_count),
        ledger_persistence_attached: public(false),
        workflow_engine_attached: public(false),
        statutory_filing_rail_attached: public(false),
        payment_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(ACCOUNTING_STATUTORY_RULEPACK_SCHEMA_VERSION),
    })
}

fn build_accounting_rulepack_source(
    source: AccountingRulepackSourceInput,
) -> Result<AccountingRulepackSource, AccountingDomainError> {
    validate_ref(
        &source.source_ref,
        ACCOUNTING_RULEPACK_SOURCE_REF_PREFIX,
        AccountingDomainError::InvalidRulepackSourceRef,
    )?;
    validate_official_source_url(&source.official_url)?;
    validate_source_version(&source.version_label)?;
    validate_iso_date(&source.effective_date)?;
    if source.retrieved_at_epoch_seconds == 0 {
        return Err(AccountingDomainError::InvalidRulepackSourceRetrievedAt);
    }
    validate_ref(
        &source.evidence_ref,
        AUDIT_REF_PREFIX,
        AccountingDomainError::InvalidEvidenceRef,
    )?;
    validate_digest(&source.digest)?;

    Ok(AccountingRulepackSource {
        source_kind: internal(source.source_kind),
        source_ref: internal(source.source_ref),
        official_url: public(source.official_url),
        version_label: internal(source.version_label),
        effective_date: internal(RulepackEffectiveDate {
            value: source.effective_date,
        }),
        retrieved_at_epoch_seconds: internal(source.retrieved_at_epoch_seconds),
        evidence_ref: internal(EvidenceRef {
            value: source.evidence_ref,
        }),
        digest: internal(EvidenceDigest {
            value: source.digest,
        }),
    })
}

fn validate_journal_header(
    journal_id: &str,
    tenant_id: &str,
    legal_entity_id: &str,
    period: &str,
) -> Result<(), AccountingDomainError> {
    validate_identifier(
        journal_id,
        JOURNAL_ID_PREFIX,
        AccountingDomainError::InvalidJournalId,
    )?;
    validate_identifier(
        tenant_id,
        TENANT_ID_PREFIX,
        AccountingDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        AccountingDomainError::InvalidLegalEntityId,
    )?;
    validate_period(period)
}

fn build_lines(
    lines: Vec<JournalLineInput>,
) -> Result<(Vec<JournalLine>, i64, i64), AccountingDomainError> {
    if lines.is_empty() {
        return Err(AccountingDomainError::JournalLinesRequired);
    }
    let mut debit = 0_i64;
    let mut credit = 0_i64;
    let mut built = Vec::with_capacity(lines.len());
    for line in lines {
        validate_account_code(&line.account_code)?;
        let has_debit = line.debit_minor > 0;
        let has_credit = line.credit_minor > 0;
        if line.debit_minor < 0 || line.credit_minor < 0 || has_debit == has_credit {
            return Err(AccountingDomainError::InvalidMoney);
        }
        debit += line.debit_minor;
        credit += line.credit_minor;
        built.push(JournalLine {
            account_code: internal(line.account_code),
            debit_minor: internal(line.debit_minor),
            credit_minor: internal(line.credit_minor),
        });
    }
    if debit != credit {
        return Err(AccountingDomainError::UnbalancedJournal);
    }
    Ok((built, debit, credit))
}

fn validate_identifier(
    value: &str,
    prefix: &str,
    error: AccountingDomainError,
) -> Result<(), AccountingDomainError> {
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

fn validate_ref(
    value: &str,
    prefix: &str,
    error: AccountingDomainError,
) -> Result<(), AccountingDomainError> {
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
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("token")
        || lowered.contains("secret")
        || lowered.contains("bearer")
        || lowered.contains("password")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), AccountingDomainError> {
    let Some(hex) = value.strip_prefix(HASH_PREFIX) else {
        return Err(AccountingDomainError::InvalidEvidenceDigest);
    };
    if hex.len() != 64 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(AccountingDomainError::InvalidEvidenceDigest);
    }
    Ok(())
}

fn validate_source_version(value: &str) -> Result<(), AccountingDomainError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.len() > 96
        || has_unsafe_text(trimmed)
        || trimmed.contains("..")
        || !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
    {
        return Err(AccountingDomainError::InvalidRulepackSourceVersion);
    }
    Ok(())
}

fn validate_official_source_url(value: &str) -> Result<(), AccountingDomainError> {
    if has_unsafe_text(value) || !value.starts_with("https://") {
        return Err(AccountingDomainError::InvalidRulepackSourceUrl);
    }
    let allowed = [
        "https://www.nts.go.kr/",
        "https://nts.go.kr/",
        "https://www.hometax.go.kr/",
        "https://hometax.go.kr/",
        "https://law.go.kr/",
        "https://www.law.go.kr/",
        "https://www.irs.gov/",
    ];
    if !allowed.iter().any(|prefix| value.starts_with(prefix)) {
        return Err(AccountingDomainError::InvalidRulepackSourceUrl);
    }
    if value.contains("..") || value.contains('\\') {
        return Err(AccountingDomainError::InvalidRulepackSourceUrl);
    }
    Ok(())
}

fn validate_iso_date(value: &str) -> Result<(), AccountingDomainError> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| matches!(idx, 4 | 7) || byte.is_ascii_digit())
    {
        return Err(AccountingDomainError::InvalidRulepackEffectiveDate);
    }
    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| AccountingDomainError::InvalidRulepackEffectiveDate)?;
    let day = value[8..10]
        .parse::<u8>()
        .map_err(|_| AccountingDomainError::InvalidRulepackEffectiveDate)?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(AccountingDomainError::InvalidRulepackEffectiveDate);
    }
    Ok(())
}

fn validate_period(value: &str) -> Result<(), AccountingDomainError> {
    let bytes = value.as_bytes();
    if bytes.len() != 7
        || bytes[4] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| idx == 4 || byte.is_ascii_digit())
    {
        return Err(AccountingDomainError::InvalidPeriod);
    }
    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| AccountingDomainError::InvalidPeriod)?;
    if !(1..=12).contains(&month) {
        return Err(AccountingDomainError::InvalidPeriod);
    }
    Ok(())
}

fn validate_account_code(value: &str) -> Result<(), AccountingDomainError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(AccountingDomainError::InvalidAccountCode);
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

fn financial<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Financial)
}
