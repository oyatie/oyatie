# IP-005 — Materialized View Canon (Default Stream Tier)

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics)
**Authority ADRs:** ADR-0195 default tier, ADR-0193 MV semantics, ADR-AN-005-materialized-view-cadence
**Depends on:** IP-004
**Status:** Planned

## Scope

Establish the canonical Materialized View patterns for the three Phase-01 dashboard verticals. Per ADR-0195, MV is the default stream-processing primitive; this IP delivers the canonical templates so every downstream dashboard MV inherits the patterns. Per ADR-AN-005, the templates conform to the naming convention (`mv_${cadence}_${entity}_${dimension}`) and chain-depth limit (≤ 2).

## Deliverables

1. MV template — per-tenant per-time-bucket rolling aggregate (`AggregatingMergeTree` target).
2. MV template — top-K ranking per ADR-0195 class A.
3. MV template — percentile rollup using `quantilesState`.
4. MV template — anomaly window with threshold output.
5. MV template — daily and monthly billing rollup (chained).
6. Standards doc reference at `docs/standards/stream-processing-rubric.md` (sibling output of this batch; out of µservice scope).
7. Test harness verifying each template against synthetic data (round-trip aggregate value matches expected).

## Acceptance criteria

- Each MV template ships as a DDL file in `microservices/analytics/iac/clickhouse/mv-templates/`.
- A test harness verifies each template against synthetic data (round-trip aggregate value matches expected).
- Each template is documented with the workload class it targets (per ADR-0195 §"Class A").
- All templates conform to the naming convention per ADR-AN-005.
- Chain depth ≤ 2 (per ADR-AN-005).

## MV templates (5)

### Template 1 — per-tenant rolling aggregate

File: `microservices/analytics/iac/clickhouse/mv-templates/mv-hour-workflow-per-tenant.sql` (already authored).

```sql
CREATE MATERIALIZED VIEW IF NOT EXISTS tenant_${tid}.mv_hour_workflow_per_tenant
ON CLUSTER analytics-clickhouse-1
TO tenant_${tid}.workflow_hour
AS
SELECT
    toStartOfHour(emitted_at) AS hour,
    tenant_id,
    countState() AS run_count,
    sumState(duration_ms) AS total_duration_ms,
    quantilesState(0.5, 0.95, 0.99)(duration_ms) AS duration_percentiles
FROM oya_events_kafka_source
WHERE event_type = 'oya.workflow.executed.v1' AND tenant_id = '${tid}'
GROUP BY hour, tenant_id;
```

Target table:

```sql
CREATE TABLE tenant_${tid}.workflow_hour (
    hour DateTime,
    tenant_id String,
    run_count AggregateFunction(count, UInt64),
    total_duration_ms AggregateFunction(sum, UInt64),
    duration_percentiles AggregateFunction(quantiles(0.5, 0.95, 0.99), UInt32)
)
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMMDD(hour)
ORDER BY (tenant_id, hour)
TTL hour + INTERVAL 90 DAY TO DISK 's3_cold',
    hour + INTERVAL 365 DAY DELETE
SETTINGS storage_policy = 'hot_cold';
```

Workload class: Business KPI (per ADR-AN-001 retention).

### Template 2 — top-K ranking

File: `microservices/analytics/iac/clickhouse/mv-templates/mv-hour-topk-workflow-per-tenant.sql`

```sql
CREATE MATERIALIZED VIEW IF NOT EXISTS tenant_${tid}.mv_hour_topk_workflow_per_tenant
ON CLUSTER analytics-clickhouse-1
TO tenant_${tid}.topk_workflow_hour
AS
SELECT
    toStartOfHour(emitted_at) AS hour,
    tenant_id,
    topKState(10)(JSONExtractString(payload, 'workflow_id')) AS top10
FROM oya_events_kafka_source
WHERE event_type = 'oya.workflow.executed.v1' AND tenant_id = '${tid}'
GROUP BY hour, tenant_id;

CREATE TABLE tenant_${tid}.topk_workflow_hour (
    hour DateTime,
    tenant_id String,
    top10 AggregateFunction(topK(10), String)
)
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMMDD(hour)
ORDER BY (tenant_id, hour)
TTL hour + INTERVAL 90 DAY TO DISK 's3_cold',
    hour + INTERVAL 365 DAY DELETE
SETTINGS storage_policy = 'hot_cold';
```

### Template 3 — percentile rollup

Already in Template 1 — `quantilesState` is the percentile primitive. Standalone template not duplicated.

Query-time merge example:

```sql
SELECT
    hour,
    quantilesMerge(0.5, 0.95, 0.99)(duration_percentiles) AS p
FROM tenant_${tid}.workflow_hour
WHERE hour > now() - INTERVAL 24 HOUR
GROUP BY hour
ORDER BY hour;
```

### Template 4 — anomaly window

File: `microservices/analytics/iac/clickhouse/mv-templates/mv-minute-error-burst-per-tenant.sql` (already authored).

```sql
CREATE MATERIALIZED VIEW IF NOT EXISTS tenant_${tid}.mv_minute_error_burst_per_tenant
ON CLUSTER analytics-clickhouse-1
TO tenant_${tid}.error_burst_minute
AS
SELECT
    toStartOfMinute(emitted_at) AS minute,
    tenant_id,
    count() AS error_count
FROM oya_events_kafka_source
WHERE event_type = 'oya.workflow.failed.v1' AND tenant_id = '${tid}'
GROUP BY minute, tenant_id
HAVING error_count > 100;

CREATE TABLE tenant_${tid}.error_burst_minute (
    minute DateTime,
    tenant_id String,
    error_count UInt64
)
ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(minute)
ORDER BY (tenant_id, minute)
TTL minute + INTERVAL 7 DAY DELETE;
```

Workload class: Anomaly window (real-time; cadence L1).

### Template 5 — billing daily and monthly (chained)

Files: `mv-day-billing-per-resource.sql` and `mv-month-billing-per-resource.sql` (already authored, used by IP-009).

Chain depth 2 (max per ADR-AN-005). Daily → Monthly is the canonical chain.

## Query-time finalization

Consumers always use the `*Merge` combinators to finalize aggregate states:

```sql
SELECT
    hour,
    countMerge(run_count) AS total_runs,
    quantilesMerge(0.5, 0.95, 0.99)(duration_percentiles) AS p
FROM tenant_${tid}.workflow_hour
WHERE hour > now() - INTERVAL 24 HOUR
GROUP BY hour
ORDER BY hour;
```

Querying the raw `AggregateFunction` columns without `Merge` returns binary state; the application crate's SQL renderer always emits `Merge` for consumer queries.

## Test harness

File: `crates/oya-shared-olap-clickhouse-adapter/tests/mv_canon.rs`

```rust
#[tokio::test]
async fn test_mv_hour_workflow_round_trip() {
    let adapter = setup_test_adapter_with_mv("mv-hour-workflow-per-tenant.sql", "test_mv").await;

    // Insert 1000 synthetic events at known hours.
    for i in 0..1000 {
        adapter.exec_ddl_raw(&format!(
            "INSERT INTO oya_events_kafka_source (event_id, event_type, tenant_id, source_id, payload, emitted_at) \
             VALUES (generateUUIDv4(), 'oya.workflow.executed.v1', 'test_mv', 'src', '{{\"workflow_id\":\"wf{i}\",\"duration_ms\":{i},\"status\":\"succeeded\"}}', toDateTime('2026-05-15 12:00:00'))"
        )).await.unwrap();
    }

    // Wait for MV insertion.
    tokio::time::sleep(Duration::from_secs(5)).await;
    adapter.exec_ddl_raw("OPTIMIZE TABLE tenant_test_mv.workflow_hour FINAL").await.unwrap();

    // Query merged values.
    let rows: Vec<(DateTime<Utc>, u64, u64)> = adapter.query_raw(
        "SELECT hour, countMerge(run_count), sumMerge(total_duration_ms) \
         FROM tenant_test_mv.workflow_hour \
         WHERE hour = toDateTime('2026-05-15 12:00:00') \
         GROUP BY hour"
    ).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, 1000);                                  // count
    assert_eq!(rows[0].2, (0..1000).sum::<u64>() as u64);         // sum
}

#[tokio::test]
async fn test_mv_topk_workflow() {
    let adapter = setup_test_adapter_with_mv("mv-hour-topk-workflow-per-tenant.sql", "test_topk").await;
    // Insert events where "wf_popular" appears 100x and other workflows 1x.
    for _ in 0..100 {
        publish_event(&adapter, "test_topk", "oya.workflow.executed.v1", json!({"workflow_id": "wf_popular"})).await;
    }
    for i in 0..10 {
        publish_event(&adapter, "test_topk", "oya.workflow.executed.v1", json!({"workflow_id": format!("wf_other_{i}")})).await;
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
    let rows: Vec<(String,)> = adapter.query_raw(
        "SELECT topKMerge(10)(top10)[1] FROM tenant_test_topk.topk_workflow_hour"
    ).await.unwrap();
    assert_eq!(rows[0].0, "wf_popular");
}

#[tokio::test]
async fn test_mv_minute_error_burst() {
    let adapter = setup_test_adapter_with_mv("mv-minute-error-burst-per-tenant.sql", "test_burst").await;
    for _ in 0..150 {
        publish_event(&adapter, "test_burst", "oya.workflow.failed.v1", json!({"workflow_id": "wf_burst"})).await;
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
    let rows: Vec<(DateTime<Utc>, u64)> = adapter.query_raw(
        "SELECT minute, error_count FROM tenant_test_burst.error_burst_minute ORDER BY minute DESC LIMIT 1"
    ).await.unwrap();
    assert!(rows[0].1 > 100);  // HAVING clause filters error_count > 100
}

#[tokio::test]
async fn test_billing_daily_monthly_chain() {
    let adapter = setup_test_adapter_with_mv("mv-day-billing-per-resource.sql", "test_chain").await;
    setup_test_adapter_with_mv("mv-month-billing-per-resource.sql", "test_chain").await;

    publish_usage_counter(&adapter, "test_chain", "workflow_run", 100, "2026-04-15T12:00:00Z").await;
    publish_usage_counter(&adapter, "test_chain", "workflow_run", 200, "2026-04-30T12:00:00Z").await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    adapter.exec_ddl_raw("OPTIMIZE TABLE tenant_test_chain.billing_day FINAL").await.unwrap();
    adapter.exec_ddl_raw("OPTIMIZE TABLE tenant_test_chain.billing_month FINAL").await.unwrap();

    let monthly_total: u64 = adapter.query_raw_scalar(
        "SELECT sumMerge(count_state) FROM tenant_test_chain.billing_month WHERE month = '2026-04-01' AND resource_type = 'workflow_run'"
    ).await.unwrap();
    assert_eq!(monthly_total, 300);
}
```

## Per-tenant fan-out

The bootstrap controller (IP-002) renders each MV template per-tenant at onboard. The `${tid}` placeholder is substituted; the rendered DDL is executed via the adapter. Idempotent (`IF NOT EXISTS`).

## Out of scope

- Flink-class stream processing (ADR-0195 escalation tier; ADR amendment required).
- Cross-tenant MV (forbidden — tenants do not pollute each other).
- MV chains depth > 2 (forbidden per ADR-AN-005).
- Streaming join across multiple source tables (not in 26.3 LTS; deferred).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| MV target table not present | MV insertion error log | Bootstrap controller re-reconciles |
| MV logic bug (wrong aggregation) | Reconciliation lane | Backfill via `backfill-replay.md` |
| Source event schema drift | MV `JSONExtract` returns default | ADR-0154 schema versioning + new MV |
| MV slow (target merge backlog) | `ClickHouseMetrics_PartsDelayInsert > 0` | Runbook ingest-lag-burn.md |

## SLO commitment (downstream IP-014)

- MV freshness: target table reflects source insertion within 5s p99 (per `slos/clickhouse-ingest-lag.openslo.yaml`).
- MV query latency: `SELECT ... MERGE ... GROUP BY tenant_id` over 24h window p99 ≤ 100ms.

## Rollback

- Each MV template can be detached individually: `DETACH MATERIALIZED VIEW tenant_${tid}.${mv_name}`.
- Source events continue to land in the Kafka engine table; backfill from there if MV reattaches.

## Evidence emission

- Per MV insertion: ClickHouse `system.materialized_views` row.
- Per MV insertion error: ClickHouse log + Prometheus counter `oya_analytics_mv_insertion_errors_total`.
- Per test-harness run: `evidence/mv-canon-tests/<date>.json`.

## References

- ADR-0195 §"Default: ClickHouse Materialized Views + Kafka Engine".
- ADR-0193 §"Materialized Views — the stream-processing default".
- ADR-AN-005-materialized-view-cadence (canonical naming + chain depth).
- docs/standards/stream-processing-rubric.md (sibling output of this batch).
- ClickHouse `AggregatingMergeTree`: https://clickhouse.com/docs/engines/table-engines/mergetree-family/aggregatingmergetree.
- ClickHouse `quantilesState` / `quantilesMerge`: https://clickhouse.com/docs/sql-reference/aggregate-functions/combinators#-state.
