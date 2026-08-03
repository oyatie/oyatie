//! Payroll run API DTO contract layer.
//!
//! Serializable request shapes convert into payroll domain inputs while staying
//! transport-neutral. This crate does not calculate taxes, disburse funds,
//! persist payroll, submit filings, or call accounting/runtime services.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_payroll_run_domain::{
    CalculationBoundary, HrLeaveImpactIntake, HrLeaveImpactIntakeInput, HrLeaveImpactKind,
    MoneyAmount, PayeeClass, PayeeInput, PayrollJournalInput, PayrollJournalLineInput,
    PayrollRulepackJurisdiction, PayrollTrialCloseInput, PreparedYearEndSettlementInput,
    StatutoryCalculationDraft, StatutoryCalculationInput, StatutoryDeductionKind,
    StatutoryRateLineInput, WageLedgerEntryInput, WageLineKind, YearEndEvidenceRefInput,
    YearEndRegionalDependency, YearEndSettlementInput, YearEndSettlementSourceKind,
};
use serde::{Deserialize, Serialize};

pub const STATUTORY_PREVIEW_FIXTURE_NOTE: &str =
    "synthetic/non-authoritative fixture: no official KR/US/EU rate correctness claim";
pub const YEAR_END_PREVIEW_FIXTURE_NOTE: &str =
    "synthetic/non-authoritative fixture: no production year-end settlement or filing claim";

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

impl From<&MoneyAmount> for MoneyAmountDto {
    fn from(value: &MoneyAmount) -> Self {
        Self {
            amount_minor: value.amount_minor,
            currency: value.currency.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatutoryCalculationPreviewRequest {
    pub run_id: String,                               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                      // data_class: INTERNAL_ONLY
    pub payee_id: String,                             // data_class: INTERNAL_ONLY
    pub payroll_period: String,                       // data_class: FINANCIAL
    pub jurisdiction: PayrollRulepackJurisdictionDto, // data_class: INTERNAL_ONLY
    pub required_regional_pack: String,               // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                         // data_class: INTERNAL_ONLY
    pub rulepack_manifest_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub rulepack_source_version: Option<String>,      // data_class: INTERNAL_ONLY
    pub official_source_evidence_refs: Vec<String>,   // data_class: INTERNAL_ONLY
    pub unofficial_source_fixture: bool,              // data_class: INTERNAL_ONLY
    pub gross_pay: MoneyAmountDto,                    // data_class: FINANCIAL
    pub rate_lines: Vec<StatutoryRateLineRequest>,    // data_class: FINANCIAL
    pub fixture_note: String,                         // data_class: PUBLIC
    pub filing_rail_requested: bool,                  // data_class: PUBLIC
    pub disbursement_rail_requested: bool,            // data_class: PUBLIC
    pub production_close_requested: bool,             // data_class: PUBLIC
    pub cloud_deployment_requested: bool,             // data_class: PUBLIC
}

impl StatutoryCalculationPreviewRequest {
    pub fn into_domain(self) -> StatutoryCalculationInput {
        let fixture_note = if self.fixture_note == STATUTORY_PREVIEW_FIXTURE_NOTE {
            STATUTORY_PREVIEW_FIXTURE_NOTE
        } else {
            "invalid statutory calculation preview fixture note"
        };
        StatutoryCalculationInput {
            run_id: self.run_id,
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            payee_id: self.payee_id,
            payroll_period: self.payroll_period,
            jurisdiction: self.jurisdiction.into(),
            required_regional_pack: self.required_regional_pack,
            rulepack_ref: self.rulepack_ref,
            rulepack_manifest_ref: self.rulepack_manifest_ref,
            rulepack_source_version: self.rulepack_source_version,
            official_source_evidence_refs: self.official_source_evidence_refs,
            unofficial_source_fixture: self.unofficial_source_fixture,
            gross_pay_minor: self.gross_pay.amount_minor,
            currency: self.gross_pay.currency,
            rate_lines: self
                .rate_lines
                .into_iter()
                .map(StatutoryRateLineRequest::into_domain)
                .collect(),
            fixture_note,
            filing_rail_requested: self.filing_rail_requested,
            disbursement_rail_requested: self.disbursement_rail_requested,
            production_close_requested: self.production_close_requested,
            cloud_deployment_requested: self.cloud_deployment_requested,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatutoryRateLineRequest {
    pub kind: StatutoryDeductionKindDto,  // data_class: INTERNAL_ONLY
    pub synthetic_rate_basis_points: u32, // data_class: FINANCIAL
    pub source_evidence_ref: String,      // data_class: INTERNAL_ONLY
}

impl StatutoryRateLineRequest {
    pub fn into_domain(self) -> StatutoryRateLineInput {
        StatutoryRateLineInput {
            kind: self.kind.into(),
            synthetic_rate_basis_points: self.synthetic_rate_basis_points,
            source_evidence_ref: self.source_evidence_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatutoryCalculationPreviewResponse {
    pub run_id: String,                                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                         // data_class: INTERNAL_ONLY
    pub payee_id: String,                                // data_class: INTERNAL_ONLY
    pub payroll_period: String,                          // data_class: FINANCIAL
    pub jurisdiction: PayrollRulepackJurisdictionDto,    // data_class: INTERNAL_ONLY
    pub required_regional_pack: String,                  // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                            // data_class: INTERNAL_ONLY
    pub rulepack_manifest_ref: String,                   // data_class: INTERNAL_ONLY
    pub rulepack_source_version: String,                 // data_class: INTERNAL_ONLY
    pub official_source_evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
    pub fixture_note: String,                            // data_class: PUBLIC
    pub gross_pay: MoneyAmountDto,                       // data_class: FINANCIAL
    pub deductions: Vec<StatutoryDeductionLineResponse>, // data_class: FINANCIAL
    pub net_pay: MoneyAmountDto,                         // data_class: FINANCIAL
    pub boundary: CalculationBoundaryDto,                // data_class: PUBLIC
    pub direct_agency_submission_attached: bool,         // data_class: PUBLIC
    pub filing_rail_attached: bool,                      // data_class: PUBLIC
    pub disbursement_rail_attached: bool,                // data_class: PUBLIC
    pub production_close_attached: bool,                 // data_class: PUBLIC
    pub cloud_deployment_attached: bool,                 // data_class: PUBLIC
    pub payload_data_class: String,                      // data_class: INTERNAL_ONLY
    pub schema_version: u32,                             // data_class: PUBLIC
}

impl StatutoryCalculationPreviewResponse {
    pub fn from_draft(draft: &StatutoryCalculationDraft) -> Self {
        Self {
            run_id: draft.run_id.value.clone(),
            tenant_id: draft.tenant_id.value.clone(),
            legal_entity_id: draft.legal_entity_id.value.clone(),
            payee_id: draft.payee_id.value.clone(),
            payroll_period: draft.payroll_period.value.clone(),
            jurisdiction: draft.jurisdiction.value.into(),
            required_regional_pack: draft.required_regional_pack.value.clone(),
            rulepack_ref: draft.rulepack_ref.value.clone(),
            rulepack_manifest_ref: draft.rulepack_manifest_ref.value.clone(),
            rulepack_source_version: draft.rulepack_source_version.value.clone(),
            official_source_evidence_refs: draft
                .official_source_evidence_refs
                .value
                .iter()
                .map(|evidence| evidence.value.clone())
                .collect(),
            fixture_note: draft.fixture_note.value.clone(),
            gross_pay: (&draft.gross_pay.value).into(),
            deductions: draft
                .deductions
                .value
                .iter()
                .map(|line| StatutoryDeductionLineResponse {
                    kind: line.kind.value.into(),
                    amount: (&line.amount.value).into(),
                    synthetic_rate_basis_points: line.synthetic_rate_basis_points.value,
                    source_evidence_ref: line.source_evidence_ref.value.value.clone(),
                })
                .collect(),
            net_pay: (&draft.net_pay.value).into(),
            boundary: draft.boundary.value.into(),
            direct_agency_submission_attached: draft.direct_agency_submission_attached.value,
            filing_rail_attached: draft.filing_rail_attached.value,
            disbursement_rail_attached: draft.disbursement_rail_attached.value,
            production_close_attached: draft.production_close_attached.value,
            cloud_deployment_attached: draft.cloud_deployment_attached.value,
            payload_data_class: "FINANCIAL".to_owned(),
            schema_version: 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatutoryDeductionLineResponse {
    pub kind: StatutoryDeductionKindDto,  // data_class: INTERNAL_ONLY
    pub amount: MoneyAmountDto,           // data_class: FINANCIAL
    pub synthetic_rate_basis_points: u32, // data_class: FINANCIAL
    pub source_evidence_ref: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YearEndSettlementPreviewRequest {
    pub run_id: String,                                // data_class: INTERNAL_ONLY
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                       // data_class: INTERNAL_ONLY
    pub payroll_year: u32,                             // data_class: FINANCIAL
    pub jurisdiction: PayrollRulepackJurisdictionDto,  // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                          // data_class: INTERNAL_ONLY
    pub rulepack_manifest_ref: Option<String>,         // data_class: INTERNAL_ONLY
    pub source_version: Option<String>,                // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<YearEndEvidenceRefRequest>, // data_class: INTERNAL_ONLY
    pub regional_dependencies: Vec<YearEndRegionalDependencyRequest>, // data_class: INTERNAL_ONLY
    pub employee_inputs: Vec<YearEndEmployeeInputRequest>, // data_class: PII_IDENTIFYING + FINANCIAL
    pub fixture_note: String,                              // data_class: PUBLIC
    pub unofficial_source_fixture: bool,                   // data_class: INTERNAL_ONLY
    pub direct_agency_submission_requested: bool,          // data_class: PUBLIC
    pub filing_rail_requested: bool,                       // data_class: PUBLIC
    pub disbursement_rail_requested: bool,                 // data_class: PUBLIC
    pub production_close_requested: bool,                  // data_class: PUBLIC
    pub cloud_deployment_requested: bool,                  // data_class: PUBLIC
}

impl YearEndSettlementPreviewRequest {
    pub fn into_domain(self) -> YearEndSettlementInput {
        let fixture_note = if self.fixture_note == YEAR_END_PREVIEW_FIXTURE_NOTE {
            YEAR_END_PREVIEW_FIXTURE_NOTE
        } else {
            "invalid year-end settlement preview fixture note"
        };
        YearEndSettlementInput {
            run_id: self.run_id,
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            payroll_year: self.payroll_year,
            jurisdiction: self.jurisdiction.into(),
            rulepack_ref: self.rulepack_ref,
            rulepack_manifest_ref: self.rulepack_manifest_ref,
            source_version: self.source_version,
            evidence_refs: self
                .evidence_refs
                .into_iter()
                .map(YearEndEvidenceRefRequest::into_domain)
                .collect(),
            regional_dependencies: self
                .regional_dependencies
                .into_iter()
                .map(YearEndRegionalDependencyRequest::into_domain)
                .collect(),
            employee_inputs: self
                .employee_inputs
                .into_iter()
                .map(YearEndEmployeeInputRequest::into_domain)
                .collect(),
            fixture_note,
            unofficial_source_fixture: self.unofficial_source_fixture,
            direct_agency_submission_requested: self.direct_agency_submission_requested,
            filing_rail_requested: self.filing_rail_requested,
            disbursement_rail_requested: self.disbursement_rail_requested,
            production_close_requested: self.production_close_requested,
            cloud_deployment_requested: self.cloud_deployment_requested,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YearEndEvidenceRefRequest {
    pub source_kind: YearEndSettlementSourceKindDto, // data_class: INTERNAL_ONLY
    pub ref_value: String,                           // data_class: INTERNAL_ONLY
    pub source_version: String,                      // data_class: INTERNAL_ONLY
}

impl YearEndEvidenceRefRequest {
    pub fn into_domain(self) -> YearEndEvidenceRefInput {
        YearEndEvidenceRefInput {
            source_kind: self.source_kind.into(),
            ref_value: self.ref_value,
            source_version: self.source_version,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YearEndRegionalDependencyRequest {
    pub pack_code: String,      // data_class: INTERNAL_ONLY
    pub source_version: String, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,   // data_class: INTERNAL_ONLY
}

impl YearEndRegionalDependencyRequest {
    pub fn into_domain(self) -> YearEndRegionalDependency {
        YearEndRegionalDependency {
            pack_code: self.pack_code,
            source_version: self.source_version,
            evidence_ref: self.evidence_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YearEndEmployeeInputRequest {
    pub payee_id: String,                 // data_class: INTERNAL_ONLY
    pub employee_ref: String,             // data_class: PII_IDENTIFYING
    pub gross_pay: MoneyAmountDto,        // data_class: FINANCIAL
    pub withholding: MoneyAmountDto,      // data_class: FINANCIAL
    pub wage_ledger_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub declaration_evidence_ref: String, // data_class: INTERNAL_ONLY
}

impl YearEndEmployeeInputRequest {
    pub fn into_domain(self) -> oya_payroll_run_domain::YearEndEmployeeInput {
        let gross_currency = self.gross_pay.currency;
        let withholding_currency = self.withholding.currency;
        // The domain seam carries one employee money currency; preserve fail-closed
        // behavior instead of silently dropping a mismatched withholding currency.
        let currency = if gross_currency == withholding_currency {
            gross_currency
        } else {
            String::new()
        };

        oya_payroll_run_domain::YearEndEmployeeInput {
            payee_id: self.payee_id,
            employee_ref: self.employee_ref,
            gross_pay_minor: self.gross_pay.amount_minor,
            withholding_minor: self.withholding.amount_minor,
            currency,
            wage_ledger_evidence_ref: self.wage_ledger_evidence_ref,
            declaration_evidence_ref: self.declaration_evidence_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YearEndSettlementPreviewResponse {
    pub run_id: String,                               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                      // data_class: INTERNAL_ONLY
    pub payroll_year: u32,                            // data_class: FINANCIAL
    pub jurisdiction: PayrollRulepackJurisdictionDto, // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                         // data_class: INTERNAL_ONLY
    pub rulepack_manifest_ref: String,                // data_class: INTERNAL_ONLY
    pub source_version: String,                       // data_class: INTERNAL_ONLY
    pub fixture_note: String,                         // data_class: PUBLIC
    pub evidence_ref_count: usize,                    // data_class: INTERNAL_ONLY
    pub regional_dependency_count: usize,             // data_class: INTERNAL_ONLY
    pub employee_input_count: usize,                  // data_class: INTERNAL_ONLY
    pub direct_agency_submission_attached: bool,      // data_class: PUBLIC
    pub filing_rail_attached: bool,                   // data_class: PUBLIC
    pub disbursement_rail_attached: bool,             // data_class: PUBLIC
    pub production_close_attached: bool,              // data_class: PUBLIC
    pub cloud_deployment_attached: bool,              // data_class: PUBLIC
    pub payload_data_class: String,                   // data_class: INTERNAL_ONLY
    pub schema_version: u32,                          // data_class: PUBLIC
}

impl YearEndSettlementPreviewResponse {
    pub fn from_prepared(prepared: &PreparedYearEndSettlementInput) -> Self {
        Self {
            run_id: prepared.run_id.value.clone(),
            tenant_id: prepared.tenant_id.value.clone(),
            legal_entity_id: prepared.legal_entity_id.value.clone(),
            payroll_year: prepared.payroll_year.value,
            jurisdiction: prepared.jurisdiction.value.into(),
            rulepack_ref: prepared.rulepack_ref.value.clone(),
            rulepack_manifest_ref: prepared.rulepack_manifest_ref.value.clone(),
            source_version: prepared.source_version.value.clone(),
            fixture_note: prepared.fixture_note.value.clone(),
            evidence_ref_count: prepared.evidence_refs.value.len(),
            regional_dependency_count: prepared.regional_dependencies.value.len(),
            employee_input_count: prepared.employee_inputs.value.len(),
            direct_agency_submission_attached: prepared.direct_agency_submission_attached.value,
            filing_rail_attached: prepared.filing_rail_attached.value,
            disbursement_rail_attached: prepared.disbursement_rail_attached.value,
            production_close_attached: prepared.production_close_attached.value,
            cloud_deployment_attached: prepared.cloud_deployment_attached.value,
            payload_data_class: "PII_IDENTIFYING+FINANCIAL".to_owned(),
            schema_version: 1,
        }
    }
}

const HR_LEAVE_IMPACT_SOURCE_TOPIC: &str = "integration.hr.payroll.leave-impact";
const HR_LEAVE_IMPACT_SOURCE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HrLeaveImpactSourceEnvelope {
    pub integration_topic: String,           // data_class: INTERNAL_ONLY
    pub idempotency_key: String,             // data_class: INTERNAL_ONLY
    pub source_topic: String,                // data_class: INTERNAL_ONLY
    pub source_hr_idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,             // data_class: INTERNAL_ONLY
    pub employee_id: String,                 // data_class: INTERNAL_ONLY
    pub leave_request_id: String,            // data_class: INTERNAL_ONLY
    pub decision_evidence_ref: String,       // data_class: INTERNAL_ONLY
    pub routing_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub payroll_impact_evidence_ref: String, // data_class: FINANCIAL
    pub hr_rulepack_ref: String,             // data_class: INTERNAL_ONLY
    pub hr_rulepack_effective_date: String,  // data_class: INTERNAL_ONLY
    pub decided_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub payroll_period: String,              // data_class: FINANCIAL
    pub payroll_impact_kind: HrLeaveImpactKindDto, // data_class: FINANCIAL
    pub payload_data_class: String,          // data_class: INTERNAL_ONLY
    pub payroll_calculation_attached: bool,  // data_class: PUBLIC
    pub payroll_network_call: bool,          // data_class: PUBLIC
    pub workflow_execution: bool,            // data_class: PUBLIC
    pub storage_attached: bool,              // data_class: PUBLIC
    pub runtime_audit_emission: bool,        // data_class: PUBLIC
    pub schema_version: u32,                 // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrLeaveImpactIntakeContext {
    pub run_id: String,                      // data_class: INTERNAL_ONLY
    pub payee_id: String,                    // data_class: INTERNAL_ONLY
    pub payroll_intake_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HrLeaveImpactSourceEnvelopeError {
    InvalidSourceTopic,
    UnsupportedSchemaVersion,
    UnexpectedPayloadDataClass,
    MismatchedSourceIdempotency,
    SourceOverclaimsRuntimeWork,
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
    pub fn from_hr_leave_impact_source(
        source: HrLeaveImpactSourceEnvelope,
        context: HrLeaveImpactIntakeContext,
    ) -> Result<Self, HrLeaveImpactSourceEnvelopeError> {
        if source.integration_topic != HR_LEAVE_IMPACT_SOURCE_TOPIC
            || source.source_topic != HR_LEAVE_IMPACT_SOURCE_TOPIC
        {
            return Err(HrLeaveImpactSourceEnvelopeError::InvalidSourceTopic);
        }
        if source.schema_version != HR_LEAVE_IMPACT_SOURCE_SCHEMA_VERSION {
            return Err(HrLeaveImpactSourceEnvelopeError::UnsupportedSchemaVersion);
        }
        if source.payload_data_class != "FINANCIAL" {
            return Err(HrLeaveImpactSourceEnvelopeError::UnexpectedPayloadDataClass);
        }
        if source.idempotency_key != source.source_hr_idempotency_key {
            return Err(HrLeaveImpactSourceEnvelopeError::MismatchedSourceIdempotency);
        }
        if source.payroll_calculation_attached
            || source.payroll_network_call
            || source.workflow_execution
            || source.storage_attached
            || source.runtime_audit_emission
        {
            return Err(HrLeaveImpactSourceEnvelopeError::SourceOverclaimsRuntimeWork);
        }

        Ok(Self {
            run_id: context.run_id,
            tenant_id: source.tenant_id,
            legal_entity_id: source.legal_entity_id,
            payroll_period: source.payroll_period,
            payee_id: context.payee_id,
            employee_id: source.employee_id,
            leave_request_id: source.leave_request_id,
            impact_kind: source.payroll_impact_kind,
            source_topic: source.source_topic,
            source_hr_idempotency_key: source.source_hr_idempotency_key,
            decision_evidence_ref: source.decision_evidence_ref,
            routing_evidence_ref: source.routing_evidence_ref,
            payroll_impact_evidence_ref: source.payroll_impact_evidence_ref,
            payroll_intake_evidence_ref: context.payroll_intake_evidence_ref,
            rulepack_ref: source.hr_rulepack_ref,
            rulepack_effective_date: source.hr_rulepack_effective_date,
            received_at_epoch_seconds: context.received_at_epoch_seconds,
        })
    }

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
pub enum PayrollRulepackJurisdictionDto {
    Korea,
    UnitedStatesFederal,
    EuropeanUnion,
}

impl From<PayrollRulepackJurisdictionDto> for PayrollRulepackJurisdiction {
    fn from(value: PayrollRulepackJurisdictionDto) -> Self {
        match value {
            PayrollRulepackJurisdictionDto::Korea => Self::Korea,
            PayrollRulepackJurisdictionDto::UnitedStatesFederal => Self::UnitedStatesFederal,
            PayrollRulepackJurisdictionDto::EuropeanUnion => Self::EuropeanUnion,
        }
    }
}

impl From<PayrollRulepackJurisdiction> for PayrollRulepackJurisdictionDto {
    fn from(value: PayrollRulepackJurisdiction) -> Self {
        match value {
            PayrollRulepackJurisdiction::Korea => Self::Korea,
            PayrollRulepackJurisdiction::UnitedStatesFederal => Self::UnitedStatesFederal,
            PayrollRulepackJurisdiction::EuropeanUnion => Self::EuropeanUnion,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatutoryDeductionKindDto {
    IncomeTax,
    SocialInsurance,
}

impl From<StatutoryDeductionKindDto> for StatutoryDeductionKind {
    fn from(value: StatutoryDeductionKindDto) -> Self {
        match value {
            StatutoryDeductionKindDto::IncomeTax => Self::IncomeTax,
            StatutoryDeductionKindDto::SocialInsurance => Self::SocialInsurance,
        }
    }
}

impl From<StatutoryDeductionKind> for StatutoryDeductionKindDto {
    fn from(value: StatutoryDeductionKind) -> Self {
        match value {
            StatutoryDeductionKind::IncomeTax => Self::IncomeTax,
            StatutoryDeductionKind::SocialInsurance => Self::SocialInsurance,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalculationBoundaryDto {
    PureDomainNoFilingTransport,
}

impl From<CalculationBoundary> for CalculationBoundaryDto {
    fn from(value: CalculationBoundary) -> Self {
        match value {
            CalculationBoundary::PureDomainNoFilingTransport => Self::PureDomainNoFilingTransport,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum YearEndSettlementSourceKindDto {
    WageLedgerDigest,
    WithholdingEvidence,
    EmployeeDeclaration,
}

impl From<YearEndSettlementSourceKindDto> for YearEndSettlementSourceKind {
    fn from(value: YearEndSettlementSourceKindDto) -> Self {
        match value {
            YearEndSettlementSourceKindDto::WageLedgerDigest => Self::WageLedgerDigest,
            YearEndSettlementSourceKindDto::WithholdingEvidence => Self::WithholdingEvidence,
            YearEndSettlementSourceKindDto::EmployeeDeclaration => Self::EmployeeDeclaration,
        }
    }
}

impl From<YearEndSettlementSourceKind> for YearEndSettlementSourceKindDto {
    fn from(value: YearEndSettlementSourceKind) -> Self {
        match value {
            YearEndSettlementSourceKind::WageLedgerDigest => Self::WageLedgerDigest,
            YearEndSettlementSourceKind::WithholdingEvidence => Self::WithholdingEvidence,
            YearEndSettlementSourceKind::EmployeeDeclaration => Self::EmployeeDeclaration,
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
