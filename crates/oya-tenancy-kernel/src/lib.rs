//! Tenancy kernel final-shape contracts.
//!
//! This crate owns the cross-microservice tenancy primitives from ADR-0002 and
//! PRD-tenancy: [`TenantId`], immutable [`RegionBinding`], [`ResidencyClass`],
//! [`Tenant`], and the row-level isolation guard that every adapter can apply
//! before persistence-specific RLS policies run.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

const TENANT_SCHEMA_VERSION: u32 = 1;
const REGION_BINDING_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantKernelError {
    InvalidTenantId,
    InvalidTenantSlug,
    TenantSlugEmpty,
    TenantSlugTooLong {
        actual: usize,
    },
    TenantSlugInvalidChar,
    InvalidRegionCode,
    InvalidEvidenceRef,
    InvalidLegalName,
    InvalidRegulatoryPack,
    DuplicateRegulatoryPack,
    EmptyPlaneGrantSet,
    FailoverMatchesPrimary,
    PrimaryRegionDeniedForResidency,
    FailoverRegionDeniedForResidency,
    ResidencyBindingMismatch,
    CrossTenantAccessDenied {
        context_tenant_id: TenantId,
        record_tenant_id: TenantId,
    },
}

/// Globally unique tenant identifier owned by the tenancy kernel.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenantId(String); // data_class: INTERNAL_ONLY

impl TenantId {
    pub fn new(value: impl Into<String>) -> Result<Self, TenantKernelError> {
        let value = value.into();
        if is_valid_prefixed_token(&value, "ten_") {
            Ok(Self(value))
        } else {
            Err(TenantKernelError::InvalidTenantId)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for TenantId {
    type Err = TenantKernelError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Maximum permitted length of a `TenantSlug`. 128 bytes is wide enough for
/// any URL-safe tenant id customers actually use (typical ≤32) while bounded
/// for hash-table label cardinality and audit-chain field budgets.
pub const TENANT_SLUG_MAX_LEN: usize = 128;

/// Customer-facing tenant slug — the form a tenant id takes on the wire
/// (HTTP `x-tenant-id` header, URL path captures, etc.) BEFORE it's
/// translated into the internal canonical `TenantId` (which carries the
/// `ten_` prefix) via a directory lookup at the API boundary.
///
/// Grammar: 1..=`TENANT_SLUG_MAX_LEN` bytes of ASCII alphanumeric + `-` + `_`.
///
/// Per ADR-0095: this type centralizes the slug grammar that the HTTP tenant
/// middleware previously defined inline. Defense in depth — the type itself
/// enforces invariants, not just the middleware extracting it.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenantSlug(String); // data_class: TENANT_SCOPED (customer-facing identifier)

impl TenantSlug {
    pub fn try_new(value: impl Into<String>) -> Result<Self, TenantKernelError> {
        let value = value.into();
        if value.is_empty() {
            return Err(TenantKernelError::TenantSlugEmpty);
        }
        if value.len() > TENANT_SLUG_MAX_LEN {
            return Err(TenantKernelError::TenantSlugTooLong {
                actual: value.len(),
            });
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(TenantKernelError::TenantSlugInvalidChar);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for TenantSlug {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for TenantSlug {
    type Err = TenantKernelError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for TenantSlug {
    type Error = TenantKernelError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// Canonical region code carried by the tenant binding.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RegionCode(String); // data_class: INTERNAL_ONLY

impl RegionCode {
    pub fn new(value: impl Into<String>) -> Result<Self, TenantKernelError> {
        let value = value.into();
        if is_valid_region_code(&value) {
            Ok(Self(value))
        } else {
            Err(TenantKernelError::InvalidRegionCode)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RegionCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for RegionCode {
    type Err = TenantKernelError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Tenant residency class. The value is immutable once bound to a tenant.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResidencyClass {
    StrictKr,
    StrictEu,
    KrWithUsFailover,
    Global,
}

impl ResidencyClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::StrictKr => "strict_kr",
            Self::StrictEu => "strict_eu",
            Self::KrWithUsFailover => "kr_with_us_failover",
            Self::Global => "global",
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "strict_kr" => Some(Self::StrictKr),
            "strict_eu" => Some(Self::StrictEu),
            "kr_with_us_failover" => Some(Self::KrWithUsFailover),
            "global" => Some(Self::Global),
            _ => None,
        }
    }

    pub fn allows_primary_region(self, region: &RegionCode) -> bool {
        match self {
            Self::StrictKr | Self::KrWithUsFailover => region.as_str().starts_with("kr-"),
            Self::StrictEu => region.as_str().starts_with("eu-"),
            Self::Global => true,
        }
    }

    pub fn allows_failover_region(self, region: &RegionCode) -> bool {
        match self {
            Self::StrictKr | Self::StrictEu => false,
            Self::KrWithUsFailover => region.as_str().starts_with("us-"),
            Self::Global => true,
        }
    }
}

/// Immutable post-create region binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionBinding {
    primary: RegionCode,             // data_class: INTERNAL_ONLY
    failover: Option<RegionCode>,    // data_class: INTERNAL_ONLY
    residency_class: ResidencyClass, // data_class: INTERNAL_ONLY
    evidence_ref: String,            // data_class: INTERNAL_ONLY
    bound_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    schema_version: u32,             // data_class: PUBLIC
}

impl RegionBinding {
    pub fn new(
        primary: RegionCode,
        failover: Option<RegionCode>,
        residency_class: ResidencyClass,
        evidence_ref: impl Into<String>,
        bound_at_epoch_seconds: u64,
    ) -> Result<Self, TenantKernelError> {
        let evidence_ref = evidence_ref.into();
        if evidence_ref.trim().is_empty() {
            return Err(TenantKernelError::InvalidEvidenceRef);
        }
        if !residency_class.allows_primary_region(&primary) {
            return Err(TenantKernelError::PrimaryRegionDeniedForResidency);
        }
        if let Some(ref failover_region) = failover {
            if failover_region == &primary {
                return Err(TenantKernelError::FailoverMatchesPrimary);
            }
            if !residency_class.allows_failover_region(failover_region) {
                return Err(TenantKernelError::FailoverRegionDeniedForResidency);
            }
        }
        Ok(Self {
            primary,
            failover,
            residency_class,
            evidence_ref,
            bound_at_epoch_seconds,
            schema_version: REGION_BINDING_SCHEMA_VERSION,
        })
    }

    pub fn primary(&self) -> &RegionCode {
        &self.primary
    }

    pub fn failover(&self) -> Option<&RegionCode> {
        self.failover.as_ref()
    }

    pub fn residency_class(&self) -> ResidencyClass {
        self.residency_class
    }

    pub fn evidence_ref(&self) -> &str {
        &self.evidence_ref
    }

    pub fn bound_at_epoch_seconds(&self) -> u64 {
        self.bound_at_epoch_seconds
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TenantPlane {
    Control,
    Data,
    Analytics,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantPlaneGrants {
    planes: BTreeSet<TenantPlane>, // data_class: INTERNAL_ONLY
}

impl TenantPlaneGrants {
    pub fn new(planes: impl IntoIterator<Item = TenantPlane>) -> Result<Self, TenantKernelError> {
        let planes = planes.into_iter().collect::<BTreeSet<_>>();
        if planes.is_empty() {
            return Err(TenantKernelError::EmptyPlaneGrantSet);
        }
        Ok(Self { planes })
    }

    pub fn all() -> Self {
        Self {
            planes: [
                TenantPlane::Control,
                TenantPlane::Data,
                TenantPlane::Analytics,
            ]
            .into(),
        }
    }

    pub fn contains(&self, plane: TenantPlane) -> bool {
        self.planes.contains(&plane)
    }

    pub fn planes(&self) -> impl Iterator<Item = TenantPlane> + '_ {
        self.planes.iter().copied()
    }
}

/// Authoritative tenant shape consumed by all microservices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tenant {
    id: TenantId,                    // data_class: INTERNAL_ONLY
    legal_name: String,              // data_class: INTERNAL_ONLY
    region_binding: RegionBinding,   // data_class: INTERNAL_ONLY
    residency_class: ResidencyClass, // data_class: INTERNAL_ONLY
    regulatory_packs: Vec<String>,   // data_class: INTERNAL_ONLY
    plane_grants: TenantPlaneGrants, // data_class: INTERNAL_ONLY
    schema_version: u32,             // data_class: PUBLIC
}

impl Tenant {
    pub fn new(
        id: TenantId,
        legal_name: impl Into<String>,
        region_binding: RegionBinding,
        regulatory_packs: Vec<String>,
        plane_grants: TenantPlaneGrants,
    ) -> Result<Self, TenantKernelError> {
        let legal_name = legal_name.into();
        if legal_name.trim().is_empty() {
            return Err(TenantKernelError::InvalidLegalName);
        }
        validate_regulatory_packs(&regulatory_packs)?;
        let residency_class = region_binding.residency_class();
        Ok(Self {
            id,
            legal_name,
            region_binding,
            residency_class,
            regulatory_packs,
            plane_grants,
            schema_version: TENANT_SCHEMA_VERSION,
        })
    }

    pub fn id(&self) -> &TenantId {
        &self.id
    }

    pub fn legal_name(&self) -> &str {
        &self.legal_name
    }

    pub fn region_binding(&self) -> &RegionBinding {
        &self.region_binding
    }

    pub fn residency_class(&self) -> ResidencyClass {
        self.residency_class
    }

    pub fn regulatory_packs(&self) -> &[String] {
        &self.regulatory_packs
    }

    pub fn plane_grants(&self) -> &TenantPlaneGrants {
        &self.plane_grants
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

pub trait TenantContext {
    fn tenant_id(&self) -> &TenantId;
}

impl TenantContext for Tenant {
    fn tenant_id(&self) -> &TenantId {
        self.id()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticTenantContext {
    tenant_id: TenantId, // data_class: INTERNAL_ONLY
}

impl StaticTenantContext {
    pub fn new(tenant_id: TenantId) -> Self {
        Self { tenant_id }
    }
}

impl TenantContext for StaticTenantContext {
    fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
}

/// Tenant-owned row payload. Adapters can map this check to SQL RLS, document
/// partition keys, event partitioning, or search-index filters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantScopedRecord<T> {
    tenant_id: TenantId, // data_class: INTERNAL_ONLY
    payload: T,          // data_class: TENANT_PAYLOAD
}

impl<T> TenantScopedRecord<T> {
    pub fn new(tenant_id: TenantId, payload: T) -> Self {
        Self { tenant_id, payload }
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn payload_for<C: TenantContext>(&self, context: &C) -> Result<&T, TenantKernelError> {
        if context.tenant_id() == &self.tenant_id {
            Ok(&self.payload)
        } else {
            Err(TenantKernelError::CrossTenantAccessDenied {
                context_tenant_id: context.tenant_id().clone(),
                record_tenant_id: self.tenant_id.clone(),
            })
        }
    }
}

fn validate_regulatory_packs(packs: &[String]) -> Result<(), TenantKernelError> {
    if packs.is_empty() {
        return Err(TenantKernelError::InvalidRegulatoryPack);
    }
    let mut seen = BTreeSet::new();
    for pack in packs {
        if !is_valid_pack_id(pack) {
            return Err(TenantKernelError::InvalidRegulatoryPack);
        }
        if !seen.insert(pack) {
            return Err(TenantKernelError::DuplicateRegulatoryPack);
        }
    }
    Ok(())
}

fn is_valid_prefixed_token(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() > prefix.len()
        && value[prefix.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn is_valid_region_code(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.len() >= 5
        && bytes.contains(&b'-')
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn is_valid_pack_id(value: &str) -> bool {
    value.starts_with("oya-pack-")
        && value.len() > "oya-pack-".len()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant_id(value: &str) -> TenantId {
        TenantId::new(value).expect("valid tenant id")
    }

    fn region(value: &str) -> RegionCode {
        RegionCode::new(value).expect("valid region")
    }

    fn kr_binding() -> RegionBinding {
        RegionBinding::new(
            region("kr-seoul"),
            None,
            ResidencyClass::StrictKr,
            "audit_evt_tenant_bound_001",
            1_762_992_000,
        )
        .expect("strict KR binding is valid")
    }

    #[test]
    fn tenant_id_is_a_validated_kernel_newtype() {
        assert_eq!(tenant_id("ten_kr").as_str(), "ten_kr");
        assert_eq!("ten_kr".parse::<TenantId>(), Ok(tenant_id("ten_kr")));
        assert_eq!(
            TenantId::new("tenant-kr"),
            Err(TenantKernelError::InvalidTenantId)
        );
    }

    #[test]
    fn region_binding_enforces_residency_primary_and_failover_rules() {
        assert_eq!(kr_binding().residency_class(), ResidencyClass::StrictKr);
        assert_eq!(kr_binding().schema_version(), REGION_BINDING_SCHEMA_VERSION);

        assert_eq!(
            RegionBinding::new(
                region("us-east"),
                None,
                ResidencyClass::StrictKr,
                "audit_evt_bad_region",
                1,
            ),
            Err(TenantKernelError::PrimaryRegionDeniedForResidency)
        );
        assert_eq!(
            RegionBinding::new(
                region("kr-seoul"),
                Some(region("jp-tokyo")),
                ResidencyClass::KrWithUsFailover,
                "audit_evt_bad_failover",
                1,
            ),
            Err(TenantKernelError::FailoverRegionDeniedForResidency)
        );
        assert!(
            RegionBinding::new(
                region("kr-seoul"),
                Some(region("us-east")),
                ResidencyClass::KrWithUsFailover,
                "audit_evt_good_failover",
                1,
            )
            .is_ok()
        );
    }

    #[test]
    fn tenant_contract_exposes_immutable_region_binding_by_accessors_only() {
        let tenant = Tenant::new(
            tenant_id("ten_kr"),
            "KR Tenant Ltd",
            kr_binding(),
            vec!["oya-pack-kr".to_string()],
            TenantPlaneGrants::all(),
        )
        .expect("tenant is valid");

        assert_eq!(tenant.id().as_str(), "ten_kr");
        assert_eq!(tenant.legal_name(), "KR Tenant Ltd");
        assert_eq!(tenant.region_binding().primary().as_str(), "kr-seoul");
        assert_eq!(tenant.residency_class().label(), "strict_kr");
        assert_eq!(tenant.schema_version(), TENANT_SCHEMA_VERSION);
        assert!(tenant.plane_grants().contains(TenantPlane::Control));
    }

    #[test]
    fn tenant_creation_rejects_empty_or_duplicate_regulatory_packs() {
        let base = (tenant_id("ten_kr"), "KR Tenant Ltd", kr_binding());
        assert_eq!(
            Tenant::new(
                base.0.clone(),
                base.1,
                base.2.clone(),
                vec![],
                TenantPlaneGrants::all(),
            ),
            Err(TenantKernelError::InvalidRegulatoryPack)
        );
        assert_eq!(
            Tenant::new(
                base.0,
                base.1,
                base.2,
                vec!["oya-pack-kr".to_string(), "oya-pack-kr".to_string()],
                TenantPlaneGrants::all(),
            ),
            Err(TenantKernelError::DuplicateRegulatoryPack)
        );
    }

    #[test]
    fn row_level_isolation_allows_same_tenant_and_denies_cross_tenant() {
        let record = TenantScopedRecord::new(tenant_id("ten_alpha"), "row payload");
        let same = StaticTenantContext::new(tenant_id("ten_alpha"));
        let other = StaticTenantContext::new(tenant_id("ten_beta"));

        assert_eq!(record.payload_for(&same), Ok(&"row payload"));
        assert_eq!(
            record.payload_for(&other),
            Err(TenantKernelError::CrossTenantAccessDenied {
                context_tenant_id: tenant_id("ten_beta"),
                record_tenant_id: tenant_id("ten_alpha"),
            })
        );
    }

    #[test]
    fn plane_grants_are_non_empty() {
        assert_eq!(
            TenantPlaneGrants::new([]),
            Err(TenantKernelError::EmptyPlaneGrantSet)
        );
        let grants =
            TenantPlaneGrants::new([TenantPlane::Control]).expect("grant set is non-empty");
        assert!(grants.contains(TenantPlane::Control));
        assert!(!grants.contains(TenantPlane::Analytics));
    }

    // ---- TenantSlug tests (ADR-0095 + F-MULTI-Q5) ----

    #[test]
    fn tenant_slug_accepts_alphanumeric_dash_underscore_at_short_lengths() {
        assert!(TenantSlug::try_new("acme").is_ok());
        assert!(TenantSlug::try_new("acme-co").is_ok());
        assert!(TenantSlug::try_new("acme_co").is_ok());
        assert!(TenantSlug::try_new("acme-co-123").is_ok());
        assert!(TenantSlug::try_new("A").is_ok()); // single uppercase
    }

    // F3 adversarial: every rejection path returns a specific error variant.
    #[test]
    fn tenant_slug_rejects_empty() {
        assert_eq!(
            TenantSlug::try_new(""),
            Err(TenantKernelError::TenantSlugEmpty)
        );
    }

    #[test]
    fn tenant_slug_rejects_too_long() {
        let too_long = "a".repeat(TENANT_SLUG_MAX_LEN + 1);
        assert_eq!(
            TenantSlug::try_new(&too_long),
            Err(TenantKernelError::TenantSlugTooLong {
                actual: TENANT_SLUG_MAX_LEN + 1,
            })
        );
    }

    #[test]
    fn tenant_slug_accepts_max_length() {
        let max_len = "a".repeat(TENANT_SLUG_MAX_LEN);
        assert!(TenantSlug::try_new(&max_len).is_ok());
    }

    #[test]
    fn tenant_slug_rejects_invalid_char_slash() {
        assert_eq!(
            TenantSlug::try_new("abc/def"),
            Err(TenantKernelError::TenantSlugInvalidChar)
        );
    }

    #[test]
    fn tenant_slug_rejects_invalid_char_space() {
        assert_eq!(
            TenantSlug::try_new("ab cd"),
            Err(TenantKernelError::TenantSlugInvalidChar)
        );
    }

    #[test]
    fn tenant_slug_rejects_invalid_char_unicode() {
        // Unicode letter that LOOKS like Latin 'a' but isn't ASCII —
        // homoglyph-class attack defense.
        assert_eq!(
            TenantSlug::try_new("аbc"),
            Err(TenantKernelError::TenantSlugInvalidChar)
        );
    }

    #[test]
    fn tenant_slug_rejects_dot_path_traversal_shape() {
        // S5 path-traversal class — dot segments must not survive a tenant
        // slug; the alphanumeric+dash+underscore rule excludes '.' which
        // closes the obvious traversal vector.
        assert_eq!(
            TenantSlug::try_new(".."),
            Err(TenantKernelError::TenantSlugInvalidChar)
        );
        assert_eq!(
            TenantSlug::try_new("."),
            Err(TenantKernelError::TenantSlugInvalidChar)
        );
        assert_eq!(
            TenantSlug::try_new("acme.co"),
            Err(TenantKernelError::TenantSlugInvalidChar)
        );
    }

    // F3 adversarial: try_from / parse APIs both delegate to try_new.
    #[test]
    fn tenant_slug_try_from_str_works() {
        let slug = TenantSlug::try_from("acme").unwrap();
        assert_eq!(slug.as_str(), "acme");
    }

    #[test]
    fn tenant_slug_from_str_parse_works() {
        let slug: TenantSlug = "acme-co".parse().unwrap();
        assert_eq!(slug.as_str(), "acme-co");
    }

    #[test]
    fn tenant_slug_invariants_documented_in_max_len_constant() {
        // Public surface declares the limit; consumers can read it without
        // parsing comments.
        assert_eq!(TENANT_SLUG_MAX_LEN, 128);
    }

    #[test]
    fn tenant_slug_is_distinct_from_tenant_id() {
        // TenantSlug is the customer-facing form; TenantId is internal
        // canonical. The internal grammar (ten_ prefix) does NOT validate
        // a slug, and a slug does NOT satisfy TenantId construction.
        assert!(TenantSlug::try_new("acme-co").is_ok());
        assert_eq!(
            TenantId::new("acme-co"),
            Err(TenantKernelError::InvalidTenantId)
        );
        // Conversely, "ten_kr" parses as both (internal IDs are slug-shaped).
        assert!(TenantSlug::try_new("ten_kr").is_ok());
        assert!(TenantId::new("ten_kr").is_ok());
    }
}
