# ADR-AN-005 — Materialized View Cadence and Naming Convention

**Scope:** analytics µservice only. Parent: ADR-0193 §"Materialized Views — the stream-processing default", ADR-0195.
**Status:** Accepted
**Date:** 2026-05-18
**Owner:** council-analytics

## Context

ADR-0195 establishes ClickHouse Materialized Views as the default stream-processing tier. ADR-0193 §"Materialized Views" describes the high-level semantics. Neither prescribes:

- The aggregation cadence (per-minute / per-hour / per-day).
- The naming convention.
- The fan-out per tenant vs per-cluster.
- The chain depth allowed.

In the absence of a convention, MV proliferation becomes hard to govern (operators cannot tell which MV produces which target; chained MVs hide latency).

## Decision

### Cadence

| Tier | Source row freshness | Aggregate cadence | Target SLO |
|---|---|---|---|
| L1 (per-minute) | < 60 s | 1-minute buckets | rare; only anomaly windows |
| L2 (per-5min) | < 60 s | 5-minute buckets | percentile rollups, top-K |
| L3 (per-hour) | < 5 s | 1-hour buckets | DEFAULT for dashboards |
| L4 (per-day) | < 5 s | 1-day buckets | billing daily |
| L5 (per-month) | < 60 s (chained from L4) | 1-month buckets | billing monthly |

**L3 (per-hour) is the default.** Tenants get hourly rollups unless their dashboard semantics demand otherwise.

### Naming convention

```
mv_${cadence}_${entity}_${dimension}
```

Examples:

- `mv_hour_workflow_per_tenant` — hourly workflow rollup, per-tenant dimension.
- `mv_minute_error_burst_per_tenant` — minute-granularity error burst MV.
- `mv_day_billing_per_resource` — daily billing rollup by resource type.

The cadence prefix is mandatory; if no cadence (e.g. straight projection), use `mv_proj_*`.

### Target table naming

```
${entity}_${cadence}
```

Examples: `workflow_hour`, `error_burst_minute`, `billing_day`.

### Fan-out

- **Per-tenant fan-out (preferred).** One MV instance per tenant, projecting from the global Kafka source filtered by `tenant_id = ${tid}`. Target table is in `tenant_${tid}.${target}`.
- **Per-cluster (rare).** Used only for fleet-wide ops dashboards. Target table is in `fleet_internal.${target}`. Row-level policy per ADR-AN-003.

### Chain depth

- **Maximum 2 levels.** L4 daily billing → L5 monthly billing is allowed (chain of 2). Anything deeper is forbidden by convention — operationally too hard to debug.
- **L5 monthly is the deepest target.** If a tenant needs quarter or year aggregation, query the L5 monthly MV directly with `WHERE month BETWEEN ...`.

### State engine

| Aggregation kind | Target engine | Combinator |
|---|---|---|
| sum | `AggregatingMergeTree` | `sumState` / `sumMerge` |
| count | `AggregatingMergeTree` | `countState` / `countMerge` |
| percentile | `AggregatingMergeTree` | `quantilesState` / `quantilesMerge` |
| top-K | `AggregatingMergeTree` | `topKState` / `topKMerge` |
| threshold-emitter (anomaly window) | `MergeTree` | none — emits rows directly |

## Alternatives considered

1. **Free-form MV cadence.** Rejected — proliferates; ops can't reason about freshness.
2. **Single canonical cadence (always hourly).** Rejected — anomaly windows need minute granularity; billing needs daily.
3. **Deeper MV chains.** Rejected — debug pain outweighs storage savings.

## Consequences

- **Positive.** MV catalog is consistent; new MV proposal is reviewed against the matrix.
- **Negative.** Coarse cadences (hour, day) introduce dashboard staleness budget. Documented in `capacity-model.md`.
- **Mitigation.** L1/L2 cadences available for tenants that need real-time anomaly views.

## CI enforcement

Per `oya-foundry-fitness-mv-naming` (deferred — F-AN-003): every MV in the `mv-templates/` directory and in every per-tenant rendered DDL conforms to the naming convention.

## Rollout

- New MVs: comply at first PR.
- Existing MVs: rename via migration in phase 2 (cosmetic; no behavior change).

## References

- ADR-0193, ADR-0195.
- IP-005 Materialized View canon.
- `microservices/analytics/iac/clickhouse/mv-templates/`.
- ClickHouse `AggregatingMergeTree` docs: https://clickhouse.com/docs/engines/table-engines/mergetree-family/aggregatingmergetree
