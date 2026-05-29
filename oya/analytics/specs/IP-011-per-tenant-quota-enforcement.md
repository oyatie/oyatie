# IP-011 — Per-Tenant Quota Enforcement

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics + council-tenancy)
**Authority ADRs:** ADR-0155 quotas, ADR-0193, ADR-0007 Cedar, ADR-AN-004-query-budget-tenant-class
**Depends on:** IP-002
**Status:** Planned

## Scope

Project the ADR-0155 per-tenant resource quota model into ClickHouse `QUOTA` objects. Tenant_class (`demo_trial` / `paid`) determines the QUOTA limits. The IP-002 bootstrap controller applies and re-applies the QUOTA on tenant_class change. The handler layer surfaces quota-exceeded as HTTP 429 + `Retry-After` + audit-chain event.

The canonical tenant_class matrix is owned by ADR-AN-004 (a service-scoped ADR projecting the fleet-wide ADR-0155 into specific values).

## Deliverables

1. Per-tenant_class QUOTA DDL template at `microservices/analytics/iac/clickhouse/quota-templates/<tenant_class>.sql`.
2. Adapter error mapping: ClickHouse error 201 → `KernelError::AdapterError("quota_exceeded")`.
3. API handler: 429 + `Retry-After: <seconds>` + audit-chain event.
4. Tenant_class conversion reconciliation in IP-002 controller.
5. Per-tenant quota conversion idempotency.
6. Prometheus metric `oya_analytics_quota_exceeded_total{tenant_id, kind}`.
7. Integration test verifying each tenant_class enforcement boundary.

## Acceptance criteria

- demo_trial tenant exceeding 100 queries/hr gets HTTP 429 on the 101st query.
- demo_trial tenant exceeding 10M read_rows/hr gets HTTP 429 on the read query that crosses the boundary.
- Tenant_class conversion (demo_trial → paid) reflects in QUOTA within 30s of `tenant.tenant_class_changed` event.
- Paid billing_components review reflects within 30s; in-flight queries complete under the old quota.
- Quota-exceeded event lands in audit-chain with `(tenant_id, quota_kind, observed, limit)`.
- Prometheus metric `oya_analytics_quota_exceeded_total` increments per 429.

## Quota matrix (canonical per ADR-AN-004)

| tenant_class | max queries / hr | max read_rows / hr | max insert_rows / hr | max concurrent | max execution time |
|---|---|---|---|---|---|
| demo_trial | 100 | 10 M | 1 M | 4 | 30 s |
| paid | 10,000 | 10 B | 1 B | 32 | 120 s |
| paid_contract_overlay | 100,000 | 1 T (capped) | 100 B (capped) | 64 | 300 s |

## Implementation tasks

### T1 — QUOTA DDL templates

File: `microservices/analytics/iac/clickhouse/quota-templates/demo_trial.sql`

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

Sibling templates: `paid.sql`, `paid_contract_overlay.sql` (numeric substitutions per the matrix).

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

### T4 — Tenant_class conversion reconciliation in IP-002 controller

In `crates/oya-analytics-tenant-bootstrap-app/src/reconcile.rs`:

```rust
async fn handle_tenant_class_changed(event: &TenantClassChanged, deps: &Deps) -> Result<()> {
    let template = quota_template_for_tenant_class(event.new_tenant_class);
    let rendered = template.replace("${tid}", &event.tenant_id);
    deps.olap.exec_ddl(&rendered).await?;
    // Idempotent: CREATE QUOTA IF NOT EXISTS; ALTER USER is overwrite-safe.
    deps.audit_chain.emit("oya.analytics.tenant.quota_applied.v1", json!({
        "tenant_id": event.tenant_id, "old_tenant_class": event.old_tenant_class, "new_tenant_class": event.new_tenant_class
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

### T7 — Tenant_class conversion in-flight semantics

ClickHouse `QUOTA` is checked at query-start; in-flight queries that started under the old quota complete. New queries observe the new quota. This is the desired semantic; documented in the runbook.

If a downgrade should cancel in-flight queries (rare; only on enforcement breach), the operator uses `KILL QUERY WHERE user = 'tenant_${tid}_reader'` after applying the new quota.

### T8 — Integration test

File: `crates/oya-analytics-api/tests/quota.rs`

```rust
#[tokio::test]
async fn test_demo_trial_tenant_class_blocked_at_101_queries() {
    let app = setup_test_app().await;
    bootstrap_tenant_at_class(&app, "tenant_demo_trial", "demo_trial").await;

    let principal = Principal::tenant("tenant_demo_trial");
    for i in 0..100 {
        let res = get_dashboard(&app, &principal).await;
        assert_eq!(res.status(), 200, "query {i} should succeed");
    }
    let res = get_dashboard(&app, &principal).await;
    assert_eq!(res.status(), 429);
    assert!(res.headers().get("retry-after").is_some());
}

#[tokio::test]
async fn test_tenant_class_conversion_reflected_in_30s() {
    let app = setup_test_app().await;
    bootstrap_tenant_at_class(&app, "test_conversion", "demo_trial").await;
    // demo_trial limit = 100. Use up the budget.
    use_quota_budget(&app, "test_conversion", 100).await;

    // Convert to paid.
    emit_tenant_class_changed(&app, "test_conversion", "demo_trial", "paid").await;
    tokio::time::sleep(Duration::from_secs(35)).await;

    // Should be allowed again.
    let res = get_dashboard(&app, &Principal::tenant("test_conversion")).await;
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn test_quota_exceeded_emits_audit_event() {
    let app = setup_test_app().await;
    bootstrap_tenant_at_class(&app, "tenant_audit", "demo_trial").await;
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
| Tenant_class change race (two changes in <30s) | controller observes both events | last-write-wins; cursor ensures correctness |
| Quota threshold breached but ClickHouse fails to enforce | metric divergence; reconciliation lane | alert; investigate ClickHouse |
| Tenant_class event lost | controller cursor lag | re-publish; controller reconciles |

## SLO commitment (downstream IP-014)

- Tenant_class conversion reflected within 30s: 99% (per `slos/tenant-bootstrap-latency.openslo.yaml` — same controller).
- 429 emission accurate vs actual quota state: 99.99% (reconciliation lane verifies).

## Rollback

- Per-tenant_class templates are stored as files; rollback = revert the template change.
- Tenant_class change events are persisted; replaying them re-reconciles QUOTA.

## Evidence emission

- Per QUOTA application: `oya.analytics.tenant.quota_applied.v1`.
- Per quota-exceeded: `oya.analytics.quota_exceeded.v1`.
- Prometheus: `oya_analytics_quota_exceeded_total{tenant_id, quota_kind}`.

## References

- ADR-0155 per-tenant resource quotas.
- ADR-0193 §"Multi-tenancy isolation".
- ADR-0007 Cedar.
- ADR-AN-004-query-budget-tenant-class (canonical tenant_class matrix).
- `microservices/analytics/iac/clickhouse/quota-templates/`.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/analytics/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/analytics/specs/IP-011-per-tenant-quota-enforcement.md:233` - ## SLO commitment (downstream IP-014).

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/analytics/specs/IP-011-per-tenant-quota-enforcement.md:133` - ### T5 — Audit-chain emission on quota exceeded; `microservices/analytics/specs/IP-011-per-tenant-quota-enforcement.md:163` - The metric increments at the same site as the audit-chain emission..
