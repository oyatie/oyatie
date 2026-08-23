//! Cloud billing event kernel.
//!
//! Cloud billing owns cloud-resource billing events and records them through the
//! platform metering kernel so FinOps, tax, marketplace, and tenant billing all
//! consume one metering path.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use billing_metering::{
    AxisId, Meter, MeterEvent, MeterEventCreate, MeterUnit, MeteringError, PlaneTag,
};
use cell_region::RegionCode;
use compute_resource::ResourceId;
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const BILLING_ACCOUNT_SCHEMA_VERSION: u32 = 1;
const CLOUD_BILLING_EVENT_SCHEMA_VERSION: u32 = 1;
const CLOUD_INVOICE_SCHEMA_VERSION: u32 = 1;
const CLOUD_BILLING_TENANT_GUARDRAIL_SCHEMA_VERSION: u32 = 1;
const BILLING_ACCOUNT_ID_PREFIX: &str = "ba_";
const CLOUD_BILLING_EVENT_ID_PREFIX: &str = "cbill_";
const INVOICE_ID_PREFIX: &str = "inv_";
const INVOICE_LINE_ITEM_ID_PREFIX: &str = "ili_";
const TENANT_ID_PREFIX: &str = "ten_";
const PAYMENT_METHOD_PREFIX: &str = "pm_";
const RATE_CARD_PREFIX: &str = "rate/";
const REGIONAL_PACK_PREFIX: &str = "pack-";
const TAX_INVOICE_FORMAT_PREFIX: &str = "tax-format/";
const TAX_REGISTRATION_ID_PREFIX: &str = "tax-registration/";
pub const BILLING_METERING_EVIDENCE_PREFIX: &str = "evidence/billing/metering/";
pub const BILLING_INVOICE_EVIDENCE_PREFIX: &str = "evidence/billing/invoice/";
pub const BILLING_TAX_EVIDENCE_PREFIX: &str = "evidence/billing/tax/";
pub const BILLING_AUDIT_CHAIN_PREFIX: &str = "audit-chain/billing/";
pub const BILLING_DEMO_TRIAL_CAP_EVIDENCE_PREFIX: &str = "evidence/billing/demo-trial-cap/";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BillingAccountId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudBillingEventId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InvoiceId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InvoiceLineItemId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TaxRegistrationId {
    pub value: String, // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PaymentMethodRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RateCardRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CurrencyCode {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BillingAccountState {
    Active,
    Suspended,
    Delinquent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TenantClass {
    DemoTrial,
    Paid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BillingComponent {
    RevenueShare,
    PerSeat,
    PerUsage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudBillingEventKind {
    ResourceCreated,
    ResourceTerminated,
    Usage,
    Reservation,
    Commitment,
    Credit,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Money {
    pub currency: CurrencyCode, // data_class: INTERNAL_ONLY
    pub minor_units: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TaxInvoiceFormat {
    ElectronicTaxInvoice,
    QualifiedTaxInvoice,
    CountryEInvoice,
    GstTaxInvoice,
    FiscalDocumentInvoice,
    ClearanceQrInvoice,
    VatRegistrationInvoice,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InvoiceState {
    Issued,
    Paid,
    Overdue,
    Void,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BillingPeriod {
    pub start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceLineItemCreate {
    pub id: String,            // data_class: INTERNAL_ONLY
    pub resource_id: String,   // data_class: INTERNAL_ONLY
    pub description: String,   // data_class: INTERNAL_ONLY
    pub units: Vec<MeterUnit>, // data_class: INTERNAL_ONLY
    pub subtotal: Money,       // data_class: INTERNAL_ONLY
    pub data_class: DataClass, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceLineItem {
    pub id: Classified<InvoiceLineItemId>, // data_class: INTERNAL_ONLY
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub description: Classified<String>,   // data_class: INTERNAL_ONLY
    pub units: Classified<Vec<MeterUnit>>, // data_class: INTERNAL_ONLY
    pub subtotal: Classified<Money>,       // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceGenerate {
    pub id: String,                             // data_class: INTERNAL_ONLY
    pub billing_account_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub regional_pack: String,                  // data_class: INTERNAL_ONLY
    pub period: BillingPeriod,                  // data_class: INTERNAL_ONLY
    pub line_items: Vec<InvoiceLineItemCreate>, // data_class: INTERNAL_ONLY
    pub subtotal: Money,                        // data_class: INTERNAL_ONLY
    pub tax: Money,                             // data_class: INTERNAL_ONLY
    pub total: Money,                           // data_class: INTERNAL_ONLY
    pub tax_invoice_format: TaxInvoiceFormat,   // data_class: INTERNAL_ONLY
    pub tax_registration_id: String,            // data_class: FINANCIAL
    pub issued_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub due_at_epoch_seconds: u64,              // data_class: INTERNAL_ONLY
    pub data_class: DataClass,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invoice {
    pub id: Classified<InvoiceId>, // data_class: INTERNAL_ONLY
    pub billing_account_id: Classified<BillingAccountId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub regional_pack: Classified<String>, // data_class: INTERNAL_ONLY
    pub period: Classified<BillingPeriod>, // data_class: INTERNAL_ONLY
    pub line_items: Classified<Vec<InvoiceLineItem>>, // data_class: INTERNAL_ONLY
    pub subtotal: Classified<Money>, // data_class: INTERNAL_ONLY
    pub tax: Classified<Money>,    // data_class: INTERNAL_ONLY
    pub total: Classified<Money>,  // data_class: INTERNAL_ONLY
    pub tax_invoice_format: Classified<TaxInvoiceFormat>, // data_class: INTERNAL_ONLY
    pub tax_registration_id: Classified<TaxRegistrationId>, // data_class: FINANCIAL
    pub state: Classified<InvoiceState>, // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub due_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreditNoteCreate {
    pub invoice_id: String,      // data_class: INTERNAL_ONLY
    pub line_item_id: String,    // data_class: INTERNAL_ONLY
    pub resource_id: String,     // data_class: INTERNAL_ONLY
    pub description: String,     // data_class: INTERNAL_ONLY
    pub units: Vec<MeterUnit>,   // data_class: INTERNAL_ONLY
    pub credit_minor_units: u64, // data_class: INTERNAL_ONLY
    pub currency: String,        // data_class: INTERNAL_ONLY
    pub data_class: DataClass,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingAccountCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub regional_pack: String,         // data_class: INTERNAL_ONLY
    pub payment_method: String,        // data_class: INTERNAL_ONLY
    pub credit_balance: Money,         // data_class: INTERNAL_ONLY
    pub state: BillingAccountState,    // data_class: INTERNAL_ONLY
    pub data_class: DataClass,         // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingAccount {
    pub id: Classified<BillingAccountId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,   // data_class: PUBLIC
    pub regional_pack: Classified<String>, // data_class: INTERNAL_ONLY
    pub payment_method: Classified<PaymentMethodRef>, // data_class: INTERNAL_ONLY
    pub credit_balance: Classified<Money>, // data_class: INTERNAL_ONLY
    pub state: Classified<BillingAccountState>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTenantGuardrailCreate {
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub region: String,                              // data_class: PUBLIC
    pub billing_account_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_class: TenantClass,                   // data_class: INTERNAL_ONLY
    pub billing_components: Vec<BillingComponent>,   // data_class: INTERNAL_ONLY
    pub regional_pack: String,                       // data_class: INTERNAL_ONLY
    pub tax_invoice_format: TaxInvoiceFormat,        // data_class: INTERNAL_ONLY
    pub rate_card_ref: String,                       // data_class: INTERNAL_ONLY
    pub invoice_id: String,                          // data_class: INTERNAL_ONLY
    pub billing_period: BillingPeriod,               // data_class: INTERNAL_ONLY
    pub metering_evidence_refs: Vec<String>,         // data_class: AUDIT
    pub invoice_evidence_ref: String,                // data_class: AUDIT
    pub tax_evidence_ref: String,                    // data_class: AUDIT
    pub audit_chain_ref: String,                     // data_class: AUDIT
    pub demo_trial_cap_evidence_ref: Option<String>, // data_class: AUDIT
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTenantGuardrail {
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub billing_account_id: Classified<BillingAccountId>, // data_class: INTERNAL_ONLY
    pub tenant_class: Classified<TenantClass>, // data_class: INTERNAL_ONLY
    pub billing_components: Classified<Vec<BillingComponent>>, // data_class: INTERNAL_ONLY
    pub regional_pack: Classified<String>, // data_class: INTERNAL_ONLY
    pub tax_invoice_format: Classified<TaxInvoiceFormat>, // data_class: INTERNAL_ONLY
    pub rate_card_ref: Classified<RateCardRef>, // data_class: INTERNAL_ONLY
    pub invoice_id: Classified<InvoiceId>, // data_class: INTERNAL_ONLY
    pub billing_period: Classified<BillingPeriod>, // data_class: INTERNAL_ONLY
    pub metering_evidence_refs: Classified<Vec<String>>, // data_class: AUDIT
    pub invoice_evidence_ref: Classified<String>, // data_class: AUDIT
    pub tax_evidence_ref: Classified<String>, // data_class: AUDIT
    pub audit_chain_ref: Classified<String>, // data_class: AUDIT
    pub demo_trial_cap_evidence_ref: Classified<Option<String>>, // data_class: AUDIT
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingEventCreate {
    pub id: String,                     // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub resource_id: String,            // data_class: INTERNAL_ONLY
    pub region: String,                 // data_class: PUBLIC
    pub metering_tag: String,           // data_class: INTERNAL_ONLY
    pub kind: CloudBillingEventKind,    // data_class: INTERNAL_ONLY
    pub units: Vec<MeterUnit>,          // data_class: INTERNAL_ONLY
    pub rate_card_ref: String,          // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub data_class: DataClass,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingEvent {
    pub id: Classified<CloudBillingEventId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub metering_tag: Classified<String>,    // data_class: INTERNAL_ONLY
    pub kind: Classified<CloudBillingEventKind>, // data_class: INTERNAL_ONLY
    pub units: Classified<Vec<MeterUnit>>,   // data_class: INTERNAL_ONLY
    pub rate_card_ref: Classified<RateCardRef>, // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudBillingError {
    InvalidBillingAccountId,
    InvalidCloudBillingEventId,
    InvalidInvoiceId,
    InvalidInvoiceLineItemId,
    InvalidTaxRegistrationId,
    InvalidTenantId,
    InvalidPaymentMethodRef,
    InvalidRateCardRef,
    InvalidRegionalPack,
    InvalidCurrencyCode,
    InvalidResourceId,
    InvalidMeteringTag,
    InvalidOccurredAt,
    InvalidBillingPeriod,
    InvalidInvoiceLineItem,
    InvalidInvoiceTotal,
    InvalidTaxInvoiceFormat,
    InvalidTenantClassPolicy,
    InvalidBillingComponentPolicy,
    InvalidBillingEvidenceRef,
    InvalidAuditChainRef,
    InvalidDataClass,
    BillingAccountInactive,
    TenantMismatch,
    RegionMismatch,
    DuplicateBillingEvent,
    DuplicateInvoice,
    MeteringRejected(MeteringError),
    InvoiceNotFound,
    IllegalInvoiceTransition {
        from: InvoiceState,
        to: InvoiceState,
    },
    CreditNoteOverCredit,
    CreditNoteTargetVoid,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudBillingLedger {
    events_by_id: BTreeMap<CloudBillingEventId, CloudBillingEvent>,
    events_by_idempotency: BTreeMap<String, CloudBillingEventId>, // data_class: INTERNAL_ONLY
    invoices_by_id: BTreeMap<InvoiceId, Invoice>,
}

impl BillingAccountId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudBillingError> {
        prefixed_id(
            value.into(),
            BILLING_ACCOUNT_ID_PREFIX,
            CloudBillingError::InvalidBillingAccountId,
        )
        .map(|value| Self { value })
    }
}

impl CloudBillingEventId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudBillingError> {
        prefixed_id(
            value.into(),
            CLOUD_BILLING_EVENT_ID_PREFIX,
            CloudBillingError::InvalidCloudBillingEventId,
        )
        .map(|value| Self { value })
    }
}

impl InvoiceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudBillingError> {
        prefixed_id(
            value.into(),
            INVOICE_ID_PREFIX,
            CloudBillingError::InvalidInvoiceId,
        )
        .map(|value| Self { value })
    }
}

impl InvoiceLineItemId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudBillingError> {
        prefixed_id(
            value.into(),
            INVOICE_LINE_ITEM_ID_PREFIX,
            CloudBillingError::InvalidInvoiceLineItemId,
        )
        .map(|value| Self { value })
    }
}

impl TaxRegistrationId {
    pub fn new(
        value: impl Into<String>,
        format: TaxInvoiceFormat,
    ) -> Result<Self, CloudBillingError> {
        let value = value.into();
        let valid = match format {
            TaxInvoiceFormat::ElectronicTaxInvoice => value
                .strip_prefix("taxid/electronic/")
                .is_some_and(|id| id.len() == 10 && id.bytes().all(|byte| byte.is_ascii_digit())),
            TaxInvoiceFormat::QualifiedTaxInvoice => value
                .strip_prefix("taxid/qualified/T")
                .is_some_and(|id| id.len() == 13 && id.bytes().all(|byte| byte.is_ascii_digit())),
            TaxInvoiceFormat::CountryEInvoice => value
                .strip_prefix("taxid/vat/")
                .is_some_and(|id| id.len() >= 8 && is_ascii_token(id)),
            TaxInvoiceFormat::GstTaxInvoice => value.strip_prefix("taxid/gst/").is_some_and(|id| {
                id.len() == 15 && id.bytes().all(|byte| byte.is_ascii_alphanumeric())
            }),
            TaxInvoiceFormat::FiscalDocumentInvoice => value
                .strip_prefix("taxid/fiscal/")
                .is_some_and(|id| id.len() == 14 && id.bytes().all(|byte| byte.is_ascii_digit())),
            TaxInvoiceFormat::ClearanceQrInvoice => value
                .strip_prefix("taxid/clearance/")
                .is_some_and(|id| id.len() == 15 && id.bytes().all(|byte| byte.is_ascii_digit())),
            TaxInvoiceFormat::VatRegistrationInvoice => value
                .strip_prefix("taxid/registration/")
                .is_some_and(|id| id.len() == 15 && id.bytes().all(|byte| byte.is_ascii_digit())),
        };
        if valid {
            Ok(Self { value })
        } else {
            Err(CloudBillingError::InvalidTaxRegistrationId)
        }
    }
}

impl PaymentMethodRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudBillingError> {
        prefixed_id(
            value.into(),
            PAYMENT_METHOD_PREFIX,
            CloudBillingError::InvalidPaymentMethodRef,
        )
        .map(|value| Self { value })
    }
}

impl RateCardRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudBillingError> {
        prefixed_id(
            value.into(),
            RATE_CARD_PREFIX,
            CloudBillingError::InvalidRateCardRef,
        )
        .map(|value| Self { value })
    }
}

impl CurrencyCode {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudBillingError> {
        let value = value.into();
        if value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase()) {
            Ok(Self { value })
        } else {
            Err(CloudBillingError::InvalidCurrencyCode)
        }
    }
}

impl Money {
    pub fn new(currency: impl Into<String>, minor_units: u64) -> Result<Self, CloudBillingError> {
        Ok(Self {
            currency: CurrencyCode::new(currency)?,
            minor_units,
        })
    }

    fn checked_add(&self, other: &Self) -> Result<Self, CloudBillingError> {
        if self.currency != other.currency {
            return Err(CloudBillingError::InvalidInvoiceTotal);
        }
        Ok(Self {
            currency: self.currency.clone(),
            minor_units: self
                .minor_units
                .checked_add(other.minor_units)
                .ok_or(CloudBillingError::InvalidInvoiceTotal)?,
        })
    }

    fn checked_sub(&self, amount: u64) -> Result<Self, CloudBillingError> {
        Ok(Self {
            currency: self.currency.clone(),
            minor_units: self
                .minor_units
                .checked_sub(amount)
                .ok_or(CloudBillingError::InvalidInvoiceTotal)?,
        })
    }
}

impl TenantClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::DemoTrial => "demo_trial",
            Self::Paid => "paid",
        }
    }
}

impl BillingComponent {
    pub const fn label(self) -> &'static str {
        match self {
            Self::RevenueShare => "revenue_share",
            Self::PerSeat => "per_seat",
            Self::PerUsage => "per_usage",
        }
    }
}

pub const fn tenant_class_label(value: TenantClass) -> &'static str {
    value.label()
}

pub const fn billing_component_label(value: BillingComponent) -> &'static str {
    value.label()
}

impl TaxInvoiceFormat {
    pub fn for_regional_pack(value: &str) -> Result<Self, CloudBillingError> {
        match value {
            "pack-electronic-tax" => Ok(Self::ElectronicTaxInvoice),
            "pack-qualified-tax" => Ok(Self::QualifiedTaxInvoice),
            "pack-country-tax" | "pack-market-tax" | "pack-trade-tax" | "pack-vat-tax" => {
                Ok(Self::CountryEInvoice)
            }
            "pack-gst-tax" => Ok(Self::GstTaxInvoice),
            "pack-fiscal-tax" => Ok(Self::FiscalDocumentInvoice),
            "pack-clearance-tax" => Ok(Self::ClearanceQrInvoice),
            "pack-registration-tax" => Ok(Self::VatRegistrationInvoice),
            _ => Err(CloudBillingError::InvalidRegionalPack),
        }
    }
}

impl BillingPeriod {
    pub fn new(
        start_epoch_seconds: u64,
        end_epoch_seconds: u64,
    ) -> Result<Self, CloudBillingError> {
        if start_epoch_seconds == 0 || end_epoch_seconds <= start_epoch_seconds {
            return Err(CloudBillingError::InvalidBillingPeriod);
        }
        Ok(Self {
            start_epoch_seconds,
            end_epoch_seconds,
        })
    }
}

impl InvoiceLineItem {
    pub fn new(tenant_id: &str, input: InvoiceLineItemCreate) -> Result<Self, CloudBillingError> {
        let id = InvoiceLineItemId::new(input.id)?;
        let resource_id =
            ResourceId::new(input.resource_id).map_err(|_| CloudBillingError::InvalidResourceId)?;
        if resource_id
            .tenant_id()
            .map_err(|_| CloudBillingError::InvalidResourceId)?
            != tenant_id
        {
            return Err(CloudBillingError::TenantMismatch);
        }
        if input.description.trim().is_empty()
            || input.description.len() > 160
            || input.units.is_empty()
            || input.subtotal.minor_units == 0
        {
            return Err(CloudBillingError::InvalidInvoiceLineItem);
        }
        Ok(Self {
            id: internal(id),
            resource_id: internal(resource_id),
            description: internal(input.description),
            units: internal(input.units),
            subtotal: internal(input.subtotal),
            data_class: internal(financial_data_class(input.data_class)?),
        })
    }
}

impl Invoice {
    pub fn generate(
        account: &BillingAccount,
        input: InvoiceGenerate,
    ) -> Result<Self, CloudBillingError> {
        let id = InvoiceId::new(input.id)?;
        let billing_account_id = BillingAccountId::new(input.billing_account_id)?;
        if billing_account_id != account.id.value {
            return Err(CloudBillingError::InvalidBillingAccountId);
        }
        validate_tenant_id(&input.tenant_id)?;
        if input.tenant_id != account.tenant_id.value {
            return Err(CloudBillingError::TenantMismatch);
        }
        validate_regional_pack(&input.regional_pack)?;
        if input.regional_pack != account.regional_pack.value {
            return Err(CloudBillingError::InvalidRegionalPack);
        }
        if account.state.value != BillingAccountState::Active {
            return Err(CloudBillingError::BillingAccountInactive);
        }
        if input.due_at_epoch_seconds <= input.issued_at_epoch_seconds
            || input.issued_at_epoch_seconds < input.period.end_epoch_seconds
        {
            return Err(CloudBillingError::InvalidBillingPeriod);
        }
        let expected_format = TaxInvoiceFormat::for_regional_pack(&input.regional_pack)?;
        if input.tax_invoice_format != expected_format {
            return Err(CloudBillingError::InvalidTaxInvoiceFormat);
        }
        let tax_registration_id =
            TaxRegistrationId::new(input.tax_registration_id, expected_format)?;
        let line_items = invoice_line_items(&input.tenant_id, input.line_items)?;
        let computed_subtotal = sum_line_items(&line_items)?;
        if computed_subtotal != input.subtotal
            || input.subtotal.checked_add(&input.tax)? != input.total
        {
            return Err(CloudBillingError::InvalidInvoiceTotal);
        }
        Ok(Self {
            id: internal(id),
            billing_account_id: internal(billing_account_id),
            tenant_id: internal(input.tenant_id),
            regional_pack: internal(input.regional_pack),
            period: internal(input.period),
            line_items: internal(line_items),
            subtotal: internal(input.subtotal),
            tax: internal(input.tax),
            total: internal(input.total),
            tax_invoice_format: internal(input.tax_invoice_format),
            tax_registration_id: Classified::new(tax_registration_id, DataClass::Financial),
            state: internal(InvoiceState::Issued),
            issued_at_epoch_seconds: internal(input.issued_at_epoch_seconds),
            due_at_epoch_seconds: internal(input.due_at_epoch_seconds),
            data_class: internal(financial_data_class(input.data_class)?),
            schema_version: public(CLOUD_BILLING_TENANT_GUARDRAIL_SCHEMA_VERSION),
        })
    }
}

impl BillingAccount {
    pub fn new(input: BillingAccountCreate) -> Result<Self, CloudBillingError> {
        let id = BillingAccountId::new(input.id)?;
        validate_tenant_id(&input.tenant_id)?;
        let region =
            RegionCode::new(input.region).map_err(|_| CloudBillingError::RegionMismatch)?;
        validate_regional_pack(&input.regional_pack)?;
        let payment_method = PaymentMethodRef::new(input.payment_method)?;
        let data_class = financial_data_class(input.data_class)?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            regional_pack: internal(input.regional_pack),
            payment_method: internal(payment_method),
            credit_balance: internal(input.credit_balance),
            state: internal(input.state),
            data_class: internal(data_class),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(BILLING_ACCOUNT_SCHEMA_VERSION),
        })
    }
}

impl CloudBillingTenantGuardrail {
    pub fn new(input: CloudBillingTenantGuardrailCreate) -> Result<Self, CloudBillingError> {
        validate_tenant_id(&input.tenant_id)?;
        let region =
            RegionCode::new(input.region).map_err(|_| CloudBillingError::RegionMismatch)?;
        let billing_account_id = BillingAccountId::new(input.billing_account_id)?;
        let billing_components =
            validate_billing_components(input.tenant_class, input.billing_components)?;
        validate_regional_pack(&input.regional_pack)?;
        let expected_tax_invoice_format =
            TaxInvoiceFormat::for_regional_pack(&input.regional_pack)?;
        if input.tax_invoice_format != expected_tax_invoice_format {
            return Err(CloudBillingError::InvalidTaxInvoiceFormat);
        }
        let rate_card_ref = RateCardRef::new(input.rate_card_ref)?;
        let invoice_id = InvoiceId::new(input.invoice_id)?;
        let metering_evidence_refs = validate_evidence_refs(
            input.metering_evidence_refs,
            BILLING_METERING_EVIDENCE_PREFIX,
            &input.tenant_id,
            &region.value,
            CloudBillingError::InvalidBillingEvidenceRef,
        )?;
        let invoice_evidence_ref = validate_evidence_ref(
            input.invoice_evidence_ref,
            BILLING_INVOICE_EVIDENCE_PREFIX,
            &input.tenant_id,
            &region.value,
            CloudBillingError::InvalidBillingEvidenceRef,
        )?;
        let tax_evidence_ref = validate_evidence_ref(
            input.tax_evidence_ref,
            BILLING_TAX_EVIDENCE_PREFIX,
            &input.tenant_id,
            &region.value,
            CloudBillingError::InvalidBillingEvidenceRef,
        )?;
        let audit_chain_ref = validate_evidence_ref(
            input.audit_chain_ref,
            BILLING_AUDIT_CHAIN_PREFIX,
            &input.tenant_id,
            &region.value,
            CloudBillingError::InvalidAuditChainRef,
        )?;
        let demo_trial_cap_evidence_ref = validate_demo_trial_cap_evidence_ref(
            input.tenant_class,
            input.demo_trial_cap_evidence_ref,
            &input.tenant_id,
            &region.value,
        )?;
        Ok(Self {
            tenant_id: internal(input.tenant_id),
            region: public(region),
            billing_account_id: internal(billing_account_id),
            tenant_class: internal(input.tenant_class),
            billing_components: internal(billing_components),
            regional_pack: internal(input.regional_pack),
            tax_invoice_format: internal(input.tax_invoice_format),
            rate_card_ref: internal(rate_card_ref),
            invoice_id: internal(invoice_id),
            billing_period: internal(input.billing_period),
            metering_evidence_refs: audit(metering_evidence_refs),
            invoice_evidence_ref: audit(invoice_evidence_ref),
            tax_evidence_ref: audit(tax_evidence_ref),
            audit_chain_ref: audit(audit_chain_ref),
            demo_trial_cap_evidence_ref: audit(demo_trial_cap_evidence_ref),
            schema_version: public(CLOUD_INVOICE_SCHEMA_VERSION),
        })
    }
}

impl CloudBillingEvent {
    pub fn new(input: CloudBillingEventCreate) -> Result<Self, CloudBillingError> {
        let id = CloudBillingEventId::new(input.id)?;
        validate_tenant_id(&input.tenant_id)?;
        let resource_id =
            ResourceId::new(input.resource_id).map_err(|_| CloudBillingError::InvalidResourceId)?;
        let resource_tenant = resource_id
            .tenant_id()
            .map_err(|_| CloudBillingError::InvalidResourceId)?;
        if resource_tenant != input.tenant_id {
            return Err(CloudBillingError::TenantMismatch);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudBillingError::RegionMismatch)?;
        let resource_region = resource_id
            .region()
            .map_err(|_| CloudBillingError::InvalidResourceId)?;
        if resource_region != region {
            return Err(CloudBillingError::RegionMismatch);
        }
        validate_metering_tag(&input.metering_tag, &input.tenant_id, &resource_id)?;
        let rate_card_ref = RateCardRef::new(input.rate_card_ref)?;
        if input.occurred_at_epoch_seconds == 0 {
            return Err(CloudBillingError::InvalidOccurredAt);
        }
        let data_class = public_data_class(input.data_class)?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(input.tenant_id),
            resource_id: internal(resource_id),
            region: public(region),
            metering_tag: internal(input.metering_tag),
            kind: internal(input.kind),
            units: internal(input.units),
            rate_card_ref: internal(rate_card_ref),
            occurred_at_epoch_seconds: internal(input.occurred_at_epoch_seconds),
            idempotency_key: internal(input.idempotency_key),
            data_class: public(data_class),
            schema_version: public(CLOUD_BILLING_EVENT_SCHEMA_VERSION),
        })
    }

    pub fn to_meter_event_create(&self) -> MeterEventCreate {
        MeterEventCreate {
            id: format!(
                "mtr_{}",
                self.id
                    .value
                    .value
                    .trim_start_matches(CLOUD_BILLING_EVENT_ID_PREFIX)
            ),
            tenant_id: self.tenant_id.value.clone(),
            capability_id: self.kind.value.capability_id().to_string(),
            plane: PlaneTag::Data,
            units: self.units.value.clone(),
            source_axis: AxisId::Cloud,
            recorded_at_epoch_seconds: self.occurred_at_epoch_seconds.value,
            idempotency_key: self.idempotency_key.value.clone(),
            data_class: DataClass::Public,
        }
    }
}

impl CloudBillingEventKind {
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::ResourceCreated | Self::ResourceTerminated => {
                "cap.cloud.billing.resource-lifecycle"
            }
            Self::Usage => "cap.cloud.billing.usage",
            Self::Reservation => "cap.cloud.billing.reservation",
            Self::Commitment => "cap.cloud.billing.commitment",
            Self::Credit => "cap.cloud.billing.credit",
        }
    }
}

impl CloudBillingLedger {
    pub fn ingest(
        &mut self,
        meter: &mut Meter,
        input: CloudBillingEventCreate,
    ) -> Result<(CloudBillingEvent, MeterEvent), CloudBillingError> {
        let event = CloudBillingEvent::new(input)?;
        if let Some(existing_id) = self.events_by_idempotency.get(&event.idempotency_key.value) {
            let existing = self
                .events_by_id
                .get(existing_id)
                .cloned()
                .ok_or(CloudBillingError::DuplicateBillingEvent)?;
            let meter_event = meter
                .record(existing.to_meter_event_create())
                .map_err(CloudBillingError::MeteringRejected)?;
            return Ok((existing, meter_event));
        }
        if self.events_by_id.contains_key(&event.id.value) {
            return Err(CloudBillingError::DuplicateBillingEvent);
        }
        let meter_event = meter
            .record(event.to_meter_event_create())
            .map_err(CloudBillingError::MeteringRejected)?;
        self.events_by_id
            .insert(event.id.value.clone(), event.clone());
        self.events_by_idempotency
            .insert(event.idempotency_key.value.clone(), event.id.value.clone());
        Ok((event, meter_event))
    }

    pub fn events(&self) -> impl Iterator<Item = &CloudBillingEvent> {
        self.events_by_id.values()
    }

    pub fn generate_invoice(
        &mut self,
        account: &BillingAccount,
        input: InvoiceGenerate,
    ) -> Result<Invoice, CloudBillingError> {
        let invoice = Invoice::generate(account, input)?;
        if self.invoices_by_id.contains_key(&invoice.id.value) {
            return Err(CloudBillingError::DuplicateInvoice);
        }
        self.invoices_by_id
            .insert(invoice.id.value.clone(), invoice.clone());
        Ok(invoice)
    }

    pub fn invoices(&self) -> impl Iterator<Item = &Invoice> {
        self.invoices_by_id.values()
    }

    pub fn get_invoice(&self, id: &InvoiceId) -> Option<&Invoice> {
        self.invoices_by_id.get(id)
    }

    pub fn transition_invoice(
        &mut self,
        id: &InvoiceId,
        target: InvoiceState,
    ) -> Result<&Invoice, CloudBillingError> {
        let invoice = self
            .invoices_by_id
            .get_mut(id)
            .ok_or(CloudBillingError::InvoiceNotFound)?;
        let from = invoice.state.value;
        // Same-state re-transition is idempotent (billing de-dup doctrine).
        if from == target {
            return Ok(self.invoices_by_id.get(id).expect("just checked"));
        }
        let legal = matches!(
            (from, target),
            (InvoiceState::Issued, InvoiceState::Paid)
                | (InvoiceState::Issued, InvoiceState::Overdue)
                | (InvoiceState::Issued, InvoiceState::Void)
                | (InvoiceState::Overdue, InvoiceState::Paid)
                | (InvoiceState::Overdue, InvoiceState::Void)
        );
        if !legal {
            return Err(CloudBillingError::IllegalInvoiceTransition { from, to: target });
        }
        invoice.state = internal(target);
        Ok(self.invoices_by_id.get(id).expect("just inserted"))
    }

    pub fn apply_credit_note(
        &mut self,
        tenant_id: &str,
        input: CreditNoteCreate,
    ) -> Result<&Invoice, CloudBillingError> {
        let invoice_id = InvoiceId::new(input.invoice_id.clone())?;
        {
            let invoice = self
                .invoices_by_id
                .get(&invoice_id)
                .ok_or(CloudBillingError::InvoiceNotFound)?;
            if invoice.state.value == InvoiceState::Void {
                return Err(CloudBillingError::CreditNoteTargetVoid);
            }
            if invoice.subtotal.value.currency.value != input.currency {
                return Err(CloudBillingError::InvalidInvoiceTotal);
            }
            if input.credit_minor_units == 0 {
                return Err(CloudBillingError::InvalidInvoiceLineItem);
            }
            if input.credit_minor_units > invoice.subtotal.value.minor_units {
                return Err(CloudBillingError::CreditNoteOverCredit);
            }
        }
        let description = format!("[CREDIT] {}", input.description);
        let line_item = InvoiceLineItem::new(
            tenant_id,
            InvoiceLineItemCreate {
                id: input.line_item_id,
                resource_id: input.resource_id,
                description,
                units: input.units,
                subtotal: Money::new(input.currency.clone(), input.credit_minor_units)?,
                data_class: input.data_class,
            },
        )?;
        let invoice = self
            .invoices_by_id
            .get_mut(&invoice_id)
            .expect("checked above");
        let new_subtotal = invoice
            .subtotal
            .value
            .checked_sub(input.credit_minor_units)?;
        invoice.line_items.value.push(line_item);
        invoice.subtotal = internal(new_subtotal);
        Ok(self.invoices_by_id.get(&invoice_id).expect("just modified"))
    }
}

fn invoice_line_items(
    tenant_id: &str,
    input: Vec<InvoiceLineItemCreate>,
) -> Result<Vec<InvoiceLineItem>, CloudBillingError> {
    if input.is_empty() {
        return Err(CloudBillingError::InvalidInvoiceLineItem);
    }
    let mut line_items = Vec::with_capacity(input.len());
    for item in input {
        line_items.push(InvoiceLineItem::new(tenant_id, item)?);
    }
    Ok(line_items)
}

fn sum_line_items(line_items: &[InvoiceLineItem]) -> Result<Money, CloudBillingError> {
    let mut iter = line_items.iter();
    let Some(first) = iter.next() else {
        return Err(CloudBillingError::InvalidInvoiceLineItem);
    };
    let mut total = first.subtotal.value.clone();
    for item in iter {
        total = total.checked_add(&item.subtotal.value)?;
    }
    Ok(total)
}

fn validate_billing_components(
    tenant_class: TenantClass,
    components: Vec<BillingComponent>,
) -> Result<Vec<BillingComponent>, CloudBillingError> {
    match tenant_class {
        TenantClass::DemoTrial => {
            if components.is_empty() {
                Ok(components)
            } else {
                Err(CloudBillingError::InvalidBillingComponentPolicy)
            }
        }
        TenantClass::Paid => {
            if components.is_empty() {
                return Err(CloudBillingError::InvalidBillingComponentPolicy);
            }
            let mut seen = BTreeSet::new();
            for component in &components {
                if !seen.insert(*component) {
                    return Err(CloudBillingError::InvalidBillingComponentPolicy);
                }
            }
            Ok(components)
        }
    }
}

fn validate_demo_trial_cap_evidence_ref(
    tenant_class: TenantClass,
    value: Option<String>,
    tenant_id: &str,
    region: &str,
) -> Result<Option<String>, CloudBillingError> {
    match (tenant_class, value) {
        (TenantClass::DemoTrial, Some(value)) => validate_evidence_ref(
            value,
            BILLING_DEMO_TRIAL_CAP_EVIDENCE_PREFIX,
            tenant_id,
            region,
            CloudBillingError::InvalidBillingEvidenceRef,
        )
        .map(Some),
        (TenantClass::DemoTrial, None) => Err(CloudBillingError::InvalidTenantClassPolicy),
        (TenantClass::Paid, Some(_)) => Err(CloudBillingError::InvalidTenantClassPolicy),
        (TenantClass::Paid, None) => Ok(None),
    }
}

fn validate_evidence_refs(
    values: Vec<String>,
    prefix: &str,
    tenant_id: &str,
    region: &str,
    error: CloudBillingError,
) -> Result<Vec<String>, CloudBillingError> {
    if values.is_empty() {
        return Err(error);
    }
    let mut seen = BTreeSet::new();
    let mut validated = Vec::with_capacity(values.len());
    for value in values {
        let value = validate_evidence_ref(value, prefix, tenant_id, region, error.clone())?;
        if !seen.insert(value.clone()) {
            return Err(error);
        }
        validated.push(value);
    }
    Ok(validated)
}

fn validate_evidence_ref(
    value: String,
    prefix: &str,
    tenant_id: &str,
    region: &str,
    error: CloudBillingError,
) -> Result<String, CloudBillingError> {
    if !value.starts_with(prefix)
        || !is_safe_reference(&value)
        || contains_secret_marker(&value)
        || !reference_matches_tenant_region(&value, prefix, tenant_id, region)
    {
        Err(error)
    } else {
        Ok(value)
    }
}

fn validate_metering_tag(
    value: &str,
    tenant_id: &str,
    resource_id: &ResourceId,
) -> Result<(), CloudBillingError> {
    let kind = resource_id
        .kind_label()
        .map_err(|_| CloudBillingError::InvalidResourceId)?;
    let expected = format!("oya:metering:{tenant_id}:{kind}");
    if value == expected {
        Ok(())
    } else {
        Err(CloudBillingError::InvalidMeteringTag)
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudBillingError> {
    if let Some(segment) = value.strip_prefix(TENANT_ID_PREFIX)
        && is_canonical_tenant_segment(segment)
    {
        Ok(())
    } else {
        Err(CloudBillingError::InvalidTenantId)
    }
}

fn validate_regional_pack(value: &str) -> Result<(), CloudBillingError> {
    if value.starts_with(REGIONAL_PACK_PREFIX) && value.len() > REGIONAL_PACK_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudBillingError::InvalidRegionalPack)
    }
}

fn public_data_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudBillingError> {
    let data_class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudBillingError::InvalidDataClass)?;
    if data_class.data_class() == DataClass::Public {
        Ok(data_class)
    } else {
        Err(CloudBillingError::InvalidDataClass)
    }
}

fn financial_data_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudBillingError> {
    let data_class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudBillingError::InvalidDataClass)?;
    if matches!(data_class.data_class(), DataClass::Financial) {
        Ok(data_class)
    } else {
        Err(CloudBillingError::InvalidDataClass)
    }
}

fn prefixed_id(
    value: String,
    prefix: &str,
    error: CloudBillingError,
) -> Result<String, CloudBillingError> {
    if value.starts_with(prefix) && value.len() > prefix.len() && is_safe_reference(&value) {
        Ok(value)
    } else {
        Err(error)
    }
}

fn reference_matches_tenant_region(
    value: &str,
    prefix: &str,
    tenant_id: &str,
    region: &str,
) -> bool {
    let Some(suffix) = value.strip_prefix(prefix) else {
        return false;
    };
    let mut parts = suffix.split('/');
    parts.next() == Some(tenant_id) && parts.next() == Some(region) && parts.next().is_some()
}

fn is_canonical_tenant_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.starts_with('-')
        && !segment.ends_with('-')
        && !segment.contains("--")
        && segment.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

fn is_safe_reference(value: &str) -> bool {
    value == value.trim()
        && !value.is_empty()
        && !value.contains("//")
        && !value.bytes().any(|byte| {
            byte.is_ascii_control()
                || byte.is_ascii_whitespace()
                || matches!(byte, b'\\' | b'?' | b'#')
        })
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn contains_secret_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "access_token",
        "api_key",
        "apikey",
        "bearer",
        "credential",
        "kubeconfig",
        "openbao",
        "password",
        "private_key",
        "secret",
        "tax_registration_secret",
        "token=",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn is_ascii_token(value: &str) -> bool {
    !value.is_empty()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

fn audit<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Audit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use billing_metering::{MeterUnit, MeterUnitKind};

    fn units() -> Vec<MeterUnit> {
        vec![
            MeterUnit::new(MeterUnitKind::ResourceSecond, 3_600_000_000)
                .expect("unit fixture is valid"),
        ]
    }

    fn account_create() -> BillingAccountCreate {
        BillingAccountCreate {
            id: "ba_ten_alpha".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha".to_string(),
            regional_pack: "pack-electronic-tax".to_string(),
            payment_method: "pm_card_001".to_string(),
            credit_balance: Money::new("OYC", 10_000).expect("money fixture valid"),
            state: BillingAccountState::Active,
            data_class: DataClass::Financial,
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn event_create() -> CloudBillingEventCreate {
        CloudBillingEventCreate {
            id: "cbill_resource_created_001".to_string(),
            tenant_id: "ten_alpha".to_string(),
            resource_id: "oya:cloud:region-alpha:ten_alpha:instance:api-001".to_string(),
            region: "region-alpha".to_string(),
            metering_tag: "oya:metering:ten_alpha:instance".to_string(),
            kind: CloudBillingEventKind::ResourceCreated,
            units: units(),
            rate_card_ref: "rate/region-alpha/compute/v1".to_string(),
            occurred_at_epoch_seconds: 1_700_000_100,
            idempotency_key: "idem_ten_alpha_resource_created_api_001".to_string(),
            data_class: DataClass::Public,
        }
    }

    fn invoice_line_item() -> InvoiceLineItemCreate {
        InvoiceLineItemCreate {
            id: "ili_compute_001".to_string(),
            resource_id: "oya:cloud:region-alpha:ten_alpha:instance:api-001".to_string(),
            description: "instance api-001 resource seconds".to_string(),
            units: units(),
            subtotal: Money::new("OYC", 100_000).expect("money fixture valid"),
            data_class: DataClass::Financial,
        }
    }

    fn invoice_generate() -> InvoiceGenerate {
        InvoiceGenerate {
            id: "inv_alpha_202605_001".to_string(),
            billing_account_id: "ba_ten_alpha".to_string(),
            tenant_id: "ten_alpha".to_string(),
            regional_pack: "pack-electronic-tax".to_string(),
            period: BillingPeriod::new(1_700_000_000, 1_700_086_400).expect("period fixture valid"),
            line_items: vec![invoice_line_item()],
            subtotal: Money::new("OYC", 100_000).expect("money fixture valid"),
            tax: Money::new("OYC", 10_000).expect("money fixture valid"),
            total: Money::new("OYC", 110_000).expect("money fixture valid"),
            tax_invoice_format: TaxInvoiceFormat::ElectronicTaxInvoice,
            tax_registration_id: "taxid/electronic/1234567890".to_string(),
            issued_at_epoch_seconds: 1_700_086_500,
            due_at_epoch_seconds: 1_700_604_900,
            data_class: DataClass::Financial,
        }
    }

    #[test]
    fn validates_billing_account_financial_class_and_regional_pack() {
        let account = BillingAccount::new(account_create()).expect("account fixture valid");

        assert_eq!(account.region.value.value, "region-alpha");
        assert_eq!(account.regional_pack.value, "pack-electronic-tax");
        assert_eq!(account.credit_balance.value.currency.value, "OYC");
    }

    #[test]
    fn ingests_cloud_billing_event_through_platform_meter() {
        let mut ledger = CloudBillingLedger::default();
        let mut meter = Meter::default();

        let (event, meter_event) = ledger
            .ingest(&mut meter, event_create())
            .expect("billing event records through meter");

        assert_eq!(event.kind.value, CloudBillingEventKind::ResourceCreated);
        assert_eq!(meter_event.source_axis.value, AxisId::Cloud);
        assert_eq!(
            meter_event.capability_id.value.value,
            "cap.cloud.billing.resource-lifecycle"
        );
        assert_eq!(meter.events().count(), 1);
    }

    #[test]
    fn billing_event_idempotency_replays_original_event_and_meter_record() {
        let mut ledger = CloudBillingLedger::default();
        let mut meter = Meter::default();
        let (first, _) = ledger
            .ingest(&mut meter, event_create())
            .expect("first event records");
        let (replay, _) = ledger
            .ingest(
                &mut meter,
                CloudBillingEventCreate {
                    id: "cbill_resource_created_002".to_string(),
                    ..event_create()
                },
            )
            .expect("same idempotency key returns original event");

        assert_eq!(first.id.value, replay.id.value);
        assert_eq!(ledger.events().count(), 1);
        assert_eq!(meter.events().count(), 1);
    }

    #[test]
    fn generates_electronic_tax_invoice_with_regional_format_and_exact_totals() {
        let account = BillingAccount::new(account_create()).expect("account fixture valid");
        let invoice = Invoice::generate(&account, invoice_generate()).expect("invoice is valid");

        assert_eq!(invoice.id.value.value, "inv_alpha_202605_001");
        assert_eq!(
            invoice.tax_invoice_format.value,
            TaxInvoiceFormat::ElectronicTaxInvoice
        );
        assert_eq!(invoice.line_items.value.len(), 1);
        assert_eq!(invoice.total.value.minor_units, 110_000);
        assert_eq!(invoice.state.value, InvoiceState::Issued);
        assert_eq!(invoice.schema_version.value, CLOUD_INVOICE_SCHEMA_VERSION);
    }

    #[test]
    fn ledger_rejects_duplicate_invoice_ids() {
        let account = BillingAccount::new(account_create()).expect("account fixture valid");
        let mut ledger = CloudBillingLedger::default();
        ledger
            .generate_invoice(&account, invoice_generate())
            .expect("first invoice records");
        let duplicate = ledger
            .generate_invoice(&account, invoice_generate())
            .expect_err("closed invoice id is immutable");

        assert_eq!(duplicate, CloudBillingError::DuplicateInvoice);
        assert_eq!(ledger.invoices().count(), 1);
    }

    #[test]
    fn rejects_invoice_format_tax_registration_total_and_inactive_account() {
        let account = BillingAccount::new(account_create()).expect("account fixture valid");

        let format_error = Invoice::generate(
            &account,
            InvoiceGenerate {
                tax_invoice_format: TaxInvoiceFormat::QualifiedTaxInvoice,
                tax_registration_id: "taxid/qualified/T1234567890123".to_string(),
                ..invoice_generate()
            },
        )
        .expect_err("regional pack determines tax invoice format");
        assert_eq!(format_error, CloudBillingError::InvalidTaxInvoiceFormat);

        let registration_error = Invoice::generate(
            &account,
            InvoiceGenerate {
                tax_registration_id: "taxid/electronic/notdigits".to_string(),
                ..invoice_generate()
            },
        )
        .expect_err("electronic tax invoices require canonical registration shape");
        assert_eq!(
            registration_error,
            CloudBillingError::InvalidTaxRegistrationId
        );

        let total_error = Invoice::generate(
            &account,
            InvoiceGenerate {
                total: Money::new("OYC", 109_999).expect("money fixture valid"),
                ..invoice_generate()
            },
        )
        .expect_err("invoice totals are recomputed");
        assert_eq!(total_error, CloudBillingError::InvalidInvoiceTotal);

        let inactive_account = BillingAccount::new(BillingAccountCreate {
            state: BillingAccountState::Suspended,
            ..account_create()
        })
        .expect("suspended account fixture valid");
        let inactive_error = Invoice::generate(&inactive_account, invoice_generate())
            .expect_err("inactive billing accounts cannot issue invoices");
        assert_eq!(inactive_error, CloudBillingError::BillingAccountInactive);
    }

    #[test]
    fn rejects_resource_tenant_region_and_metering_tag_mismatch() {
        let tenant_error = CloudBillingEvent::new(CloudBillingEventCreate {
            tenant_id: "ten_other".to_string(),
            metering_tag: "oya:metering:ten_other:instance".to_string(),
            ..event_create()
        })
        .expect_err("resource tenant must match billing tenant");
        assert_eq!(tenant_error, CloudBillingError::TenantMismatch);

        let region_error = CloudBillingEvent::new(CloudBillingEventCreate {
            region: "region-beta".to_string(),
            ..event_create()
        })
        .expect_err("resource region must match billing event region");
        assert_eq!(region_error, CloudBillingError::RegionMismatch);

        let metering_error = CloudBillingEvent::new(CloudBillingEventCreate {
            metering_tag: "oya:metering:ten_alpha:bucket".to_string(),
            ..event_create()
        })
        .expect_err("metering tag must match resource tenant and type");
        assert_eq!(metering_error, CloudBillingError::InvalidMeteringTag);
    }

    #[test]
    fn rejects_non_public_event_metadata_and_non_financial_account_class() {
        let event_error = CloudBillingEvent::new(CloudBillingEventCreate {
            data_class: DataClass::Audit,
            ..event_create()
        })
        .expect_err("billing event metadata must be public privacy metadata");
        assert_eq!(event_error, CloudBillingError::InvalidDataClass);

        let account_error = BillingAccount::new(BillingAccountCreate {
            data_class: DataClass::Public,
            ..account_create()
        })
        .expect_err("billing account data class must be financial");
        assert_eq!(account_error, CloudBillingError::InvalidDataClass);
    }

    // --- ST1: transition_invoice ---

    fn ledger_with_invoice() -> (CloudBillingLedger, InvoiceId) {
        let account = BillingAccount::new(account_create()).expect("account fixture valid");
        let mut ledger = CloudBillingLedger::default();
        let invoice = ledger
            .generate_invoice(&account, invoice_generate())
            .expect("invoice fixture valid");
        let id = invoice.id.value.clone();
        (ledger, id)
    }

    #[test]
    fn transition_issued_to_paid() {
        let (mut ledger, id) = ledger_with_invoice();
        let inv = ledger
            .transition_invoice(&id, InvoiceState::Paid)
            .expect("Issued -> Paid is legal");
        assert_eq!(inv.state.value, InvoiceState::Paid);
    }

    #[test]
    fn transition_issued_to_overdue() {
        let (mut ledger, id) = ledger_with_invoice();
        let inv = ledger
            .transition_invoice(&id, InvoiceState::Overdue)
            .expect("Issued -> Overdue is legal");
        assert_eq!(inv.state.value, InvoiceState::Overdue);
    }

    #[test]
    fn transition_issued_to_void() {
        let (mut ledger, id) = ledger_with_invoice();
        let inv = ledger
            .transition_invoice(&id, InvoiceState::Void)
            .expect("Issued -> Void is legal");
        assert_eq!(inv.state.value, InvoiceState::Void);
    }

    #[test]
    fn transition_overdue_to_paid() {
        let (mut ledger, id) = ledger_with_invoice();
        ledger
            .transition_invoice(&id, InvoiceState::Overdue)
            .expect("Issued -> Overdue is legal");
        let inv = ledger
            .transition_invoice(&id, InvoiceState::Paid)
            .expect("Overdue -> Paid is legal");
        assert_eq!(inv.state.value, InvoiceState::Paid);
    }

    #[test]
    fn transition_overdue_to_void() {
        let (mut ledger, id) = ledger_with_invoice();
        ledger
            .transition_invoice(&id, InvoiceState::Overdue)
            .expect("Issued -> Overdue is legal");
        let inv = ledger
            .transition_invoice(&id, InvoiceState::Void)
            .expect("Overdue -> Void is legal");
        assert_eq!(inv.state.value, InvoiceState::Void);
    }

    #[test]
    fn transition_paid_rejects_all() {
        let (mut ledger, id) = ledger_with_invoice();
        ledger
            .transition_invoice(&id, InvoiceState::Paid)
            .expect("Issued -> Paid first");
        for target in [
            InvoiceState::Issued,
            InvoiceState::Overdue,
            InvoiceState::Void,
        ] {
            let err = ledger
                .transition_invoice(&id, target)
                .expect_err("Paid is terminal");
            assert_eq!(
                err,
                CloudBillingError::IllegalInvoiceTransition {
                    from: InvoiceState::Paid,
                    to: target,
                }
            );
        }
    }

    #[test]
    fn transition_void_rejects_all() {
        let (mut ledger, id) = ledger_with_invoice();
        ledger
            .transition_invoice(&id, InvoiceState::Void)
            .expect("Issued -> Void first");
        for target in [
            InvoiceState::Issued,
            InvoiceState::Paid,
            InvoiceState::Overdue,
        ] {
            let err = ledger
                .transition_invoice(&id, target)
                .expect_err("Void is terminal");
            assert_eq!(
                err,
                CloudBillingError::IllegalInvoiceTransition {
                    from: InvoiceState::Void,
                    to: target,
                }
            );
        }
    }

    #[test]
    fn transition_same_state_is_idempotent() {
        let (mut ledger, id) = ledger_with_invoice();
        // Same-state re-transition must succeed and return the unchanged invoice.
        let inv = ledger
            .transition_invoice(&id, InvoiceState::Issued)
            .expect("same-state transition is idempotent");
        assert_eq!(inv.state.value, InvoiceState::Issued);
    }

    #[test]
    fn transition_invoice_not_found() {
        let mut ledger = CloudBillingLedger::default();
        let id = InvoiceId::new("inv_nonexistent_001").expect("id fixture valid");
        let err = ledger
            .transition_invoice(&id, InvoiceState::Paid)
            .expect_err("unknown invoice returns InvoiceNotFound");
        assert_eq!(err, CloudBillingError::InvoiceNotFound);
    }

    // --- ST2: apply_credit_note ---

    fn credit_note_create(credit_minor_units: u64) -> CreditNoteCreate {
        CreditNoteCreate {
            invoice_id: "inv_alpha_202605_001".to_string(),
            line_item_id: "ili_credit_001".to_string(),
            resource_id: "oya:cloud:region-alpha:ten_alpha:instance:api-001".to_string(),
            description: "compute overage correction".to_string(),
            units: units(),
            credit_minor_units,
            currency: "OYC".to_string(),
            data_class: DataClass::Financial,
        }
    }

    #[test]
    fn credit_note_reduces_subtotal() {
        let account = BillingAccount::new(account_create()).expect("account fixture valid");
        let mut ledger = CloudBillingLedger::default();
        ledger
            .generate_invoice(&account, invoice_generate())
            .expect("invoice fixture valid");
        let inv = ledger
            .apply_credit_note("ten_alpha", credit_note_create(10_000))
            .expect("valid credit note reduces subtotal");
        assert_eq!(inv.subtotal.value.minor_units, 90_000);
        assert_eq!(inv.line_items.value.len(), 2);
        assert!(
            inv.line_items.value[1]
                .description
                .value
                .starts_with("[CREDIT] ")
        );
    }

    #[test]
    fn credit_note_over_credit_rejected() {
        let account = BillingAccount::new(account_create()).expect("account fixture valid");
        let mut ledger = CloudBillingLedger::default();
        ledger
            .generate_invoice(&account, invoice_generate())
            .expect("invoice fixture valid");
        let err = ledger
            .apply_credit_note("ten_alpha", credit_note_create(100_001))
            .expect_err("credit exceeding subtotal is rejected");
        assert_eq!(err, CloudBillingError::CreditNoteOverCredit);
    }

    #[test]
    fn credit_note_against_void_rejected() {
        let account = BillingAccount::new(account_create()).expect("account fixture valid");
        let mut ledger = CloudBillingLedger::default();
        let invoice = ledger
            .generate_invoice(&account, invoice_generate())
            .expect("invoice fixture valid");
        let id = invoice.id.value.clone();
        ledger
            .transition_invoice(&id, InvoiceState::Void)
            .expect("Issued -> Void is legal");
        let err = ledger
            .apply_credit_note("ten_alpha", credit_note_create(10_000))
            .expect_err("credit note against Void invoice is rejected");
        assert_eq!(err, CloudBillingError::CreditNoteTargetVoid);
    }

    #[test]
    fn credit_note_currency_mismatch_rejected() {
        let account = BillingAccount::new(account_create()).expect("account fixture valid");
        let mut ledger = CloudBillingLedger::default();
        ledger
            .generate_invoice(&account, invoice_generate())
            .expect("invoice fixture valid");
        let err = ledger
            .apply_credit_note(
                "ten_alpha",
                CreditNoteCreate {
                    currency: "USD".to_string(),
                    ..credit_note_create(10_000)
                },
            )
            .expect_err("currency mismatch is rejected");
        assert_eq!(err, CloudBillingError::InvalidInvoiceTotal);
    }
}
