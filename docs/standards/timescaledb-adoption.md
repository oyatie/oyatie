# TimescaleDB Adoption — Per-µService Pattern

**Authority:** ADR-0194
**Status:** Canonical (2026-05-18)
**Owner:** council-architecture + council-ontology

This standards doc shows the canonical per-µservice pattern for adopting the TimescaleDB community-edition (Apache-2.0) Postgres extension when a µservice has tenant-facing time-series workloads.

## When to adopt

Adopt TimescaleDB when ALL of these hold:

- The µservice has tenant-visible time-series data (workflow execution metrics, business KPIs, asset telemetry).
- The data has row-level update semantics (workflow run outcomes that can be amended).
- The data must participate in transactions with adjacent Tier 1 Postgres OLTP rows.
- The cardinality is bounded enough to fit the four cardinality classes (≤1M series per tenant typically).

Do NOT adopt TimescaleDB when:

- Workload is append-only wide aggregate — that's ClickHouse per ADR-0193.
- Workload is ops/SRE telemetry — that's Prometheus + Mimir per ADR-0186.
- Workload is sparse-tag high-cardinality vector search — that's Milvus per ADR-0192.
- Workload is full-text search — that's Meilisearch per ADR-0184 Tier 4.

## Adoption pattern (per µservice)

### 1. Manifest declaration

```json
{
  "data": {
    "timeseries": {
      "enabled": true,
      "hypertables": [
        {
          "name": "workflow_execution_metrics",
          "timeColumn": "ts",
          "spaceColumn": "tenant_id",
          "cardinalityClass": "medium",
          "retentionDays": 90
        }
      ]
    }
  }
}
```

### 2. Helm post-install hook

Add the canonical chart `microservices/<ms>/iac/helm/timescaledb-extension/` to the µservice's Flux Kustomization. The post-install job creates the extension + renders `create_hypertable()` per manifest declarations.

### 3. Per-µservice refresh + retention worker

Per ADR-0194 §"Continuous aggregates" + §"Retention", TimescaleDB's automated refresh + retention policies are TSL-only. Each µservice ships a sibling worker binary that calls `CALL refresh_continuous_aggregate()` + `SELECT drop_chunks()` on schedule. Canonical worker scaffold at `crates/shared-timescale-policy-worker/` (Phase-2 follow-on; not yet authored). Until that scaffold lands, per-µservice teams write the equivalent ~30 LOC inline in their existing worker binary.

### 4. Kernel-only API surface

Use the `shared-timeseries-kernel` trait surface for all hypertable / continuous-aggregate / retention operations. The kernel rejects TSL function names at compile-or-runtime (depending on call site) per §"TSL component fence". Do not embed raw SQL containing forbidden TSL functions anywhere in the µservice.

### 5. SLO authoring

Author per-hypertable SLOs at `microservices/<ms>/slos/timescaledb-*.openslo.yaml` covering INSERT p99, query p99, refresh lag, retention drop completion.

## Cardinality classes — chunk sizing

| Class | Series per tenant | Chunk interval | Rationale |
|---|---|---|---|
| Low | ≤ 1K | 7 days | Few chunks; planner exclusion easy |
| Medium | 1K–100K | 1 day | Balanced chunk count vs scan range |
| High | 100K–1M | 6 hours | Tight chunk pruning for sparse queries |
| Very high | >1M | 1 hour | Aggressive pruning; capacity-planner flag |

Override in manifest's per-hypertable declaration only when the cardinality class shift is genuine.

## Forbidden TSL functions

The `check-license-policy` CI lane rejects any SQL fragment containing the following function names (enumerated in `shared-timeseries-kernel::FORBIDDEN_TSL_FUNCTIONS`):

- `add_retention_policy`, `add_compression_policy`, `add_continuous_aggregate_policy`, `add_reorder_policy`
- `policy_compression`, `policy_refresh_continuous_aggregate`, `policy_retention`
- `tiered_storage`, `attach_tiered_chunk`
- `approx_percentile`, `approx_count_distinct`, `asof_join`, `lttb`, `timeweight`, `time_weight`
- `interpolated_average`, `interpolated_integral`, `rolling_avg`, `rolling_stderror`
- `skip_scan_enabled`

If a workload genuinely needs one of these, the µservice must either (a) implement the equivalent in pure Postgres / community-edition surface or (b) move the workload to ClickHouse per ADR-0193.

## Sample per-µservice IP

Use this as a starting point for adopting TimescaleDB in a new µservice:

```markdown
# IP-NNN — Adopt TimescaleDB for <workload>

**Authority:** ADR-0194, docs/standards/timescaledb-adoption.md
**Status:** Planned

## Scope

Install the TimescaleDB community-edition extension into <ms>'s Postgres
cluster; declare the canonical hypertable for <workload>; wire per-µservice
refresh + retention worker.

## Deliverables

1. Helm chart consumption: add `microservices/<ms>/iac/helm/timescaledb-extension/`.
2. Manifest update: declare hypertable.
3. Refresh worker addition: ~30 LOC in `crates/oyatie-<ms>-worker/`.
4. SLO authoring: 3-4 OpenSLO sources.
5. Runbook: `microservices/<ms>/runbooks/timescaledb.md`.

## Acceptance

(...)
```

## References

- ADR-0194 — tenant-facing time-series TimescaleDB.
- ADR-0184 — storage tier layering.
- `crates/shared-timeseries-kernel/`
