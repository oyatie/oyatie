# IP-012 — Metric Attribution (ClickHouse)

**microservice**: feature-flags
**bc**: metric
**layer**: kernel
**crate**: oya-feature-flags-metric-kernel
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0244, ADR-0248, ADR-0252, ADR-0263, ADR-0276
**companion_ips**: IP-008, IP-009, IP-020

## Scope

Metric event ingest, attribution to experiment assignments, aggregation for statistical engines, DSAR portability export, ClickHouse cold-tier storage. Ingest target: 860 GB/day per cell.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `MetricEvent` struct | Fields: event_id UUID, event_class (14-class), flag_key, tenant_id, timestamp HLC, hlc_timestamp, actor_principal_id, experiment_id, variant, payload_json |
| 2 | `MetricIngestService` | Batched writes to ClickHouse `feature_flags_audit_events` table; batch size 1000; flush interval 100ms |
| 3 | `MetricAttributionService` | Joins `MetricEvent` with `ExperimentAssignment` on `(tenant_id, experiment_id, user_id_hash)`; produces attribution record |
| 4 | `MetricAggregationService` | Pre-aggregates per (experiment_id, variant, metric_name) for `BayesianEngine` + `FrequentistEngine` input |
| 5 | `DsarExportService` | Exports all metric events for `user_id_hash`; encrypted with DEK per ADR-0276; GDPR Art. 20 portability |
| 6 | `ReAttributionService` | Replays events with corrected assignment salt; idempotent (event_id deduplication) |
| 7 | ClickHouse schema | `ReplicatedMergeTree`; `ORDER BY (tenant_id, timestamp, event_class)`; `PARTITION BY toYYYYMM(timestamp)`; TTL 24mo |
| 8 | Tests | Ingest throughput: 10k events/s single-node benchmark; attribution join: 1M events → correct variant counts |

## Capacity Math

- Ingest rate: 10M eval/s × 86% experiment participation × ~100 bytes/event = 860 GB/day
- Batch write: 1000 events × 100ms flush = 10k/s per writer goroutine
- ClickHouse hot tier: 30 days × 860 GB = 25.8 TB
- Cold tier (S3-backed): 24 months × 860 GB = 619 TB

## Definition of Done

- `cargo test -p oya-feature-flags-metric-kernel` green
- Ingest: 10k events/s under benchmark without OOM
- Attribution: zero orphaned metric events after 1M-event simulation
- DSAR export: AES-256-GCM encrypted zip, schema matches `FlagDefinitionExport` in openapi-v1.yaml
