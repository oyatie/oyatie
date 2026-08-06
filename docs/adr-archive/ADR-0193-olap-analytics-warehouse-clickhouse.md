---
id: ADR-0193
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-observability, axis-analytics
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0031, ADR-0042, ADR-0043-secrets-management-openbao-and-hsm-per-cell, ADR-0131-per-microservice-flat-layout, ADR-0145, ADR-0152, ADR-0155, ADR-0184, ADR-0186, ADR-0192, ADR-0194, ADR-0195]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0193 — OLAP analytics warehouse canonical: ClickHouse 26.3 LTS; tenant-facing analytics, telemetry rollups, audit-log query, billing rollups

## Status

Accepted (2026-05-18). Mandates **ClickHouse 26.3 LTS** (latest LTS: 26.3.10.60-lts as of 2026-05-18; Apache-2.0) as the canonical OLAP analytics warehouse for tenant-facing dashboards, telemetry rollups, audit-log query, and billing aggregation across all 32 µservices. The OLTP / read-replica / cache / search layering from ADR-0184 stands; ClickHouse is the analytics tier that ADR-0184 explicitly does not own.

## Context

Per ADR-0184 (four-tier storage layering: OLTP / read-replica / cache / search) and ADR-0042 (observability stack), oyatie generates three distinct analytics workload classes that the Tier 1-4 storage layering does not serve:

1. **Tenant-facing analytics dashboards.** Workflow execution metrics, business KPIs, agent activity, application telemetry — the data a tenant sees in their ops portal. Wide aggregates over multi-billion-row event tables; query latency budget < 500ms p99.
2. **Ops portal rollups.** Per-tenant SLO compliance, per-µservice capacity utilization, fleet-wide cost attribution — internal queries over the same shape.
3. **Audit-log query + billing rollups.** Audit-chain event search with arbitrary filter predicates over hundreds of billions of rows; billing rollup queries over per-tenant per-resource counters with per-month aggregation windows.

Pushing any of these into Tier 1 Postgres saturates the OLTP write path, ruins p99 for the transactional fleet, and exceeds the read-replica's effective scan ceiling. Pushing them into Tier 4 Meilisearch is a shape mismatch (full-text search ≠ wide aggregate). They need a purpose-built columnar analytics engine.

Hyperscaler practice for OLAP analytics at oyatie's target scale:

- **Cloudflare** — ClickHouse is the named OLAP substrate for their Workers Analytics Engine, R2 metrics, and HTTP analytics dashboards. Petabyte-scale; tenant-facing; multi-region.
- **Uber** — Pinot for one slice of their analytics fleet, ClickHouse for another; both pure columnar engines. ClickHouse won the per-tenant-dashboard slice because of its materialized-view + tiered-storage shape.
- **Shopify** — moved analytics off BigQuery to ClickHouse + Iceberg federation for cost reasons; ClickHouse serves the hot OLAP path.
- **eBay, Tinybird, Plausible** — ClickHouse as the canonical OLAP backend; well-documented at scale.

Anti-patterns this ADR forecloses:

1. Pushing tenant-facing analytics into Postgres OLTP — collision with the transactional fleet.
2. Per-µservice analytics-engine choice (one µservice on Pinot, another on Druid, another on DuckDB) — operator-skill sprawl.
3. Vendor-managed analytics warehouses (Snowflake, BigQuery) — sovereignty failure; data leaves cells.

## Decision

Oyatie adopts **ClickHouse 26.3 LTS** (Apache-2.0) as the canonical OLAP analytics warehouse for tenant-facing dashboards, telemetry rollups, audit-log query, and billing aggregation across the fleet. ClickHouse is deployed cell-locally; per-cell clusters are coordinator-free via ClickHouse Keeper (the Raft-based replacement for ZooKeeper). The `observability` µservice owns the cell-wide observability-rollup cluster; the new `analytics` µservice (or co-owned by `observability` until `analytics` is promoted) owns tenant-facing analytics.

### Cluster shape — coordinator-free via ClickHouse Keeper

ClickHouse 26.3 supports two deployment shapes:

1. **Single-binary** — sufficient for cells with ≤ 1TB hot data + ≤ 100K rows/sec ingest. Acceptable for small cells and dev environments.
2. **Distributed via ClickHouse Keeper** — Raft consensus replaces ZooKeeper; 3-node Keeper minimum for production; per-cell ClickHouse cluster.

oyatie's canonical deployment is distributed via ClickHouse Keeper for production cells, single-binary for cells in dev/preview.

| Plane | Components | Replica count (production cell) |
|---|---|---|
| Keeper (consensus) | ClickHouse Keeper | 3 (Raft quorum) |
| Shard | ClickHouse server (data + query) | 3 shards × 2 replicas = 6 server nodes |
| Coordinator (query routing) | ChProxy or built-in distributed table | 2 instances |

### Use cases — what ClickHouse owns

| Workload | Source pipeline | Shape | Retention |
|---|---|---|---|
| Tenant-facing analytics dashboards | µservice outbox → CDC → ClickHouse `MergeTree` per-tenant partition | Per-tenant per-event-type table; wide aggregates | 90 days hot + 1 year cold (S3 disk) |
| Ops portal rollups | OpenTelemetry Collector → Vector → ClickHouse | Internal per-µservice per-cell tables | 365 days |
| Telemetry aggregations | Prometheus remote-write to ClickHouse (cold-tier alternative to Mimir) | Per-metric per-tenant rollup | 5 years cold |
| Audit-log query | Audit-chain emitter → outbox → ClickHouse | Per-tenant partition; per-axis index | 7 years (compliance) |
| Billing rollups | Per-µservice usage counter → outbox → ClickHouse | Per-tenant per-resource per-day rollup | 7 years (financial) |

ClickHouse does NOT own: transactional writes (Tier 1 Postgres), session/cache (Tier 3 Valkey), full-text search (Tier 4 Meilisearch), vector retrieval (ADR-0192 Milvus), time-series with per-row labels and continuous aggregates (ADR-0194 TimescaleDB).

### Materialized Views — the stream-processing default

ClickHouse Materialized Views are the canonical stream-processing default per ADR-0195. Most stream-processing needs at oyatie's shape — per-tenant rolling aggregates, percentile dashboards, top-K tenant ranking, anomaly windows — are MV in ClickHouse, not Flink. MV semantics:

- **INSERT trigger.** MV runs on every INSERT to the source table; the aggregated row lands in the MV's target table.
- **Kafka Engine integration.** ClickHouse's `Kafka` engine consumes directly from the log-broker substrate (Pulsar 4.2 — Pulsar exposes a Kafka protocol; ClickHouse Kafka engine connects via the Kafka wire protocol).
- **AggregatingMergeTree.** Pre-aggregated state stored compactly; downstream queries do `finalizeAggregation()` at read time.

When ClickHouse MV is insufficient (complex windowing across multiple streams, exactly-once with side effects beyond ClickHouse, complex CEP rules), the workload escalates to Apache Flink per ADR-0195 with explicit ADR amendment.

### Multi-tenancy isolation

ClickHouse 26.3 supports:

1. **Per-tenant database.** Naming pattern: `tenant_{tenant_id}` (e.g., `tenant_ten_acme`).
2. **Row-level policies.** `CREATE ROW POLICY ... USING tenant_id = currentUser()` to enforce per-row tenancy at the engine level.
3. **Per-tenant resource quotas.** `CREATE QUOTA` per user with `MAX queries = X PER hour` + `MAX read_rows = Y PER hour`; projects from ADR-0155 per-tenant resource quotas.

Per-tenant database is the canonical isolation primitive; row-level policies layer on top for tables shared across tenants (e.g., per-µservice ops-portal rollups).

### TTL + partition rotation + cold tier

ClickHouse 26.3 native S3 disk + tiered storage:

- **Hot tier.** Local NVMe (CSI fast-class per ADR-0161). Default 90 days.
- **Cold tier.** S3-compat (SeaweedFS per Fix-S). Per-table `TTL` clause moves rows to cold disk after the hot retention window.
- **TTL DELETE.** Rows beyond compliance retention dropped via `TTL` clause; emits a tombstone event for audit-chain.

Per-table TTL declared in DDL; per-µservice schema lives at `microservices/<ms>/iac/clickhouse/schemas/`.

### Native client — `clickhouse-rs` crate

The `oya-shared-olap-client-kernel` binds to the official ClickHouse Rust client (`clickhouse` crate v0.14.x as of 2026-05-18; Apache-2.0; maintained by the ClickHouse project). HTTP fallback via the native HTTP API is implemented for adapter-level resilience (e.g., when ClickHouse Native protocol versioning drifts ahead of the client crate's compatible range).

### Secrets — OpenBao SecretReference

- ClickHouse admin credentials at `secret/observability/clickhouse/admin-password`.
- Per-µservice service-account credentials at `secret/<ms>/clickhouse/user-password`; rotated on the cell's 90-day schedule.
- The `ClickHouseOlapClient` adapter reads via OpenBao SecretReference; no plaintext credentials in Helm values.

### Helm charts

Per-cell ClickHouse deployment ownership:

- **Observability-owned cluster** — telemetry rollups, ops portal queries. Helm at `microservices/observability/iac/helm/clickhouse/`.
- **Analytics-owned cluster** (or co-owned at observability until analytics promotes) — tenant-facing analytics dashboards, billing rollups, audit-log query. Helm at `microservices/analytics/iac/helm/clickhouse/` (created if `analytics` µservice exists; otherwise co-deployed under observability).

Both clusters share the ClickHouse Keeper component:

- `microservices/observability/iac/kustomize/components/clickhouse-keeper/` — reusable Keeper StatefulSet Kustomize component.

### Self-monitoring

ClickHouse system tables (`system.query_log`, `system.metric_log`, `system.part_log`) export via OpenTelemetry Collector to the cell-meta tier per ADR-0186. ClickHouse's own SLOs (query p99, ingest rate, partition merge backlog) are scraped to the second-tier federated Prometheus.

## Alternatives considered

### (a) **CHOSEN: ClickHouse 26.3 LTS with ClickHouse Keeper**

- **Pros:** Apache 2.0 (clean license); single-binary distributed via Keeper (no ZooKeeper); native columnar engine with state-of-the-art compression; Materialized Views = canonical stream-processing primitive (ADR-0195 default); native S3 cold tier; mature multi-tenancy via row-level policies + per-tenant databases; sub-second query latency at petabyte scale (Cloudflare reference); huge open-source community; first-party Rust client; aligned with hyperscaler practice (Cloudflare, Uber, Shopify, eBay, Plausible).
- **Cons:** distinct ops surface from Postgres; query SQL dialect has ClickHouse-specific extensions; eventually-consistent for replicated tables. Mitigation: SQL dialect drift contained by the kernel layer; ReplicatedMergeTree consistency model documented in operator runbook.
- **Accepted.**

### (b) DuckDB embedded — REJECTED at multi-tenant scale

- **Pros:** embedded; zero ops; great for single-process analytics.
- **Cons:** single-process; no built-in multi-tenancy (process-per-tenant doesn't scale to >5K tenants per cell); no replication; no Materialized Views streaming primitive; no native S3 cold tier; no query coordinator. DuckDB's design point is interactive single-user analytics, not multi-tenant fleet OLAP.
- **Rejected** at multi-tenant scale. DuckDB remains permitted as an in-process adapter for µservice-internal analytics scripts where the dataset fits in one process; not the canonical primary.

### (c) Apache Pinot — REJECTED

- **Pros:** LinkedIn-grade; built for tenant-facing dashboards; pre-aggregation primitives.
- **Cons:** materially more components than ClickHouse (Controller + Broker + Server + Minion + Pinot Server + ZooKeeper or Helix); operator-skill surface much larger; JVM dependency; smaller community than ClickHouse for the analytics-warehouse use case. ClickHouse's MV primitive covers Pinot's pre-aggregation slice with less ops overhead.
- **Rejected** on ops overhead + community size.

### (d) Apache Druid — REJECTED

- **Pros:** real-time analytics; columnar; battle-tested.
- **Cons:** even heavier multi-component shape than Pinot (Broker + Coordinator + Historical + Indexer + MiddleManager + Router + ZooKeeper + Metadata-Store); JVM heavy; ingest-side configuration complexity exceeds ClickHouse.
- **Rejected** on ops overhead.

### (e) Trino + Iceberg / Delta Lake / Hudi (federated query over object store) — REJECTED for primary OLAP

- **Pros:** federated query across multiple sources; open table formats; lake-house pattern.
- **Cons:** query latency materially higher than ClickHouse for the tenant-dashboard slice (federation overhead + object-store IO); coordinator is JVM-heavy; Iceberg / Delta / Hudi licensing matrix (Iceberg Apache-2.0, Delta Lake Apache-2.0 since 2022, Hudi Apache-2.0) is fine but the operational substrate (Trino) is JVM.
- **Rejected** for the tenant-facing-dashboard primary slot. Trino + Iceberg remains valid as a longer-horizon data-lake federation lane for cross-cell ad-hoc analytics where seconds-to-tens-of-seconds latency is acceptable; not the canonical OLAP primary.

### (f) BigQuery / Snowflake / Redshift (managed SaaS) — REJECTED

- **Pros:** zero ops.
- **Cons:** closed-source or partly-closed (Redshift Postgres fork is partially open but the engine is closed); sovereignty failure; data leaves cells; per-pack KR / EU residency impossible. Conflicts with ADR-0014 (build-vs-buy) for substrate primitives.
- **Rejected** on sovereignty.

### (g) StarRocks / Doris (younger Rust+C++ MPP) — DEFERRED

- **Pros:** competitive columnar engines; Apache-2.0; some compelling performance numbers.
- **Cons:** materially younger community than ClickHouse; fewer integrations; ecosystem still consolidating. Re-evaluate at GA+24 months if ClickHouse drift becomes problematic.
- **Deferred** for now.

## Consequences

### Positive

1. **One OLAP substrate fleet-wide.** No per-µservice analytics engine drift.
2. **Materialized Views = stream-processing default.** Cheap, in-OLAP, sub-second freshness for the majority of stream workloads (ADR-0195 default tier).
3. **Apache-2.0 + permissive ecosystem.** ClickHouse + clickhouse-keeper + clickhouse-rs client all Apache-2.0.
4. **Petabyte scale.** Cloudflare reference: petabyte-scale tenant-facing analytics on ClickHouse.
5. **Native cold tier.** S3 disk + TTL move to cold tier without external orchestration.
6. **Per-tenant database + row-level policies = multi-tenant by default.**

### Negative

1. **SQL dialect has ClickHouse extensions.** Mitigation: the `oya-shared-olap-client-kernel` exposes a typed query API; raw SQL only at adapter layer; per-µservice schemas live in source control with version-pinned migrations.
2. **Eventually-consistent for ReplicatedMergeTree.** Mitigation: documented in operator runbook; per-shard read-your-writes is achievable by routing per-tenant queries to the same shard replica.
3. **Distinct ops surface from Postgres.** Mitigation: canonical Helm chart + runbook at `microservices/observability/iac/helm/clickhouse/`; ops-sre-reliability operates ClickHouse as a substrate service.

### Operational

1. Per-µservice manifest declares `data.olap_client.enabled: true` when the µservice publishes events to ClickHouse; `data.olap_client.databases[]` declares per-database schema reference.
2. The `oya-check-olap-tier-discipline` lane is advisory at PR-#145; flips to BLOCKER post-wave when wide-aggregate-on-Postgres-OLTP patterns are exorcised.
3. SLO: ClickHouse query p99 ≤ 500ms for tenant-facing dashboards; ingest backlog < 60s; partition merge backlog < 5 minutes. Authored at `microservices/observability/slos/clickhouse.openslo.yaml`.
4. Capacity: per-cell sizing tracked via `microservices/observability/capacity-model.md` ClickHouse section.
5. Backup: native ClickHouse `BACKUP` command to S3-compat (SeaweedFS); daily incremental + weekly full per ADR-0152.

## In-house roadmap

Per the in-house tech stack policy (user directive 2026-05-18) — "wherever possible, support in-house tech stack like AWS / Google / Microsoft / Oracle do" — ClickHouse is **vendor-replaceable** in oyatie's substrate plan:

### Phase 0 — ClickHouse-via-adapter (current, this ADR)

- ClickHouse 26.3 LTS deployed per the decision above.
- All consumer µservices query through the `oya-shared-olap-client-kernel` port; the `ClickHouseOlapClient` adapter is the only ClickHouse-binding code.
- Kernel trait surface is engine-agnostic (typed query DSL + insert + MV DDL + per-tenant database management).

### Phase 1 — Operational maturity (~Q1 2027 → Q4 2027)

- Per-cell sizing model proven at multi-TB hot tier + multi-cell-replication for the audit-log workload.
- ClickHouse Keeper cluster shape proven at 3-replica + cross-cell federation.
- TTL + cold-tier S3 disk proven at 1-year retention with sub-second hot query.
- Materialized View ingest cadence proven at sub-5-second freshness for 95th-percentile workloads.

### Phase 2 — In-house replacement: `oya-olap-warehouse-server` (~Q1 2028)

A Rust-native columnar OLAP warehouse, shipped as `oya-olap-warehouse-server` under `crates/oya-olap-warehouse-*` and `microservices/observability/iac/helm/oya-olap-warehouse/`:

- **Query engine.** Apache DataFusion (Apache-2.0; Rust-native columnar query engine; first-class in the Arrow ecosystem). DataFusion is already an active open-source query engine; oyatie's in-house lane builds the storage + cluster layer around it.
- **In-memory format.** Apache Arrow (zero-copy columnar IPC).
- **At-rest format.** Apache Parquet (columnar persistence).
- **Custom merge-tree.** Per-tenant LSM-style merge tree with TTL + tiered storage (NVMe → S3-compat) primitives owned in-house. ClickHouse's MergeTree is the spiritual reference; the in-house implementation is owned by oyatie.
- **Multi-tenancy.** Per-tenant database isolation native; per-row policy at the engine layer.
- **Wire protocol.** Postgres-wire-compatible read path (consumers query via the standard Postgres driver where practical) + native gRPC for high-throughput ingest. Postgres-wire compatibility is the Cloudflare-class hyperscaler pattern (e.g., RisingWave, ParadeDB) and dramatically lowers consumer migration cost.

**Trigger conditions** (value-anchored, NOT date-anchored — any one promotes the in-house lane to active development; the date below is a planning anchor only):

1. ≥100 TB per cluster sustained. **(value-anchored)**
2. Cross-tenant query-isolation breach demonstrated (despite per-tenant database + row-level policies + adapter-layer `assert_same_tenant`). **(event-anchored)**
3. ClickHouse license posture changes (the 2022 ClickHouse Inc. fork remained Apache-2.0; this trigger guards against future drift). **(event-anchored)**
4. Materialized View capability ceiling reached for the canonical workload set (escalation tier per ADR-0195 is fully consumed by Flink — meaning MV is no longer the 95% path). **(workload-shape-anchored)**

**Production-validation evidence** for ClickHouse multi-tenant isolation at scale (basis for the multi-tenancy posture in §"Multi-tenancy isolation"): Cloudflare's HTTP analytics petabyte-scale ClickHouse deployment; Tinybird multi-workspace ClickHouse SaaS at thousands of tenants; eBay, Uber, Plausible production references. The per-tenant-database + row-level-policy + per-tenant QUOTA pattern is the canonical industry shape. Per-cell tenant sharding above ~10K tenants documented in `microservices/analytics/capacity-model.md` §"Sharding above 10K tenants per cell".

**Industry parallels** — AWS Redshift (in-house at AWS; built on Postgres lineage with custom columnar storage), Google BigQuery (in-house at Google; Dremel lineage), Microsoft Azure Synapse (in-house at Microsoft; PolyBase lineage), Oracle Exadata + Autonomous Data Warehouse (in-house at Oracle), Snowflake (independent but built columnar from scratch; in-house in their cloud). Every hyperscaler operates their own OLAP warehouse for first-party analytics; oyatie's Phase 2 plan parallels this trajectory.

### Phase 3 — Migration (post Phase-2 GA)

- Per-µservice repointing of the kernel adapter: `olap_backend: "in_house"` in manifest.
- Per-database migration via dual-write window + cutover after query-parity validation.
- ClickHouse retired per cell after all consumer µservices migrate.

The in-house roadmap is a commitment of trajectory, not a near-term deliverable. Phase 0 ships now; Phase 2 is a real engineering investment behind concrete trigger conditions.

## Rollback

- **Helm rollback** — `helm rollback observability-clickhouse` reverts to prior Helm release; data persists in S3 disk + local NVMe.
- **Schema rollback** — per-µservice migration framework supports DDL rollback; MV recompute on rollback if the source table schema reverts.
- **Wholesale rollback** — fall back to Postgres-OLTP wide-aggregate queries; degraded p99 (multi-second); acceptable as a temporary band-aid only.

## References

- ClickHouse — https://clickhouse.com/ ; Apache 2.0.
- ClickHouse 26.3 LTS release notes — https://clickhouse.com/docs/whats-new/changelog
- ClickHouse 26.3 LTS guide (third-party) — https://quantrail-data.com/clickhouse-26-3-lts-features-performance-guide/
- ClickHouse Keeper — https://clickhouse.com/docs/en/guides/sre/keeper/clickhouse-keeper
- ClickHouse Rust client — https://github.com/ClickHouse/clickhouse-rs ; https://crates.io/crates/clickhouse
- ClickHouse Kafka engine — https://clickhouse.com/docs/en/engines/table-engines/integrations/kafka
- ClickHouse Materialized Views — https://clickhouse.com/docs/en/sql-reference/statements/create/view
- ClickHouse multi-tenancy — https://clickhouse.com/docs/en/operations/access-rights
- Cloudflare Workers Analytics on ClickHouse — https://blog.cloudflare.com/http-analytics-for-6m-requests-per-second-using-clickhouse/
- ADR-0031 — ads and analytics µservice architecture.
- ADR-0042 — observability stack (OTel + in-house UI).
- ADR-0043-secrets-management-openbao-and-hsm-per-cell — OpenBao SecretReference.
- ADR-0131-per-microservice-flat-layout — flat layout under `microservices/<ms>/iac/helm/clickhouse/`.
- ADR-0145 — inter-microservice communication reform.
- ADR-0152 — RPO / RTO canonical.
- ADR-0155 — per-tenant resource quotas.
- ADR-0184 — storage tier layering (Tier 1–4; ClickHouse is the analytics tier outside the four-tier model).
- ADR-0186 — observability backplane layering.
- ADR-0192 — vector database canonical (Milvus).
- ADR-0194 — time-series for tenant-facing (TimescaleDB extension).
- ADR-0195 — stream processing tier (ClickHouse MV default; Flink escalation).
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098.
