//! Accounting journal API DTO contract layer.
//!
//! Serializable request shapes convert into accounting domain inputs while
//! staying transport-neutral. This crate does not persist ledgers, file tax
//! returns, execute payments, dispatch Workflow, or emit audit records.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use billing_accounting_journal::{
    JournalLineInput, JournalPostInput, Jurisdiction, PayrollPostingInput, PeriodState,
    VatDeadlineInput,
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
pub struct JournalPostRequest {
    pub journal_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,        // data_class: INTERNAL_ONLY
    pub period: String,                 // data_class: INTERNAL_ONLY
    pub period_state: PeriodStateDto,   // data_class: INTERNAL_ONLY
    pub source_documents: Vec<String>,  // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub lines: Vec<JournalLineRequest>, // data_class: FINANCIAL
}

impl JournalPostRequest {
    pub fn into_domain(self) -> JournalPostInput {
        JournalPostInput {
            journal_id: self.journal_id,
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            period: self.period,
            period_state: self.period_state.into(),
            source_documents: self.source_documents,
            approval_evidence_ref: self.approval_evidence_ref,
            lines: self
                .lines
                .into_iter()
                .map(JournalLineRequest::into_domain)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalLineRequest {
    pub account_code: String, // data_class: INTERNAL_ONLY
    pub debit_minor: i64,     // data_class: FINANCIAL
    pub credit_minor: i64,    // data_class: FINANCIAL
}

impl JournalLineRequest {
    pub fn into_domain(self) -> JournalLineInput {
        JournalLineInput {
            account_code: self.account_code,
            debit_minor: self.debit_minor,
            credit_minor: self.credit_minor,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollPostingRequest {
    pub journal_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,        // data_class: INTERNAL_ONLY
    pub period: String,                 // data_class: INTERNAL_ONLY
    pub source_payroll_digest: String,  // data_class: FINANCIAL
    pub wage_ledger_refs: Vec<String>,  // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub reversal_path_ref: String,      // data_class: INTERNAL_ONLY
    pub lines: Vec<JournalLineRequest>, // data_class: FINANCIAL
}

impl PayrollPostingRequest {
    pub fn into_domain(self) -> PayrollPostingInput {
        PayrollPostingInput {
            journal_id: self.journal_id,
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            period: self.period,
            source_payroll_digest: self.source_payroll_digest,
            wage_ledger_refs: self.wage_ledger_refs,
            approval_evidence_ref: self.approval_evidence_ref,
            reversal_path_ref: self.reversal_path_ref,
            lines: self
                .lines
                .into_iter()
                .map(JournalLineRequest::into_domain)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VatDeadlineRequest {
    pub return_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub jurisdiction: JurisdictionDto, // data_class: INTERNAL_ONLY
    pub period: String,                // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub workflow_ref: String,          // data_class: INTERNAL_ONLY
    pub hometax_export_hash: String,   // data_class: FINANCIAL
    pub evidence_ref: String,          // data_class: INTERNAL_ONLY
}

impl VatDeadlineRequest {
    pub fn into_domain(self) -> VatDeadlineInput {
        VatDeadlineInput {
            return_id: self.return_id,
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            jurisdiction: self.jurisdiction.into(),
            period: self.period,
            deadline_epoch_seconds: self.deadline_epoch_seconds,
            now_epoch_seconds: self.now_epoch_seconds,
            workflow_ref: self.workflow_ref,
            hometax_export_hash: self.hometax_export_hash,
            evidence_ref: self.evidence_ref,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PeriodStateDto {
    Open,
    SoftClosed,
    Closed,
}

impl From<PeriodStateDto> for PeriodState {
    fn from(value: PeriodStateDto) -> Self {
        match value {
            PeriodStateDto::Open => Self::Open,
            PeriodStateDto::SoftClosed => Self::SoftClosed,
            PeriodStateDto::Closed => Self::Closed,
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
