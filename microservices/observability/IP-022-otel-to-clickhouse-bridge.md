# IP-022 — OpenTelemetry Collector → ClickHouse Bridge

**Phase:** PHASE-02-OBSERVABILITY-CLICKHOUSE-EXTENSION
**Owner:** infra (axis-observability)
**Authority ADRs:** ADR-0186 observability backplane (Stage 1 ingest + Stage 2 storage), ADR-0193 OLAP analytics warehouse, ADR-0145 inter-microservice communication, ADR-0193 §"Multi-tenancy isolation"
**Depends on:** IP-021
**Status:** Planned
**Phase trace:** PHASE-02 §"Bridge — OTel collector to ClickHouse" (addendum lines 22-28).

## Scope

Configure the **OpenTelemetry Collector gateway**'s `clickhouseexporter` to ship metric / log / trace rollups into ClickHouse. ClickHouse is the **long-retention** specialized backend for telemetry rollups beyond the hot windows of Prometheus (15d) / Loki (30d) / Tempo (15d). Per ADR-0186 Stage 2 "Storage" specialization, the OTel Collector multi-pipelines telemetry: hot tier → Prometheus / Loki / Tempo; warm + cold tier → ClickHouse for ops-portal rollups + per-µservice 90-365 day retention.

Per ADR-0193 §"Multi-tenancy isolation", every row carries a `tenant_id` column (or `internal` sentinel for cross-tenant infra metrics); per-tenant database namespacing is enforced at the Materialized View boundary in IP-023.

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `microservices/observability/iac/helm/otel-collector-gateway/values.yaml` | edit | extend exporters block (lines ~120-220) | infra |
| `microservices/observability/iac/helm/otel-collector-gateway/templates/collector-config.yaml.tpl` | edit | add `clickhouse` exporter + 3 pipelines (lines ~80-200) | infra |
| `microservices/observability/contracts/clickhouse-tables/metrics-table.sql` | create | 1-90 | DDL for `otel_metrics` |
| `microservices/observability/contracts/clickhouse-tables/logs-table.sql` | create | 1-80 | DDL for `otel_logs` |
| `microservices/observability/contracts/clickhouse-tables/traces-table.sql` | create | 1-100 | DDL for `otel_traces` |
| `microservices/observability/contracts/clickhouse-tables/mv-rollup-microservice-cell-hourly.sql` | create | 1-110 | MV for per-µservice per-cell per-hour aggregation |
| `microservices/observability/iac/kustomize/components/clickhouse-ddl-bootstrap/job.yaml` | create | 1-90 | k8s Job runs DDL on cluster bootstrap |
| `microservices/observability/iac/kustomize/components/clickhouse-ddl-bootstrap/configmap.yaml` | create | 1-40 | mounts DDL files |
| `microservices/observability/tests/integration/otel_bridge_metrics.rs` | create | 1-180 | metrics round-trip |
| `microservices/observability/tests/integration/otel_bridge_logs.rs` | create | 1-160 | logs round-trip |
| `microservices/observability/tests/integration/otel_bridge_traces.rs` | create | 1-180 | traces round-trip |
| `microservices/observability/tests/integration/otel_bridge_backpressure.rs` | create | 1-140 | exporter back-off on ClickHouse pressure |

## OTel Collector exporter config (extract)

```yaml
exporters:
  clickhouse/observability:
    endpoint: tcp://observability-clickhouse:9000?secure=true
    auth:
      basic_auth:
        username_from_env: CH_OTEL_WRITER_USER
        password_from_env: CH_OTEL_WRITER_PASS
    database: telemetry
    metrics_table_name: otel_metrics
    logs_table_name: otel_logs
    traces_table_name: otel_traces
    ttl_days: 365
    compress: lz4
    create_schema: false   # DDL owned by the bootstrap Job, not the exporter
    sending_queue:
      enabled: true
      queue_size: 5000
    retry_on_failure:
      enabled: true
      initial_interval: 5s
      max_interval: 60s
      max_elapsed_time: 30m
service:
  pipelines:
    metrics/clickhouse: { receivers: [otlp], processors: [batch, attributes/tenant_label, resourcedetection], exporters: [clickhouse/observability] }
    logs/clickhouse: { receivers: [otlp], processors: [batch, attributes/tenant_label], exporters: [clickhouse/observability] }
    traces/clickhouse: { receivers: [otlp], processors: [batch, tail_sampling, attributes/tenant_label], exporters: [clickhouse/observability] }
```

## Per-signal-type target tables

| Signal | Table | Engine | Partition key | TTL hot→cold (IP-024) |
|---|---|---|---|---|
| Metrics | `otel_metrics` | `MergeTree` | `toYYYYMM(timestamp)` | 90d hot → cold; 365d delete |
| Logs | `otel_logs` | `MergeTree` | `toYYYYMM(timestamp)` | 90d hot → cold; 365d delete |
| Traces | `otel_traces` | `MergeTree` | `toYYYYMM(start_timestamp)` | 30d hot → cold; 180d delete |

Every table includes `tenant_id LowCardinality(String)`, `microservice_id LowCardinality(String)`, `cell_id LowCardinality(String)`, `pack LowCardinality(String)`.

## Acceptance criteria

- Metrics flow into ClickHouse `otel_metrics` within **30s p99** of emission (matches `clickhouse-ingest-throughput.openslo.yaml`).
- Logs flow into `otel_logs` within **30s p99**.
- Traces flow into `otel_traces` within **60s p99** (longer due to tail-sampling delay).
- MV rollups produce per-µservice per-cell per-hour aggregates with refresh lag **< 5s p99**.
- DDL bootstrap Job runs idempotently on cluster bootstrap; re-runs on schema-version bump.
- ClickHouse credentials for the OTel writer have INSERT permission only (no SELECT / DROP / ALTER).
- Backpressure: when ClickHouse is overloaded, exporter queue back-pressures; receivers drop ingest with metrics, no panic.
- All rows carry `tenant_id`, `microservice_id`, `cell_id`, `pack` for downstream MV partitioning.

## Test plan

| Test | Verifies |
|---|---|
| `test_metric_emit_lands_in_clickhouse` | OTLP metric → otel_metrics row visible within 30s |
| `test_log_emit_lands_in_clickhouse` | OTLP log → otel_logs row |
| `test_trace_emit_lands_in_clickhouse` | OTLP trace → otel_traces row |
| `test_rollup_mv_refresh_lag_under_5s` | sample writes → MV aggregate visible within 5s |
| `test_ddl_bootstrap_idempotent` | re-running job is no-op |
| `test_ddl_bootstrap_handles_schema_bump` | bump → ALTER applied non-destructively |
| `test_writer_credentials_least_privilege` | writer cannot DROP TABLE |
| `test_backpressure_drops_with_metric` | ClickHouse paused → `otelcol_exporter_send_failed_records_total` increments |
| `test_tenant_label_present_on_all_rows` | every row has non-null tenant_id |
| `test_microservice_label_present_on_all_rows` | every row has microservice_id |
| `test_compression_lz4` | objects on disk use LZ4 codec |
| `test_otel_exporter_retry_on_5xx` | exporter retries with backoff |

## Evidence emission

- **Audit chain (ADR-0145):** `clickhouse.ddl.applied`, `clickhouse.ddl.failed`, `clickhouse.bridge.exporter.degraded` events to `oya.observability.audit.clickhouse.bridge`.
- **Metrics:** `otelcol_exporter_sent_records_total{exporter="clickhouse"}`, `otelcol_exporter_send_failed_records_total`, `otelcol_exporter_queue_size`.
- **DDL evidence:** `evidence/observability/clickhouse-ddl-<schema-version>.json` capturing the applied DDL hash per cell.
- **Bridge health:** Prometheus `up{job="otel-collector-gateway"}` and exporter-specific metrics on the cluster-overview dashboard.

## Rollback procedure

1. **Bad DDL applied.** DDL bootstrap Job records the prior schema version; rollback re-applies the prior DDL. ALTER TABLE conventions ensure backwards compatibility (additive columns only); destructive changes require a 2-phase migration (parallel-write, cutover, drop).
2. **Bad exporter config.** Helm rollback the OTel Collector chart; in-flight buffered records are flushed within `max_elapsed_time` (30m default).
3. **ClickHouse unhealthy.** Exporter back-pressures; receivers signal the upstream Collector agents; agents buffer locally up to the agent's queue cap. Beyond cap, data is dropped with a metric — this is the ADR-0186 contract.
4. **Tenant label missing.** Bad upstream change → `tenant_label_present` test should catch in CI before merge; emergency hot-fix is a Collector restart with a fixed `attributes/tenant_label` processor.

## Blocking deps

- IP-021 (cluster) accepted.
- OTel Collector gateway deployed (separately under the observability µservice's general OTel IaC).
- DDL bootstrap Job's ServiceAccount has CREATE / ALTER on the `telemetry` database.
- ExternalSecret operator provides the OTel writer credentials.

## Exit criteria

All test rows green; bridge ingestion sustained for 7 consecutive days at production-equivalent volumes (≥ 10K records/sec) with 0 data-loss events; DDL evidence pack accepted; ingest-throughput SLO budget unburned over the burn-in window.

## Out of scope

- Rollup MVs themselves (IP-023; this IP only sets up the **canary** MV at `mv_rollup_microservice_cell_hourly`).
- Cold-tier retention (IP-024).
- Backup (IP-025).
- Per-tenant analytics dashboards (analytics µservice).

## Schema evolution discipline

- **Additive-only column changes** within a schema-version generation.
- Destructive changes require a **2-phase migration**: (1) ship the new schema with parallel write to both old + new tables; (2) cut over readers; (3) drop the old table.
- Schema version is tracked in `system.schema_version_observability` (single-row metadata table); bumped only by the DDL bootstrap Job.
- Per ADR-0193, ClickHouse schema evolution follows the canonical ALTER patterns documented at `microservices/observability/runbooks/clickhouse.md` §"Schema migration".

## Per-pack residency for ingest path

The OTel Collector gateway is **per-pack**: the kr-* pack's gateway writes to the kr-* ClickHouse cluster; the eu-* pack's gateway writes to the eu-* cluster. Cross-pack writes are denied by:

1. NetworkPolicy on the ClickHouse cluster (ingress allowlist limited to local-pack gateways).
2. Cedar policy at the gateway's middleware (deny cross-pack export).

This is defense-in-depth — neither layer alone is sufficient.

## References

- ADR-0186 — observability backplane (Stages 1 + 2).
- ADR-0193 — OLAP analytics warehouse.
- ADR-0145 — communication reform.
- OpenSLO: `clickhouse-ingest-throughput.openslo.yaml`.
- Upstream OTel: github.com/open-telemetry/opentelemetry-collector-contrib (`clickhouseexporter`).
