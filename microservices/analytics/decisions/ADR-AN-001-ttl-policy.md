# ADR-AN-001 — TTL Policy Per Workload Class

**Scope:** analytics µservice only (service-scoped). Parent: ADR-0193.
**Status:** Accepted
**Date:** 2026-05-18
**Owner:** council-analytics

## Context

Per ADR-0193 §"TTL + partition rotation + cold tier", each table declares a TTL. The analytics µservice serves three workload classes (audit log, business KPI, billing rollup) with substantially different retention obligations:

- **Audit log** — 7 yr (SOC 2 + ISO 27001 + tenant compliance contracts).
- **Business KPI** — 1 yr (tenant operational utility; older data low signal).
- **Billing rollup** — 7 yr (tax + dispute window).

A single TTL policy across these workloads would either over-retain (waste storage) or under-retain (regulatory exposure). Per-workload TTL is therefore canonical.

## Decision

Adopt the per-workload TTL policy below. The Helm chart `microservices/analytics/iac/helm/clickhouse-analytics/` exposes these as `values.workloadTtl.<workload>.{hot, cold, delete}` overrideable per pack.

| Workload class | Hot tier (NVMe) | Cold tier (S3) | Final delete |
|---|---|---|---|
| Audit log | 90 d | 7 yr | 7 yr |
| Business KPI (per-tenant rolling aggregates) | 90 d | 1 yr | 1 yr |
| Billing rollup (daily) | 30 d | 7 yr | 7 yr |
| Billing rollup (monthly) | 30 d | 7 yr | 7 yr |
| Telemetry rollup (observability cross-feed; rare) | 30 d | 1 yr | 1 yr |
| MV intermediate (AggregatingMergeTree finalization) | 90 d | none | 90 d |
| Per-tenant proof-of-erasure receipts | 7 yr (hot) | none | NEVER (compliance evidence) |

The DDL emits the TTL clause at table-creation time per IP-002 + IP-006:

```sql
CREATE TABLE tenant_${tid}.audit_events (
    emitted_at DateTime, tenant_id String, axis String, event_type String, payload String
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(emitted_at)
ORDER BY (tenant_id, axis, emitted_at)
TTL emitted_at + INTERVAL 90 DAY TO DISK 's3_cold',
    emitted_at + INTERVAL 7 YEAR DELETE
SETTINGS storage_policy = 'hot_cold';
```

## Alternatives considered

1. **Single fleet-wide TTL.** Rejected — over-retains KPIs (waste) or under-retains audit (compliance).
2. **Tenant-tier-aware TTL** (e.g., Enterprise gets 365 d hot for KPI). Deferred to a phase-2 amendment; first pass uses class-based, not tier-based.
3. **No TTL; rely on manual partition drop.** Rejected — fragile; misses compliance evidence.

## Consequences

- **Positive.** Storage cost predictable per workload. Compliance auditable from DDL itself.
- **Negative.** Per-table DDL is more complex; mistake risk if a new table omits the TTL clause.
- **Mitigation.** CI lane `oya-foundry-fitness-ttl-presence` (deferred — F-AN-002) verifies every `tenant_*.*` table has a TTL clause and that the clause matches the workload class declared in the table's `comment`.

## Rollout

- IP-002 controller renders TTL based on table's workload class.
- Existing tables without TTL: backfilled by reconciliation job at next deploy.

## References

- ADR-0193 §"TTL + partition rotation + cold tier".
- IP-006 cold-tier S3 disk + TTL retention.
- `microservices/analytics/compliance.md` (retention requirements).
