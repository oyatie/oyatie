# IP-004 — Outbox → ClickHouse CDC Ingest Pipeline

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics)
**Authority ADRs:** ADR-0193, ADR-0153 outbox pattern, ADR-0154 event schema versioning, ADR-0195 stream processing
**Depends on:** IP-002, IP-003
**Status:** Planned

## Scope

Bridge the canonical Postgres-outbox stream (per ADR-0153) into ClickHouse via the `Kafka` engine consuming from Pulsar's Kafka-on-Pulsar (KoP) endpoint. Per ADR-0195, this is the default stream-processing path — no separate Flink cluster. The pipeline projects per-µservice outbox events into per-tenant `AggregatingMergeTree` and raw-event tables.

The pipeline has three layers:

1. **Source layer** — ClickHouse `Kafka` engine consumes from Pulsar (KoP protocol on port 9092).
2. **MV projection layer** — Materialized Views per event type fan out to per-tenant target tables.
3. **Idempotency layer** — `ReplacingMergeTree` on target tables ensures replayed events do not double-count.

Per ADR-0154, event schema versioning is enforced: every event carries a `type` field with version suffix (`oya.workflow.executed.v1`); the Kafka engine table accepts any version, and the MV filters on version match.

## Deliverables

1. ClickHouse `Kafka`-engine source tables for each canonical event stream (workflow events, audit events, billing events).
2. Materialized Views `oya.events.*` projecting from the Kafka source into the per-tenant target tables (rendered per-tenant by IP-002's bootstrap controller).
3. Per-µservice outbox-publisher contract — every source µservice's outbox publishes to the canonical Pulsar topic.
4. End-to-end ingest lag SLO: <5s from outbox commit to MV target row visibility (p99).
5. Backpressure verified — Kafka engine slows ingest when MV target merge backlog grows.
6. Failover semantics verified — Pulsar broker rolling restart does not lose events.
7. Reconciliation lane comparing source-µservice outbox row count to MV target row count daily.

## Acceptance criteria

- Insert a row into a source µservice's outbox; row appears in `tenant_${tid}.events` within 5s p99.
- Kafka engine consumer lag stays <10K messages per shard at steady state (100K rows/s sustained).
- MV target table rows are idempotent (re-publishing the same outbox row does not double-count).
- Pulsar broker failover (rolling restart) does not lose events; the Kafka consumer resumes from the last committed offset.
- Reconciliation drift (source outbox vs MV target) < 0.01% over 24h.
- Schema drift (source µservice adds a field) does not break ingest; the new field is dropped until the Kafka engine table is altered.

## Implementation tasks

### T1 — Kafka engine source tables

File: `microservices/analytics/iac/clickhouse/kafka-source-events.sql`

```sql
CREATE TABLE IF NOT EXISTS oya_events_kafka_source
ON CLUSTER analytics-clickhouse-1
(
    event_id        UUID,
    event_type      String,
    tenant_id       String,
    source_id       String,
    payload         String,  -- JSON; downstream MVs JSONExtract
    emitted_at      DateTime64(3, 'UTC')
)
ENGINE = Kafka
SETTINGS
    kafka_broker_list = 'pulsar-kafka.observability.svc.cluster.local:9092',
    kafka_topic_list = 'oya.events.outbox',
    kafka_group_name = 'analytics-clickhouse-events',
    kafka_format = 'JSONEachRow',
    kafka_num_consumers = 6,
    kafka_max_block_size = 65536,
    kafka_skip_broken_messages = 100;  -- skip schema-mismatched rows; logged as errors
```

Similar source tables:

- `oya_audit_events_kafka_source` — consumes `oya.events.audit`.
- `oya_usage_counters_kafka_source` — consumes `oya.usage.counters` (per IP-009 T1).

### T2 — Per-tenant MV templates

The MV templates render at tenant onboard:

- `iac/clickhouse/mv-templates/mv-hour-workflow-per-tenant.sql` (per IP-005).
- `iac/clickhouse/mv-templates/audit-log-table.sql` + per-tenant insertion MV.
- `iac/clickhouse/mv-templates/mv-day-billing-per-resource.sql` (per IP-009).

Each MV filters `WHERE tenant_id = '${tid}'` and projects into the per-tenant target table.

### T3 — Idempotency

Target tables use `ReplacingMergeTree(version_col)` or `AggregatingMergeTree` per the workload:

```sql
-- Raw audit-event table — replacing semantics.
CREATE TABLE tenant_${tid}.audit_events (
    event_id UUID, emitted_at DateTime64(3), ...
)
ENGINE = ReplacingMergeTree(emitted_at)
ORDER BY (tenant_id, axis, emitted_at, event_id);
```

For `ReplacingMergeTree`, re-publishing the same `event_id` with the same `emitted_at` results in deduplication at merge time. The dedup key is the `ORDER BY` tuple.

For `AggregatingMergeTree`, each MV insertion is independent; if the same source event is replayed, the MV would emit twice. Mitigation: the upstream Kafka consumer commits offsets only after the MV insertion completes (per ClickHouse Kafka engine semantics).

### T4 — Failover semantics

Kafka consumer offsets persist via the Kafka engine's standard offset commit to the broker; Pulsar's KoP backs offsets in BookKeeper. Pulsar broker restart triggers consumer rebalance; offsets resume; no data loss within the at-least-once guarantee.

For exactly-once semantics, dedup on `event_id` at the target table's `ORDER BY` provides the second leg.

### T5 — Backpressure

When MV target table merge backlog grows beyond threshold (`ClickHouseMetrics_PartsDelayInsert > 0`), the Kafka engine consumer slows. This is surfaced as:

- Prometheus metric `ClickHouseMetrics_PartsDelayInsert`.
- Alert `ClickHousePartMergeBacklog` (per IP-001).
- Customer-visible effect: ingest lag SLO burn → IP-014 alerts → runbook `ingest-lag-burn.md`.

### T6 — Schema drift handling

When a source µservice adds a field, the Kafka engine table does not need to be altered — `kafka_skip_broken_messages` is non-zero, and missing-field rows are dropped with a logged error. The recommended path is:

1. Source µservice opens a PR adding the new field to `oya.events.outbox` events.
2. Analytics µservice opens a follow-up PR adding the column to the Kafka engine table + the relevant MV.
3. CI lane `oya-governance-event-schema-coverage` (per ADR-0154; deferred — F-AN-004) gates the two PRs to land in order.

### T7 — Reconciliation lane

File: `crates/oya-analytics-ingest-reconciler-app/` (new crate; Kubernetes CronJob daily 03:00 UTC).

```rust
// Compare source outbox row count vs MV target row count over the last 24h.
// Source: query each source µservice's outbox table (cross-µservice; needs read-only grant).
// Target: SELECT count() FROM tenant_*.events WHERE emitted_at > now() - INTERVAL 1 DAY.
// Emit drift metric to Prometheus.
```

Drift threshold:

- < 0.01% — green.
- 0.01-1% — ticket.
- > 1% — page.

### T8 — Integration test

File: `crates/oya-analytics-ingest-reconciler-app/tests/integration.rs`

```rust
#[tokio::test]
async fn test_outbox_to_target_within_5s() {
    let app = setup_test_app().await;
    bootstrap_tenant(&app, "test_ingest").await;
    let event_id = uuid::Uuid::new_v4();
    publish_outbox_event(&app, "oya.workflow.executed.v1", "test_ingest", event_id, json!({
        "workflow_id": "wf_1", "duration_ms": 100, "status": "succeeded"
    })).await;

    // Poll target table.
    let start = Instant::now();
    let mut observed = 0;
    while start.elapsed() < Duration::from_secs(5) {
        observed = query_count(&app, "tenant_test_ingest.workflow_hour", event_id).await;
        if observed > 0 { break; }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(observed > 0, "event did not project within 5s");
}

#[tokio::test]
async fn test_duplicate_event_id_deduplicated() {
    let app = setup_test_app().await;
    bootstrap_tenant(&app, "test_dedup").await;
    let event_id = uuid::Uuid::new_v4();
    let payload = json!({"workflow_id": "wf_1", "duration_ms": 100, "status": "succeeded"});
    publish_outbox_event(&app, "oya.workflow.executed.v1", "test_dedup", event_id, payload.clone()).await;
    publish_outbox_event(&app, "oya.workflow.executed.v1", "test_dedup", event_id, payload).await;
    wait_for_mv_lag(&app, Duration::from_secs(10)).await;
    optimize_table(&app, "tenant_test_dedup.audit_events").await;
    let count = query_event_count(&app, "tenant_test_dedup.audit_events", event_id).await;
    assert_eq!(count, 1, "duplicate event_id should dedupe");
}

#[tokio::test]
async fn test_pulsar_failover_no_data_loss() {
    let app = setup_test_app().await;
    bootstrap_tenant(&app, "test_failover").await;

    // Start emitting events.
    let emit_handle = tokio::spawn(async move {
        for i in 0..1000 {
            publish_outbox_event(&app, "...", "test_failover", uuid::Uuid::new_v4(), json!({"i": i})).await;
        }
    });

    // Rolling-restart Pulsar brokers.
    rolling_restart_pulsar(&app).await;

    emit_handle.await.unwrap();
    wait_for_mv_lag(&app, Duration::from_secs(15)).await;

    let count = query_target_count(&app, "tenant_test_failover.audit_events").await;
    assert_eq!(count, 1000, "no events lost during failover");
}
```

## Out of scope

- Per-tenant routing of cross-tenant events (forbidden by design — tenants do not cross-pollute).
- Event-schema versioning beyond ADR-0154 (per-source µservice owns its schema).
- Flink-tier streaming compute (escalation per ADR-0195; requires ADR amendment).
- Schema evolution that drops a field (breaking change; new event type version required).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| Pulsar broker total outage | Consumer lag alert | Pulsar HA (3 brokers); k8s automatic restart |
| MV target table corruption | ClickHouse `system.parts` check | Restore from backup (IP-012) |
| Per-tenant target table missing | MV insertion error log | Bootstrap controller re-reconciles (IP-002) |
| Schema-mismatched event in Kafka | `kafka_skip_broken_messages` skips + logs | Investigate source µservice; align schema |
| Consumer lag growing | `clickhouse_kafka_consumer_lag` > 10K | Runbook ingest-lag-burn.md; increase `kafka_num_consumers` |
| Cross-tenant event landing in wrong DB | impossible — MV filters `WHERE tenant_id = '${tid}'` | Defense-in-depth via adapter `assert_same_tenant` |

## SLO commitment (downstream IP-014)

- Ingest lag p99 ≤ 5s (per `slos/clickhouse-ingest-lag.openslo.yaml`).
- Consumer lag p99 ≤ 10K messages.
- Reconciliation drift < 0.01% daily.

## Rollback

- Kafka engine table is independent; DROP halts ingest cleanly. Outbox accumulates in Pulsar with configurable retention.
- MV detach is safe; reattaching resumes from current Kafka offset.

## Evidence emission

- Per Kafka consumer rebalance: ClickHouse log event.
- Per `kafka_skip_broken_messages` skip: log + Prometheus counter.
- Per reconciliation run: `evidence/reconciliation/analytics-<date>.json`.

## References

- ADR-0193 §"Materialized Views — the stream-processing default".
- ADR-0195 §"Default: ClickHouse Materialized Views + Kafka Engine".
- ADR-0153 outbox pattern.
- ADR-0154 event schema versioning.
- ClickHouse Kafka engine docs: https://clickhouse.com/docs/engines/table-engines/integrations/kafka.
- ClickHouse Materialized View docs: https://clickhouse.com/docs/sql-reference/statements/create/view#materialized-view.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/analytics/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/analytics/specs/IP-004-outbox-cdc-ingest-pipeline.md:26` - 4. End-to-end ingest lag SLO: <5s from outbox commit to MV target row visibility (p99).; `microservices/analytics/specs/IP-004-outbox-cdc-ingest-pipeline.md:33` - - Insert a row into a source µservice's outbox; row appears in `tenant_${tid}.events` within 5s p99..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/analytics/specs/IP-004-outbox-cdc-ingest-pipeline.md:230` - ## Evidence emission.
