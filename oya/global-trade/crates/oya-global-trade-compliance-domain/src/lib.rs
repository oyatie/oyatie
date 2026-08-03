//! Global trade compliance domain foundation.
//!
//! This crate owns pure, metadata-only global-trade invariants for party
//! screening, trade-item classification, export-control assessment, customs
//! declaration preparation, and landed-cost simulation. It does not perform live
//! denied-party network calls, sanctioned-list downloads, government customs or
//! export filing, broker submission, legal-ruling retrieval, shipment/order/
//! inventory/accounting mutation, Workflow execution, runtime audit-chain
//! emission, or cloud runtime I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const PARTY_SCREENING_ID_PREFIX: &str = "screen_";
const TRADE_PARTY_ID_PREFIX: &str = "party_";
const CLASSIFICATION_ID_PREFIX: &str = "class_";
const EXPORT_ASSESSMENT_ID_PREFIX: &str = "export_";
const DECLARATION_ID_PREFIX: &str = "decl_";
const LANDED_COST_ID_PREFIX: &str = "landed_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const ITEM_ID_PREFIX: &str = "item_";
const SANCTIONS_LIST_REF_PREFIX: &str = "list/";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const GLOBAL_TRADE_COMPLIANCE_SCHEMA_VERSION: u32 = 1;
const BASIS_POINTS_DENOMINATOR: u64 = 10_000;
const SANCTIONS_HOLD_THRESHOLD_BPS: u16 = 8_500;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GlobalTradeComplianceDomain;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartyScreeningId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TradePartyId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClassificationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExportAssessmentId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CustomsDeclarationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LandedCostId {
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
pub struct ItemId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CountryCode {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartyName {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SanctionsListRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HsCode {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExportControlClassification {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IncotermCode {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CustomsProcedureCode {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceDocumentRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TradePartyRole {
    Customer,
    Supplier,
    Consignee,
    Broker,
    Forwarder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ScreeningOutcome {
    Cleared,
    PotentialMatchHold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PartyScreeningState {
    Screened,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TradeItemClassificationState {
    Classified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExportControlDecision {
    AllowedNoLicense,
    AllowedWithLicense,
    LicenseRequiredHold,
    EmbargoHold,
    PartyHold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExportControlAssessmentState {
    Assessed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CustomsDeclarationType {
    Export,
    Import,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CustomsDeclarationState {
    Prepared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LandedCostState {
    Simulated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradePartyScreeningInput {
    pub screening_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,            // data_class: INTERNAL_ONLY
    pub trade_party_id: String,             // data_class: INTERNAL_ONLY
    pub party_role: TradePartyRole,         // data_class: INTERNAL_ONLY
    pub country_code: String,               // data_class: INTERNAL_ONLY
    pub normalized_party_name: String,      // data_class: INTERNAL_ONLY
    pub sanctions_list_version_ref: String, // data_class: INTERNAL_ONLY
    pub screening_score_bps: u16,           // data_class: INTERNAL_ONLY
    pub screening_source_ref: String,       // data_class: INTERNAL_ONLY
    pub screening_evidence_ref: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradePartyScreening {
    pub screening_id: Classified<PartyScreeningId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,            // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub trade_party_id: Classified<TradePartyId>,   // data_class: INTERNAL_ONLY
    pub party_role: Classified<TradePartyRole>,     // data_class: INTERNAL_ONLY
    pub country_code: Classified<CountryCode>,      // data_class: INTERNAL_ONLY
    pub normalized_party_name: Classified<PartyName>, // data_class: INTERNAL_ONLY
    pub sanctions_list_version_ref: Classified<SanctionsListRef>, // data_class: INTERNAL_ONLY
    pub screening_score_bps: Classified<u16>,       // data_class: INTERNAL_ONLY
    pub screening_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub screening_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub outcome: Classified<ScreeningOutcome>,      // data_class: INTERNAL_ONLY
    pub state: Classified<PartyScreeningState>,     // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,        // data_class: INTERNAL_ONLY
    pub restricted_party_hold_required: Classified<bool>, // data_class: PUBLIC
    pub live_sanctions_provider_attached: Classified<bool>, // data_class: PUBLIC
    pub government_list_download_attached: Classified<bool>, // data_class: PUBLIC
    pub business_transaction_block_mutation_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeItemClassificationInput {
    pub classification_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,          // data_class: INTERNAL_ONLY
    pub item_id: String,                  // data_class: INTERNAL_ONLY
    pub country_of_origin_code: String,   // data_class: INTERNAL_ONLY
    pub destination_country_code: String, // data_class: INTERNAL_ONLY
    pub hs_code: String,                  // data_class: INTERNAL_ONLY
    pub export_control_classification_number: String, // data_class: INTERNAL_ONLY
    pub unit_customs_value_cents: u64,    // data_class: FINANCIAL
    pub duty_rate_bps: u16,               // data_class: FINANCIAL
    pub classification_source_ref: String, // data_class: INTERNAL_ONLY
    pub classification_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeItemClassification {
    pub classification_id: Classified<ClassificationId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,      // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                     // data_class: INTERNAL_ONLY
    pub country_of_origin_code: Classified<CountryCode>, // data_class: INTERNAL_ONLY
    pub destination_country_code: Classified<CountryCode>, // data_class: INTERNAL_ONLY
    pub hs_code: Classified<HsCode>,                     // data_class: INTERNAL_ONLY
    pub export_control_classification_number: Classified<ExportControlClassification>, // data_class: INTERNAL_ONLY
    pub unit_customs_value_cents: Classified<u64>, // data_class: FINANCIAL
    pub duty_rate_bps: Classified<u16>,            // data_class: FINANCIAL
    pub classification_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub classification_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<TradeItemClassificationState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,       // data_class: INTERNAL_ONLY
    pub regulatory_content_provider_attached: Classified<bool>, // data_class: PUBLIC
    pub legal_ruling_attached: Classified<bool>,   // data_class: PUBLIC
    pub product_master_mutation_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportControlAssessmentInput {
    pub assessment_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,          // data_class: INTERNAL_ONLY
    pub classification_id: String,        // data_class: INTERNAL_ONLY
    pub item_id: String,                  // data_class: INTERNAL_ONLY
    pub destination_country_code: String, // data_class: INTERNAL_ONLY
    pub export_control_classification_number: String, // data_class: INTERNAL_ONLY
    pub shipment_value_cents: u64,        // data_class: FINANCIAL
    pub trade_party_screened: bool,       // data_class: INTERNAL_ONLY
    pub trade_item_classified: bool,      // data_class: INTERNAL_ONLY
    pub party_screening_outcome: ScreeningOutcome, // data_class: INTERNAL_ONLY
    pub license_present: bool,            // data_class: INTERNAL_ONLY
    pub embargo_country: bool,            // data_class: INTERNAL_ONLY
    pub assessment_source_ref: String,    // data_class: INTERNAL_ONLY
    pub assessment_evidence_ref: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportControlAssessment {
    pub assessment_id: Classified<ExportAssessmentId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,    // data_class: INTERNAL_ONLY
    pub classification_id: Classified<ClassificationId>, // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                   // data_class: INTERNAL_ONLY
    pub destination_country_code: Classified<CountryCode>, // data_class: INTERNAL_ONLY
    pub export_control_classification_number: Classified<ExportControlClassification>, // data_class: INTERNAL_ONLY
    pub shipment_value_cents: Classified<u64>, // data_class: FINANCIAL
    pub party_screening_outcome: Classified<ScreeningOutcome>, // data_class: INTERNAL_ONLY
    pub license_present: Classified<bool>,     // data_class: INTERNAL_ONLY
    pub embargo_country: Classified<bool>,     // data_class: INTERNAL_ONLY
    pub decision: Classified<ExportControlDecision>, // data_class: INTERNAL_ONLY
    pub assessment_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub assessment_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<ExportControlAssessmentState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,   // data_class: INTERNAL_ONLY
    pub compliance_hold_required: Classified<bool>, // data_class: PUBLIC
    pub export_license_management_attached: Classified<bool>, // data_class: PUBLIC
    pub government_export_filing_attached: Classified<bool>, // data_class: PUBLIC
    pub order_hold_mutation_attached: Classified<bool>, // data_class: PUBLIC
    pub workflow_execution_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomsDeclarationInput {
    pub declaration_id: String,                   // data_class: INTERNAL_ONLY
    pub assessment_id: String,                    // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                  // data_class: INTERNAL_ONLY
    pub item_id: String,                          // data_class: INTERNAL_ONLY
    pub trade_party_id: String,                   // data_class: INTERNAL_ONLY
    pub declaration_type: CustomsDeclarationType, // data_class: INTERNAL_ONLY
    pub export_control_assessed: bool,            // data_class: INTERNAL_ONLY
    pub export_control_decision: ExportControlDecision, // data_class: INTERNAL_ONLY
    pub quantity: u32,                            // data_class: FINANCIAL
    pub unit_customs_value_cents: u64,            // data_class: FINANCIAL
    pub freight_cents: u64,                       // data_class: FINANCIAL
    pub insurance_cents: u64,                     // data_class: FINANCIAL
    pub incoterm_code: String,                    // data_class: INTERNAL_ONLY
    pub customs_procedure_code: String,           // data_class: INTERNAL_ONLY
    pub declaration_source_ref: String,           // data_class: INTERNAL_ONLY
    pub declaration_evidence_ref: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomsDeclarationPreparation {
    pub declaration_id: Classified<CustomsDeclarationId>, // data_class: INTERNAL_ONLY
    pub assessment_id: Classified<ExportAssessmentId>,    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,       // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                      // data_class: INTERNAL_ONLY
    pub trade_party_id: Classified<TradePartyId>,         // data_class: INTERNAL_ONLY
    pub declaration_type: Classified<CustomsDeclarationType>, // data_class: INTERNAL_ONLY
    pub export_control_decision: Classified<ExportControlDecision>, // data_class: INTERNAL_ONLY
    pub quantity: Classified<u32>,                        // data_class: FINANCIAL
    pub unit_customs_value_cents: Classified<u64>,        // data_class: FINANCIAL
    pub goods_value_cents: Classified<u64>,               // data_class: FINANCIAL
    pub freight_cents: Classified<u64>,                   // data_class: FINANCIAL
    pub insurance_cents: Classified<u64>,                 // data_class: FINANCIAL
    pub declared_customs_value_cents: Classified<u64>,    // data_class: FINANCIAL
    pub incoterm_code: Classified<IncotermCode>,          // data_class: INTERNAL_ONLY
    pub customs_procedure_code: Classified<CustomsProcedureCode>, // data_class: INTERNAL_ONLY
    pub declaration_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub declaration_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<CustomsDeclarationState>,       // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,              // data_class: INTERNAL_ONLY
    pub customs_authority_submission_attached: Classified<bool>, // data_class: PUBLIC
    pub broker_network_attached: Classified<bool>,        // data_class: PUBLIC
    pub shipment_mutation_attached: Classified<bool>,     // data_class: PUBLIC
    pub document_archive_attached: Classified<bool>,      // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,      // data_class: PUBLIC
    pub schema_version: Classified<u32>,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedCostSimulationInput {
    pub landed_cost_id: String,            // data_class: INTERNAL_ONLY
    pub declaration_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub declaration_prepared: bool,        // data_class: INTERNAL_ONLY
    pub declared_customs_value_cents: u64, // data_class: FINANCIAL
    pub duty_rate_bps: u16,                // data_class: FINANCIAL
    pub brokerage_fee_cents: u64,          // data_class: FINANCIAL
    pub local_transport_cents: u64,        // data_class: FINANCIAL
    pub cost_source_ref: String,           // data_class: INTERNAL_ONLY
    pub cost_evidence_ref: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedCostSimulation {
    pub landed_cost_id: Classified<LandedCostId>, // data_class: INTERNAL_ONLY
    pub declaration_id: Classified<CustomsDeclarationId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,          // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub declared_customs_value_cents: Classified<u64>, // data_class: FINANCIAL
    pub duty_rate_bps: Classified<u16>,           // data_class: FINANCIAL
    pub duty_amount_cents: Classified<u64>,       // data_class: FINANCIAL
    pub brokerage_fee_cents: Classified<u64>,     // data_class: FINANCIAL
    pub local_transport_cents: Classified<u64>,   // data_class: FINANCIAL
    pub total_landed_cost_cents: Classified<u64>, // data_class: FINANCIAL
    pub cost_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub cost_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<LandedCostState>,       // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,      // data_class: INTERNAL_ONLY
    pub accounting_posting_attached: Classified<bool>, // data_class: PUBLIC
    pub inventory_cost_update_attached: Classified<bool>, // data_class: PUBLIC
    pub payment_disbursement_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobalTradeComplianceError {
    InvalidScreeningId,
    InvalidTradePartyId,
    InvalidClassificationId,
    InvalidExportAssessmentId,
    InvalidCustomsDeclarationId,
    InvalidLandedCostId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidItemId,
    InvalidCountryCode,
    InvalidPartyName,
    InvalidSanctionsListRef,
    InvalidHsCode,
    InvalidExportControlClassification,
    InvalidAmount,
    InvalidScreeningScore,
    InvalidDutyRate,
    InvalidQuantity,
    InvalidIncotermCode,
    InvalidCustomsProcedureCode,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    PartyScreeningRequired,
    ItemClassificationRequired,
    ExportControlAssessmentRequired,
    CustomsDeclarationRequired,
    ComplianceHoldRequired,
}

pub fn screen_trade_party(
    input: TradePartyScreeningInput,
) -> Result<TradePartyScreening, GlobalTradeComplianceError> {
    validate_id(
        &input.screening_id,
        PARTY_SCREENING_ID_PREFIX,
        GlobalTradeComplianceError::InvalidScreeningId,
    )?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_id(
        &input.trade_party_id,
        TRADE_PARTY_ID_PREFIX,
        GlobalTradeComplianceError::InvalidTradePartyId,
    )?;
    validate_country_code(&input.country_code)?;
    validate_party_name(&input.normalized_party_name)?;
    validate_list_ref(&input.sanctions_list_version_ref)?;
    validate_screening_score(input.screening_score_bps)?;
    validate_source_ref(&input.screening_source_ref)?;
    validate_evidence_ref(&input.screening_evidence_ref)?;
    let restricted_party_hold_required = input.screening_score_bps >= SANCTIONS_HOLD_THRESHOLD_BPS;
    let outcome = if restricted_party_hold_required {
        ScreeningOutcome::PotentialMatchHold
    } else {
        ScreeningOutcome::Cleared
    };
    let idempotency_key = format!(
        "global-trade:screen:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.screening_id
    );

    Ok(TradePartyScreening {
        screening_id: internal(PartyScreeningId {
            value: input.screening_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        trade_party_id: internal(TradePartyId {
            value: input.trade_party_id,
        }),
        party_role: internal(input.party_role),
        country_code: internal(CountryCode {
            value: input.country_code,
        }),
        normalized_party_name: internal(PartyName {
            value: input.normalized_party_name,
        }),
        sanctions_list_version_ref: internal(SanctionsListRef {
            value: input.sanctions_list_version_ref,
        }),
        screening_score_bps: internal(input.screening_score_bps),
        screening_source_ref: internal(SourceDocumentRef {
            value: input.screening_source_ref,
        }),
        screening_evidence_ref: internal(EvidenceRef {
            value: input.screening_evidence_ref,
        }),
        outcome: internal(outcome),
        state: internal(PartyScreeningState::Screened),
        idempotency_key: internal(idempotency_key),
        restricted_party_hold_required: public(restricted_party_hold_required),
        live_sanctions_provider_attached: public(false),
        government_list_download_attached: public(false),
        business_transaction_block_mutation_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(GLOBAL_TRADE_COMPLIANCE_SCHEMA_VERSION),
    })
}

pub fn classify_trade_item(
    input: TradeItemClassificationInput,
) -> Result<TradeItemClassification, GlobalTradeComplianceError> {
    validate_id(
        &input.classification_id,
        CLASSIFICATION_ID_PREFIX,
        GlobalTradeComplianceError::InvalidClassificationId,
    )?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_item_id(&input.item_id)?;
    validate_country_code(&input.country_of_origin_code)?;
    validate_country_code(&input.destination_country_code)?;
    validate_hs_code(&input.hs_code)?;
    validate_export_control_classification(&input.export_control_classification_number)?;
    validate_positive_amount(input.unit_customs_value_cents)?;
    validate_duty_rate(input.duty_rate_bps)?;
    validate_source_ref(&input.classification_source_ref)?;
    validate_evidence_ref(&input.classification_evidence_ref)?;
    let idempotency_key = format!(
        "global-trade:classify:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.classification_id
    );

    Ok(TradeItemClassification {
        classification_id: internal(ClassificationId {
            value: input.classification_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        country_of_origin_code: internal(CountryCode {
            value: input.country_of_origin_code,
        }),
        destination_country_code: internal(CountryCode {
            value: input.destination_country_code,
        }),
        hs_code: internal(HsCode {
            value: input.hs_code,
        }),
        export_control_classification_number: internal(ExportControlClassification {
            value: input.export_control_classification_number,
        }),
        unit_customs_value_cents: financial(input.unit_customs_value_cents),
        duty_rate_bps: financial(input.duty_rate_bps),
        classification_source_ref: internal(SourceDocumentRef {
            value: input.classification_source_ref,
        }),
        classification_evidence_ref: internal(EvidenceRef {
            value: input.classification_evidence_ref,
        }),
        state: internal(TradeItemClassificationState::Classified),
        idempotency_key: internal(idempotency_key),
        regulatory_content_provider_attached: public(false),
        legal_ruling_attached: public(false),
        product_master_mutation_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(GLOBAL_TRADE_COMPLIANCE_SCHEMA_VERSION),
    })
}

pub fn assess_export_control(
    input: ExportControlAssessmentInput,
) -> Result<ExportControlAssessment, GlobalTradeComplianceError> {
    validate_id(
        &input.assessment_id,
        EXPORT_ASSESSMENT_ID_PREFIX,
        GlobalTradeComplianceError::InvalidExportAssessmentId,
    )?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_id(
        &input.classification_id,
        CLASSIFICATION_ID_PREFIX,
        GlobalTradeComplianceError::InvalidClassificationId,
    )?;
    validate_item_id(&input.item_id)?;
    validate_country_code(&input.destination_country_code)?;
    validate_export_control_classification(&input.export_control_classification_number)?;
    validate_positive_amount(input.shipment_value_cents)?;
    if !input.trade_party_screened {
        return Err(GlobalTradeComplianceError::PartyScreeningRequired);
    }
    if !input.trade_item_classified {
        return Err(GlobalTradeComplianceError::ItemClassificationRequired);
    }
    validate_source_ref(&input.assessment_source_ref)?;
    validate_evidence_ref(&input.assessment_evidence_ref)?;
    let decision = export_decision(
        input.party_screening_outcome,
        &input.export_control_classification_number,
        input.license_present,
        input.embargo_country,
    );
    let idempotency_key = format!(
        "global-trade:export:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.assessment_id
    );

    Ok(ExportControlAssessment {
        assessment_id: internal(ExportAssessmentId {
            value: input.assessment_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        classification_id: internal(ClassificationId {
            value: input.classification_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        destination_country_code: internal(CountryCode {
            value: input.destination_country_code,
        }),
        export_control_classification_number: internal(ExportControlClassification {
            value: input.export_control_classification_number,
        }),
        shipment_value_cents: financial(input.shipment_value_cents),
        party_screening_outcome: internal(input.party_screening_outcome),
        license_present: internal(input.license_present),
        embargo_country: internal(input.embargo_country),
        decision: internal(decision),
        assessment_source_ref: internal(SourceDocumentRef {
            value: input.assessment_source_ref,
        }),
        assessment_evidence_ref: internal(EvidenceRef {
            value: input.assessment_evidence_ref,
        }),
        state: internal(ExportControlAssessmentState::Assessed),
        idempotency_key: internal(idempotency_key),
        compliance_hold_required: public(is_hold_decision(decision)),
        export_license_management_attached: public(false),
        government_export_filing_attached: public(false),
        order_hold_mutation_attached: public(false),
        workflow_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(GLOBAL_TRADE_COMPLIANCE_SCHEMA_VERSION),
    })
}

pub fn prepare_customs_declaration(
    input: CustomsDeclarationInput,
) -> Result<CustomsDeclarationPreparation, GlobalTradeComplianceError> {
    validate_id(
        &input.declaration_id,
        DECLARATION_ID_PREFIX,
        GlobalTradeComplianceError::InvalidCustomsDeclarationId,
    )?;
    validate_id(
        &input.assessment_id,
        EXPORT_ASSESSMENT_ID_PREFIX,
        GlobalTradeComplianceError::InvalidExportAssessmentId,
    )?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_item_id(&input.item_id)?;
    validate_id(
        &input.trade_party_id,
        TRADE_PARTY_ID_PREFIX,
        GlobalTradeComplianceError::InvalidTradePartyId,
    )?;
    if !input.export_control_assessed {
        return Err(GlobalTradeComplianceError::ExportControlAssessmentRequired);
    }
    if is_hold_decision(input.export_control_decision) {
        return Err(GlobalTradeComplianceError::ComplianceHoldRequired);
    }
    validate_positive_quantity(input.quantity)?;
    validate_positive_amount(input.unit_customs_value_cents)?;
    validate_incoterm_code(&input.incoterm_code)?;
    validate_customs_procedure_code(&input.customs_procedure_code)?;
    validate_source_ref(&input.declaration_source_ref)?;
    validate_evidence_ref(&input.declaration_evidence_ref)?;
    let goods_value_cents = input
        .unit_customs_value_cents
        .checked_mul(u64::from(input.quantity))
        .ok_or(GlobalTradeComplianceError::InvalidAmount)?;
    let declared_customs_value_cents = goods_value_cents
        .checked_add(input.freight_cents)
        .and_then(|value| value.checked_add(input.insurance_cents))
        .ok_or(GlobalTradeComplianceError::InvalidAmount)?;
    let idempotency_key = format!(
        "global-trade:customs:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.declaration_id
    );

    Ok(CustomsDeclarationPreparation {
        declaration_id: internal(CustomsDeclarationId {
            value: input.declaration_id,
        }),
        assessment_id: internal(ExportAssessmentId {
            value: input.assessment_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        trade_party_id: internal(TradePartyId {
            value: input.trade_party_id,
        }),
        declaration_type: internal(input.declaration_type),
        export_control_decision: internal(input.export_control_decision),
        quantity: financial(input.quantity),
        unit_customs_value_cents: financial(input.unit_customs_value_cents),
        goods_value_cents: financial(goods_value_cents),
        freight_cents: financial(input.freight_cents),
        insurance_cents: financial(input.insurance_cents),
        declared_customs_value_cents: financial(declared_customs_value_cents),
        incoterm_code: internal(IncotermCode {
            value: input.incoterm_code,
        }),
        customs_procedure_code: internal(CustomsProcedureCode {
            value: input.customs_procedure_code,
        }),
        declaration_source_ref: internal(SourceDocumentRef {
            value: input.declaration_source_ref,
        }),
        declaration_evidence_ref: internal(EvidenceRef {
            value: input.declaration_evidence_ref,
        }),
        state: internal(CustomsDeclarationState::Prepared),
        idempotency_key: internal(idempotency_key),
        customs_authority_submission_attached: public(false),
        broker_network_attached: public(false),
        shipment_mutation_attached: public(false),
        document_archive_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(GLOBAL_TRADE_COMPLIANCE_SCHEMA_VERSION),
    })
}

pub fn simulate_landed_cost(
    input: LandedCostSimulationInput,
) -> Result<LandedCostSimulation, GlobalTradeComplianceError> {
    validate_id(
        &input.landed_cost_id,
        LANDED_COST_ID_PREFIX,
        GlobalTradeComplianceError::InvalidLandedCostId,
    )?;
    validate_id(
        &input.declaration_id,
        DECLARATION_ID_PREFIX,
        GlobalTradeComplianceError::InvalidCustomsDeclarationId,
    )?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    if !input.declaration_prepared {
        return Err(GlobalTradeComplianceError::CustomsDeclarationRequired);
    }
    validate_positive_amount(input.declared_customs_value_cents)?;
    validate_duty_rate(input.duty_rate_bps)?;
    validate_source_ref(&input.cost_source_ref)?;
    validate_evidence_ref(&input.cost_evidence_ref)?;
    let duty_amount_cents =
        prorated_amount(input.declared_customs_value_cents, input.duty_rate_bps)?;
    let total_landed_cost_cents = input
        .declared_customs_value_cents
        .checked_add(duty_amount_cents)
        .and_then(|value| value.checked_add(input.brokerage_fee_cents))
        .and_then(|value| value.checked_add(input.local_transport_cents))
        .ok_or(GlobalTradeComplianceError::InvalidAmount)?;
    let idempotency_key = format!(
        "global-trade:landed-cost:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.landed_cost_id
    );

    Ok(LandedCostSimulation {
        landed_cost_id: internal(LandedCostId {
            value: input.landed_cost_id,
        }),
        declaration_id: internal(CustomsDeclarationId {
            value: input.declaration_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        declared_customs_value_cents: financial(input.declared_customs_value_cents),
        duty_rate_bps: financial(input.duty_rate_bps),
        duty_amount_cents: financial(duty_amount_cents),
        brokerage_fee_cents: financial(input.brokerage_fee_cents),
        local_transport_cents: financial(input.local_transport_cents),
        total_landed_cost_cents: financial(total_landed_cost_cents),
        cost_source_ref: internal(SourceDocumentRef {
            value: input.cost_source_ref,
        }),
        cost_evidence_ref: internal(EvidenceRef {
            value: input.cost_evidence_ref,
        }),
        state: internal(LandedCostState::Simulated),
        idempotency_key: internal(idempotency_key),
        accounting_posting_attached: public(false),
        inventory_cost_update_attached: public(false),
        payment_disbursement_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(GLOBAL_TRADE_COMPLIANCE_SCHEMA_VERSION),
    })
}

fn export_decision(
    screening_outcome: ScreeningOutcome,
    export_control_classification_number: &str,
    license_present: bool,
    embargo_country: bool,
) -> ExportControlDecision {
    if screening_outcome == ScreeningOutcome::PotentialMatchHold {
        return ExportControlDecision::PartyHold;
    }
    if embargo_country {
        return ExportControlDecision::EmbargoHold;
    }
    if export_control_classification_number.eq_ignore_ascii_case("EAR99") {
        return ExportControlDecision::AllowedNoLicense;
    }
    if license_present {
        ExportControlDecision::AllowedWithLicense
    } else {
        ExportControlDecision::LicenseRequiredHold
    }
}

fn is_hold_decision(decision: ExportControlDecision) -> bool {
    matches!(
        decision,
        ExportControlDecision::LicenseRequiredHold
            | ExportControlDecision::EmbargoHold
            | ExportControlDecision::PartyHold
    )
}

fn validate_tenant_id(value: &str) -> Result<(), GlobalTradeComplianceError> {
    validate_id(
        value,
        TENANT_ID_PREFIX,
        GlobalTradeComplianceError::InvalidTenantId,
    )
}

fn validate_legal_entity_id(value: &str) -> Result<(), GlobalTradeComplianceError> {
    validate_id(
        value,
        LEGAL_ENTITY_ID_PREFIX,
        GlobalTradeComplianceError::InvalidLegalEntityId,
    )
}

fn validate_item_id(value: &str) -> Result<(), GlobalTradeComplianceError> {
    validate_id(
        value,
        ITEM_ID_PREFIX,
        GlobalTradeComplianceError::InvalidItemId,
    )
}

fn validate_id(
    value: &str,
    prefix: &str,
    error: GlobalTradeComplianceError,
) -> Result<(), GlobalTradeComplianceError> {
    if !value.starts_with(prefix) || value.len() <= prefix.len() || has_unsafe_text(value) {
        return Err(error);
    }
    if has_path_traversal(value) || has_credential_shape(value) {
        return Err(error);
    }
    Ok(())
}

fn validate_country_code(value: &str) -> Result<(), GlobalTradeComplianceError> {
    if value.len() != 2 || !value.chars().all(|ch| ch.is_ascii_uppercase()) {
        return Err(GlobalTradeComplianceError::InvalidCountryCode);
    }
    Ok(())
}

fn validate_party_name(value: &str) -> Result<(), GlobalTradeComplianceError> {
    if value.trim().len() < 2 || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(GlobalTradeComplianceError::InvalidPartyName);
    }
    Ok(())
}

fn validate_list_ref(value: &str) -> Result<(), GlobalTradeComplianceError> {
    if !value.starts_with(SANCTIONS_LIST_REF_PREFIX)
        || value.len() <= SANCTIONS_LIST_REF_PREFIX.len()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
    {
        return Err(GlobalTradeComplianceError::InvalidSanctionsListRef);
    }
    Ok(())
}

fn validate_hs_code(value: &str) -> Result<(), GlobalTradeComplianceError> {
    if !(6..=12).contains(&value.len()) || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(GlobalTradeComplianceError::InvalidHsCode);
    }
    Ok(())
}

fn validate_export_control_classification(value: &str) -> Result<(), GlobalTradeComplianceError> {
    if value.is_empty()
        || value.len() > 32
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '.'))
    {
        return Err(GlobalTradeComplianceError::InvalidExportControlClassification);
    }
    Ok(())
}

fn validate_positive_amount(value: u64) -> Result<(), GlobalTradeComplianceError> {
    if value == 0 {
        return Err(GlobalTradeComplianceError::InvalidAmount);
    }
    Ok(())
}

fn validate_bps(
    value: u16,
    error: GlobalTradeComplianceError,
) -> Result<(), GlobalTradeComplianceError> {
    if value > 10_000 {
        return Err(error);
    }
    Ok(())
}

fn validate_screening_score(value: u16) -> Result<(), GlobalTradeComplianceError> {
    validate_bps(value, GlobalTradeComplianceError::InvalidScreeningScore)
}

fn validate_duty_rate(value: u16) -> Result<(), GlobalTradeComplianceError> {
    validate_bps(value, GlobalTradeComplianceError::InvalidDutyRate)
}

fn validate_positive_quantity(value: u32) -> Result<(), GlobalTradeComplianceError> {
    if value == 0 {
        return Err(GlobalTradeComplianceError::InvalidQuantity);
    }
    Ok(())
}

fn validate_incoterm_code(value: &str) -> Result<(), GlobalTradeComplianceError> {
    if value.len() != 3 || !value.chars().all(|ch| ch.is_ascii_uppercase()) {
        return Err(GlobalTradeComplianceError::InvalidIncotermCode);
    }
    Ok(())
}

fn validate_customs_procedure_code(value: &str) -> Result<(), GlobalTradeComplianceError> {
    if value.is_empty()
        || value.len() > 16
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        return Err(GlobalTradeComplianceError::InvalidCustomsProcedureCode);
    }
    Ok(())
}

fn validate_source_ref(value: &str) -> Result<(), GlobalTradeComplianceError> {
    if !value.starts_with(SOURCE_REF_PREFIX)
        || value.len() <= SOURCE_REF_PREFIX.len()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
    {
        return Err(GlobalTradeComplianceError::InvalidSourceDocumentRef);
    }
    Ok(())
}

fn validate_evidence_ref(value: &str) -> Result<(), GlobalTradeComplianceError> {
    if !value.starts_with(AUDIT_REF_PREFIX)
        || value.len() <= AUDIT_REF_PREFIX.len()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
    {
        return Err(GlobalTradeComplianceError::InvalidEvidenceRef);
    }
    Ok(())
}

fn prorated_amount(amount_cents: u64, rate_bps: u16) -> Result<u64, GlobalTradeComplianceError> {
    amount_cents
        .checked_mul(u64::from(rate_bps))
        .map(|value| value / BASIS_POINTS_DENOMINATOR)
        .ok_or(GlobalTradeComplianceError::InvalidAmount)
}

fn has_unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace) || value.chars().any(char::is_control)
}

fn has_path_traversal(value: &str) -> bool {
    value.contains("..") || value.contains('\\') || value.contains("//")
}

fn has_credential_shape(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("api_key")
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
