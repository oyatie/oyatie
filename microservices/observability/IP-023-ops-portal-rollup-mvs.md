# IP-023 — Ops Portal Rollup Materialized Views

**Phase:** PHASE-02-OBSERVABILITY-CLICKHOUSE-EXTENSION
**Owner:** backend (axis-observability)
**Authority ADRs:** ADR-0193 §"Materialized Views as canonical stream-processing default", ADR-0195 ops portal authority, ADR-0067 ops portal authority, ADR-0186 observability backplane, ADR-0001 cohesion authority (cost attribution), ADR-0145 inter-microservice communication
**Depends on:** IP-022
**Status:** Planned
**Phase trace:** PHASE-02 §"Ops portal rollups" (addendum lines 30-36).

## Scope

Author the **Materialized Views** that drive `ops.oyatie.com`. Per ADR-0193, MVs are the canonical stream-processing default — every aggregation we'd otherwise run in a separate stream processor is expressed as a ClickHouse MV. These four MVs power the ops portal's primary tiles:

1. **Per-µservice per-cell per-hour health** (rollup of error rate, p99 latency, request count).
2. **Per-cell capacity hourly** (CPU, memory, network, storage rollups).
3. **Per-tenant cost daily** (per ADR-0001 cohesion authority, attribute cost back to tenants).
4. **SLO-burn-rate observed** (multi-window burn rate; feeds the SLO alert).

MVs are **append-mode** writing to `*_target` tables (the ops portal reads from the target tables, not the MVs themselves) — this is the ADR-0193 canonical pattern.

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `microservices/observability/contracts/clickhouse-tables/mv-ops-microservice-health-hourly.sql` | create | 1-140 | MV + target table DDL |
| `microservices/observability/contracts/clickhouse-tables/mv-ops-per-cell-capacity-hourly.sql` | create | 1-130 | MV + target |
| `microservices/observability/contracts/clickhouse-tables/mv-ops-per-tenant-cost-daily.sql` | create | 1-160 | MV + target; per ADR-0001 |
| `microservices/observability/contracts/clickhouse-tables/mv-ops-slo-burn-rate-observed.sql` | create | 1-140 | MV + target; multi-window |
| `microservices/observability/contracts/clickhouse-views/v_ops_portal_microservice_health.sql` | create | 1-90 | Public view consumed by ops portal |
| `microservices/observability/contracts/clickhouse-views/v_ops_portal_cell_capacity.sql` | create | 1-80 | Public view |
| `microservices/observability/contracts/clickhouse-views/v_ops_portal_tenant_cost.sql` | create | 1-90 | Public view (row-policy enforced) |
| `microservices/observability/contracts/clickhouse-views/v_ops_portal_slo_burn.sql` | create | 1-100 | Public view |
| `microservices/observability/iac/kustomize/components/clickhouse-ddl-bootstrap/configmap.yaml` | edit | extend with MV SQL (lines 30-60) | infra |
| `microservices/observability/tests/integration/mv_microservice_health_correctness.rs` | create | 1-180 | synthetic input → expected aggregate |
| `microservices/observability/tests/integration/mv_per_cell_capacity_correctness.rs` | create | 1-160 | synthetic input → expected aggregate |
| `microservices/observability/tests/integration/mv_per_tenant_cost_correctness.rs` | create | 1-200 | cost attribution math |
| `microservices/observability/tests/integration/mv_slo_burn_rate_correctness.rs` | create | 1-200 | 1h + 6h + 24h windows |
| `microservices/observability/tests/integration/ops_portal_query_hits_mv_target.rs` | create | 1-140 | EXPLAIN shows target table read |
| `microservices/observability/tests/integration/mv_refresh_lag.rs` | create | 1-100 | refresh lag p99 < 5s |

## MV catalog

### `mv_ops_microservice_health_hourly` (and target `_target`)

```sql
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_ops_microservice_health_hourly
TO ops.mv_ops_microservice_health_hourly_target
AS SELECT
    microservice_id,
    cell_id,
    pack,
    toStartOfHour(timestamp) AS hour,
    countIf(error = true) / count() AS error_rate,
    quantileExact(0.99)(latency_ms) AS p99_latency_ms,
    count() AS request_count
FROM telemetry.otel_metrics
WHERE metric_name IN ('request_count', 'latency_ms', 'error_count')
GROUP BY microservice_id, cell_id, pack, hour;
```

Target schema includes the materialised columns + `_inserted_at` for freshness diagnostics.

### `mv_ops_per_cell_capacity_hourly`

Aggregates CPU / memory / network / storage from node-exporter scraping into per-cell totals.

### `mv_ops_per_tenant_cost_daily`

Per ADR-0001 cohesion authority — attribute compute + storage + network cost back to the originating tenant:

```sql
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_ops_per_tenant_cost_daily
TO ops.mv_ops_per_tenant_cost_daily_target
AS SELECT
    tenant_id,
    pack,
    toDate(timestamp) AS day,
    sumIf(value, metric_name = 'cpu_seconds') * cpu_unit_cost AS compute_cost,
    sumIf(value, metric_name = 'bytes_stored') * storage_unit_cost AS storage_cost,
    sumIf(value, metric_name = 'bytes_network') * network_unit_cost AS network_cost
FROM telemetry.otel_metrics
WHERE tenant_id != 'internal'
GROUP BY tenant_id, pack, day;
```

(`cpu_unit_cost`, `storage_unit_cost`, `network_unit_cost` are read from a dictionary `ops.cost_unit_rates` updated daily.)

### `mv_ops_slo_burn_rate_observed`

Multi-window burn rate per ADR-0186 SLO model (1h, 6h, 24h windows; per-SLO).

## Acceptance criteria

- Each MV produces **correct aggregates** against synthetic input (verified by per-MV correctness tests).
- Ops portal queries hit the **MV target tables** (not raw event tables) — verified by `EXPLAIN` showing the target-table read.
- **Refresh lag < 5s p99** for each MV (verified by `mv_refresh_lag.rs`).
- MV definitions are idempotent — re-applying DDL is a no-op (`CREATE MATERIALIZED VIEW IF NOT EXISTS`).
- Per-tenant cost MV honours row-level policy (per ADR-0193) — a tenant can only see their own row via the public view `v_ops_portal_tenant_cost`.
- Cost unit rate dictionary refresh is daily; rate change is audit-chain logged.
- SLO burn rate output matches the canonical Mimir-computed burn rate within 1% over a 24h window (cross-check guards against arithmetic drift).
- All 4 MVs survive ClickHouse Keeper leader election with no aggregation gaps.

## Test plan

| Test | Verifies |
|---|---|
| `test_mv_microservice_health_hourly_correctness` | error_rate + p99 latency math |
| `test_mv_per_cell_capacity_hourly_correctness` | CPU + memory + network aggregations |
| `test_mv_per_tenant_cost_daily_correctness` | cost attribution math + dictionary lookup |
| `test_mv_slo_burn_rate_multi_window` | 1h + 6h + 24h burn rates emitted |
| `test_ops_portal_query_hits_target_not_view` | EXPLAIN ANALYZE shows `_target` table read |
| `test_mv_refresh_lag_under_5s` | refresh lag p99 < 5s |
| `test_per_tenant_cost_row_policy_enforced` | tenant A cannot see tenant B rows |
| `test_cost_unit_rate_dictionary_refresh_daily` | dictionary reloads at 02:00 UTC |
| `test_slo_burn_rate_matches_mimir` | cross-check against Mimir ≤ 1% drift |
| `test_mv_idempotent_dll_apply` | re-applying DDL is no-op |
| `test_mv_survives_keeper_leader_election` | leader switch → no aggregation gap |
| `test_mv_handles_schema_evolution_additive` | additive column on source table → MV still works |

## Evidence emission

- **Audit chain (ADR-0145):** `clickhouse.mv.applied`, `clickhouse.mv.refreshed`, `clickhouse.cost.unit_rate.changed` events to `oya.observability.audit.clickhouse.mvs`.
- **Metrics:** `clickhouse_mv_refresh_lag_seconds{mv_name}`, `clickhouse_mv_refresh_errors_total{mv_name}`, `clickhouse_mv_target_row_count{mv_name}`.
- **Cost attestation:** monthly per-tenant cost attestation pack at `evidence/cost-attribution/observability-clickhouse-<tenant>-<month>.json`.
- **MV correctness drift:** daily cross-check job emits `evidence/mv-cross-check/observability-<date>.json`.

## Rollback procedure

1. **Bad MV definition.** MVs are deterministic from upstream events; rollback = `DROP MATERIALIZED VIEW IF EXISTS mv_<name>` + re-apply prior version + back-fill from raw events (the target table is recoverable via `INSERT SELECT FROM raw`).
2. **Bad cost-rate change.** Dictionary updates are versioned; rollback = re-apply prior dictionary version; affected days are recomputed via the manual back-fill procedure documented at `microservices/observability/runbooks/clickhouse.md`.
3. **Aggregation drift detected.** If the daily cross-check vs Mimir exceeds 1%, the MV is automatically quarantined (replaced with last-known-good output) until the drift is investigated.
4. **Schema evolution mismatch.** Schema-version bumped on the source table without a corresponding MV update → MV halts; alert fires; rollback the source-table change OR ship the MV update.

## Blocking deps

- IP-022 (bridge) accepted — raw tables populated with sufficient data to test MVs.
- Cost unit rate dictionary published by the finance / capacity-planning µservice.
- Ops portal frontend reads `v_*` views (cross-µservice contract per ADR-0067).

## Exit criteria

All 4 MVs in production; cross-check daily job has 7 consecutive days < 1% drift; ops portal queries demonstrably hit target tables; per-tenant cost attestation accepted by finance; runbook `clickhouse.md` covers MV back-fill procedure.

## Out of scope

- Cold-tier retention (IP-024) — TTL on target tables is set there.
- Backup (IP-025).
- Tenant-facing analytics MVs (analytics µservice).

## References

- ADR-0193 §"Materialized Views as canonical stream-processing default".
- ADR-0195 — ops portal authority.
- ADR-0067 — ops portal authority.
- ADR-0186 — observability backplane (SLO model).
- ADR-0001 — cohesion authority (cost attribution).
- ADR-0145 — communication reform.
- Runbook: `microservices/observability/runbooks/clickhouse.md`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/observability/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/observability/IP-023-ops-portal-rollup-mvs.md` matched `p99, SLO`; anchors `microservices/observability/runbooks/clickhouse-restore.md, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/observability/IP-023-ops-portal-rollup-mvs.md` matched `cost, attribution, emission`; anchors `microservices/observability/manifest.json, crates/oya-cloud-observability-api/src/lib.rs`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.
