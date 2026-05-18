# Analytics µservice — Capacity Model

**Authority:** ADR-0193, ADR-0001 cohesion, hyperscaler-architecture-invariants.json (4-INV overlay)
**Last reviewed:** 2026-05-18
**Numbers:** Concrete. No aspirational targets.

## Per-cluster (per-cell) concrete ceilings

| Dimension | Steady-state target | Hard ceiling | Trigger above hard ceiling |
|---|---|---|---|
| Tenants per cluster | ≤ 5,000 | 10,000 | Shard into a second cell-local cluster |
| Tenant databases per cluster | = tenants × 1 | 10,000 | Same — database catalog throughput degrades >10K |
| Hot tier (NVMe) | 50 TB | 100 TB | Add NVMe shards or rebalance to cold-tier earlier |
| Cold tier (S3-compat) | 500 TB | 1 PB | Per-tenant cold-tier rotation cadence increases |
| Ingest rate | 100 K rows/sec | 500 K rows/sec | MV ingest lag SLO drift; capacity team paged |
| Query QPS | 10 K qps fleet | 50 K qps fleet | Add server replicas; consider compute/storage separation |
| Per-tenant tables | ≤ 50 | 200 | Tenant exceeds template — review schema design |
| Per-table partitions | ≤ 1,000 | 5,000 | Partition strategy review |
| Per-table rows | ≤ 10 B | 100 B | Move to a dedicated cell-local cluster |
| Concurrent queries (per server replica) | ≤ 64 | 128 | Set via `max_concurrent_queries`; reject above |

## Per-tenant resource ceilings (project from ADR-0155 + tier matrix)

| Tier | Queries/hr | Read rows/hr | Insert rows/hr | Concurrent queries |
|---|---|---|---|---|
| Trial | 100 | 10 M | 1 M | 4 |
| Starter | 1,000 | 1 B | 100 M | 16 |
| Growth | 10,000 | 10 B | 1 B | 32 |
| Enterprise | 100,000 | 1 T (capped) | 100 B (capped) | 64 |

Quota exceedance returns HTTP 429 + Cedar evidence; documented in `microservices/analytics/runbooks/clickhouse.md`.

## Sharding above 10K tenants per cell

When a cell approaches the 10K-tenant ceiling:

1. Provision a second ClickHouse cluster in the same cell (`analytics-clickhouse-2`).
2. Per-tenant routing: the analytics API's connection-pool selects cluster by `hash(tenant_id) mod N`.
3. Tenant migration during onboard transition is forbidden — new tenants land on the under-loaded cluster.
4. Cross-cluster queries (rare; only fleet-wide ops dashboards) use `remote()` per IP-010.

This pattern is the Cloudflare / Tinybird canonical multi-tenant scaling shape; documented in CloudFlare's R2 analytics blog post and Tinybird's per-workspace isolation docs.

## Materialized View (MV) lag budget — concrete

Per ADR-0195 §"Default tier" + IP-005:

| MV class | Lag p99 target | Hard ceiling | Trigger |
|---|---|---|---|
| Per-tenant rolling 1h aggregate | < 5 s | 30 s | Add Kafka consumers (parallelism) |
| Top-K per hour | < 5 s | 30 s | Same |
| Percentile rollup per 5min | < 5 s | 30 s | Same |
| Anomaly window per minute | < 2 s | 10 s | Same (tighter — real-time alert) |
| Billing rollup daily | < 60 s | 5 min | Daily window — coarser SLO |

## ClickHouse Keeper Raft quorum

- 3-replica Raft quorum. Tolerates 1 failure → 2 of 3 = quorum.
- `keeper_server.coordination_settings.session_timeout_ms = 30000` (30s).
- Hard limit: 4 simultaneous failures → quorum lost; cluster DDL fails. Mitigation: 5-replica Keeper for production cells (override in pack-eu / pack-kr overlay).

## Cold-tier rotation cadence (concrete)

| Workload class | Hot retention | Cold retention | Delete |
|---|---|---|---|
| Tenant business KPIs | 90 d | 1 yr (cold) | 1 yr |
| Tenant audit log | 90 d | 7 yr (cold) | 7 yr |
| Billing rollups | 30 d (hot) | 7 yr (cold) | 7 yr |
| Telemetry rollups | 30 d | 365 d (cold) | 365 d |

TTL clauses applied at table-creation time per IP-006.

## 4-INV overlay (hyperscaler-architecture-invariants)

Per `specs/hyperscaler-architecture-invariants.json`:

| Invariant | Status for analytics µservice | Evidence |
|---|---|---|
| INV-1: Idempotent writes | YES — `ReplacingMergeTree` + outbox `event_id` | IP-004 |
| INV-2: OpenTelemetry trace propagation | YES — query handlers emit spans with `tenant_id` | IP-014 + ADR-0186 |
| INV-3: Ontology projection | YES — per-tenant events land in canonical ClickHouse tables | IP-002 + ADR-0145 |
| INV-4: Per-tenant resource quotas | YES — ClickHouse QUOTA per ADR-0155 | IP-011 |

## Tier-A patterns (REST/gRPC API) — concrete

Per Tier-A patterns from hyperscaler-architecture-invariants:

- **Idempotency keys.** `Idempotency-Key` header per ADR-0150 + ADR-0151; analytics API stores 24h key history in Valkey per tenant.
- **Cursor pagination.** Opaque cursor (HMAC-SHA256-signed) per ADR-0150; default page size 50; max 500.
- **X-Request-Id propagation.** Per ADR-0151; required header; analytics binds to inbound + emits to ClickHouse via `client_id` query setting.
- **429 Retry-After.** Per-tenant quota-exceeded returns 429 + `Retry-After: <seconds>`.
- **Cedar-authorized.** Every external API path authorized per ADR-0007.
- **Audit-chain emission.** Per external query per ADR-0003.

## Capacity-planning escalation

When 70% of any hard ceiling above is breached for 7 consecutive days, capacity-planning team is paged via PagerDuty `capacity-planning`. Pre-defined escalation actions (sharding, replica add, tier downgrade) documented in `microservices/analytics/runbooks/capacity-rebalance.md` (deferred to follow-on doc batch).

## References

- ADR-0193, ADR-0155, ADR-0001 cohesion, ADR-0150 cursor pagination, ADR-0151 X-Request-Id.
- specs/hyperscaler-architecture-invariants.json
- Cloudflare R2 analytics ClickHouse blog post — public reference for multi-tenant ClickHouse at petabyte scale.
- Tinybird workspace isolation docs — public reference for per-workspace ClickHouse isolation.
