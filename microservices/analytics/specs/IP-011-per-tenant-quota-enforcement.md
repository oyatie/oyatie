# IP-011 — Per-Tenant Quota Enforcement

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics + council-tenancy)
**Authority ADRs:** ADR-0155 quotas, ADR-0193, ADR-0007 Cedar, ADR-AN-004-query-budget-tier
**Depends on:** IP-002
**Status:** Planned

## Scope

Project the ADR-0155 per-tenant resource quota model into ClickHouse `QUOTA` objects. Per-tenant tier (Trial / Starter / Growth / Enterprise per `oya-tenancy-kernel::B2bTenantTier`) determines the QUOTA limits. The IP-002 bootstrap controller applies and re-applies the QUOTA on tier change. The handler layer surfaces quota-exceeded as HTTP 429 + `Retry-After` + audit-chain event.

The canonical tier matrix is owned by ADR-AN-004 (a service-scoped ADR projecting the fleet-wide ADR-0155 into specific values).

## Deliverables

1. Per-tier QUOTA DDL template at `microservices/analytics/iac/clickhouse/quota-templates/<tier>.sql`.
2. Adapter error mapping: ClickHouse error 201 → `KernelError::AdapterError("quota_exceeded")`.
3. API handler: 429 + `Retry-After: <seconds>` + audit-chain event.
4. Tier upgrade reconciliation in IP-002 controller.
5. Per-tenant quota upgrade-and-downgrade idempotency.
6. Prometheus metric `oya_analytics_quota_exceeded_total{tenant_id, kind}`.
7. Integration test verifying each tier's enforcement boundary.

## Acceptance criteria

- Trial tenant exceeding 100 queries/hr gets HTTP 429 on the 101st query.
- Trial tenant exceeding 10M read_rows/hr gets HTTP 429 on the read query that crosses the boundary.
- Tier upgrade (Starter → Growth) reflects in QUOTA within 30s of `tenant.tier_changed` event.
- Tier downgrade (Growth → Starter) reflects within 30s; in-flight queries complete under the old quota.
- Quota-exceeded event lands in audit-chain with `(tenant_id, quota_kind, observed, limit)`.
- Prometheus metric `oya_analytics_quota_exceeded_total` increments per 429.

## Quota matrix (canonical per ADR-AN-004)

| Tier | max queries / hr | max read_rows / hr | max insert_rows / hr | max concurrent | max execution time |
|---|---|---|---|---|---|
| Trial | 100 | 10 M | 1 M | 4 | 30 s |
| Starter | 1,000 | 1 B | 100 M | 16 | 60 s |
| Growth | 10,000 | 10 B | 1 B | 32 | 120 s |
| Enterprise | 100,000 | 1 T (capped) | 100 B (capped) | 64 | 300 s |

## Implementation tasks

### T1 — QUOTA DDL templates

File: `microservices/analytics/iac/clickhouse/quota-templates/trial.sql`

```sql
CREATE QUOTA IF NOT EXISTS quota_tenant_${tid}
ON CLUSTER analytics-clickhouse-1
KEYED BY user_name
FOR INTERVAL 1 HOUR
  MAX queries = 100,
      read_rows = 10000000,
      written_rows = 1000000
TO tenant_${tid}_reader, tenant_${tid}_writer;

ALTER USER tenant_${tid}_reader ON CLUSTER analytics-clickhouse-1
SETTINGS
  max_concurrent_queries_for_user = 4,
  max_execution_time = 30;

ALTER USER tenant_${tid}_writer ON CLUSTER analytics-clickhouse-1
SETTINGS
  max_concurrent_queries_for_user = 4,
  max_execution_time = 30;
```

Sibling templates: `starter.sql`, `growth.sql`, `enterprise.sql` (numeric substitutions per the matrix).

### T2 — Adapter error mapping

In `crates/oya-shared-olap-clickhouse-adapter/src/error.rs`:

```rust
impl From<clickhouse::Error> for KernelError {
    fn from(e: clickhouse::Error) -> Self {
        // ClickHouse error code surfaced via the "code: <n>" string.
        if let Some(code) = extract_error_code(&e.to_string()) {
            match code {
                201 => return KernelError::AdapterError(format!("quota_exceeded: {}", e)),
                81 => return KernelError::AdapterError("database not found".into()),
                60 => return KernelError::AdapterError("table not found".into()),
                _ => {}
            }
        }
        KernelError::AdapterError(e.to_string())
    }
}
```

### T3 — API 429 handling

In `crates/oya-analytics-api/src/error.rs`:

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::QuotaExceeded(QuotaExceeded { kind, observed, limit, retry_after_seconds }) => {
                let mut headers = HeaderMap::new();
                headers.insert("Retry-After", retry_after_seconds.to_string().parse().unwrap());
                let body = Json(ErrorBody {
                    code: "quota_exceeded",
                    message: format!("{}: {}/{}", kind, observed, limit),
                    request_id: current_request_id(),
                });
                (StatusCode::TOO_MANY_REQUESTS, headers, body).into_response()
            }
            // ... other mappings
        }
    }
}
```

### T4 — Tier upgrade reconciliation in IP-002 controller

In `crates/oya-analytics-tenant-bootstrap-app/src/reconcile.rs`:

```rust
async fn handle_tier_changed(event: &TenantTierChanged, deps: &Deps) -> Result<()> {
    let template = quota_template_for_tier(event.new_tier);
    let rendered = template.replace("${tid}", &event.tenant_id);
    deps.olap.exec_ddl(&rendered).await?;
    // Idempotent: CREATE QUOTA IF NOT EXISTS; ALTER USER is overwrite-safe.
    deps.audit_chain.emit("oya.analytics.tenant.quota_applied.v1", json!({
        "tenant_id": event.tenant_id, "old_tier": event.old_tier, "new_tier": event.new_tier
    })).await?;
    Ok(())
}
```

### T5 — Audit-chain emission on quota exceeded

In `crates/oya-analytics-api/src/quota_audit.rs`:

```rust
pub async fn on_quota_exceeded(audit_chain: &AuditChainPublisher, tid: &str, kind: &str, observed: u64, limit: u64) {
    let _ = audit_chain.emit("oya.analytics.quota_exceeded.v1", json!({
        "tenant_id": tid,
        "quota_kind": kind,          // "queries_per_hour", "read_rows_per_hour", "insert_rows_per_hour"
        "observed": observed,
        "limit": limit,
        "ts": Utc::now(),
    })).await;
}
```

### T6 — Prometheus metric

In `crates/oya-analytics-api/src/metrics.rs`:

```rust
pub static QUOTA_EXCEEDED_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "oya_analytics_quota_exceeded_total",
        "Count of 429 responses due to quota exceeded",
        &["tenant_id", "quota_kind"]
    ).unwrap()
});
```

The metric increments at the same site as the audit-chain emission.

### T7 — Tier downgrade in-flight semantics

ClickHouse `QUOTA` is checked at query-start; in-flight queries that started under the old quota complete. New queries observe the new quota. This is the desired semantic; documented in the runbook.

If a downgrade should cancel in-flight queries (rare; only on enforcement breach), the operator uses `KILL QUERY WHERE user = 'tenant_${tid}_reader'` after applying the new quota.

### T8 — Integration test

File: `crates/oya-analytics-api/tests/quota.rs`

```rust
#[tokio::test]
async fn test_trial_tier_blocked_at_101_queries() {
    let app = setup_test_app().await;
    bootstrap_tenant_at_tier(&app, "tenant_trial", "Trial").await;

    let principal = Principal::tenant("tenant_trial");
    for i in 0..100 {
        let res = get_dashboard(&app, &principal).await;
        assert_eq!(res.status(), 200, "query {i} should succeed");
    }
    let res = get_dashboard(&app, &principal).await;
    assert_eq!(res.status(), 429);
    assert!(res.headers().get("retry-after").is_some());
}

#[tokio::test]
async fn test_tier_upgrade_reflected_in_30s() {
    let app = setup_test_app().await;
    bootstrap_tenant_at_tier(&app, "test_upgrade", "Trial").await;
    // Trial limit = 100. Use up the budget.
    use_quota_budget(&app, "test_upgrade", 100).await;

    // Upgrade to Starter.
    emit_tier_changed(&app, "test_upgrade", "Trial", "Starter").await;
    tokio::time::sleep(Duration::from_secs(35)).await;

    // Should be allowed again.
    let res = get_dashboard(&app, &Principal::tenant("test_upgrade")).await;
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn test_quota_exceeded_emits_audit_event() {
    let app = setup_test_app().await;
    bootstrap_tenant_at_tier(&app, "tenant_audit", "Trial").await;
    use_quota_budget(&app, "tenant_audit", 101).await;
    let events = audit_chain_events_for_tenant(&app, "tenant_audit", "oya.analytics.quota_exceeded.v1").await;
    assert!(!events.is_empty());
    assert_eq!(events[0].data["quota_kind"], "queries_per_hour");
}
```

## Out of scope

- Sub-hour granularity (per ADR-AN-004 §"Alternatives considered" — rejected).
- Per-µservice quota (we quota per-tenant, not per-µservice).
- Adaptive quota (per-tenant burst allowance) — deferred to phase 2.

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| QUOTA DDL fails on apply | controller error log | retry; alert if persistent (likely Keeper quorum loss) |
| Tier change race (two changes in <30s) | controller observes both events | last-write-wins; cursor ensures correctness |
| Quota threshold breached but ClickHouse fails to enforce | metric divergence; reconciliation lane | alert; investigate ClickHouse |
| Tenant tier event lost | controller cursor lag | re-publish; controller reconciles |

## SLO commitment (downstream IP-014)

- Tier upgrade reflected within 30s: 99% (per `slos/tenant-bootstrap-latency.openslo.yaml` — same controller).
- 429 emission accurate vs actual quota state: 99.99% (reconciliation lane verifies).

## Rollback

- Per-tier templates are stored as files; rollback = revert the template change.
- Tier change events are persisted; replaying them re-reconciles QUOTA.

## Evidence emission

- Per QUOTA application: `oya.analytics.tenant.quota_applied.v1`.
- Per quota-exceeded: `oya.analytics.quota_exceeded.v1`.
- Prometheus: `oya_analytics_quota_exceeded_total{tenant_id, quota_kind}`.

## References

- ADR-0155 per-tenant resource quotas.
- ADR-0193 §"Multi-tenancy isolation".
- ADR-0007 Cedar.
- ADR-AN-004-query-budget-tier (canonical tier matrix).
- `microservices/analytics/iac/clickhouse/quota-templates/`.
