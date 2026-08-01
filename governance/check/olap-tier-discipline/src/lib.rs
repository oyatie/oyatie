//! `check-olap-tier-discipline` — advisory gate per ADR-0193 §"Use
//! cases — what ClickHouse owns" and ADR-0184 §"Tier boundary rules".
//!
//! Ensures no µservice runs wide-aggregate queries against Tier 1
//! Postgres OLTP — those workloads belong on ClickHouse (Phase 0) or the
//! Phase-2 in-house `oya-olap-warehouse-server`.
//!
//! Heuristic — the runner pre-harvests SQL-fragment usage from
//! µservices (either by scanning `*.sql` files, `query!()`/`sqlx`
//! invocations, or by sampling Postgres `pg_stat_statements`) and tags
//! each fragment with `(microservice, source_file, tier, sql_excerpt)`.
//! This kernel flags Tier 1 OLTP fragments that exhibit "wide aggregate"
//! shape:
//!
//!   - `GROUP BY` over a non-indexed dimension, OR
//!   - `COUNT(DISTINCT ...)` over > 100K-row table (caller annotates row
//!     count via [`OlapUsage::row_count_estimate`]), OR
//!   - window functions (`OVER`) across multi-month windows.
//!
//! Lane mode follows the canonical [`CheckMode`].
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// Threshold above which `COUNT(DISTINCT)` on a Tier 1 OLTP table is
/// flagged as belonging to ClickHouse per ADR-0193.
pub const WIDE_AGGREGATE_ROW_THRESHOLD: u64 = 100_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CheckMode {
    #[default]
    ReportOnly,
    Blocker,
}

impl CheckMode {
    pub fn is_blocker(self) -> bool {
        matches!(self, Self::Blocker)
    }
}

impl fmt::Display for CheckMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReportOnly => write!(f, "report-only"),
            Self::Blocker => write!(f, "blocker"),
        }
    }
}

/// Storage tier the SQL fragment runs against, per ADR-0184 layering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StorageTier {
    /// Tier 1 — Postgres OLTP write primary.
    OltpPrimary,
    /// Tier 2 — Postgres OLTP read replica.
    OltpReplica,
    /// Tier 3 — Valkey (cache).
    Cache,
    /// Tier 4 — Meilisearch (search).
    Search,
    /// ClickHouse OLAP analytics warehouse per ADR-0193.
    Olap,
    /// TimescaleDB hypertable per ADR-0194 (Postgres-resident).
    TimescaleDbHypertable,
}

impl StorageTier {
    pub const fn label(self) -> &'static str {
        match self {
            Self::OltpPrimary => "oltp_primary",
            Self::OltpReplica => "oltp_replica",
            Self::Cache => "cache",
            Self::Search => "search",
            Self::Olap => "olap",
            Self::TimescaleDbHypertable => "timescaledb_hypertable",
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "oltp_primary" => Some(Self::OltpPrimary),
            "oltp_replica" => Some(Self::OltpReplica),
            "cache" => Some(Self::Cache),
            "search" => Some(Self::Search),
            "olap" => Some(Self::Olap),
            "timescaledb_hypertable" => Some(Self::TimescaleDbHypertable),
            _ => None,
        }
    }

    /// Whether wide-aggregate-on-this-tier is permitted.
    /// OLTP primary + replica forbid wide aggregates. ClickHouse OLAP and
    /// TimescaleDB hypertables are wide-aggregate-friendly.
    pub fn permits_wide_aggregate(self) -> bool {
        matches!(
            self,
            Self::Olap | Self::TimescaleDbHypertable | Self::Cache | Self::Search
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OlapUsage {
    pub microservice: String,
    pub source_file: String,
    pub tier: StorageTier,
    pub sql_excerpt: String,
    /// Optional row-count estimate for the target table; used by the
    /// `COUNT(DISTINCT)` heuristic threshold.
    pub row_count_estimate: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ViolationKind {
    /// Wide aggregate (`GROUP BY` on Tier 1 OLTP).
    WideAggregateOnOltp,
    /// `COUNT(DISTINCT)` over a large table on Tier 1 OLTP.
    HighCardinalityCountDistinctOnOltp,
    /// Window function on Tier 1 OLTP.
    WindowFunctionOnOltp,
    /// Caller fed a malformed record (empty fields).
    MalformedRecord,
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WideAggregateOnOltp => write!(f, "wide_aggregate_on_oltp"),
            Self::HighCardinalityCountDistinctOnOltp => {
                write!(f, "high_cardinality_count_distinct_on_oltp")
            }
            Self::WindowFunctionOnOltp => write!(f, "window_function_on_oltp"),
            Self::MalformedRecord => write!(f, "malformed_record"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub microservice: String,
    pub source_file: String,
    pub kind: ViolationKind,
    pub excerpt: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub records_checked: usize,
    pub violations: Vec<Violation>,
}

fn sql_lc(s: &str) -> String {
    s.to_lowercase()
}

fn looks_like_wide_aggregate(sql: &str) -> bool {
    let lc = sql_lc(sql);
    // GROUP BY appears → wide aggregate shape; the Tier 1 ban is unconditional
    // for the canonical OLTP-primary path. Conservative; better to over-flag
    // and let the reviewer suppress than to under-flag and quietly burn p99.
    lc.contains("group by")
}

fn looks_like_count_distinct(sql: &str) -> bool {
    let lc = sql_lc(sql);
    lc.contains("count(distinct") || lc.contains("count (distinct")
}

fn looks_like_window_function(sql: &str) -> bool {
    let lc = sql_lc(sql);
    lc.contains(" over (") || lc.contains(" over(")
}

pub fn check(records: &[OlapUsage]) -> Report {
    let mut violations = Vec::new();
    for rec in records {
        if rec.microservice.trim().is_empty()
            || rec.source_file.trim().is_empty()
            || rec.sql_excerpt.trim().is_empty()
        {
            violations.push(Violation {
                microservice: rec.microservice.clone(),
                source_file: rec.source_file.clone(),
                kind: ViolationKind::MalformedRecord,
                excerpt: rec.sql_excerpt.clone(),
            });
            continue;
        }

        // Only Tier 1 OLTP primary triggers violations. Replica + OLAP +
        // hypertable + cache + search are all permitted.
        if rec.tier != StorageTier::OltpPrimary {
            continue;
        }

        let sql = &rec.sql_excerpt;
        if looks_like_wide_aggregate(sql) {
            violations.push(Violation {
                microservice: rec.microservice.clone(),
                source_file: rec.source_file.clone(),
                kind: ViolationKind::WideAggregateOnOltp,
                excerpt: sql.clone(),
            });
        }
        if looks_like_count_distinct(sql) && rec.row_count_estimate > WIDE_AGGREGATE_ROW_THRESHOLD {
            violations.push(Violation {
                microservice: rec.microservice.clone(),
                source_file: rec.source_file.clone(),
                kind: ViolationKind::HighCardinalityCountDistinctOnOltp,
                excerpt: sql.clone(),
            });
        }
        if looks_like_window_function(sql) {
            violations.push(Violation {
                microservice: rec.microservice.clone(),
                source_file: rec.source_file.clone(),
                kind: ViolationKind::WindowFunctionOnOltp,
                excerpt: sql.clone(),
            });
        }
    }
    Report {
        records_checked: records.len(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(ms: &str, file: &str, tier: StorageTier, sql: &str, rows: u64) -> OlapUsage {
        OlapUsage {
            microservice: ms.into(),
            source_file: file.into(),
            tier,
            sql_excerpt: sql.into(),
            row_count_estimate: rows,
        }
    }

    #[test]
    fn empty_input_passes() {
        let r = check(&[]);
        assert_eq!(r.records_checked, 0);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn group_by_on_oltp_flagged() {
        let r = check(&[rec(
            "foundry",
            "src/repo.rs",
            StorageTier::OltpPrimary,
            "SELECT tenant_id, count(*) FROM events GROUP BY tenant_id",
            0,
        )]);
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::WideAggregateOnOltp);
    }

    #[test]
    fn group_by_on_olap_permitted() {
        let r = check(&[rec(
            "foundry",
            "src/repo.rs",
            StorageTier::Olap,
            "SELECT tenant_id, count(*) FROM events GROUP BY tenant_id",
            0,
        )]);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn count_distinct_on_small_oltp_table_not_flagged() {
        let r = check(&[rec(
            "foundry",
            "src/repo.rs",
            StorageTier::OltpPrimary,
            "SELECT COUNT(DISTINCT email) FROM users",
            10_000,
        )]);
        // GROUP BY absent + row count < threshold → no violation.
        assert!(r.violations.is_empty());
    }

    #[test]
    fn count_distinct_on_large_oltp_table_flagged() {
        let r = check(&[rec(
            "foundry",
            "src/repo.rs",
            StorageTier::OltpPrimary,
            "SELECT COUNT(DISTINCT email) FROM users",
            10_000_000,
        )]);
        assert_eq!(r.violations.len(), 1);
        assert_eq!(
            r.violations[0].kind,
            ViolationKind::HighCardinalityCountDistinctOnOltp
        );
    }

    #[test]
    fn window_function_on_oltp_flagged() {
        let r = check(&[rec(
            "foundry",
            "src/repo.rs",
            StorageTier::OltpPrimary,
            "SELECT id, row_number() OVER (PARTITION BY t ORDER BY ts) FROM events",
            0,
        )]);
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::WindowFunctionOnOltp);
    }

    #[test]
    fn window_function_on_timescale_permitted() {
        let r = check(&[rec(
            "foundry",
            "src/repo.rs",
            StorageTier::TimescaleDbHypertable,
            "SELECT id, row_number() OVER (PARTITION BY t ORDER BY ts) FROM events",
            0,
        )]);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn malformed_record_flagged() {
        let r = check(&[rec("", "x", StorageTier::OltpPrimary, "...", 0)]);
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::MalformedRecord);
    }

    #[test]
    fn replica_tier_permits_wide_aggregates() {
        let r = check(&[rec(
            "foundry",
            "src/repo.rs",
            StorageTier::OltpReplica,
            "SELECT tenant_id, count(*) FROM events GROUP BY tenant_id",
            10_000_000,
        )]);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn check_mode_default_and_blocker_behaviour() {
        assert_eq!(CheckMode::default(), CheckMode::ReportOnly);
        assert!(CheckMode::Blocker.is_blocker());
        assert!(!CheckMode::ReportOnly.is_blocker());
        assert_eq!(CheckMode::Blocker.to_string(), "blocker");
    }

    #[test]
    fn storage_tier_label_round_trips() {
        for t in [
            StorageTier::OltpPrimary,
            StorageTier::OltpReplica,
            StorageTier::Cache,
            StorageTier::Search,
            StorageTier::Olap,
            StorageTier::TimescaleDbHypertable,
        ] {
            assert_eq!(StorageTier::parse_label(t.label()), Some(t));
        }
        assert_eq!(StorageTier::parse_label("nope"), None);
    }

    #[test]
    fn permits_wide_aggregate_matrix() {
        assert!(!StorageTier::OltpPrimary.permits_wide_aggregate());
        assert!(!StorageTier::OltpReplica.permits_wide_aggregate());
        assert!(StorageTier::Olap.permits_wide_aggregate());
        assert!(StorageTier::TimescaleDbHypertable.permits_wide_aggregate());
        assert!(StorageTier::Cache.permits_wide_aggregate());
        assert!(StorageTier::Search.permits_wide_aggregate());
    }

    #[test]
    fn multi_violation_per_record() {
        // single SQL fragment with GROUP BY + COUNT(DISTINCT) over a big
        // table + window function → three distinct violations.
        let r = check(&[rec(
            "foundry",
            "src/repo.rs",
            StorageTier::OltpPrimary,
            "SELECT tenant_id, COUNT(DISTINCT email), row_number() OVER (PARTITION BY t ORDER BY ts) FROM events GROUP BY tenant_id",
            1_000_000,
        )]);
        assert_eq!(r.violations.len(), 3);
    }
}
