//! SaaS plugin marketplace — manifest registry + trust-tier listing + ads auction kernel.
//!
//! Owns the public marketplace contracts required by M03-P04-IP-002 +
//! M03-P04-IP-003:
//! * [`PluginManifest`] — Cosign-signed plugin descriptor per ADR-0039,
//! * [`TrustTier`] — per ADR-0036 (Verified / Reviewed / Community),
//! * [`MarketplaceListing`] — per-vertical / per-region filterable index.
//!
//! M06-P01-IP-001 merge-variant delta-1 (2026-05-17):
//! * [`auction::Auction`] — internal-tenant ad slot auction,
//! * [`auction::Bid`] — single advertiser bid with tenant-isolation enforcement.
//!
//! M06-P02 extension (merge-variant delta-1):
//! * [`vertical_fanout`] — 13-vertical capability-pack roster (preview tier).
//!
//! No external Rust deps — std + workspace path deps only per ADR-0015.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod auction;
pub use auction::{
    Auction, AuctionCreate, AuctionError, AuctionId, AuctionState, Bid, BidCreate, BidId,
};
pub mod vertical_fanout;
pub use vertical_fanout::{
    FANOUT_SCHEMA_VERSION, FANOUT_VERTICAL_COUNT, FanoutError, FanoutRoster, FanoutTarget,
    FanoutVertical,
};

use std::collections::BTreeMap;

const PLUGIN_MANIFEST_ID_PREFIX: &str = "plg_";
const LISTING_ID_PREFIX: &str = "lst_";
const COSIGN_SIGNATURE_PREFIX: &str = "cosign:";
const REGIONAL_PACK_PREFIX: &str = "pack-";
const PLUGIN_MANIFEST_SCHEMA_VERSION: u32 = 1;
const MARKETPLACE_LISTING_SCHEMA_VERSION: u32 = 1;

/// Errors raised by manifest / listing validation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MarketplaceError {
    InvalidManifestId,
    InvalidListingId,
    InvalidPublisherId,
    InvalidVertical,
    InvalidRegionalPack,
    InvalidSemver,
    InvalidCosignSignature,
    DuplicateManifest,
    DuplicateListing,
    UnknownManifest,
    EmptyRegionalPacks,
    EmptyVerticals,
}

/// Plugin manifest identifier (Cosign-signed bundle).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PluginManifestId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl PluginManifestId {
    pub fn new(value: impl Into<String>) -> Result<Self, MarketplaceError> {
        prefixed(
            value.into(),
            PLUGIN_MANIFEST_ID_PREFIX,
            MarketplaceError::InvalidManifestId,
        )
        .map(|value| Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Marketplace listing identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MarketplaceListingId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl MarketplaceListingId {
    pub fn new(value: impl Into<String>) -> Result<Self, MarketplaceError> {
        prefixed(
            value.into(),
            LISTING_ID_PREFIX,
            MarketplaceError::InvalidListingId,
        )
        .map(|value| Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Cosign-attested signature reference (ADR-0039).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CosignSignature {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl CosignSignature {
    pub fn new(value: impl Into<String>) -> Result<Self, MarketplaceError> {
        prefixed(
            value.into(),
            COSIGN_SIGNATURE_PREFIX,
            MarketplaceError::InvalidCosignSignature,
        )
        .map(|value| Self { value })
    }
}

/// Trust tier of a marketplace listing per ADR-0036.
///
/// `Verified` listings have passed security review + supply-chain attestation.
/// `Reviewed` listings have passed code review only.
/// `Community` listings are sandbox-only and surfaced with an explicit warning.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TrustTier {
    Verified,
    Reviewed,
    Community,
}

/// Vertical taxonomy used to filter the marketplace listing index.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Vertical {
    Agentic,
    Development,
    Business,
    Healthcare,
    SupplyChain,
    Delivery,
}

/// Cosign-signed plugin manifest descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginManifest {
    pub id: PluginManifestId,              // data_class: INTERNAL_ONLY
    pub publisher_id: String,              // data_class: INTERNAL_ONLY
    pub name: String,                      // data_class: INTERNAL_ONLY
    pub semver: String,                    // data_class: INTERNAL_ONLY
    pub cosign_signature: CosignSignature, // data_class: INTERNAL_ONLY
    pub entrypoint: String,                // data_class: INTERNAL_ONLY
    pub registered_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub schema_version: u32,               // data_class: INTERNAL_ONLY
}

/// Public marketplace listing entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketplaceListing {
    pub id: MarketplaceListingId,        // data_class: INTERNAL_ONLY
    pub manifest_id: PluginManifestId,   // data_class: INTERNAL_ONLY
    pub trust_tier: TrustTier,           // data_class: INTERNAL_ONLY
    pub verticals: Vec<Vertical>,        // data_class: INTERNAL_ONLY
    pub regional_packs: Vec<String>,     // data_class: INTERNAL_ONLY
    pub headline: String,                // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,             // data_class: INTERNAL_ONLY
}

/// Inputs to `plugin.manifest.register`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginManifestRegister {
    pub id: String,                       // data_class: INTERNAL_ONLY
    pub publisher_id: String,             // data_class: INTERNAL_ONLY
    pub name: String,                     // data_class: INTERNAL_ONLY
    pub semver: String,                   // data_class: INTERNAL_ONLY
    pub cosign_signature: String,         // data_class: INTERNAL_ONLY
    pub entrypoint: String,               // data_class: INTERNAL_ONLY
    pub registered_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

/// Inputs to `marketplace.listing.publish`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketplaceListingPublish {
    pub id: String,                      // data_class: INTERNAL_ONLY
    pub manifest_id: String,             // data_class: INTERNAL_ONLY
    pub trust_tier: TrustTier,           // data_class: INTERNAL_ONLY
    pub verticals: Vec<Vertical>,        // data_class: INTERNAL_ONLY
    pub regional_packs: Vec<String>,     // data_class: INTERNAL_ONLY
    pub headline: String,                // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

/// In-process marketplace registry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MarketplaceRegistry {
    manifests: BTreeMap<PluginManifestId, PluginManifest>,
    listings: BTreeMap<MarketplaceListingId, MarketplaceListing>,
}

impl PluginManifest {
    /// Construct + validate a manifest from public input.
    pub fn register(input: PluginManifestRegister) -> Result<Self, MarketplaceError> {
        let id = PluginManifestId::new(input.id)?;
        if !is_publisher(&input.publisher_id) {
            return Err(MarketplaceError::InvalidPublisherId);
        }
        if input.name.trim().is_empty() || input.name.len() > 160 {
            return Err(MarketplaceError::InvalidManifestId);
        }
        if !is_semver(&input.semver) {
            return Err(MarketplaceError::InvalidSemver);
        }
        let cosign_signature = CosignSignature::new(input.cosign_signature)?;
        if input.entrypoint.trim().is_empty() {
            return Err(MarketplaceError::InvalidManifestId);
        }
        Ok(Self {
            id,
            publisher_id: input.publisher_id,
            name: input.name,
            semver: input.semver,
            cosign_signature,
            entrypoint: input.entrypoint,
            registered_at_epoch_seconds: input.registered_at_epoch_seconds,
            schema_version: PLUGIN_MANIFEST_SCHEMA_VERSION,
        })
    }
}

impl MarketplaceListing {
    /// Construct + validate a listing referencing a registered manifest.
    pub fn publish(
        manifest_id: PluginManifestId,
        input: MarketplaceListingPublish,
    ) -> Result<Self, MarketplaceError> {
        let id = MarketplaceListingId::new(input.id)?;
        if manifest_id.value != input.manifest_id {
            return Err(MarketplaceError::UnknownManifest);
        }
        if input.verticals.is_empty() {
            return Err(MarketplaceError::EmptyVerticals);
        }
        if input.regional_packs.is_empty() {
            return Err(MarketplaceError::EmptyRegionalPacks);
        }
        for pack in &input.regional_packs {
            if !pack.starts_with(REGIONAL_PACK_PREFIX) || pack.len() <= REGIONAL_PACK_PREFIX.len() {
                return Err(MarketplaceError::InvalidRegionalPack);
            }
        }
        if input.headline.trim().is_empty() || input.headline.len() > 240 {
            return Err(MarketplaceError::InvalidListingId);
        }
        Ok(Self {
            id,
            manifest_id,
            trust_tier: input.trust_tier,
            verticals: input.verticals,
            regional_packs: input.regional_packs,
            headline: input.headline,
            published_at_epoch_seconds: input.published_at_epoch_seconds,
            schema_version: MARKETPLACE_LISTING_SCHEMA_VERSION,
        })
    }

    pub fn matches(&self, vertical: Vertical, regional_pack: &str) -> bool {
        self.verticals.contains(&vertical) && self.regional_packs.iter().any(|p| p == regional_pack)
    }
}

impl MarketplaceRegistry {
    /// `plugin.manifest.register` — store a Cosign-signed manifest.
    pub fn register_manifest(
        &mut self,
        input: PluginManifestRegister,
    ) -> Result<PluginManifest, MarketplaceError> {
        let manifest = PluginManifest::register(input)?;
        if self.manifests.contains_key(&manifest.id) {
            return Err(MarketplaceError::DuplicateManifest);
        }
        self.manifests.insert(manifest.id.clone(), manifest.clone());
        Ok(manifest)
    }

    /// `marketplace.listing.publish` — publish a listing for a manifest.
    pub fn publish_listing(
        &mut self,
        input: MarketplaceListingPublish,
    ) -> Result<MarketplaceListing, MarketplaceError> {
        let manifest_id = PluginManifestId::new(input.manifest_id.clone())?;
        if !self.manifests.contains_key(&manifest_id) {
            return Err(MarketplaceError::UnknownManifest);
        }
        let listing = MarketplaceListing::publish(manifest_id, input)?;
        if self.listings.contains_key(&listing.id) {
            return Err(MarketplaceError::DuplicateListing);
        }
        self.listings.insert(listing.id.clone(), listing.clone());
        Ok(listing)
    }

    pub fn manifest(&self, id: &PluginManifestId) -> Option<&PluginManifest> {
        self.manifests.get(id)
    }

    pub fn listing(&self, id: &MarketplaceListingId) -> Option<&MarketplaceListing> {
        self.listings.get(id)
    }

    /// Filter listings by vertical + regional pack.
    pub fn filter(&self, vertical: Vertical, regional_pack: &str) -> Vec<&MarketplaceListing> {
        self.listings
            .values()
            .filter(|listing| listing.matches(vertical, regional_pack))
            .collect()
    }
}

fn prefixed(
    value: String,
    prefix: &str,
    error: MarketplaceError,
) -> Result<String, MarketplaceError> {
    if value.starts_with(prefix) && value.len() > prefix.len() {
        Ok(value)
    } else {
        Err(error)
    }
}

fn is_publisher(value: &str) -> bool {
    value.starts_with("pub_") && value.len() > "pub_".len()
}

fn is_semver(value: &str) -> bool {
    // Minimal MAJOR.MINOR.PATCH check (no pre-release/build metadata in preview).
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts
        .iter()
        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_input(id: &str) -> PluginManifestRegister {
        PluginManifestRegister {
            id: id.to_string(),
            publisher_id: "pub_acme".to_string(),
            name: "Summarizer".to_string(),
            semver: "1.2.3".to_string(),
            cosign_signature: "cosign:sha256:abc123".to_string(),
            entrypoint: "wasm/summarizer.wasm".to_string(),
            registered_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn listing_input(id: &str, manifest_id: &str) -> MarketplaceListingPublish {
        MarketplaceListingPublish {
            id: id.to_string(),
            manifest_id: manifest_id.to_string(),
            trust_tier: TrustTier::Verified,
            verticals: vec![Vertical::Agentic, Vertical::Development],
            regional_packs: vec!["pack-alpha".to_string(), "pack-beta".to_string()],
            headline: "Summarize anything".to_string(),
            published_at_epoch_seconds: 1_700_000_100,
        }
    }

    #[test]
    fn manifest_register_validates_id_publisher_semver_and_signature() {
        let m = PluginManifest::register(manifest_input("plg_sum_v1")).expect("valid manifest");
        assert_eq!(m.id.as_str(), "plg_sum_v1");
        assert_eq!(m.semver, "1.2.3");

        let bad_pub = PluginManifest::register(PluginManifestRegister {
            publisher_id: "acme".to_string(),
            ..manifest_input("plg_a")
        })
        .expect_err("publisher must use pub_ prefix");
        assert_eq!(bad_pub, MarketplaceError::InvalidPublisherId);

        let bad_semver = PluginManifest::register(PluginManifestRegister {
            semver: "v1".to_string(),
            ..manifest_input("plg_b")
        })
        .expect_err("semver must be MAJOR.MINOR.PATCH");
        assert_eq!(bad_semver, MarketplaceError::InvalidSemver);

        let bad_cosign = PluginManifest::register(PluginManifestRegister {
            cosign_signature: "openssl:".to_string(),
            ..manifest_input("plg_c")
        })
        .expect_err("cosign signature required");
        assert_eq!(bad_cosign, MarketplaceError::InvalidCosignSignature);
    }

    #[test]
    fn listing_publish_validates_verticals_regional_packs_and_manifest_match() {
        let manifest_id = PluginManifestId::new("plg_sum_v1").unwrap();
        let listing = MarketplaceListing::publish(
            manifest_id.clone(),
            listing_input("lst_sum_alpha", "plg_sum_v1"),
        )
        .expect("valid listing");
        assert!(listing.matches(Vertical::Agentic, "pack-alpha"));
        assert!(!listing.matches(Vertical::Healthcare, "pack-alpha"));

        let mismatch =
            MarketplaceListing::publish(manifest_id.clone(), listing_input("lst_mis", "plg_other"))
                .expect_err("listing manifest must match");
        assert_eq!(mismatch, MarketplaceError::UnknownManifest);

        let empty_verticals = MarketplaceListing::publish(
            manifest_id.clone(),
            MarketplaceListingPublish {
                verticals: vec![],
                ..listing_input("lst_a", "plg_sum_v1")
            },
        )
        .expect_err("verticals required");
        assert_eq!(empty_verticals, MarketplaceError::EmptyVerticals);

        let bad_pack = MarketplaceListing::publish(
            manifest_id,
            MarketplaceListingPublish {
                regional_packs: vec!["region-home".to_string()],
                ..listing_input("lst_b", "plg_sum_v1")
            },
        )
        .expect_err("regional pack prefix enforced");
        assert_eq!(bad_pack, MarketplaceError::InvalidRegionalPack);
    }

    #[test]
    fn registry_registers_manifests_once_and_filters_listings() {
        let mut reg = MarketplaceRegistry::default();
        reg.register_manifest(manifest_input("plg_sum_v1")).unwrap();
        let dup = reg
            .register_manifest(manifest_input("plg_sum_v1"))
            .expect_err("duplicate manifest rejected");
        assert_eq!(dup, MarketplaceError::DuplicateManifest);

        reg.publish_listing(listing_input("lst_alpha", "plg_sum_v1"))
            .unwrap();
        let dup_listing = reg
            .publish_listing(listing_input("lst_alpha", "plg_sum_v1"))
            .expect_err("duplicate listing rejected");
        assert_eq!(dup_listing, MarketplaceError::DuplicateListing);

        let agentic_kr = reg.filter(Vertical::Agentic, "pack-alpha");
        assert_eq!(agentic_kr.len(), 1);
        let healthcare = reg.filter(Vertical::Healthcare, "pack-alpha");
        assert!(healthcare.is_empty());
    }

    #[test]
    fn listing_without_known_manifest_rejected_by_registry() {
        let mut reg = MarketplaceRegistry::default();
        let err = reg
            .publish_listing(listing_input("lst_orphan", "plg_ghost"))
            .expect_err("manifest must be registered first");
        assert_eq!(err, MarketplaceError::UnknownManifest);
    }

    #[test]
    fn trust_tier_ordering_supports_tier_filtering() {
        assert!(TrustTier::Verified < TrustTier::Reviewed);
        assert!(TrustTier::Reviewed < TrustTier::Community);
    }
}
