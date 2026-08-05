//! Payroll run domain foundation.
//!
//! This crate owns pure payroll-run invariants for trial close, group rollup,
//! statutory-export evidence envelopes, payroll-to-accounting journal drafts,
//! and rollback-first promotion decisions. It does not perform tax-rate
//! calculation, disbursement, storage, regulator filing I/O, or workflow
//! execution.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const RUN_ID_PREFIX: &str = "prun_";
const GROUP_ROLLUP_ID_PREFIX: &str = "pgrp_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const PAYEE_ID_PREFIX: &str = "payee_";
const WAGE_ENTRY_ID_PREFIX: &str = "wage_";
const PERSON_REF_PREFIX: &str = "person/";
const VENDOR_REF_PREFIX: &str = "vendor/";
const TAX_PROFILE_REF_PREFIX: &str = "tax/";
const RULEPACK_REF_PREFIX: &str = "rulepack/";
const AUDIT_REF_PREFIX: &str = "audit/";
const HASH_PREFIX: &str = "sha256:";
const JOURNAL_ID_PREFIX: &str = "jrn_";
const EMPLOYEE_ID_PREFIX: &str = "emp_";
const LEAVE_REQUEST_ID_PREFIX: &str = "leave_";
const HR_LEAVE_IMPACT_SOURCE_TOPIC: &str = "integration.hr.payroll.leave-impact";
const HR_LEAVE_IMPACT_SCHEMA_VERSION: u32 = 1;
const RULEPACK_SOURCE_REF_PREFIX: &str = "rulepack-source/";
const RULEPACK_SOURCE_PACK_REF_PREFIX: &str = "rulepack-source-pack/";
const STATUTORY_RULEPACK_SCHEMA_VERSION: u32 = 1;
const STATUTORY_SOURCE_PACK_SCHEMA_VERSION: u32 = 1;
const VARIANCE_VERDICT_SCHEMA_VERSION: u32 = 1;
const SYNTHETIC_NON_AUTHORITATIVE_FIXTURE_LABEL: &str = "synthetic/non-authoritative fixture";
/// Schema version for `RetroAdjustmentVerdict`.
pub const RETRO_ADJUSTMENT_SCHEMA_VERSION: u32 = 1;
/// Sentinel BPS value used for dropped-payee lines (no current amount).
const DROPPED_PAYEE_SENTINEL_BPS: i64 = -10_000;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PayrollRunId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GroupPayrollRollupId {
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
pub struct PayeeId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EmployeeId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LeaveRequestId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WageLedgerEntryId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartyRef {
    pub value: String, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TaxProfileRef {
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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceDigest {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct JournalId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayeeClass {
    Employee,
    Contractor,
    StatutoryDirector,
    Vendor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WageLineKind {
    GrossEarnings,
    TaxWithholding,
    SocialInsurance,
    EmployerContribution,
    NetPay,
    Reversal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayrollRunState {
    Draft,
    TrialClosed,
    EntityClosed,
    GroupRolledUp,
    ProductionClosed,
    RollbackQuarantined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StatutoryExportKind {
    KoreaHomeTaxWithholding,
    KoreaFourInsurance,
    KoreaYearEndSettlement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StatutoryDeductionKind {
    IncomeTax,
    SocialInsurance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CalculationBoundary {
    PureDomainNoFilingTransport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayrollRulepackJurisdiction {
    Korea,
    UnitedStatesFederal,
    EuropeanUnion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayrollRulepackSourceKind {
    EmployerTaxGuide,
    WithholdingMethod,
    WageRecordkeeping,
    LaborStandards,
    SocialInsurance,
    YearEndSettlement,
    StatutoryFilingSchema,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayrollStatutorySourceRetrievalStatus {
    Retrieved,
    Approved,
    Superseded,
    Expired,
    Blocked,
    Missing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayrollStatutorySourceApplicability {
    Calculation,
    YearEndSettlement,
    CalculationAndYearEndSettlement,
    RegionalPackInventoryOnly,
    StatutoryFilingSchema,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayrollStatutorySourceCadence {
    Annual,
    Quarterly,
    Monthly,
    EventDriven,
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum YearEndSettlementSourceKind {
    WageLedgerDigest,
    WithholdingEvidence,
    EmployeeDeclaration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RepairRoute {
    HotfixPullRequest,
    OpenTofuOpsConvergence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseRollbackRouteMetadata {
    pub route_label: String,                     // data_class: INTERNAL_ONLY
    pub hotfix_pr_required: bool,                // data_class: INTERNAL_ONLY
    pub opentofu_ops_convergence_required: bool, // data_class: INTERNAL_ONLY
    pub production_deploy_attached: bool,        // data_class: PUBLIC
    pub workflow_execution_attached: bool,       // data_class: PUBLIC
    pub opentofu_execution_attached: bool,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseHealthObservabilityAttribute {
    pub key: String,   // data_class: PUBLIC
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum HrLeaveImpactKind {
    PaidLeave,
    UnpaidLeaveDeduction,
    AttendanceCorrection,
    NoPayrollImpact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoneyAmount {
    pub amount_minor: i64, // data_class: FINANCIAL
    pub currency: String,  // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WageLedgerEntryInput {
    pub entry_id: String,        // data_class: INTERNAL_ONLY
    pub payee_id: String,        // data_class: INTERNAL_ONLY
    pub line_kind: WageLineKind, // data_class: INTERNAL_ONLY
    pub amount: MoneyAmount,     // data_class: FINANCIAL
    pub source_ref: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayeeInput {
    pub payee_id: String,                       // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                // data_class: INTERNAL_ONLY
    pub payee_class: PayeeClass,                // data_class: INTERNAL_ONLY
    pub person_or_vendor_ref: String,           // data_class: PII_IDENTIFYING
    pub tax_profile_ref: String,                // data_class: INTERNAL_ONLY
    pub wage_ledger: Vec<WageLedgerEntryInput>, // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollTrialCloseInput {
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,         // data_class: INTERNAL_ONLY
    pub period: String,                  // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,            // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String, // data_class: INTERNAL_ONLY
    pub evidence_digest: String,         // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String,   // data_class: INTERNAL_ONLY
    pub payees: Vec<PayeeInput>,         // data_class: PII_IDENTIFYING + FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrLeaveImpactIntakeInput {
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,             // data_class: INTERNAL_ONLY
    pub payroll_period: String,              // data_class: FINANCIAL
    pub payee_id: String,                    // data_class: INTERNAL_ONLY
    pub employee_id: String,                 // data_class: INTERNAL_ONLY
    pub leave_request_id: String,            // data_class: INTERNAL_ONLY
    pub impact_kind: HrLeaveImpactKind,      // data_class: FINANCIAL
    pub source_topic: String,                // data_class: INTERNAL_ONLY
    pub source_hr_idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub decision_evidence_ref: String,       // data_class: INTERNAL_ONLY
    pub routing_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub payroll_impact_evidence_ref: String, // data_class: FINANCIAL
    pub payroll_intake_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String,     // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrLeaveImpactIntake {
    pub run_id: Classified<PayrollRunId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,  // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub payroll_period: Classified<String>, // data_class: FINANCIAL
    pub payee_id: Classified<PayeeId>,    // data_class: INTERNAL_ONLY
    pub employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub leave_request_id: Classified<LeaveRequestId>, // data_class: INTERNAL_ONLY
    pub impact_kind: Classified<HrLeaveImpactKind>, // data_class: FINANCIAL
    pub source_topic: Classified<String>, // data_class: INTERNAL_ONLY
    pub source_hr_idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub decision_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub routing_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub payroll_impact_evidence_ref: Classified<EvidenceRef>, // data_class: FINANCIAL
    pub payroll_intake_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub rulepack_ref: Classified<RulepackRef>, // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WageLedgerEntry {
    pub entry_id: Classified<WageLedgerEntryId>, // data_class: INTERNAL_ONLY
    pub payee_id: Classified<PayeeId>,           // data_class: INTERNAL_ONLY
    pub line_kind: Classified<WageLineKind>,     // data_class: INTERNAL_ONLY
    pub amount: Classified<MoneyAmount>,         // data_class: FINANCIAL
    pub source_ref: Classified<EvidenceRef>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payee {
    pub payee_id: Classified<PayeeId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub payee_class: Classified<PayeeClass>, // data_class: INTERNAL_ONLY
    pub person_or_vendor_ref: Classified<PartyRef>, // data_class: PII_IDENTIFYING
    pub tax_profile_ref: Classified<TaxProfileRef>, // data_class: INTERNAL_ONLY
    pub wage_ledger: Classified<Vec<WageLedgerEntry>>, // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollRun {
    pub run_id: Classified<PayrollRunId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,  // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub period: Classified<String>,       // data_class: INTERNAL_ONLY
    pub state: Classified<PayrollRunState>, // data_class: INTERNAL_ONLY
    pub rulepack_ref: Classified<RulepackRef>, // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub evidence_digest: Classified<EvidenceDigest>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub payees: Classified<Vec<Payee>>,   // data_class: PII_IDENTIFYING + FINANCIAL
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityCloseSnapshot {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub run_id: String,                    // data_class: INTERNAL_ONLY
    pub state: PayrollRunState,            // data_class: INTERNAL_ONLY
    pub evidence_digest: String,           // data_class: INTERNAL_ONLY
    pub detachment_history_redacted: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupPayrollRollup {
    pub rollup_id: Classified<GroupPayrollRollupId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,             // data_class: INTERNAL_ONLY
    pub state: Classified<PayrollRunState>,          // data_class: INTERNAL_ONLY
    pub entity_closes: Classified<Vec<EntityCloseSnapshot>>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatutoryExportInput {
    pub run_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,          // data_class: INTERNAL_ONLY
    pub export_kind: StatutoryExportKind, // data_class: INTERNAL_ONLY
    pub export_hash: String,              // data_class: INTERNAL_ONLY
    pub receipt_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub rejection_reason: Option<String>, // data_class: INTERNAL_ONLY
    pub rollback_plan_ref: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatutoryExportEvidence {
    pub run_id: Classified<PayrollRunId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,  // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub export_kind: Classified<StatutoryExportKind>, // data_class: INTERNAL_ONLY
    pub export_hash: Classified<EvidenceDigest>, // data_class: INTERNAL_ONLY
    pub receipt_ref: Classified<Option<EvidenceRef>>, // data_class: INTERNAL_ONLY
    pub rejection_reason: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub rollback_plan_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollRulepackSourceInput {
    pub source_kind: PayrollRulepackSourceKind, // data_class: INTERNAL_ONLY
    pub source_ref: String,                     // data_class: INTERNAL_ONLY
    pub official_url: String,                   // data_class: PUBLIC
    pub version_label: String,                  // data_class: INTERNAL_ONLY
    pub effective_date: String,                 // data_class: INTERNAL_ONLY
    pub retrieved_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                   // data_class: INTERNAL_ONLY
    pub digest: String,                         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollStatutoryRulepackManifestInput {
    pub rulepack_ref: String,                      // data_class: INTERNAL_ONLY
    pub jurisdiction: PayrollRulepackJurisdiction, // data_class: INTERNAL_ONLY
    pub payroll_period: String,                    // data_class: FINANCIAL
    pub source_version: String,                    // data_class: INTERNAL_ONLY
    pub effective_date: String,                    // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String,             // data_class: INTERNAL_ONLY
    pub sources: Vec<PayrollRulepackSourceInput>,  // data_class: INTERNAL_ONLY
    pub calculation_engine_attached: bool,         // data_class: PUBLIC
    pub filing_rail_attached: bool,                // data_class: PUBLIC
    pub disbursement_rail_attached: bool,          // data_class: PUBLIC
    pub cloud_deployment_attached: bool,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollRulepackSource {
    pub source_kind: Classified<PayrollRulepackSourceKind>, // data_class: INTERNAL_ONLY
    pub source_ref: Classified<String>,                     // data_class: INTERNAL_ONLY
    pub official_url: Classified<String>,                   // data_class: PUBLIC
    pub version_label: Classified<String>,                  // data_class: INTERNAL_ONLY
    pub effective_date: Classified<RulepackEffectiveDate>,  // data_class: INTERNAL_ONLY
    pub retrieved_at_epoch_seconds: Classified<u64>,        // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<EvidenceRef>,              // data_class: INTERNAL_ONLY
    pub digest: Classified<EvidenceDigest>,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollStatutoryRulepackManifest {
    pub rulepack_ref: Classified<RulepackRef>, // data_class: INTERNAL_ONLY
    pub jurisdiction: Classified<PayrollRulepackJurisdiction>, // data_class: INTERNAL_ONLY
    pub payroll_period: Classified<String>,    // data_class: FINANCIAL
    pub source_version: Classified<String>,    // data_class: INTERNAL_ONLY
    pub effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub sources: Classified<Vec<PayrollRulepackSource>>, // data_class: INTERNAL_ONLY
    pub source_count: Classified<usize>,       // data_class: PUBLIC
    pub calculation_engine_attached: Classified<bool>, // data_class: PUBLIC
    pub filing_rail_attached: Classified<bool>, // data_class: PUBLIC
    pub disbursement_rail_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollStatutorySourceRowInput {
    pub region: PayrollRulepackJurisdiction, // data_class: INTERNAL_ONLY
    pub source_kind: PayrollRulepackSourceKind, // data_class: INTERNAL_ONLY
    pub publisher: String,                   // data_class: PUBLIC
    pub official_url_or_path: Option<String>, // data_class: PUBLIC
    pub version_label: String,               // data_class: INTERNAL_ONLY
    pub effective_date: Option<String>,      // data_class: INTERNAL_ONLY
    pub retrieval_status: PayrollStatutorySourceRetrievalStatus, // data_class: INTERNAL_ONLY
    pub source_digest: Option<String>,       // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub applicability: PayrollStatutorySourceApplicability, // data_class: INTERNAL_ONLY
    pub cadence: PayrollStatutorySourceCadence, // data_class: INTERNAL_ONLY
    pub owner: String,                       // data_class: INTERNAL_ONLY
    pub supersedes_source_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub expires_on: Option<String>,          // data_class: INTERNAL_ONLY
    pub unresolved_blocker_reason: Option<String>, // data_class: INTERNAL_ONLY
    pub fixture_note: &'static str,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollStatutorySourcePackInput {
    pub source_pack_ref: String, // data_class: INTERNAL_ONLY
    pub payroll_year: u32,       // data_class: FINANCIAL
    pub rows: Vec<PayrollStatutorySourceRowInput>, // data_class: INTERNAL_ONLY
    pub fixture_note: &'static str, // data_class: PUBLIC
    pub official_tax_rate_correctness_requested: bool, // data_class: PUBLIC
    pub calculation_engine_requested: bool, // data_class: PUBLIC
    pub filing_rail_requested: bool, // data_class: PUBLIC
    pub disbursement_rail_requested: bool, // data_class: PUBLIC
    pub cloud_deployment_requested: bool, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollStatutorySourceRow {
    pub region: Classified<PayrollRulepackJurisdiction>, // data_class: INTERNAL_ONLY
    pub source_kind: Classified<PayrollRulepackSourceKind>, // data_class: INTERNAL_ONLY
    pub publisher: Classified<String>,                   // data_class: PUBLIC
    pub official_url_or_path: Classified<Option<String>>, // data_class: PUBLIC
    pub version_label: Classified<String>,               // data_class: INTERNAL_ONLY
    pub effective_date: Classified<Option<RulepackEffectiveDate>>, // data_class: INTERNAL_ONLY
    pub retrieval_status: Classified<PayrollStatutorySourceRetrievalStatus>, // data_class: INTERNAL_ONLY
    pub source_digest: Classified<Option<EvidenceDigest>>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<Option<EvidenceRef>>, // data_class: INTERNAL_ONLY
    pub applicability: Classified<PayrollStatutorySourceApplicability>, // data_class: INTERNAL_ONLY
    pub cadence: Classified<PayrollStatutorySourceCadence>, // data_class: INTERNAL_ONLY
    pub owner: Classified<String>,                         // data_class: INTERNAL_ONLY
    pub supersedes_source_ref: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub expires_on: Classified<Option<RulepackEffectiveDate>>, // data_class: INTERNAL_ONLY
    pub unresolved_blocker_reason: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub fixture_note: Classified<String>,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollStatutorySourcePack {
    pub source_pack_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub payroll_year: Classified<u32>,       // data_class: FINANCIAL
    pub rows: Classified<Vec<PayrollStatutorySourceRow>>, // data_class: INTERNAL_ONLY
    pub region_count: Classified<usize>,     // data_class: PUBLIC
    pub has_unresolved_blockers: Classified<bool>, // data_class: PUBLIC
    pub fixture_note: Classified<String>,    // data_class: PUBLIC
    pub official_tax_rate_correctness_attached: Classified<bool>, // data_class: PUBLIC
    pub calculation_engine_attached: Classified<bool>, // data_class: PUBLIC
    pub filing_rail_attached: Classified<bool>, // data_class: PUBLIC
    pub disbursement_rail_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatutoryRateLineInput {
    pub kind: StatutoryDeductionKind,     // data_class: INTERNAL_ONLY
    pub synthetic_rate_basis_points: u32, // data_class: FINANCIAL
    pub source_evidence_ref: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatutoryCalculationInput {
    pub run_id: String,                             // data_class: INTERNAL_ONLY
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                    // data_class: INTERNAL_ONLY
    pub payee_id: String,                           // data_class: INTERNAL_ONLY
    pub payroll_period: String,                     // data_class: FINANCIAL
    pub jurisdiction: PayrollRulepackJurisdiction,  // data_class: INTERNAL_ONLY
    pub required_regional_pack: String,             // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                       // data_class: INTERNAL_ONLY
    pub rulepack_manifest_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub rulepack_source_version: Option<String>,    // data_class: INTERNAL_ONLY
    pub official_source_evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub unofficial_source_fixture: bool,            // data_class: INTERNAL_ONLY
    pub gross_pay_minor: i64,                       // data_class: FINANCIAL
    pub currency: String,                           // data_class: FINANCIAL
    pub rate_lines: Vec<StatutoryRateLineInput>,    // data_class: FINANCIAL
    pub fixture_note: &'static str,                 // data_class: PUBLIC
    pub filing_rail_requested: bool,                // data_class: PUBLIC
    pub disbursement_rail_requested: bool,          // data_class: PUBLIC
    pub production_close_requested: bool,           // data_class: PUBLIC
    pub cloud_deployment_requested: bool,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatutoryDeductionLine {
    pub kind: Classified<StatutoryDeductionKind>, // data_class: INTERNAL_ONLY
    pub amount: Classified<MoneyAmount>,          // data_class: FINANCIAL
    pub synthetic_rate_basis_points: Classified<u32>, // data_class: FINANCIAL
    pub source_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatutoryCalculationDraft {
    pub run_id: Classified<String>,          // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub payee_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub payroll_period: Classified<String>,  // data_class: FINANCIAL
    pub jurisdiction: Classified<PayrollRulepackJurisdiction>, // data_class: INTERNAL_ONLY
    pub required_regional_pack: Classified<String>, // data_class: INTERNAL_ONLY
    pub rulepack_ref: Classified<String>,    // data_class: INTERNAL_ONLY
    pub rulepack_manifest_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub rulepack_source_version: Classified<String>, // data_class: INTERNAL_ONLY
    pub official_source_evidence_refs: Classified<Vec<EvidenceRef>>, // data_class: INTERNAL_ONLY
    pub fixture_note: Classified<String>,    // data_class: PUBLIC
    pub gross_pay: Classified<MoneyAmount>,  // data_class: FINANCIAL
    pub deductions: Classified<Vec<StatutoryDeductionLine>>, // data_class: FINANCIAL
    pub net_pay: Classified<MoneyAmount>,    // data_class: FINANCIAL
    pub boundary: Classified<CalculationBoundary>, // data_class: PUBLIC
    pub direct_agency_submission_attached: Classified<bool>, // data_class: PUBLIC
    pub filing_rail_attached: Classified<bool>, // data_class: PUBLIC
    pub disbursement_rail_attached: Classified<bool>, // data_class: PUBLIC
    pub production_close_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YearEndEvidenceRefInput {
    pub source_kind: YearEndSettlementSourceKind, // data_class: INTERNAL_ONLY
    pub ref_value: String,                        // data_class: INTERNAL_ONLY
    pub source_version: String,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YearEndRegionalDependency {
    pub pack_code: String,      // data_class: INTERNAL_ONLY
    pub source_version: String, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YearEndEmployeeInput {
    pub payee_id: String,                 // data_class: INTERNAL_ONLY
    pub employee_ref: String,             // data_class: PII_IDENTIFYING
    pub gross_pay_minor: i64,             // data_class: FINANCIAL
    pub withholding_minor: i64,           // data_class: FINANCIAL
    pub currency: String,                 // data_class: FINANCIAL
    pub wage_ledger_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub declaration_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YearEndSettlementInput {
    pub run_id: String,                              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                     // data_class: INTERNAL_ONLY
    pub payroll_year: u32,                           // data_class: FINANCIAL
    pub jurisdiction: PayrollRulepackJurisdiction,   // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                        // data_class: INTERNAL_ONLY
    pub rulepack_manifest_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub source_version: Option<String>,              // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<YearEndEvidenceRefInput>, // data_class: INTERNAL_ONLY
    pub regional_dependencies: Vec<YearEndRegionalDependency>, // data_class: INTERNAL_ONLY
    pub employee_inputs: Vec<YearEndEmployeeInput>,  // data_class: PII_IDENTIFYING + FINANCIAL
    pub fixture_note: &'static str,                  // data_class: PUBLIC
    pub unofficial_source_fixture: bool,             // data_class: INTERNAL_ONLY
    pub direct_agency_submission_requested: bool,    // data_class: PUBLIC
    pub filing_rail_requested: bool,                 // data_class: PUBLIC
    pub disbursement_rail_requested: bool,           // data_class: PUBLIC
    pub production_close_requested: bool,            // data_class: PUBLIC
    pub cloud_deployment_requested: bool,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YearEndEvidenceRef {
    pub source_kind: Classified<YearEndSettlementSourceKind>, // data_class: INTERNAL_ONLY
    pub ref_value: Classified<String>,                        // data_class: INTERNAL_ONLY
    pub source_version: Classified<String>,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedYearEndRegionalDependency {
    pub pack_code: Classified<String>,      // data_class: INTERNAL_ONLY
    pub source_version: Classified<String>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedYearEndEmployeeInput {
    pub payee_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub employee_ref: Classified<String>,   // data_class: PII_IDENTIFYING
    pub gross_pay: Classified<MoneyAmount>, // data_class: FINANCIAL
    pub withholding: Classified<MoneyAmount>, // data_class: FINANCIAL
    pub wage_ledger_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub declaration_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedYearEndSettlementInput {
    pub run_id: Classified<String>,          // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub payroll_year: Classified<u32>,       // data_class: FINANCIAL
    pub jurisdiction: Classified<PayrollRulepackJurisdiction>, // data_class: INTERNAL_ONLY
    pub rulepack_ref: Classified<String>,    // data_class: INTERNAL_ONLY
    pub rulepack_manifest_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub source_version: Classified<String>,  // data_class: INTERNAL_ONLY
    pub fixture_note: Classified<String>,    // data_class: PUBLIC
    pub evidence_refs: Classified<Vec<YearEndEvidenceRef>>, // data_class: INTERNAL_ONLY
    pub regional_dependencies: Classified<Vec<PreparedYearEndRegionalDependency>>, // data_class: INTERNAL_ONLY
    pub employee_inputs: Classified<Vec<PreparedYearEndEmployeeInput>>, // data_class: PII_IDENTIFYING + FINANCIAL
    pub direct_agency_submission_attached: Classified<bool>,            // data_class: PUBLIC
    pub filing_rail_attached: Classified<bool>,                         // data_class: PUBLIC
    pub disbursement_rail_attached: Classified<bool>,                   // data_class: PUBLIC
    pub production_close_attached: Classified<bool>,                    // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollJournalLineInput {
    pub account_code: String, // data_class: INTERNAL_ONLY
    pub debit_minor: i64,     // data_class: INTERNAL_ONLY
    pub credit_minor: i64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollJournalInput {
    pub journal_id: String,                  // data_class: INTERNAL_ONLY
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,             // data_class: INTERNAL_ONLY
    pub period: String,                      // data_class: INTERNAL_ONLY
    pub source_payroll_digest: String,       // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String,       // data_class: INTERNAL_ONLY
    pub lines: Vec<PayrollJournalLineInput>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollJournalDraft {
    pub journal_id: Classified<JournalId>, // data_class: INTERNAL_ONLY
    pub run_id: Classified<PayrollRunId>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,   // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub source_payroll_digest: Classified<EvidenceDigest>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub total_debit_minor: Classified<i64>, // data_class: INTERNAL_ONLY
    pub total_credit_minor: Classified<i64>, // data_class: INTERNAL_ONLY
    pub reversal_required_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseHealthInput {
    pub run_id: String,                          // data_class: INTERNAL_ONLY
    pub canary_passed: bool,                     // data_class: INTERNAL_ONLY
    pub evidence_gate_passed: bool,              // data_class: INTERNAL_ONLY
    pub rollback_evidence_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub quarantine_evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub repair_route: Option<RepairRoute>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClosePromotionDecision {
    Promote,
    RollbackFirst {
        rollback_evidence_ref: EvidenceRef,
        quarantine_evidence_ref: EvidenceRef,
        repair_route: RepairRoute,
        promotion_stopped: bool,
        route_metadata: CloseRollbackRouteMetadata,
        observability_attributes: Vec<CloseHealthObservabilityAttribute>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PayrollDomainError {
    InvalidRunId,
    InvalidGroupRollupId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidPayeeId,
    InvalidWageEntryId,
    InvalidPartyRef,
    InvalidTaxProfileRef,
    InvalidRulepackRef,
    InvalidRulepackEffectiveDate,
    InvalidRulepackSourceRef,
    InvalidRulepackSourceVersion,
    InvalidRulepackSourceUrl,
    InvalidPeriod,
    InvalidEvidenceRef,
    InvalidEvidenceDigest,
    InvalidJournalId,
    InvalidAccountCode,
    InvalidMoney,
    InvalidEmployeeId,
    InvalidLeaveRequestId,
    InvalidHrLeaveImpactTopic,
    InvalidIdempotencyKey,
    InvalidReceivedAt,
    TrialCloseRequiresPayees,
    PayeeLegalEntityMismatch,
    PayeeMissingWageLedger,
    WageEntryPayeeMismatch,
    EntityCloseIncomplete,
    DetachmentHistoryNotRedacted,
    MissingReceiptOrRejection,
    RulepackSourcesRequired,
    UnsupportedRulepackCapabilityClaim,
    StatutoryRulepackManifestRequired,
    OfficialRulepackSourceEvidenceRequired,
    OfficialSourceDigestRequired,
    OfficialSourceApprovalEvidenceRequired,
    OfficialSourceOwnerRequired,
    OfficialSourceApplicabilityRequired,
    OfficialSourceCadenceRequired,
    OfficialSourceBlockerReasonRequired,
    StatutoryRegionalPackRequired,
    YearEndSettlementSourceEvidenceRequired,
    OfficialYearEndSourceEvidenceRequired,
    YearEndRegionalPackRequired,
    JournalLinesRequired,
    UnbalancedJournal,
    RollbackEvidenceRequired,
    /// Returned when `variance_tolerance_bps` is zero (tolerance must be
    /// explicitly set by the caller — a default of zero would silently flag
    /// every payee as anomalous).
    VarianceToleranceRequired,
    /// Returned when `evaluate_payroll_variance` encounters a current-period
    /// payee that has no matching prior-period baseline entry.  The whole
    /// verdict is aborted so the caller can decide whether to onboard the
    /// payee explicitly or relax the baseline requirement.
    MissingBaselineForPayee,
    /// Returned when `build_group_gl_posting` receives an empty `entries` vec.
    /// At least one per-entity journal entry is required to form a group batch.
    GroupPostingEntitiesRequired,
    /// Returned when `build_group_gl_posting` detects the same `legal_entity_id`
    /// appearing in more than one entry of the same group batch.
    DuplicateLegalEntityInGroup,
    /// Returned when `evaluate_retro_adjustment` detects that the same payee has
    /// different currency codes in the original and corrected totals.
    CurrencyMismatch,
    /// Returned when the `run_ref` field of `RetroAdjustmentInput` fails the
    /// `audit/` prefix and path-safety check.
    InvalidRunRef,
    /// Returned when `evaluate_retro_adjustment` receives an empty `evidence_refs`
    /// vec. At least one evidence reference is required.
    RetroEvidenceRequired,
}

// ── Variance gate types ────────────────────────────────────────────────────

/// Flat per-payee net total used as input to the variance gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayeeVarianceTotal {
    pub payee_id: String,        // data_class: INTERNAL_ONLY
    pub net_amount: MoneyAmount, // data_class: FINANCIAL
}

/// Input to `evaluate_payroll_variance`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollVarianceInput {
    pub run_id: String,                                 // data_class: INTERNAL_ONLY
    pub current_period_totals: Vec<PayeeVarianceTotal>, // data_class: FINANCIAL
    pub prior_period_totals: Vec<PayeeVarianceTotal>,   // data_class: FINANCIAL
    /// Must be > 0. Basis points threshold; swings exceeding this are anomalies.
    pub variance_tolerance_bps: u32, // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                           // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String,                // data_class: INTERNAL_ONLY (ISO date)
    /// Each entry must be a valid `audit/` ref.
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
    /// Epoch seconds; must be > 0.
    pub evaluated_at: u64, // data_class: INTERNAL_ONLY
}

/// Classified per-payee variance line in the verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollVarianceLine {
    pub payee_id: Classified<PayeeId>, // data_class: INTERNAL_ONLY
    pub current_amount: Classified<MoneyAmount>, // data_class: FINANCIAL
    pub prior_amount: Classified<MoneyAmount>, // data_class: FINANCIAL
    /// Signed BPS. Positive = increase. `DROPPED_PAYEE_SENTINEL_BPS` for dropped payees.
    pub variance_bps: Classified<i64>, // data_class: INTERNAL_ONLY
    pub anomaly: Classified<bool>,     // data_class: INTERNAL_ONLY
}

/// Anomaly flag variants; each carries the affected payee identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnomalyFlag {
    /// The payee's variance exceeds `variance_tolerance_bps`.
    OverToleranceSwing { payee_id: PayeeId },
    /// The payee's net amount flipped sign (positive→negative or vice-versa).
    SignFlip { payee_id: PayeeId },
    /// A prior-period payee is entirely absent from the current period.
    DroppedPayee { payee_id: PayeeId },
}

/// Classified verdict returned by `evaluate_payroll_variance`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayrollVarianceVerdict {
    pub run_id: Classified<PayrollRunId>, // data_class: INTERNAL_ONLY
    pub lines: Classified<Vec<PayrollVarianceLine>>, // data_class: FINANCIAL
    pub run_net_variance_bps: Classified<i64>, // data_class: INTERNAL_ONLY
    pub anomaly_flags: Classified<Vec<AnomalyFlag>>, // data_class: INTERNAL_ONLY
    pub gate_passed: Classified<bool>,    // data_class: INTERNAL_ONLY
    pub rulepack_ref: Classified<RulepackRef>, // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub evidence_digest: Classified<EvidenceDigest>, // data_class: INTERNAL_ONLY
    pub evaluated_at: Classified<u64>,    // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: PUBLIC
}

// ── Retro adjustment types ─────────────────────────────────────────────────

/// Classification of how a payee changed between original and corrected runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RetroPayeeClass {
    /// Payee present in corrected totals but absent from original.
    Added,
    /// Payee present in original totals but absent from corrected.
    Removed,
    /// Payee present in both; corrected amount differs from original.
    Changed,
    /// Payee present in both; corrected amount equals original (delta = 0).
    Unchanged,
}

/// Per-payee signed delta line produced by `evaluate_retro_adjustment`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroDeltaLine {
    pub payee_id: Classified<PayeeId>, // data_class: INTERNAL_ONLY
    /// Original amount. For `Added` payees, `amount_minor` is 0.
    pub original_amount: Classified<MoneyAmount>, // data_class: FINANCIAL
    /// Corrected amount. For `Removed` payees, `amount_minor` is 0.
    pub corrected_amount: Classified<MoneyAmount>, // data_class: FINANCIAL
    /// Signed delta: `corrected_amount.amount_minor - original_amount.amount_minor`.
    pub delta_amount: Classified<MoneyAmount>, // data_class: FINANCIAL
    pub payee_class: Classified<RetroPayeeClass>, // data_class: INTERNAL_ONLY
}

/// Input to `evaluate_retro_adjustment`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroAdjustmentInput {
    /// Payroll run being adjusted. Must have the `prun_` prefix.
    pub run_id: String, // data_class: INTERNAL_ONLY
    /// Audit reference for this retro run. Must have the `audit/` prefix.
    pub run_ref: String, // data_class: INTERNAL_ONLY
    /// Baseline (original-period) per-payee net totals.
    pub original_period_totals: Vec<PayeeVarianceTotal>, // data_class: FINANCIAL
    /// Corrected per-payee net totals.
    pub corrected_period_totals: Vec<PayeeVarianceTotal>, // data_class: FINANCIAL
    /// Non-empty vec of `audit/` evidence refs. Used for the evidence digest.
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

/// Classified verdict returned by `evaluate_retro_adjustment`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetroAdjustmentVerdict {
    pub run_id: Classified<PayrollRunId>, // data_class: INTERNAL_ONLY
    /// One line per payee from the union of original and corrected sets.
    pub lines: Classified<Vec<RetroDeltaLine>>, // data_class: FINANCIAL
    /// Sum of all `delta_amount.amount_minor` across all lines.
    pub run_net_delta: Classified<MoneyAmount>, // data_class: FINANCIAL
    /// True iff `run_net_delta` equals sum(corrected) minus sum(original).
    pub balanced: Classified<bool>, // data_class: PUBLIC
    /// XOR-fold evidence digest (same algorithm as variance verdict).
    pub evidence_digest: Classified<EvidenceDigest>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: PUBLIC
}

pub fn evaluate_retro_adjustment(
    input: RetroAdjustmentInput,
) -> Result<RetroAdjustmentVerdict, PayrollDomainError> {
    // ── Validate scalar fields ────────────────────────────────────────────
    validate_identifier(
        &input.run_id,
        RUN_ID_PREFIX,
        PayrollDomainError::InvalidRunId,
    )?;
    validate_ref(
        &input.run_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidRunRef,
    )?;
    if input.evidence_refs.is_empty() {
        return Err(PayrollDomainError::RetroEvidenceRequired);
    }
    for ev_ref in &input.evidence_refs {
        validate_ref(
            ev_ref,
            AUDIT_REF_PREFIX,
            PayrollDomainError::InvalidEvidenceRef,
        )?;
    }

    // ── Validate per-payee totals ─────────────────────────────────────────
    for total in input
        .original_period_totals
        .iter()
        .chain(input.corrected_period_totals.iter())
    {
        validate_identifier(
            &total.payee_id,
            PAYEE_ID_PREFIX,
            PayrollDomainError::InvalidPayeeId,
        )?;
        // Allow zero amounts in retro context (e.g., zero-delta payees can
        // appear with zero original or corrected; the currency must still be 3 chars).
        if total.net_amount.currency.len() != 3 || has_unsafe_text(&total.net_amount.currency) {
            return Err(PayrollDomainError::InvalidMoney);
        }
    }

    // ── Build lookup maps ─────────────────────────────────────────────────
    let original_map: std::collections::HashMap<&str, &MoneyAmount> = input
        .original_period_totals
        .iter()
        .map(|t| (t.payee_id.as_str(), &t.net_amount))
        .collect();

    let corrected_map: std::collections::HashMap<&str, &MoneyAmount> = input
        .corrected_period_totals
        .iter()
        .map(|t| (t.payee_id.as_str(), &t.net_amount))
        .collect();

    // ── Determine currency for the run (from first non-empty total) ───────
    // All entries must use the same currency; pick it from the first available.
    let run_currency: String = input
        .original_period_totals
        .first()
        .or_else(|| input.corrected_period_totals.first())
        .map(|t| t.net_amount.currency.clone())
        .unwrap_or_else(|| "XXX".to_owned());

    // ── Build delta lines ─────────────────────────────────────────────────
    let mut lines: Vec<RetroDeltaLine> = Vec::new();
    let mut run_net_delta_minor: i64 = 0;

    // Process original-period payees first (in input order).
    for orig_total in &input.original_period_totals {
        let payee_id_val = PayeeId {
            value: orig_total.payee_id.clone(),
        };
        let orig_amount = &orig_total.net_amount;

        match corrected_map.get(orig_total.payee_id.as_str()) {
            Some(corr_amount) => {
                // Payee appears in both — check currency consistency.
                if orig_amount.currency != corr_amount.currency {
                    return Err(PayrollDomainError::CurrencyMismatch);
                }
                let delta_minor = corr_amount
                    .amount_minor
                    .saturating_sub(orig_amount.amount_minor);
                run_net_delta_minor = run_net_delta_minor.saturating_add(delta_minor);
                let payee_class = if delta_minor == 0 {
                    RetroPayeeClass::Unchanged
                } else {
                    RetroPayeeClass::Changed
                };
                lines.push(RetroDeltaLine {
                    payee_id: internal(payee_id_val),
                    original_amount: financial(MoneyAmount {
                        amount_minor: orig_amount.amount_minor,
                        currency: orig_amount.currency.clone(),
                    }),
                    corrected_amount: financial(MoneyAmount {
                        amount_minor: corr_amount.amount_minor,
                        currency: corr_amount.currency.clone(),
                    }),
                    delta_amount: financial(MoneyAmount {
                        amount_minor: delta_minor,
                        currency: orig_amount.currency.clone(),
                    }),
                    payee_class: internal(payee_class),
                });
            }
            None => {
                // Payee was removed in the corrected run.
                let delta_minor = 0_i64.saturating_sub(orig_amount.amount_minor);
                run_net_delta_minor = run_net_delta_minor.saturating_add(delta_minor);
                lines.push(RetroDeltaLine {
                    payee_id: internal(payee_id_val),
                    original_amount: financial(MoneyAmount {
                        amount_minor: orig_amount.amount_minor,
                        currency: orig_amount.currency.clone(),
                    }),
                    corrected_amount: financial(MoneyAmount {
                        amount_minor: 0,
                        currency: orig_amount.currency.clone(),
                    }),
                    delta_amount: financial(MoneyAmount {
                        amount_minor: delta_minor,
                        currency: orig_amount.currency.clone(),
                    }),
                    payee_class: internal(RetroPayeeClass::Removed),
                });
            }
        }
    }

    // Process added payees (present in corrected but not in original), in corrected input order.
    for corr_total in &input.corrected_period_totals {
        if original_map.contains_key(corr_total.payee_id.as_str()) {
            // Already handled above.
            continue;
        }
        let payee_id_val = PayeeId {
            value: corr_total.payee_id.clone(),
        };
        let corr_amount = &corr_total.net_amount;
        run_net_delta_minor = run_net_delta_minor.saturating_add(corr_amount.amount_minor);
        lines.push(RetroDeltaLine {
            payee_id: internal(payee_id_val),
            original_amount: financial(MoneyAmount {
                amount_minor: 0,
                currency: corr_amount.currency.clone(),
            }),
            corrected_amount: financial(MoneyAmount {
                amount_minor: corr_amount.amount_minor,
                currency: corr_amount.currency.clone(),
            }),
            delta_amount: financial(MoneyAmount {
                amount_minor: corr_amount.amount_minor,
                currency: corr_amount.currency.clone(),
            }),
            payee_class: internal(RetroPayeeClass::Added),
        });
    }

    // ── Balanced aggregate check ──────────────────────────────────────────
    let sum_original: i64 = input
        .original_period_totals
        .iter()
        .map(|t| t.net_amount.amount_minor)
        .fold(0_i64, i64::saturating_add);
    let sum_corrected: i64 = input
        .corrected_period_totals
        .iter()
        .map(|t| t.net_amount.amount_minor)
        .fold(0_i64, i64::saturating_add);
    let expected_net = sum_corrected.saturating_sub(sum_original);
    let balanced = run_net_delta_minor == expected_net;

    // ── Evidence digest (XOR-fold, same as variance verdict) ─────────────
    let mut buf = [0u8; 32];
    let mut pos = 0usize;
    for ev_ref in &input.evidence_refs {
        for byte in ev_ref.as_bytes() {
            buf[pos % 32] ^= byte;
            pos += 1;
        }
    }
    let hex_chars: String = buf.iter().map(|b| format!("{b:02x}")).collect();
    let evidence_digest = format!("{HASH_PREFIX}{hex_chars}");

    Ok(RetroAdjustmentVerdict {
        run_id: internal(PayrollRunId {
            value: input.run_id,
        }),
        lines: financial(lines),
        run_net_delta: financial(MoneyAmount {
            amount_minor: run_net_delta_minor,
            currency: run_currency,
        }),
        balanced: Classified::new(balanced, DataClass::Public),
        evidence_digest: internal(EvidenceDigest {
            value: evidence_digest,
        }),
        schema_version: public(RETRO_ADJUSTMENT_SCHEMA_VERSION),
    })
}

pub fn trial_close(input: PayrollTrialCloseInput) -> Result<PayrollRun, PayrollDomainError> {
    validate_trial_close_input(&input)?;
    let mut payees = Vec::with_capacity(input.payees.len());
    for payee in input.payees {
        payees.push(build_payee(
            &input.tenant_id,
            &input.legal_entity_id,
            payee,
        )?);
    }
    let idempotency_key = format!("{}:{}:trial", input.run_id, input.rulepack_effective_date);
    Ok(PayrollRun {
        run_id: internal(PayrollRunId {
            value: input.run_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        period: internal(input.period),
        state: internal(PayrollRunState::TrialClosed),
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        rulepack_effective_date: internal(RulepackEffectiveDate {
            value: input.rulepack_effective_date,
        }),
        evidence_digest: internal(EvidenceDigest {
            value: input.evidence_digest,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        payees: Classified::new(payees, PrivacyDataClass::pii_identifying()),
        idempotency_key: internal(idempotency_key),
        schema_version: public(1),
    })
}

pub fn ingest_hr_leave_impact(
    input: HrLeaveImpactIntakeInput,
) -> Result<HrLeaveImpactIntake, PayrollDomainError> {
    validate_identifier(
        &input.run_id,
        RUN_ID_PREFIX,
        PayrollDomainError::InvalidRunId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        PayrollDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        PayrollDomainError::InvalidLegalEntityId,
    )?;
    validate_period(&input.payroll_period)?;
    validate_identifier(
        &input.payee_id,
        PAYEE_ID_PREFIX,
        PayrollDomainError::InvalidPayeeId,
    )?;
    validate_identifier(
        &input.employee_id,
        EMPLOYEE_ID_PREFIX,
        PayrollDomainError::InvalidEmployeeId,
    )?;
    validate_identifier(
        &input.leave_request_id,
        LEAVE_REQUEST_ID_PREFIX,
        PayrollDomainError::InvalidLeaveRequestId,
    )?;
    if input.source_topic != HR_LEAVE_IMPACT_SOURCE_TOPIC {
        return Err(PayrollDomainError::InvalidHrLeaveImpactTopic);
    }
    validate_idempotency_key(&input.source_hr_idempotency_key)?;
    validate_ref(
        &input.decision_evidence_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    validate_ref(
        &input.routing_evidence_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    validate_ref(
        &input.payroll_impact_evidence_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    validate_ref(
        &input.payroll_intake_evidence_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        PayrollDomainError::InvalidRulepackRef,
    )?;
    validate_iso_date(&input.rulepack_effective_date)?;
    if input.received_at_epoch_seconds == 0 {
        return Err(PayrollDomainError::InvalidReceivedAt);
    }
    let idempotency_key = format!(
        "{}:{}:{}:{}:{:?}:{}",
        input.run_id,
        input.payee_id,
        input.leave_request_id,
        input.payroll_period,
        input.impact_kind,
        input.source_hr_idempotency_key
    );
    Ok(HrLeaveImpactIntake {
        run_id: internal(PayrollRunId {
            value: input.run_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        payroll_period: financial(input.payroll_period),
        payee_id: internal(PayeeId {
            value: input.payee_id,
        }),
        employee_id: internal(EmployeeId {
            value: input.employee_id,
        }),
        leave_request_id: internal(LeaveRequestId {
            value: input.leave_request_id,
        }),
        impact_kind: financial(input.impact_kind),
        source_topic: internal(input.source_topic),
        source_hr_idempotency_key: internal(input.source_hr_idempotency_key),
        decision_evidence_ref: internal(EvidenceRef {
            value: input.decision_evidence_ref,
        }),
        routing_evidence_ref: internal(EvidenceRef {
            value: input.routing_evidence_ref,
        }),
        payroll_impact_evidence_ref: financial(EvidenceRef {
            value: input.payroll_impact_evidence_ref,
        }),
        payroll_intake_evidence_ref: internal(EvidenceRef {
            value: input.payroll_intake_evidence_ref,
        }),
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        rulepack_effective_date: internal(RulepackEffectiveDate {
            value: input.rulepack_effective_date,
        }),
        idempotency_key: internal(idempotency_key),
        received_at_epoch_seconds: internal(input.received_at_epoch_seconds),
        schema_version: public(HR_LEAVE_IMPACT_SCHEMA_VERSION),
    })
}

pub fn close_group_rollup(
    rollup_id: &str,
    tenant_id: &str,
    entity_closes: Vec<EntityCloseSnapshot>,
) -> Result<GroupPayrollRollup, PayrollDomainError> {
    validate_identifier(
        rollup_id,
        GROUP_ROLLUP_ID_PREFIX,
        PayrollDomainError::InvalidGroupRollupId,
    )?;
    validate_identifier(
        tenant_id,
        TENANT_ID_PREFIX,
        PayrollDomainError::InvalidTenantId,
    )?;
    if entity_closes.is_empty() {
        return Err(PayrollDomainError::EntityCloseIncomplete);
    }
    for close in &entity_closes {
        validate_identifier(
            &close.tenant_id,
            TENANT_ID_PREFIX,
            PayrollDomainError::InvalidTenantId,
        )?;
        validate_identifier(
            &close.legal_entity_id,
            LEGAL_ENTITY_ID_PREFIX,
            PayrollDomainError::InvalidLegalEntityId,
        )?;
        validate_identifier(
            &close.run_id,
            RUN_ID_PREFIX,
            PayrollDomainError::InvalidRunId,
        )?;
        validate_digest(&close.evidence_digest)?;
        if close.tenant_id != tenant_id || close.state != PayrollRunState::EntityClosed {
            return Err(PayrollDomainError::EntityCloseIncomplete);
        }
        if !close.detachment_history_redacted {
            return Err(PayrollDomainError::DetachmentHistoryNotRedacted);
        }
    }
    Ok(GroupPayrollRollup {
        rollup_id: internal(GroupPayrollRollupId {
            value: rollup_id.to_owned(),
        }),
        tenant_id: internal(TenantId {
            value: tenant_id.to_owned(),
        }),
        state: internal(PayrollRunState::GroupRolledUp),
        entity_closes: internal(entity_closes),
        idempotency_key: internal(format!("{tenant_id}:{rollup_id}:group-rollup")),
    })
}

pub fn statutory_export_evidence(
    input: StatutoryExportInput,
) -> Result<StatutoryExportEvidence, PayrollDomainError> {
    validate_identifier(
        &input.run_id,
        RUN_ID_PREFIX,
        PayrollDomainError::InvalidRunId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        PayrollDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        PayrollDomainError::InvalidLegalEntityId,
    )?;
    validate_digest(&input.export_hash)?;
    validate_ref(
        &input.rollback_plan_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    let receipt_ref = input
        .receipt_ref
        .map(|receipt| {
            validate_ref(
                &receipt,
                AUDIT_REF_PREFIX,
                PayrollDomainError::InvalidEvidenceRef,
            )?;
            Ok(EvidenceRef { value: receipt })
        })
        .transpose()?;
    let rejection_reason = input
        .rejection_reason
        .map(|reason| reason.trim().to_owned())
        .filter(|reason| !reason.is_empty());
    if receipt_ref.is_none() && rejection_reason.is_none() {
        return Err(PayrollDomainError::MissingReceiptOrRejection);
    }
    Ok(StatutoryExportEvidence {
        run_id: internal(PayrollRunId {
            value: input.run_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        export_kind: internal(input.export_kind),
        export_hash: internal(EvidenceDigest {
            value: input.export_hash,
        }),
        receipt_ref: internal(receipt_ref),
        rejection_reason: internal(rejection_reason),
        rollback_plan_ref: internal(EvidenceRef {
            value: input.rollback_plan_ref,
        }),
    })
}

pub fn build_statutory_rulepack_manifest(
    input: PayrollStatutoryRulepackManifestInput,
) -> Result<PayrollStatutoryRulepackManifest, PayrollDomainError> {
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        PayrollDomainError::InvalidRulepackRef,
    )?;
    validate_period(&input.payroll_period)?;
    validate_source_version(&input.source_version)?;
    validate_iso_date(&input.effective_date)?;
    validate_ref(
        &input.approval_evidence_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    if input.sources.is_empty() {
        return Err(PayrollDomainError::RulepackSourcesRequired);
    }
    if input.calculation_engine_attached
        || input.filing_rail_attached
        || input.disbursement_rail_attached
        || input.cloud_deployment_attached
    {
        return Err(PayrollDomainError::UnsupportedRulepackCapabilityClaim);
    }

    let source_count = input.sources.len();
    let mut sources = Vec::with_capacity(source_count);
    for source in input.sources {
        sources.push(build_rulepack_source(source)?);
    }

    Ok(PayrollStatutoryRulepackManifest {
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        jurisdiction: internal(input.jurisdiction),
        payroll_period: financial(input.payroll_period),
        source_version: internal(input.source_version),
        effective_date: internal(RulepackEffectiveDate {
            value: input.effective_date,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        sources: internal(sources),
        source_count: public(source_count),
        calculation_engine_attached: public(false),
        filing_rail_attached: public(false),
        disbursement_rail_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(STATUTORY_RULEPACK_SCHEMA_VERSION),
    })
}

pub fn build_payroll_statutory_source_pack(
    input: PayrollStatutorySourcePackInput,
) -> Result<PayrollStatutorySourcePack, PayrollDomainError> {
    validate_ref(
        &input.source_pack_ref,
        RULEPACK_SOURCE_PACK_REF_PREFIX,
        PayrollDomainError::InvalidRulepackSourceRef,
    )?;
    if input.payroll_year == 0 || input.payroll_year > 9999 {
        return Err(PayrollDomainError::InvalidPeriod);
    }
    validate_source_pack_fixture_note(input.fixture_note)?;
    if input.rows.is_empty() {
        return Err(PayrollDomainError::RulepackSourcesRequired);
    }
    if input.official_tax_rate_correctness_requested
        || input.calculation_engine_requested
        || input.filing_rail_requested
        || input.disbursement_rail_requested
        || input.cloud_deployment_requested
    {
        return Err(PayrollDomainError::UnsupportedRulepackCapabilityClaim);
    }

    let mut regions = std::collections::BTreeSet::new();
    let mut has_unresolved_blockers = false;
    let mut rows = Vec::with_capacity(input.rows.len());
    for row in input.rows {
        regions.insert(row.region);
        let (row, row_unresolved) = build_payroll_statutory_source_row(row)?;
        has_unresolved_blockers |= row_unresolved;
        rows.push(row);
    }

    Ok(PayrollStatutorySourcePack {
        source_pack_ref: internal(input.source_pack_ref),
        payroll_year: financial(input.payroll_year),
        rows: internal(rows),
        region_count: public(regions.len()),
        has_unresolved_blockers: public(has_unresolved_blockers),
        fixture_note: public(input.fixture_note.to_owned()),
        official_tax_rate_correctness_attached: public(false),
        calculation_engine_attached: public(false),
        filing_rail_attached: public(false),
        disbursement_rail_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(STATUTORY_SOURCE_PACK_SCHEMA_VERSION),
    })
}

pub fn calculate_statutory_deductions(
    input: StatutoryCalculationInput,
) -> Result<StatutoryCalculationDraft, PayrollDomainError> {
    let Some(rulepack_manifest_ref) = input.rulepack_manifest_ref.clone() else {
        return Err(PayrollDomainError::StatutoryRulepackManifestRequired);
    };
    let Some(rulepack_source_version) = input.rulepack_source_version.clone() else {
        return Err(PayrollDomainError::StatutoryRulepackManifestRequired);
    };

    validate_identifier(
        &input.run_id,
        RUN_ID_PREFIX,
        PayrollDomainError::InvalidRunId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        PayrollDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        PayrollDomainError::InvalidLegalEntityId,
    )?;
    validate_identifier(
        &input.payee_id,
        PAYEE_ID_PREFIX,
        PayrollDomainError::InvalidPayeeId,
    )?;
    validate_period(&input.payroll_period)?;
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        PayrollDomainError::InvalidRulepackRef,
    )?;
    validate_ref(
        &rulepack_manifest_ref,
        RULEPACK_SOURCE_REF_PREFIX,
        PayrollDomainError::InvalidRulepackSourceRef,
    )?;
    validate_source_version(&rulepack_source_version)?;

    if input.unofficial_source_fixture
        || rulepack_source_version == "synthetic-only"
        || input.official_source_evidence_refs.is_empty()
    {
        return Err(PayrollDomainError::OfficialRulepackSourceEvidenceRequired);
    }
    validate_fixture_note(
        input.fixture_note,
        &["no official", "rate correctness claim"],
        PayrollDomainError::OfficialRulepackSourceEvidenceRequired,
    )?;

    let expected_pack = expected_regional_pack(input.jurisdiction);
    if input.required_regional_pack != expected_pack {
        return Err(PayrollDomainError::StatutoryRegionalPackRequired);
    }

    if input.filing_rail_requested
        || input.disbursement_rail_requested
        || input.production_close_requested
        || input.cloud_deployment_requested
    {
        return Err(PayrollDomainError::UnsupportedRulepackCapabilityClaim);
    }

    let gross_pay = MoneyAmount {
        amount_minor: input.gross_pay_minor,
        currency: input.currency.clone(),
    };
    validate_money(&gross_pay)?;
    if input.rate_lines.is_empty() {
        return Err(PayrollDomainError::OfficialRulepackSourceEvidenceRequired);
    }

    let official_source_evidence_refs = input
        .official_source_evidence_refs
        .iter()
        .map(|source_ref| {
            validate_ref(
                source_ref,
                AUDIT_REF_PREFIX,
                PayrollDomainError::InvalidEvidenceRef,
            )?;
            Ok(EvidenceRef {
                value: source_ref.clone(),
            })
        })
        .collect::<Result<Vec<_>, PayrollDomainError>>()?;

    let mut total_deductions_minor = 0_i64;
    let mut deductions = Vec::with_capacity(input.rate_lines.len());
    for rate_line in input.rate_lines {
        if rate_line.synthetic_rate_basis_points == 0
            || rate_line.synthetic_rate_basis_points > 10_000
        {
            return Err(PayrollDomainError::InvalidMoney);
        }
        validate_ref(
            &rate_line.source_evidence_ref,
            AUDIT_REF_PREFIX,
            PayrollDomainError::InvalidEvidenceRef,
        )?;
        let amount_minor = input
            .gross_pay_minor
            .saturating_mul(i64::from(rate_line.synthetic_rate_basis_points))
            .saturating_div(10_000);
        total_deductions_minor = total_deductions_minor.saturating_add(amount_minor);
        deductions.push(StatutoryDeductionLine {
            kind: internal(rate_line.kind),
            amount: financial(MoneyAmount {
                amount_minor,
                currency: input.currency.clone(),
            }),
            synthetic_rate_basis_points: financial(rate_line.synthetic_rate_basis_points),
            source_evidence_ref: internal(EvidenceRef {
                value: rate_line.source_evidence_ref,
            }),
        });
    }
    if total_deductions_minor > input.gross_pay_minor {
        return Err(PayrollDomainError::InvalidMoney);
    }

    Ok(StatutoryCalculationDraft {
        run_id: internal(input.run_id),
        tenant_id: internal(input.tenant_id),
        legal_entity_id: internal(input.legal_entity_id),
        payee_id: internal(input.payee_id),
        payroll_period: financial(input.payroll_period),
        jurisdiction: internal(input.jurisdiction),
        required_regional_pack: internal(input.required_regional_pack),
        rulepack_ref: internal(input.rulepack_ref),
        rulepack_manifest_ref: internal(rulepack_manifest_ref),
        rulepack_source_version: internal(rulepack_source_version),
        official_source_evidence_refs: internal(official_source_evidence_refs),
        fixture_note: public(input.fixture_note.to_owned()),
        gross_pay: financial(gross_pay),
        deductions: financial(deductions),
        net_pay: financial(MoneyAmount {
            amount_minor: input.gross_pay_minor.saturating_sub(total_deductions_minor),
            currency: input.currency,
        }),
        boundary: public(CalculationBoundary::PureDomainNoFilingTransport),
        direct_agency_submission_attached: public(false),
        filing_rail_attached: public(false),
        disbursement_rail_attached: public(false),
        production_close_attached: public(false),
        cloud_deployment_attached: public(false),
    })
}

pub fn prepare_year_end_settlement_inputs(
    input: YearEndSettlementInput,
) -> Result<PreparedYearEndSettlementInput, PayrollDomainError> {
    let Some(rulepack_manifest_ref) = input.rulepack_manifest_ref.clone() else {
        return Err(PayrollDomainError::YearEndSettlementSourceEvidenceRequired);
    };
    let Some(source_version) = input.source_version.clone() else {
        return Err(PayrollDomainError::YearEndSettlementSourceEvidenceRequired);
    };
    if input.evidence_refs.is_empty() {
        return Err(PayrollDomainError::YearEndSettlementSourceEvidenceRequired);
    }

    validate_identifier(
        &input.run_id,
        RUN_ID_PREFIX,
        PayrollDomainError::InvalidRunId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        PayrollDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        PayrollDomainError::InvalidLegalEntityId,
    )?;
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        PayrollDomainError::InvalidRulepackRef,
    )?;
    validate_ref(
        &rulepack_manifest_ref,
        RULEPACK_SOURCE_REF_PREFIX,
        PayrollDomainError::InvalidRulepackSourceRef,
    )?;
    validate_source_version(&source_version)?;
    if !(1900..=2200).contains(&input.payroll_year) {
        return Err(PayrollDomainError::InvalidPeriod);
    }

    if input.unofficial_source_fixture || source_version == "synthetic-only" {
        return Err(PayrollDomainError::OfficialYearEndSourceEvidenceRequired);
    }
    validate_fixture_note(
        input.fixture_note,
        &["no production", "filing claim"],
        PayrollDomainError::OfficialYearEndSourceEvidenceRequired,
    )?;

    if input.regional_dependencies.is_empty() {
        return Err(PayrollDomainError::YearEndRegionalPackRequired);
    }
    let expected_pack = expected_regional_pack(input.jurisdiction);
    if !input
        .regional_dependencies
        .iter()
        .any(|dependency| dependency.pack_code == expected_pack)
    {
        return Err(PayrollDomainError::YearEndRegionalPackRequired);
    }

    if input.direct_agency_submission_requested
        || input.filing_rail_requested
        || input.disbursement_rail_requested
        || input.production_close_requested
        || input.cloud_deployment_requested
    {
        return Err(PayrollDomainError::UnsupportedRulepackCapabilityClaim);
    }

    let evidence_refs = input
        .evidence_refs
        .into_iter()
        .map(|evidence| {
            validate_ref(
                &evidence.ref_value,
                AUDIT_REF_PREFIX,
                PayrollDomainError::InvalidEvidenceRef,
            )?;
            validate_source_version(&evidence.source_version)?;
            if evidence.source_version != source_version {
                return Err(PayrollDomainError::OfficialYearEndSourceEvidenceRequired);
            }
            Ok(YearEndEvidenceRef {
                source_kind: internal(evidence.source_kind),
                ref_value: internal(evidence.ref_value),
                source_version: internal(evidence.source_version),
            })
        })
        .collect::<Result<Vec<_>, PayrollDomainError>>()?;

    let regional_dependencies = input
        .regional_dependencies
        .into_iter()
        .map(|dependency| {
            validate_pack_code(&dependency.pack_code)?;
            validate_source_version(&dependency.source_version)?;
            validate_ref(
                &dependency.evidence_ref,
                AUDIT_REF_PREFIX,
                PayrollDomainError::InvalidEvidenceRef,
            )?;
            if dependency.source_version != source_version {
                return Err(PayrollDomainError::OfficialYearEndSourceEvidenceRequired);
            }
            Ok(PreparedYearEndRegionalDependency {
                pack_code: internal(dependency.pack_code),
                source_version: internal(dependency.source_version),
                evidence_ref: internal(EvidenceRef {
                    value: dependency.evidence_ref,
                }),
            })
        })
        .collect::<Result<Vec<_>, PayrollDomainError>>()?;

    let employee_inputs = input
        .employee_inputs
        .into_iter()
        .map(|employee| {
            validate_identifier(
                &employee.payee_id,
                PAYEE_ID_PREFIX,
                PayrollDomainError::InvalidPayeeId,
            )?;
            validate_ref(
                &employee.employee_ref,
                PERSON_REF_PREFIX,
                PayrollDomainError::InvalidPartyRef,
            )?;
            let gross_pay = MoneyAmount {
                amount_minor: employee.gross_pay_minor,
                currency: employee.currency.clone(),
            };
            validate_money(&gross_pay)?;
            let withholding = MoneyAmount {
                amount_minor: employee.withholding_minor,
                currency: employee.currency.clone(),
            };
            if withholding.amount_minor < 0
                || withholding.currency.len() != 3
                || has_unsafe_text(&withholding.currency)
            {
                return Err(PayrollDomainError::InvalidMoney);
            }
            validate_ref(
                &employee.wage_ledger_evidence_ref,
                AUDIT_REF_PREFIX,
                PayrollDomainError::InvalidEvidenceRef,
            )?;
            validate_ref(
                &employee.declaration_evidence_ref,
                AUDIT_REF_PREFIX,
                PayrollDomainError::InvalidEvidenceRef,
            )?;
            Ok(PreparedYearEndEmployeeInput {
                payee_id: internal(employee.payee_id),
                employee_ref: Classified::new(
                    employee.employee_ref,
                    PrivacyDataClass::pii_identifying(),
                ),
                gross_pay: financial(gross_pay),
                withholding: financial(withholding),
                wage_ledger_evidence_ref: internal(EvidenceRef {
                    value: employee.wage_ledger_evidence_ref,
                }),
                declaration_evidence_ref: internal(EvidenceRef {
                    value: employee.declaration_evidence_ref,
                }),
            })
        })
        .collect::<Result<Vec<_>, PayrollDomainError>>()?;

    Ok(PreparedYearEndSettlementInput {
        run_id: internal(input.run_id),
        tenant_id: internal(input.tenant_id),
        legal_entity_id: internal(input.legal_entity_id),
        payroll_year: financial(input.payroll_year),
        jurisdiction: internal(input.jurisdiction),
        rulepack_ref: internal(input.rulepack_ref),
        rulepack_manifest_ref: internal(rulepack_manifest_ref),
        source_version: internal(source_version),
        fixture_note: public(input.fixture_note.to_owned()),
        evidence_refs: internal(evidence_refs),
        regional_dependencies: internal(regional_dependencies),
        employee_inputs: Classified::new(employee_inputs, PrivacyDataClass::pii_identifying()),
        direct_agency_submission_attached: public(false),
        filing_rail_attached: public(false),
        disbursement_rail_attached: public(false),
        production_close_attached: public(false),
        cloud_deployment_attached: public(false),
    })
}

pub fn build_payroll_journal(
    input: PayrollJournalInput,
) -> Result<PayrollJournalDraft, PayrollDomainError> {
    validate_identifier(
        &input.journal_id,
        JOURNAL_ID_PREFIX,
        PayrollDomainError::InvalidJournalId,
    )?;
    validate_identifier(
        &input.run_id,
        RUN_ID_PREFIX,
        PayrollDomainError::InvalidRunId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        PayrollDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        PayrollDomainError::InvalidLegalEntityId,
    )?;
    validate_period(&input.period)?;
    validate_digest(&input.source_payroll_digest)?;
    validate_ref(
        &input.approval_evidence_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    if input.lines.is_empty() {
        return Err(PayrollDomainError::JournalLinesRequired);
    }
    let mut debit = 0_i64;
    let mut credit = 0_i64;
    for line in &input.lines {
        validate_account_code(&line.account_code)?;
        let has_debit = line.debit_minor > 0;
        let has_credit = line.credit_minor > 0;
        if line.debit_minor < 0 || line.credit_minor < 0 || has_debit == has_credit {
            return Err(PayrollDomainError::InvalidMoney);
        }
        debit += line.debit_minor;
        credit += line.credit_minor;
    }
    if debit != credit {
        return Err(PayrollDomainError::UnbalancedJournal);
    }
    let reversal_required_ref = format!(
        "audit/{}/payroll/{}/reversal",
        input.legal_entity_id, input.run_id
    );
    Ok(PayrollJournalDraft {
        journal_id: internal(JournalId {
            value: input.journal_id,
        }),
        run_id: internal(PayrollRunId {
            value: input.run_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        source_payroll_digest: internal(EvidenceDigest {
            value: input.source_payroll_digest,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        total_debit_minor: internal(debit),
        total_credit_minor: internal(credit),
        reversal_required_ref: internal(EvidenceRef {
            value: reversal_required_ref,
        }),
    })
}

// ── Group GL posting dispatch types ───────────────────────────────────────

/// Input to `build_group_gl_posting`.  Carries one `PayrollJournalInput` per
/// closed legal entity in the rollup; each entry is independently validated
/// and balanced by the delegate `build_payroll_journal`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupGlPostingInput {
    pub rollup_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub entries: Vec<PayrollJournalInput>, // data_class: INTERNAL_ONLY
    pub group_idempotency_key: String,     // data_class: INTERNAL_ONLY
}

/// Output of `build_group_gl_posting`.  Holds one balanced `PayrollJournalDraft`
/// per entity, plus aggregated group-level totals and the idempotency key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupGlPostingBatch {
    pub rollup_id: Classified<GroupPayrollRollupId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,             // data_class: INTERNAL_ONLY
    pub drafts: Classified<Vec<PayrollJournalDraft>>, // data_class: INTERNAL_ONLY
    pub total_debit_minor: Classified<i64>,          // data_class: INTERNAL_ONLY
    pub total_credit_minor: Classified<i64>,         // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,         // data_class: INTERNAL_ONLY
}

pub fn build_group_gl_posting(
    input: GroupGlPostingInput,
) -> Result<GroupGlPostingBatch, PayrollDomainError> {
    validate_identifier(
        &input.rollup_id,
        GROUP_ROLLUP_ID_PREFIX,
        PayrollDomainError::InvalidGroupRollupId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        PayrollDomainError::InvalidTenantId,
    )?;
    validate_idempotency_key(&input.group_idempotency_key)?;
    if input.entries.is_empty() {
        return Err(PayrollDomainError::GroupPostingEntitiesRequired);
    }
    // Detect duplicate legal_entity_id values before any expensive work.
    let mut seen = std::collections::HashSet::new();
    for entry in &input.entries {
        if !seen.insert(entry.legal_entity_id.as_str()) {
            return Err(PayrollDomainError::DuplicateLegalEntityInGroup);
        }
    }
    let mut drafts: Vec<PayrollJournalDraft> = Vec::with_capacity(input.entries.len());
    let mut total_debit: i64 = 0;
    let mut total_credit: i64 = 0;
    for entry in input.entries {
        let draft = build_payroll_journal(entry)?;
        total_debit = total_debit.saturating_add(draft.total_debit_minor.value);
        total_credit = total_credit.saturating_add(draft.total_credit_minor.value);
        drafts.push(draft);
    }
    Ok(GroupGlPostingBatch {
        rollup_id: internal(GroupPayrollRollupId {
            value: input.rollup_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        drafts: internal(drafts),
        total_debit_minor: internal(total_debit),
        total_credit_minor: internal(total_credit),
        idempotency_key: internal(input.group_idempotency_key),
    })
}

pub fn evaluate_close_promotion(
    input: CloseHealthInput,
) -> Result<ClosePromotionDecision, PayrollDomainError> {
    validate_identifier(
        &input.run_id,
        RUN_ID_PREFIX,
        PayrollDomainError::InvalidRunId,
    )?;
    if input.canary_passed && input.evidence_gate_passed {
        return Ok(ClosePromotionDecision::Promote);
    }
    let Some(rollback_ref) = input.rollback_evidence_ref else {
        return Err(PayrollDomainError::RollbackEvidenceRequired);
    };
    let Some(quarantine_ref) = input.quarantine_evidence_ref else {
        return Err(PayrollDomainError::RollbackEvidenceRequired);
    };
    let Some(repair_route) = input.repair_route else {
        return Err(PayrollDomainError::RollbackEvidenceRequired);
    };
    validate_ref(
        &rollback_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    validate_ref(
        &quarantine_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    let route_metadata = close_rollback_route_metadata(repair_route);
    let observability_attributes = close_health_observability_attributes(
        &input.run_id,
        &rollback_ref,
        &quarantine_ref,
        &route_metadata.route_label,
    );
    Ok(ClosePromotionDecision::RollbackFirst {
        rollback_evidence_ref: EvidenceRef {
            value: rollback_ref,
        },
        quarantine_evidence_ref: EvidenceRef {
            value: quarantine_ref,
        },
        repair_route,
        promotion_stopped: true,
        route_metadata,
        observability_attributes,
    })
}

fn close_rollback_route_metadata(repair_route: RepairRoute) -> CloseRollbackRouteMetadata {
    match repair_route {
        RepairRoute::HotfixPullRequest => CloseRollbackRouteMetadata {
            route_label: "hotfix_pr".to_owned(),
            hotfix_pr_required: true,
            opentofu_ops_convergence_required: false,
            production_deploy_attached: false,
            workflow_execution_attached: false,
            opentofu_execution_attached: false,
        },
        RepairRoute::OpenTofuOpsConvergence => CloseRollbackRouteMetadata {
            route_label: "opentofu_ops_convergence".to_owned(),
            hotfix_pr_required: false,
            opentofu_ops_convergence_required: true,
            production_deploy_attached: false,
            workflow_execution_attached: false,
            opentofu_execution_attached: false,
        },
    }
}

fn close_health_observability_attributes(
    run_id: &str,
    rollback_ref: &str,
    quarantine_ref: &str,
    repair_route_label: &str,
) -> Vec<CloseHealthObservabilityAttribute> {
    [
        ("service.name", "payroll"),
        ("payroll.run_id", run_id),
        ("payroll.close.promotion_allowed", "false"),
        ("payroll.close.stop_reason", "close_health_gate_failed"),
        ("payroll.close.rollback_evidence_ref", rollback_ref),
        ("payroll.close.quarantine_evidence_ref", quarantine_ref),
        ("payroll.close.repair_route", repair_route_label),
        ("payroll.close.production_deploy_attached", "false"),
        ("payroll.close.workflow_execution_attached", "false"),
        ("payroll.close.opentofu_execution_attached", "false"),
    ]
    .into_iter()
    .map(|(key, value)| CloseHealthObservabilityAttribute {
        key: key.to_owned(),
        value: value.to_owned(),
    })
    .collect()
}

pub fn evaluate_payroll_variance(
    input: PayrollVarianceInput,
) -> Result<PayrollVarianceVerdict, PayrollDomainError> {
    // ── Validate scalar fields ────────────────────────────────────────────
    validate_identifier(
        &input.run_id,
        RUN_ID_PREFIX,
        PayrollDomainError::InvalidRunId,
    )?;
    if input.variance_tolerance_bps == 0 {
        return Err(PayrollDomainError::VarianceToleranceRequired);
    }
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        PayrollDomainError::InvalidRulepackRef,
    )?;
    validate_iso_date(&input.rulepack_effective_date)?;
    for ev_ref in &input.evidence_refs {
        validate_ref(
            ev_ref,
            AUDIT_REF_PREFIX,
            PayrollDomainError::InvalidEvidenceRef,
        )?;
    }
    if input.evaluated_at == 0 {
        return Err(PayrollDomainError::InvalidReceivedAt);
    }

    // ── Validate per-payee totals ─────────────────────────────────────────
    for total in input
        .current_period_totals
        .iter()
        .chain(input.prior_period_totals.iter())
    {
        validate_identifier(
            &total.payee_id,
            PAYEE_ID_PREFIX,
            PayrollDomainError::InvalidPayeeId,
        )?;
        validate_money(&total.net_amount)?;
    }

    // ── Build prior lookup map ─────────────────────────────────────────────
    // Collect prior payee IDs for dropped-payee detection.
    let prior_map: std::collections::HashMap<&str, &MoneyAmount> = input
        .prior_period_totals
        .iter()
        .map(|t| (t.payee_id.as_str(), &t.net_amount))
        .collect();

    let current_ids: std::collections::HashSet<&str> = input
        .current_period_totals
        .iter()
        .map(|t| t.payee_id.as_str())
        .collect();

    let tolerance = input.variance_tolerance_bps as u64;
    let mut lines: Vec<PayrollVarianceLine> = Vec::new();
    let mut anomaly_flags: Vec<AnomalyFlag> = Vec::new();
    let mut run_net_variance_bps: i64 = 0;

    // ── Per current-period payee: compute variance ────────────────────────
    for total in &input.current_period_totals {
        let payee_id_val = PayeeId {
            value: total.payee_id.clone(),
        };
        let current_minor = total.net_amount.amount_minor;

        let (prior_amount, variance_bps, is_anomaly) = match prior_map.get(total.payee_id.as_str())
        {
            Some(prior) => {
                let prior_minor = prior.amount_minor;
                let (bps, anomalous) = if prior_minor == 0 {
                    // Prior zero → cannot compute ratio; treat as over-tolerance.
                    (0_i64, true)
                } else {
                    let raw = (current_minor.saturating_sub(prior_minor))
                        .saturating_mul(10_000)
                        .saturating_div(prior_minor.abs());
                    let over = raw.unsigned_abs() > tolerance;
                    let sign_flip = current_minor != 0
                        && prior_minor != 0
                        && current_minor.signum() != prior_minor.signum();
                    if over {
                        anomaly_flags.push(AnomalyFlag::OverToleranceSwing {
                            payee_id: payee_id_val.clone(),
                        });
                    }
                    if sign_flip {
                        anomaly_flags.push(AnomalyFlag::SignFlip {
                            payee_id: payee_id_val.clone(),
                        });
                    }
                    (raw, over || sign_flip)
                };
                let prior_clone = MoneyAmount {
                    amount_minor: prior.amount_minor,
                    currency: prior.currency.clone(),
                };
                (prior_clone, bps, anomalous)
            }
            None => {
                // A current-period payee with no prior-period baseline entry is
                // a strict-mode error: the caller must supply a complete baseline.
                return Err(PayrollDomainError::MissingBaselineForPayee);
            }
        };

        run_net_variance_bps = run_net_variance_bps.saturating_add(variance_bps);
        lines.push(PayrollVarianceLine {
            payee_id: internal(payee_id_val),
            current_amount: financial(total.net_amount.clone()),
            prior_amount: financial(prior_amount),
            variance_bps: internal(variance_bps),
            anomaly: internal(is_anomaly),
        });
    }

    // ── Dropped-payee detection ───────────────────────────────────────────
    for prior_total in &input.prior_period_totals {
        if !current_ids.contains(prior_total.payee_id.as_str()) {
            let payee_id_val = PayeeId {
                value: prior_total.payee_id.clone(),
            };
            anomaly_flags.push(AnomalyFlag::DroppedPayee {
                payee_id: payee_id_val.clone(),
            });
            run_net_variance_bps = run_net_variance_bps.saturating_add(DROPPED_PAYEE_SENTINEL_BPS);
            // Emit a synthetic line so callers can audit the dropped entry.
            lines.push(PayrollVarianceLine {
                payee_id: internal(payee_id_val),
                current_amount: financial(MoneyAmount {
                    amount_minor: 0,
                    currency: prior_total.net_amount.currency.clone(),
                }),
                prior_amount: financial(prior_total.net_amount.clone()),
                variance_bps: internal(DROPPED_PAYEE_SENTINEL_BPS),
                anomaly: internal(true),
            });
        }
    }

    let gate_passed = anomaly_flags.is_empty();

    // ── Evidence digest (deterministic XOR fold, no external crate) ───────
    // XOR all UTF-8 bytes of each evidence_ref (concatenated in order) into a
    // 32-byte buffer by position modulo 32, then hex-encode.  Label as
    // "sha256:" to match the existing EvidenceDigest prefix convention; this
    // is a structural fingerprint, not a cryptographic hash.
    let mut buf = [0u8; 32];
    let mut pos = 0usize;
    for ev_ref in &input.evidence_refs {
        for byte in ev_ref.as_bytes() {
            buf[pos % 32] ^= byte;
            pos += 1;
        }
    }
    let hex_chars: String = buf.iter().map(|b| format!("{b:02x}")).collect();
    let evidence_digest = format!("{HASH_PREFIX}{hex_chars}");

    Ok(PayrollVarianceVerdict {
        run_id: internal(PayrollRunId {
            value: input.run_id,
        }),
        lines: financial(lines),
        run_net_variance_bps: internal(run_net_variance_bps),
        anomaly_flags: internal(anomaly_flags),
        gate_passed: internal(gate_passed),
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        rulepack_effective_date: internal(RulepackEffectiveDate {
            value: input.rulepack_effective_date,
        }),
        evidence_digest: internal(EvidenceDigest {
            value: evidence_digest,
        }),
        evaluated_at: internal(input.evaluated_at),
        schema_version: public(VARIANCE_VERDICT_SCHEMA_VERSION),
    })
}

fn build_payroll_statutory_source_row(
    row: PayrollStatutorySourceRowInput,
) -> Result<(PayrollStatutorySourceRow, bool), PayrollDomainError> {
    validate_statutory_source_text(
        &row.publisher,
        PayrollDomainError::InvalidRulepackSourceVersion,
    )?;
    validate_source_version(&row.version_label)?;
    validate_source_pack_fixture_note(row.fixture_note)?;
    validate_statutory_source_owner(&row.owner)?;

    let official_url_or_path = match row.official_url_or_path {
        Some(url_or_path) => {
            validate_official_source_url(&url_or_path)?;
            Some(url_or_path)
        }
        None => None,
    };
    let effective_date = match row.effective_date {
        Some(date) => {
            validate_iso_date(&date)?;
            Some(RulepackEffectiveDate { value: date })
        }
        None => None,
    };
    let expires_on = match row.expires_on {
        Some(date) => {
            validate_iso_date(&date)?;
            Some(RulepackEffectiveDate { value: date })
        }
        None => None,
    };
    if let Some(source_ref) = &row.supersedes_source_ref {
        validate_payroll_source_ref(source_ref)?;
    }

    let unresolved_status = matches!(
        row.retrieval_status,
        PayrollStatutorySourceRetrievalStatus::Blocked
            | PayrollStatutorySourceRetrievalStatus::Missing
    );
    let unresolved_blocker_reason = match row.unresolved_blocker_reason {
        Some(reason) => {
            validate_statutory_source_text(
                &reason,
                PayrollDomainError::OfficialSourceBlockerReasonRequired,
            )?;
            Some(reason)
        }
        None if unresolved_status => {
            return Err(PayrollDomainError::OfficialSourceBlockerReasonRequired);
        }
        None => None,
    };

    if !unresolved_status {
        if official_url_or_path.is_none() {
            return Err(PayrollDomainError::InvalidRulepackSourceUrl);
        }
        if effective_date.is_none() {
            return Err(PayrollDomainError::InvalidRulepackEffectiveDate);
        }
    }

    let source_digest = match row.source_digest {
        Some(digest) => {
            validate_source_pack_digest(&digest)?;
            Some(EvidenceDigest { value: digest })
        }
        None if !unresolved_status => {
            return Err(PayrollDomainError::OfficialSourceDigestRequired);
        }
        None => None,
    };
    let approval_evidence_ref = match row.approval_evidence_ref {
        Some(evidence_ref) => {
            validate_ref(
                &evidence_ref,
                AUDIT_REF_PREFIX,
                PayrollDomainError::InvalidEvidenceRef,
            )?;
            Some(EvidenceRef {
                value: evidence_ref,
            })
        }
        None if !unresolved_status => {
            return Err(PayrollDomainError::OfficialSourceApprovalEvidenceRequired);
        }
        None => None,
    };

    validate_statutory_source_lifecycle(row.applicability, row.cadence, unresolved_status)?;

    let has_unresolved_blocker = unresolved_status || unresolved_blocker_reason.is_some();

    Ok((
        PayrollStatutorySourceRow {
            region: internal(row.region),
            source_kind: internal(row.source_kind),
            publisher: public(row.publisher),
            official_url_or_path: public(official_url_or_path),
            version_label: internal(row.version_label),
            effective_date: internal(effective_date),
            retrieval_status: internal(row.retrieval_status),
            source_digest: internal(source_digest),
            approval_evidence_ref: internal(approval_evidence_ref),
            applicability: internal(row.applicability),
            cadence: internal(row.cadence),
            owner: internal(row.owner),
            supersedes_source_ref: internal(row.supersedes_source_ref),
            expires_on: internal(expires_on),
            unresolved_blocker_reason: internal(unresolved_blocker_reason),
            fixture_note: public(row.fixture_note.to_owned()),
        },
        has_unresolved_blocker,
    ))
}

fn build_rulepack_source(
    source: PayrollRulepackSourceInput,
) -> Result<PayrollRulepackSource, PayrollDomainError> {
    validate_ref(
        &source.source_ref,
        RULEPACK_SOURCE_REF_PREFIX,
        PayrollDomainError::InvalidRulepackSourceRef,
    )?;
    validate_official_source_url(&source.official_url)?;
    validate_source_version(&source.version_label)?;
    validate_iso_date(&source.effective_date)?;
    if source.retrieved_at_epoch_seconds == 0 {
        return Err(PayrollDomainError::InvalidReceivedAt);
    }
    validate_ref(
        &source.evidence_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    validate_digest(&source.digest)?;

    Ok(PayrollRulepackSource {
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

fn validate_trial_close_input(input: &PayrollTrialCloseInput) -> Result<(), PayrollDomainError> {
    validate_identifier(
        &input.run_id,
        RUN_ID_PREFIX,
        PayrollDomainError::InvalidRunId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        PayrollDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        PayrollDomainError::InvalidLegalEntityId,
    )?;
    validate_period(&input.period)?;
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        PayrollDomainError::InvalidRulepackRef,
    )?;
    validate_iso_date(&input.rulepack_effective_date)?;
    validate_digest(&input.evidence_digest)?;
    validate_ref(
        &input.approval_evidence_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    if input.payees.is_empty() {
        return Err(PayrollDomainError::TrialCloseRequiresPayees);
    }
    Ok(())
}

fn build_payee(
    tenant_id: &str,
    run_legal_entity_id: &str,
    payee: PayeeInput,
) -> Result<Payee, PayrollDomainError> {
    validate_identifier(
        &payee.payee_id,
        PAYEE_ID_PREFIX,
        PayrollDomainError::InvalidPayeeId,
    )?;
    validate_identifier(
        &payee.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        PayrollDomainError::InvalidLegalEntityId,
    )?;
    if payee.legal_entity_id != run_legal_entity_id {
        return Err(PayrollDomainError::PayeeLegalEntityMismatch);
    }
    let party_ref_prefix = if payee.person_or_vendor_ref.starts_with(PERSON_REF_PREFIX) {
        PERSON_REF_PREFIX
    } else if payee.person_or_vendor_ref.starts_with(VENDOR_REF_PREFIX) {
        VENDOR_REF_PREFIX
    } else {
        return Err(PayrollDomainError::InvalidPartyRef);
    };
    validate_ref(
        &payee.person_or_vendor_ref,
        party_ref_prefix,
        PayrollDomainError::InvalidPartyRef,
    )?;
    validate_ref(
        &payee.tax_profile_ref,
        TAX_PROFILE_REF_PREFIX,
        PayrollDomainError::InvalidTaxProfileRef,
    )?;
    if payee.wage_ledger.is_empty() {
        return Err(PayrollDomainError::PayeeMissingWageLedger);
    }
    let mut wage_ledger = Vec::with_capacity(payee.wage_ledger.len());
    for entry in payee.wage_ledger {
        wage_ledger.push(build_wage_entry(&payee.payee_id, entry)?);
    }
    Ok(Payee {
        payee_id: internal(PayeeId {
            value: payee.payee_id,
        }),
        tenant_id: internal(TenantId {
            value: tenant_id.to_owned(),
        }),
        legal_entity_id: internal(LegalEntityId {
            value: payee.legal_entity_id,
        }),
        payee_class: internal(payee.payee_class),
        person_or_vendor_ref: Classified::new(
            PartyRef {
                value: payee.person_or_vendor_ref,
            },
            PrivacyDataClass::pii_identifying(),
        ),
        tax_profile_ref: internal(TaxProfileRef {
            value: payee.tax_profile_ref,
        }),
        wage_ledger: financial(wage_ledger),
    })
}

fn build_wage_entry(
    expected_payee_id: &str,
    entry: WageLedgerEntryInput,
) -> Result<WageLedgerEntry, PayrollDomainError> {
    validate_identifier(
        &entry.entry_id,
        WAGE_ENTRY_ID_PREFIX,
        PayrollDomainError::InvalidWageEntryId,
    )?;
    validate_identifier(
        &entry.payee_id,
        PAYEE_ID_PREFIX,
        PayrollDomainError::InvalidPayeeId,
    )?;
    if entry.payee_id != expected_payee_id {
        return Err(PayrollDomainError::WageEntryPayeeMismatch);
    }
    validate_money(&entry.amount)?;
    validate_ref(
        &entry.source_ref,
        AUDIT_REF_PREFIX,
        PayrollDomainError::InvalidEvidenceRef,
    )?;
    Ok(WageLedgerEntry {
        entry_id: internal(WageLedgerEntryId {
            value: entry.entry_id,
        }),
        payee_id: internal(PayeeId {
            value: entry.payee_id,
        }),
        line_kind: internal(entry.line_kind),
        amount: financial(entry.amount),
        source_ref: internal(EvidenceRef {
            value: entry.source_ref,
        }),
    })
}

fn validate_identifier(
    value: &str,
    prefix: &str,
    error: PayrollDomainError,
) -> Result<(), PayrollDomainError> {
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
    error: PayrollDomainError,
) -> Result<(), PayrollDomainError> {
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

fn validate_digest(value: &str) -> Result<(), PayrollDomainError> {
    let Some(hex) = value.strip_prefix(HASH_PREFIX) else {
        return Err(PayrollDomainError::InvalidEvidenceDigest);
    };
    if hex.len() != 64 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(PayrollDomainError::InvalidEvidenceDigest);
    }
    Ok(())
}

fn validate_source_pack_digest(value: &str) -> Result<(), PayrollDomainError> {
    validate_digest(value)
}

fn validate_period(value: &str) -> Result<(), PayrollDomainError> {
    let bytes = value.as_bytes();
    if bytes.len() != 7
        || bytes[4] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| idx == 4 || byte.is_ascii_digit())
    {
        return Err(PayrollDomainError::InvalidPeriod);
    }
    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| PayrollDomainError::InvalidPeriod)?;
    if !(1..=12).contains(&month) {
        return Err(PayrollDomainError::InvalidPeriod);
    }
    Ok(())
}

fn validate_iso_date(value: &str) -> Result<(), PayrollDomainError> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| matches!(idx, 4 | 7) || byte.is_ascii_digit())
    {
        return Err(PayrollDomainError::InvalidRulepackEffectiveDate);
    }
    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| PayrollDomainError::InvalidRulepackEffectiveDate)?;
    let day = value[8..10]
        .parse::<u8>()
        .map_err(|_| PayrollDomainError::InvalidRulepackEffectiveDate)?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(PayrollDomainError::InvalidRulepackEffectiveDate);
    }
    Ok(())
}

fn validate_source_version(value: &str) -> Result<(), PayrollDomainError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.len() > 96
        || has_unsafe_text(trimmed)
        || trimmed.contains("..")
        || !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
    {
        return Err(PayrollDomainError::InvalidRulepackSourceVersion);
    }
    Ok(())
}

fn validate_fixture_note(
    value: &str,
    required_markers: &[&str],
    error: PayrollDomainError,
) -> Result<(), PayrollDomainError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(error);
    }
    let lowered = value.to_ascii_lowercase();
    if !lowered.contains(SYNTHETIC_NON_AUTHORITATIVE_FIXTURE_LABEL)
        || required_markers
            .iter()
            .any(|marker| !lowered.contains(marker))
    {
        return Err(error);
    }
    Ok(())
}

fn validate_source_pack_fixture_note(value: &str) -> Result<(), PayrollDomainError> {
    validate_statutory_source_text(
        value,
        PayrollDomainError::OfficialRulepackSourceEvidenceRequired,
    )?;
    let lowered = value.to_ascii_lowercase();
    if !lowered.contains("no official") || !lowered.contains("rate correctness claim") {
        return Err(PayrollDomainError::OfficialRulepackSourceEvidenceRequired);
    }
    Ok(())
}

fn validate_statutory_source_lifecycle(
    applicability: PayrollStatutorySourceApplicability,
    cadence: PayrollStatutorySourceCadence,
    unresolved_status: bool,
) -> Result<(), PayrollDomainError> {
    if unresolved_status {
        if applicability != PayrollStatutorySourceApplicability::RegionalPackInventoryOnly {
            return Err(PayrollDomainError::OfficialSourceApplicabilityRequired);
        }
        if cadence != PayrollStatutorySourceCadence::Unresolved {
            return Err(PayrollDomainError::OfficialSourceCadenceRequired);
        }
        return Ok(());
    }

    if applicability == PayrollStatutorySourceApplicability::RegionalPackInventoryOnly {
        return Err(PayrollDomainError::OfficialSourceApplicabilityRequired);
    }
    if cadence == PayrollStatutorySourceCadence::Unresolved {
        return Err(PayrollDomainError::OfficialSourceCadenceRequired);
    }
    Ok(())
}

fn validate_statutory_source_text(
    value: &str,
    error: PayrollDomainError,
) -> Result<(), PayrollDomainError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(error);
    }
    Ok(())
}

fn validate_statutory_source_owner(value: &str) -> Result<(), PayrollDomainError> {
    if value.trim().is_empty()
        || value != value.trim()
        || value.chars().any(char::is_control)
        || value.contains("..")
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '.'))
    {
        return Err(PayrollDomainError::OfficialSourceOwnerRequired);
    }
    Ok(())
}

fn validate_payroll_source_ref(value: &str) -> Result<(), PayrollDomainError> {
    if value.starts_with(RULEPACK_SOURCE_PACK_REF_PREFIX) {
        validate_ref(
            value,
            RULEPACK_SOURCE_PACK_REF_PREFIX,
            PayrollDomainError::InvalidRulepackSourceRef,
        )
    } else {
        validate_ref(
            value,
            RULEPACK_SOURCE_REF_PREFIX,
            PayrollDomainError::InvalidRulepackSourceRef,
        )
    }
}

fn validate_official_source_url(value: &str) -> Result<(), PayrollDomainError> {
    if has_unsafe_text(value) || !value.starts_with("https://") {
        return Err(PayrollDomainError::InvalidRulepackSourceUrl);
    }
    let allowed = [
        "https://www.irs.gov/",
        "https://www.dol.gov/",
        "https://www.moel.go.kr/",
        "https://law.go.kr/",
    ];
    if !allowed.iter().any(|prefix| value.starts_with(prefix)) {
        return Err(PayrollDomainError::InvalidRulepackSourceUrl);
    }
    if value.contains("..") || value.contains('\\') {
        return Err(PayrollDomainError::InvalidRulepackSourceUrl);
    }
    Ok(())
}

fn validate_pack_code(value: &str) -> Result<(), PayrollDomainError> {
    if !matches!(value, "KR" | "US" | "EU") {
        return Err(PayrollDomainError::YearEndRegionalPackRequired);
    }
    Ok(())
}

fn expected_regional_pack(jurisdiction: PayrollRulepackJurisdiction) -> &'static str {
    match jurisdiction {
        PayrollRulepackJurisdiction::Korea => "KR",
        PayrollRulepackJurisdiction::UnitedStatesFederal => "US",
        PayrollRulepackJurisdiction::EuropeanUnion => "EU",
    }
}

fn validate_money(amount: &MoneyAmount) -> Result<(), PayrollDomainError> {
    if amount.amount_minor == 0 || amount.currency.len() != 3 || has_unsafe_text(&amount.currency) {
        return Err(PayrollDomainError::InvalidMoney);
    }
    Ok(())
}

fn validate_account_code(value: &str) -> Result<(), PayrollDomainError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(PayrollDomainError::InvalidAccountCode);
    }
    Ok(())
}

fn validate_idempotency_key(value: &str) -> Result<(), PayrollDomainError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || value.contains("..")
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-'))
    {
        return Err(PayrollDomainError::InvalidIdempotencyKey);
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
