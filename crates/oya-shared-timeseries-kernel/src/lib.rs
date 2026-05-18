//! Time-series kernel per ADR-0194.
//!
//! Owns the engine-agnostic `TimeseriesStore` trait, hypertable DDL
//! declaration with the ADR-0194 chunk-sizing rubric, continuous-aggregate
//! declaration, retention-policy declaration (executed by per-µservice
//! workers per ADR-0194 §"Retention — per-µservice worker, not TSL"),
//! and the **TSL-fence**: the kernel rejects any SQL fragment that names
//! a TimescaleDB TSL-only function so the Apache-2.0 community-edition
//! contract is enforced at the type layer.
//!
//! Per ADR-0083, the kernel is I/O-free. Adapter crates:
//!   - `oya-shared-timeseries-timescaledb-adapter` — Postgres 18.4 +
//!      TimescaleDB 2.26 community extension; tokio-postgres at the
//!      adapter layer.
//!   - `oya-shared-timeseries-memory-adapter` — in-process reference impl
//!      shipped here as a module for tests.
//!
//! ## In-house roadmap parity (ADR-0194 §"In-house roadmap")
//!
//! TimescaleDB community-edition is KEEP per the policy; no Phase 2
//! in-house replacement is planned absent the trigger conditions in
//! ADR-0194. The trait surface nonetheless stays engine-agnostic so a
//! contingency in-house lane can implement it.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::fmt;

pub const KERNEL_SCHEMA_VERSION: u32 = 1;
pub const TENANT_ID_MAX_LEN: usize = 128;
pub const HYPERTABLE_NAME_MAX_LEN: usize = 96;

/// TSL-only function names that the Apache-2.0 community-edition fence
/// (ADR-0194 §"TSL component fence") forbids. The kernel rejects any
/// hypertable / continuous-aggregate / retention declaration whose SQL
/// fragment matches one of these names.
pub const FORBIDDEN_TSL_FUNCTIONS: &[&str] = &[
    // automated background refresh / retention / compression policy APIs
    "add_retention_policy",
    "add_compression_policy",
    "add_continuous_aggregate_policy",
    "add_reorder_policy",
    "policy_compression",
    "policy_refresh_continuous_aggregate",
    "policy_retention",
    // tiered storage
    "tiered_storage",
    "attach_tiered_chunk",
    // hyperfunctions — selected core (oyatie does NOT depend on these)
    "approx_percentile",
    "approx_count_distinct",
    "asof_join",
    "lttb",
    "timeweight",
    "time_weight",
    "interpolated_average",
    "interpolated_integral",
    "rolling_avg",
    "rolling_stderror",
    // OSM-only "skip-scan" optimizer hints (TSL)
    "skip_scan_enabled",
];

/// Cardinality class per ADR-0194 §"Hypertable patterns — chunk sizing".
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CardinalityClass {
    /// ≤ 1K series per tenant → chunk interval 7d.
    Low,
    /// 1K–100K series per tenant → chunk interval 1d.
    Medium,
    /// >100K series per tenant → chunk interval 6h.
    High,
    /// >1M series per tenant → chunk interval 1h; runbook flag for capacity.
    VeryHigh,
}

impl CardinalityClass {
    /// Canonical chunk interval per ADR-0194 §"Hypertable patterns".
    pub const fn chunk_interval_seconds(self) -> u64 {
        match self {
            Self::Low => 7 * 24 * 3600,
            Self::Medium => 24 * 3600,
            Self::High => 6 * 3600,
            Self::VeryHigh => 3600,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::VeryHigh => "very_high",
        }
    }

    pub fn classify_series_count(count: u64) -> Self {
        if count <= 1_000 {
            Self::Low
        } else if count <= 100_000 {
            Self::Medium
        } else if count <= 1_000_000 {
            Self::High
        } else {
            Self::VeryHigh
        }
    }
}

/// Validated tenant id. Locally defined; zero-dep.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenantId(String);

impl TenantId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, KernelError> {
        let value = value.into();
        if value.is_empty() {
            return Err(KernelError::TenantIdEmpty);
        }
        if value.len() > TENANT_ID_MAX_LEN {
            return Err(KernelError::TenantIdTooLong {
                actual: value.len(),
            });
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(KernelError::TenantIdInvalidChar);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Validated hypertable name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HypertableName(String);

impl HypertableName {
    pub fn try_new(value: impl Into<String>) -> Result<Self, KernelError> {
        let value = value.into();
        if value.is_empty() {
            return Err(KernelError::HypertableNameEmpty);
        }
        if value.len() > HYPERTABLE_NAME_MAX_LEN {
            return Err(KernelError::HypertableNameTooLong {
                actual: value.len(),
            });
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(KernelError::HypertableNameInvalidChar);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HypertableName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Time-series column type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SeriesColumnType {
    Float64,
    Int64,
    UInt64,
    Boolean,
    Text,
}

/// Hypertable schema. Time column + optional space (partition) column +
/// data columns. ADR-0194 §"Hypertable patterns".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HypertableSchema {
    pub name: HypertableName,
    pub time_column: String,
    /// Optional secondary partition column (e.g., `tenant_id`); enables
    /// per-tenant partition pruning when set.
    pub space_column: Option<String>,
    pub columns: Vec<(String, SeriesColumnType)>,
    pub cardinality_class: CardinalityClass,
    /// Retention window in days; enforced by per-µservice retention worker
    /// (TSL fence — kernel does NOT emit `add_retention_policy`).
    pub retention_days: u32,
}

impl HypertableSchema {
    /// Derived chunk interval per the cardinality class.
    pub fn chunk_interval_seconds(&self) -> u64 {
        self.cardinality_class.chunk_interval_seconds()
    }
}

/// Continuous-aggregate (CAGG) declaration. The kernel emits the
/// `CREATE MATERIALIZED VIEW ... WITH (timescaledb.continuous)` DDL; the
/// per-µservice refresh worker invokes `CALL refresh_continuous_aggregate`
/// on the declared interval (TSL fence — kernel does NOT emit
/// `add_continuous_aggregate_policy`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuousAggregateSchema {
    pub view_name: HypertableName,
    pub source: HypertableName,
    /// SELECT expression — must use `time_bucket()` (Apache-2.0 surface,
    /// not TSL).
    pub select_expr: String,
    /// Refresh interval in seconds; consumed by the refresh worker.
    pub refresh_interval_seconds: u64,
}

/// Retention-policy declaration. Executed by the per-µservice retention
/// worker calling `SELECT drop_chunks(...)` (Apache-2.0 surface).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    pub hypertable: HypertableName,
    pub retention_days: u32,
}

/// Time-series sample row submitted via `insert`.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub time_epoch_seconds: u64,
    /// Tenant id ALSO carried in the row (for `space_column` partition
    /// pruning + row-level-security per ADR-0184 Tier 1).
    pub tenant_id: TenantId,
    pub values: BTreeMap<String, SeriesValue>,
}

/// Cell value within a sample row.
#[derive(Clone, Debug, PartialEq)]
pub enum SeriesValue {
    Float64(f64),
    Int64(i64),
    UInt64(u64),
    Boolean(bool),
    Text(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KernelError {
    TenantIdEmpty,
    TenantIdTooLong { actual: usize },
    TenantIdInvalidChar,
    HypertableNameEmpty,
    HypertableNameTooLong { actual: usize },
    HypertableNameInvalidChar,
    /// TSL-fence violation per ADR-0194 §"TSL component fence".
    TslFenceViolation { function_name: String },
    /// `time_bucket` is required in continuous-aggregate SELECT expressions.
    ContinuousAggregateMissingTimeBucket,
    UnknownColumn { column: String },
    AdapterError(String),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TenantIdEmpty => write!(f, "tenant id is empty"),
            Self::TenantIdTooLong { actual } => write!(f, "tenant id length {actual} exceeds {TENANT_ID_MAX_LEN}"),
            Self::TenantIdInvalidChar => write!(f, "tenant id contains invalid character"),
            Self::HypertableNameEmpty => write!(f, "hypertable name is empty"),
            Self::HypertableNameTooLong { actual } => write!(f, "hypertable name length {actual} exceeds {HYPERTABLE_NAME_MAX_LEN}"),
            Self::HypertableNameInvalidChar => write!(f, "hypertable name contains invalid character"),
            Self::TslFenceViolation { function_name } => {
                write!(f, "TSL fence violation: function {function_name} is TSL-only (ADR-0194)")
            }
            Self::ContinuousAggregateMissingTimeBucket => {
                write!(f, "continuous aggregate SELECT must use time_bucket()")
            }
            Self::UnknownColumn { column } => write!(f, "unknown column {column}"),
            Self::AdapterError(msg) => write!(f, "adapter error: {msg}"),
        }
    }
}

impl std::error::Error for KernelError {}

/// Engine-agnostic time-series port.
pub trait TimeseriesStore {
    /// Idempotent hypertable creation. Adapter emits
    /// `SELECT create_hypertable(...)` for the TimescaleDB adapter; in-house
    /// future adapter emits its native partitioned-table DDL.
    fn ensure_hypertable(&mut self, schema: &HypertableSchema) -> Result<(), KernelError>;

    /// Idempotent continuous-aggregate creation.
    fn ensure_continuous_aggregate(
        &mut self,
        schema: &ContinuousAggregateSchema,
    ) -> Result<(), KernelError>;

    /// Insert a batch of samples; the adapter MAY batch within a single
    /// transaction for throughput.
    fn insert(
        &mut self,
        hypertable: &HypertableName,
        samples: &[Sample],
    ) -> Result<u64, KernelError>;

    /// Per-tenant DSR cascade — delete all rows for the tenant in the
    /// hypertable. Implemented as `DELETE FROM ... WHERE tenant_id = $1`.
    fn dsr_delete_tenant(
        &mut self,
        hypertable: &HypertableName,
        tenant_id: &TenantId,
    ) -> Result<u64, KernelError>;

    /// Drop chunks older than the retention horizon — invoked by the
    /// per-µservice retention worker.
    fn drop_chunks_older_than(
        &mut self,
        hypertable: &HypertableName,
        older_than_epoch_seconds: u64,
    ) -> Result<u64, KernelError>;

    /// Refresh a continuous aggregate over the (start, end) window —
    /// invoked by the per-µservice refresh worker.
    fn refresh_continuous_aggregate(
        &mut self,
        view: &HypertableName,
        start_epoch_seconds: u64,
        end_epoch_seconds: u64,
    ) -> Result<(), KernelError>;
}

/// Scan a SQL fragment for TSL-fenced function names. Returns the first
/// violation encountered (so callers can fail fast).
pub fn check_tsl_fence(sql_fragment: &str) -> Result<(), KernelError> {
    for func in FORBIDDEN_TSL_FUNCTIONS {
        if sql_fragment.contains(func) {
            return Err(KernelError::TslFenceViolation {
                function_name: (*func).to_string(),
            });
        }
    }
    Ok(())
}

/// Validate a continuous-aggregate SELECT expression — must use
/// `time_bucket(`, must not name TSL functions.
pub fn validate_cagg_select(select_expr: &str) -> Result<(), KernelError> {
    if !select_expr.contains("time_bucket(") {
        return Err(KernelError::ContinuousAggregateMissingTimeBucket);
    }
    check_tsl_fence(select_expr)
}

/// In-process reference adapter.
pub mod memory_adapter {
    use super::*;

    #[derive(Debug, Default)]
    pub struct InMemoryTimeseriesStore {
        tables: BTreeMap<String, HypertableState>,
        views: BTreeMap<String, ContinuousAggregateSchema>,
    }

    #[derive(Debug)]
    struct HypertableState {
        schema: HypertableSchema,
        samples: Vec<Sample>,
    }

    impl InMemoryTimeseriesStore {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl TimeseriesStore for InMemoryTimeseriesStore {
        fn ensure_hypertable(&mut self, schema: &HypertableSchema) -> Result<(), KernelError> {
            let key = schema.name.as_str().to_string();
            if let Some(existing) = self.tables.get(&key) {
                if existing.schema != *schema {
                    return Err(KernelError::AdapterError(format!(
                        "schema drift on hypertable {key}"
                    )));
                }
                return Ok(());
            }
            self.tables.insert(
                key,
                HypertableState {
                    schema: schema.clone(),
                    samples: Vec::new(),
                },
            );
            Ok(())
        }

        fn ensure_continuous_aggregate(
            &mut self,
            schema: &ContinuousAggregateSchema,
        ) -> Result<(), KernelError> {
            validate_cagg_select(&schema.select_expr)?;
            self.views.insert(schema.view_name.as_str().to_string(), schema.clone());
            Ok(())
        }

        fn insert(
            &mut self,
            hypertable: &HypertableName,
            samples: &[Sample],
        ) -> Result<u64, KernelError> {
            let state = self.tables.get_mut(hypertable.as_str()).ok_or_else(|| {
                KernelError::AdapterError(format!("hypertable {hypertable} does not exist"))
            })?;
            state.samples.extend(samples.iter().cloned());
            Ok(samples.len() as u64)
        }

        fn dsr_delete_tenant(
            &mut self,
            hypertable: &HypertableName,
            tenant_id: &TenantId,
        ) -> Result<u64, KernelError> {
            let state = self.tables.get_mut(hypertable.as_str()).ok_or_else(|| {
                KernelError::AdapterError(format!("hypertable {hypertable} does not exist"))
            })?;
            let before = state.samples.len() as u64;
            state.samples.retain(|s| s.tenant_id != *tenant_id);
            Ok(before - state.samples.len() as u64)
        }

        fn drop_chunks_older_than(
            &mut self,
            hypertable: &HypertableName,
            older_than_epoch_seconds: u64,
        ) -> Result<u64, KernelError> {
            let state = self.tables.get_mut(hypertable.as_str()).ok_or_else(|| {
                KernelError::AdapterError(format!("hypertable {hypertable} does not exist"))
            })?;
            let before = state.samples.len() as u64;
            state
                .samples
                .retain(|s| s.time_epoch_seconds >= older_than_epoch_seconds);
            Ok(before - state.samples.len() as u64)
        }

        fn refresh_continuous_aggregate(
            &mut self,
            view: &HypertableName,
            _start: u64,
            _end: u64,
        ) -> Result<(), KernelError> {
            if !self.views.contains_key(view.as_str()) {
                return Err(KernelError::AdapterError(format!(
                    "continuous aggregate {view} does not exist"
                )));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::memory_adapter::InMemoryTimeseriesStore;
    use super::*;

    fn tid(s: &str) -> TenantId {
        TenantId::try_new(s).unwrap()
    }

    fn hname(s: &str) -> HypertableName {
        HypertableName::try_new(s).unwrap()
    }

    #[test]
    fn cardinality_class_chunk_intervals_pinned_to_adr_0194() {
        assert_eq!(CardinalityClass::Low.chunk_interval_seconds(), 7 * 86_400);
        assert_eq!(CardinalityClass::Medium.chunk_interval_seconds(), 86_400);
        assert_eq!(CardinalityClass::High.chunk_interval_seconds(), 6 * 3600);
        assert_eq!(CardinalityClass::VeryHigh.chunk_interval_seconds(), 3600);
    }

    #[test]
    fn cardinality_class_classifies_series_count() {
        assert_eq!(CardinalityClass::classify_series_count(500), CardinalityClass::Low);
        assert_eq!(
            CardinalityClass::classify_series_count(50_000),
            CardinalityClass::Medium
        );
        assert_eq!(
            CardinalityClass::classify_series_count(500_000),
            CardinalityClass::High
        );
        assert_eq!(
            CardinalityClass::classify_series_count(5_000_000),
            CardinalityClass::VeryHigh
        );
    }

    #[test]
    fn tsl_fence_rejects_forbidden_function_names() {
        assert!(check_tsl_fence("SELECT count(*) FROM events").is_ok());
        let err = check_tsl_fence("SELECT add_retention_policy('events', INTERVAL '30 days')")
            .unwrap_err();
        assert_eq!(
            err,
            KernelError::TslFenceViolation {
                function_name: "add_retention_policy".into()
            }
        );
        let err = check_tsl_fence("SELECT approx_percentile(0.99, foo) FROM events").unwrap_err();
        assert!(matches!(err, KernelError::TslFenceViolation { .. }));
    }

    #[test]
    fn tsl_fence_rejects_compression_policy() {
        let err = check_tsl_fence("CALL add_compression_policy('events', INTERVAL '7 days')")
            .unwrap_err();
        assert_eq!(
            err,
            KernelError::TslFenceViolation {
                function_name: "add_compression_policy".into()
            }
        );
    }

    #[test]
    fn cagg_select_requires_time_bucket() {
        assert_eq!(
            validate_cagg_select("SELECT tenant_id, count(*) FROM events GROUP BY tenant_id"),
            Err(KernelError::ContinuousAggregateMissingTimeBucket)
        );
        assert!(validate_cagg_select(
            "SELECT time_bucket('1 hour', ts) AS bucket, tenant_id, count(*) FROM events"
        )
        .is_ok());
    }

    #[test]
    fn cagg_select_rejects_tsl_function_even_with_time_bucket() {
        let err = validate_cagg_select(
            "SELECT time_bucket('1 hour', ts), approx_percentile(0.99, v) FROM events",
        )
        .unwrap_err();
        assert!(matches!(err, KernelError::TslFenceViolation { .. }));
    }

    #[test]
    fn hypertable_name_rejects_invalid_chars() {
        assert!(HypertableName::try_new("events_metrics").is_ok());
        assert_eq!(
            HypertableName::try_new("Events"),
            Err(KernelError::HypertableNameInvalidChar)
        );
        assert_eq!(
            HypertableName::try_new(""),
            Err(KernelError::HypertableNameEmpty)
        );
    }

    #[test]
    fn memory_adapter_hypertable_round_trip() {
        let mut s = InMemoryTimeseriesStore::new();
        let h = hname("events_metrics");
        let schema = HypertableSchema {
            name: h.clone(),
            time_column: "ts".into(),
            space_column: Some("tenant_id".into()),
            columns: vec![
                ("ts".into(), SeriesColumnType::UInt64),
                ("tenant_id".into(), SeriesColumnType::Text),
                ("value".into(), SeriesColumnType::Float64),
            ],
            cardinality_class: CardinalityClass::Medium,
            retention_days: 90,
        };
        s.ensure_hypertable(&schema).unwrap();
        let sample = Sample {
            time_epoch_seconds: 1_000,
            tenant_id: tid("ten_acme"),
            values: {
                let mut m = BTreeMap::new();
                m.insert("value".into(), SeriesValue::Float64(42.0));
                m
            },
        };
        let n = s.insert(&h, &[sample]).unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn memory_adapter_dsr_deletes_tenant_samples() {
        let mut s = InMemoryTimeseriesStore::new();
        let h = hname("events");
        s.ensure_hypertable(&HypertableSchema {
            name: h.clone(),
            time_column: "ts".into(),
            space_column: None,
            columns: vec![("ts".into(), SeriesColumnType::UInt64)],
            cardinality_class: CardinalityClass::Low,
            retention_days: 30,
        })
        .unwrap();
        for (epoch, tnt) in [(100, "ten_a"), (200, "ten_b"), (300, "ten_a")] {
            s.insert(
                &h,
                &[Sample {
                    time_epoch_seconds: epoch,
                    tenant_id: tid(tnt),
                    values: BTreeMap::new(),
                }],
            )
            .unwrap();
        }
        let deleted = s.dsr_delete_tenant(&h, &tid("ten_a")).unwrap();
        assert_eq!(deleted, 2);
    }

    #[test]
    fn memory_adapter_drop_chunks_evicts_old_rows() {
        let mut s = InMemoryTimeseriesStore::new();
        let h = hname("events");
        s.ensure_hypertable(&HypertableSchema {
            name: h.clone(),
            time_column: "ts".into(),
            space_column: None,
            columns: vec![("ts".into(), SeriesColumnType::UInt64)],
            cardinality_class: CardinalityClass::Low,
            retention_days: 30,
        })
        .unwrap();
        for epoch in [100, 200, 300, 400, 500] {
            s.insert(
                &h,
                &[Sample {
                    time_epoch_seconds: epoch,
                    tenant_id: tid("ten_a"),
                    values: BTreeMap::new(),
                }],
            )
            .unwrap();
        }
        let dropped = s.drop_chunks_older_than(&h, 300).unwrap();
        assert_eq!(dropped, 2);
    }

    #[test]
    fn memory_adapter_continuous_aggregate_validated_on_create() {
        let mut s = InMemoryTimeseriesStore::new();
        let view = hname("events_hourly");
        let source = hname("events");
        // First create the source hypertable
        s.ensure_hypertable(&HypertableSchema {
            name: source.clone(),
            time_column: "ts".into(),
            space_column: None,
            columns: vec![("ts".into(), SeriesColumnType::UInt64)],
            cardinality_class: CardinalityClass::Low,
            retention_days: 30,
        })
        .unwrap();
        // Valid CAGG with time_bucket
        s.ensure_continuous_aggregate(&ContinuousAggregateSchema {
            view_name: view.clone(),
            source: source.clone(),
            select_expr:
                "SELECT time_bucket('1 hour', ts) AS bucket, count(*) FROM events".into(),
            refresh_interval_seconds: 300,
        })
        .unwrap();
        // Refresh works on existing view
        assert!(s.refresh_continuous_aggregate(&view, 0, 1_000).is_ok());
        // Refresh fails on missing view
        let err = s
            .refresh_continuous_aggregate(&hname("nope"), 0, 1_000)
            .unwrap_err();
        assert!(matches!(err, KernelError::AdapterError(_)));
    }

    #[test]
    fn memory_adapter_cagg_rejects_tsl_function() {
        let mut s = InMemoryTimeseriesStore::new();
        let view = hname("events_p99");
        let source = hname("events");
        s.ensure_hypertable(&HypertableSchema {
            name: source.clone(),
            time_column: "ts".into(),
            space_column: None,
            columns: vec![("ts".into(), SeriesColumnType::UInt64)],
            cardinality_class: CardinalityClass::Low,
            retention_days: 30,
        })
        .unwrap();
        let err = s
            .ensure_continuous_aggregate(&ContinuousAggregateSchema {
                view_name: view,
                source,
                select_expr: "SELECT time_bucket('1 hour', ts), approx_percentile(0.99, v) FROM events"
                    .into(),
                refresh_interval_seconds: 300,
            })
            .unwrap_err();
        assert!(matches!(err, KernelError::TslFenceViolation { .. }));
    }

    #[test]
    fn schema_drift_on_re_ensure_returns_adapter_error() {
        let mut s = InMemoryTimeseriesStore::new();
        let h = hname("events");
        let schema_a = HypertableSchema {
            name: h.clone(),
            time_column: "ts".into(),
            space_column: None,
            columns: vec![("ts".into(), SeriesColumnType::UInt64)],
            cardinality_class: CardinalityClass::Low,
            retention_days: 30,
        };
        let schema_b = HypertableSchema {
            cardinality_class: CardinalityClass::High, // drift
            ..schema_a.clone()
        };
        s.ensure_hypertable(&schema_a).unwrap();
        let err = s.ensure_hypertable(&schema_b).unwrap_err();
        assert!(matches!(err, KernelError::AdapterError(_)));
    }

    #[test]
    fn hypertable_schema_resolves_chunk_interval_via_cardinality_class() {
        let schema = HypertableSchema {
            name: hname("events"),
            time_column: "ts".into(),
            space_column: None,
            columns: vec![("ts".into(), SeriesColumnType::UInt64)],
            cardinality_class: CardinalityClass::High,
            retention_days: 90,
        };
        assert_eq!(schema.chunk_interval_seconds(), 6 * 3600);
    }
}
