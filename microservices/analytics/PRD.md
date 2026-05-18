# Analytics µservice — Product Requirements Document

**Status:** Draft (introduced 2026-05-18 by data-substrate batch)
**Owner:** council-analytics
**Layout:** Flat per ADR-0131
**Primary ADRs:** ADR-0193 (ClickHouse OLAP), ADR-0184 (storage tier), ADR-0195 (stream processing)

## 1. Purpose

The analytics µservice owns the tenant-facing OLAP analytics warehouse — the place where every tenant's dashboards, audit-log queries, billing rollups, and ops-portal aggregations are served from. It is the canonical home for any read workload whose shape is *wide aggregate over many rows in a columnar store*. Per ADR-0184, Tier 1 Postgres OLTP does not own this shape; per ADR-0193, ClickHouse 26.3 LTS does.

This µservice is intentionally distinct from:

- **observability** µservice — owns ops/SRE telemetry (Prometheus + Mimir + Loki + Tempo per ADR-0186). Observability stores fleet-internal signals; analytics stores tenant-visible business data.
- **foundry** µservice — owns AI substrate including Milvus vector retrieval per ADR-0192. Foundry serves embedding similarity; analytics serves wide aggregates.

The two-µservice split is deliberate: blast-radius isolation, capacity isolation, ownership clarity. A tenant query storm on analytics never starves SRE telemetry; an SRE telemetry storm on observability never starves tenant dashboards.

## 2. Personas

| Persona | Workload | Latency budget |
|---|---|---|
| Tenant admin viewing workflow execution dashboards | Per-tenant rollups over per-day windows | <500ms p99 |
| Tenant ops viewing audit-log search | Filter+paginate over per-tenant per-axis event stream | <800ms p99 |
| Tenant finance viewing billing rollup | Per-day per-resource counter aggregates | <1s p99 |
| Internal SRE viewing ops portal | Per-cell capacity utilization | <1s p99 |
| Internal compliance officer running regulator export | Multi-month event range with axis filter | minutes (bulk export) |
| Internal capacity planner viewing fleet-wide cost attribution | Cross-cell per-µservice spend | <2s p99 |

## 3. Goals and Non-Goals

### Goals

- Sub-second query latency for tenant-facing dashboards (>=p99 < 500ms for the standard dashboard shape).
- Multi-billion row capacity per tenant per cell.
- Per-tenant strict isolation — no tenant can see another tenant's data via any query path.
- Multi-region residency — KR / EU strict residency packs.
- 7-year retention for audit + billing (compliance).
- Cold-tier S3 disk for retention beyond hot window.
- Native Materialized View ingest for rolling rollups (ADR-0195 default tier).

### Non-Goals

- Transactional consistency (OLTP) — that's Tier 1 Postgres.
- Vector retrieval — that's Milvus per ADR-0192.
- Full-text search — that's Meilisearch per ADR-0184 Tier 4.
- Sub-millisecond cache — that's Valkey per ADR-0184 Tier 3.
- Stream-processing escalation (Flink) — that's per-µservice opt-in per ADR-0195.

## 4. Architecture summary

### 4.1 Components

- **ClickHouse cluster** (3 shards × 2 replicas; ClickHouse Keeper 3-node quorum). Helm at `microservices/analytics/iac/helm/clickhouse/`.
- **OLAP client kernel** (`oya-shared-olap-client-kernel`) — engine-agnostic port.
- **ClickHouse adapter crate** (`oya-shared-olap-clickhouse-adapter` — to be authored when this µservice's app is wired; not in this batch).
- **REST + gRPC API surface** at `crates/oya-analytics-api/` (out of this batch's scope — IP-007 + IP-008 + IP-015).
- **Per-tenant database bootstrap controller** — listens for tenant-onboarded events from the tenancy µservice and creates the `tenant_{tenant_id}` database + per-table grants.

### 4.2 Per-tenant isolation

Per ADR-0193 §"Multi-tenancy isolation":
- **Database-per-tenant.** Naming pattern `tenant_{tenant_id}`.
- **Row-level policies.** Layered defense for tables that legitimately share rows (rare; reserved for fleet-wide ops dashboards).
- **Per-tenant quotas.** `CREATE QUOTA tenant_{id}` with `MAX queries`, `MAX read_rows`, `MAX insert_rows` per ADR-0155 projection.

### 4.3 Ingest pipeline

Per ADR-0153 outbox pattern: source µservices emit events to their transactional outbox; the analytics CDC pipeline projects them into ClickHouse via the `Kafka` engine consuming from Pulsar's Kafka-protocol endpoint. Materialized Views on the source table emit rolled-up rows into target `AggregatingMergeTree` tables for sub-second dashboard freshness.

### 4.4 Cold tier

Per ADR-0193 §"TTL + partition rotation + cold tier": hot tier on local NVMe (CSI fast-class); cold tier on SeaweedFS S3-compat. Per-table TTL clause moves rows after the hot window (default 90 days).

### 4.5 Cross-cell federation

ClickHouse `Distributed` table engine routes cross-shard queries within a cell. Cross-cell federation (rare; for global ops aggregates) goes through the `remote()` function with explicit per-cell ClickHouse endpoint enumeration in the federated table's DDL. Tenant queries never federate across cells — tenant data is residency-bound per ADR-0049.

## 5. Capacity targets

- Per-cell: 100TB hot tier (NVMe) + 1PB cold tier (S3). Ingest 100K rows/sec sustained, 500K rows/sec burst.
- Query QPS: 10K qps fleet-wide per cell at p99 <500ms.
- Tenant ceiling: top tenant ≤ 10B rows per table; above that, capacity planner pages.
- Daily backup window: 4h overnight for daily incremental; weekend window for full.

## 6. Cost model (rough)

Per cell at sizing target (100TB hot + 1PB cold):
- 6 ClickHouse server nodes × c5n.4xlarge equivalent → ~$2K/month per cell on commodity.
- 3 Keeper nodes × c5n.large → ~$200/month.
- S3 cold tier at 1PB → ~$25K/month (highly compressible — actual: ~$8K/month after ClickHouse compression).
- Per-cell total ~$10K-$30K/month depending on cold-tier fill.

Detailed cost model at `microservices/analytics/cost-budget.md` (deferred to follow-on doc batch — see parent-wiring-todo).

## 7. Compliance posture

- **PII handling.** Per ADR-0156 PII registry — analytics serves PII-tagged columns only via Cedar-authorized read paths. Tenant audit-log query is allowed; cross-tenant aggregation that would surface PII is forbidden by Cedar.
- **GDPR DSR.** Per ADR-0038 — tenant offboard drops the `tenant_{tenant_id}` database; proof-of-erasure emitted.
- **Audit chain.** Every analytics query against the audit-log surface emits its own audit event (recursive — auditing the audit-log query — to prevent silent observation).

## 8. Open questions

1. Does the tenant-facing dashboard surface live in the application µservice or in a new analytics-facing UI µservice? — Default: application µservice consumes via REST/gRPC; new UI surface deferred.
2. Cross-cell federation for global ops aggregates — performance vs simplicity? — Default: per-cell rollups + scheduled cross-cell aggregation jobs; live federation deferred.

## 9. Phase plan

- **PHASE-01: ANALYTICS-OLAP-BOOTSTRAP.** Stands up the cluster, the per-tenant bootstrap controller, the outbox→ClickHouse CDC pipeline, and the first three tenant dashboards (workflow execution metrics; audit-log search; billing rollup). 15 IPs. (PHASE-01 spec at `microservices/analytics/PHASE-01-ANALYTICS-OLAP-BOOTSTRAP.md`.)

## 10. References

See ADR list at the top.
