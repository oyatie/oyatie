//! Analytics domain — pure business logic, no I/O, no async (ADR-0083 Tier-3).
//!
//! ## Bounded contexts
//!
//! - **tenant-dashboard-query** — [`TenantDashboardQuery`] aggregate root.
//! - **billing-rollup** — [`BillingRollup`] aggregate root.
//! - **audit-log-search** — [`AuditLogSearch`] aggregate root.
//! - **data-export** — [`DataExport`] aggregate root.
//!
//! ## Tenancy
//!
//! Every aggregate carries a [`TenantId`] (re-exported from the kernel). Domain
//! operations that produce [`Query`] values for the OLAP port always embed the
//! owning tenant's ID so the adapter can enforce per-tenant isolation via
//! `assert_same_tenant`.
//!
//! ## Honest-claims note
//!
//! Status is "planned". Aggregate query-building is functional; full
//! event-sourcing / command/event wiring is deferred per manifest status.
//!
//! non_claim: no persistence, no event-bus wiring, no live Cedar authorization
//! call in this scaffolding.

// ADR-0083 Tier 3: production code stays panic-free; tests use unwrap/expect.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use serde::{Deserialize, Serialize};

pub use shared_olap_client_kernel::{
    Aggregate, Filter, KernelError, OrderBy, OrderDir, QualifiedTable, Query, Row, TableName,
    TenantId, Value,
};

// =====================================================================
// Shared value types
// =====================================================================

/// Time range for analytic queries (inclusive on both ends, epoch seconds).
///
/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Epoch seconds, start (inclusive).
    pub from_secs: u64,
    /// Epoch seconds, end (inclusive).
    pub to_secs: u64,
}

impl TimeRange {
    /// Validate that `from_secs < to_secs`.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidTimeRange`] if `from_secs >= to_secs`.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.from_secs >= self.to_secs {
            Err(DomainError::InvalidTimeRange)
        } else {
            Ok(())
        }
    }
}

/// Cursor-based pagination parameters.
///
/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: u64,
}

impl Pagination {
    /// Default page size used when no limit is specified.
    pub const DEFAULT_LIMIT: u64 = 100;

    #[must_use]
    pub fn new(limit: u64) -> Self {
        Self { limit }
    }
}

// =====================================================================
// Domain error
// =====================================================================

/// Domain-layer errors (not engine errors; those surface via [`KernelError`]).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    /// A dashboard query was submitted with zero metrics.
    EmptyMetrics,
    /// Time range is invalid (`from_secs >= to_secs`).
    InvalidTimeRange,
    /// The tenant ID is syntactically invalid.
    InvalidTenantId(KernelError),
    /// The table name is syntactically invalid.
    InvalidTableName(KernelError),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMetrics => write!(f, "at least one metric must be requested"),
            Self::InvalidTimeRange => write!(f, "time range 'from' must precede 'to'"),
            Self::InvalidTenantId(e) => write!(f, "invalid tenant id: {e}"),
            Self::InvalidTableName(e) => write!(f, "invalid table name: {e}"),
        }
    }
}

impl std::error::Error for DomainError {}

// =====================================================================
// Tenant-dashboard-query bounded context
// =====================================================================

/// Metric key requested by a tenant dashboard.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DashboardMetric {
    WorkflowExecutionCount,
    ApiCallCount,
    ErrorRate,
    P99LatencyMs,
    ActiveUsers,
    StorageUsedBytes,
}

impl DashboardMetric {
    /// The ClickHouse column name for this metric.
    #[must_use]
    pub fn column_name(&self) -> &'static str {
        match self {
            Self::WorkflowExecutionCount => "workflow_execution_count",
            Self::ApiCallCount => "api_call_count",
            Self::ErrorRate => "error_rate",
            Self::P99LatencyMs => "p99_latency_ms",
            Self::ActiveUsers => "active_users",
            Self::StorageUsedBytes => "storage_used_bytes",
        }
    }
}

/// Aggregate root for a tenant dashboard query request.
///
/// data_class: TENANT_PUBLIC
#[derive(Clone, Debug)]
pub struct TenantDashboardQuery {
    pub tenant_id: TenantId,
    pub metrics: Vec<DashboardMetric>,
    pub time_range: TimeRange,
    pub pagination: Pagination,
}

impl TenantDashboardQuery {
    /// Build a [`Query`] scoped to this tenant for the OLAP port.
    ///
    /// # Errors
    /// Returns [`DomainError::EmptyMetrics`] if no metrics are requested, or
    /// [`DomainError::InvalidTimeRange`] / [`DomainError::InvalidTableName`]
    /// on invalid inputs.
    pub fn to_olap_query(&self) -> Result<Query, DomainError> {
        if self.metrics.is_empty() {
            return Err(DomainError::EmptyMetrics);
        }
        self.time_range.validate()?;
        let table_name =
            TableName::try_new("tenant_metrics").map_err(DomainError::InvalidTableName)?;
        let table = QualifiedTable::new(self.tenant_id.clone(), table_name);
        let columns: Vec<String> = self
            .metrics
            .iter()
            .map(|m| m.column_name().to_string())
            .collect();
        Ok(Query {
            source: table,
            columns,
            aggregates: vec![],
            filter: Some(Filter::And(vec![
                Filter::Ge {
                    column: "ts".to_string(),
                    value: Value::DateTime(self.time_range.from_secs),
                },
                Filter::Le {
                    column: "ts".to_string(),
                    value: Value::DateTime(self.time_range.to_secs),
                },
            ])),
            group_by: vec![],
            order_by: vec![OrderBy {
                column: "ts".to_string(),
                dir: OrderDir::Desc,
            }],
            limit: Some(self.pagination.limit),
        })
    }
}

// =====================================================================
// Billing-rollup bounded context
// =====================================================================

/// Granularity of a billing rollup aggregation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RollupGranularity {
    Hourly,
    Daily,
    Monthly,
}

impl fmt::Display for RollupGranularity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hourly => write!(f, "hourly"),
            Self::Daily => write!(f, "daily"),
            Self::Monthly => write!(f, "monthly"),
        }
    }
}

/// Aggregate root for a billing rollup request (IP-009).
///
/// data_class: FINANCIAL
#[derive(Clone, Debug)]
pub struct BillingRollup {
    pub tenant_id: TenantId,
    pub granularity: RollupGranularity,
    pub time_range: TimeRange,
}

impl BillingRollup {
    /// Build a [`Query`] for the billing rollup.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidTimeRange`] if `from_secs >= to_secs`.
    pub fn to_olap_query(&self) -> Result<Query, DomainError> {
        self.time_range.validate()?;
        let table_name =
            TableName::try_new("billing_events").map_err(DomainError::InvalidTableName)?;
        let table = QualifiedTable::new(self.tenant_id.clone(), table_name);
        Ok(Query {
            source: table,
            columns: vec![],
            aggregates: vec![(
                Aggregate::Sum {
                    column: "amount".to_string(),
                },
                "total".to_string(),
            )],
            filter: Some(Filter::And(vec![
                Filter::Ge {
                    column: "ts".to_string(),
                    value: Value::DateTime(self.time_range.from_secs),
                },
                Filter::Le {
                    column: "ts".to_string(),
                    value: Value::DateTime(self.time_range.to_secs),
                },
            ])),
            group_by: vec!["period".to_string()],
            order_by: vec![OrderBy {
                column: "period".to_string(),
                dir: OrderDir::Asc,
            }],
            limit: None,
        })
    }
}

// =====================================================================
// Audit-log-search bounded context
// =====================================================================

/// Filter axes for audit log search (IP-008).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLogFilter {
    pub actor_id: Option<String>,
    pub resource_type: Option<String>,
    pub action: Option<String>,
    pub time_range: Option<TimeRange>,
}

/// Aggregate root for an audit log search request.
///
/// data_class: AUDIT_INTERNAL, TENANT_AUDIT
#[derive(Clone, Debug)]
pub struct AuditLogSearch {
    pub tenant_id: TenantId,
    pub filter: AuditLogFilter,
    pub pagination: Pagination,
}

impl AuditLogSearch {
    /// Build a [`Query`] for audit log search.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidTableName`] if the table name is invalid.
    pub fn to_olap_query(&self) -> Result<Query, DomainError> {
        let table_name = TableName::try_new("audit_log").map_err(DomainError::InvalidTableName)?;
        let table = QualifiedTable::new(self.tenant_id.clone(), table_name);

        let mut filters: Vec<Filter> = Vec::new();
        if let Some(actor_id) = &self.filter.actor_id {
            filters.push(Filter::Eq {
                column: "actor_id".to_string(),
                value: Value::String(actor_id.clone()),
            });
        }
        if let Some(action) = &self.filter.action {
            filters.push(Filter::Eq {
                column: "action".to_string(),
                value: Value::String(action.clone()),
            });
        }
        if let Some(tr) = &self.filter.time_range {
            filters.push(Filter::Ge {
                column: "ts".to_string(),
                value: Value::DateTime(tr.from_secs),
            });
            filters.push(Filter::Le {
                column: "ts".to_string(),
                value: Value::DateTime(tr.to_secs),
            });
        }

        let filter = if filters.is_empty() {
            None
        } else {
            Some(Filter::And(filters))
        };

        Ok(Query {
            source: table,
            columns: vec![
                "ts".to_string(),
                "actor_id".to_string(),
                "action".to_string(),
                "resource_id".to_string(),
            ],
            aggregates: vec![],
            filter,
            group_by: vec![],
            order_by: vec![OrderBy {
                column: "ts".to_string(),
                dir: OrderDir::Desc,
            }],
            limit: Some(self.pagination.limit),
        })
    }
}

// =====================================================================
// Data-export bounded context
// =====================================================================

/// Export format for regulator or tenant-initiated data exports (IP-013).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ExportFormat {
    JsonLines,
    Parquet,
    Csv,
}

/// Export scope — which dataset to export.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ExportScope {
    AuditLog { time_range: TimeRange },
    BillingHistory { time_range: TimeRange },
    WorkflowExecutions { time_range: TimeRange },
}

/// Aggregate root for a data export request.
///
/// data_class: TENANT_PRIVATE, FINANCIAL, PII_IDENTIFYING (depends on scope)
#[derive(Clone, Debug)]
pub struct DataExport {
    pub tenant_id: TenantId,
    pub scope: ExportScope,
    pub format: ExportFormat,
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn tid(s: &str) -> TenantId {
        TenantId::try_new(s).unwrap()
    }

    fn range() -> TimeRange {
        TimeRange {
            from_secs: 1_735_689_600, // 2026-01-01
            to_secs: 1_738_368_000,   // 2026-02-01
        }
    }

    #[test]
    fn dashboard_query_to_olap_query_scoped_to_tenant() {
        let dq = TenantDashboardQuery {
            tenant_id: tid("t1"),
            metrics: vec![DashboardMetric::WorkflowExecutionCount],
            time_range: range(),
            pagination: Pagination::new(50),
        };
        let oq = dq.to_olap_query().unwrap();
        assert_eq!(oq.source.tenant_id().as_str(), "t1");
        assert_eq!(oq.limit, Some(50));
        assert!(oq.columns.contains(&"workflow_execution_count".to_string()));
    }

    #[test]
    fn dashboard_query_empty_metrics_is_error() {
        let dq = TenantDashboardQuery {
            tenant_id: tid("t1"),
            metrics: vec![],
            time_range: range(),
            pagination: Pagination::new(10),
        };
        assert_eq!(dq.to_olap_query().unwrap_err(), DomainError::EmptyMetrics);
    }

    #[test]
    fn billing_rollup_invalid_time_range() {
        let br = BillingRollup {
            tenant_id: tid("t1"),
            granularity: RollupGranularity::Daily,
            time_range: TimeRange {
                from_secs: 1_738_368_000,
                to_secs: 1_735_689_600, // from > to
            },
        };
        assert_eq!(
            br.to_olap_query().unwrap_err(),
            DomainError::InvalidTimeRange
        );
    }

    #[test]
    fn billing_rollup_valid_query_has_sum_aggregate() {
        let br = BillingRollup {
            tenant_id: tid("t1"),
            granularity: RollupGranularity::Daily,
            time_range: range(),
        };
        let q = br.to_olap_query().unwrap();
        assert_eq!(q.source.tenant_id().as_str(), "t1");
        assert!(!q.aggregates.is_empty());
    }

    #[test]
    fn audit_log_search_produces_scoped_query() {
        let s = AuditLogSearch {
            tenant_id: tid("t1"),
            filter: AuditLogFilter {
                actor_id: Some("user-42".to_string()),
                ..Default::default()
            },
            pagination: Pagination::new(25),
        };
        let q = s.to_olap_query().unwrap();
        assert_eq!(q.source.tenant_id().as_str(), "t1");
        assert_eq!(q.limit, Some(25));
    }
}
