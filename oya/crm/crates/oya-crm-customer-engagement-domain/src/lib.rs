//! CRM customer-engagement domain foundation.
//!
//! This crate owns pure CRM/CX invariants for account registration metadata,
//! opportunity qualification metadata, quote preparation metadata, service-case
//! opening metadata, marketing-campaign plan metadata, and loyalty activity
//! metadata. It does not perform durable persistence, customer data platform
//! unification, CPQ pricing, order-management mutation, service routing,
//! marketing journey execution, message delivery, loyalty wallet settlement,
//! Workflow execution, runtime audit-chain emission, or cloud runtime I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const ACCOUNT_ID_PREFIX: &str = "acct_";
const LEAD_ID_PREFIX: &str = "lead_";
const OPPORTUNITY_ID_PREFIX: &str = "opp_";
const QUOTE_ID_PREFIX: &str = "quote_";
const SERVICE_CASE_ID_PREFIX: &str = "case_";
const CAMPAIGN_ID_PREFIX: &str = "camp_";
const LOYALTY_ACTIVITY_ID_PREFIX: &str = "loyalty_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const MARKET_ID_PREFIX: &str = "market_";
const CUSTOMER_PROFILE_ID_PREFIX: &str = "cust_";
const CONTACT_ID_PREFIX: &str = "contact_";
const SEGMENT_ID_PREFIX: &str = "seg_";
const PRODUCT_REF_PREFIX: &str = "item_";
const POLICY_REF_PREFIX: &str = "policy/";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const CRM_CUSTOMER_ENGAGEMENT_SCHEMA_VERSION: u32 = 1;
const BASIS_POINTS_DENOMINATOR: u64 = 10_000;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AccountId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LeadId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OpportunityId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct QuoteId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ServiceCaseId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CampaignId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LoyaltyActivityId {
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
pub struct MarketId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CustomerProfileId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ContactId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SegmentId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProductRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PolicyRef {
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
pub enum RelationshipTier {
    Prospect,
    Customer,
    StrategicAccount,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OpportunitySalesStage {
    Discover,
    Qualify,
    Propose,
    Negotiate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServiceCasePriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServiceCaseSeverity {
    Informational,
    Minor,
    Major,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MarketingChannelMix {
    EmailOnly,
    SmsAndEmail,
    PushSmsEmail,
    Omnichannel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CustomerAccountState {
    Registered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OpportunityState {
    Qualified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QuoteState {
    Prepared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServiceCaseState {
    Opened,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MarketingCampaignState {
    Planned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LoyaltyActivityState {
    Recorded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomerAccountInput {
    pub account_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,             // data_class: INTERNAL_ONLY
    pub market_id: String,                   // data_class: INTERNAL_ONLY
    pub customer_profile_id: String,         // data_class: INTERNAL_ONLY
    pub primary_contact_id: String,          // data_class: INTERNAL_ONLY
    pub relationship_tier: RelationshipTier, // data_class: INTERNAL_ONLY
    pub lifecycle_score: u16,                // data_class: FINANCIAL
    pub consent_basis_ref: String,           // data_class: INTERNAL_ONLY
    pub account_source_ref: String,          // data_class: INTERNAL_ONLY
    pub registration_evidence_ref: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomerAccountRegistration {
    pub account_id: Classified<AccountId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,   // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub market_id: Classified<MarketId>,   // data_class: INTERNAL_ONLY
    pub customer_profile_id: Classified<CustomerProfileId>, // data_class: INTERNAL_ONLY
    pub primary_contact_id: Classified<ContactId>, // data_class: INTERNAL_ONLY
    pub relationship_tier: Classified<RelationshipTier>, // data_class: INTERNAL_ONLY
    pub lifecycle_score: Classified<u16>,  // data_class: FINANCIAL
    pub consent_basis_ref: Classified<PolicyRef>, // data_class: INTERNAL_ONLY
    pub account_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub registration_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<CustomerAccountState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub durable_customer_master_attached: Classified<bool>, // data_class: PUBLIC
    pub cdp_unification_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpportunityQualificationInput {
    pub opportunity_id: String,             // data_class: INTERNAL_ONLY
    pub lead_id: String,                    // data_class: INTERNAL_ONLY
    pub account_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,            // data_class: INTERNAL_ONLY
    pub market_id: String,                  // data_class: INTERNAL_ONLY
    pub account_registered: bool,           // data_class: INTERNAL_ONLY
    pub sales_stage: OpportunitySalesStage, // data_class: INTERNAL_ONLY
    pub estimated_revenue_cents: u64,       // data_class: FINANCIAL
    pub win_probability_bps: u16,           // data_class: FINANCIAL
    pub expected_close_yyyymmdd: u32,       // data_class: INTERNAL_ONLY
    pub opportunity_source_ref: String,     // data_class: INTERNAL_ONLY
    pub qualification_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpportunityQualification {
    pub opportunity_id: Classified<OpportunityId>, // data_class: INTERNAL_ONLY
    pub lead_id: Classified<LeadId>,               // data_class: INTERNAL_ONLY
    pub account_id: Classified<AccountId>,         // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,           // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub market_id: Classified<MarketId>,           // data_class: INTERNAL_ONLY
    pub sales_stage: Classified<OpportunitySalesStage>, // data_class: INTERNAL_ONLY
    pub estimated_revenue_cents: Classified<u64>,  // data_class: FINANCIAL
    pub win_probability_bps: Classified<u16>,      // data_class: FINANCIAL
    pub weighted_revenue_cents: Classified<u64>,   // data_class: FINANCIAL
    pub expected_close_yyyymmdd: Classified<u32>,  // data_class: INTERNAL_ONLY
    pub opportunity_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub qualification_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<OpportunityState>,       // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,       // data_class: INTERNAL_ONLY
    pub cpq_runtime_attached: Classified<bool>,    // data_class: PUBLIC
    pub forecast_write_attached: Classified<bool>, // data_class: PUBLIC
    pub workflow_execution_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotePreparationInput {
    pub quote_id: String,              // data_class: INTERNAL_ONLY
    pub opportunity_id: String,        // data_class: INTERNAL_ONLY
    pub account_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub market_id: String,             // data_class: INTERNAL_ONLY
    pub opportunity_qualified: bool,   // data_class: INTERNAL_ONLY
    pub quote_amount_cents: u64,       // data_class: FINANCIAL
    pub discount_bps: u16,             // data_class: FINANCIAL
    pub margin_guardrail_bps: u16,     // data_class: FINANCIAL
    pub valid_until_yyyymmdd: u32,     // data_class: INTERNAL_ONLY
    pub quote_source_ref: String,      // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotePreparation {
    pub quote_id: Classified<QuoteId>, // data_class: INTERNAL_ONLY
    pub opportunity_id: Classified<OpportunityId>, // data_class: INTERNAL_ONLY
    pub account_id: Classified<AccountId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub market_id: Classified<MarketId>, // data_class: INTERNAL_ONLY
    pub quote_amount_cents: Classified<u64>, // data_class: FINANCIAL
    pub discount_bps: Classified<u16>, // data_class: FINANCIAL
    pub discount_amount_cents: Classified<u64>, // data_class: FINANCIAL
    pub net_quote_amount_cents: Classified<u64>, // data_class: FINANCIAL
    pub margin_guardrail_bps: Classified<u16>, // data_class: FINANCIAL
    pub valid_until_yyyymmdd: Classified<u32>, // data_class: INTERNAL_ONLY
    pub quote_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<QuoteState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub cpq_price_engine_attached: Classified<bool>, // data_class: PUBLIC
    pub order_management_attached: Classified<bool>, // data_class: PUBLIC
    pub erp_price_sync_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceCaseInput {
    pub case_id: String,               // data_class: INTERNAL_ONLY
    pub account_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub market_id: String,             // data_class: INTERNAL_ONLY
    pub product_ref: String,           // data_class: INTERNAL_ONLY
    pub account_registered: bool,      // data_class: INTERNAL_ONLY
    pub priority: ServiceCasePriority, // data_class: INTERNAL_ONLY
    pub severity: ServiceCaseSeverity, // data_class: INTERNAL_ONLY
    pub opened_on_yyyymmdd: u32,       // data_class: INTERNAL_ONLY
    pub sla_due_yyyymmdd: u32,         // data_class: INTERNAL_ONLY
    pub case_source_ref: String,       // data_class: INTERNAL_ONLY
    pub case_evidence_ref: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceCaseOpening {
    pub case_id: Classified<ServiceCaseId>, // data_class: INTERNAL_ONLY
    pub account_id: Classified<AccountId>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,    // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub market_id: Classified<MarketId>,    // data_class: INTERNAL_ONLY
    pub product_ref: Classified<ProductRef>, // data_class: INTERNAL_ONLY
    pub priority: Classified<ServiceCasePriority>, // data_class: INTERNAL_ONLY
    pub severity: Classified<ServiceCaseSeverity>, // data_class: INTERNAL_ONLY
    pub opened_on_yyyymmdd: Classified<u32>, // data_class: INTERNAL_ONLY
    pub sla_due_yyyymmdd: Classified<u32>,  // data_class: INTERNAL_ONLY
    pub escalation_required: Classified<bool>, // data_class: PUBLIC
    pub case_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub case_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<ServiceCaseState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub service_routing_attached: Classified<bool>, // data_class: PUBLIC
    pub knowledge_base_attached: Classified<bool>, // data_class: PUBLIC
    pub workflow_execution_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketingCampaignInput {
    pub campaign_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,          // data_class: INTERNAL_ONLY
    pub market_id: String,                // data_class: INTERNAL_ONLY
    pub segment_id: String,               // data_class: INTERNAL_ONLY
    pub channel_mix: MarketingChannelMix, // data_class: INTERNAL_ONLY
    pub consent_basis_ref: String,        // data_class: INTERNAL_ONLY
    pub planned_start_yyyymmdd: u32,      // data_class: INTERNAL_ONLY
    pub planned_end_yyyymmdd: u32,        // data_class: INTERNAL_ONLY
    pub campaign_budget_cents: u64,       // data_class: FINANCIAL
    pub campaign_source_ref: String,      // data_class: INTERNAL_ONLY
    pub campaign_evidence_ref: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketingCampaignPlan {
    pub campaign_id: Classified<CampaignId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,     // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub market_id: Classified<MarketId>,     // data_class: INTERNAL_ONLY
    pub segment_id: Classified<SegmentId>,   // data_class: INTERNAL_ONLY
    pub channel_mix: Classified<MarketingChannelMix>, // data_class: INTERNAL_ONLY
    pub consent_basis_ref: Classified<PolicyRef>, // data_class: INTERNAL_ONLY
    pub planned_start_yyyymmdd: Classified<u32>, // data_class: INTERNAL_ONLY
    pub planned_end_yyyymmdd: Classified<u32>, // data_class: INTERNAL_ONLY
    pub campaign_budget_cents: Classified<u64>, // data_class: FINANCIAL
    pub campaign_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub campaign_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<MarketingCampaignState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub journey_runtime_attached: Classified<bool>, // data_class: PUBLIC
    pub message_delivery_attached: Classified<bool>, // data_class: PUBLIC
    pub cdp_segment_runtime_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoyaltyActivityInput {
    pub loyalty_activity_id: String,   // data_class: INTERNAL_ONLY
    pub account_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub market_id: String,             // data_class: INTERNAL_ONLY
    pub account_registered: bool,      // data_class: INTERNAL_ONLY
    pub points_delta: i32,             // data_class: FINANCIAL
    pub monetary_value_cents: u64,     // data_class: FINANCIAL
    pub activity_yyyymmdd: u32,        // data_class: INTERNAL_ONLY
    pub activity_source_ref: String,   // data_class: INTERNAL_ONLY
    pub activity_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoyaltyActivityRecord {
    pub loyalty_activity_id: Classified<LoyaltyActivityId>, // data_class: INTERNAL_ONLY
    pub account_id: Classified<AccountId>,                  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                    // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,         // data_class: INTERNAL_ONLY
    pub market_id: Classified<MarketId>,                    // data_class: INTERNAL_ONLY
    pub points_delta: Classified<i32>,                      // data_class: FINANCIAL
    pub points_earned: Classified<u32>,                     // data_class: FINANCIAL
    pub points_redeemed: Classified<u32>,                   // data_class: FINANCIAL
    pub monetary_value_cents: Classified<u64>,              // data_class: FINANCIAL
    pub activity_yyyymmdd: Classified<u32>,                 // data_class: INTERNAL_ONLY
    pub activity_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub activity_evidence_ref: Classified<EvidenceRef>,     // data_class: INTERNAL_ONLY
    pub state: Classified<LoyaltyActivityState>,            // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,                // data_class: INTERNAL_ONLY
    pub loyalty_wallet_runtime_attached: Classified<bool>,  // data_class: PUBLIC
    pub reward_settlement_attached: Classified<bool>,       // data_class: PUBLIC
    pub marketing_journey_attached: Classified<bool>,       // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,        // data_class: PUBLIC
    pub schema_version: Classified<u32>,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CrmCustomerEngagementError {
    InvalidAccountId,
    InvalidLeadId,
    InvalidOpportunityId,
    InvalidQuoteId,
    InvalidServiceCaseId,
    InvalidCampaignId,
    InvalidLoyaltyActivityId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidMarketId,
    InvalidCustomerProfileId,
    InvalidContactId,
    InvalidSegmentId,
    InvalidProductRef,
    InvalidPolicyRef,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidDate,
    InvalidScore,
    InvalidAmount,
    InvalidProbability,
    InvalidDiscount,
    InvalidLoyaltyPoints,
    AccountRegistrationRequired,
    OpportunityQualificationRequired,
}

pub fn register_customer_account(
    input: CustomerAccountInput,
) -> Result<CustomerAccountRegistration, CrmCustomerEngagementError> {
    validate_account_id(&input.account_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_market_id(&input.market_id)?;
    validate_customer_profile_id(&input.customer_profile_id)?;
    validate_contact_id(&input.primary_contact_id)?;
    validate_score(input.lifecycle_score)?;
    validate_policy_ref(&input.consent_basis_ref)?;
    validate_source_ref(&input.account_source_ref)?;
    validate_evidence_ref(&input.registration_evidence_ref)?;
    let idempotency_key = format!(
        "crm:account:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.market_id, input.account_id
    );

    Ok(CustomerAccountRegistration {
        account_id: internal(AccountId {
            value: input.account_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        market_id: internal(MarketId {
            value: input.market_id,
        }),
        customer_profile_id: internal(CustomerProfileId {
            value: input.customer_profile_id,
        }),
        primary_contact_id: internal(ContactId {
            value: input.primary_contact_id,
        }),
        relationship_tier: internal(input.relationship_tier),
        lifecycle_score: financial(input.lifecycle_score),
        consent_basis_ref: internal(PolicyRef {
            value: input.consent_basis_ref,
        }),
        account_source_ref: internal(SourceDocumentRef {
            value: input.account_source_ref,
        }),
        registration_evidence_ref: internal(EvidenceRef {
            value: input.registration_evidence_ref,
        }),
        state: internal(CustomerAccountState::Registered),
        idempotency_key: internal(idempotency_key),
        durable_customer_master_attached: public(false),
        cdp_unification_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(CRM_CUSTOMER_ENGAGEMENT_SCHEMA_VERSION),
    })
}

pub fn qualify_opportunity(
    input: OpportunityQualificationInput,
) -> Result<OpportunityQualification, CrmCustomerEngagementError> {
    validate_opportunity_id(&input.opportunity_id)?;
    validate_lead_id(&input.lead_id)?;
    validate_account_id(&input.account_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_market_id(&input.market_id)?;
    if !input.account_registered {
        return Err(CrmCustomerEngagementError::AccountRegistrationRequired);
    }
    validate_amount(input.estimated_revenue_cents)?;
    validate_probability(input.win_probability_bps)?;
    validate_yyyymmdd(input.expected_close_yyyymmdd)?;
    validate_source_ref(&input.opportunity_source_ref)?;
    validate_evidence_ref(&input.qualification_evidence_ref)?;
    let weighted_revenue_cents = input
        .estimated_revenue_cents
        .checked_mul(u64::from(input.win_probability_bps))
        .and_then(|value| value.checked_div(BASIS_POINTS_DENOMINATOR))
        .ok_or(CrmCustomerEngagementError::InvalidAmount)?;
    let idempotency_key = format!(
        "crm:opportunity:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.market_id, input.opportunity_id
    );

    Ok(OpportunityQualification {
        opportunity_id: internal(OpportunityId {
            value: input.opportunity_id,
        }),
        lead_id: internal(LeadId {
            value: input.lead_id,
        }),
        account_id: internal(AccountId {
            value: input.account_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        market_id: internal(MarketId {
            value: input.market_id,
        }),
        sales_stage: internal(input.sales_stage),
        estimated_revenue_cents: financial(input.estimated_revenue_cents),
        win_probability_bps: financial(input.win_probability_bps),
        weighted_revenue_cents: financial(weighted_revenue_cents),
        expected_close_yyyymmdd: internal(input.expected_close_yyyymmdd),
        opportunity_source_ref: internal(SourceDocumentRef {
            value: input.opportunity_source_ref,
        }),
        qualification_evidence_ref: internal(EvidenceRef {
            value: input.qualification_evidence_ref,
        }),
        state: internal(OpportunityState::Qualified),
        idempotency_key: internal(idempotency_key),
        cpq_runtime_attached: public(false),
        forecast_write_attached: public(false),
        workflow_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(CRM_CUSTOMER_ENGAGEMENT_SCHEMA_VERSION),
    })
}

pub fn prepare_quote(
    input: QuotePreparationInput,
) -> Result<QuotePreparation, CrmCustomerEngagementError> {
    validate_quote_id(&input.quote_id)?;
    validate_opportunity_id(&input.opportunity_id)?;
    validate_account_id(&input.account_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_market_id(&input.market_id)?;
    if !input.opportunity_qualified {
        return Err(CrmCustomerEngagementError::OpportunityQualificationRequired);
    }
    validate_amount(input.quote_amount_cents)?;
    validate_discount(input.discount_bps, input.margin_guardrail_bps)?;
    validate_yyyymmdd(input.valid_until_yyyymmdd)?;
    validate_source_ref(&input.quote_source_ref)?;
    validate_evidence_ref(&input.approval_evidence_ref)?;
    let discount_amount_cents = input
        .quote_amount_cents
        .checked_mul(u64::from(input.discount_bps))
        .and_then(|value| value.checked_div(BASIS_POINTS_DENOMINATOR))
        .ok_or(CrmCustomerEngagementError::InvalidAmount)?;
    let net_quote_amount_cents = input
        .quote_amount_cents
        .checked_sub(discount_amount_cents)
        .ok_or(CrmCustomerEngagementError::InvalidDiscount)?;
    let idempotency_key = format!(
        "crm:quote:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.market_id, input.quote_id
    );

    Ok(QuotePreparation {
        quote_id: internal(QuoteId {
            value: input.quote_id,
        }),
        opportunity_id: internal(OpportunityId {
            value: input.opportunity_id,
        }),
        account_id: internal(AccountId {
            value: input.account_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        market_id: internal(MarketId {
            value: input.market_id,
        }),
        quote_amount_cents: financial(input.quote_amount_cents),
        discount_bps: financial(input.discount_bps),
        discount_amount_cents: financial(discount_amount_cents),
        net_quote_amount_cents: financial(net_quote_amount_cents),
        margin_guardrail_bps: financial(input.margin_guardrail_bps),
        valid_until_yyyymmdd: internal(input.valid_until_yyyymmdd),
        quote_source_ref: internal(SourceDocumentRef {
            value: input.quote_source_ref,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        state: internal(QuoteState::Prepared),
        idempotency_key: internal(idempotency_key),
        cpq_price_engine_attached: public(false),
        order_management_attached: public(false),
        erp_price_sync_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(CRM_CUSTOMER_ENGAGEMENT_SCHEMA_VERSION),
    })
}

pub fn open_service_case(
    input: ServiceCaseInput,
) -> Result<ServiceCaseOpening, CrmCustomerEngagementError> {
    validate_service_case_id(&input.case_id)?;
    validate_account_id(&input.account_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_market_id(&input.market_id)?;
    validate_product_ref(&input.product_ref)?;
    if !input.account_registered {
        return Err(CrmCustomerEngagementError::AccountRegistrationRequired);
    }
    validate_yyyymmdd(input.opened_on_yyyymmdd)?;
    validate_yyyymmdd(input.sla_due_yyyymmdd)?;
    if input.sla_due_yyyymmdd < input.opened_on_yyyymmdd {
        return Err(CrmCustomerEngagementError::InvalidDate);
    }
    validate_source_ref(&input.case_source_ref)?;
    validate_evidence_ref(&input.case_evidence_ref)?;
    let escalation_required = input.priority == ServiceCasePriority::Critical
        || input.severity == ServiceCaseSeverity::Critical;
    let idempotency_key = format!(
        "crm:case:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.market_id, input.case_id
    );

    Ok(ServiceCaseOpening {
        case_id: internal(ServiceCaseId {
            value: input.case_id,
        }),
        account_id: internal(AccountId {
            value: input.account_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        market_id: internal(MarketId {
            value: input.market_id,
        }),
        product_ref: internal(ProductRef {
            value: input.product_ref,
        }),
        priority: internal(input.priority),
        severity: internal(input.severity),
        opened_on_yyyymmdd: internal(input.opened_on_yyyymmdd),
        sla_due_yyyymmdd: internal(input.sla_due_yyyymmdd),
        escalation_required: public(escalation_required),
        case_source_ref: internal(SourceDocumentRef {
            value: input.case_source_ref,
        }),
        case_evidence_ref: internal(EvidenceRef {
            value: input.case_evidence_ref,
        }),
        state: internal(ServiceCaseState::Opened),
        idempotency_key: internal(idempotency_key),
        service_routing_attached: public(false),
        knowledge_base_attached: public(false),
        workflow_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(CRM_CUSTOMER_ENGAGEMENT_SCHEMA_VERSION),
    })
}

pub fn plan_marketing_campaign(
    input: MarketingCampaignInput,
) -> Result<MarketingCampaignPlan, CrmCustomerEngagementError> {
    validate_campaign_id(&input.campaign_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_market_id(&input.market_id)?;
    validate_segment_id(&input.segment_id)?;
    validate_policy_ref(&input.consent_basis_ref)?;
    validate_yyyymmdd(input.planned_start_yyyymmdd)?;
    validate_yyyymmdd(input.planned_end_yyyymmdd)?;
    if input.planned_end_yyyymmdd < input.planned_start_yyyymmdd {
        return Err(CrmCustomerEngagementError::InvalidDate);
    }
    validate_amount(input.campaign_budget_cents)?;
    validate_source_ref(&input.campaign_source_ref)?;
    validate_evidence_ref(&input.campaign_evidence_ref)?;
    let idempotency_key = format!(
        "crm:campaign:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.market_id, input.campaign_id
    );

    Ok(MarketingCampaignPlan {
        campaign_id: internal(CampaignId {
            value: input.campaign_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        market_id: internal(MarketId {
            value: input.market_id,
        }),
        segment_id: internal(SegmentId {
            value: input.segment_id,
        }),
        channel_mix: internal(input.channel_mix),
        consent_basis_ref: internal(PolicyRef {
            value: input.consent_basis_ref,
        }),
        planned_start_yyyymmdd: internal(input.planned_start_yyyymmdd),
        planned_end_yyyymmdd: internal(input.planned_end_yyyymmdd),
        campaign_budget_cents: financial(input.campaign_budget_cents),
        campaign_source_ref: internal(SourceDocumentRef {
            value: input.campaign_source_ref,
        }),
        campaign_evidence_ref: internal(EvidenceRef {
            value: input.campaign_evidence_ref,
        }),
        state: internal(MarketingCampaignState::Planned),
        idempotency_key: internal(idempotency_key),
        journey_runtime_attached: public(false),
        message_delivery_attached: public(false),
        cdp_segment_runtime_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(CRM_CUSTOMER_ENGAGEMENT_SCHEMA_VERSION),
    })
}

pub fn record_loyalty_activity(
    input: LoyaltyActivityInput,
) -> Result<LoyaltyActivityRecord, CrmCustomerEngagementError> {
    validate_loyalty_activity_id(&input.loyalty_activity_id)?;
    validate_account_id(&input.account_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_market_id(&input.market_id)?;
    if !input.account_registered {
        return Err(CrmCustomerEngagementError::AccountRegistrationRequired);
    }
    if input.points_delta == 0 {
        return Err(CrmCustomerEngagementError::InvalidLoyaltyPoints);
    }
    validate_amount(input.monetary_value_cents)?;
    validate_yyyymmdd(input.activity_yyyymmdd)?;
    validate_source_ref(&input.activity_source_ref)?;
    validate_evidence_ref(&input.activity_evidence_ref)?;
    let points_earned = u32::try_from(input.points_delta.max(0))
        .map_err(|_| CrmCustomerEngagementError::InvalidLoyaltyPoints)?;
    let points_redeemed = input
        .points_delta
        .unsigned_abs()
        .saturating_sub(points_earned);
    let idempotency_key = format!(
        "crm:loyalty:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.market_id, input.loyalty_activity_id
    );

    Ok(LoyaltyActivityRecord {
        loyalty_activity_id: internal(LoyaltyActivityId {
            value: input.loyalty_activity_id,
        }),
        account_id: internal(AccountId {
            value: input.account_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        market_id: internal(MarketId {
            value: input.market_id,
        }),
        points_delta: financial(input.points_delta),
        points_earned: financial(points_earned),
        points_redeemed: financial(points_redeemed),
        monetary_value_cents: financial(input.monetary_value_cents),
        activity_yyyymmdd: internal(input.activity_yyyymmdd),
        activity_source_ref: internal(SourceDocumentRef {
            value: input.activity_source_ref,
        }),
        activity_evidence_ref: internal(EvidenceRef {
            value: input.activity_evidence_ref,
        }),
        state: internal(LoyaltyActivityState::Recorded),
        idempotency_key: internal(idempotency_key),
        loyalty_wallet_runtime_attached: public(false),
        reward_settlement_attached: public(false),
        marketing_journey_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(CRM_CUSTOMER_ENGAGEMENT_SCHEMA_VERSION),
    })
}

fn validate_account_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        ACCOUNT_ID_PREFIX,
        CrmCustomerEngagementError::InvalidAccountId,
    )
}

fn validate_lead_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        LEAD_ID_PREFIX,
        CrmCustomerEngagementError::InvalidLeadId,
    )
}

fn validate_opportunity_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        OPPORTUNITY_ID_PREFIX,
        CrmCustomerEngagementError::InvalidOpportunityId,
    )
}

fn validate_quote_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        QUOTE_ID_PREFIX,
        CrmCustomerEngagementError::InvalidQuoteId,
    )
}

fn validate_service_case_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        SERVICE_CASE_ID_PREFIX,
        CrmCustomerEngagementError::InvalidServiceCaseId,
    )
}

fn validate_campaign_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        CAMPAIGN_ID_PREFIX,
        CrmCustomerEngagementError::InvalidCampaignId,
    )
}

fn validate_loyalty_activity_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        LOYALTY_ACTIVITY_ID_PREFIX,
        CrmCustomerEngagementError::InvalidLoyaltyActivityId,
    )
}

fn validate_tenant_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        TENANT_ID_PREFIX,
        CrmCustomerEngagementError::InvalidTenantId,
    )
}

fn validate_legal_entity_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        LEGAL_ENTITY_ID_PREFIX,
        CrmCustomerEngagementError::InvalidLegalEntityId,
    )
}

fn validate_market_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        MARKET_ID_PREFIX,
        CrmCustomerEngagementError::InvalidMarketId,
    )
}

fn validate_customer_profile_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        CUSTOMER_PROFILE_ID_PREFIX,
        CrmCustomerEngagementError::InvalidCustomerProfileId,
    )
}

fn validate_contact_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        CONTACT_ID_PREFIX,
        CrmCustomerEngagementError::InvalidContactId,
    )
}

fn validate_segment_id(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        SEGMENT_ID_PREFIX,
        CrmCustomerEngagementError::InvalidSegmentId,
    )
}

fn validate_product_ref(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_prefixed_identifier(
        value,
        PRODUCT_REF_PREFIX,
        CrmCustomerEngagementError::InvalidProductRef,
    )
}

fn validate_prefixed_identifier(
    value: &str,
    prefix: &str,
    error: CrmCustomerEngagementError,
) -> Result<(), CrmCustomerEngagementError> {
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

fn validate_policy_ref(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_ref(
        value,
        POLICY_REF_PREFIX,
        CrmCustomerEngagementError::InvalidPolicyRef,
    )
}

fn validate_source_ref(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_ref(
        value,
        SOURCE_REF_PREFIX,
        CrmCustomerEngagementError::InvalidSourceDocumentRef,
    )
}

fn validate_evidence_ref(value: &str) -> Result<(), CrmCustomerEngagementError> {
    validate_ref(
        value,
        AUDIT_REF_PREFIX,
        CrmCustomerEngagementError::InvalidEvidenceRef,
    )
}

fn validate_ref(
    value: &str,
    prefix: &str,
    error: CrmCustomerEngagementError,
) -> Result<(), CrmCustomerEngagementError> {
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

fn validate_score(value: u16) -> Result<(), CrmCustomerEngagementError> {
    if value > 1_000 {
        return Err(CrmCustomerEngagementError::InvalidScore);
    }
    Ok(())
}

fn validate_amount(value: u64) -> Result<(), CrmCustomerEngagementError> {
    if value == 0 {
        return Err(CrmCustomerEngagementError::InvalidAmount);
    }
    Ok(())
}

fn validate_probability(value: u16) -> Result<(), CrmCustomerEngagementError> {
    if !(1..=10_000).contains(&value) {
        return Err(CrmCustomerEngagementError::InvalidProbability);
    }
    Ok(())
}

fn validate_discount(
    discount_bps: u16,
    margin_guardrail_bps: u16,
) -> Result<(), CrmCustomerEngagementError> {
    if discount_bps > 9_000 || margin_guardrail_bps > 10_000 || discount_bps > margin_guardrail_bps
    {
        return Err(CrmCustomerEngagementError::InvalidDiscount);
    }
    Ok(())
}

fn validate_yyyymmdd(value: u32) -> Result<(), CrmCustomerEngagementError> {
    let year = value / 10_000;
    let month = (value / 100) % 100;
    let day = value % 100;
    if !(2020..=2100).contains(&year) || !(1..=12).contains(&month) {
        return Err(CrmCustomerEngagementError::InvalidDate);
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
        _ => return Err(CrmCustomerEngagementError::InvalidDate),
    };
    if day == 0 || day > max_day {
        return Err(CrmCustomerEngagementError::InvalidDate);
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
