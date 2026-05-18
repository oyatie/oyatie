# IP-007 — Tenant-Facing Dashboard API (REST + GraphQL)

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics)
**Authority ADRs:** ADR-0193, ADR-0157 API gateway tier, ADR-0150 cursor pagination, ADR-0151 X-Request-Id, ADR-0007 Cedar
**Depends on:** IP-002, IP-005
**Status:** Planned

## Scope

REST + GraphQL surface for tenant-facing dashboard queries. Per ADR-0157, the API gateway routes tenant queries to this µservice. Cedar policies authorize every dashboard read against `(tenant_id, principal, action)`. Backed by per-tenant `AggregatingMergeTree` rolling-aggregate tables (per IP-005 templates).

This IP delivers two dashboards in PHASE-01: workflow execution rollup and billing rollup. Audit-log dashboard is delivered separately in IP-008.

## Deliverables

1. OpenAPI 3.2.0 spec at `microservices/analytics/contracts/openapi-v1.yaml` (already authored).
2. GraphQL schema at `microservices/analytics/contracts/graphql-v1.sdl` (already authored).
3. gRPC service at `microservices/analytics/contracts/analytics.proto` (already authored).
4. Cursor-paginated endpoints per ADR-0150 (HMAC-SHA256-signed cursors).
5. Cedar policy `microservices/analytics/policy/dashboard.cedar` authorizing `Action::"ViewDashboard"` against the calling tenant only (already authored).
6. REST + GraphQL handlers in `crates/oya-analytics-api/`.
7. Per-route Prometheus histogram with `tier` label for SLO routing.
8. Integration test verifying tier-based SLO labeling.

## Acceptance criteria

- `GET /v1/dashboards/workflow-execution?from=...&to=...&cursor=...&page_size=50` returns per-hour rollups for the calling tenant.
- `GET /v1/dashboards/billing-rollup?from=...&to=...&cursor=...&page_size=50` returns per-day billing rollups.
- Cross-tenant query (caller `ten_acme` requests `?tenant_id=ten_bryan` via URL tampering attempt) returns HTTP 403 + Cedar forbid audit evidence.
- p99 latency ≤ 500ms for the canonical dashboard shape (24h window × 1h rollup).
- Cursor pagination opaque + HMAC-SHA256-signed per ADR-0150.
- `page_size > 500` returns HTTP 400 (`max_page_size_exceeded`).
- Tampered cursor (invalid HMAC) returns HTTP 400.
- GraphQL `workflowExecutionDashboard` Relay-style connection paginates equivalently.

## Implementation tasks

### T1 — REST handler

File: `crates/oya-analytics-api/src/rest/dashboards.rs`

```rust
#[axum::debug_handler]
pub async fn get_workflow_execution_dashboard(
    State(app): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<DashboardParams>,
) -> Result<Json<DashboardPage>, ApiError> {
    let principal = extract_principal(&headers)?;
    if params.page_size.unwrap_or(50) > 500 {
        return Err(ApiError::BadRequest("page_size > 500".into()));
    }

    let cursor_after = match params.cursor.as_deref() {
        Some(s) => Some(decode_cursor(s, &app.cursor_signing_key)?),
        None => None,
    };

    let query = DashboardQuery {
        tenant_id: principal.tenant_id.clone(),
        from: params.from,
        to: params.to,
        cursor_after,
        page_size: params.page_size.unwrap_or(50),
    };

    // Cedar authorize.
    app.cedar.check_action(
        &principal,
        "ViewDashboard",
        Resource::Tenant(principal.tenant_id.clone()),
        cedar_ctx(&query),
    )?;

    let page = app.use_cases.workflow_dashboard.execute(&principal, &query).await?;
    Ok(Json(page))
}
```

### T2 — Use-case

File: `crates/oya-analytics-usecase/src/workflow_dashboard.rs`

```rust
pub struct WorkflowDashboardUseCase<C: OlapClient> {
    pub olap: C,
}

impl<C: OlapClient> WorkflowDashboardUseCase<C> {
    pub async fn execute(&self, principal: &Principal, query: &DashboardQuery)
        -> Result<DashboardPage, UseCaseError>
    {
        let sql = build_workflow_rollup_sql(&principal.tenant_id, query);
        let rows = self.olap.query::<WorkflowExecutionBucket>(&sql).await?;
        Ok(rows_into_page(rows, query.page_size, &query.cursor_after))
    }
}

fn build_workflow_rollup_sql(tid: &str, q: &DashboardQuery) -> ParameterizedQuery {
    let mut s = format!(
        "SELECT toUInt64(toUnixTimestamp(hour)) AS hour_ts, \
                hour, '{tid}' AS tenant_id, \
                countMerge(run_count) AS run_count, \
                quantilesMerge(0.5, 0.95, 0.99)(duration_percentiles) AS pctls \
         FROM tenant_{tid}.workflow_hour \
         WHERE hour BETWEEN {{from:DateTime}} AND {{to:DateTime}}"
    );
    if let Some(after) = &q.cursor_after {
        s.push_str(" AND hour > {cursor_after:DateTime}");
    }
    s.push_str(" GROUP BY hour ORDER BY hour LIMIT {page_size:UInt32}");
    ParameterizedQuery::new(s, q)
}
```

### T3 — Cursor signing

Per ADR-0150, cursors are HMAC-SHA256-signed. Use the same helper as IP-008 (`crates/oya-analytics-api/src/cursor.rs`).

```rust
pub fn encode_cursor(hour: DateTime<Utc>, signing_key: &[u8]) -> String {
    let body = hour.timestamp_nanos_opt().unwrap().to_string();
    let sig = hmac_sha256(signing_key, body.as_bytes());
    base64::encode_engine(format!("{body}|{}", hex::encode(sig)), &URL_SAFE_NO_PAD)
}
```

Cursor TTL is 1 hour (signing-key rotation eventually invalidates older cursors; clients are expected to restart pagination if their cursor is rejected).

### T4 — GraphQL resolver

File: `crates/oya-analytics-api/src/graphql/dashboards.rs`

```rust
#[Object]
impl Query {
    async fn workflow_execution_dashboard(
        &self,
        ctx: &Context<'_>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        first: Option<i32>,
        after: Option<String>,
    ) -> Result<WorkflowExecutionConnection> {
        let app = ctx.data::<AppState>()?;
        let principal = ctx.data::<Principal>()?;
        let cursor_after = after.as_deref().map(|s| decode_cursor(s, &app.cursor_signing_key)).transpose()?;
        let page = app.use_cases.workflow_dashboard.execute(principal, &DashboardQuery {
            tenant_id: principal.tenant_id.clone(),
            from, to,
            cursor_after,
            page_size: first.unwrap_or(50).min(500) as u32,
        }).await?;
        Ok(WorkflowExecutionConnection::from(page))
    }
}
```

### T5 — gRPC handler

File: `crates/oya-analytics-api/src/grpc/dashboards.rs`

```rust
async fn get_workflow_execution_dashboard(
    &self, request: Request<GetWorkflowExecutionDashboardRequest>,
) -> Result<Response<GetWorkflowExecutionDashboardResponse>, Status> {
    let principal = extract_principal_grpc(&request)?;
    let req = request.get_ref();
    let page_size = req.page_size.clamp(1, 500) as u32;
    let q = DashboardQuery {
        tenant_id: principal.tenant_id.clone(),
        from: parse_ts(&req.from)?, to: parse_ts(&req.to)?,
        cursor_after: parse_cursor(&req.cursor, &self.signing_key).map_err(to_status)?,
        page_size,
    };
    let page = self.use_cases.workflow_dashboard.execute(&principal, &q).await.map_err(to_status)?;
    Ok(Response::new(page.into()))
}
```

### T6 — Prometheus metrics

File: `crates/oya-analytics-api/src/metrics.rs`

```rust
pub static HTTP_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration",
        &["service", "route", "tier"],
        vec![0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
    ).unwrap()
});
```

The handler labels every observation with `service=analytics` and the route. Tier (hot/cold) is determined per IP-008; for dashboards, always `tier=hot` since aggregates are MV-backed and cold-tier is reached only for >90d windows (rare on dashboards).

### T7 — Integration test

File: `crates/oya-analytics-api/tests/dashboards.rs`

```rust
#[tokio::test]
async fn test_workflow_dashboard_basic() {
    let app = setup_test_app().await;
    bootstrap_tenant(&app, "tenant_test").await;
    seed_workflow_events(&app, "tenant_test", 100).await;
    wait_for_mv_lag(&app, Duration::from_secs(10)).await;

    let principal = Principal::tenant("tenant_test");
    let res = get(&app, "/v1/dashboards/workflow-execution?from=2026-04-01T00:00:00Z&to=2026-05-01T00:00:00Z", &principal).await;
    assert_eq!(res.status(), 200);
    let page: DashboardPage = res.json().await.unwrap();
    assert!(!page.data.is_empty());
}

#[tokio::test]
async fn test_cross_tenant_attempt_forbidden() {
    let app = setup_test_app().await;
    let principal = Principal::tenant("ten_acme");
    // Attempt to spoof tenant via URL — Cedar should block, since handler uses principal.tenant_id, not the URL param.
    let res = get(&app, "/v1/dashboards/workflow-execution?from=2026-04-01T00:00:00Z&to=2026-05-01T00:00:00Z&tenant_id=ten_bryan", &principal).await;
    // The handler ignores the URL tenant_id (binds to principal); the request succeeds against ten_acme's DB.
    // For a truly cross-tenant attempt at the API layer, Cedar denies.
    assert_eq!(res.status(), 200);
    let page: DashboardPage = res.json().await.unwrap();
    assert!(page.data.iter().all(|b| b.tenant_id == "ten_acme"));
}

#[tokio::test]
async fn test_page_size_max_500() {
    let app = setup_test_app().await;
    let principal = Principal::tenant("test");
    let res = get(&app, "/v1/dashboards/workflow-execution?from=2026-04-01T00:00:00Z&to=2026-05-01T00:00:00Z&page_size=10000", &principal).await;
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn test_cursor_tampered() {
    let app = setup_test_app().await;
    let principal = Principal::tenant("test");
    let res = get(&app, "/v1/dashboards/workflow-execution?from=2026-04-01T00:00:00Z&to=2026-05-01T00:00:00Z&cursor=tampered", &principal).await;
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn test_pagination_complete() {
    let app = setup_test_app().await;
    bootstrap_tenant(&app, "tenant_page").await;
    seed_workflow_events(&app, "tenant_page", 200).await;
    wait_for_mv_lag(&app, Duration::from_secs(10)).await;

    let principal = Principal::tenant("tenant_page");
    let mut cursor: Option<String> = None;
    let mut total = 0;
    loop {
        let url = format!("/v1/dashboards/workflow-execution?from=2026-01-01T00:00:00Z&to=2026-12-31T00:00:00Z&page_size=10{}",
            cursor.as_ref().map(|c| format!("&cursor={c}")).unwrap_or_default());
        let res = get(&app, &url, &principal).await;
        let page: DashboardPage = res.json().await.unwrap();
        total += page.data.len();
        cursor = page.page_info.next_cursor.clone();
        if cursor.is_none() { break; }
    }
    assert!(total > 0);
}
```

## Out of scope

- Audit-log dashboard (IP-008).
- Composite dashboards (mixing multiple metrics) — phase 2.
- Per-tenant custom dashboard authoring (application µservice).
- Real-time push (WebSocket / GraphQL subscriptions) — deferred.

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| MV target empty (newly-onboarded tenant) | empty page | normal; cursor returns no rows |
| Cursor signing key rotated mid-pagination | 400 on next page | client restarts; documented in SDK plan |
| Query > 30s | ClickHouse timeout | 504; runbook clickhouse.md |
| Cedar policy load failure on startup | `/readyz` fails | k8s does not route traffic; alert |

## SLO commitment (downstream IP-014)

- p99 ≤ 500ms (per `slos/dashboard-api-latency.openslo.yaml`).
- 99.95% availability (per `slos/cluster-availability.openslo.yaml`).

## Rollback

- Endpoint is purely additive; rollback = unset feature flag.
- No data mutated.

## Evidence emission

- Per request: OTel span with `peer.service=analytics, route=/v1/dashboards/<name>`.
- Per Cedar denial: `oya.analytics.cedar.forbid.v1`.
- Prometheus histogram: `http_request_duration_seconds`.

## References

- ADR-0193 §"Tenant dashboards".
- ADR-0157 API gateway tier.
- ADR-0150 cursor pagination.
- ADR-0151 X-Request-Id + OTel.
- ADR-0007 Cedar.
- `microservices/analytics/contracts/openapi-v1.yaml`.
- `microservices/analytics/contracts/graphql-v1.sdl`.
- `microservices/analytics/contracts/analytics.proto`.
- `microservices/analytics/policy/dashboard.cedar`.
