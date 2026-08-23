//! # billing-metering-pipeline-kernel
//!
//! The D-14 metering PIPELINE contract (ADR-0536; G009 lane): usage is a
//! pipeline, not a query — events flow dedup → rate → aggregate → invoice
//! and operational databases are never aggregated at query time.
//!
//! This kernel owns the pipeline doctrine; the ingest EVENT vocabulary
//! stays in `metering-domain` (cloud-billing) and adapters map events
//! into [`UsageRecord`]s at the pipeline edge:
//!
//! - **The dedup key** is `(tenant, resource, dimension, usage_hour)` —
//!   [`DedupKey`]. The sink holds exactly one usage record per key; a
//!   replay is an idempotent duplicate, a conflicting replay is a
//!   surfaced error, never a silent overwrite (precedent: Azure metered
//!   billing idempotent hourly usage ingestion).
//! - **Three-clock doctrine**: accrual per-second ([`UsageRecord`]
//!   timestamps), rating hourly ([`UsageHour`] buckets), invoicing
//!   monthly (downstream of this crate).
//! - **Lateness is explicit**: events older than the
//!   [`LatenessPolicy`] window (6h at launch) are REJECTED with a typed
//!   reason — never silently dropped, never silently backfilled
//!   (precedent: AWS CUR pipeline lateness doctrine).
//! - **FOCUS 1.2 from day one**: the internal cost/usage projection is
//!   [`FocusRecord`] with `x_tenant_id` and `x_cell_id` first-class
//!   extension columns (FinOps FOCUS 1.2).
//! - **No floating-point money or quantity**: integer microunits only
//!   (rejected anti-pattern, ADR-0536 D-14).
//!
//! The [`MeteringSink`] port is the never-lose idempotent sink; the
//! durable implementation arrives via the G03 `data` port, and the
//! conformance harness in [`conformance`] holds every implementation to
//! this one specification.
//!
//! # Naming justification
//! `billing-metering-pipeline-kernel` is the ADR-0562 de-branded
//! billing capability home for the metering pipeline kernel.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

pub mod conformance;
pub mod reference;

/// Seconds per usage-hour bucket.
pub const SECONDS_PER_HOUR: u64 = 3600;
/// Launch lateness window: six hours (ADR-0536 D-14 / G009 acceptance).
pub const DEFAULT_LATENESS_WINDOW_SECONDS: u64 = 6 * SECONDS_PER_HOUR;
/// Maximum accepted id/slug length.
pub const MAX_ID_LEN: usize = 255;

// =====================================================================
// Errors
// =====================================================================

/// Pipeline contract errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeteringPipelineError {
    /// Tenant id is not `ten_`-prefixed (vocabulary shared with
    /// `metering-domain`).
    InvalidTenantId { value: String },
    /// Cell id is not a canonical slug.
    InvalidCellId { value: String },
    /// Resource id is empty or oversized.
    InvalidResourceId { value: String },
    /// Dimension is not a canonical slug.
    InvalidDimension { value: String },
    /// Unit is not a canonical slug.
    InvalidUnit { value: String },
    /// The event was explicitly rejected by pipeline policy.
    Rejected(UsageRejection),
    /// A replay under an existing dedup key carried a DIFFERENT quantity:
    /// either a producer bug or tampering — surfaced, never overwritten.
    QuantityConflict {
        key: DedupKey,
        recorded_microunits: u64,
        replayed_microunits: u64,
    },
    /// Transient sink failure; the caller retries (idempotency makes the
    /// retry safe by contract).
    SinkUnavailable { detail: String },
}

impl fmt::Display for MeteringPipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTenantId { value } => write!(f, "invalid tenant id: {value:?}"),
            Self::InvalidCellId { value } => write!(f, "invalid cell id: {value:?}"),
            Self::InvalidResourceId { value } => write!(f, "invalid resource id: {value:?}"),
            Self::InvalidDimension { value } => write!(f, "invalid dimension: {value:?}"),
            Self::InvalidUnit { value } => write!(f, "invalid unit: {value:?}"),
            Self::Rejected(rejection) => write!(f, "rejected: {rejection}"),
            Self::QuantityConflict {
                key,
                recorded_microunits,
                replayed_microunits,
            } => write!(
                f,
                "quantity conflict for {key}: recorded {recorded_microunits}, \
                 replay carried {replayed_microunits}"
            ),
            Self::SinkUnavailable { detail } => write!(f, "sink unavailable: {detail}"),
        }
    }
}

impl std::error::Error for MeteringPipelineError {}

/// Why the pipeline explicitly refused an event. Rejections are part of
/// the contract: producers get a typed reason, operators get a count —
/// silence is the rejected anti-pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageRejection {
    /// The event arrived more than the lateness window after the close of
    /// the usage hour it claims.
    LateArrival {
        usage_hour: UsageHour,
        arrived_at_epoch_seconds: u64,
        window_seconds: u64,
    },
    /// The event claims a usage hour that has not started yet.
    FutureUsage {
        usage_hour: UsageHour,
        arrived_at_epoch_seconds: u64,
    },
}

impl fmt::Display for UsageRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LateArrival {
                usage_hour,
                arrived_at_epoch_seconds,
                window_seconds,
            } => write!(
                f,
                "late arrival: usage hour {usage_hour} closed more than \
                 {window_seconds}s before arrival at {arrived_at_epoch_seconds}"
            ),
            Self::FutureUsage {
                usage_hour,
                arrived_at_epoch_seconds,
            } => write!(
                f,
                "future usage: hour {usage_hour} has not started at \
                 arrival time {arrived_at_epoch_seconds}"
            ),
        }
    }
}

// =====================================================================
// Identity vocabulary
// =====================================================================

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_LEN
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
        && !value.starts_with(['-', '.'])
        && !value.ends_with(['-', '.'])
}

/// Tenant identity, `ten_`-prefixed (the `metering-domain` /
/// platform-contracts tenant vocabulary).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TenantId(String);

impl TenantId {
    /// Parses and validates a tenant id.
    ///
    /// # Errors
    /// Returns [`MeteringPipelineError::InvalidTenantId`] when the value
    /// is not `ten_`-prefixed with a non-empty suffix.
    pub fn parse(value: &str) -> Result<Self, MeteringPipelineError> {
        let suffix = value.strip_prefix("ten_").unwrap_or("");
        if suffix.is_empty() || value.len() > MAX_ID_LEN {
            return Err(MeteringPipelineError::InvalidTenantId {
                value: value.to_owned(),
            });
        }
        Ok(Self(value.to_owned()))
    }

    /// The canonical string form.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

macro_rules! slug_newtype {
    ($(#[$doc:meta])* $name:ident, $error:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Parses and validates the canonical slug shape.
            ///
            /// # Errors
            /// Returns the corresponding `MeteringPipelineError` variant
            /// when the value is not a canonical slug.
            pub fn parse(value: &str) -> Result<Self, MeteringPipelineError> {
                if valid_slug(value) {
                    Ok(Self(value.to_owned()))
                } else {
                    Err(MeteringPipelineError::$error {
                        value: value.to_owned(),
                    })
                }
            }

            /// The canonical string form.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

slug_newtype!(
    /// Cell identity (cell-based architecture; first-class in FOCUS
    /// projection per the founder amendment).
    CellId,
    InvalidCellId
);
slug_newtype!(
    /// The metered resource (service/capability instance slug).
    ResourceId,
    InvalidResourceId
);
slug_newtype!(
    /// The usage dimension being measured (e.g. `requests`,
    /// `storage-gb-seconds`).
    Dimension,
    InvalidDimension
);
slug_newtype!(
    /// The unit of the consumed quantity (e.g. `request`, `gb-second`).
    ConsumedUnit,
    InvalidUnit
);

// =====================================================================
// Time buckets
// =====================================================================

/// One rating bucket: an hour since the epoch (three-clock doctrine —
/// accrual per-second, rating hourly, invoicing monthly).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UsageHour(u64);

impl UsageHour {
    /// The bucket containing `epoch_seconds`.
    #[must_use]
    pub fn from_epoch_seconds(epoch_seconds: u64) -> Self {
        Self(epoch_seconds / SECONDS_PER_HOUR)
    }

    /// Hours since the epoch.
    #[must_use]
    pub fn hours_since_epoch(self) -> u64 {
        self.0
    }

    /// First second inside the bucket.
    #[must_use]
    pub fn start_epoch_seconds(self) -> u64 {
        self.0.saturating_mul(SECONDS_PER_HOUR)
    }

    /// First second AFTER the bucket (charge period end, exclusive).
    #[must_use]
    pub fn end_epoch_seconds(self) -> u64 {
        self.start_epoch_seconds().saturating_add(SECONDS_PER_HOUR)
    }
}

impl fmt::Display for UsageHour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "usage-hour-{}", self.0)
    }
}

/// Lateness acceptance policy: an event for usage hour H is accepted
/// while `now < H.end + window`; later arrivals are explicitly rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LatenessPolicy {
    /// Seconds after the close of a usage hour during which events for
    /// that hour are still accepted.
    pub window_seconds: u64,
}

impl Default for LatenessPolicy {
    fn default() -> Self {
        Self {
            window_seconds: DEFAULT_LATENESS_WINDOW_SECONDS,
        }
    }
}

impl LatenessPolicy {
    /// Evaluates an arrival against the policy.
    ///
    /// # Errors
    /// Returns the typed [`UsageRejection`] for late or future events.
    pub fn admit(
        &self,
        usage_hour: UsageHour,
        arrived_at_epoch_seconds: u64,
    ) -> Result<(), UsageRejection> {
        if arrived_at_epoch_seconds < usage_hour.start_epoch_seconds() {
            return Err(UsageRejection::FutureUsage {
                usage_hour,
                arrived_at_epoch_seconds,
            });
        }
        let cutoff = usage_hour
            .end_epoch_seconds()
            .saturating_add(self.window_seconds);
        if arrived_at_epoch_seconds >= cutoff {
            return Err(UsageRejection::LateArrival {
                usage_hour,
                arrived_at_epoch_seconds,
                window_seconds: self.window_seconds,
            });
        }
        Ok(())
    }
}

// =====================================================================
// Records and keys
// =====================================================================

/// THE dedup key: one usage record per
/// `(tenant, resource, dimension, usage_hour)`.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DedupKey {
    pub tenant: TenantId,      // data_class: INTERNAL_ONLY
    pub resource: ResourceId,  // data_class: INTERNAL_ONLY
    pub dimension: Dimension,  // data_class: INTERNAL_ONLY
    pub usage_hour: UsageHour, // data_class: INTERNAL_ONLY
}

impl fmt::Display for DedupKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {}, {})",
            self.tenant, self.resource, self.dimension, self.usage_hour
        )
    }
}

/// One hourly usage record — the unit the sink holds per [`DedupKey`].
/// Quantities are integer microunits; floating-point money/quantity is a
/// rejected anti-pattern (ADR-0536 D-14).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageRecord {
    pub tenant: TenantId,                  // data_class: INTERNAL_ONLY
    pub cell: CellId,                      // data_class: INTERNAL_ONLY
    pub resource: ResourceId,              // data_class: INTERNAL_ONLY
    pub dimension: Dimension,              // data_class: INTERNAL_ONLY
    pub usage_hour: UsageHour,             // data_class: INTERNAL_ONLY
    pub consumed_quantity_microunits: u64, // data_class: INTERNAL_ONLY
    pub consumed_unit: ConsumedUnit,       // data_class: INTERNAL_ONLY
}

impl UsageRecord {
    /// The record's dedup key.
    #[must_use]
    pub fn dedup_key(&self) -> DedupKey {
        DedupKey {
            tenant: self.tenant.clone(),
            resource: self.resource.clone(),
            dimension: self.dimension.clone(),
            usage_hour: self.usage_hour,
        }
    }

    /// Projects the record into its FOCUS 1.2 row.
    #[must_use]
    pub fn to_focus_record(&self) -> FocusRecord {
        FocusRecord {
            charge_period_start_epoch_seconds: self.usage_hour.start_epoch_seconds(),
            charge_period_end_epoch_seconds: self.usage_hour.end_epoch_seconds(),
            resource_id: self.resource.clone(),
            sku_id: self.dimension.clone(),
            consumed_quantity_microunits: self.consumed_quantity_microunits,
            consumed_unit: self.consumed_unit.clone(),
            x_tenant_id: self.tenant.clone(),
            x_cell_id: self.cell.clone(),
        }
    }
}

/// The FOCUS 1.2-aligned internal cost/usage projection (FinOps FOCUS
/// 1.2 column vocabulary; `x_`-prefixed columns are the spec's extension
/// convention — tenant and cell are first-class here by founder
/// directive). Monetary columns (BilledCost, ContractedCost, …) attach
/// at the rating stage downstream; this is the metered-usage subset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusRecord {
    /// FOCUS `ChargePeriodStart`.
    pub charge_period_start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    /// FOCUS `ChargePeriodEnd` (exclusive).
    pub charge_period_end_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    /// FOCUS `ResourceId`.
    pub resource_id: ResourceId, // data_class: INTERNAL_ONLY
    /// FOCUS `SkuId` (the rated dimension).
    pub sku_id: Dimension, // data_class: INTERNAL_ONLY
    /// FOCUS `ConsumedQuantity`, integer microunits.
    pub consumed_quantity_microunits: u64, // data_class: INTERNAL_ONLY
    /// FOCUS `ConsumedUnit`.
    pub consumed_unit: ConsumedUnit, // data_class: INTERNAL_ONLY
    /// FOCUS extension column: owning tenant (first-class).
    pub x_tenant_id: TenantId, // data_class: INTERNAL_ONLY
    /// FOCUS extension column: serving cell (first-class).
    pub x_cell_id: CellId, // data_class: INTERNAL_ONLY
}

/// Outcome of an idempotent ingest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngestOutcome {
    /// First write for the key.
    Recorded,
    /// Identical replay; nothing changed (at-least-once made safe).
    Duplicate,
}

/// One row in a batch ingest request: a usage record plus the trusted
/// pipeline arrival time used for lateness enforcement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchUsageRecord {
    pub record: UsageRecord,           // data_class: INTERNAL_ONLY
    pub arrived_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

/// Per-row result from batch ingest. Batch ingest is not silent best-effort:
/// each input row receives either its idempotent outcome or its typed
/// rejection/conflict reason, while other rows continue through the same
/// sink contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchIngestResult {
    pub key: DedupKey, // data_class: INTERNAL_ONLY
    pub outcome: Result<IngestOutcome, MeteringPipelineError>, // data_class: INTERNAL_ONLY
}

// =====================================================================
// The sink port
// =====================================================================

/// The never-lose idempotent metering sink (ADR-0536 D-14). One record
/// per [`DedupKey`]; replays are duplicates; conflicting replays are
/// errors; lateness is enforced HERE so no implementation can forget it.
/// The durable implementation arrives via the G03 `data` port; the
/// reference implementation lives in [`crate::reference`].
pub trait MeteringSink {
    /// The sink's lateness policy.
    fn lateness_policy(&self) -> LatenessPolicy;

    /// Idempotently ingests one usage record observed at
    /// `arrived_at_epoch_seconds`.
    ///
    /// # Errors
    /// Returns [`MeteringPipelineError::Rejected`] for late/future
    /// events, [`MeteringPipelineError::QuantityConflict`] for a replay
    /// that disagrees with the stored quantity, and
    /// [`MeteringPipelineError::SinkUnavailable`] on transient failure.
    fn ingest(
        &self,
        record: UsageRecord,
        arrived_at_epoch_seconds: u64,
    ) -> Result<IngestOutcome, MeteringPipelineError>;

    /// Reads back the stored record for a key, if any (the hourly rating
    /// stage's input — reads come from the sink, never from operational
    /// databases at query time).
    ///
    /// # Errors
    /// Returns [`MeteringPipelineError::SinkUnavailable`] on transient
    /// failure.
    fn lookup(&self, key: &DedupKey) -> Result<Option<UsageRecord>, MeteringPipelineError>;

    /// Idempotently ingests a batch of hourly usage rows.
    ///
    /// The default contract intentionally delegates every row to
    /// [`MeteringSink::ingest`] so duplicate, conflict, future, late, and
    /// sink-unavailable behavior cannot diverge from the single-row path.
    /// A rejected row is reported in its own [`BatchIngestResult`] and is
    /// not allowed to silently drop, overwrite, or stop later independent
    /// rows in the batch.
    #[must_use]
    fn ingest_batch(&self, records: &[BatchUsageRecord]) -> Vec<BatchIngestResult> {
        records
            .iter()
            .map(|entry| {
                let key = entry.record.dedup_key();
                let outcome = self.ingest(entry.record.clone(), entry.arrived_at_epoch_seconds);
                BatchIngestResult { key, outcome }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_ids_require_the_ten_prefix() {
        assert!(TenantId::parse("ten_0001").is_ok());
        assert!(TenantId::parse("ten_").is_err());
        assert!(TenantId::parse("0001").is_err());
    }

    #[test]
    fn usage_hours_bucket_and_bound_correctly() {
        let hour = UsageHour::from_epoch_seconds(7250);
        assert_eq!(hour.hours_since_epoch(), 2);
        assert_eq!(hour.start_epoch_seconds(), 7200);
        assert_eq!(hour.end_epoch_seconds(), 10800);
    }

    #[test]
    fn lateness_policy_admits_within_window_and_rejects_outside() {
        let policy = LatenessPolicy::default();
        let hour = UsageHour::from_epoch_seconds(0);
        // Inside the hour.
        assert!(policy.admit(hour, 10).is_ok());
        // After the hour but inside the 6h window.
        assert!(policy.admit(hour, SECONDS_PER_HOUR + 5).is_ok());
        // Exactly at the cutoff: rejected.
        let cutoff = hour.end_epoch_seconds() + DEFAULT_LATENESS_WINDOW_SECONDS;
        assert!(matches!(
            policy.admit(hour, cutoff),
            Err(UsageRejection::LateArrival { .. })
        ));
        // Future usage: rejected.
        let future = UsageHour::from_epoch_seconds(SECONDS_PER_HOUR * 10);
        assert!(matches!(
            policy.admit(future, 0),
            Err(UsageRejection::FutureUsage { .. })
        ));
    }

    #[test]
    fn focus_projection_carries_tenant_and_cell_first_class() {
        let record = UsageRecord {
            tenant: TenantId::parse("ten_a").unwrap(),
            cell: CellId::parse("cell-kr-1").unwrap(),
            resource: ResourceId::parse("meter").unwrap(),
            dimension: Dimension::parse("requests").unwrap(),
            usage_hour: UsageHour::from_epoch_seconds(7200),
            consumed_quantity_microunits: 5_000_000,
            consumed_unit: ConsumedUnit::parse("request").unwrap(),
        };
        let focus = record.to_focus_record();
        assert_eq!(focus.x_tenant_id.as_str(), "ten_a");
        assert_eq!(focus.x_cell_id.as_str(), "cell-kr-1");
        assert_eq!(focus.charge_period_start_epoch_seconds, 7200);
        assert_eq!(focus.charge_period_end_epoch_seconds, 10800);
        assert_eq!(focus.consumed_quantity_microunits, 5_000_000);
    }
}
