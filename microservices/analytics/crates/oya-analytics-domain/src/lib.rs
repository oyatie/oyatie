//! Analytics µservice domain surface per ADR-0193.
//!
//! This crate owns tenant-facing analytics query shapes before any REST/gRPC or
//! ClickHouse adapter wiring. It translates PRD surfaces (workflow dashboards,
//! audit-log search, and billing rollups) into the zero-I/O OLAP kernel DSL so
//! adapters can render and bind SQL without accepting raw query strings.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_shared_olap_client_kernel::{
    Aggregate, Filter, KernelError, OrderBy, OrderDir, QualifiedTable, Query, RenderedQuery,
    TableName, TenantId, Value, render_clickhouse_query, validate_column_name,
};

pub const ANALYTICS_DOMAIN_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_PAGE_SIZE: u64 = 100;
pub const MAX_PAGE_SIZE: u64 = 10_000;
pub const MAX_INTERACTIVE_WINDOW_SECONDS: u64 = 366 * 24 * 60 * 60;
pub const WORKFLOW_ROLLUP_TABLE: &str = "workflow_hour_rollup";
pub const AUDIT_EVENTS_TABLE: &str = "audit_events";
pub const BILLING_ROLLUP_TABLE: &str = "billing_day_rollup";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalyticsError {
    EmptyAxis,
    InvalidCursor,
    InvalidTimeWindow,
    WindowTooLarge {
        actual_seconds: u64,
        max_seconds: u64,
    },
    PageSizeTooLarge {
        actual: u64,
        max: u64,
    },
    Olap(KernelError),
}

impl From<KernelError> for AnalyticsError {
    fn from(value: KernelError) -> Self {
        Self::Olap(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalyticsSurface {
    WorkflowExecutionDashboard,
    AuditLogSearch,
    BillingRollup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeWindow {
    pub start_epoch_seconds: u64,
    pub end_epoch_seconds: u64,
}

impl TimeWindow {
    pub fn try_new(
        start_epoch_seconds: u64,
        end_epoch_seconds: u64,
    ) -> Result<Self, AnalyticsError> {
        if start_epoch_seconds >= end_epoch_seconds {
            return Err(AnalyticsError::InvalidTimeWindow);
        }
        let actual_seconds = end_epoch_seconds - start_epoch_seconds;
        if actual_seconds > MAX_INTERACTIVE_WINDOW_SECONDS {
            return Err(AnalyticsError::WindowTooLarge {
                actual_seconds,
                max_seconds: MAX_INTERACTIVE_WINDOW_SECONDS,
            });
        }
        Ok(Self {
            start_epoch_seconds,
            end_epoch_seconds,
        })
    }

    fn emitted_at_filter(self) -> Filter {
        Filter::And(vec![
            Filter::Ge {
                column: "emitted_at".to_string(),
                value: Value::DateTime(self.start_epoch_seconds),
            },
            Filter::Lt {
                column: "emitted_at".to_string(),
                value: Value::DateTime(self.end_epoch_seconds),
            },
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditCursor(String);

impl AuditCursor {
    pub fn try_new(value: impl Into<String>) -> Result<Self, AnalyticsError> {
        let value = value.into();
        if value.is_empty() || value.len() > 256 || value.contains('\n') || value.contains('\r') {
            return Err(AnalyticsError::InvalidCursor);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowExecutionDashboardRequest {
    pub tenant_id: TenantId,
    pub window: TimeWindow,
    pub limit: u64,
}

impl WorkflowExecutionDashboardRequest {
    pub fn try_new(
        tenant_id: TenantId,
        window: TimeWindow,
        limit: Option<u64>,
    ) -> Result<Self, AnalyticsError> {
        let limit = bounded_page_size(limit.unwrap_or(DEFAULT_PAGE_SIZE))?;
        Ok(Self {
            tenant_id,
            window,
            limit,
        })
    }

    pub fn surface(&self) -> AnalyticsSurface {
        AnalyticsSurface::WorkflowExecutionDashboard
    }

    pub fn to_olap_query(&self) -> Result<Query, AnalyticsError> {
        Ok(Query {
            source: analytics_table(&self.tenant_id, WORKFLOW_ROLLUP_TABLE)?,
            columns: vec!["hour_bucket".to_string(), "status".to_string()],
            aggregates: vec![
                (
                    Aggregate::Sum {
                        column: "execution_count".to_string(),
                    },
                    "execution_count".to_string(),
                ),
                (
                    Aggregate::Avg {
                        column: "duration_ms_p99".to_string(),
                    },
                    "duration_ms_p99".to_string(),
                ),
            ],
            filter: Some(self.window.emitted_at_filter()),
            group_by: vec!["hour_bucket".to_string(), "status".to_string()],
            order_by: vec![OrderBy {
                column: "hour_bucket".to_string(),
                dir: OrderDir::Asc,
            }],
            limit: Some(self.limit),
        })
    }

    pub fn render_clickhouse(&self) -> Result<RenderedQuery, AnalyticsError> {
        let query = self.to_olap_query()?;
        render_clickhouse_query(&self.tenant_id, &query).map_err(AnalyticsError::from)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditLogSearchRequest {
    pub tenant_id: TenantId,
    pub axis: String,
    pub window: TimeWindow,
    pub page_size: u64,
    pub cursor: Option<AuditCursor>,
}

impl AuditLogSearchRequest {
    pub fn try_new(
        tenant_id: TenantId,
        axis: impl Into<String>,
        window: TimeWindow,
        page_size: Option<u64>,
        cursor: Option<AuditCursor>,
    ) -> Result<Self, AnalyticsError> {
        let axis = axis.into();
        if axis.is_empty() {
            return Err(AnalyticsError::EmptyAxis);
        }
        validate_column_name(&axis).map_err(AnalyticsError::from)?;
        Ok(Self {
            tenant_id,
            axis,
            window,
            page_size: bounded_page_size(page_size.unwrap_or(DEFAULT_PAGE_SIZE))?,
            cursor,
        })
    }

    pub fn surface(&self) -> AnalyticsSurface {
        AnalyticsSurface::AuditLogSearch
    }

    pub fn to_olap_query(&self) -> Result<Query, AnalyticsError> {
        let mut filters = vec![
            self.window.emitted_at_filter(),
            Filter::Eq {
                column: "axis".to_string(),
                value: Value::String(self.axis.clone()),
            },
        ];
        if let Some(cursor) = &self.cursor {
            filters.push(Filter::Lt {
                column: "cursor_key".to_string(),
                value: Value::String(cursor.as_str().to_string()),
            });
        }
        Ok(Query {
            source: analytics_table(&self.tenant_id, AUDIT_EVENTS_TABLE)?,
            columns: vec![
                "event_id".to_string(),
                "axis".to_string(),
                "actor_ref".to_string(),
                "action".to_string(),
                "outcome".to_string(),
                "emitted_at".to_string(),
                "cursor_key".to_string(),
            ],
            aggregates: vec![],
            filter: Some(Filter::And(filters)),
            group_by: vec![],
            order_by: vec![
                OrderBy {
                    column: "emitted_at".to_string(),
                    dir: OrderDir::Desc,
                },
                OrderBy {
                    column: "cursor_key".to_string(),
                    dir: OrderDir::Desc,
                },
            ],
            limit: Some(self.page_size),
        })
    }

    pub fn render_clickhouse(&self) -> Result<RenderedQuery, AnalyticsError> {
        let query = self.to_olap_query()?;
        render_clickhouse_query(&self.tenant_id, &query).map_err(AnalyticsError::from)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingRollupRequest {
    pub tenant_id: TenantId,
    pub window: TimeWindow,
    pub resource_kind: Option<String>,
    pub limit: u64,
}

impl BillingRollupRequest {
    pub fn try_new(
        tenant_id: TenantId,
        window: TimeWindow,
        resource_kind: Option<String>,
        limit: Option<u64>,
    ) -> Result<Self, AnalyticsError> {
        if let Some(resource_kind) = &resource_kind {
            validate_column_name(resource_kind).map_err(AnalyticsError::from)?;
        }
        Ok(Self {
            tenant_id,
            window,
            resource_kind,
            limit: bounded_page_size(limit.unwrap_or(DEFAULT_PAGE_SIZE))?,
        })
    }

    pub fn surface(&self) -> AnalyticsSurface {
        AnalyticsSurface::BillingRollup
    }

    pub fn to_olap_query(&self) -> Result<Query, AnalyticsError> {
        let filter = match &self.resource_kind {
            Some(resource_kind) => Filter::And(vec![
                self.window.emitted_at_filter(),
                Filter::Eq {
                    column: "resource_kind".to_string(),
                    value: Value::String(resource_kind.clone()),
                },
            ]),
            None => self.window.emitted_at_filter(),
        };
        Ok(Query {
            source: analytics_table(&self.tenant_id, BILLING_ROLLUP_TABLE)?,
            columns: vec!["day_bucket".to_string(), "resource_kind".to_string()],
            aggregates: vec![
                (
                    Aggregate::Sum {
                        column: "usage_units".to_string(),
                    },
                    "usage_units".to_string(),
                ),
                (
                    Aggregate::Sum {
                        column: "cost_usd_minor_units".to_string(),
                    },
                    "cost_usd_minor_units".to_string(),
                ),
            ],
            filter: Some(filter),
            group_by: vec!["day_bucket".to_string(), "resource_kind".to_string()],
            order_by: vec![OrderBy {
                column: "day_bucket".to_string(),
                dir: OrderDir::Asc,
            }],
            limit: Some(self.limit),
        })
    }

    pub fn render_clickhouse(&self) -> Result<RenderedQuery, AnalyticsError> {
        let query = self.to_olap_query()?;
        render_clickhouse_query(&self.tenant_id, &query).map_err(AnalyticsError::from)
    }
}

fn analytics_table(tenant_id: &TenantId, table: &str) -> Result<QualifiedTable, AnalyticsError> {
    Ok(QualifiedTable::new(
        tenant_id.clone(),
        TableName::try_new(table).map_err(AnalyticsError::from)?,
    ))
}

fn bounded_page_size(value: u64) -> Result<u64, AnalyticsError> {
    if value == 0 || value > MAX_PAGE_SIZE {
        return Err(AnalyticsError::PageSizeTooLarge {
            actual: value,
            max: MAX_PAGE_SIZE,
        });
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tid() -> TenantId {
        TenantId::try_new("ten_acme").unwrap()
    }

    fn window() -> TimeWindow {
        TimeWindow::try_new(1_776_000_000, 1_776_086_400).unwrap()
    }

    #[test]
    fn workflow_dashboard_renders_tenant_scoped_rollup_query() {
        let req = WorkflowExecutionDashboardRequest::try_new(tid(), window(), Some(50)).unwrap();
        let rendered = req.render_clickhouse().unwrap();

        assert!(
            rendered
                .sql
                .contains("FROM `tenant_ten_acme`.`workflow_hour_rollup`")
        );
        assert!(
            rendered
                .sql
                .contains("sum(`execution_count`) AS `execution_count`")
        );
        assert!(rendered.sql.contains("GROUP BY `hour_bucket`, `status`"));
        assert_eq!(
            rendered.params.get("p0"),
            Some(&Value::DateTime(1_776_000_000))
        );
        assert_eq!(
            rendered.params.get("p1"),
            Some(&Value::DateTime(1_776_086_400))
        );
    }

    #[test]
    fn audit_search_renders_cursor_paginated_query() {
        let req = AuditLogSearchRequest::try_new(
            tid(),
            "identity",
            window(),
            Some(25),
            Some(AuditCursor::try_new("cur/abc").unwrap()),
        )
        .unwrap();
        let rendered = req.render_clickhouse().unwrap();

        assert!(
            rendered
                .sql
                .contains("FROM `tenant_ten_acme`.`audit_events`")
        );
        assert!(rendered.sql.contains("`axis` = {p2:String}"));
        assert!(rendered.sql.contains("`cursor_key` < {p3:String}"));
        assert!(
            rendered
                .sql
                .ends_with("ORDER BY `emitted_at` DESC, `cursor_key` DESC LIMIT 25")
        );
        assert_eq!(
            rendered.params.get("p2"),
            Some(&Value::String("identity".into()))
        );
        assert_eq!(
            rendered.params.get("p3"),
            Some(&Value::String("cur/abc".into()))
        );
    }

    #[test]
    fn billing_rollup_renders_cost_and_usage_aggregates() {
        let req =
            BillingRollupRequest::try_new(tid(), window(), Some("compute".to_string()), Some(365))
                .unwrap();
        let rendered = req.render_clickhouse().unwrap();

        assert!(
            rendered
                .sql
                .contains("FROM `tenant_ten_acme`.`billing_day_rollup`")
        );
        assert!(rendered.sql.contains("sum(`usage_units`) AS `usage_units`"));
        assert!(
            rendered
                .sql
                .contains("sum(`cost_usd_minor_units`) AS `cost_usd_minor_units`")
        );
        assert!(rendered.sql.contains("`resource_kind` = {p2:String}"));
    }

    #[test]
    fn rejects_oversized_interactive_window() {
        let err = TimeWindow::try_new(0, MAX_INTERACTIVE_WINDOW_SECONDS + 1).unwrap_err();
        assert_eq!(
            err,
            AnalyticsError::WindowTooLarge {
                actual_seconds: MAX_INTERACTIVE_WINDOW_SECONDS + 1,
                max_seconds: MAX_INTERACTIVE_WINDOW_SECONDS,
            }
        );
    }

    #[test]
    fn rejects_raw_axis_syntax_before_rendering() {
        let err = AuditLogSearchRequest::try_new(tid(), "identity;drop", window(), None, None)
            .unwrap_err();
        assert!(matches!(
            err,
            AnalyticsError::Olap(KernelError::ColumnNameInvalidChar { .. })
        ));
    }

    #[test]
    fn rejects_zero_page_size() {
        let err = WorkflowExecutionDashboardRequest::try_new(tid(), window(), Some(0)).unwrap_err();
        assert_eq!(
            err,
            AnalyticsError::PageSizeTooLarge {
                actual: 0,
                max: MAX_PAGE_SIZE,
            }
        );
    }
}
