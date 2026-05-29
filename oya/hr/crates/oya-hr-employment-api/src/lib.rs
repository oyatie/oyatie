//! HR employment API DTO contract layer.
//!
//! This crate owns serializable boundary shapes and deterministic conversion
//! into the HR employment app layer. It is transport-neutral: no router,
//! network listener, persistence, Workflow client, or audit emitter lives here.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_hr_employment_app::{
    LeavePayrollImpactOutcome, OnboardEmployeeCommand, SensitiveHrReadOutcome,
};
use oya_hr_employment_domain::{
    EmployeeCreate, EmploymentStatus, HrLifecycleKind, Jurisdiction, LeaveDecision,
    LeavePayrollImpactInput, LeaveRoutingMode, LegalEntityWorkforceSnapshot, PayrollImpactKind,
    SensitiveHrDataKind, SensitiveHrReadInput, SensitiveReadDecisionStatus,
    SensitiveReadLegalBasis, SensitiveReadPurpose, TenantTierSnapshot,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorBody {
    pub code: String,            // data_class: INTERNAL_ONLY
    pub message: String,         // data_class: INTERNAL_ONLY
    pub details: Option<String>, // data_class: INTERNAL_ONLY
}

impl ApiErrorEnvelope {
    pub fn validation(message: impl Into<String>, details: Option<String>) -> Self {
        Self {
            error: ApiErrorBody {
                code: "VALIDATION_ERROR".to_owned(),
                message: message.into(),
                details,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardEmployeeRequest {
    pub employee_id: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                     // data_class: INTERNAL_ONLY
    pub person_ref: String,                          // data_class: PII_IDENTIFYING
    pub manager_id: Option<String>,                  // data_class: INTERNAL_ONLY
    pub employment_status: EmploymentStatusDto,      // data_class: INTERNAL_ONLY
    pub tenant_tier_snapshot: TenantTierSnapshotDto, // data_class: INTERNAL_ONLY
    pub audit_evidence_ref: String,                  // data_class: INTERNAL_ONLY
    pub version: u32,                                // data_class: INTERNAL_ONLY
    pub event_id: String,                            // data_class: INTERNAL_ONLY
    pub lifecycle_kind: HrLifecycleKindDto,          // data_class: INTERNAL_ONLY
}

impl OnboardEmployeeRequest {
    pub fn into_command(self) -> OnboardEmployeeCommand {
        OnboardEmployeeCommand {
            employee: EmployeeCreate {
                employee_id: self.employee_id,
                tenant_id: self.tenant_id,
                legal_entity_id: self.legal_entity_id,
                person_ref: self.person_ref,
                manager_id: self.manager_id,
                employment_status: self.employment_status.into(),
                tenant_tier_snapshot: self.tenant_tier_snapshot.into(),
                audit_evidence_ref: self.audit_evidence_ref,
                data_class: None,
                version: self.version,
            },
            event_id: self.event_id,
            lifecycle_kind: self.lifecycle_kind.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaborCompliancePlanRequest {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,         // data_class: INTERNAL_ONLY
    pub jurisdiction: JurisdictionDto,   // data_class: INTERNAL_ONLY
    pub active_employee_count: u32,      // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,            // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String, // data_class: INTERNAL_ONLY
    pub workflow_ref: String,            // data_class: INTERNAL_ONLY
    pub evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

impl LaborCompliancePlanRequest {
    pub fn into_snapshot(self) -> LegalEntityWorkforceSnapshot {
        LegalEntityWorkforceSnapshot {
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            jurisdiction: self.jurisdiction.into(),
            active_employee_count: self.active_employee_count,
            rulepack_ref: self.rulepack_ref,
            rulepack_effective_date: self.rulepack_effective_date,
            workflow_ref: self.workflow_ref,
            evidence_ref: self.evidence_ref,
            evaluated_at_epoch_seconds: self.evaluated_at_epoch_seconds,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeavePayrollImpactRequest {
    pub leave_request_id: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub employee_id: String,               // data_class: INTERNAL_ONLY
    pub approver_id: String,               // data_class: INTERNAL_ONLY
    pub decision: LeaveDecisionDto,        // data_class: INTERNAL_ONLY
    pub routing_mode: LeaveRoutingModeDto, // data_class: INTERNAL_ONLY
    pub start_date: String,                // data_class: INTERNAL_ONLY
    pub end_date: String,                  // data_class: INTERNAL_ONLY
    pub payroll_period: String,            // data_class: FINANCIAL
    pub payroll_impact_kind: PayrollImpactKindDto, // data_class: FINANCIAL
    pub workflow_ref: String,              // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,              // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String,   // data_class: INTERNAL_ONLY
    pub decision_evidence_ref: String,     // data_class: INTERNAL_ONLY
    pub routing_evidence_ref: String,      // data_class: INTERNAL_ONLY
    pub payroll_impact_evidence_ref: String, // data_class: FINANCIAL
    pub decided_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

impl LeavePayrollImpactRequest {
    pub fn into_domain_input(self) -> LeavePayrollImpactInput {
        LeavePayrollImpactInput {
            leave_request_id: self.leave_request_id,
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            employee_id: self.employee_id,
            approver_id: self.approver_id,
            decision: self.decision.into(),
            routing_mode: self.routing_mode.into(),
            start_date: self.start_date,
            end_date: self.end_date,
            payroll_period: self.payroll_period,
            payroll_impact_kind: self.payroll_impact_kind.into(),
            workflow_ref: self.workflow_ref,
            rulepack_ref: self.rulepack_ref,
            rulepack_effective_date: self.rulepack_effective_date,
            decision_evidence_ref: self.decision_evidence_ref,
            routing_evidence_ref: self.routing_evidence_ref,
            payroll_impact_evidence_ref: self.payroll_impact_evidence_ref,
            decided_at_epoch_seconds: self.decided_at_epoch_seconds,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeavePayrollImpactResponse {
    pub integration_topic: String, // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub payroll_period: String,    // data_class: FINANCIAL
    pub payroll_impact_kind: PayrollImpactKindDto, // data_class: FINANCIAL
    pub payload_data_class: String, // data_class: INTERNAL_ONLY
    pub schema_version: u32,       // data_class: PUBLIC
}

impl LeavePayrollImpactResponse {
    pub fn from_outcome(outcome: &LeavePayrollImpactOutcome) -> Self {
        Self {
            integration_topic: outcome.payroll_impact_envelope.topic.value.clone(),
            idempotency_key: outcome
                .payroll_impact_envelope
                .idempotency_key
                .value
                .clone(),
            payroll_period: outcome.payroll_impact_envelope.payroll_period.value.clone(),
            payroll_impact_kind: outcome
                .payroll_impact_envelope
                .payroll_impact_kind
                .value
                .into(),
            payload_data_class: outcome
                .payroll_impact_envelope
                .payload_data_class
                .value
                .label()
                .to_owned(),
            schema_version: outcome.payroll_impact_envelope.schema_version.value,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveHrReadPolicyRequest {
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                 // data_class: INTERNAL_ONLY
    pub actor_employee_id: String,               // data_class: INTERNAL_ONLY
    pub subject_employee_id: String,             // data_class: INTERNAL_ONLY
    pub data_kind: SensitiveHrDataKindDto,       // data_class: SENSITIVE_PIPA_ART23
    pub purpose: SensitiveReadPurposeDto,        // data_class: INTERNAL_ONLY
    pub legal_basis: SensitiveReadLegalBasisDto, // data_class: INTERNAL_ONLY
    pub policy_ref: String,                      // data_class: INTERNAL_ONLY
    pub basis_evidence_ref: String,              // data_class: INTERNAL_ONLY
    pub consent_evidence_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub read_log_evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub evaluated_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

impl SensitiveHrReadPolicyRequest {
    pub fn into_domain_input(self) -> SensitiveHrReadInput {
        SensitiveHrReadInput {
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            actor_employee_id: self.actor_employee_id,
            subject_employee_id: self.subject_employee_id,
            data_kind: self.data_kind.into(),
            purpose: self.purpose.into(),
            legal_basis: self.legal_basis.into(),
            policy_ref: self.policy_ref,
            basis_evidence_ref: self.basis_evidence_ref,
            consent_evidence_ref: self.consent_evidence_ref,
            request_evidence_ref: self.request_evidence_ref,
            read_log_evidence_ref: self.read_log_evidence_ref,
            evaluated_at_epoch_seconds: self.evaluated_at_epoch_seconds,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveReadPolicyDecisionResponse {
    pub decision_status: SensitiveReadDecisionStatusDto, // data_class: INTERNAL_ONLY
    pub audit_topic: String,                             // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                         // data_class: INTERNAL_ONLY
    pub payload_data_class: String,                      // data_class: INTERNAL_ONLY
    pub schema_version: u32,                             // data_class: PUBLIC
}

impl SensitiveReadPolicyDecisionResponse {
    pub fn from_outcome(outcome: &SensitiveHrReadOutcome) -> Self {
        Self {
            decision_status: outcome.audit_envelope.decision_status.value.into(),
            audit_topic: outcome.audit_envelope.topic.value.clone(),
            idempotency_key: outcome.audit_envelope.idempotency_key.value.clone(),
            payload_data_class: outcome
                .audit_envelope
                .payload_data_class
                .value
                .label()
                .to_owned(),
            schema_version: outcome.audit_envelope.schema_version.value,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmploymentStatusDto {
    Draft,
    Active,
    Suspended,
    Terminated,
}

impl From<EmploymentStatusDto> for EmploymentStatus {
    fn from(value: EmploymentStatusDto) -> Self {
        match value {
            EmploymentStatusDto::Draft => Self::Draft,
            EmploymentStatusDto::Active => Self::Active,
            EmploymentStatusDto::Suspended => Self::Suspended,
            EmploymentStatusDto::Terminated => Self::Terminated,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantTierSnapshotDto {
    SmbSelfServe,
    EnterpriseSingleEntity,
    EnterpriseGroup,
    RegulatedEnterprise,
}

impl From<TenantTierSnapshotDto> for TenantTierSnapshot {
    fn from(value: TenantTierSnapshotDto) -> Self {
        match value {
            TenantTierSnapshotDto::SmbSelfServe => Self::SmbSelfServe,
            TenantTierSnapshotDto::EnterpriseSingleEntity => Self::EnterpriseSingleEntity,
            TenantTierSnapshotDto::EnterpriseGroup => Self::EnterpriseGroup,
            TenantTierSnapshotDto::RegulatedEnterprise => Self::RegulatedEnterprise,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HrLifecycleKindDto {
    Created,
    Updated,
    Suspended,
    Terminated,
}

impl From<HrLifecycleKindDto> for HrLifecycleKind {
    fn from(value: HrLifecycleKindDto) -> Self {
        match value {
            HrLifecycleKindDto::Created => Self::Created,
            HrLifecycleKindDto::Updated => Self::Updated,
            HrLifecycleKindDto::Suspended => Self::Suspended,
            HrLifecycleKindDto::Terminated => Self::Terminated,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JurisdictionDto {
    Korea,
    UnitedStates,
    EuropeanUnion,
}

impl From<JurisdictionDto> for Jurisdiction {
    fn from(value: JurisdictionDto) -> Self {
        match value {
            JurisdictionDto::Korea => Self::Korea,
            JurisdictionDto::UnitedStates => Self::UnitedStates,
            JurisdictionDto::EuropeanUnion => Self::EuropeanUnion,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaveDecisionDto {
    Approved,
    Rejected,
}

impl From<LeaveDecisionDto> for LeaveDecision {
    fn from(value: LeaveDecisionDto) -> Self {
        match value {
            LeaveDecisionDto::Approved => Self::Approved,
            LeaveDecisionDto::Rejected => Self::Rejected,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaveRoutingModeDto {
    DirectManager,
    DelegatedApprover,
    EscalatedHr,
}

impl From<LeaveRoutingModeDto> for LeaveRoutingMode {
    fn from(value: LeaveRoutingModeDto) -> Self {
        match value {
            LeaveRoutingModeDto::DirectManager => Self::DirectManager,
            LeaveRoutingModeDto::DelegatedApprover => Self::DelegatedApprover,
            LeaveRoutingModeDto::EscalatedHr => Self::EscalatedHr,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayrollImpactKindDto {
    PaidLeave,
    UnpaidLeaveDeduction,
    AttendanceCorrection,
    NoPayrollImpact,
}

impl From<PayrollImpactKindDto> for PayrollImpactKind {
    fn from(value: PayrollImpactKindDto) -> Self {
        match value {
            PayrollImpactKindDto::PaidLeave => Self::PaidLeave,
            PayrollImpactKindDto::UnpaidLeaveDeduction => Self::UnpaidLeaveDeduction,
            PayrollImpactKindDto::AttendanceCorrection => Self::AttendanceCorrection,
            PayrollImpactKindDto::NoPayrollImpact => Self::NoPayrollImpact,
        }
    }
}

impl From<PayrollImpactKind> for PayrollImpactKindDto {
    fn from(value: PayrollImpactKind) -> Self {
        match value {
            PayrollImpactKind::PaidLeave => Self::PaidLeave,
            PayrollImpactKind::UnpaidLeaveDeduction => Self::UnpaidLeaveDeduction,
            PayrollImpactKind::AttendanceCorrection => Self::AttendanceCorrection,
            PayrollImpactKind::NoPayrollImpact => Self::NoPayrollImpact,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SensitiveHrDataKindDto {
    Medical,
    DisabilityAccommodation,
    Disciplinary,
    Compensation,
    GovernmentIdentifier,
}

impl From<SensitiveHrDataKindDto> for SensitiveHrDataKind {
    fn from(value: SensitiveHrDataKindDto) -> Self {
        match value {
            SensitiveHrDataKindDto::Medical => Self::Medical,
            SensitiveHrDataKindDto::DisabilityAccommodation => Self::DisabilityAccommodation,
            SensitiveHrDataKindDto::Disciplinary => Self::Disciplinary,
            SensitiveHrDataKindDto::Compensation => Self::Compensation,
            SensitiveHrDataKindDto::GovernmentIdentifier => Self::GovernmentIdentifier,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SensitiveReadPurposeDto {
    BenefitsAdministration,
    AccommodationReview,
    PayrollAudit,
    LegalCompliance,
    GeneralBrowsing,
}

impl From<SensitiveReadPurposeDto> for SensitiveReadPurpose {
    fn from(value: SensitiveReadPurposeDto) -> Self {
        match value {
            SensitiveReadPurposeDto::BenefitsAdministration => Self::BenefitsAdministration,
            SensitiveReadPurposeDto::AccommodationReview => Self::AccommodationReview,
            SensitiveReadPurposeDto::PayrollAudit => Self::PayrollAudit,
            SensitiveReadPurposeDto::LegalCompliance => Self::LegalCompliance,
            SensitiveReadPurposeDto::GeneralBrowsing => Self::GeneralBrowsing,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SensitiveReadLegalBasisDto {
    Consent,
    EmploymentLawObligation,
    LegalClaim,
    None,
}

impl From<SensitiveReadLegalBasisDto> for SensitiveReadLegalBasis {
    fn from(value: SensitiveReadLegalBasisDto) -> Self {
        match value {
            SensitiveReadLegalBasisDto::Consent => Self::Consent,
            SensitiveReadLegalBasisDto::EmploymentLawObligation => Self::EmploymentLawObligation,
            SensitiveReadLegalBasisDto::LegalClaim => Self::LegalClaim,
            SensitiveReadLegalBasisDto::None => Self::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SensitiveReadDecisionStatusDto {
    Allowed,
}

impl From<SensitiveReadDecisionStatus> for SensitiveReadDecisionStatusDto {
    fn from(value: SensitiveReadDecisionStatus) -> Self {
        match value {
            SensitiveReadDecisionStatus::Allowed => Self::Allowed,
        }
    }
}
