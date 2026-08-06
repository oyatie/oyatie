---
id: ADR-0045
status: Proposed
doc_status: published
---

# ADR-0045: Database tier strategy — PostgreSQL + Citus for OLTP, ClickHouse-fork for OLAP, Iceberg + DataFusion for lakehouse

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0028, ADR-0033, ADR-0034, ADR-0040, ADR-0042, ADR-0049

---

## Context

Every axis stores state. The pack-of-19 foundation ADRs decided that database choice is a substrate concern but did not pin the per-tier strategy: which engine for OLTP, which for OLAP, which for lakehouse. Without a pinned strategy, every axis re-decides; every per-microservice choice has its own license posture, scaling profile, backup story, residency story; the cohesion thesis collapses at the data plane.

The license dimension is sharp: PostgreSQL is PostgreSQL Lic (clean); Citus extension is Apache-2 (clean); ClickHouse is Apache-2 but the licensing of some ClickHouse-derived enterprise products has shifted (we use the Apache-2 fork explicitly); Iceberg is Apache-2; DataFusion is Apache-2. This ADR pins the per-tier engine + per-tier extension topology + per-tenant per-cell sharding + retention + DSR cascade integration.

---

## Decision

We adopt **PostgreSQL + Citus** (Apache-2) as the canonical OLTP engine; **per-tenant per-cell shard topology**; **ClickHouse Apache-2 fork** as the canonical OLAP engine (with explicit fork-license verification per License Policy); **Iceberg + DataFusion** (Apache-2) as the canonical lakehouse format + query engine; backup orchestration per ADR-0040 release management; per-store retention + DSR cascade per ADR-0034 + ADR-0038.

### OLTP: PostgreSQL + Citus extension

```yaml
# infra/postgres/per-cell-cluster.yaml
postgres:
  version: "16"
  extensions:
    - citus           # Apache-2; horizontal scaling
    - pgvector        # PostgreSQL Lic; vector search per ADR-0046
    - pgroonga        # LGPL — KR search per ADR-0047 (legal isolation analysis)
    - timescaledb     # Apache-2 (community ed); time-series for telemetry
  per_cell:
    primaries: 3      # one per AZ
    replicas: 6       # 2x read scaling
    storage_class: "premium-nvme"
    backup:
      strategy: "wal-g + per-tenant pg_dump weekly"
      retention: "per-tenant policy from ADR-0034 + per-region statutory minimum"
```

- **Per-tenant per-cell shard.** A tenant's tables live in one cell's Citus cluster; shard distribution by `tenant_id`.
- **Replication factor.** N=3 within a cell across AZs; cross-cell replication opt-in per ADR-0049 residency.
- **Connection pooling.** PgBouncer per cell; per-tenant pool isolation.

### Per-axis tier mapping

| Axis | OLTP usage | OLAP usage | Lakehouse usage |
|---|---|---|---|
| SaaS platform | Tenant tables, audit indexes | Per-tenant analytics rollups | - |
| Workspace | Mail metadata, calendar events, doc graph, permission graph | Per-tenant Workspace usage analytics | Mail body archive (cold; Drive object body in object store) |
| Vertical | Per-vertical canonical entity model | Per-tenant per-vertical analytics | Long-term vertical event history |
| Foundry | Capability registry, agent runs, eval results | Per-capability invocation analytics | Replay corpus + eval corpus |
| Cloud | DCIM inventory, capacity, asset lifecycle | Per-cell utilization, FinOps | Long-term capacity history |
| Search | Index metadata, crawler queue, RTBF queue | Per-query analytics | Crawl corpus, indexed-doc archive |
| Ads/Analytics | Campaign, ad, advertiser, publisher | Per-campaign realtime analytics | Per-tenant warehouse (DP-budgeted), attribution event history |

### OLAP: ClickHouse Apache-2 fork

The official ClickHouse is Apache-2 (verified at adoption). We track license posture per ADR (License Policy); if the upstream license posture changes, we maintain the Apache-2 fork (e.g. clickhouse-bundle-oss).

- Per-cell deployment.
- Per-tenant database within a cell.
- Per-axis usage per the table above.
- Materialized view per common analytics path.

### Lakehouse: Iceberg + DataFusion

- **Format.** Apache Iceberg (Apache-2; CNCF) for table format on object storage.
- **Catalog.** Polaris (Apache-2) at GA; in-house long-horizon.
- **Query engine.** DataFusion (Apache-2; Apache Arrow) for SQL + ad-hoc.
- **Object store.** Per-cell S3-compatible store (per ADR-0028 storage surface).
- **Per-tenant table.** `tenant_<id>.<axis>.<table>`.
- **Per-tenant lifecycle.** Per-tenant retention; partition pruning on read.

### Backup orchestration per ADR-0040 release management

- **OLTP.** WAL-G (Apache-2) per cell; per-tenant `pg_dump` weekly to object storage; PITR window 14d default.
- **OLAP.** ClickHouse `BACKUP TABLE` per per-tenant database; weekly full + daily incremental.
- **Lakehouse.** Iceberg snapshots managed; per-snapshot retention; per-tenant snapshot pruning per DSR.

Per-cell DR drill (per ADR-0040): full restore from backup to standby cell quarterly.

### Per-store retention + DSR cascade integration

- **OLTP.** Per-tenant retention from per-vertical override pack (per ADR-0034). DSR delete: row-level + WAL-purge on schedule. Per-tenant proof-of-erasure emitted (per ADR-0038).
- **OLAP.** DSR delete: per-row delete in mutable tables; per-cohort recompute for aggregates.
- **Lakehouse.** DSR delete: per-row delete via Iceberg `DELETE`; per-snapshot expiration accelerates physical purge.

### Per-tenant residency binding

Per ADR-0049 residency class:

- `strict_kr`: tenant's OLTP + OLAP + lakehouse data resides exclusively in KR cells.
- `kr_with_us_failover`: OLTP primaries in KR; cold backup replicated to US for DR (with per-region encryption + per-region key custody).
- `global`: tenant data may reside in any cell; latency-optimized.

Residency change is destructive (re-create tenant in new cell + DSR cascade old cell).

### Per-axis read/write SLA

Per ADR-0042 SLO catalog:

| Axis | OLTP read P95 | OLTP write P95 | OLAP query P95 |
|---|---|---|---|
| SaaS platform | 5ms | 20ms | 5s |
| Workspace mail metadata | 10ms | 30ms | 10s |
| Vertical claim adjudication | 20ms | 50ms | 30s |
| Foundry capability registry | 5ms | 15ms | 5s |
| Cloud DCIM inventory | 10ms | 30ms | 5s |
| Search index metadata | 10ms | 30ms | 5s |
| Ads campaign | 5ms | 20ms | 30s |

### Schema migration discipline

- **Per-PR migration.** Each schema-affecting PR ships forward + backward migration.
- **Per-migration audit.** Migration application audit-chained.
- **Backward-compatible schema changes.** Required for stable + GA tier per ADR-0037; new tables / new optional columns OK; column drops require deprecation cycle.
- **Per-migration drill.** Per-migration tested against synthetic data corpus before production rollout per ADR-0040.

### Anti-scope

This ADR does not own the vector store (per ADR-0046, but pgvector lives in OLTP). Does not own search backend (per ADR-0047). Does not own the audit chain (per ADR-0003).

---

## Consequences

### Positive

- Single OLTP engine across all axes — uniform operational surface, uniform backup, uniform DSR.
- Per-tenant per-cell sharding maps cleanly to residency commitments.
- License-clean across the tier (PostgreSQL Lic + Apache-2) protects the product surface.
- Iceberg lakehouse gives us ACID + time-travel + per-tenant table isolation at object-store cost.
- Per-axis tier mapping makes "where does this data live" answerable in one table.

### Negative

- Citus has scale ceilings per cluster; very high-throughput tenants need per-cell partitioning beyond what Citus single-cluster supports.
- ClickHouse fork tracking is a license-management cost.
- Schema migration discipline at GA tier is real engineering cost.
- Per-tenant cold backup at object storage scales but recovery time grows with tenant count.

### Operational

- Per-cell PostgreSQL + Citus health alarmed.
- Per-cell ClickHouse health alarmed.
- Per-tenant backup completion tracked.
- Per-quarter DR restore drill.
- Per-cell HSM-wrapped backup encryption per ADR-0043.
- Per-cell connection pool saturation alarmed.

---

## Alternatives considered

### Alternative A — TiDB (distributed SQL) for OLTP

- **Pros:** native horizontal scaling.
- **Cons:** less mature in KR ops contexts; per-cell topology less friendly to our cell isolation pattern; license verified Apache-2 but operational surface heavier than Postgres+Citus.
- **Rejected because:** Postgres+Citus matches our team's expertise and our cell isolation pattern.

### Alternative B — CockroachDB (BSL)

- **Pros:** distributed SQL.
- **Cons:** BSL forbidden in product surface per License Policy.
- **Rejected because:** license posture incompatible.

### Alternative C — DynamoDB / Spanner / proprietary cloud DB

- **Pros:** managed.
- **Cons:** vendor lock-in incompatible with three-phase Cloud trajectory (per ADR-0028); Phase 3 self-hosted fulfillment would re-platform.
- **Rejected because:** the surface invariance per ADR-0028 forbids hyperscaler-only DBs.

### Alternative D — Per-axis DB choice

- **Pros:** microservice-team flexibility.
- **Cons:** N engines; per-engine ops; per-DSR multiplied; cohesion violated.
- **Rejected because:** DB tier is a substrate concern.

---

## Open questions

1. **Q1.** PostgreSQL major-version cadence — track upstream LTS or stay one version behind? Default: track LTS minus one major (currently PG 16). → owner: `foundry`.
2. **Q2.** Citus per-shard count default — 32 or 64? Default: 32 at GA; per-cell tunable. → owner: `foundry`.
3. **Q3.** ClickHouse fork tracking cadence — quarterly merge or per-release? Default: per-release with quarterly review. → ADR-0013.
4. **Q4.** Iceberg catalog (Polaris vs in-house) — at GA Polaris or in-house? Default: Polaris at GA; in-house at W+24+. → owner: `foundry`.
5. **Q5.** Per-tenant encryption-BYOK for OLTP encryption-at-rest at GA, or W+12? Default: W+12 (per ADR-0043 encryption-BYOK question). → ADR-0043.

---

## References

- `docs/PRD.md` §10 (data plane)
- `docs/DESIGN.md` §11 (database tier), §10 (cross-microservice contracts)
- PostgreSQL docs; Citus extension docs; ClickHouse docs; Apache Iceberg + DataFusion docs
- KR 「개인정보의 안전성 확보조치 기준」 (encryption-at-rest requirements)
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0028 (cloud), ADR-0033 (vertical pack), ADR-0034 (per-vertical override), ADR-0040 (progressive delivery), ADR-0042 (observability), ADR-0049 (residency)
