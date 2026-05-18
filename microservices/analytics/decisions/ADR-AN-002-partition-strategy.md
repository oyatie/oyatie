# ADR-AN-002 — Partition Strategy

**Scope:** analytics µservice only. Parent: ADR-0193.
**Status:** Accepted
**Date:** 2026-05-18
**Owner:** council-analytics

## Context

ClickHouse partition strategy affects:
- Query prune efficiency (queries on the partition key column scan only relevant parts).
- TTL behavior (TTL operates at the partition level).
- Mutation cost (`ALTER ... DELETE WHERE` is partition-wise).
- Background merge cost (smaller partitions = more parts = more merges; larger partitions = slower TTL).

The canonical ClickHouse recommendation is "monthly partitions" but that is workload-dependent. Analytics has three regimes (audit log, KPI rolling aggregates, billing rollup) with very different volume profiles.

## Decision

Adopt per-workload partition strategy:

| Workload class | Partition key | Rationale |
|---|---|---|
| Audit log (raw) | `toYYYYMM(emitted_at)` | ~1 month, balances scan prune vs merge cost |
| Audit log (cold tier, post-TTL) | `toYYYY(emitted_at)` | ~1 year — coarser; reads are rare |
| Business KPI hourly rolling aggregate | `toYYYYMMDD(hour)` | daily — granularity matches dashboard window |
| Billing rollup daily | `toYYYYMM(day)` | monthly — finalization is monthly |
| Billing rollup monthly | `toYYYY(month)` | yearly — small data; large partitions OK |
| MV intermediate | `toYYYYMMDD(hour)` | daily — matches business KPI parent |

The ORDER BY (primary key) is always `(tenant_id, ...)` so tenant queries prune efficiently within a partition.

## Per-table partition cardinality budget

Per `capacity-model.md` §1, hard ceiling is 5,000 partitions per table. Workloads:

- Audit log: 84 monthly partitions over 7 yr = 84. Plus cold-year partitions = ~90.
- KPI rolling: 365 daily partitions over 1 yr = 365.
- Billing daily: 84 monthly partitions × 1 = 84.

All within budget.

## Alternatives considered

1. **Fixed monthly partitioning everywhere.** Rejected — KPI rolling aggregates would have too few partitions for efficient daily-window queries.
2. **Daily partitioning everywhere.** Rejected — audit log over 7 yr = 2,555 daily partitions, approaching hard ceiling.
3. **Adaptive partitioning (ClickHouse 27 feature).** Not yet in 26.3 LTS; revisit at engine upgrade.

## Consequences

- **Positive.** Query prune efficient; TTL operates on appropriately-sized chunks; merge cost predictable.
- **Negative.** Per-table partition strategy is non-uniform; ops must know per-table convention.
- **Mitigation.** Convention encoded in DDL templates at `iac/clickhouse/mv-templates/`; CI lane validates conformance.

## Rollout

- Phase 1 (this batch): apply to all new tables.
- Phase 2: migrate existing tables (rare; mostly green-field).

## References

- ADR-0193, ADR-AN-001-ttl-policy.
- ClickHouse partitioning docs: https://clickhouse.com/docs/engines/table-engines/mergetree-family/custom-partitioning-key
