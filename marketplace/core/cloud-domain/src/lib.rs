//! Cloud marketplace aggregate kernel.
//!
//! This crate owns cloud-native ISV onboarding, public listing publication,
//! private offers, tenant entitlements, revenue share, and marketplace fee
//! metering for the Cloud axis. The contracts are control-plane only and keep
//! public listing metadata separated from seller, tenant, and financial fields.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use billing_domain::{BillingAccountId, CloudBillingError, CurrencyCode, Money};
use billing_metering::{
    AxisId, Meter, MeterEvent, MeterEventCreate, MeterUnit, MeteringError, PlaneTag,
};
use cell_region::RegionCode;
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const MARKETPLACE_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const SELLER_ID_PREFIX: &str = "isv_";
const LISTING_ID_PREFIX: &str = "cml_";
const PRIVATE_OFFER_ID_PREFIX: &str = "cpo_";
const ENTITLEMENT_ID_PREFIX: &str = "cme_";
const REVIEW_REF_PREFIX: &str = "review/";
const KYB_EVIDENCE_REF_PREFIX: &str = "kyb/";
const SUPPORT_REF_PREFIX: &str = "support/";
const LEGAL_TERMS_REF_PREFIX: &str = "legal/";
const PROVISIONING_HOOK_PREFIX: &str = "cap.cloud.marketplace.";
const MARKETPLACE_FEE_CAPABILITY_ID: &str = "cap.cloud.marketplace.fee";
const MAX_PRIVATE_OFFER_DISCOUNT_BPS: u16 = 5_000;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SellerId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketplaceListingId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PrivateOfferId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EntitlementId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReviewRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KybEvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SupportRef {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LegalTermsRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProvisioningHookRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ListingCategory {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MarketplaceTrustTier {
    VerifiedIsv,
    Community,
    Experimental,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SellerState {
    Applying,
    Verified,
    Suspended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ListingState {
    Draft,
    Published,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BillingModel {
    Free,
    FlatMonthly,
    UsageMetered,
    PrivateOfferOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayoutCadence {
    Monthly,
    Quarterly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PrivateOfferState {
    Offered,
    Accepted,
    Expired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EntitlementState {
    Active,
    Suspended,
    Terminated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevenueShareCreate {
    pub seller_bps: u16,               // data_class: FINANCIAL
    pub platform_bps: u16,             // data_class: FINANCIAL
    pub payout_cadence: PayoutCadence, // data_class: INTERNAL_ONLY
    pub settlement_currency: String,   // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevenueShare {
    pub seller_bps: u16,                   // data_class: FINANCIAL
    pub platform_bps: u16,                 // data_class: FINANCIAL
    pub payout_cadence: PayoutCadence,     // data_class: INTERNAL_ONLY
    pub settlement_currency: CurrencyCode, // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SellerApplicationCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub legal_name: String,            // data_class: INTERNAL_ONLY
    pub home_region: String,           // data_class: PUBLIC
    pub kyb_evidence_ref: String,      // data_class: INTERNAL_ONLY
    pub support_ref: String,           // data_class: PUBLIC
    pub data_class: DataClass,         // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SellerVerificationCreate {
    pub seller_id: String,                // data_class: INTERNAL_ONLY
    pub trust_tier: MarketplaceTrustTier, // data_class: PUBLIC
    pub security_review_ref: String,      // data_class: INTERNAL_ONLY
    pub verified_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMarketplaceSeller {
    pub id: Classified<SellerId>,            // data_class: INTERNAL_ONLY
    pub legal_name: Classified<String>,      // data_class: INTERNAL_ONLY
    pub home_region: Classified<RegionCode>, // data_class: PUBLIC
    pub kyb_evidence_ref: Classified<KybEvidenceRef>, // data_class: INTERNAL_ONLY
    pub support_ref: Classified<SupportRef>, // data_class: PUBLIC
    pub trust_tier: Classified<Option<MarketplaceTrustTier>>, // data_class: PUBLIC
    pub security_review_ref: Classified<Option<ReviewRef>>, // data_class: INTERNAL_ONLY
    pub state: Classified<SellerState>,      // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub verified_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListingDraftCreate {
    pub id: String,                        // data_class: INTERNAL_ONLY
    pub seller_id: String,                 // data_class: INTERNAL_ONLY
    pub version: String,                   // data_class: PUBLIC
    pub title: String,                     // data_class: PUBLIC
    pub summary: String,                   // data_class: PUBLIC
    pub categories: Vec<String>,           // data_class: PUBLIC
    pub supported_regions: Vec<String>,    // data_class: PUBLIC
    pub provisioning_hook: String,         // data_class: INTERNAL_ONLY
    pub support_ref: String,               // data_class: PUBLIC
    pub billing_model: BillingModel,       // data_class: PUBLIC
    pub revenue_share: RevenueShareCreate, // data_class: FINANCIAL
    pub data_class: DataClass,             // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListingPublicationCreate {
    pub listing_id: String,              // data_class: INTERNAL_ONLY
    pub security_review_ref: String,     // data_class: INTERNAL_ONLY
    pub license_review_ref: String,      // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMarketplaceListing {
    pub id: Classified<MarketplaceListingId>, // data_class: INTERNAL_ONLY
    pub seller_id: Classified<SellerId>,      // data_class: INTERNAL_ONLY
    pub version: Classified<String>,          // data_class: PUBLIC
    pub title: Classified<String>,            // data_class: PUBLIC
    pub summary: Classified<String>,          // data_class: PUBLIC
    pub categories: Classified<Vec<ListingCategory>>, // data_class: PUBLIC
    pub supported_regions: Classified<Vec<RegionCode>>, // data_class: PUBLIC
    pub provisioning_hook: Classified<ProvisioningHookRef>, // data_class: INTERNAL_ONLY
    pub support_ref: Classified<SupportRef>,  // data_class: PUBLIC
    pub billing_model: Classified<BillingModel>, // data_class: PUBLIC
    pub revenue_share: Classified<RevenueShare>, // data_class: FINANCIAL
    pub security_review_ref: Classified<Option<ReviewRef>>, // data_class: INTERNAL_ONLY
    pub license_review_ref: Classified<Option<ReviewRef>>, // data_class: INTERNAL_ONLY
    pub state: Classified<ListingState>,      // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateOfferCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub listing_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub billing_account_id: String,    // data_class: INTERNAL_ONLY
    pub legal_terms_ref: String,       // data_class: INTERNAL_ONLY
    pub fixed_price: Money,            // data_class: FINANCIAL
    pub discount_bps: u16,             // data_class: FINANCIAL
    pub starts_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,         // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudPrivateOffer {
    pub id: Classified<PrivateOfferId>, // data_class: INTERNAL_ONLY
    pub listing_id: Classified<MarketplaceListingId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub billing_account_id: Classified<BillingAccountId>, // data_class: INTERNAL_ONLY
    pub legal_terms_ref: Classified<LegalTermsRef>, // data_class: INTERNAL_ONLY
    pub fixed_price: Classified<Money>, // data_class: FINANCIAL
    pub discount_bps: Classified<u16>,  // data_class: FINANCIAL
    pub starts_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub accepted_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub state: Classified<PrivateOfferState>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: FINANCIAL
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntitlementCreate {
    pub id: String,                                // data_class: INTERNAL_ONLY
    pub listing_id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                            // data_class: PUBLIC
    pub billing_account_id: String,                // data_class: INTERNAL_ONLY
    pub accepted_private_offer_id: Option<String>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub data_class: DataClass,                     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMarketplaceEntitlement {
    pub id: Classified<EntitlementId>, // data_class: INTERNAL_ONLY
    pub listing_id: Classified<MarketplaceListingId>, // data_class: INTERNAL_ONLY
    pub seller_id: Classified<SellerId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub billing_account_id: Classified<BillingAccountId>, // data_class: INTERNAL_ONLY
    pub accepted_private_offer_id: Classified<Option<PrivateOfferId>>, // data_class: INTERNAL_ONLY
    pub provisioning_hook: Classified<ProvisioningHookRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<EntitlementState>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketplaceFeeCreate {
    pub meter_event_id: String,         // data_class: INTERNAL_ONLY
    pub entitlement_id: String,         // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub units: Vec<MeterUnit>,          // data_class: INTERNAL_ONLY
    pub recorded_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub data_class: DataClass,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudMarketplaceError {
    InvalidSellerId,
    InvalidListingId,
    InvalidPrivateOfferId,
    InvalidEntitlementId,
    InvalidTenantId,
    InvalidRegion,
    InvalidReviewRef,
    InvalidKybEvidenceRef,
    InvalidSupportRef,
    InvalidLegalTermsRef,
    InvalidProvisioningHookRef,
    InvalidCategory,
    InvalidTitle,
    InvalidSummary,
    InvalidVersion,
    InvalidDataClass,
    InvalidTrustTier,
    InvalidSellerState,
    InvalidListingState,
    InvalidOfferState,
    InvalidTimeOrder,
    InvalidRevenueShare,
    InvalidCurrency,
    InvalidDiscount,
    RegionNotSupported,
    UnknownSeller,
    UnknownListing,
    UnknownPrivateOffer,
    UnknownEntitlement,
    DuplicateSeller,
    DuplicateListing,
    DuplicatePrivateOffer,
    DuplicateEntitlement,
    BillingAccountInvalid,
    MeteringRejected,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudMarketplaceCatalog {
    sellers: BTreeMap<SellerId, CloudMarketplaceSeller>,
    listings: BTreeMap<MarketplaceListingId, CloudMarketplaceListing>,
    private_offers: BTreeMap<PrivateOfferId, CloudPrivateOffer>,
    entitlements: BTreeMap<EntitlementId, CloudMarketplaceEntitlement>,
    meter: Meter,
}

pub trait MarketplaceRepo {
    fn submit_seller_application(
        &mut self,
        input: SellerApplicationCreate,
    ) -> Result<CloudMarketplaceSeller, CloudMarketplaceError>;
    fn verify_seller(
        &mut self,
        input: SellerVerificationCreate,
    ) -> Result<CloudMarketplaceSeller, CloudMarketplaceError>;
    fn create_listing_draft(
        &mut self,
        input: ListingDraftCreate,
    ) -> Result<CloudMarketplaceListing, CloudMarketplaceError>;
    fn publish_listing(
        &mut self,
        input: ListingPublicationCreate,
    ) -> Result<CloudMarketplaceListing, CloudMarketplaceError>;
    fn create_private_offer(
        &mut self,
        input: PrivateOfferCreate,
    ) -> Result<CloudPrivateOffer, CloudMarketplaceError>;
    fn accept_private_offer(
        &mut self,
        id: String,
        accepted_at_epoch_seconds: u64,
    ) -> Result<CloudPrivateOffer, CloudMarketplaceError>;
    fn activate_entitlement(
        &mut self,
        input: EntitlementCreate,
    ) -> Result<CloudMarketplaceEntitlement, CloudMarketplaceError>;
    fn record_marketplace_fee(
        &mut self,
        input: MarketplaceFeeCreate,
    ) -> Result<MeterEvent, CloudMarketplaceError>;
}

impl SellerId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        prefixed_token(
            value.into(),
            SELLER_ID_PREFIX,
            CloudMarketplaceError::InvalidSellerId,
        )
        .map(|value| Self { value })
    }
}

impl MarketplaceListingId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        prefixed_token(
            value.into(),
            LISTING_ID_PREFIX,
            CloudMarketplaceError::InvalidListingId,
        )
        .map(|value| Self { value })
    }
}

impl PrivateOfferId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        prefixed_token(
            value.into(),
            PRIVATE_OFFER_ID_PREFIX,
            CloudMarketplaceError::InvalidPrivateOfferId,
        )
        .map(|value| Self { value })
    }
}

impl EntitlementId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        prefixed_token(
            value.into(),
            ENTITLEMENT_ID_PREFIX,
            CloudMarketplaceError::InvalidEntitlementId,
        )
        .map(|value| Self { value })
    }
}

impl ReviewRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        prefixed_ref(
            value.into(),
            REVIEW_REF_PREFIX,
            CloudMarketplaceError::InvalidReviewRef,
        )
        .map(|value| Self { value })
    }
}

impl KybEvidenceRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        prefixed_ref(
            value.into(),
            KYB_EVIDENCE_REF_PREFIX,
            CloudMarketplaceError::InvalidKybEvidenceRef,
        )
        .map(|value| Self { value })
    }
}

impl SupportRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        prefixed_ref(
            value.into(),
            SUPPORT_REF_PREFIX,
            CloudMarketplaceError::InvalidSupportRef,
        )
        .map(|value| Self { value })
    }
}

impl LegalTermsRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        prefixed_ref(
            value.into(),
            LEGAL_TERMS_REF_PREFIX,
            CloudMarketplaceError::InvalidLegalTermsRef,
        )
        .map(|value| Self { value })
    }
}

impl ProvisioningHookRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        prefixed_ref(
            value.into(),
            PROVISIONING_HOOK_PREFIX,
            CloudMarketplaceError::InvalidProvisioningHookRef,
        )
        .map(|value| Self { value })
    }
}

impl ListingCategory {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudMarketplaceError> {
        let value = value.into();
        if is_public_slug(&value) {
            Ok(Self { value })
        } else {
            Err(CloudMarketplaceError::InvalidCategory)
        }
    }
}

impl RevenueShare {
    pub fn new(input: RevenueShareCreate) -> Result<Self, CloudMarketplaceError> {
        if input.seller_bps == 0
            || input.platform_bps == 0
            || input.seller_bps + input.platform_bps != 10_000
            || input.seller_bps < 5_000
            || input.seller_bps > 9_500
        {
            return Err(CloudMarketplaceError::InvalidRevenueShare);
        }
        let settlement_currency =
            CurrencyCode::new(input.settlement_currency).map_err(map_billing_error)?;
        validate_marketplace_currency(&settlement_currency)?;
        Ok(Self {
            seller_bps: input.seller_bps,
            platform_bps: input.platform_bps,
            payout_cadence: input.payout_cadence,
            settlement_currency,
        })
    }
}

impl CloudMarketplaceSeller {
    pub fn apply(input: SellerApplicationCreate) -> Result<Self, CloudMarketplaceError> {
        validate_nonzero_time(input.created_at_epoch_seconds)?;
        Ok(Self {
            id: internal(SellerId::new(input.id)?),
            legal_name: internal(non_empty_limited(
                input.legal_name,
                2,
                160,
                CloudMarketplaceError::InvalidSellerId,
            )?),
            home_region: public(
                RegionCode::new(input.home_region)
                    .map_err(|_| CloudMarketplaceError::InvalidRegion)?,
            ),
            kyb_evidence_ref: internal(KybEvidenceRef::new(input.kyb_evidence_ref)?),
            support_ref: public(SupportRef::new(input.support_ref)?),
            trust_tier: public(None),
            security_review_ref: internal(None),
            state: public(SellerState::Applying),
            data_class: internal_class(input.data_class)?,
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            verified_at_epoch_seconds: internal(None),
            schema_version: public(MARKETPLACE_SCHEMA_VERSION),
        })
    }

    pub fn verified(&self, input: SellerVerificationCreate) -> Result<Self, CloudMarketplaceError> {
        if self.state.value != SellerState::Applying {
            return Err(CloudMarketplaceError::InvalidSellerState);
        }
        let seller_id = SellerId::new(input.seller_id)?;
        if seller_id != self.id.value {
            return Err(CloudMarketplaceError::UnknownSeller);
        }
        if input.verified_at_epoch_seconds <= self.created_at_epoch_seconds.value {
            return Err(CloudMarketplaceError::InvalidTimeOrder);
        }
        if input.trust_tier != MarketplaceTrustTier::VerifiedIsv {
            return Err(CloudMarketplaceError::InvalidTrustTier);
        }
        let mut seller = self.clone();
        seller.trust_tier = public(Some(input.trust_tier));
        seller.security_review_ref = internal(Some(ReviewRef::new(input.security_review_ref)?));
        seller.state = public(SellerState::Verified);
        seller.verified_at_epoch_seconds = internal(Some(input.verified_at_epoch_seconds));
        Ok(seller)
    }
}

impl CloudMarketplaceListing {
    pub fn draft(
        seller: &CloudMarketplaceSeller,
        input: ListingDraftCreate,
    ) -> Result<Self, CloudMarketplaceError> {
        if seller.state.value != SellerState::Verified
            || seller.trust_tier.value != Some(MarketplaceTrustTier::VerifiedIsv)
        {
            return Err(CloudMarketplaceError::InvalidSellerState);
        }
        let seller_id = SellerId::new(input.seller_id)?;
        if seller_id != seller.id.value {
            return Err(CloudMarketplaceError::UnknownSeller);
        }
        validate_nonzero_time(input.created_at_epoch_seconds)?;
        let categories = listing_categories(input.categories)?;
        let supported_regions = regions(input.supported_regions)?;
        Ok(Self {
            id: internal(MarketplaceListingId::new(input.id)?),
            seller_id: internal(seller_id),
            version: public(validate_version(input.version)?),
            title: public(non_empty_limited(
                input.title,
                3,
                120,
                CloudMarketplaceError::InvalidTitle,
            )?),
            summary: public(non_empty_limited(
                input.summary,
                12,
                500,
                CloudMarketplaceError::InvalidSummary,
            )?),
            categories: public(categories),
            supported_regions: public(supported_regions),
            provisioning_hook: internal(ProvisioningHookRef::new(input.provisioning_hook)?),
            support_ref: public(SupportRef::new(input.support_ref)?),
            billing_model: public(input.billing_model),
            revenue_share: financial(RevenueShare::new(input.revenue_share)?),
            security_review_ref: internal(None),
            license_review_ref: internal(None),
            state: public(ListingState::Draft),
            data_class: public_class(input.data_class)?,
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            published_at_epoch_seconds: internal(None),
            schema_version: public(MARKETPLACE_SCHEMA_VERSION),
        })
    }

    pub fn published(
        &self,
        input: ListingPublicationCreate,
    ) -> Result<Self, CloudMarketplaceError> {
        if self.state.value != ListingState::Draft {
            return Err(CloudMarketplaceError::InvalidListingState);
        }
        let listing_id = MarketplaceListingId::new(input.listing_id)?;
        if listing_id != self.id.value {
            return Err(CloudMarketplaceError::UnknownListing);
        }
        if input.published_at_epoch_seconds <= self.created_at_epoch_seconds.value {
            return Err(CloudMarketplaceError::InvalidTimeOrder);
        }
        let mut listing = self.clone();
        listing.security_review_ref = internal(Some(ReviewRef::new(input.security_review_ref)?));
        listing.license_review_ref = internal(Some(ReviewRef::new(input.license_review_ref)?));
        listing.state = public(ListingState::Published);
        listing.published_at_epoch_seconds = internal(Some(input.published_at_epoch_seconds));
        Ok(listing)
    }
}

impl CloudPrivateOffer {
    pub fn offered(
        listing: &CloudMarketplaceListing,
        input: PrivateOfferCreate,
    ) -> Result<Self, CloudMarketplaceError> {
        if listing.state.value != ListingState::Published {
            return Err(CloudMarketplaceError::InvalidListingState);
        }
        validate_tenant_id(&input.tenant_id)?;
        let listing_id = MarketplaceListingId::new(input.listing_id)?;
        if listing_id != listing.id.value {
            return Err(CloudMarketplaceError::UnknownListing);
        }
        if input.starts_at_epoch_seconds == 0
            || input.expires_at_epoch_seconds <= input.starts_at_epoch_seconds
        {
            return Err(CloudMarketplaceError::InvalidTimeOrder);
        }
        if input.discount_bps > MAX_PRIVATE_OFFER_DISCOUNT_BPS {
            return Err(CloudMarketplaceError::InvalidDiscount);
        }
        validate_marketplace_currency(&input.fixed_price.currency)?;
        Ok(Self {
            id: internal(PrivateOfferId::new(input.id)?),
            listing_id: internal(listing_id),
            tenant_id: internal(input.tenant_id),
            billing_account_id: internal(
                BillingAccountId::new(input.billing_account_id).map_err(map_billing_error)?,
            ),
            legal_terms_ref: internal(LegalTermsRef::new(input.legal_terms_ref)?),
            fixed_price: financial(input.fixed_price),
            discount_bps: financial(input.discount_bps),
            starts_at_epoch_seconds: internal(input.starts_at_epoch_seconds),
            expires_at_epoch_seconds: internal(input.expires_at_epoch_seconds),
            accepted_at_epoch_seconds: internal(None),
            state: internal(PrivateOfferState::Offered),
            data_class: financial_class(input.data_class)?,
            schema_version: public(MARKETPLACE_SCHEMA_VERSION),
        })
    }

    pub fn accepted(&self, accepted_at_epoch_seconds: u64) -> Result<Self, CloudMarketplaceError> {
        if self.state.value != PrivateOfferState::Offered {
            return Err(CloudMarketplaceError::InvalidOfferState);
        }
        if accepted_at_epoch_seconds < self.starts_at_epoch_seconds.value
            || accepted_at_epoch_seconds >= self.expires_at_epoch_seconds.value
        {
            return Err(CloudMarketplaceError::InvalidTimeOrder);
        }
        let mut offer = self.clone();
        offer.accepted_at_epoch_seconds = internal(Some(accepted_at_epoch_seconds));
        offer.state = internal(PrivateOfferState::Accepted);
        Ok(offer)
    }
}

impl CloudMarketplaceEntitlement {
    pub fn active(
        listing: &CloudMarketplaceListing,
        accepted_offer: Option<&CloudPrivateOffer>,
        input: EntitlementCreate,
    ) -> Result<Self, CloudMarketplaceError> {
        if listing.state.value != ListingState::Published {
            return Err(CloudMarketplaceError::InvalidListingState);
        }
        validate_tenant_id(&input.tenant_id)?;
        let listing_id = MarketplaceListingId::new(input.listing_id)?;
        if listing_id != listing.id.value {
            return Err(CloudMarketplaceError::UnknownListing);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudMarketplaceError::InvalidRegion)?;
        if !listing.supported_regions.value.contains(&region) {
            return Err(CloudMarketplaceError::RegionNotSupported);
        }
        let accepted_private_offer_id = match (input.accepted_private_offer_id, accepted_offer) {
            (Some(id), Some(offer)) => {
                let id = PrivateOfferId::new(id)?;
                if id != offer.id.value
                    || offer.listing_id.value != listing.id.value
                    || offer.tenant_id.value != input.tenant_id
                    || offer.state.value != PrivateOfferState::Accepted
                {
                    return Err(CloudMarketplaceError::UnknownPrivateOffer);
                }
                Some(id)
            }
            (None, None) => None,
            _ => return Err(CloudMarketplaceError::UnknownPrivateOffer),
        };
        validate_nonzero_time(input.requested_at_epoch_seconds)?;
        Ok(Self {
            id: internal(EntitlementId::new(input.id)?),
            listing_id: internal(listing.id.value.clone()),
            seller_id: internal(listing.seller_id.value.clone()),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            billing_account_id: internal(
                BillingAccountId::new(input.billing_account_id).map_err(map_billing_error)?,
            ),
            accepted_private_offer_id: internal(accepted_private_offer_id),
            provisioning_hook: internal(listing.provisioning_hook.value.clone()),
            state: internal(EntitlementState::Active),
            data_class: internal_class(input.data_class)?,
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            schema_version: public(MARKETPLACE_SCHEMA_VERSION),
        })
    }
}

impl MarketplaceRepo for CloudMarketplaceCatalog {
    fn submit_seller_application(
        &mut self,
        input: SellerApplicationCreate,
    ) -> Result<CloudMarketplaceSeller, CloudMarketplaceError> {
        let seller = CloudMarketplaceSeller::apply(input)?;
        if self.sellers.contains_key(&seller.id.value) {
            return Err(CloudMarketplaceError::DuplicateSeller);
        }
        self.sellers.insert(seller.id.value.clone(), seller.clone());
        Ok(seller)
    }

    fn verify_seller(
        &mut self,
        input: SellerVerificationCreate,
    ) -> Result<CloudMarketplaceSeller, CloudMarketplaceError> {
        let seller_id = SellerId::new(input.seller_id.clone())?;
        let seller = self
            .sellers
            .get(&seller_id)
            .ok_or(CloudMarketplaceError::UnknownSeller)?;
        let seller = seller.verified(input)?;
        self.sellers.insert(seller_id, seller.clone());
        Ok(seller)
    }

    fn create_listing_draft(
        &mut self,
        input: ListingDraftCreate,
    ) -> Result<CloudMarketplaceListing, CloudMarketplaceError> {
        let seller_id = SellerId::new(input.seller_id.clone())?;
        let seller = self
            .sellers
            .get(&seller_id)
            .ok_or(CloudMarketplaceError::UnknownSeller)?;
        let listing = CloudMarketplaceListing::draft(seller, input)?;
        if self.listings.contains_key(&listing.id.value) {
            return Err(CloudMarketplaceError::DuplicateListing);
        }
        self.listings
            .insert(listing.id.value.clone(), listing.clone());
        Ok(listing)
    }

    fn publish_listing(
        &mut self,
        input: ListingPublicationCreate,
    ) -> Result<CloudMarketplaceListing, CloudMarketplaceError> {
        let listing_id = MarketplaceListingId::new(input.listing_id.clone())?;
        let listing = self
            .listings
            .get(&listing_id)
            .ok_or(CloudMarketplaceError::UnknownListing)?;
        let listing = listing.published(input)?;
        self.listings.insert(listing_id, listing.clone());
        Ok(listing)
    }

    fn create_private_offer(
        &mut self,
        input: PrivateOfferCreate,
    ) -> Result<CloudPrivateOffer, CloudMarketplaceError> {
        let listing_id = MarketplaceListingId::new(input.listing_id.clone())?;
        let listing = self
            .listings
            .get(&listing_id)
            .ok_or(CloudMarketplaceError::UnknownListing)?;
        let offer = CloudPrivateOffer::offered(listing, input)?;
        if self.private_offers.contains_key(&offer.id.value) {
            return Err(CloudMarketplaceError::DuplicatePrivateOffer);
        }
        self.private_offers
            .insert(offer.id.value.clone(), offer.clone());
        Ok(offer)
    }

    fn accept_private_offer(
        &mut self,
        id: String,
        accepted_at_epoch_seconds: u64,
    ) -> Result<CloudPrivateOffer, CloudMarketplaceError> {
        let offer_id = PrivateOfferId::new(id)?;
        let offer = self
            .private_offers
            .get(&offer_id)
            .ok_or(CloudMarketplaceError::UnknownPrivateOffer)?;
        let offer = offer.accepted(accepted_at_epoch_seconds)?;
        self.private_offers.insert(offer_id, offer.clone());
        Ok(offer)
    }

    fn activate_entitlement(
        &mut self,
        input: EntitlementCreate,
    ) -> Result<CloudMarketplaceEntitlement, CloudMarketplaceError> {
        let listing_id = MarketplaceListingId::new(input.listing_id.clone())?;
        let offer = input
            .accepted_private_offer_id
            .as_ref()
            .map(|id| PrivateOfferId::new(id.clone()))
            .transpose()?
            .and_then(|id| self.private_offers.get(&id));
        let listing = self
            .listings
            .get(&listing_id)
            .ok_or(CloudMarketplaceError::UnknownListing)?;
        let entitlement = CloudMarketplaceEntitlement::active(listing, offer, input)?;
        if self.entitlements.contains_key(&entitlement.id.value) {
            return Err(CloudMarketplaceError::DuplicateEntitlement);
        }
        self.entitlements
            .insert(entitlement.id.value.clone(), entitlement.clone());
        Ok(entitlement)
    }

    fn record_marketplace_fee(
        &mut self,
        input: MarketplaceFeeCreate,
    ) -> Result<MeterEvent, CloudMarketplaceError> {
        validate_tenant_id(&input.tenant_id)?;
        let entitlement_id = EntitlementId::new(input.entitlement_id)?;
        let entitlement = self
            .entitlements
            .get(&entitlement_id)
            .ok_or(CloudMarketplaceError::UnknownEntitlement)?;
        if entitlement.tenant_id.value != input.tenant_id
            || entitlement.state.value != EntitlementState::Active
        {
            return Err(CloudMarketplaceError::UnknownEntitlement);
        }
        let event = self
            .meter
            .record(MeterEventCreate {
                id: input.meter_event_id,
                tenant_id: input.tenant_id,
                capability_id: MARKETPLACE_FEE_CAPABILITY_ID.to_string(),
                plane: PlaneTag::Control,
                units: input.units,
                source_axis: AxisId::Marketplace,
                recorded_at_epoch_seconds: input.recorded_at_epoch_seconds,
                idempotency_key: input.idempotency_key,
                data_class: input.data_class,
            })
            .map_err(map_metering_error)?;
        Ok(event)
    }
}

impl CloudMarketplaceCatalog {
    pub fn sellers(&self) -> impl Iterator<Item = &CloudMarketplaceSeller> {
        self.sellers.values()
    }

    pub fn listings(&self) -> impl Iterator<Item = &CloudMarketplaceListing> {
        self.listings.values()
    }

    pub fn private_offers(&self) -> impl Iterator<Item = &CloudPrivateOffer> {
        self.private_offers.values()
    }

    pub fn entitlements(&self) -> impl Iterator<Item = &CloudMarketplaceEntitlement> {
        self.entitlements.values()
    }

    pub fn meter_events(&self) -> impl Iterator<Item = &MeterEvent> {
        self.meter.events()
    }
}

fn listing_categories(input: Vec<String>) -> Result<Vec<ListingCategory>, CloudMarketplaceError> {
    if input.is_empty() {
        return Err(CloudMarketplaceError::InvalidCategory);
    }
    let mut seen = BTreeSet::new();
    let mut categories = Vec::with_capacity(input.len());
    for category in input {
        let category = ListingCategory::new(category)?;
        if !seen.insert(category.clone()) {
            return Err(CloudMarketplaceError::InvalidCategory);
        }
        categories.push(category);
    }
    Ok(categories)
}

fn regions(input: Vec<String>) -> Result<Vec<RegionCode>, CloudMarketplaceError> {
    if input.is_empty() {
        return Err(CloudMarketplaceError::InvalidRegion);
    }
    let mut seen = BTreeSet::new();
    let mut regions = Vec::with_capacity(input.len());
    for region in input {
        let region = RegionCode::new(region).map_err(|_| CloudMarketplaceError::InvalidRegion)?;
        if !seen.insert(region.clone()) {
            return Err(CloudMarketplaceError::InvalidRegion);
        }
        regions.push(region);
    }
    Ok(regions)
}

fn validate_version(value: String) -> Result<String, CloudMarketplaceError> {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        Ok(value)
    } else {
        Err(CloudMarketplaceError::InvalidVersion)
    }
}

fn non_empty_limited(
    value: String,
    min: usize,
    max: usize,
    error: CloudMarketplaceError,
) -> Result<String, CloudMarketplaceError> {
    let trimmed = value.trim();
    if (min..=max).contains(&trimmed.len()) && !trimmed.bytes().any(|byte| byte.is_ascii_control())
    {
        Ok(trimmed.to_string())
    } else {
        Err(error)
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudMarketplaceError> {
    if let Some(segment) = value.strip_prefix(TENANT_ID_PREFIX)
        && is_canonical_tenant_segment(segment)
    {
        Ok(())
    } else {
        Err(CloudMarketplaceError::InvalidTenantId)
    }
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

fn validate_nonzero_time(value: u64) -> Result<(), CloudMarketplaceError> {
    if value == 0 {
        Err(CloudMarketplaceError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_marketplace_currency(value: &CurrencyCode) -> Result<(), CloudMarketplaceError> {
    if matches!(value.value.as_str(), "OYC" | "USD") {
        Ok(())
    } else {
        Err(CloudMarketplaceError::InvalidCurrency)
    }
}

fn prefixed_token(
    value: String,
    prefix: &str,
    error: CloudMarketplaceError,
) -> Result<String, CloudMarketplaceError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

fn prefixed_ref(
    value: String,
    prefix: &str,
    error: CloudMarketplaceError,
) -> Result<String, CloudMarketplaceError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

fn is_public_slug(value: &str) -> bool {
    (2..=64).contains(&value.len())
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn public_class(
    data_class: DataClass,
) -> Result<Classified<PrivacyDataClass>, CloudMarketplaceError> {
    let class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudMarketplaceError::InvalidDataClass)?;
    if class.data_class() == DataClass::Public {
        Ok(public(class))
    } else {
        Err(CloudMarketplaceError::InvalidDataClass)
    }
}

fn internal_class(
    data_class: DataClass,
) -> Result<Classified<PrivacyDataClass>, CloudMarketplaceError> {
    let class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudMarketplaceError::InvalidDataClass)?;
    if class.data_class() == DataClass::InternalOnly {
        Ok(internal(class))
    } else {
        Err(CloudMarketplaceError::InvalidDataClass)
    }
}

fn financial_class(
    data_class: DataClass,
) -> Result<Classified<PrivacyDataClass>, CloudMarketplaceError> {
    let class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudMarketplaceError::InvalidDataClass)?;
    if matches!(class.data_class(), DataClass::Financial) {
        Ok(financial(class))
    } else {
        Err(CloudMarketplaceError::InvalidDataClass)
    }
}

fn map_billing_error(error: CloudBillingError) -> CloudMarketplaceError {
    match error {
        CloudBillingError::InvalidBillingAccountId => CloudMarketplaceError::BillingAccountInvalid,
        CloudBillingError::InvalidCurrencyCode => CloudMarketplaceError::InvalidCurrency,
        _ => CloudMarketplaceError::BillingAccountInvalid,
    }
}

fn map_metering_error(_error: MeteringError) -> CloudMarketplaceError {
    CloudMarketplaceError::MeteringRejected
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

fn financial<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Financial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use billing_metering::{MeterUnit, MeterUnitKind};

    fn seller_application() -> SellerApplicationCreate {
        SellerApplicationCreate {
            id: "isv_observability_plus".to_string(),
            legal_name: "Observability Plus Inc".to_string(),
            home_region: "region-alpha1".to_string(),
            kyb_evidence_ref: "kyb/region-alpha1/observability-plus".to_string(),
            support_ref: "support/observability-plus".to_string(),
            data_class: DataClass::InternalOnly,
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn seller_verification() -> SellerVerificationCreate {
        SellerVerificationCreate {
            seller_id: "isv_observability_plus".to_string(),
            trust_tier: MarketplaceTrustTier::VerifiedIsv,
            security_review_ref: "review/security/observability-plus/2026q2".to_string(),
            verified_at_epoch_seconds: 1_700_000_100,
        }
    }

    fn listing_draft() -> ListingDraftCreate {
        ListingDraftCreate {
            id: "cml_observability_agent".to_string(),
            seller_id: "isv_observability_plus".to_string(),
            version: "1.2.3".to_string(),
            title: "Observability Agent".to_string(),
            summary: "Cloud-native telemetry collector for tenant workloads".to_string(),
            categories: vec!["observability".to_string(), "sre".to_string()],
            supported_regions: vec!["region-alpha1".to_string(), "region-beta1".to_string()],
            provisioning_hook: "cap.cloud.marketplace.provision-observability-agent".to_string(),
            support_ref: "support/observability-plus".to_string(),
            billing_model: BillingModel::UsageMetered,
            revenue_share: RevenueShareCreate {
                seller_bps: 8_500,
                platform_bps: 1_500,
                payout_cadence: PayoutCadence::Monthly,
                settlement_currency: "OYC".to_string(),
            },
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_200,
        }
    }

    fn publication() -> ListingPublicationCreate {
        ListingPublicationCreate {
            listing_id: "cml_observability_agent".to_string(),
            security_review_ref: "review/security/listing/observability-agent".to_string(),
            license_review_ref: "review/license/listing/observability-agent".to_string(),
            published_at_epoch_seconds: 1_700_000_300,
        }
    }

    fn private_offer() -> PrivateOfferCreate {
        PrivateOfferCreate {
            id: "cpo_observability_agent_tenant_alpha".to_string(),
            listing_id: "cml_observability_agent".to_string(),
            tenant_id: "ten_alpha".to_string(),
            billing_account_id: "ba_alpha_cloud".to_string(),
            legal_terms_ref: "legal/private-offer/observability-agent/ten-alpha".to_string(),
            fixed_price: Money::new("OYC", 50_000_000).expect("valid money"),
            discount_bps: 1_500,
            starts_at_epoch_seconds: 1_700_000_400,
            expires_at_epoch_seconds: 1_700_086_800,
            data_class: DataClass::Financial,
        }
    }

    fn entitlement(offer_id: Option<&str>) -> EntitlementCreate {
        EntitlementCreate {
            id: "cme_observability_agent_tenant_alpha".to_string(),
            listing_id: "cml_observability_agent".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha1".to_string(),
            billing_account_id: "ba_alpha_cloud".to_string(),
            accepted_private_offer_id: offer_id.map(str::to_string),
            requested_at_epoch_seconds: 1_700_000_500,
            data_class: DataClass::InternalOnly,
        }
    }

    fn verified_catalog() -> CloudMarketplaceCatalog {
        let mut catalog = CloudMarketplaceCatalog::default();
        catalog
            .submit_seller_application(seller_application())
            .expect("seller application");
        catalog
            .verify_seller(seller_verification())
            .expect("seller verified");
        catalog
    }

    fn published_catalog() -> CloudMarketplaceCatalog {
        let mut catalog = verified_catalog();
        catalog
            .create_listing_draft(listing_draft())
            .expect("listing draft");
        catalog
            .publish_listing(publication())
            .expect("published listing");
        catalog
    }

    #[test]
    fn seller_onboarding_requires_review_before_verified_state() {
        let mut catalog = CloudMarketplaceCatalog::default();
        let seller = catalog
            .submit_seller_application(seller_application())
            .expect("seller application is valid");
        assert_eq!(seller.state.value, SellerState::Applying);
        assert_eq!(seller.trust_tier.value, None);

        let tier_error = catalog
            .verify_seller(SellerVerificationCreate {
                trust_tier: MarketplaceTrustTier::Community,
                ..seller_verification()
            })
            .expect_err("cloud marketplace production sellers must be verified ISVs");
        assert_eq!(tier_error, CloudMarketplaceError::InvalidTrustTier);

        let verified = catalog
            .verify_seller(seller_verification())
            .expect("seller review promotes to verified");
        assert_eq!(verified.state.value, SellerState::Verified);
        assert_eq!(
            verified.trust_tier.value,
            Some(MarketplaceTrustTier::VerifiedIsv)
        );
    }

    #[test]
    fn listing_publication_requires_verified_seller_public_metadata_and_reviews() {
        let mut catalog = verified_catalog();
        let draft = catalog
            .create_listing_draft(listing_draft())
            .expect("listing draft is valid");
        assert_eq!(draft.state.value, ListingState::Draft);
        assert_eq!(draft.data_class.value.data_class(), DataClass::Public);
        assert_eq!(draft.categories.value.len(), 2);

        let published = catalog
            .publish_listing(publication())
            .expect("publication review is valid");
        assert_eq!(published.state.value, ListingState::Published);
        assert!(published.security_review_ref.value.is_some());
        assert!(published.license_review_ref.value.is_some());
    }

    #[test]
    fn rejects_listing_from_unverified_seller_bad_share_or_non_public_metadata() {
        let mut catalog = CloudMarketplaceCatalog::default();
        catalog
            .submit_seller_application(seller_application())
            .expect("seller application");
        let seller_error = catalog
            .create_listing_draft(listing_draft())
            .expect_err("unverified seller cannot draft cloud marketplace listings");
        assert_eq!(seller_error, CloudMarketplaceError::InvalidSellerState);

        let mut catalog = verified_catalog();
        let share_error = catalog
            .create_listing_draft(ListingDraftCreate {
                revenue_share: RevenueShareCreate {
                    seller_bps: 9_800,
                    platform_bps: 200,
                    payout_cadence: PayoutCadence::Monthly,
                    settlement_currency: "OYC".to_string(),
                },
                ..listing_draft()
            })
            .expect_err("revenue share bounds are exact");
        assert_eq!(share_error, CloudMarketplaceError::InvalidRevenueShare);

        let class_error = catalog
            .create_listing_draft(ListingDraftCreate {
                id: "cml_bad_class".to_string(),
                data_class: DataClass::InternalOnly,
                ..listing_draft()
            })
            .expect_err("listing metadata is public-indexable only");
        assert_eq!(class_error, CloudMarketplaceError::InvalidDataClass);
    }

    #[test]
    fn private_offer_requires_published_listing_terms_currency_and_time_order() {
        let mut catalog = published_catalog();
        let offer = catalog
            .create_private_offer(private_offer())
            .expect("private offer is valid");
        assert_eq!(offer.state.value, PrivateOfferState::Offered);
        assert_eq!(offer.fixed_price.value.currency.value, "OYC");

        let discount_error = catalog
            .create_private_offer(PrivateOfferCreate {
                id: "cpo_discount_too_high".to_string(),
                discount_bps: 5_001,
                ..private_offer()
            })
            .expect_err("private offer discounts are bounded");
        assert_eq!(discount_error, CloudMarketplaceError::InvalidDiscount);

        let currency_error = catalog
            .create_private_offer(PrivateOfferCreate {
                id: "cpo_bad_currency".to_string(),
                fixed_price: Money::new("ABC", 10_000).expect("valid fixture currency"),
                ..private_offer()
            })
            .expect_err("cloud marketplace settlement opens with OYC and USD");
        assert_eq!(currency_error, CloudMarketplaceError::InvalidCurrency);

        let time_error = catalog
            .create_private_offer(PrivateOfferCreate {
                id: "cpo_bad_time".to_string(),
                expires_at_epoch_seconds: 1_700_000_400,
                ..private_offer()
            })
            .expect_err("private offer expiry must be after start");
        assert_eq!(time_error, CloudMarketplaceError::InvalidTimeOrder);
    }

    #[test]
    fn entitlement_requires_published_listing_supported_region_and_accepted_offer() {
        let mut catalog = published_catalog();
        catalog
            .create_private_offer(private_offer())
            .expect("private offer");
        catalog
            .accept_private_offer(
                "cpo_observability_agent_tenant_alpha".to_string(),
                1_700_000_450,
            )
            .expect("accepted private offer");
        let active = catalog
            .activate_entitlement(entitlement(Some("cpo_observability_agent_tenant_alpha")))
            .expect("entitlement is valid");
        assert_eq!(active.state.value, EntitlementState::Active);
        assert_eq!(active.region.value.value, "region-alpha1");
        assert!(active.accepted_private_offer_id.value.is_some());

        let region_error = catalog
            .activate_entitlement(EntitlementCreate {
                id: "cme_wrong_region".to_string(),
                region: "region-gamma1".to_string(),
                accepted_private_offer_id: None,
                ..entitlement(None)
            })
            .expect_err("entitlement region must be listed");
        assert_eq!(region_error, CloudMarketplaceError::RegionNotSupported);
    }

    #[test]
    fn marketplace_fee_events_use_platform_metering_with_idempotency() {
        let mut catalog = published_catalog();
        catalog
            .activate_entitlement(entitlement(None))
            .expect("active entitlement");
        let units = vec![MeterUnit::new(MeterUnitKind::Request, 1_000_000).expect("unit")];
        let first = catalog
            .record_marketplace_fee(MarketplaceFeeCreate {
                meter_event_id: "mtr_market_fee_001".to_string(),
                entitlement_id: "cme_observability_agent_tenant_alpha".to_string(),
                tenant_id: "ten_alpha".to_string(),
                units: units.clone(),
                recorded_at_epoch_seconds: 1_700_000_600,
                idempotency_key: "idem_market_fee_001".to_string(),
                data_class: DataClass::Public,
            })
            .expect("meter event is valid");
        let replay = catalog
            .record_marketplace_fee(MarketplaceFeeCreate {
                meter_event_id: "mtr_market_fee_ignored".to_string(),
                entitlement_id: "cme_observability_agent_tenant_alpha".to_string(),
                tenant_id: "ten_alpha".to_string(),
                units,
                recorded_at_epoch_seconds: 1_700_000_601,
                idempotency_key: "idem_market_fee_001".to_string(),
                data_class: DataClass::Public,
            })
            .expect("same idempotency key replays original event");
        assert_eq!(first.id.value, replay.id.value);
        assert_eq!(first.source_axis.value, AxisId::Marketplace);
        assert_eq!(catalog.meter_events().count(), 1);
    }

    #[test]
    fn marketplace_tenant_references_use_canonical_tenant_segments() {
        let mut catalog = published_catalog();

        let private_offer_error = catalog
            .create_private_offer(PrivateOfferCreate {
                tenant_id: "ten_alpha ".to_string(),
                ..private_offer()
            })
            .expect_err("private offer tenant ids must be canonical");
        assert_eq!(private_offer_error, CloudMarketplaceError::InvalidTenantId);

        let entitlement_error = catalog
            .activate_entitlement(EntitlementCreate {
                tenant_id: "ten_Alpha".to_string(),
                ..entitlement(None)
            })
            .expect_err("entitlement tenant ids must be canonical");
        assert_eq!(entitlement_error, CloudMarketplaceError::InvalidTenantId);

        catalog
            .activate_entitlement(entitlement(None))
            .expect("active entitlement");
        let fee_error = catalog
            .record_marketplace_fee(MarketplaceFeeCreate {
                meter_event_id: "mtr_market_fee_bad_tenant".to_string(),
                entitlement_id: "cme_observability_agent_tenant_alpha".to_string(),
                tenant_id: "ten_alpha--east".to_string(),
                units: vec![MeterUnit::new(MeterUnitKind::Request, 1).expect("unit")],
                recorded_at_epoch_seconds: 1_700_000_600,
                idempotency_key: "idem_market_fee_bad_tenant".to_string(),
                data_class: DataClass::Public,
            })
            .expect_err("fee tenant ids must be canonical before metering");
        assert_eq!(fee_error, CloudMarketplaceError::InvalidTenantId);
    }
}
