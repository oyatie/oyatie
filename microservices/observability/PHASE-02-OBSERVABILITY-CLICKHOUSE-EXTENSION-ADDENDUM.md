# PHASE-02 Observability ClickHouse Extension — Addendum

**Authored:** 2026-05-18
**Authority:** ADR-0193 (OLAP analytics warehouse canonical ClickHouse), ADR-0186 (observability backplane), ADR-0131 (per-microservice flat layout).
**Status:** Drafting — promotes alongside IP-021..IP-025 acceptance.

## Why an addendum

The Observability µservice's existing `PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md` predates the ClickHouse canonical decision (ADR-0193 promoted 2026-05-18). The 5 ClickHouse IPs (021..025) introduced 2026-05-18 by Fix-R require a new phase entry — **PHASE-02 Observability ClickHouse Extension** — to anchor them inside the M01 milestone sequencing.

This addendum is the temporary anchor. When the next PHASE consolidation lands, this file is folded into `PHASE-02-OBSERVABILITY-CLICKHOUSE-EXTENSION.md` (canonical) and retired.

## Phase scope

PHASE-02 extends the observability µservice's storage layer with a ClickHouse 26.3 LTS cluster for telemetry rollups + ops-portal queries. The OTel Collector gateway's `clickhouseexporter` ships metrics / logs / traces to ClickHouse; Materialized Views feed `ops.oyatie.com`. Cold-tier retention rotates partitions to SeaweedFS S3-compat at 90 days. Backups land in SeaweedFS S3-compat with daily incremental + weekly full + quarterly drill.

Distinct from the analytics µservice's tenant-facing ClickHouse cluster (separate deployment). The two clusters never share state.

## Phase work items (IP traceability)

### §"Cluster bootstrap — first work item" (IP-021)

6 ClickHouse server pods (3 shards × 2 replicas) + 3 ClickHouse Keeper pods (Raft quorum). Helm chart already authored; this IP validates + adds per-pack overlays + RBAC. **All other work items depend on IP-021.**

### §"Bridge — OTel collector to ClickHouse" (IP-022)

`clickhouseexporter` configured in the OTel Collector gateway with 3 pipelines (metrics / logs / traces). Per-signal-type tables in the `telemetry` database. Backpressure on exporter queue. DDL bootstrap Job applies schema + handles additive schema evolution.

### §"Ops portal rollups" (IP-023)

4 Materialized Views feed `ops.oyatie.com`: per-µservice health hourly, per-cell capacity hourly, per-tenant cost daily (per ADR-0001 cohesion authority), SLO-burn-rate observed. Per-tenant cost MV honours row-level policies.

### §"Hot → cold tiered retention" (IP-024)

`TTL ... TO DISK 's3_cold'` for 90d hot→cold transition; `TTL ... DELETE` at 365d. Per-pack residency: KR cold objects in kr-* buckets only, EU in eu-* only. Per-tenant cost MV retained 7 years (finance policy).

### §"Backup + DR drill" (IP-025)

Native ClickHouse `BACKUP` SQL to SeaweedFS S3-compat. Daily incremental + weekly full + quarterly drill. RPO ≤ 24h, RTO ≤ 30min. Re-ingest from Prometheus / Loki / Tempo is the **preferred** recovery for in-hot-window data loss.

## Phase exit gate

- All 5 IPs at `Accepted`.
- ClickHouse ingest-throughput SLO (`clickhouse-ingest-throughput.openslo.yaml`) unburned over 30 consecutive days.
- Query-latency SLO (`query-latency-logs.openslo.yaml`) unburned over 30 consecutive days.
- First quarterly backup drill complete with RTO + RPO targets met.
- First quarterly residency attestation accepted (KR + EU cold-tier residency intact).
- Observability-oncall has drilled all 3 ClickHouse runbooks (`clickhouse.md`, `clickhouse-restore.md`, `clickhouse-cold-tier-incident.md`).
- Ops portal queries demonstrably hit MV target tables (not raw event tables).

## Dependency graph

```
IP-021 (cluster IaC)
  ├── IP-022 (OTel bridge)
  │     └── IP-023 (ops portal MVs)
  ├── IP-024 (cold-tier retention)
  └── IP-025 (backup + drill)
        depends on IP-021 (+ IP-024 storage policy applied to cold partitions)
```

## References

- ADR-0193 — OLAP analytics warehouse canonical ClickHouse.
- ADR-0186 — observability backplane.
- ADR-0131 — per-microservice flat layout.
- ADR-0184 — storage tier layering.
- ADR-0145 — inter-microservice communication.
- ADR-0152 — RPO/RTO targets.
- ADR-0180 — DR business-continuity portfolio.
- ADR-0049 — cross-region replication.
- ADR-0067 — ops portal authority.
- ADR-0195 — ops portal authority.
- ADR-0001 — cohesion authority (cost attribution).
