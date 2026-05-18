# IP-009 — Billing Rollup Pipeline

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics + council-cloud)
**Authority ADRs:** ADR-0193, ADR-0195, ADR-0155 quotas, ADR-0153 outbox, ADR-0003 audit chain
**Depends on:** IP-004, IP-005
**Status:** Planned

## Scope

Daily + monthly per-tenant per-resource rollup tables that drive the billing µservice's invoice generation. Source: per-µservice `oya.usage.counters` outbox events. Targets: `AggregatingMergeTree` daily and monthly tables. Month-end finalization produces the canonical billable-usage record (audit-chain-signed).

The pipeline has three stages:

1. **Source ingest** — Kafka engine table `oya_usage_counters_kafka_source` consumes from Pulsar.
2. **Daily MV** — `mv_day_billing_per_resource` (template at `iac/clickhouse/mv-templates/mv-day-billing-per-resource.sql`).
3. **Monthly MV** — `mv_month_billing_per_resource` chained from daily (chain depth 2, max per ADR-AN-005).

## Deliverables

1. Source `Kafka` engine table consuming `oya.usage.counters` from Pulsar (DDL committed at `iac/clickhouse/usage-source.sql`).
2. MV `mv_day_billing_per_resource` per tenant — rendered at tenant onboard by IP-002 controller.
3. MV `mv_month_billing_per_resource` per tenant — chained from daily.
4. Month-end finalization job (Kubernetes CronJob) emitting `oya.analytics.tenant.billable_usage.v1` per `(tenant_id, resource_type)` on month close.
5. Reconciliation lane — daily MV sum equals monthly MV value (±0.01%).
6. Cross-cell aggregation — per-cell rollups + scheduled cross-cell aggregation job for global tenants.
7. Cedar policy authorizing billing µservice to subscribe to `tenant.billable_usage` events.
8. Integration test.

## Acceptance criteria

- A `usage.counter.incremented` event for tenant `ten_acme`, resource `workflow_run`, increment `1` lands in `mv_day_billing_per_resource` (target table `tenant_ten_acme.billing_day`) within 5s p99.
- Month-end finalization emits one `tenant.billable_usage` event per `(tenant_id, resource_type)` for the closed month.
- Reconciliation: sum of daily rollups equals monthly rollup with drift < 0.01%.
- Audit-chain entry per finalized billable usage with cosign signature.
- Out-of-order events (late-arriving with `event_time < ingest_time - 2h`) attributed to the event_time bucket, not the ingest_time bucket.
- Month-end finalization job completes within 60 minutes after month close (UTC).

## Implementation tasks

### T1 — Kafka engine source table

File: `microservices/analytics/iac/clickhouse/usage-source.sql`

```sql
CREATE TABLE IF NOT EXISTS oya_usage_counters_kafka_source
ON CLUSTER analytics-clickhouse-1
(
    event_id UUID,
    event_type String,
    tenant_id String,
    resource_type String,
    increment Int64,
    emitted_at DateTime64(3, 'UTC')
)
ENGINE = Kafka
SETTINGS
    kafka_broker_list = 'pulsar-kafka.observability.svc.cluster.local:9092',
    kafka_topic_list = 'oya.usage.counters',
    kafka_group_name = 'analytics-clickhouse-billing',
    kafka_format = 'JSONEachRow',
    kafka_num_consumers = 6,
    kafka_max_block_size = 65536;
```

### T2 — Daily MV (per-tenant render)

The template lives at `microservices/analytics/iac/clickhouse/mv-templates/mv-day-billing-per-resource.sql` (already authored as part of the IaC scaffolding).

The IP-002 bootstrap controller renders this template per-tenant. The rendering substitutes `${tid}` with the tenant id.

### T3 — Monthly MV (chained)

File: `microservices/analytics/iac/clickhouse/mv-templates/mv-month-billing-per-resource.sql` (already authored).

This is chain depth 2 per ADR-AN-005 (maximum allowed). The monthly MV reads from `tenant_${tid}.billing_day` and writes to `tenant_${tid}.billing_month`.

### T4 — Month-end finalization job

File: `crates/oya-analytics-billing-finalizer-app/` (new crate).

Runs as a Kubernetes CronJob at `1 0 1 * *` (1 minute past midnight, 1st of each month UTC).

```rust
// crates/oya-analytics-billing-finalizer-app/src/main.rs (sketch)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::load()?;
    let olap = ClickHouseOlapClient::connect(&cfg.clickhouse).await?;
    let audit_chain = AuditChainPublisher::connect(&cfg.audit_chain).await?;
    let cosign = CosignSigner::from_openbao(&cfg.cosign_ref).await?;

    let last_month = previous_month();  // (year, month)
    let tenants = olap.list_tenant_databases().await?;

    for tenant_id in tenants {
        let rows = olap.query_billable_usage(&tenant_id, last_month).await?;
        for row in rows {
            let event = build_billable_usage_event(&tenant_id, &row);
            let signed = cosign.sign_event(&event).await?;
            audit_chain.emit("oya.analytics.tenant.billable_usage.v1", &signed).await?;
        }
    }

    // Emit reconciliation evidence.
    emit_reconciliation_report(&olap, last_month).await?;
    Ok(())
}
```

CronJob spec:

```yaml
apiVersion: batch/v1
kind: CronJob
metadata: { name: billing-finalizer, namespace: analytics }
spec:
  schedule: "1 0 1 * *"
  jobTemplate:
    spec:
      activeDeadlineSeconds: 3600  # 60 min budget
      template:
        spec:
          serviceAccountName: oya-analytics-billing-finalizer
          containers:
            - name: finalizer
              image: ghcr.io/oyatie/analytics-billing-finalizer:<sha>
          restartPolicy: OnFailure
```

### T5 — Reconciliation lane

A daily (not just monthly) reconciliation:

```sql
-- Daily total via daily MV.
WITH daily AS (
    SELECT tenant_id, resource_type, sumMerge(count_state) AS daily_total
    FROM tenant_${tid}.billing_day
    WHERE day = yesterday()
    GROUP BY tenant_id, resource_type
),
-- Daily total recomputed from source (the truth).
source AS (
    SELECT tenant_id, resource_type, sum(increment) AS source_total
    FROM oya_usage_counters_kafka_source  -- NOT this; instead use a snapshot
    WHERE toDate(emitted_at) = yesterday() AND tenant_id = '${tid}'
    GROUP BY tenant_id, resource_type
)
SELECT
    daily.tenant_id, daily.resource_type,
    daily_total, source_total,
    abs(daily_total - source_total) / nullIf(source_total, 0) AS drift_pct
FROM daily JOIN source USING (tenant_id, resource_type);
```

The reconciliation result emits to Prometheus as the SLI for `slos/billing-reconciliation.openslo.yaml` (already authored).

Drift > 0.01% triggers an alert; drift > 1% pages immediately.

### T6 — Out-of-order handling

The MV's `GROUP BY toDate(emitted_at)` (not `toDate(now())`) ensures attribution by event time, not ingest time. Late-arriving events (e.g., emitted last week, ingested today) update the correct day's bucket via `AggregatingMergeTree`'s eventual-consistent merge.

The acceptance criterion is verified by the integration test (T8).

### T7 — Cross-cell aggregation for global tenants

For tenants with `ResidencyClass::Global`, billing finalization aggregates across cells via the cross-cell aggregation job (per IP-010):

```sql
INSERT INTO fleet_internal.global_billable_usage
SELECT tenant_id, resource_type, sumMerge(count_state) AS total
FROM cluster('cross-cell-global', tenant_*.billing_month)
WHERE month = previous_month()
GROUP BY tenant_id, resource_type;
```

Tenants with `ResidencyClass::StrictKR / StrictEU / etc.` do not federate; their billing is per-cell.

### T8 — Integration test

File: `crates/oya-analytics-billing-finalizer-app/tests/integration.rs`

```rust
#[tokio::test]
async fn test_daily_to_monthly_chain() {
    let app = setup_test_app().await;
    bootstrap_tenant(&app, "test_tenant").await;
    emit_usage_event(&app, "test_tenant", "workflow_run", 100, "2026-04-01T00:00:00Z").await;
    emit_usage_event(&app, "test_tenant", "workflow_run", 200, "2026-04-15T00:00:00Z").await;
    emit_usage_event(&app, "test_tenant", "workflow_run", 50, "2026-04-30T00:00:00Z").await;

    wait_for_mv_lag(&app, Duration::from_secs(10)).await;

    let daily_total = query_daily(&app, "test_tenant", "workflow_run", "2026-04").await;
    assert_eq!(daily_total, 350);

    let monthly = query_monthly(&app, "test_tenant", "workflow_run", "2026-04").await;
    assert_eq!(monthly, 350);
}

#[tokio::test]
async fn test_out_of_order_event_attributed_to_event_time() {
    let app = setup_test_app().await;
    bootstrap_tenant(&app, "test_tenant").await;
    // Emit an event with old event_time but current ingest_time.
    let now = Utc::now();
    let event_time = now - Duration::days(7);
    emit_usage_event_at(&app, "test_tenant", "workflow_run", 999, event_time).await;
    wait_for_mv_lag(&app, Duration::from_secs(10)).await;

    let total = query_daily(&app, "test_tenant", "workflow_run", &event_time.date_naive().to_string()).await;
    assert_eq!(total, 999);

    let today_total = query_daily(&app, "test_tenant", "workflow_run", &now.date_naive().to_string()).await;
    assert_eq!(today_total, 0);
}

#[tokio::test]
async fn test_finalization_emits_billable_usage() {
    let app = setup_test_app().await;
    seed_full_month(&app, "test_tenant", "workflow_run", 12345).await;
    run_finalizer(&app, previous_month()).await;
    let events = audit_chain_events(&app, "oya.analytics.tenant.billable_usage.v1").await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data["count"], 12345);
}
```

## Out of scope

- Invoice generation (billing µservice consumes the `billable_usage` event).
- Tenant-facing billing UI (application µservice).
- Per-second / per-API-call usage (billing is per-counter; counters are aggregated by source µservice).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| Reconciliation drift > 0.01% | SLO burn alert | runbook ingest-lag-burn + investigate MV |
| Finalization > 60min | activeDeadlineSeconds | next-run; alert |
| Cross-cell aggregation partial (some cells offline) | aggregator job warning | retry; manual reconciliation if persistent |
| Late event arrives > 30 days late | event dropped (out of TTL hot window) | mv reads from cold tier; documented limit |

## SLO commitment (downstream IP-014)

- Ingest lag ≤ 5s p99 (per `slos/clickhouse-ingest-lag.openslo.yaml`).
- Month-end finalization within 60min (per `slos/billing-reconciliation.openslo.yaml`).
- Reconciliation drift < 0.01% across all tenants in test.

## Rollback

- Per ADR-0159 feature-flag-gated.
- MV chain can be paused via `DETACH MATERIALIZED VIEW`; ingest accumulates in Kafka source until reattach.

## Evidence emission

- Per finalization: `oya.analytics.tenant.billable_usage.v1` per `(tenant_id, resource_type)`.
- Per reconciliation check: emit `oya_analytics_billing_reconcile_within_tolerance_total` Prometheus metric.
- Per cross-cell aggregation: `oya.analytics.fleet.global_billable_aggregated.v1`.

## References

- ADR-0193 §"Materialized Views".
- ADR-0195 stream-processing default tier.
- ADR-0155 per-tenant resource quotas (which the usage counters mirror).
- ADR-0003 audit chain.
- ADR-AN-005-materialized-view-cadence.
- `microservices/analytics/iac/clickhouse/mv-templates/mv-day-billing-per-resource.sql`.
- `microservices/analytics/iac/clickhouse/mv-templates/mv-month-billing-per-resource.sql`.
- `microservices/analytics/slos/billing-reconciliation.openslo.yaml`.
