---
id: ADR-0194
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-ontology, axis-analytics
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
related: [ADR-0042, ADR-0043-secrets-management-openbao-and-hsm-per-cell, ADR-0131-per-microservice-flat-layout, ADR-0145, ADR-0153, ADR-0155, ADR-0179-postgres-connection-pooling-pgcat, ADR-0184, ADR-0186, ADR-0192, ADR-0193, ADR-0195]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0194 — Tenant-facing time-series canonical: TimescaleDB 2.26 Community Edition (Apache-2.0) as a Postgres 18 extension; ops/SRE metrics remain in Prometheus + Mimir

## Status

Accepted (2026-05-18). Mandates **TimescaleDB 2.26.x Community Edition (Apache-2.0)** as the canonical tenant-facing time-series store — a Postgres extension on the existing Tier 1 Postgres 18.4 OLTP fleet per ADR-0184. Per-µservice opt-in via manifest. Distinct from ops/SRE metrics, which remain in Prometheus 3.12 + Grafana Mimir 3.0 per ADR-0186.

**License posture is non-trivial:** TimescaleDB ships in two editions — Community (Apache-2.0 since v2.0) and TSL (Timescale License; source-available but not OSI-OSS). This ADR scopes oyatie to the **Apache-2.0 community core only**; TSL-only features are forbidden under the open-standard primitive doctrine. Section "TSL component fence" below enumerates exactly what is in-scope vs out-of-scope.

## Context

Per ADR-0184, Tier 1 (Postgres 18.4 OLTP) owns transactional writes; Tier 2 (read replica) owns OLTP read scaling. Neither tier handles wide time-series scans efficiently — sparse-row queries over month-long windows on un-partitioned tables exhibit catastrophic worst-case planner behavior (sequential scan of multi-billion-row table; no partition pruning; no hypertable chunk exclusion).

ADR-0193 places ClickHouse as the OLAP analytics warehouse for tenant-facing dashboards. ClickHouse handles wide time-series **aggregates** well, but loses to TimescaleDB on:

1. **Sparse high-cardinality time-series.** Per-tenant per-asset per-metric (e.g., 100K assets × 50 metrics × 1Hz sample = 5M series per tenant). ClickHouse MergeTree handles this but each metric becomes a row with a label string; planner overhead climbs.
2. **OLTP-style row-level updates on time-series rows.** TimescaleDB supports `UPDATE` and `DELETE` on hypertable rows with the same semantics as plain Postgres. ClickHouse is append-only with `ALTER TABLE ... DELETE` being expensive.
3. **Continuous aggregates with incremental refresh.** TimescaleDB's continuous aggregates (community-edition incremental refresh) maintain rolling rollups with near-real-time freshness.
4. **Postgres-wire compatibility.** Existing tooling, ORMs, JDBC/ODBC, pgcat connection pooling (ADR-0179-postgres-connection-pooling-pgcat) all just work. ClickHouse needs a separate client + driver.

Hyperscaler practice for tenant-facing time-series at oyatie's shape:

- **Stripe** — Postgres + TimescaleDB extension for tenant-facing usage metrics dashboards (within Stripe's billing surface); ClickHouse for internal analytics. The split mirrors what this ADR establishes.
- **Datadog** — internally uses Cassandra + custom storage for SRE telemetry, but third-party telemetry-storage-as-a-product offerings increasingly converge on TimescaleDB or InfluxDB for tenant-facing analytics.
- **Robinhood, Uber Eats, Toyota Connected** — public references for TimescaleDB at scale for tenant-visible time-series.

oyatie's tenant-facing time-series workload classes:

| Workload | Example | Cardinality | Latency budget |
|---|---|---|---|
| Workflow execution metrics | per-run duration, success/fail rate, per-step time | per-tenant × per-workflow × per-step | <300ms p99 |
| Business KPIs | tenant's MRR by cohort by day; conversion funnels | per-tenant × per-KPI × hour | <500ms p99 |
| Asset telemetry | IoT-like per-device samples for industrial verticals | per-tenant × per-device × per-metric × 1Hz | <1s p99 |
| User-visible audit timelines | per-tenant security event timeline | per-tenant × per-event-type × second | <300ms p99 |

These are **distinct from**:

- **Ops / SRE telemetry** — Prometheus 3.12 hot path + Grafana Mimir 3.0 long-retention path per ADR-0186.
- **Tenant-facing wide aggregates** — ClickHouse per ADR-0193 (the analytics-warehouse slot).
- **Event-search audit logs** — ClickHouse per ADR-0193 (audit-log query slot).

## Decision

Oyatie adopts **TimescaleDB 2.26.x Community Edition (Apache-2.0)** as a Postgres 18 extension installed onto the existing Tier 1 Postgres OLTP cluster (per ADR-0184), opt-in per µservice via manifest. TimescaleDB community-edition v2.26 supports Postgres 18 since v2.23 (March 2026 release line).

### Scope (in-scope features — Apache-2.0 community only)

| Feature | Edition | In-scope? |
|---|---|---|
| Hypertables (auto-partitioned tables) | Community Apache-2.0 | YES |
| `create_hypertable()` API | Community Apache-2.0 | YES |
| Chunk auto-creation by time + space | Community Apache-2.0 | YES |
| Continuous aggregates (CAGGs) | Community Apache-2.0 | YES |
| Continuous aggregate refresh policies | TSL — partial | LIMITED (community provides the materialized view; explicit `CALL refresh_continuous_aggregate()` invocation OK; automated background refresh is TSL — oyatie schedules refresh via per-µservice worker, not TSL) |
| Retention policies (`add_retention_policy`) | TSL — automated | NO — oyatie implements retention via per-µservice worker calling `DROP CHUNKS` on schedule, NOT via TimescaleDB job scheduler |
| Compression | TSL | NO — out-of-scope; oyatie uses tiered storage (CSI cold-class per ADR-0161) for cold-tier hypertable storage |
| Hyperfunctions (advanced time-series functions) | TSL | NO |
| Tiered storage (TimescaleDB-managed S3 tier) | TSL | NO — oyatie uses Postgres native partitioning + CSI cold-class |

### TSL component fence (license enforcement)

The `oya-shared-timeseries-kernel` adapter binds **only** to the in-scope feature surface above. The `oya-check-license-policy` lane (existing) verifies that no SQL fragment in oyatie's codebase invokes TSL-only function names (e.g., `add_retention_policy`, `add_compression_policy`, `policy_compression`, hyperfunctions). The check is BLOCKER day-1 for this surface.

### Hypertable patterns — chunk sizing

Per-µservice hypertable DDL is generated by the kernel; chunk-size policy:

| Cardinality class | Chunk interval | Rationale |
|---|---|---|
| Low (≤1K series per tenant) | 7 days | Few chunks; planner exclusion easy |
| Medium (1K–100K series) | 1 day | Balance chunk count vs scan range |
| High (>100K series) | 6 hours | Tight chunk pruning for sparse queries |
| Very high (>1M series — rare for tenant-facing) | 1 hour | Aggressive pruning; runbook flag for capacity team |

Per-µservice manifest declares `data.timeseries.hypertables[].chunk_interval` allowing override. Default per-cardinality-class lives in the kernel.

### Continuous aggregates — refresh strategy

Community-edition continuous aggregates ship the materialized view; refresh-policy automation is TSL. oyatie's workaround:

1. The kernel-emitted DDL creates the continuous aggregate view (`CREATE MATERIALIZED VIEW ... WITH (timescaledb.continuous)`).
2. A per-µservice **refresh worker** (Rust binary; reuses the canonical worker scaffold) invokes `CALL refresh_continuous_aggregate(view_name, start, end)` on a per-µservice-declared interval.
3. Refresh interval pinned per CAGG via manifest `data.timeseries.continuous_aggregates[].refresh_interval` (e.g., `"5m"`).
4. Worker SLO: refresh lag ≤ 2× interval; otherwise pages ops-sre-reliability.

This is the canonical "do not depend on TSL" pattern; it costs ~30 lines of worker code per µservice and is documented at `docs/standards/timeseries-continuous-aggregate-refresh.md` (created in this batch's follow-on).

### Retention — per-µservice worker, not TSL

`add_retention_policy` is TSL-only. oyatie's equivalent: per-µservice retention worker calls `SELECT drop_chunks(hypertable_name, older_than => now() - interval '<retention>')` on a per-µservice-declared schedule. The retention worker is a sibling of the refresh worker; both ship in the canonical Rust worker scaffold.

### Multi-tenancy

Hypertables are per-µservice; per-tenant isolation enforced via Tier-1 Postgres row-level security (per ADR-0184). Per-tenant resource quotas (ADR-0155) project as Postgres role-level `statement_timeout` + pgcat connection-pool limit per tenant.

### Backup + DR

TimescaleDB hypertables are Postgres tables; standard Postgres backup (pg_basebackup + WAL archiving) covers them. Per ADR-0152 RPO/RTO canonical: RPO ≤ 24h for tenant-facing time-series (recoverable from canonical entity stream via re-emission if RPO breached).

### Why not just ClickHouse for everything?

The mission's user-directive scope-question is real. ClickHouse does handle most of what TimescaleDB does. The cases where ClickHouse loses:

1. **Sparse-time-series with row-level updates.** Tenant edits a workflow run's outcome → must update the row, not append a new event. ClickHouse `ALTER TABLE DELETE` is expensive; TimescaleDB hypertable UPDATE is standard.
2. **Postgres-wire compatibility.** Existing µservice repositories already speak Postgres; TimescaleDB is a `CREATE EXTENSION`. ClickHouse is a separate driver, separate connection pool, separate SQL dialect.
3. **Transactional consistency with OLTP rows.** A workflow run's metadata row (Tier 1 Postgres) + its time-series metrics (TimescaleDB hypertable in the same Postgres) participate in the same transaction. ClickHouse cannot.

For µservices whose workload is **append-only + wide-aggregate-only + no row update + no OLTP transaction with metrics**, ClickHouse is the right answer (per ADR-0193). For µservices with the row-update / transactional / Postgres-native shape, TimescaleDB is the right answer.

The manifest field `data.timeseries.enabled` opts a µservice into TimescaleDB; default OFF. The check `oya-check-vendor-lockin-discipline` advisory verifies that no µservice uses TimescaleDB and Postgres `time` column types for the same data class — collision indicates indecision and should be normalized.

## Alternatives considered

### (a) **CHOSEN: TimescaleDB 2.26 Community Edition (Apache-2.0) as a PG18 extension**

- **Pros:** Apache-2.0 community core (license-clean for the scoped surface); rides on the existing Tier 1 Postgres fleet (no new substrate to deploy); Postgres-wire compatible (existing tooling just works); transactional with adjacent OLTP rows; hypertable + continuous-aggregate primitives are state-of-the-art for the sparse time-series shape; mature at multi-billion-row scale per Stripe / Uber Eats references.
- **Cons:** TSL component fence is non-trivial — must avoid `add_retention_policy`, `add_compression_policy`, hyperfunctions, tiered storage; oyatie reimplements refresh + retention via per-µservice workers (~30 lines per µservice; documented standard). Mitigation: kernel exposes only in-scope surface; CI gate forbids TSL function names.
- **Accepted.**

### (b) InfluxDB 3.x — REJECTED

- **Pros:** purpose-built; Rust-native (InfluxDB 3 IOx engine in Rust); strong query language for time-series.
- **Cons:** InfluxDB 3 commercial-tier moves are concerning — InfluxData's posture on the open-source edition vs the cloud-only commercial edition has shifted multiple times (v3 OSS shipped late + with reduced feature parity to v3 cloud). Storage engine (IOx with Parquet on object store) is interesting but cuts against transactional consistency with adjacent Postgres rows. Distinct ops surface from the Postgres fleet.
- **Rejected** on commercial-tier uncertainty + ops-surface duplication.

### (c) VictoriaMetrics — REJECTED for tenant-facing (right for ops)

- **Pros:** Apache-2.0; better than Prometheus for long-retention; well-engineered.
- **Cons:** designed for ops/SRE metric storage (high-write, append-only, label-set query model); not optimized for tenant-facing time-series with row updates + transactional consistency requirements.
- **Rejected** for tenant-facing slot. VictoriaMetrics remains valid as an alternate / future-supplement for Grafana Mimir per ADR-0186 (ops/SRE metrics), not this slot.

### (d) QuestDB — REJECTED

- **Pros:** very fast ingest; SQL-native.
- **Cons:** single-node primary architecture; replication for tenant SLO requires retrofit (read replicas exist but the HA story is immature for multi-cell oyatie); Java-based (operator-skill mismatch with Rust-first fleet); smaller community.
- **Rejected** on HA + ops profile.

### (e) ClickHouse-only (drop TimescaleDB entirely) — REJECTED

- **Pros:** one substrate; less to operate.
- **Cons:** loses row-level update + transactional consistency with adjacent OLTP rows + Postgres-wire compatibility for existing repositories.
- **Rejected** for the workflow-execution-metrics-class workload. ClickHouse retains its slot per ADR-0193 for the wide-aggregate slot.

### (f) Postgres native partitioning + time-bucketed indexes (no TimescaleDB extension) — REJECTED for hypertable shape

- **Pros:** zero extension; pure stock Postgres.
- **Cons:** manual partition management; no `time_bucket()` continuous-aggregate primitive; planner exclusion isn't as aggressive as TimescaleDB's chunk pruning. The TimescaleDB extension is ~Postgres-native + a thin layer of hypertable-specific optimizations; reinventing it is gold-plating.
- **Rejected** on reinvention cost.

## Consequences

### Positive

1. **One Postgres-wire substrate for OLTP + time-series.** Same connection pool, same auth, same tooling, same transaction semantics.
2. **Apache-2.0 community core.** License-clean for the scoped surface; TSL fence enforced by CI.
3. **Per-µservice opt-in.** µservices without time-series needs pay nothing; manifest field gates extension creation.
4. **Transactional consistency.** Workflow run's metadata + metrics participate in the same Postgres transaction.
5. **Continuous aggregates** maintain rolling rollups without external stream-processing infrastructure.

### Negative

1. **TSL component fence requires CI vigilance.** Mitigation: `oya-check-license-policy` (existing crate) regex-checks for TSL function names in SQL fragments; CI BLOCKER day-1 for this surface.
2. **Reimplements refresh + retention via per-µservice workers.** Mitigation: ~30 LOC per µservice; canonical worker scaffold; documented standard at `docs/standards/timeseries-continuous-aggregate-refresh.md`.
3. **Hypertable chunk sizing is per-workload-class.** Mitigation: defaults per cardinality class declared in kernel; per-µservice manifest override.

### Operational

1. Per-µservice manifest declares `data.timeseries.enabled: true` + `data.timeseries.hypertables[]` with `(table_name, time_column, space_column?, chunk_interval, retention)`.
2. The `oya-check-license-policy` lane is BLOCKER day-1 for TSL function-name patterns in `microservices/**/*.sql` and Rust string literals.
3. SLO: hypertable INSERT p99 ≤ 30ms (matches Postgres OLTP write SLO); query p99 ≤ 300ms for typical tenant-facing dashboards.
4. Capacity: chunk count per hypertable monitored; alerts when chunk count > 10K (planner-overhead threshold).

## In-house roadmap

Per the in-house tech stack policy (user directive 2026-05-18) — "wherever possible, support in-house tech stack like AWS / Google / Microsoft / Oracle do" — TimescaleDB's posture is **KEEP** (not vendor-replaceable in the same sense as Milvus or ClickHouse):

### Why KEEP, not in-house replacement

- **Postgres is the substrate.** Per ADR-0184 Tier 1, Postgres 18.4 is oyatie's canonical OLTP substrate. Replacing Postgres is out of scope for any plausible roadmap.
- **TimescaleDB is a Postgres extension.** It is not a separate engine; it is a `CREATE EXTENSION` shipped against the canonical OLTP fleet. Replacing the extension means writing per-µservice hypertable partition management + continuous-aggregate logic — which is functionally rewriting a slice of TimescaleDB. The build-vs-buy posture per ADR-0014 favors keeping the Apache-2.0 community core.
- **Community core is Apache-2.0.** As long as the community core stays Apache-2.0, oyatie's exposure is bounded to the in-scope feature surface declared in this ADR.

### TSL fence as the durable risk control

The in-scope vs out-of-scope feature surface declared in §"Scope" + §"TSL component fence" of this ADR is the canonical risk control. `oya-check-license-policy` enforces it CI-BLOCKER day-1. As long as oyatie binds only to the Apache-2.0 community surface, no commercial-license exposure exists.

### Phase 2 trigger condition (if it ever activates)

A Phase 2 in-house replacement activates **only** if:

1. TimescaleDB community-edition licensing changes (e.g., the Apache-2.0 community core is relicensed) — the 2024-Redis-event class of risk.
2. Postgres extension support for TimescaleDB drops Postgres 18 / 19 / 20 within a maintenance window oyatie cannot accommodate.

If either condition trips, the in-house path is per-µservice hypertable management built on stock Postgres native partitioning + custom worker for chunk lifecycle + custom continuous-aggregate materialization (effectively oyatie's own thin extension or extension-equivalent worker). This is documented as a contingency, not a commitment.

### Industry parallels

- **AWS Aurora** — managed Postgres at AWS; TimescaleDB-on-Aurora isn't fully supported (Aurora has its own extension surface) but Aurora's time-series posture relies on partitioning + their own internal optimizations. AWS Timestream is a separate time-series service.
- **Google Cloud SQL for Postgres** — supports TimescaleDB community-edition extension.
- **Microsoft Azure Database for Postgres** — supports TimescaleDB community-edition extension.
- **Oracle** — Oracle's time-series capability lives inside the Oracle Database; not a separate engine.

The Apache-2.0 community-edition TimescaleDB lane is the cross-hyperscaler-aligned position. Phase 0 ships now; no Phase 2 is planned absent the trigger conditions above.

## Rollback

- **Drop the extension.** `DROP EXTENSION timescaledb CASCADE` removes hypertable wrappers; underlying chunk tables remain as plain Postgres tables. Read access continues; write requires reverting per-µservice repository code to plain-Postgres inserts.
- **Per-µservice opt-out.** Set `data.timeseries.enabled: false`; redeploy; the µservice no longer creates new hypertables; existing hypertables remain accessible until explicit migration.

## References

- TimescaleDB — https://github.com/timescale/timescaledb ; Apache 2.0 (community core).
- TimescaleDB 2.26 release — https://github.com/timescale/timescaledb/releases (released 2026-03-24).
- TimescaleDB Postgres 18 support — TimescaleDB v2.23+ supports Postgres 18; v2.26 is current.
- TimescaleDB Community vs TSL — https://docs.timescale.com/use-timescale/latest/community/
- Continuous aggregates — https://docs.timescale.com/use-timescale/latest/continuous-aggregates/
- ADR-0042 — observability stack (distinct concern from this ADR).
- ADR-0043-secrets-management-openbao-and-hsm-per-cell — OpenBao SecretReference for Postgres credentials.
- ADR-0131-per-microservice-flat-layout — per-µservice Helm overlay at `microservices/<ms>/iac/helm/timescaledb-extension/`.
- ADR-0145 — inter-microservice communication reform.
- ADR-0152 — RPO / RTO canonical.
- ADR-0153 — outbox pattern (time-series events flow through outbox).
- ADR-0155 — per-tenant resource quotas (project as per-role statement_timeout).
- ADR-0179-postgres-connection-pooling-pgcat — pgcat fronts the TimescaleDB-extended Postgres cluster.
- ADR-0184 — storage tier layering (TimescaleDB is a Tier 1 add-on, not a new tier).
- ADR-0186 — observability backplane layering (ops/SRE metrics path; distinct from this ADR).
- ADR-0192 — vector database canonical (Milvus).
- ADR-0193 — OLAP analytics warehouse (ClickHouse).
- ADR-0195 — stream processing tier.
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098.
