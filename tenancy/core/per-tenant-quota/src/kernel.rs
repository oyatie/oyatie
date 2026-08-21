//! Quota kernel: the closed vocabulary every other layer speaks — quota
//! resources, reset windows, allowances, the decision value object, the two
//! read ports, and the two error enums.
//!
//! Nothing here performs I/O, reads a clock, or draws randomness: a decision
//! is a pure function of (policy, class, pack, override), and a window is a
//! parameter the caller supplies.

use core::fmt;

/// The closed set of metered resources (IP-022 §D.1).
///
/// The wire form carried in [`QuotaKey::resource`] is the snake_case name;
/// [`QuotaResource::parse`] also accepts the kebab-case spelling used by the
/// REST surface, and rejects everything else rather than silently defaulting.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum QuotaResource {
    /// Inbound requests admitted per rolling minute.
    RequestRatePerMinute,
    /// Bytes at rest attributable to the tenant.
    StorageBytes,
    /// Public API calls per day.
    ApiCallsPerDay,
    /// Capability invocations per day.
    CapabilityInvocationsPerDay,
    /// Concurrently entitled seats.
    SeatCount,
    /// Outbound webhook deliveries fanned out per minute.
    WebhookFanoutPerMinute,
}

impl QuotaResource {
    /// Every resource, in declaration order — the iteration order callers get
    /// when they resolve a tenant's whole quota sheet.
    pub const ALL: [QuotaResource; 6] = [
        QuotaResource::RequestRatePerMinute,
        QuotaResource::StorageBytes,
        QuotaResource::ApiCallsPerDay,
        QuotaResource::CapabilityInvocationsPerDay,
        QuotaResource::SeatCount,
        QuotaResource::WebhookFanoutPerMinute,
    ];

    /// The canonical wire name of this resource.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            QuotaResource::RequestRatePerMinute => "request_rate_per_minute",
            QuotaResource::StorageBytes => "storage_bytes",
            QuotaResource::ApiCallsPerDay => "api_calls_per_day",
            QuotaResource::CapabilityInvocationsPerDay => "capability_invocations_per_day",
            QuotaResource::SeatCount => "seat_count",
            QuotaResource::WebhookFanoutPerMinute => "webhook_fanout_per_minute",
        }
    }

    /// Parse a wire name. Unknown names are an error, never a default: an
    /// unrecognised resource that resolved to some fallback ceiling would
    /// enforce a limit nobody declared.
    ///
    /// # Errors
    /// [`QuotaUsecaseError::UnknownResource`] when the name is not in the
    /// closed set above.
    pub fn parse(raw: &str) -> Result<Self, QuotaUsecaseError> {
        let normalized = raw.trim().to_ascii_lowercase().replace('-', "_");
        match normalized.as_str() {
            "request_rate_per_minute" => Ok(QuotaResource::RequestRatePerMinute),
            "storage_bytes" => Ok(QuotaResource::StorageBytes),
            "api_calls_per_day" => Ok(QuotaResource::ApiCallsPerDay),
            "capability_invocations_per_day" => Ok(QuotaResource::CapabilityInvocationsPerDay),
            "seat_count" => Ok(QuotaResource::SeatCount),
            "webhook_fanout_per_minute" => Ok(QuotaResource::WebhookFanoutPerMinute),
            _ => Err(QuotaUsecaseError::UnknownResource {
                resource: raw.to_owned(),
            }),
        }
    }

    /// The natural reset window for this resource. Storage and seats are
    /// stock measures — they are not consumed per window, so they never reset.
    #[must_use]
    pub const fn natural_window(self) -> ResetWindow {
        match self {
            QuotaResource::RequestRatePerMinute | QuotaResource::WebhookFanoutPerMinute => {
                ResetWindow::Seconds(60)
            }
            QuotaResource::ApiCallsPerDay | QuotaResource::CapabilityInvocationsPerDay => {
                ResetWindow::Seconds(86_400)
            }
            QuotaResource::StorageBytes | QuotaResource::SeatCount => ResetWindow::Never,
        }
    }
}

impl fmt::Display for QuotaResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How often consumption counters roll over.
///
/// The window is a *parameter*, not a clock read: callers hand the ledger the
/// instant they observed, so a reset is reproducible in a test.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ResetWindow {
    /// Stock measure — consumption is never forgiven by the passage of time.
    Never,
    /// Flow measure — counters roll over every N seconds.
    Seconds(u64),
}

impl fmt::Display for ResetWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResetWindow::Never => f.write_str("never"),
            ResetWindow::Seconds(seconds) => write!(f, "{seconds}s"),
        }
    }
}

/// One declared allowance: how much, how close before we warn, how often it
/// rolls over. This is what a *policy layer* (class default or pack override)
/// contributes to the precedence chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuotaAllowance {
    /// The declared ceiling for this layer, before any hard cap clamps it.
    pub limit: u64, // data_class: TENANT_SCOPED
    /// Percent of the *effective* ceiling at which consumption stops being
    /// quiet and starts warning. `0..=100`; 100 means "never warn early".
    ///
    /// [`QuotaAllowance::new`] rejects a larger value, but the field is
    /// public, so a struct literal can still carry one. Resolution therefore
    /// does not trust it: [`crate::domain::soft_threshold_of`] clamps to 100,
    /// because a threshold above the ceiling could never be crossed and the
    /// tenant would run to 100% utilisation with no warning event at all.
    pub soft_threshold_percent: u8, // data_class: INTERNAL_ONLY
    /// How often consumption counters roll over.
    pub window: ResetWindow, // data_class: INTERNAL_ONLY
}

impl QuotaAllowance {
    /// Build an allowance, rejecting a soft threshold outside `0..=100`.
    ///
    /// # Errors
    /// [`QuotaUsecaseError::InvalidPolicy`] when `soft_threshold_percent > 100`.
    pub fn new(
        limit: u64,
        soft_threshold_percent: u8,
        window: ResetWindow,
    ) -> Result<Self, QuotaUsecaseError> {
        if soft_threshold_percent > 100 {
            return Err(QuotaUsecaseError::InvalidPolicy {
                detail: format!("soft threshold {soft_threshold_percent}% exceeds 100%"),
            });
        }
        Ok(Self {
            limit,
            soft_threshold_percent,
            window,
        })
    }

    /// An allowance warning at the platform default of 80% of the effective
    /// ceiling, using the resource's natural window.
    #[must_use]
    pub const fn standard(limit: u64, resource: QuotaResource) -> Self {
        Self {
            limit,
            soft_threshold_percent: DEFAULT_SOFT_THRESHOLD_PERCENT,
            window: resource.natural_window(),
        }
    }

    /// The same allowance with a different declared ceiling — how a tenant
    /// override replaces the number without discarding the threshold and
    /// window policy the layer below it established.
    #[must_use]
    pub const fn with_limit(self, limit: u64) -> Self {
        Self { limit, ..self }
    }
}

/// Platform default soft-threshold percentage (warn at 80% of effective).
pub const DEFAULT_SOFT_THRESHOLD_PERCENT: u8 = 80;

/// The address of one quota: which tenant, which metered resource.
///
/// `resource` is the *wire* spelling, so a REST caller can hand one straight
/// through; it is normalised through [`QuotaResource::parse`] at every point
/// that reads it, and an unrecognised name is an error rather than a default.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct QuotaKey {
    /// The tenant this quota belongs to.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// The metered resource, in wire spelling.
    pub resource: String, // data_class: INTERNAL_ONLY
}

impl QuotaKey {
    /// Build a key from a typed resource, so callers do not hand-spell the
    /// wire name.
    #[must_use]
    pub fn new(tenant_id: impl Into<String>, resource: QuotaResource) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            resource: resource.as_str().to_owned(),
        }
    }
}

/// A resolved quota, and an honest account of where the number came from.
///
/// `limit` and `effective` are deliberately different things:
/// - `limit` is what the *winning precedence layer declared* — the class
///   default, the pack override, or the tenant override, whichever won.
/// - `effective` is what is actually *enforced*: `limit` clamped down to the
///   hard cap. `effective <= limit` always holds.
///
/// When the clamp bites, `source` is [`QuotaSource::HardCap`] — because the
/// hard cap, not the override, produced the enforced number. A decision that
/// reports the losing layer is a defect: the provenance is the whole point of
/// the field, and it is what an audit event has to be able to explain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotaDecision {
    /// What the winning precedence layer declared, before clamping.
    pub limit: u64, // data_class: TENANT_SCOPED
    /// What is actually enforced: `limit` clamped to the hard cap.
    pub effective: u64, // data_class: TENANT_SCOPED
    /// Which precedence layer produced `effective`.
    pub source: QuotaSource, // data_class: INTERNAL_ONLY
    /// The resource this decision is about.
    pub resource: QuotaResource, // data_class: INTERNAL_ONLY
    /// Consumption at or below this value is quiet; strictly above it warns
    /// and still admits. Derived from `effective`, so a clamp moves it too.
    pub soft_threshold: u64, // data_class: TENANT_SCOPED
    /// How often consumption against `effective` rolls over.
    pub window: ResetWindow, // data_class: INTERNAL_ONLY
    /// The tenant class (plan tier) the class default was read from.
    pub class: String, // data_class: TENANT_SCOPED
    /// The regulatory/product pack applied, in canonical spelling, when the
    /// tenant carries one.
    ///
    /// `Some` means the catalog *declares* this pack: a pack the catalog does
    /// not know fails the resolution with
    /// [`QuotaUsecaseError::UnknownPack`] rather than appearing here, so this
    /// field never claims pack provenance that was silently missed.
    pub pack: Option<String>, // data_class: TENANT_SCOPED
}

impl QuotaDecision {
    /// Headroom left when `used` has been consumed. Saturating: consumption
    /// recorded above the ceiling (a limit lowered under a running tenant)
    /// reports zero headroom, never a wrapped-around unlimited one.
    #[must_use]
    pub const fn remaining(&self, used: u64) -> u64 {
        self.effective.saturating_sub(used)
    }

    /// Whether the hard cap clamped the winning layer's declared limit.
    #[must_use]
    pub const fn was_clamped(&self) -> bool {
        self.effective < self.limit
    }
}

/// Which precedence layer produced the enforced number.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaSource {
    /// The plan-tier default; no pack or override displaced it.
    ClassDefault,
    /// A pack override tightened the class default.
    PackOverride,
    /// A tenant-scoped override, and no cap clamped it.
    TenantOverride,
    /// A hard cap clamped whichever layer won.
    HardCap,
}

impl QuotaSource {
    /// The audit-evidence name of this provenance layer.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            QuotaSource::ClassDefault => "class_default",
            QuotaSource::PackOverride => "pack_override",
            QuotaSource::TenantOverride => "tenant_override",
            QuotaSource::HardCap => "hard_cap",
        }
    }
}

impl fmt::Display for QuotaSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub trait TenantClassReader {
    fn class(&self, tenant_id: &str) -> Result<String, QuotaUsecaseError>;

    /// The regulatory/product pack bound to the tenant, if any.
    ///
    /// Defaulted so existing implementors keep compiling: a reader that does
    /// not model packs reports "no pack", and the precedence chain simply
    /// never reaches its pack layer.
    ///
    /// # Errors
    /// [`QuotaUsecaseError::UnknownTenant`] when the tenant is not known, or
    /// [`QuotaUsecaseError::PersistenceUnavailable`] when the backing store
    /// cannot answer.
    fn pack(&self, tenant_id: &str) -> Result<Option<String>, QuotaUsecaseError> {
        let _ = tenant_id;
        Ok(None)
    }
}

pub trait QuotaOverrideRepository {
    fn lookup(&self, key: &QuotaKey) -> Result<Option<u64>, QuotaUsecaseError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaUsecaseError {
    UnknownTenant,
    PersistenceUnavailable,
    /// The key named a resource outside the closed set.
    UnknownResource {
        resource: String,
    },
    /// The catalog declares no default for this (class, resource) pair, so
    /// there is no honest number to return.
    NoPolicyForClass {
        class: String,
        resource: QuotaResource,
    },
    /// The tenant names a pack the catalog does not declare.
    ///
    /// Fails closed on purpose. A pack spelled `US-HC` where the catalog says
    /// `us-hc` is normalised and matches; a pack nobody declared is a typo or
    /// a stale tenant record, and resolving it anyway would silently drop the
    /// regulated ceiling it was supposed to impose.
    UnknownPack {
        pack: String,
    },
    /// A policy value is self-inconsistent (e.g. a threshold above 100%).
    InvalidPolicy {
        detail: String,
    },
}

impl fmt::Display for QuotaUsecaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuotaUsecaseError::UnknownTenant => f.write_str("unknown tenant"),
            QuotaUsecaseError::PersistenceUnavailable => f.write_str("persistence unavailable"),
            QuotaUsecaseError::UnknownResource { resource } => {
                write!(f, "unknown quota resource {resource:?}")
            }
            QuotaUsecaseError::NoPolicyForClass { class, resource } => {
                write!(f, "no quota policy for class {class:?} resource {resource}")
            }
            QuotaUsecaseError::UnknownPack { pack } => {
                write!(f, "unknown quota pack {pack:?}")
            }
            QuotaUsecaseError::InvalidPolicy { detail } => {
                write!(f, "invalid quota policy: {detail}")
            }
        }
    }
}

impl std::error::Error for QuotaUsecaseError {}

/// Failures of *consumption accounting*, as distinct from failures of
/// *policy resolution* ([`QuotaUsecaseError`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaUsageError {
    /// A release named more than is currently reserved. Reported rather than
    /// wrapped: an unsigned underflow here would hand the tenant a counter
    /// near `u64::MAX` of headroom, i.e. an unlimited quota.
    ReleaseWithoutReservation { requested: u64, reserved: u64 },
    /// A commit named more than is currently reserved.
    CommitWithoutReservation { requested: u64, reserved: u64 },
    /// The arithmetic would exceed `u64`.
    AmountOverflow,
    /// The observed instant is before the window this ledger already opened;
    /// time does not run backwards in a quota ledger.
    WindowRegression { observed: u64, window_start: u64 },
    /// A window length of zero seconds would reset on every observation.
    InvalidWindow,
}

impl fmt::Display for QuotaUsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuotaUsageError::ReleaseWithoutReservation {
                requested,
                reserved,
            } => write!(f, "release of {requested} exceeds {reserved} reserved"),
            QuotaUsageError::CommitWithoutReservation {
                requested,
                reserved,
            } => write!(f, "commit of {requested} exceeds {reserved} reserved"),
            QuotaUsageError::AmountOverflow => f.write_str("quota accounting overflowed u64"),
            QuotaUsageError::WindowRegression {
                observed,
                window_start,
            } => write!(
                f,
                "observed instant {observed} precedes window start {window_start}"
            ),
            QuotaUsageError::InvalidWindow => f.write_str("reset window of zero seconds"),
        }
    }
}

impl std::error::Error for QuotaUsageError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_both_wire_spellings_and_rejects_unknown() {
        assert_eq!(
            QuotaResource::parse("storage_bytes").unwrap(),
            QuotaResource::StorageBytes
        );
        assert_eq!(
            QuotaResource::parse(" Api-Calls-Per-Day ").unwrap(),
            QuotaResource::ApiCallsPerDay
        );
        assert_eq!(
            QuotaResource::parse("cpu_seconds").unwrap_err(),
            QuotaUsecaseError::UnknownResource {
                resource: "cpu_seconds".to_owned()
            }
        );
    }

    #[test]
    fn every_resource_round_trips_through_its_wire_name() {
        for resource in QuotaResource::ALL {
            assert_eq!(QuotaResource::parse(resource.as_str()).unwrap(), resource);
        }
    }

    #[test]
    fn stock_resources_never_reset_and_flow_resources_do() {
        assert_eq!(
            QuotaResource::StorageBytes.natural_window(),
            ResetWindow::Never
        );
        assert_eq!(
            QuotaResource::SeatCount.natural_window(),
            ResetWindow::Never
        );
        assert_eq!(
            QuotaResource::RequestRatePerMinute.natural_window(),
            ResetWindow::Seconds(60)
        );
        assert_eq!(
            QuotaResource::ApiCallsPerDay.natural_window(),
            ResetWindow::Seconds(86_400)
        );
    }

    #[test]
    fn allowance_rejects_a_threshold_above_one_hundred_percent() {
        assert!(QuotaAllowance::new(10, 100, ResetWindow::Never).is_ok());
        assert!(matches!(
            QuotaAllowance::new(10, 101, ResetWindow::Never),
            Err(QuotaUsecaseError::InvalidPolicy { .. })
        ));
    }

    #[test]
    fn with_limit_keeps_threshold_and_window_policy() {
        let base = QuotaAllowance::standard(100, QuotaResource::ApiCallsPerDay);
        let overridden = base.with_limit(5_000);
        assert_eq!(overridden.limit, 5_000);
        assert_eq!(
            overridden.soft_threshold_percent,
            base.soft_threshold_percent
        );
        assert_eq!(overridden.window, base.window);
    }

    #[test]
    fn errors_display_and_are_std_errors() {
        let err: &dyn std::error::Error = &QuotaUsecaseError::UnknownTenant;
        assert_eq!(err.to_string(), "unknown tenant");
        let usage: &dyn std::error::Error = &QuotaUsageError::ReleaseWithoutReservation {
            requested: 5,
            reserved: 2,
        };
        assert_eq!(usage.to_string(), "release of 5 exceeds 2 reserved");
    }
}
