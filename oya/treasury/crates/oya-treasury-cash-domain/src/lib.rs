//! Treasury cash domain foundation.
//!
//! This crate owns pure treasury invariants for bank-account approval,
//! cash-position snapshots, liquidity forecasts, and cash-transfer proposal
//! metadata for later payment/bank-network handoff. It does not perform
//! persistence, bank-network calls, payment execution, workflow dispatch,
//! runtime audit-chain emission, or cloud runtime I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const BANK_ACCOUNT_ID_PREFIX: &str = "ba_";
const BANK_ID_PREFIX: &str = "bank_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const CASH_POSITION_ID_PREFIX: &str = "cpos_";
const LIQUIDITY_FORECAST_ID_PREFIX: &str = "lfcst_";
const CASH_TRANSFER_PROPOSAL_ID_PREFIX: &str = "xfer_";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const TREASURY_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BankAccountId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BankId {
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
pub struct CashPositionId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LiquidityForecastId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CashTransferProposalId {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoneyAmount {
    pub amount_minor: i64, // data_class: FINANCIAL
    pub currency: String,  // data_class: FINANCIAL
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BankAccountState {
    Approved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CashPositionState {
    Recorded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LiquidityForecastState {
    Projected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CashTransferProposalState {
    Proposed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankAccountApprovalInput {
    pub bank_account_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,              // data_class: INTERNAL_ONLY
    pub bank_id: String,                      // data_class: INTERNAL_ONLY
    pub currency: String,                     // data_class: FINANCIAL
    pub bank_account_master_ref: String,      // data_class: INTERNAL_ONLY
    pub opening_balance_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub control_evidence_ref: String,         // data_class: INTERNAL_ONLY
    pub target_balance: MoneyAmount,          // data_class: FINANCIAL
    pub approved_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankAccountApproval {
    pub bank_account_id: Classified<BankAccountId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,            // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub bank_id: Classified<BankId>,                // data_class: INTERNAL_ONLY
    pub currency: Classified<String>,               // data_class: FINANCIAL
    pub bank_account_master_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub opening_balance_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub control_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub target_balance: Classified<MoneyAmount>,    // data_class: FINANCIAL
    pub state: Classified<BankAccountState>,        // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,        // data_class: INTERNAL_ONLY
    pub approved_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CashPositionInput {
    pub cash_position_id: String,            // data_class: INTERNAL_ONLY
    pub bank_account_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,             // data_class: INTERNAL_ONLY
    pub bank_account_approved: bool,         // data_class: INTERNAL_ONLY
    pub position_date_yyyymmdd: u32,         // data_class: INTERNAL_ONLY
    pub opening_balance: MoneyAmount,        // data_class: FINANCIAL
    pub actual_inflow: MoneyAmount,          // data_class: FINANCIAL
    pub actual_outflow: MoneyAmount,         // data_class: FINANCIAL
    pub bank_statement_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub exposure_flow_ref: String,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CashPositionSnapshot {
    pub cash_position_id: Classified<CashPositionId>, // data_class: INTERNAL_ONLY
    pub bank_account_id: Classified<BankAccountId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,   // data_class: INTERNAL_ONLY
    pub position_date_yyyymmdd: Classified<u32>,      // data_class: INTERNAL_ONLY
    pub opening_balance: Classified<MoneyAmount>,     // data_class: FINANCIAL
    pub actual_inflow: Classified<MoneyAmount>,       // data_class: FINANCIAL
    pub actual_outflow: Classified<MoneyAmount>,      // data_class: FINANCIAL
    pub closing_balance: Classified<MoneyAmount>,     // data_class: FINANCIAL
    pub bank_statement_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub exposure_flow_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<CashPositionState>,         // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,          // data_class: INTERNAL_ONLY
    pub payment_execution_attached: Classified<bool>, // data_class: PUBLIC
    pub bank_network_call_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,  // data_class: PUBLIC
    pub runtime_audit_chain_emission_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityForecastInput {
    pub forecast_id: String,                   // data_class: INTERNAL_ONLY
    pub bank_account_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,               // data_class: INTERNAL_ONLY
    pub cash_position_recorded: bool,          // data_class: INTERNAL_ONLY
    pub horizon_days: u16,                     // data_class: INTERNAL_ONLY
    pub starting_balance: MoneyAmount,         // data_class: FINANCIAL
    pub forecasted_inflow: MoneyAmount,        // data_class: FINANCIAL
    pub forecasted_outflow: MoneyAmount,       // data_class: FINANCIAL
    pub minimum_liquidity_target: MoneyAmount, // data_class: FINANCIAL
    pub forecast_source_ref: String,           // data_class: INTERNAL_ONLY
    pub forecast_evidence_ref: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityForecastProjection {
    pub forecast_id: Classified<LiquidityForecastId>, // data_class: INTERNAL_ONLY
    pub bank_account_id: Classified<BankAccountId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,   // data_class: INTERNAL_ONLY
    pub horizon_days: Classified<u16>,                // data_class: INTERNAL_ONLY
    pub starting_balance: Classified<MoneyAmount>,    // data_class: FINANCIAL
    pub forecasted_inflow: Classified<MoneyAmount>,   // data_class: FINANCIAL
    pub forecasted_outflow: Classified<MoneyAmount>,  // data_class: FINANCIAL
    pub minimum_liquidity_target: Classified<MoneyAmount>, // data_class: FINANCIAL
    pub projected_closing_balance: Classified<MoneyAmount>, // data_class: FINANCIAL
    pub shortfall_amount: Classified<MoneyAmount>,    // data_class: FINANCIAL
    pub liquidity_breach: Classified<bool>,           // data_class: PUBLIC
    pub forecast_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub forecast_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<LiquidityForecastState>,    // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,          // data_class: INTERNAL_ONLY
    pub payment_execution_attached: Classified<bool>, // data_class: PUBLIC
    pub bank_network_call_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,  // data_class: PUBLIC
    pub schema_version: Classified<u32>,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CashTransferProposalInput {
    pub transfer_proposal_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,             // data_class: INTERNAL_ONLY
    pub source_bank_account_id: String,      // data_class: INTERNAL_ONLY
    pub target_bank_account_id: String,      // data_class: INTERNAL_ONLY
    pub transfer_amount: MoneyAmount,        // data_class: FINANCIAL
    pub source_closing_balance: MoneyAmount, // data_class: FINANCIAL
    pub source_minimum_balance: MoneyAmount, // data_class: FINANCIAL
    pub target_closing_balance: MoneyAmount, // data_class: FINANCIAL
    pub target_minimum_balance: MoneyAmount, // data_class: FINANCIAL
    pub cash_pool_ref: String,               // data_class: INTERNAL_ONLY
    pub approval_policy_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub proposal_evidence_ref: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CashTransferProposal {
    pub transfer_proposal_id: Classified<CashTransferProposalId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                          // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,               // data_class: INTERNAL_ONLY
    pub source_bank_account_id: Classified<BankAccountId>,        // data_class: INTERNAL_ONLY
    pub target_bank_account_id: Classified<BankAccountId>,        // data_class: INTERNAL_ONLY
    pub transfer_amount: Classified<MoneyAmount>,                 // data_class: FINANCIAL
    pub source_closing_balance: Classified<MoneyAmount>,          // data_class: FINANCIAL
    pub source_minimum_balance: Classified<MoneyAmount>,          // data_class: FINANCIAL
    pub target_closing_balance: Classified<MoneyAmount>,          // data_class: FINANCIAL
    pub target_minimum_balance: Classified<MoneyAmount>,          // data_class: FINANCIAL
    pub source_surplus_after_transfer: Classified<MoneyAmount>,   // data_class: FINANCIAL
    pub target_shortfall_before_transfer: Classified<MoneyAmount>, // data_class: FINANCIAL
    pub cash_pool_ref: Classified<SourceDocumentRef>,             // data_class: INTERNAL_ONLY
    pub approval_policy_evidence_ref: Classified<EvidenceRef>,    // data_class: INTERNAL_ONLY
    pub proposal_evidence_ref: Classified<EvidenceRef>,           // data_class: INTERNAL_ONLY
    pub state: Classified<CashTransferProposalState>,             // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,                      // data_class: INTERNAL_ONLY
    pub payment_execution_attached: Classified<bool>,             // data_class: PUBLIC
    pub bank_network_call_attached: Classified<bool>,             // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,              // data_class: PUBLIC
    pub schema_version: Classified<u32>,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreasuryDomainError {
    InvalidBankAccountId,
    InvalidBankId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidCashPositionId,
    InvalidLiquidityForecastId,
    InvalidCashTransferProposalId,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidMoney,
    InvalidTimestamp,
    InvalidPositionDate,
    InvalidForecastHorizon,
    BankAccountApprovalRequired,
    CashPositionRequired,
    CurrencyMismatch,
    SameBankAccountTransfer,
    InsufficientSourceSurplus,
    NoTargetLiquidityNeed,
}

pub fn approve_bank_account(
    input: BankAccountApprovalInput,
) -> Result<BankAccountApproval, TreasuryDomainError> {
    validate_bank_account_id(&input.bank_account_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_bank_id(&input.bank_id)?;
    validate_currency_code(&input.currency)?;
    validate_source_ref(&input.bank_account_master_ref)?;
    validate_evidence_ref(&input.opening_balance_evidence_ref)?;
    validate_evidence_ref(&input.control_evidence_ref)?;
    validate_non_negative_money(&input.target_balance)?;
    ensure_currency(&input.target_balance, &input.currency)?;
    if input.approved_at_epoch_seconds == 0 {
        return Err(TreasuryDomainError::InvalidTimestamp);
    }

    let idempotency_key = format!(
        "treasury:bank-account:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.bank_account_id
    );

    Ok(BankAccountApproval {
        bank_account_id: internal(BankAccountId {
            value: input.bank_account_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        bank_id: internal(BankId {
            value: input.bank_id,
        }),
        currency: financial(input.currency),
        bank_account_master_ref: internal(SourceDocumentRef {
            value: input.bank_account_master_ref,
        }),
        opening_balance_evidence_ref: internal(EvidenceRef {
            value: input.opening_balance_evidence_ref,
        }),
        control_evidence_ref: internal(EvidenceRef {
            value: input.control_evidence_ref,
        }),
        target_balance: financial(input.target_balance),
        state: internal(BankAccountState::Approved),
        idempotency_key: internal(idempotency_key),
        approved_at_epoch_seconds: internal(input.approved_at_epoch_seconds),
        schema_version: public(TREASURY_SCHEMA_VERSION),
    })
}

pub fn record_cash_position(
    input: CashPositionInput,
) -> Result<CashPositionSnapshot, TreasuryDomainError> {
    validate_cash_position_id(&input.cash_position_id)?;
    validate_bank_account_id(&input.bank_account_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    if !input.bank_account_approved {
        return Err(TreasuryDomainError::BankAccountApprovalRequired);
    }
    validate_yyyymmdd(input.position_date_yyyymmdd)?;
    validate_money_currency(&input.opening_balance)?;
    validate_non_negative_money(&input.actual_inflow)?;
    validate_non_negative_money(&input.actual_outflow)?;
    ensure_same_currency(&[
        &input.opening_balance,
        &input.actual_inflow,
        &input.actual_outflow,
    ])?;
    validate_evidence_ref(&input.bank_statement_evidence_ref)?;
    validate_source_ref(&input.exposure_flow_ref)?;

    let closing_balance = MoneyAmount {
        amount_minor: input.opening_balance.amount_minor + input.actual_inflow.amount_minor
            - input.actual_outflow.amount_minor,
        currency: input.opening_balance.currency.clone(),
    };
    let idempotency_key = format!(
        "treasury:cash-position:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.bank_account_id, input.position_date_yyyymmdd
    );

    Ok(CashPositionSnapshot {
        cash_position_id: internal(CashPositionId {
            value: input.cash_position_id,
        }),
        bank_account_id: internal(BankAccountId {
            value: input.bank_account_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        position_date_yyyymmdd: internal(input.position_date_yyyymmdd),
        opening_balance: financial(input.opening_balance),
        actual_inflow: financial(input.actual_inflow),
        actual_outflow: financial(input.actual_outflow),
        closing_balance: financial(closing_balance),
        bank_statement_evidence_ref: internal(EvidenceRef {
            value: input.bank_statement_evidence_ref,
        }),
        exposure_flow_ref: internal(SourceDocumentRef {
            value: input.exposure_flow_ref,
        }),
        state: internal(CashPositionState::Recorded),
        idempotency_key: internal(idempotency_key),
        payment_execution_attached: public(false),
        bank_network_call_attached: public(false),
        cloud_deployment_attached: public(false),
        runtime_audit_chain_emission_attached: public(false),
        schema_version: public(TREASURY_SCHEMA_VERSION),
    })
}

pub fn project_liquidity_forecast(
    input: LiquidityForecastInput,
) -> Result<LiquidityForecastProjection, TreasuryDomainError> {
    validate_liquidity_forecast_id(&input.forecast_id)?;
    validate_bank_account_id(&input.bank_account_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    if !input.cash_position_recorded {
        return Err(TreasuryDomainError::CashPositionRequired);
    }
    if !(1..=366).contains(&input.horizon_days) {
        return Err(TreasuryDomainError::InvalidForecastHorizon);
    }
    validate_money_currency(&input.starting_balance)?;
    validate_non_negative_money(&input.forecasted_inflow)?;
    validate_non_negative_money(&input.forecasted_outflow)?;
    validate_non_negative_money(&input.minimum_liquidity_target)?;
    ensure_same_currency(&[
        &input.starting_balance,
        &input.forecasted_inflow,
        &input.forecasted_outflow,
        &input.minimum_liquidity_target,
    ])?;
    validate_source_ref(&input.forecast_source_ref)?;
    validate_evidence_ref(&input.forecast_evidence_ref)?;

    let projected_closing_balance = MoneyAmount {
        amount_minor: input.starting_balance.amount_minor + input.forecasted_inflow.amount_minor
            - input.forecasted_outflow.amount_minor,
        currency: input.starting_balance.currency.clone(),
    };
    let shortfall_minor = (input.minimum_liquidity_target.amount_minor
        - projected_closing_balance.amount_minor)
        .max(0);
    let shortfall_amount = MoneyAmount {
        amount_minor: shortfall_minor,
        currency: input.starting_balance.currency.clone(),
    };
    let liquidity_breach = shortfall_minor > 0;
    let idempotency_key = format!(
        "treasury:liquidity-forecast:{}:{}:{}:{}d",
        input.tenant_id, input.legal_entity_id, input.bank_account_id, input.horizon_days
    );

    Ok(LiquidityForecastProjection {
        forecast_id: internal(LiquidityForecastId {
            value: input.forecast_id,
        }),
        bank_account_id: internal(BankAccountId {
            value: input.bank_account_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        horizon_days: internal(input.horizon_days),
        starting_balance: financial(input.starting_balance),
        forecasted_inflow: financial(input.forecasted_inflow),
        forecasted_outflow: financial(input.forecasted_outflow),
        minimum_liquidity_target: financial(input.minimum_liquidity_target),
        projected_closing_balance: financial(projected_closing_balance),
        shortfall_amount: financial(shortfall_amount),
        liquidity_breach: public(liquidity_breach),
        forecast_source_ref: internal(SourceDocumentRef {
            value: input.forecast_source_ref,
        }),
        forecast_evidence_ref: internal(EvidenceRef {
            value: input.forecast_evidence_ref,
        }),
        state: internal(LiquidityForecastState::Projected),
        idempotency_key: internal(idempotency_key),
        payment_execution_attached: public(false),
        bank_network_call_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(TREASURY_SCHEMA_VERSION),
    })
}

pub fn propose_cash_transfer(
    input: CashTransferProposalInput,
) -> Result<CashTransferProposal, TreasuryDomainError> {
    validate_cash_transfer_proposal_id(&input.transfer_proposal_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_bank_account_id(&input.source_bank_account_id)?;
    validate_bank_account_id(&input.target_bank_account_id)?;
    if input.source_bank_account_id == input.target_bank_account_id {
        return Err(TreasuryDomainError::SameBankAccountTransfer);
    }
    validate_positive_money(&input.transfer_amount)?;
    validate_money_currency(&input.source_closing_balance)?;
    validate_non_negative_money(&input.source_minimum_balance)?;
    validate_money_currency(&input.target_closing_balance)?;
    validate_non_negative_money(&input.target_minimum_balance)?;
    ensure_same_currency(&[
        &input.transfer_amount,
        &input.source_closing_balance,
        &input.source_minimum_balance,
        &input.target_closing_balance,
        &input.target_minimum_balance,
    ])?;
    validate_source_ref(&input.cash_pool_ref)?;
    validate_evidence_ref(&input.approval_policy_evidence_ref)?;
    validate_evidence_ref(&input.proposal_evidence_ref)?;

    let source_surplus_after_transfer_minor = input.source_closing_balance.amount_minor
        - input.transfer_amount.amount_minor
        - input.source_minimum_balance.amount_minor;
    if source_surplus_after_transfer_minor < 0 {
        return Err(TreasuryDomainError::InsufficientSourceSurplus);
    }
    let target_shortfall_before_transfer_minor =
        input.target_minimum_balance.amount_minor - input.target_closing_balance.amount_minor;
    if target_shortfall_before_transfer_minor <= 0 {
        return Err(TreasuryDomainError::NoTargetLiquidityNeed);
    }

    let source_surplus_after_transfer = MoneyAmount {
        amount_minor: source_surplus_after_transfer_minor,
        currency: input.transfer_amount.currency.clone(),
    };
    let target_shortfall_before_transfer = MoneyAmount {
        amount_minor: target_shortfall_before_transfer_minor,
        currency: input.transfer_amount.currency.clone(),
    };
    let idempotency_key = format!(
        "treasury:cash-transfer:{}:{}:{}:{}",
        input.tenant_id,
        input.legal_entity_id,
        input.source_bank_account_id,
        input.target_bank_account_id
    );

    Ok(CashTransferProposal {
        transfer_proposal_id: internal(CashTransferProposalId {
            value: input.transfer_proposal_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        source_bank_account_id: internal(BankAccountId {
            value: input.source_bank_account_id,
        }),
        target_bank_account_id: internal(BankAccountId {
            value: input.target_bank_account_id,
        }),
        transfer_amount: financial(input.transfer_amount),
        source_closing_balance: financial(input.source_closing_balance),
        source_minimum_balance: financial(input.source_minimum_balance),
        target_closing_balance: financial(input.target_closing_balance),
        target_minimum_balance: financial(input.target_minimum_balance),
        source_surplus_after_transfer: financial(source_surplus_after_transfer),
        target_shortfall_before_transfer: financial(target_shortfall_before_transfer),
        cash_pool_ref: internal(SourceDocumentRef {
            value: input.cash_pool_ref,
        }),
        approval_policy_evidence_ref: internal(EvidenceRef {
            value: input.approval_policy_evidence_ref,
        }),
        proposal_evidence_ref: internal(EvidenceRef {
            value: input.proposal_evidence_ref,
        }),
        state: internal(CashTransferProposalState::Proposed),
        idempotency_key: internal(idempotency_key),
        payment_execution_attached: public(false),
        bank_network_call_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(TREASURY_SCHEMA_VERSION),
    })
}

fn validate_bank_account_id(value: &str) -> Result<(), TreasuryDomainError> {
    validate_prefixed_identifier(
        value,
        BANK_ACCOUNT_ID_PREFIX,
        TreasuryDomainError::InvalidBankAccountId,
    )
}

fn validate_bank_id(value: &str) -> Result<(), TreasuryDomainError> {
    validate_prefixed_identifier(value, BANK_ID_PREFIX, TreasuryDomainError::InvalidBankId)
}

fn validate_tenant_id(value: &str) -> Result<(), TreasuryDomainError> {
    validate_prefixed_identifier(
        value,
        TENANT_ID_PREFIX,
        TreasuryDomainError::InvalidTenantId,
    )
}

fn validate_legal_entity_id(value: &str) -> Result<(), TreasuryDomainError> {
    validate_prefixed_identifier(
        value,
        LEGAL_ENTITY_ID_PREFIX,
        TreasuryDomainError::InvalidLegalEntityId,
    )
}

fn validate_cash_position_id(value: &str) -> Result<(), TreasuryDomainError> {
    validate_prefixed_identifier(
        value,
        CASH_POSITION_ID_PREFIX,
        TreasuryDomainError::InvalidCashPositionId,
    )
}

fn validate_liquidity_forecast_id(value: &str) -> Result<(), TreasuryDomainError> {
    validate_prefixed_identifier(
        value,
        LIQUIDITY_FORECAST_ID_PREFIX,
        TreasuryDomainError::InvalidLiquidityForecastId,
    )
}

fn validate_cash_transfer_proposal_id(value: &str) -> Result<(), TreasuryDomainError> {
    validate_prefixed_identifier(
        value,
        CASH_TRANSFER_PROPOSAL_ID_PREFIX,
        TreasuryDomainError::InvalidCashTransferProposalId,
    )
}

fn validate_prefixed_identifier(
    value: &str,
    prefix: &str,
    error: TreasuryDomainError,
) -> Result<(), TreasuryDomainError> {
    if value == prefix
        || !value.starts_with(prefix)
        || has_unsafe_text(value)
        || value.contains('/')
        || value.contains("..")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_source_ref(value: &str) -> Result<(), TreasuryDomainError> {
    validate_ref(
        value,
        SOURCE_REF_PREFIX,
        TreasuryDomainError::InvalidSourceDocumentRef,
    )
}

fn validate_ref(
    value: &str,
    prefix: &str,
    error: TreasuryDomainError,
) -> Result<(), TreasuryDomainError> {
    if value == prefix
        || !value.starts_with(prefix)
        || has_unsafe_text(value)
        || value.contains("..")
    {
        return Err(error);
    }
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("token")
        || lowered.contains("secret")
        || lowered.contains("bearer")
        || lowered.contains("password")
        || lowered.contains("api-key")
        || lowered.contains("apikey")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_evidence_ref(value: &str) -> Result<(), TreasuryDomainError> {
    validate_ref(
        value,
        AUDIT_REF_PREFIX,
        TreasuryDomainError::InvalidEvidenceRef,
    )
}

fn validate_money_currency(amount: &MoneyAmount) -> Result<(), TreasuryDomainError> {
    validate_currency_code(&amount.currency)
}

fn validate_non_negative_money(amount: &MoneyAmount) -> Result<(), TreasuryDomainError> {
    validate_money_currency(amount)?;
    if amount.amount_minor < 0 {
        return Err(TreasuryDomainError::InvalidMoney);
    }
    Ok(())
}

fn validate_positive_money(amount: &MoneyAmount) -> Result<(), TreasuryDomainError> {
    validate_money_currency(amount)?;
    if amount.amount_minor <= 0 {
        return Err(TreasuryDomainError::InvalidMoney);
    }
    Ok(())
}

fn validate_currency_code(value: &str) -> Result<(), TreasuryDomainError> {
    if value.len() != 3
        || has_unsafe_text(value)
        || !value.chars().all(|ch| ch.is_ascii_uppercase())
    {
        return Err(TreasuryDomainError::InvalidMoney);
    }
    Ok(())
}

fn ensure_currency(amount: &MoneyAmount, currency: &str) -> Result<(), TreasuryDomainError> {
    if amount.currency != currency {
        return Err(TreasuryDomainError::CurrencyMismatch);
    }
    Ok(())
}

fn ensure_same_currency(amounts: &[&MoneyAmount]) -> Result<(), TreasuryDomainError> {
    let Some(first) = amounts.first() else {
        return Ok(());
    };
    for amount in &amounts[1..] {
        if amount.currency != first.currency {
            return Err(TreasuryDomainError::CurrencyMismatch);
        }
    }
    Ok(())
}

fn validate_yyyymmdd(value: u32) -> Result<(), TreasuryDomainError> {
    let year = value / 10_000;
    let month = (value / 100) % 100;
    let day = value % 100;
    if !(2020..=2100).contains(&year) || !(1..=12).contains(&month) {
        return Err(TreasuryDomainError::InvalidPositionDate);
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => return Err(TreasuryDomainError::InvalidPositionDate),
    };
    if day == 0 || day > max_day {
        return Err(TreasuryDomainError::InvalidPositionDate);
    }
    Ok(())
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
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
