# ADR-AN-004 — Per-Tenant Query Budget Per Tier

**Scope:** analytics µservice only. Parent: ADR-0155 quotas, ADR-0193.
**Status:** Accepted
**Date:** 2026-05-18
**Owner:** council-analytics + council-tenancy

## Context

ClickHouse `QUOTA` provides resource isolation but its parameters are coarse:

- `MAX queries` per interval.
- `MAX read_rows` per interval.
- `MAX read_bytes` per interval.
- `MAX execution_time` per query.

We need to project the canonical 4-tier tenancy (Trial / Starter / Growth / Enterprise per `oya-tenancy-kernel::B2bTenantTier`) into specific QUOTA values, and we need the values to make business sense (not too tight to drive customers away; not too loose to allow noisy-neighbor incidents).

## Decision

Adopt the per-tier matrix below. Per-hour granularity (ClickHouse `FOR INTERVAL 1 HOUR`). Single key dimension is `user_name` (which maps 1:1 to tenant via IP-002's bootstrap).

| Tier | max queries / hr | max read_rows / hr | max insert_rows / hr | max concurrent queries | max execution time |
|---|---|---|---|---|---|
| Trial | 100 | 10 M | 1 M | 4 | 30 s |
| Starter | 1,000 | 1 B | 100 M | 16 | 60 s |
| Growth | 10,000 | 10 B | 1 B | 32 | 120 s |
| Enterprise | 100,000 | 1 T (capped) | 100 B (capped) | 64 | 300 s |

QUOTA DDL template (rendered per tier):

```sql
CREATE QUOTA quota_tenant_${tid}
  KEYED BY user_name
  FOR INTERVAL 1 HOUR
    MAX queries        = ${tier.max_queries},
        read_rows      = ${tier.max_read_rows},
        written_rows   = ${tier.max_insert_rows}
  TO tenant_${tid}_reader, tenant_${tid}_writer;

ALTER USER tenant_${tid}_reader SETTINGS
  max_concurrent_queries_for_user = ${tier.max_concurrent},
  max_execution_time              = ${tier.max_execution_seconds};
```

## Quota-exceeded handling

When ClickHouse returns error 201 (QUOTA_EXCEEDED):

1. Adapter maps to `KernelError::AdapterError("quota_exceeded: queries=...")`.
2. API layer returns HTTP 429 with `Retry-After: <seconds-until-window-reset>`.
3. Cedar emits `oya.analytics.quota_exceeded.v1` audit event.
4. Tenant-portal surfaces the rate-limit alert.

## Tier upgrade path

Tier change is emitted by tenancy µservice as `oya.tenancy.tenant.tier_changed.v1`. IP-002's controller observes this and re-applies the QUOTA within 30 s. The tier change is **immediately effective**; in-flight queries that were already started under the old quota complete; new queries get the new quota.

## Alternatives considered

1. **Per-tenant individual quota negotiation.** Rejected — operationally untenable for 5,000+ tenants per cell.
2. **Per-minute granularity instead of per-hour.** Considered. Per-hour chosen because (a) ClickHouse `FOR INTERVAL 1 MINUTE` works but creates 60 quota windows per hour, which is high overhead, and (b) bursty workloads (legitimate dashboard refresh) deserve per-hour smoothing.
3. **Per-second token bucket.** Rejected — adds an out-of-engine rate limiter; complicates the failure mode (token bucket up but DB down).
4. **No quotas; rely on cluster-wide ceiling.** Rejected — fails the noisy-neighbor isolation test.

## Consequences

- **Positive.** Predictable resource budget per tenant; clear tier-upgrade path. Aligns with the canonical tier matrix.
- **Negative.** Enterprise tier "unlimited" is actually capped at 1 T rows / hr; documented.
- **Mitigation.** Capacity-planning monitors aggregate Enterprise tier usage against the cell ceiling.

## Quota review cadence

Quarterly. If >5% of tenants regularly burst above their tier's quota and tier upgrade is appropriate, account team initiates the upgrade conversation. If the quota itself is too tight (legitimate workloads hit ceiling), the matrix is revised via ADR amendment.

## References

- ADR-0155 per-tenant resource quotas.
- ADR-0193 §"Multi-tenancy isolation".
- IP-011 per-tenant quota enforcement.
- `microservices/analytics/capacity-model.md` §"Per-tenant resource ceilings".
