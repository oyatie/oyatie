//! Payroll run API DTO contract layer.
//!
//! Serializable request shapes convert into payroll domain inputs while staying
//! transport-neutral. This crate does not calculate taxes, disburse funds,
//! persist payroll, submit filings, or call accounting/runtime services.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use payroll_run_domain::{
    HrLeaveImpactIntake, HrLeaveImpactIntakeInput, HrLeaveImpactKind, MoneyAmount, PayeeClass,
    PayeeInput, PayrollJournalInput, PayrollJournalLineInput, PayrollTrialCloseInput,
    WageLedgerEntryInput, WageLineKind,
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
pub struct PayrollTrialCloseRequest {
    pub run_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,         // data_class: INTERNAL_ONLY
    pub period: String,                  // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,            // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String, // data_class: INTERNAL_ONLY
    pub evidence_digest: String,         // data_class: FINANCIAL
    pub approval_evidence_ref: String,   // data_class: INTERNAL_ONLY
    pub payees: Vec<PayeeRequest>,       // data_class: PII_IDENTIFYING + FINANCIAL
}

impl PayrollTrialCloseRequest {
    pub fn into_domain(self) -> PayrollTrialCloseInput {
        PayrollTrialCloseInput {
            run_id: self.run_id,
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            period: self.period,
            rulepack_ref: self.rulepack_ref,
            rulepack_effective_date: self.rulepack_effective_date,
            evidence_digest: self.evidence_digest,
            approval_evidence_ref: self.approval_evidence_ref,
            payees: self
                .payees
                .into_iter()
                .map(PayeeRequest::into_domain)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeeRequest {
    pub payee_id: String,                         // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                  // data_class: INTERNAL_ONLY
    pub payee_class: PayeeClassDto,               // data_class: INTERNAL_ONLY
    pub person_or_vendor_ref: String,             // data_class: PII_IDENTIFYING
    pub tax_profile_ref: String,                  // data_class: INTERNAL_ONLY
    pub wage_ledger: Vec<WageLedgerEntryRequest>, // data_class: FINANCIAL
}

impl PayeeRequest {
    pub fn into_domain(self) -> PayeeInput {
        PayeeInput {
            payee_id: self.payee_id,
            legal_entity_id: self.legal_entity_id,
            payee_class: self.payee_class.into(),
            person_or_vendor_ref: self.person_or_vendor_ref,
            tax_profile_ref: self.tax_profile_ref,
            wage_ledger: self
                .wage_ledger
                .into_iter()
                .map(WageLedgerEntryRequest::into_domain)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WageLedgerEntryRequest {
    pub entry_id: String,           // data_class: INTERNAL_ONLY
    pub payee_id: String,           // data_class: INTERNAL_ONLY
    pub line_kind: WageLineKindDto, // data_class: INTERNAL_ONLY
    pub amount: MoneyAmountDto,     // data_class: FINANCIAL
    pub source_ref: String,         // data_class: INTERNAL_ONLY
}

impl WageLedgerEntryRequest {
    pub fn into_domain(self) -> WageLedgerEntryInput {
        WageLedgerEntryInput {
            entry_id: self.entry_id,
            payee_id: self.payee_id,
            line_kind: self.line_kind.into(),
            amount: self.amount.into(),
            source_ref: self.source_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyAmountDto {
    pub amount_minor: i64, // data_class: FINANCIAL
    pub currency: String,  // data_class: FINANCIAL
}

impl From<MoneyAmountDto> for MoneyAmount {
    fn from(value: MoneyAmountDto) -> Self {
        Self {
            amount_minor: value.amount_minor,
            currency: value.currency,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HrLeaveImpactIntakeRequest {
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,             // data_class: INTERNAL_ONLY
    pub payroll_period: String,              // data_class: FINANCIAL
    pub payee_id: String,                    // data_class: INTERNAL_ONLY
    pub employee_id: String,                 // data_class: INTERNAL_ONLY
    pub leave_request_id: String,            // data_class: INTERNAL_ONLY
    pub impact_kind: HrLeaveImpactKindDto,   // data_class: FINANCIAL
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

impl HrLeaveImpactIntakeRequest {
    pub fn into_domain(self) -> HrLeaveImpactIntakeInput {
        HrLeaveImpactIntakeInput {
            run_id: self.run_id,
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            payroll_period: self.payroll_period,
            payee_id: self.payee_id,
            employee_id: self.employee_id,
            leave_request_id: self.leave_request_id,
            impact_kind: self.impact_kind.into(),
            source_topic: self.source_topic,
            source_hr_idempotency_key: self.source_hr_idempotency_key,
            decision_evidence_ref: self.decision_evidence_ref,
            routing_evidence_ref: self.routing_evidence_ref,
            payroll_impact_evidence_ref: self.payroll_impact_evidence_ref,
            payroll_intake_evidence_ref: self.payroll_intake_evidence_ref,
            rulepack_ref: self.rulepack_ref,
            rulepack_effective_date: self.rulepack_effective_date,
            received_at_epoch_seconds: self.received_at_epoch_seconds,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HrLeaveImpactIntakeResponse {
    pub integration_topic: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub payroll_period: String,            // data_class: FINANCIAL
    pub impact_kind: HrLeaveImpactKindDto, // data_class: FINANCIAL
    pub payload_data_class: String,        // data_class: INTERNAL_ONLY
    pub schema_version: u32,               // data_class: PUBLIC
}

impl HrLeaveImpactIntakeResponse {
    pub fn from_intake(intake: &HrLeaveImpactIntake) -> Self {
        Self {
            integration_topic: "integration.payroll.hr.leave-impact-intake".to_owned(),
            idempotency_key: intake.idempotency_key.value.clone(),
            payroll_period: intake.payroll_period.value.clone(),
            impact_kind: intake.impact_kind.value.into(),
            payload_data_class: "FINANCIAL".to_owned(),
            schema_version: intake.schema_version.value,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollJournalDraftRequest {
    pub journal_id: String,                    // data_class: INTERNAL_ONLY
    pub run_id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,               // data_class: INTERNAL_ONLY
    pub period: String,                        // data_class: INTERNAL_ONLY
    pub source_payroll_digest: String,         // data_class: FINANCIAL
    pub approval_evidence_ref: String,         // data_class: INTERNAL_ONLY
    pub lines: Vec<PayrollJournalLineRequest>, // data_class: FINANCIAL
}

impl PayrollJournalDraftRequest {
    pub fn into_domain(self) -> PayrollJournalInput {
        PayrollJournalInput {
            journal_id: self.journal_id,
            run_id: self.run_id,
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            period: self.period,
            source_payroll_digest: self.source_payroll_digest,
            approval_evidence_ref: self.approval_evidence_ref,
            lines: self
                .lines
                .into_iter()
                .map(PayrollJournalLineRequest::into_domain)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollJournalLineRequest {
    pub account_code: String, // data_class: INTERNAL_ONLY
    pub debit_minor: i64,     // data_class: FINANCIAL
    pub credit_minor: i64,    // data_class: FINANCIAL
}

impl PayrollJournalLineRequest {
    pub fn into_domain(self) -> PayrollJournalLineInput {
        PayrollJournalLineInput {
            account_code: self.account_code,
            debit_minor: self.debit_minor,
            credit_minor: self.credit_minor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HrLeaveImpactKindDto {
    PaidLeave,
    UnpaidLeaveDeduction,
    AttendanceCorrection,
    NoPayrollImpact,
}

impl From<HrLeaveImpactKindDto> for HrLeaveImpactKind {
    fn from(value: HrLeaveImpactKindDto) -> Self {
        match value {
            HrLeaveImpactKindDto::PaidLeave => Self::PaidLeave,
            HrLeaveImpactKindDto::UnpaidLeaveDeduction => Self::UnpaidLeaveDeduction,
            HrLeaveImpactKindDto::AttendanceCorrection => Self::AttendanceCorrection,
            HrLeaveImpactKindDto::NoPayrollImpact => Self::NoPayrollImpact,
        }
    }
}

impl From<HrLeaveImpactKind> for HrLeaveImpactKindDto {
    fn from(value: HrLeaveImpactKind) -> Self {
        match value {
            HrLeaveImpactKind::PaidLeave => Self::PaidLeave,
            HrLeaveImpactKind::UnpaidLeaveDeduction => Self::UnpaidLeaveDeduction,
            HrLeaveImpactKind::AttendanceCorrection => Self::AttendanceCorrection,
            HrLeaveImpactKind::NoPayrollImpact => Self::NoPayrollImpact,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayeeClassDto {
    Employee,
    Contractor,
    StatutoryDirector,
    Vendor,
}

impl From<PayeeClassDto> for PayeeClass {
    fn from(value: PayeeClassDto) -> Self {
        match value {
            PayeeClassDto::Employee => Self::Employee,
            PayeeClassDto::Contractor => Self::Contractor,
            PayeeClassDto::StatutoryDirector => Self::StatutoryDirector,
            PayeeClassDto::Vendor => Self::Vendor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WageLineKindDto {
    GrossEarnings,
    TaxWithholding,
    SocialInsurance,
    EmployerContribution,
    NetPay,
    Reversal,
}

impl From<WageLineKindDto> for WageLineKind {
    fn from(value: WageLineKindDto) -> Self {
        match value {
            WageLineKindDto::GrossEarnings => Self::GrossEarnings,
            WageLineKindDto::TaxWithholding => Self::TaxWithholding,
            WageLineKindDto::SocialInsurance => Self::SocialInsurance,
            WageLineKindDto::EmployerContribution => Self::EmployerContribution,
            WageLineKindDto::NetPay => Self::NetPay,
            WageLineKindDto::Reversal => Self::Reversal,
        }
    }
}
