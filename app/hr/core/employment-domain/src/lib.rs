//! HR employment domain foundation.
//!
//! This crate owns pure HR invariants for employee and employment records:
//! legal-entity-scoped employees, audit-backed lifecycle events, and
//! Korea-first labor-compliance threshold obligations. Tenant RBAC view and
//! Tenant RBAC view remain product-surface metadata; this crate is not an enterprise
//! platform boundary. It does not perform storage, workflow dispatch, payroll
//! derivation, or regulator filing I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const EMPLOYEE_SCHEMA_VERSION: u32 = 1;
const HR_EVENT_SCHEMA_VERSION: u32 = 1;
const LABOR_OBLIGATION_SCHEMA_VERSION: u32 = 1;
const EMPLOYEE_ID_PREFIX: &str = "emp_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const PERSON_REF_PREFIX: &str = "person/";
const AUDIT_EVIDENCE_PREFIX: &str = "audit/";
const RULEPACK_REF_PREFIX: &str = "rulepack/";
const HR_EVENT_ID_PREFIX: &str = "hrev_";
const WORKFLOW_REF_PREFIX: &str = "workflow/";
const LABOR_OBLIGATION_ID_PREFIX: &str = "hrobl_";
const LEAVE_REQUEST_ID_PREFIX: &str = "leave_";
const HR_RULEPACK_SOURCE_REF_PREFIX: &str = "hr-rulepack-source/";
const SOURCE_DIGEST_PREFIX: &str = "sha256:";
const LEAVE_PAYROLL_IMPACT_SCHEMA_VERSION: u32 = 1;
const HR_POLICY_REF_PREFIX: &str = "policy/hr/sensitive-read/";
const SENSITIVE_HR_READ_SCHEMA_VERSION: u32 = 1;
const HR_STATUTORY_RULEPACK_SCHEMA_VERSION: u32 = 1;
const LEAVE_BALANCE_LEDGER_SCHEMA_VERSION: u32 = 1;
const LEAVE_CARRYOVER_FORFEITURE_SCHEMA_VERSION: u32 = 1;
const ONBOARDING_READINESS_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EmployeeId {
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
pub struct PersonRef {
    pub value: String, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AuditEvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HrEventId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RulepackRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorkflowRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PolicyRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LaborComplianceObligationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LeaveRequestId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RulepackEffectiveDate {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RulepackSourceDigest {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EmploymentStatus {
    Draft,
    Active,
    Suspended,
    Terminated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TenantTierSnapshot {
    SmbSelfServe,
    EnterpriseSingleEntity,
    EnterpriseGroup,
    RegulatedEnterprise,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum HrLifecycleKind {
    Created,
    Updated,
    Suspended,
    Terminated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Jurisdiction {
    Korea,
    UnitedStates,
    EuropeanUnion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum HrRulepackSourceKind {
    LaborStandards,
    RulesOfEmployment,
    LaborManagementCouncil,
    LeaveAndHolidayStandards,
    WageHourRecordkeeping,
    EqualEmployment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LaborComplianceObligationKind {
    KoreaRulesOfEmployment,
    KoreaLaborManagementCouncil,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LaborComplianceWorkflowStep {
    Drafted,
    EmployeeReviewSent,
    MajorityConsentObtained,
    MoelFiled,
    CouncilRosterRequired,
    MeetingCadenceRequired,
    MinutesEvidenceRequired,
    Active,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LaborComplianceObligationState {
    Open,
    Active,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LeaveDecision {
    Approved,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LeaveRoutingMode {
    DirectManager,
    DelegatedApprover,
    EscalatedHr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayrollImpactKind {
    PaidLeave,
    UnpaidLeaveDeduction,
    AttendanceCorrection,
    NoPayrollImpact,
}

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
pub struct EmployeeCreate {
    pub employee_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                  // data_class: INTERNAL_ONLY
    pub person_ref: String,                       // data_class: PII_IDENTIFYING
    pub manager_id: Option<String>,               // data_class: INTERNAL_ONLY
    pub employment_status: EmploymentStatus,      // data_class: INTERNAL_ONLY
    pub tenant_tier_snapshot: TenantTierSnapshot, // data_class: INTERNAL_ONLY
    pub audit_evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>,     // data_class: INTERNAL_ONLY
    pub version: u32,                             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Employee {
    pub employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,     // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub person_ref: Classified<PersonRef>,   // data_class: PII_IDENTIFYING
    pub manager_id: Classified<Option<EmployeeId>>, // data_class: INTERNAL_ONLY
    pub employment_status: Classified<EmploymentStatus>, // data_class: INTERNAL_ONLY
    pub tenant_tier_snapshot: Classified<TenantTierSnapshot>, // data_class: INTERNAL_ONLY
    pub audit_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub version: Classified<u32>,            // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmployeeLifecycleEvent {
    pub event_id: Classified<HrEventId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub lifecycle_kind: Classified<HrLifecycleKind>, // data_class: INTERNAL_ONLY
    pub audit_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegalEntityWorkforceSnapshot {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,         // data_class: INTERNAL_ONLY
    pub jurisdiction: Jurisdiction,      // data_class: INTERNAL_ONLY
    pub active_employee_count: u32,      // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,            // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String, // data_class: INTERNAL_ONLY
    pub workflow_ref: String,            // data_class: INTERNAL_ONLY
    pub evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaborComplianceObligation {
    pub obligation_id: Classified<LaborComplianceObligationId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                        // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,             // data_class: INTERNAL_ONLY
    pub jurisdiction: Classified<Jurisdiction>,                 // data_class: INTERNAL_ONLY
    pub kind: Classified<LaborComplianceObligationKind>,        // data_class: INTERNAL_ONLY
    pub state: Classified<LaborComplianceObligationState>,      // data_class: INTERNAL_ONLY
    pub threshold_employee_count: Classified<u32>,              // data_class: INTERNAL_ONLY
    pub active_employee_count: Classified<u32>,                 // data_class: INTERNAL_ONLY
    pub rulepack_ref: Classified<RulepackRef>,                  // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub workflow_ref: Classified<WorkflowRef>,                  // data_class: INTERNAL_ONLY
    pub workflow_steps: Classified<Vec<LaborComplianceWorkflowStep>>, // data_class: INTERNAL_ONLY
    pub evidence_paths: Classified<Vec<AuditEvidenceRef>>,      // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: Classified<u64>,            // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,                        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeavePayrollImpactInput {
    pub leave_request_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                // data_class: INTERNAL_ONLY
    pub employee_id: String,                    // data_class: INTERNAL_ONLY
    pub approver_id: String,                    // data_class: INTERNAL_ONLY
    pub decision: LeaveDecision,                // data_class: INTERNAL_ONLY
    pub routing_mode: LeaveRoutingMode,         // data_class: INTERNAL_ONLY
    pub start_date: String,                     // data_class: INTERNAL_ONLY
    pub end_date: String,                       // data_class: INTERNAL_ONLY
    pub payroll_period: String,                 // data_class: FINANCIAL
    pub payroll_impact_kind: PayrollImpactKind, // data_class: FINANCIAL
    pub workflow_ref: String,                   // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                   // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String,        // data_class: INTERNAL_ONLY
    pub decision_evidence_ref: String,          // data_class: INTERNAL_ONLY
    pub routing_evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub payroll_impact_evidence_ref: String,    // data_class: FINANCIAL
    pub decided_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeavePayrollImpactPlan {
    pub leave_request_id: Classified<LeaveRequestId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,   // data_class: INTERNAL_ONLY
    pub employee_id: Classified<EmployeeId>,          // data_class: INTERNAL_ONLY
    pub approver_id: Classified<EmployeeId>,          // data_class: INTERNAL_ONLY
    pub decision: Classified<LeaveDecision>,          // data_class: INTERNAL_ONLY
    pub routing_mode: Classified<LeaveRoutingMode>,   // data_class: INTERNAL_ONLY
    pub start_date: Classified<String>,               // data_class: INTERNAL_ONLY
    pub end_date: Classified<String>,                 // data_class: INTERNAL_ONLY
    pub payroll_period: Classified<String>,           // data_class: FINANCIAL
    pub payroll_impact_kind: Classified<PayrollImpactKind>, // data_class: FINANCIAL
    pub workflow_ref: Classified<WorkflowRef>,        // data_class: INTERNAL_ONLY
    pub rulepack_ref: Classified<RulepackRef>,        // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub decision_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub routing_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub payroll_impact_evidence_ref: Classified<AuditEvidenceRef>, // data_class: FINANCIAL
    pub idempotency_key: Classified<String>,          // data_class: INTERNAL_ONLY
    pub decided_at_epoch_seconds: Classified<u64>,    // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,              // data_class: PUBLIC
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

#[derive(Clone, Debug, PartialEq)]
pub struct LeaveBalanceAccrualInput {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,         // data_class: INTERNAL_ONLY
    pub employee_id: String,             // data_class: INTERNAL_ONLY
    pub payroll_period: String,          // data_class: FINANCIAL
    pub prior_accrued_units: f64,        // data_class: FINANCIAL
    pub accrual_units: f64,              // data_class: FINANCIAL
    pub deduction_units: f64,            // data_class: FINANCIAL
    pub carry_over_cap_units: f64,       // data_class: FINANCIAL
    pub rulepack_ref: String,            // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String, // data_class: INTERNAL_ONLY
    pub accrual_evidence_ref: String,    // data_class: INTERNAL_ONLY
    pub deduction_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub decided_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, PartialEq)]
pub struct LeaveBalanceLedgerProjection {
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub payroll_period: Classified<String>, // data_class: FINANCIAL
    pub prior_accrued_units: Classified<f64>, // data_class: FINANCIAL
    pub accrual_units: Classified<f64>,  // data_class: FINANCIAL
    pub deduction_units: Classified<f64>, // data_class: FINANCIAL
    pub resulting_balance_units: Classified<f64>, // data_class: FINANCIAL
    pub carried_over_units: Classified<f64>, // data_class: FINANCIAL
    pub forfeited_units: Classified<f64>, // data_class: FINANCIAL
    pub carry_over_cap_units: Classified<f64>, // data_class: FINANCIAL
    pub rulepack_ref: Classified<RulepackRef>, // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub accrual_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub deduction_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub decided_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

/// Input to the leave carry-over / forfeiture period-boundary evaluator.
///
/// Distinct from `LeaveBalanceAccrualInput`: this function splits the closing
/// balance instead of hard-erroring when the cap is exceeded.
#[derive(Clone, Debug, PartialEq)]
pub struct LeaveCarryoverForfeitureInput {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,         // data_class: INTERNAL_ONLY
    pub employee_id: String,             // data_class: INTERNAL_ONLY
    pub period_boundary_date: String,    // data_class: INTERNAL_ONLY (ISO-8601 YYYY-MM-DD)
    pub closing_balance_units: f64,      // data_class: FINANCIAL
    pub statutory_min_floor_units: f64,  // data_class: FINANCIAL
    pub carry_over_cap_units: f64,       // data_class: FINANCIAL
    pub rulepack_ref: String,            // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

/// Projection produced by `evaluate_leave_carryover_forfeiture`.
#[derive(Clone, Debug, PartialEq)]
pub struct LeaveCarryoverForfeitureProjection {
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub period_boundary_date: Classified<String>, // data_class: INTERNAL_ONLY
    pub closing_balance_units: Classified<f64>, // data_class: FINANCIAL
    pub statutory_min_floor_units: Classified<f64>, // data_class: FINANCIAL
    pub carry_over_cap_units: Classified<f64>, // data_class: FINANCIAL
    pub carried_over_units: Classified<f64>, // data_class: FINANCIAL
    pub forfeited_units: Classified<f64>, // data_class: FINANCIAL
    pub rulepack_ref: Classified<RulepackRef>, // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

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

/// Pure period-boundary evaluator that splits a closing leave balance into
/// `carried_over_units` (clamped to `[statutory_min_floor, cap]`) and
/// `forfeited_units` (excess above cap).
///
/// Unlike `evaluate_leave_balance_accrual`, this function does **not** error
/// when the closing balance exceeds the cap; it forfeits the excess instead.
///
/// # Errors
///
/// - `InvalidTenantId` / `InvalidLegalEntityId` / `InvalidEmployeeId` — bad ID prefix or format.
/// - `InvalidRulepackEffectiveDate` — `period_boundary_date` or `rulepack_effective_date` not ISO-8601.
/// - `InvalidRulepackRef` — wrong prefix.
/// - `InvalidAuditEvidenceRef` — bad evidence-ref.
/// - `InvalidEvaluatedAt` — `evaluated_at_epoch_seconds` is zero.
/// - `InvalidAccrualUnits` — any f64 input is negative, NaN, or infinite.
/// - `CarryOverCapBelowFloor` — `carry_over_cap_units < statutory_min_floor_units`.
pub fn evaluate_leave_carryover_forfeiture(
    input: LeaveCarryoverForfeitureInput,
) -> Result<LeaveCarryoverForfeitureProjection, HrDomainError> {
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
        &input.employee_id,
        EMPLOYEE_ID_PREFIX,
        HrDomainError::InvalidEmployeeId,
    )?;
    validate_iso_date(&input.period_boundary_date)?;
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        HrDomainError::InvalidRulepackRef,
    )?;
    validate_iso_date(&input.rulepack_effective_date)?;
    validate_evidence_ref(&input.evidence_ref)?;
    if input.evaluated_at_epoch_seconds == 0 {
        return Err(HrDomainError::InvalidEvaluatedAt);
    }

    for val in [
        input.closing_balance_units,
        input.statutory_min_floor_units,
        input.carry_over_cap_units,
    ] {
        if !val.is_finite() || val < 0.0 {
            return Err(HrDomainError::InvalidAccrualUnits);
        }
    }

    if input.carry_over_cap_units < input.statutory_min_floor_units {
        return Err(HrDomainError::CarryOverCapBelowFloor);
    }

    let carried_over_units = input
        .closing_balance_units
        .clamp(input.statutory_min_floor_units, input.carry_over_cap_units);
    let forfeited_units = (input.closing_balance_units - input.carry_over_cap_units).max(0.0);

    let idempotency_key = format!(
        "{}:{}:{}:{}",
        input.tenant_id, input.employee_id, input.period_boundary_date, input.rulepack_ref
    );

    Ok(LeaveCarryoverForfeitureProjection {
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        employee_id: internal(EmployeeId {
            value: input.employee_id,
        }),
        period_boundary_date: internal(input.period_boundary_date),
        closing_balance_units: Classified::new(input.closing_balance_units, DataClass::Financial),
        statutory_min_floor_units: Classified::new(
            input.statutory_min_floor_units,
            DataClass::Financial,
        ),
        carry_over_cap_units: Classified::new(input.carry_over_cap_units, DataClass::Financial),
        carried_over_units: Classified::new(carried_over_units, DataClass::Financial),
        forfeited_units: Classified::new(forfeited_units, DataClass::Financial),
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        rulepack_effective_date: internal(RulepackEffectiveDate {
            value: input.rulepack_effective_date,
        }),
        evidence_ref: internal(AuditEvidenceRef {
            value: input.evidence_ref,
        }),
        idempotency_key: internal(idempotency_key),
        evaluated_at_epoch_seconds: internal(input.evaluated_at_epoch_seconds),
        schema_version: public(LEAVE_CARRYOVER_FORFEITURE_SCHEMA_VERSION),
    })
}

// ---------------------------------------------------------------------------
// Onboarding readiness domain model
// ---------------------------------------------------------------------------

/// The kinds of pre-hire onboarding checklist items.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum OnboardingChecklistItemKind {
    RightToWorkI9,
    BackgroundCheck,
    EquipmentProvisioning,
    AccessGrant,
    MandatoryTraining,
}

/// A single item on the onboarding checklist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardingChecklistItem {
    pub kind: OnboardingChecklistItemKind, // data_class: INTERNAL_ONLY
    pub is_mandatory: bool,                // data_class: INTERNAL_ONLY
    pub is_cleared: bool,                  // data_class: INTERNAL_ONLY
    pub evidence_ref: Option<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
}

/// Input to the onboarding readiness evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardingReadinessInput {
    pub employee_id: String,                     // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                 // data_class: INTERNAL_ONLY
    pub checklist: Vec<OnboardingChecklistItem>, // data_class: INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

/// The outcome of the onboarding readiness evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OnboardingDecision {
    Ready,
    NotReady,
}

/// Decision output from `evaluate_onboarding_readiness`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardingReadinessDecision {
    pub employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,     // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub decision: Classified<OnboardingDecision>, // data_class: INTERNAL_ONLY
    pub outstanding_items: Classified<Vec<OnboardingChecklistItemKind>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,                                 // data_class: PUBLIC
}

/// Pure evaluator: validates identifiers, rejects empty/duplicate checklists,
/// and determines whether all mandatory items are cleared with evidence.
///
/// Returns `Ok(OnboardingReadinessDecision)` on a valid, non-duplicate input.
/// Returns `Err(HrDomainError)` for invalid identifiers, empty checklist,
/// duplicate item kinds, zero `evaluated_at_epoch_seconds`, or a mandatory
/// item that is marked cleared but supplies no evidence ref.
pub fn evaluate_onboarding_readiness(
    input: OnboardingReadinessInput,
) -> Result<OnboardingReadinessDecision, HrDomainError> {
    validate_identifier(
        &input.employee_id,
        EMPLOYEE_ID_PREFIX,
        HrDomainError::InvalidEmployeeId,
    )?;
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
    if input.evaluated_at_epoch_seconds == 0 {
        return Err(HrDomainError::InvalidEvaluatedAt);
    }
    if input.checklist.is_empty() {
        return Err(HrDomainError::OnboardingItemsRequired);
    }

    // Reject duplicate item kinds.
    let mut seen = std::collections::HashSet::new();
    for item in &input.checklist {
        if !seen.insert(item.kind) {
            return Err(HrDomainError::DuplicateOnboardingItem);
        }
    }

    // Validate all evidence refs supplied on items (mandatory or optional).
    // Also enforce: a mandatory item marked is_cleared=true MUST supply an
    // evidence ref — cleared without evidence is an error, not a NOT_READY.
    for item in &input.checklist {
        if let Some(ref ev) = item.evidence_ref {
            validate_evidence_ref(&ev.value)?;
        }
        if item.is_mandatory && item.is_cleared && item.evidence_ref.is_none() {
            return Err(HrDomainError::OnboardingItemNotCleared);
        }
    }

    // Build a lookup of checklist items by kind for absent-kind detection.
    let item_map: std::collections::HashMap<OnboardingChecklistItemKind, &OnboardingChecklistItem> =
        input.checklist.iter().map(|i| (i.kind, i)).collect();

    // Canonical mandatory kinds per spec (right-to-work/I-9, background-check,
    // mandatory training are always required when at least one mandatory item
    // is present in the checklist).
    const CANONICAL_MANDATORY: [OnboardingChecklistItemKind; 3] = [
        OnboardingChecklistItemKind::RightToWorkI9,
        OnboardingChecklistItemKind::BackgroundCheck,
        OnboardingChecklistItemKind::MandatoryTraining,
    ];

    let has_any_mandatory = input.checklist.iter().any(|i| i.is_mandatory);

    let mut outstanding_set = std::collections::HashSet::new();

    // Items present and marked mandatory but not cleared+evidenced.
    for item in &input.checklist {
        if item.is_mandatory && !(item.is_cleared && item.evidence_ref.is_some()) {
            outstanding_set.insert(item.kind);
        }
    }

    // When the checklist has at least one mandatory item, canonical mandatory
    // kinds that are entirely absent from the checklist are also blockers.
    if has_any_mandatory {
        for kind in &CANONICAL_MANDATORY {
            if !item_map.contains_key(kind) {
                outstanding_set.insert(*kind);
            }
        }
    }

    let mut outstanding: Vec<OnboardingChecklistItemKind> = outstanding_set.into_iter().collect();
    outstanding.sort();

    let decision = if outstanding.is_empty() {
        OnboardingDecision::Ready
    } else {
        OnboardingDecision::NotReady
    };

    Ok(OnboardingReadinessDecision {
        employee_id: internal(EmployeeId {
            value: input.employee_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        decision: internal(decision),
        outstanding_items: internal(outstanding),
        schema_version: public(ONBOARDING_READINESS_SCHEMA_VERSION),
    })
}

impl Employee {
    pub fn new(input: EmployeeCreate) -> Result<Self, HrDomainError> {
        validate_identifier(
            &input.employee_id,
            EMPLOYEE_ID_PREFIX,
            HrDomainError::InvalidEmployeeId,
        )?;
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
        validate_ref(
            &input.person_ref,
            PERSON_REF_PREFIX,
            HrDomainError::InvalidPersonRef,
        )?;
        validate_evidence_ref(&input.audit_evidence_ref)?;
        if input.version == 0 {
            return Err(HrDomainError::InvalidVersion);
        }
        let manager_id = input
            .manager_id
            .map(|manager_id| employee_id(&manager_id))
            .transpose()?;
        let data_class = input
            .data_class
            .unwrap_or(PrivacyDataClass::pii_identifying());
        if data_class.data_class() != DataClass::PiiIdentifying {
            return Err(HrDomainError::InvalidDataClass);
        }
        Ok(Self {
            employee_id: internal(EmployeeId {
                value: input.employee_id,
            }),
            tenant_id: internal(TenantId {
                value: input.tenant_id,
            }),
            legal_entity_id: internal(LegalEntityId {
                value: input.legal_entity_id,
            }),
            person_ref: Classified::new(
                PersonRef {
                    value: input.person_ref,
                },
                PrivacyDataClass::pii_identifying(),
            ),
            manager_id: internal(manager_id),
            employment_status: internal(input.employment_status),
            tenant_tier_snapshot: internal(input.tenant_tier_snapshot),
            audit_evidence_ref: internal(AuditEvidenceRef {
                value: input.audit_evidence_ref,
            }),
            data_class: internal(data_class),
            version: internal(input.version),
            schema_version: public(EMPLOYEE_SCHEMA_VERSION),
        })
    }

    pub fn lifecycle_event(
        &self,
        event_id: &str,
        lifecycle_kind: HrLifecycleKind,
    ) -> Result<EmployeeLifecycleEvent, HrDomainError> {
        validate_identifier(
            event_id,
            HR_EVENT_ID_PREFIX,
            HrDomainError::InvalidHrEventId,
        )?;
        let idempotency_key = format!("{}:{}", self.employee_id.value.value, self.version.value);
        Ok(EmployeeLifecycleEvent {
            event_id: internal(HrEventId {
                value: event_id.to_owned(),
            }),
            tenant_id: internal(self.tenant_id.value.clone()),
            legal_entity_id: internal(self.legal_entity_id.value.clone()),
            employee_id: internal(self.employee_id.value.clone()),
            lifecycle_kind: internal(lifecycle_kind),
            audit_evidence_ref: internal(self.audit_evidence_ref.value.clone()),
            idempotency_key: internal(idempotency_key),
            schema_version: public(HR_EVENT_SCHEMA_VERSION),
        })
    }
}

pub fn evaluate_labor_compliance(
    snapshot: LegalEntityWorkforceSnapshot,
) -> Result<Vec<LaborComplianceObligation>, HrDomainError> {
    validate_snapshot(&snapshot)?;
    if snapshot.jurisdiction != Jurisdiction::Korea {
        return Ok(Vec::new());
    }

    let mut obligations = Vec::new();
    if snapshot.active_employee_count >= 10 {
        obligations.push(build_obligation(
            &snapshot,
            LaborComplianceObligationKind::KoreaRulesOfEmployment,
            10,
            vec![
                LaborComplianceWorkflowStep::Drafted,
                LaborComplianceWorkflowStep::EmployeeReviewSent,
                LaborComplianceWorkflowStep::MajorityConsentObtained,
                LaborComplianceWorkflowStep::MoelFiled,
                LaborComplianceWorkflowStep::Active,
            ],
            "moel/rules-of-employment/report",
        ));
    }
    if snapshot.active_employee_count >= 30 {
        obligations.push(build_obligation(
            &snapshot,
            LaborComplianceObligationKind::KoreaLaborManagementCouncil,
            30,
            vec![
                LaborComplianceWorkflowStep::CouncilRosterRequired,
                LaborComplianceWorkflowStep::MeetingCadenceRequired,
                LaborComplianceWorkflowStep::MinutesEvidenceRequired,
                LaborComplianceWorkflowStep::Active,
            ],
            "moel/labor-management-council/minutes",
        ));
    }
    Ok(obligations)
}

pub fn plan_leave_payroll_impact(
    input: LeavePayrollImpactInput,
) -> Result<LeavePayrollImpactPlan, HrDomainError> {
    validate_identifier(
        &input.leave_request_id,
        LEAVE_REQUEST_ID_PREFIX,
        HrDomainError::InvalidLeaveRequestId,
    )?;
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
        &input.employee_id,
        EMPLOYEE_ID_PREFIX,
        HrDomainError::InvalidEmployeeId,
    )?;
    validate_identifier(
        &input.approver_id,
        EMPLOYEE_ID_PREFIX,
        HrDomainError::InvalidApproverId,
    )?;
    validate_leave_dates(&input.start_date, &input.end_date)?;
    validate_payroll_period(&input.payroll_period)?;
    validate_ref(
        &input.workflow_ref,
        WORKFLOW_REF_PREFIX,
        HrDomainError::InvalidWorkflowRef,
    )?;
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        HrDomainError::InvalidRulepackRef,
    )?;
    validate_iso_date(&input.rulepack_effective_date)?;
    validate_evidence_ref(&input.decision_evidence_ref)?;
    validate_evidence_ref(&input.routing_evidence_ref)?;
    validate_evidence_ref(&input.payroll_impact_evidence_ref)?;
    if input.decided_at_epoch_seconds == 0 {
        return Err(HrDomainError::InvalidDecisionTimestamp);
    }

    let idempotency_key = format!(
        "{}:{}:{:?}:{}",
        input.tenant_id, input.leave_request_id, input.decision, input.payroll_period
    );

    Ok(LeavePayrollImpactPlan {
        leave_request_id: internal(LeaveRequestId {
            value: input.leave_request_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        employee_id: internal(EmployeeId {
            value: input.employee_id,
        }),
        approver_id: internal(EmployeeId {
            value: input.approver_id,
        }),
        decision: internal(input.decision),
        routing_mode: internal(input.routing_mode),
        start_date: internal(input.start_date),
        end_date: internal(input.end_date),
        payroll_period: Classified::new(input.payroll_period, DataClass::Financial),
        payroll_impact_kind: Classified::new(input.payroll_impact_kind, DataClass::Financial),
        workflow_ref: internal(WorkflowRef {
            value: input.workflow_ref,
        }),
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        rulepack_effective_date: internal(RulepackEffectiveDate {
            value: input.rulepack_effective_date,
        }),
        decision_evidence_ref: internal(AuditEvidenceRef {
            value: input.decision_evidence_ref,
        }),
        routing_evidence_ref: internal(AuditEvidenceRef {
            value: input.routing_evidence_ref,
        }),
        payroll_impact_evidence_ref: Classified::new(
            AuditEvidenceRef {
                value: input.payroll_impact_evidence_ref,
            },
            DataClass::Financial,
        ),
        idempotency_key: internal(idempotency_key),
        decided_at_epoch_seconds: internal(input.decided_at_epoch_seconds),
        schema_version: public(LEAVE_PAYROLL_IMPACT_SCHEMA_VERSION),
    })
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

pub fn evaluate_leave_balance_accrual(
    input: LeaveBalanceAccrualInput,
) -> Result<LeaveBalanceLedgerProjection, HrDomainError> {
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
        &input.employee_id,
        EMPLOYEE_ID_PREFIX,
        HrDomainError::InvalidEmployeeId,
    )?;
    validate_payroll_period(&input.payroll_period)?;
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        HrDomainError::InvalidRulepackRef,
    )?;
    validate_iso_date(&input.rulepack_effective_date)?;
    validate_evidence_ref(&input.accrual_evidence_ref)?;
    validate_evidence_ref(&input.deduction_evidence_ref)?;
    if input.decided_at_epoch_seconds == 0 {
        return Err(HrDomainError::InvalidDecisionTimestamp);
    }
    for val in [
        input.prior_accrued_units,
        input.accrual_units,
        input.deduction_units,
        input.carry_over_cap_units,
    ] {
        if !val.is_finite() || val < 0.0 {
            return Err(HrDomainError::InvalidAccrualUnits);
        }
    }

    let gross = input.prior_accrued_units + input.accrual_units;
    let after_deduction = gross - input.deduction_units;
    if after_deduction < 0.0 {
        return Err(HrDomainError::NegativeLeaveBalance);
    }
    if after_deduction > input.carry_over_cap_units {
        return Err(HrDomainError::CarryOverCapExceeded);
    }

    let resulting_balance_units = after_deduction;
    let carried_over_units = resulting_balance_units;
    let forfeited_units = 0.0_f64;

    let idempotency_key = format!(
        "{}:{}:{}:{}",
        input.tenant_id, input.employee_id, input.payroll_period, input.rulepack_ref
    );

    Ok(LeaveBalanceLedgerProjection {
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        employee_id: internal(EmployeeId {
            value: input.employee_id,
        }),
        payroll_period: Classified::new(input.payroll_period, DataClass::Financial),
        prior_accrued_units: Classified::new(input.prior_accrued_units, DataClass::Financial),
        accrual_units: Classified::new(input.accrual_units, DataClass::Financial),
        deduction_units: Classified::new(input.deduction_units, DataClass::Financial),
        resulting_balance_units: Classified::new(resulting_balance_units, DataClass::Financial),
        carried_over_units: Classified::new(carried_over_units, DataClass::Financial),
        forfeited_units: Classified::new(forfeited_units, DataClass::Financial),
        carry_over_cap_units: Classified::new(input.carry_over_cap_units, DataClass::Financial),
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        rulepack_effective_date: internal(RulepackEffectiveDate {
            value: input.rulepack_effective_date,
        }),
        accrual_evidence_ref: internal(AuditEvidenceRef {
            value: input.accrual_evidence_ref,
        }),
        deduction_evidence_ref: internal(AuditEvidenceRef {
            value: input.deduction_evidence_ref,
        }),
        idempotency_key: internal(idempotency_key),
        decided_at_epoch_seconds: internal(input.decided_at_epoch_seconds),
        schema_version: public(LEAVE_BALANCE_LEDGER_SCHEMA_VERSION),
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

fn build_obligation(
    snapshot: &LegalEntityWorkforceSnapshot,
    kind: LaborComplianceObligationKind,
    threshold_employee_count: u32,
    workflow_steps: Vec<LaborComplianceWorkflowStep>,
    evidence_suffix: &str,
) -> LaborComplianceObligation {
    let obligation_kind_key = obligation_kind_key(kind);
    let obligation_id = format!(
        "{LABOR_OBLIGATION_ID_PREFIX}{}_{}_{}",
        snapshot.legal_entity_id, obligation_kind_key, snapshot.rulepack_effective_date
    );
    let idempotency_key = format!(
        "{}:{}:{}:{}",
        snapshot.tenant_id,
        snapshot.legal_entity_id,
        obligation_kind_key,
        snapshot.rulepack_effective_date
    );
    let evidence_paths = vec![
        AuditEvidenceRef {
            value: snapshot.evidence_ref.clone(),
        },
        AuditEvidenceRef {
            value: format!("audit/{}/{evidence_suffix}", snapshot.legal_entity_id),
        },
    ];
    LaborComplianceObligation {
        obligation_id: internal(LaborComplianceObligationId {
            value: obligation_id,
        }),
        tenant_id: internal(TenantId {
            value: snapshot.tenant_id.clone(),
        }),
        legal_entity_id: internal(LegalEntityId {
            value: snapshot.legal_entity_id.clone(),
        }),
        jurisdiction: internal(snapshot.jurisdiction),
        kind: internal(kind),
        state: internal(LaborComplianceObligationState::Open),
        threshold_employee_count: internal(threshold_employee_count),
        active_employee_count: internal(snapshot.active_employee_count),
        rulepack_ref: internal(RulepackRef {
            value: snapshot.rulepack_ref.clone(),
        }),
        rulepack_effective_date: internal(RulepackEffectiveDate {
            value: snapshot.rulepack_effective_date.clone(),
        }),
        workflow_ref: internal(WorkflowRef {
            value: snapshot.workflow_ref.clone(),
        }),
        workflow_steps: internal(workflow_steps),
        evidence_paths: internal(evidence_paths),
        idempotency_key: internal(idempotency_key),
        evaluated_at_epoch_seconds: internal(snapshot.evaluated_at_epoch_seconds),
        schema_version: public(LABOR_OBLIGATION_SCHEMA_VERSION),
    }
}

fn validate_snapshot(snapshot: &LegalEntityWorkforceSnapshot) -> Result<(), HrDomainError> {
    validate_identifier(
        &snapshot.tenant_id,
        TENANT_ID_PREFIX,
        HrDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &snapshot.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        HrDomainError::InvalidLegalEntityId,
    )?;
    validate_ref(
        &snapshot.rulepack_ref,
        RULEPACK_REF_PREFIX,
        HrDomainError::InvalidRulepackRef,
    )?;
    validate_iso_date(&snapshot.rulepack_effective_date)?;
    validate_ref(
        &snapshot.workflow_ref,
        WORKFLOW_REF_PREFIX,
        HrDomainError::InvalidWorkflowRef,
    )?;
    validate_evidence_ref(&snapshot.evidence_ref)?;
    if snapshot.evaluated_at_epoch_seconds == 0 {
        return Err(HrDomainError::InvalidEvaluatedAt);
    }
    Ok(())
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

fn obligation_kind_key(kind: LaborComplianceObligationKind) -> &'static str {
    match kind {
        LaborComplianceObligationKind::KoreaRulesOfEmployment => "korea_rules_of_employment",
        LaborComplianceObligationKind::KoreaLaborManagementCouncil => {
            "korea_labor_management_council"
        }
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, PrivacyDataClass::internal_only())
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}
