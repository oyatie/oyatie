# Observability ClickHouse — Capacity Model

**Authority:** ADR-0193, ADR-0186, hyperscaler-architecture-invariants.json
**Last reviewed:** 2026-05-18
**Numbers:** Concrete.

Mirrors `microservices/analytics/capacity-model.md` but scoped to the observability-namespace ClickHouse cluster (telemetry rollups, not tenant-facing dashboards).

## Concrete ceilings

| Dimension | Steady-state | Hard ceiling | Trigger |
|---|---|---|---|
| Hot tier (NVMe) | 100 TB | 200 TB | Add NVMe shards |
| Cold tier (S3-compat) | 1 PB | 5 PB | Trigger ADR-0193 Phase-2 trigger condition #1 |
| Ingest rate | 500 K rows/sec | 2 M rows/sec | Scale data nodes; tune async-insert batch |
| Query QPS (ops portal) | 1 K qps | 5 K qps | Add query replicas |
| Per-µservice tables | ≤ 100 | 500 | Schema rationalization |
| Retention: telemetry rollups | 30 d hot + 365 d cold | — | TTL-driven |

## Concrete Phase-2 in-house trigger (per ADR-0193 §"In-house roadmap")

Any one of the following PROMOTES the `oya-olap-warehouse-server` in-house lane to active development:

1. ≥ 100 TB per cluster sustained. **(VALUE-ANCHORED.)**
2. Cross-tenant query-isolation breach demonstrated despite per-tenant database + row-level policy + adapter-layer `assert_same_tenant`.
3. ClickHouse license posture changes (relicense from Apache-2.0).
4. Materialized View capability ceiling reached — meaning the Class A workload set per ADR-0195 no longer fits MV (escalates to Flink for >50% of workloads).

Date "Q1 2028" in ADR-0193 §"In-house roadmap" is a planning anchor; the actual gate is value-anchored above.

## 4-INV overlay

| Invariant | Status | Evidence |
|---|---|---|
| INV-1: Idempotent writes | YES — async inserts + outbox event_id deduplication | IP-022 |
| INV-2: OTel trace propagation | YES — ClickHouse query_log + Pulsar consumer tracing | ADR-0186 |
| INV-3: Ontology projection | YES — telemetry rolled up per canonical schema | IP-023 |
| INV-4: Per-tenant resource quotas | YES (where applicable; ops/SRE workload is mostly fleet-internal) | IP-022 |

## References

- ADR-0193, ADR-0186, specs/hyperscaler-architecture-invariants.json
- Cloudflare HTTP analytics ClickHouse blog — petabyte-scale production reference.
