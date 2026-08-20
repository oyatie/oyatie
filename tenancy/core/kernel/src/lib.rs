//! Tenancy kernel final-shape contracts.
//!
//! This crate owns the cross-microservice tenancy primitives from ADR-0002 and
//! PRD-tenancy: [`TenantId`], immutable [`RegionBinding`], [`ResidencyClass`],
//! [`Tenant`], the row-level isolation guard that every adapter can apply
//! before persistence-specific RLS policies run, and the tier/status value
//! objects introduced by IP-001-tenancy-kernel-scaffold (P13-tenancy).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod tier_status;

pub use tier_status::{
    SuspensionReason, TenantStatus, TenantStatusParseError, TenantTier, TenantTierParseError,
};

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
    UnknownEnvironmentTier,
    UnknownApiKeyPrefix,
    LongLivedRuntimeCredentialForbidden,
    InvalidRuntimeCredentialTtl,
    TenantMutationReviewLabelRequired,
    TenantMutationReviewEvidenceRequired,
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
    StrictHomeRegion,
    StrictFederatedRegion,
    HomeWithRecoveryFailover,
    Global,
}

impl ResidencyClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::StrictHomeRegion => "strict_home_region",
            Self::StrictFederatedRegion => "strict_federated_region",
            Self::HomeWithRecoveryFailover => "home_with_recovery_failover",
            Self::Global => "global",
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "strict_home_region" => Some(Self::StrictHomeRegion),
            "strict_federated_region" => Some(Self::StrictFederatedRegion),
            "home_with_recovery_failover" => Some(Self::HomeWithRecoveryFailover),
            "global" => Some(Self::Global),
            _ => None,
        }
    }

    pub fn allows_primary_region(self, region: &RegionCode) -> bool {
        match self {
            Self::StrictHomeRegion | Self::HomeWithRecoveryFailover => {
                region.as_str().starts_with("region-home")
            }
            Self::StrictFederatedRegion => region.as_str().starts_with("region-federated"),
            Self::Global => true,
        }
    }

    pub fn allows_failover_region(self, region: &RegionCode) -> bool {
        match self {
            Self::StrictHomeRegion | Self::StrictFederatedRegion => false,
            Self::HomeWithRecoveryFailover => region.as_str().starts_with("region-recovery"),
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

/// Tenant-mutation PRs touching the tenant struct, its derived types, or the
/// catalog row MUST carry this review label before the governance pipeline may
/// accept them (ADR-0002 cross-axis change-review class).
pub const CROSS_MICROSERVICE_TENANT_MUTATION_LABEL: &str = "cross-microservice-tenant-mutation";

/// API-gateway header required for prod-tier destructive operations (ADR-0163).
pub const PROD_DESTRUCTIVE_ACK_HEADER: &str = "x-oya-prod-destructive-ack";

/// Cedar context key that must be true for prod-tier destructive operations.
pub const PROD_DESTRUCTIVE_ACK_CEDAR_CONDITION: &str = "prod_destructive_acknowledged";

/// Conservative STS runtime credential lifetime. Product code may choose a
/// lower cap, but the tenancy/identity kernel refuses long-lived credentials.
pub const MAX_RUNTIME_CREDENTIAL_TTL_SECONDS: u64 = 3_600;

/// Per-tenant environment tier from ADR-0163. This is tenant lifecycle state,
/// not code-promotion state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TenantEnvironmentTier {
    Test,
    Staging,
    Prod,
}

impl TenantEnvironmentTier {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Staging => "staging",
            Self::Prod => "prod",
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "test" => Some(Self::Test),
            "staging" => Some(Self::Staging),
            "prod" => Some(Self::Prod),
            _ => None,
        }
    }

    pub const fn audit_chain_tag(self) -> &'static str {
        match self {
            Self::Test => "env_tier=test",
            Self::Staging => "env_tier=staging",
            Self::Prod => "env_tier=prod",
        }
    }

    pub const fn outbound_mode(self) -> OutboundSideEffectMode {
        match self {
            Self::Test => OutboundSideEffectMode::Intercept,
            Self::Staging => OutboundSideEffectMode::TestRecipients,
            Self::Prod => OutboundSideEffectMode::Live,
        }
    }

    pub const fn api_key_prefix(self, kind: TenantApiKeyKind) -> &'static str {
        match (self, kind) {
            (Self::Test, TenantApiKeyKind::Server) => "sk_test_",
            (Self::Test, TenantApiKeyKind::Public) => "pk_test_",
            (Self::Staging, TenantApiKeyKind::Server) => "sk_stage_",
            (Self::Staging, TenantApiKeyKind::Public) => "pk_stage_",
            (Self::Prod, TenantApiKeyKind::Server) => "sk_live_",
            (Self::Prod, TenantApiKeyKind::Public) => "pk_live_",
        }
    }

    pub fn from_api_key_prefix(value: &str) -> Result<(Self, TenantApiKeyKind), TenantKernelError> {
        for tier in [Self::Test, Self::Staging, Self::Prod] {
            for kind in [TenantApiKeyKind::Server, TenantApiKeyKind::Public] {
                if value.starts_with(tier.api_key_prefix(kind)) {
                    return Ok((tier, kind));
                }
            }
        }
        Err(TenantKernelError::UnknownApiKeyPrefix)
    }

    pub const fn requires_destructive_acknowledgment(self) -> bool {
        matches!(self, Self::Prod)
    }
}

impl fmt::Display for TenantEnvironmentTier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

impl FromStr for TenantEnvironmentTier {
    type Err = TenantKernelError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_label(value).ok_or(TenantKernelError::UnknownEnvironmentTier)
    }
}

/// API-key family; server keys are never browser-exposable, public keys are.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TenantApiKeyKind {
    Server,
    Public,
}

/// Outbound side-effect posture for email/SMS/webhook/billing dispatch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OutboundSideEffectMode {
    /// Test tier: record/audit the dispatch, but do not deliver externally.
    Intercept,
    /// Staging tier: deliver only to tenant-configured QA recipients.
    TestRecipients,
    /// Prod tier: live delivery to real recipients.
    Live,
}

impl OutboundSideEffectMode {
    pub const fn is_intercepted(self) -> bool {
        matches!(self, Self::Intercept)
    }

    pub const fn permits_live_delivery(self) -> bool {
        matches!(self, Self::Live)
    }

    pub const fn requires_test_recipient_allowlist(self) -> bool {
        matches!(self, Self::TestRecipients)
    }
}

/// Runtime credential classes accepted/refused at the tenancy/identity boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuntimeCredentialKind {
    StsShortLived {
        issued_at_epoch_seconds: u64,
        expires_at_epoch_seconds: u64,
    },
    StaticSecret,
    LongLivedServiceAccount,
}

impl RuntimeCredentialKind {
    pub const fn validate(self) -> Result<(), TenantKernelError> {
        match self {
            Self::StsShortLived {
                issued_at_epoch_seconds,
                expires_at_epoch_seconds,
            } => {
                if expires_at_epoch_seconds <= issued_at_epoch_seconds {
                    return Err(TenantKernelError::InvalidRuntimeCredentialTtl);
                }
                if expires_at_epoch_seconds - issued_at_epoch_seconds
                    > MAX_RUNTIME_CREDENTIAL_TTL_SECONDS
                {
                    return Err(TenantKernelError::InvalidRuntimeCredentialTtl);
                }
                Ok(())
            }
            Self::StaticSecret | Self::LongLivedServiceAccount => {
                Err(TenantKernelError::LongLivedRuntimeCredentialForbidden)
            }
        }
    }
}

/// Tenant contract surfaces that trigger all-axis mutation review.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TenantMutationKind {
    TenantStruct,
    DerivedType,
    CatalogRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantMutationReview {
    kind: TenantMutationKind,
    evidence_ref: String,
}

impl TenantMutationReview {
    pub fn new<I, S>(
        kind: TenantMutationKind,
        review_labels: I,
        evidence_ref: impl Into<String>,
    ) -> Result<Self, TenantKernelError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if !review_labels
            .into_iter()
            .any(|label| label.as_ref() == CROSS_MICROSERVICE_TENANT_MUTATION_LABEL)
        {
            return Err(TenantKernelError::TenantMutationReviewLabelRequired);
        }

        let evidence_ref = evidence_ref.into();
        if evidence_ref.trim().is_empty() {
            return Err(TenantKernelError::TenantMutationReviewEvidenceRequired);
        }

        Ok(Self { kind, evidence_ref })
    }

    pub fn kind(&self) -> TenantMutationKind {
        self.kind
    }

    pub fn evidence_ref(&self) -> &str {
        &self.evidence_ref
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

/// B2B-tier primitives for M03-P06 (Application B2B Shell Live).
///
/// `B2bTenantTier` classifies the commercial arrangement under which a B2B
/// tenant operates. The tier is set at onboarding and drives product-enablement
/// limits enforced by the application-shell use-cases.
///
/// ADR-0056 layer: kernel (pure value-object, no I/O, no alloc beyond `String`).
pub mod b2b_tenant_tier {
    use std::fmt;

    /// Commercial tier that governs which products and seat counts a B2B tenant
    /// may activate.
    ///
    /// Variant ordering is intentional: `Trial < Starter < Growth < Enterprise`
    /// so that `PartialOrd`-driven limit checks read naturally.
    #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
    pub enum B2bTenantTier {
        /// Time-limited evaluation; limited seat count, read-only integrations.
        Trial,
        /// Paid entry tier; up to 50 seats, core products only.
        Starter,
        /// Mid-market tier; up to 500 seats, all products enabled.
        Growth,
        /// Unlimited seats; dedicated support SLA, custom pack overlays allowed.
        Enterprise,
    }

    impl B2bTenantTier {
        /// Maximum seat count enforced by this tier (`None` = unlimited).
        pub fn seat_limit(&self) -> Option<u32> {
            match self {
                Self::Trial => Some(10),
                Self::Starter => Some(50),
                Self::Growth => Some(500),
                Self::Enterprise => None,
            }
        }

        /// Whether custom regional-pack overlays are permitted for this tier.
        pub fn allows_custom_pack_overlay(&self) -> bool {
            matches!(self, Self::Enterprise)
        }

        /// Machine-readable label used in audit-chain events and Cedar policies.
        pub fn label(&self) -> &'static str {
            match self {
                Self::Trial => "trial",
                Self::Starter => "starter",
                Self::Growth => "growth",
                Self::Enterprise => "enterprise",
            }
        }

        /// Parse from a canonical label string produced by [`Self::label`].
        pub fn try_parse(s: &str) -> Option<Self> {
            match s {
                "trial" => Some(Self::Trial),
                "starter" => Some(Self::Starter),
                "growth" => Some(Self::Growth),
                "enterprise" => Some(Self::Enterprise),
                _ => None,
            }
        }
    }

    impl fmt::Display for B2bTenantTier {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.label())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn tier_ordering_is_trial_starter_growth_enterprise() {
            assert!(B2bTenantTier::Trial < B2bTenantTier::Starter);
            assert!(B2bTenantTier::Starter < B2bTenantTier::Growth);
            assert!(B2bTenantTier::Growth < B2bTenantTier::Enterprise);
        }

        #[test]
        fn seat_limits_match_spec() {
            assert_eq!(B2bTenantTier::Trial.seat_limit(), Some(10));
            assert_eq!(B2bTenantTier::Starter.seat_limit(), Some(50));
            assert_eq!(B2bTenantTier::Growth.seat_limit(), Some(500));
            assert_eq!(B2bTenantTier::Enterprise.seat_limit(), None);
        }

        #[test]
        fn custom_pack_overlay_only_for_enterprise() {
            assert!(!B2bTenantTier::Trial.allows_custom_pack_overlay());
            assert!(!B2bTenantTier::Starter.allows_custom_pack_overlay());
            assert!(!B2bTenantTier::Growth.allows_custom_pack_overlay());
            assert!(B2bTenantTier::Enterprise.allows_custom_pack_overlay());
        }

        #[test]
        fn label_round_trips_through_try_parse() {
            let tiers = [
                B2bTenantTier::Trial,
                B2bTenantTier::Starter,
                B2bTenantTier::Growth,
                B2bTenantTier::Enterprise,
            ];
            for tier in &tiers {
                assert_eq!(B2bTenantTier::try_parse(tier.label()), Some(tier.clone()));
            }
        }

        #[test]
        fn try_parse_rejects_unknown_labels() {
            assert_eq!(B2bTenantTier::try_parse("premium"), None);
            assert_eq!(B2bTenantTier::try_parse(""), None);
            assert_eq!(B2bTenantTier::try_parse("ENTERPRISE"), None);
        }

        #[test]
        fn display_matches_label() {
            assert_eq!(B2bTenantTier::Growth.to_string(), "growth");
        }
    }
}

pub use b2b_tenant_tier::B2bTenantTier;

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant_id(value: &str) -> TenantId {
        TenantId::new(value).expect("valid tenant id")
    }

    fn region(value: &str) -> RegionCode {
        RegionCode::new(value).expect("valid region")
    }

    fn home_binding() -> RegionBinding {
        RegionBinding::new(
            region("region-home"),
            None,
            ResidencyClass::StrictHomeRegion,
            "audit_evt_tenant_bound_001",
            1_762_992_000,
        )
        .expect("strict home-region binding is valid")
    }

    #[test]
    fn tenant_id_is_a_validated_kernel_newtype() {
        assert_eq!(tenant_id("ten_alpha").as_str(), "ten_alpha");
        assert_eq!("ten_alpha".parse::<TenantId>(), Ok(tenant_id("ten_alpha")));
        assert_eq!(
            TenantId::new("tenant-alpha"),
            Err(TenantKernelError::InvalidTenantId)
        );
    }

    #[test]
    fn region_binding_enforces_residency_primary_and_failover_rules() {
        assert_eq!(
            home_binding().residency_class(),
            ResidencyClass::StrictHomeRegion
        );
        assert_eq!(
            home_binding().schema_version(),
            REGION_BINDING_SCHEMA_VERSION
        );

        assert_eq!(
            RegionBinding::new(
                region("region-recovery"),
                None,
                ResidencyClass::StrictHomeRegion,
                "audit_evt_bad_region",
                1,
            ),
            Err(TenantKernelError::PrimaryRegionDeniedForResidency)
        );
        assert_eq!(
            RegionBinding::new(
                region("region-home"),
                Some(region("region-expansion")),
                ResidencyClass::HomeWithRecoveryFailover,
                "audit_evt_bad_failover",
                1,
            ),
            Err(TenantKernelError::FailoverRegionDeniedForResidency)
        );
        assert!(
            RegionBinding::new(
                region("region-home"),
                Some(region("region-recovery")),
                ResidencyClass::HomeWithRecoveryFailover,
                "audit_evt_good_failover",
                1,
            )
            .is_ok()
        );
    }

    #[test]
    fn tenant_contract_exposes_immutable_region_binding_by_accessors_only() {
        let tenant = Tenant::new(
            tenant_id("ten_alpha"),
            "Alpha Tenant Ltd",
            home_binding(),
            vec!["oya-pack-alpha".to_string()],
            TenantPlaneGrants::all(),
        )
        .expect("tenant is valid");

        assert_eq!(tenant.id().as_str(), "ten_alpha");
        assert_eq!(tenant.legal_name(), "Alpha Tenant Ltd");
        assert_eq!(tenant.region_binding().primary().as_str(), "region-home");
        assert_eq!(tenant.residency_class().label(), "strict_home_region");
        assert_eq!(tenant.schema_version(), TENANT_SCHEMA_VERSION);
        assert!(tenant.plane_grants().contains(TenantPlane::Control));
    }

    #[test]
    fn tenant_creation_rejects_empty_or_duplicate_regulatory_packs() {
        let base = (tenant_id("ten_alpha"), "Alpha Tenant Ltd", home_binding());
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
                vec!["oya-pack-alpha".to_string(), "oya-pack-alpha".to_string()],
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
        // Conversely, "ten_alpha" parses as both (internal IDs are slug-shaped).
        assert!(TenantSlug::try_new("ten_alpha").is_ok());
        assert!(TenantId::new("ten_alpha").is_ok());
    }

    #[test]
    fn tenant_env_tiers_match_adr0163_key_prefixes_and_outbound_modes() {
        assert_eq!(TenantEnvironmentTier::Test.label(), "test");
        assert_eq!(
            TenantEnvironmentTier::Test.audit_chain_tag(),
            "env_tier=test"
        );
        assert_eq!(
            TenantEnvironmentTier::Test.api_key_prefix(TenantApiKeyKind::Server),
            "sk_test_"
        );
        assert_eq!(
            TenantEnvironmentTier::Test.api_key_prefix(TenantApiKeyKind::Public),
            "pk_test_"
        );
        assert_eq!(
            TenantEnvironmentTier::Test.outbound_mode(),
            OutboundSideEffectMode::Intercept
        );

        assert_eq!(TenantEnvironmentTier::Staging.label(), "staging");
        assert_eq!(
            TenantEnvironmentTier::Staging.audit_chain_tag(),
            "env_tier=staging"
        );
        assert_eq!(
            TenantEnvironmentTier::Staging.api_key_prefix(TenantApiKeyKind::Server),
            "sk_stage_"
        );
        assert_eq!(
            TenantEnvironmentTier::Staging.api_key_prefix(TenantApiKeyKind::Public),
            "pk_stage_"
        );
        assert_eq!(
            TenantEnvironmentTier::Staging.outbound_mode(),
            OutboundSideEffectMode::TestRecipients
        );

        assert_eq!(TenantEnvironmentTier::Prod.label(), "prod");
        assert_eq!(
            TenantEnvironmentTier::Prod.audit_chain_tag(),
            "env_tier=prod"
        );
        assert_eq!(
            TenantEnvironmentTier::Prod.api_key_prefix(TenantApiKeyKind::Server),
            "sk_live_"
        );
        assert_eq!(
            TenantEnvironmentTier::Prod.api_key_prefix(TenantApiKeyKind::Public),
            "pk_live_"
        );
        assert_eq!(
            TenantEnvironmentTier::Prod.outbound_mode(),
            OutboundSideEffectMode::Live
        );
    }

    #[test]
    fn tenant_env_tier_routes_exact_api_key_prefixes() {
        assert_eq!(
            TenantEnvironmentTier::from_api_key_prefix("sk_test_abc"),
            Ok((TenantEnvironmentTier::Test, TenantApiKeyKind::Server))
        );
        assert_eq!(
            TenantEnvironmentTier::from_api_key_prefix("pk_stage_abc"),
            Ok((TenantEnvironmentTier::Staging, TenantApiKeyKind::Public))
        );
        assert_eq!(
            TenantEnvironmentTier::from_api_key_prefix("sk_live_abc"),
            Ok((TenantEnvironmentTier::Prod, TenantApiKeyKind::Server))
        );
        assert_eq!(
            TenantEnvironmentTier::from_api_key_prefix("sk_prod_abc"),
            Err(TenantKernelError::UnknownApiKeyPrefix)
        );
    }

    #[test]
    fn outbound_side_effect_modes_gate_external_delivery() {
        assert!(OutboundSideEffectMode::Intercept.is_intercepted());
        assert!(!OutboundSideEffectMode::Intercept.permits_live_delivery());
        assert!(!OutboundSideEffectMode::Intercept.requires_test_recipient_allowlist());
        assert!(!OutboundSideEffectMode::TestRecipients.is_intercepted());
        assert!(!OutboundSideEffectMode::TestRecipients.permits_live_delivery());
        assert!(OutboundSideEffectMode::TestRecipients.requires_test_recipient_allowlist());
        assert!(!OutboundSideEffectMode::Live.is_intercepted());
        assert!(OutboundSideEffectMode::Live.permits_live_delivery());
        assert!(!OutboundSideEffectMode::Live.requires_test_recipient_allowlist());
    }

    #[test]
    fn prod_destructive_operations_require_ack_header_and_cedar_context() {
        assert!(!TenantEnvironmentTier::Test.requires_destructive_acknowledgment());
        assert!(!TenantEnvironmentTier::Staging.requires_destructive_acknowledgment());
        assert!(TenantEnvironmentTier::Prod.requires_destructive_acknowledgment());
        assert_eq!(PROD_DESTRUCTIVE_ACK_HEADER, "x-oya-prod-destructive-ack");
        assert_eq!(
            PROD_DESTRUCTIVE_ACK_CEDAR_CONDITION,
            "prod_destructive_acknowledged"
        );
    }

    #[test]
    fn runtime_credential_posture_is_sts_short_lived_only() {
        let sts = RuntimeCredentialKind::StsShortLived {
            issued_at_epoch_seconds: 100,
            expires_at_epoch_seconds: 200,
        };
        assert_eq!(sts.validate(), Ok(()));
        assert_eq!(
            RuntimeCredentialKind::StaticSecret.validate(),
            Err(TenantKernelError::LongLivedRuntimeCredentialForbidden)
        );
        assert_eq!(
            RuntimeCredentialKind::LongLivedServiceAccount.validate(),
            Err(TenantKernelError::LongLivedRuntimeCredentialForbidden)
        );
        assert_eq!(
            RuntimeCredentialKind::StsShortLived {
                issued_at_epoch_seconds: 200,
                expires_at_epoch_seconds: 200,
            }
            .validate(),
            Err(TenantKernelError::InvalidRuntimeCredentialTtl)
        );
        assert_eq!(
            RuntimeCredentialKind::StsShortLived {
                issued_at_epoch_seconds: 1,
                expires_at_epoch_seconds: MAX_RUNTIME_CREDENTIAL_TTL_SECONDS + 2,
            }
            .validate(),
            Err(TenantKernelError::InvalidRuntimeCredentialTtl)
        );
    }

    #[test]
    fn tenant_mutation_review_requires_cross_axis_label_and_evidence() {
        let review = TenantMutationReview::new(
            TenantMutationKind::TenantStruct,
            [CROSS_MICROSERVICE_TENANT_MUTATION_LABEL],
            "evidence/review/tenant-struct.json",
        )
        .expect("mandatory label and evidence should satisfy review gate");
        assert_eq!(review.kind(), TenantMutationKind::TenantStruct);
        assert_eq!(review.evidence_ref(), "evidence/review/tenant-struct.json");

        assert_eq!(
            TenantMutationReview::new(
                TenantMutationKind::CatalogRecord,
                ["platform-only"],
                "evidence/review/catalog.json",
            ),
            Err(TenantKernelError::TenantMutationReviewLabelRequired)
        );
        assert_eq!(
            TenantMutationReview::new(
                TenantMutationKind::DerivedType,
                [CROSS_MICROSERVICE_TENANT_MUTATION_LABEL],
                " ",
            ),
            Err(TenantKernelError::TenantMutationReviewEvidenceRequired)
        );
    }
}
