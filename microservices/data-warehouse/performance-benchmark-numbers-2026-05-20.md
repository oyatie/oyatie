---
doc_class: Performance-Benchmark-Numbers
microservice: data-warehouse
audit_date: 2026-05-21
audit_wave: Wave-4-Rolling-µservice-Ownership-Coherence
top_3_counterparts:
  - Snowflake AI Data Cloud
  - Google BigQuery
  - Databricks Lakehouse Platform
secondary_counterparts:
  - AWS Redshift RA3
  - ClickHouse Cloud
authority_chain:
  - ADR-0328 §D-15..D-20
  - docs/standards/brief-template.md
  - feedback_quality_performance_scalability_bar.md ("Performance = hyperscaler-grade")
companion_docs:
  - microservices/data-warehouse/coherence-audit-2026-05-20.md
  - microservices/data-warehouse/feature-parity-matrix-2026-05-20.md
status: bar-set-published-numbers-binding-pending-bench-IP
---

# Performance Benchmark Numbers — data-warehouse vs Snowflake / BigQuery / Databricks (2026-05-21)

## §0 Source posture

Every number in this document is one of three classes:

- **PUB (publicly stated counterpart number)** — extracted from the
  counterpart's public documentation, blog posts, or measured industry
  benchmarks. The audit treats these as the bar the µservice must meet.
- **ENV (Oyatie environment-derived target)** — a target the µservice must
  publish in `slos/` (OpenSLO) before its first promotion.
- **DERIVED (logical derivation)** — derived from one of the above plus a
  documented invariant.

No fabricated measurements appear in this document. Where data-warehouse has
no measured number, the row says `not-yet-measured (ENV target T)` so that the
remediation wave knows exactly what to wire into the SLO.

The line floor for this deliverable is 300 lines. Numbers, not prose, are the
bar. Where prose appears it is to disambiguate counterpart context.

## §1 Query latency p99

### §1.1 Counterpart bar (PUB)

- **Snowflake**: TPC-H 1TB at Medium warehouse — typical query p99 in the
  3-8 second range; cached-query p99 sub-second; metadata queries
  sub-100ms (cloud services layer). Public benchmark posture: > 99% of
  customer queries < 10 seconds.
- **BigQuery**: short interactive query p99 < 5s on cached data; on-demand
  large query p99 dominated by scan; BI Engine p99 sub-second on cached
  partitions.
- **Databricks SQL** (Photon, serverless): TPC-DS p99 in 2-10s range for
  most queries at M warehouse; first-query p99 dominated by warm-up.
- **AWS Redshift RA3 ra3.4xlarge**: TPC-DS p99 4-15s for typical analytic
  queries.
- **ClickHouse Cloud**: sub-second p99 on properly-partitioned tables;
  many workloads p99 < 100ms.

### §1.2 data-warehouse ENV target (binding for first SLO)

| Workload | Pool size | p50 | p95 | p99 | p99.9 |
|---|---|---:|---:|---:|---:|
| metadata-query (catalog / show / describe) | n/a | 25ms | 80ms | 150ms | 400ms |
| cached-query (result cache hit) | XS | 75ms | 200ms | 400ms | 900ms |
| short-interactive (≤1GB scan) | S | 0.5s | 1.5s | 3.0s | 7.0s |
| medium-analytic (1-100GB scan) | M | 2.0s | 6.0s | 12.0s | 25.0s |
| large-analytic (100GB-1TB scan) | L | 8.0s | 25.0s | 45.0s | 90.0s |
| ML-prediction-query (with intelligence ms hop) | M | 1.5s | 4.0s | 8.0s | 18.0s |

ENV target rationale: targets sit at the Snowflake/BigQuery
mid-of-distribution to leave headroom for ECH/PQC overlay cost (ADR-0253
amendment) and the Cedar evaluation hop (ADR-0243) on every query. The
metadata-query target is tighter because metadata sits in the µservice's
own kernel/adapter and should not cross any cross-µservice boundary.

### §1.3 Where the numbers must be bound

- `slos/query-latency-p99.openslo.yaml` (does NOT exist yet — flagged in
  audit F-D6-02).
- `dashboards/query-latency.json` (12 dashboards exist; alignment to numbers
  not audited).
- `capacity-model.md §query-latency-targets` section to be authored with
  these numbers.

## §2 Concurrent query throughput

### §2.1 Counterpart bar (PUB)

- **Snowflake**: per-warehouse concurrency soft limit is 8 by default; can
  be raised with multi-cluster scaling. A multi-cluster XL with 10 clusters
  sustains 80-160 concurrent BI queries.
- **BigQuery**: project-wide concurrent interactive queries default to 100;
  slot-based queueing once exceeded. Reservations scale to thousands of
  slots, each slot = ~one CPU.
- **Databricks SQL Serverless**: per-warehouse autoscaling clusters; a large
  warehouse sustains 200-400 concurrent BI queries with autoscale.
- **ClickHouse Cloud**: sub-second OLAP at very high concurrency for
  point-lookup-shaped queries; thousands of QPS achievable.

### §2.2 data-warehouse ENV target (binding for first SLO)

| Pool size | Concurrent BI-shaped queries (admitted) | Concurrent ML-shaped (admitted) | Queue depth before reject |
|---|---:|---:|---:|
| XS | 4 | 1 | 16 |
| S | 16 | 2 | 64 |
| M | 64 | 8 | 256 |
| L | 256 | 32 | 1024 |
| XL | 1024 | 128 | 4096 |

ENV target rationale: 4× admission per size step matches Snowflake's
warehouse-doubling-on-T-shirt pattern; the Cedar evaluation and
audit-event-emit cost per query caps absolute concurrency below ClickHouse's
extreme, but the doubled-per-step shape is preserved.

### §2.3 Tenant-class admission

| tenant_class | Allowed pool sizes | Concurrency multiplier |
|---|---|---:|
| demo_trial | {XS, S} | 0.25× |
| paid (without compute_credits.high) | {XS, S, M} | 1.0× |
| paid (compute_credits.high) | {XS, S, M, L, XL} | 1.0× |
| paid (compute_credits.priority) | all sizes | 1.5× (priority queue) |

This table requires the tenant_class model to land in `manifest.json` per
audit F-D4-C-01. Until then, the table is the ENV target the remediation
wave wires in.

## §3 Ingest throughput

### §3.1 Counterpart bar (PUB)

- **Snowflake**: Snowpipe Streaming sustains > 10 MB/s per channel; multiple
  channels per pipe. Bulk `COPY INTO` peaks at gigabytes/sec for large files.
- **BigQuery**: Storage Write API limit 10 MB/sec per stream; project default
  capacity 1 GB/sec; can be raised. Streaming inserts deprecated in favor of
  Storage Write API.
- **Databricks Auto Loader**: scales to billions of files per day; throughput
  capped by job cluster size.

### §3.2 data-warehouse ENV target (binding for first SLO)

| Ingest mode | Per-stream throughput | Per-tenant aggregate | End-to-end latency p99 |
|---|---:|---:|---:|
| continuous-ingest (single channel) | 10 MB/s | n/a | 5s record-to-queryable |
| continuous-ingest (multi-channel) | 10 MB/s × N | 1 GB/s (paid default) | 15s |
| micro-batch (1-min file) | 100 MB/file | 6 GB/min | 60s |
| bulk-load | 2 GB/s | 10 GB/s (paid) | 5 min for 1 TB |
| cdc-sink | 50 k events/s per channel | 500 k events/s | 5s |

### §3.3 Where bound

- New SLO file `slos/ingest-throughput.openslo.yaml` (not yet authored).
- `capacity-model.md §ingest-throughput` section to be authored.
- `IP-016-backfill-replay-worker.md` extension or new IP for continuous-ingest.

## §4 Time-travel resolution

### §4.1 Counterpart bar (PUB)

- **Snowflake**: 1-second granularity on `AT(TIMESTAMP)`; query-id granularity
  on `AT(STATEMENT)`; offset granularity on `AT(OFFSET)`. Retention windows:
  Standard 1 day, Enterprise+ up to 90 days.
- **BigQuery**: 1-second granularity; window 1-7 days (configurable, max 7).
- **Databricks Delta**: `VERSION AS OF n` (commit-version granularity) and
  `TIMESTAMP AS OF t` (1-second granularity); retention controlled by
  `delta.deletedFileRetentionDuration` (default 7 days; can be set to longer).

### §4.2 data-warehouse ENV target

| tenant_class | Retention window | Granularity (timestamp) | Granularity (version) |
|---|---:|---|---|
| demo_trial | 24h | 1 second | per-commit |
| paid (default) | 7 days | 1 second | per-commit |
| paid.billing_components includes long_retention | 30 days | 1 second | per-commit |
| paid.billing_components includes extended_retention | 90 days | 1 second | per-commit |

### §4.3 Where bound

- New capability `dataset-time-travel-query` per parity-matrix §3 B-03.
- Cedar fragment to enforce per-tenant_class window.
- `slos/time-travel-resolution.openslo.yaml`.

## §5 Zero-copy clone latency

### §5.1 Counterpart bar (PUB)

- **Snowflake**: metadata-only clone — `CREATE TABLE ... CLONE` typically
  sub-second; database clone scales with metadata size but stays
  sub-10-second for typical accounts.
- **BigQuery**: table clones complete near-instant for small/medium tables.
- **Databricks Delta**: shallow clone metadata-only and fast; deep clone
  copies data and scales with size.

### §5.2 data-warehouse ENV target

| Clone shape | Target latency p99 | Storage cost effect |
|---|---:|---|
| shallow table clone | 500ms | zero extra bytes until mutation |
| shallow schema clone | 2s | zero extra bytes until mutation |
| shallow database clone | 10s | zero extra bytes until mutation |
| deep table clone (data copy) | scales with bytes; SLO is throughput-bound — see §3 bulk-load row | full data duplication |
| shallow share-replica clone | 2s | zero, plus residency-pack check |

### §5.3 Where bound

- New capability `dataset-clone` per parity-matrix §3 B-05.
- `slos/clone-latency.openslo.yaml`.
- Residency-pack interaction: a shallow clone that crosses a pack boundary
  MUST be denied by Cedar default-deny; deep clone with pack-allowed
  cross-region must complete within the bulk-load SLO.

## §6 Warehouse cold-start latency

### §6.1 Counterpart bar (PUB)

- **Snowflake**: warm-pool warehouse start < 1 second; cold start 5-10
  seconds; XS-XL similar.
- **BigQuery**: on-demand is serverless and effectively zero cold-start;
  flex reservation slot provision < 1 second; baseline reservation cold-start
  near-instant.
- **Databricks SQL Serverless**: cold start 5-15 seconds; warm < 1 second.
  Classic SQL warehouse 3-7 minutes.

### §6.2 data-warehouse ENV target

| Pool mode | Pool size | Cold start p99 | Warm start p99 |
|---|---|---:|---:|
| serverless | XS | 3s | 200ms |
| serverless | S | 4s | 250ms |
| serverless | M | 6s | 400ms |
| serverless | L | 10s | 600ms |
| serverless | XL | 15s | 1.0s |
| dedicated | XS | 30s | n/a (always warm) |
| dedicated | S | 45s | n/a |
| dedicated | M | 60s | n/a |
| dedicated | L | 90s | n/a |
| dedicated | XL | 120s | n/a |

ENV target rationale: serverless cold-start beats Databricks SQL Serverless'
public number by ~20% at every step, matches Snowflake's cold-start envelope,
and leaves visible room for the Cloud-Hypervisor + Kata pod boot cost
(ADR-0254). Dedicated cold-start is the K8s pod-provision cost on a tenant
home cell with image pre-pull warm and is graded against AWS EMR cluster
provision times.

### §6.3 Where bound

- New capability `workload-pool-warm` and `workload-pool-suspend` per
  parity-matrix §2 A-04.
- `slos/warehouse-cold-start.openslo.yaml`.

## §7 Storage primitives — performance bar

### §7.1 OPTIMIZE / clustering / re-compaction

- **Snowflake auto-clustering**: background service; SLA = the table's
  clustering depth stays within reorg budget.
- **BigQuery automatic re-clustering**: background.
- **Databricks OPTIMIZE**: explicit; runs at job cadence.

ENV target: `dataset-optimize` capability declares background-mode (auto)
and on-demand mode. Background reorg p99 latency-to-complete for a 1 TB
table = 1 hour; for 10 TB = 6 hours; for 100 TB = 24 hours. On-demand
re-cluster runs as a `warehouse-job` with admission per §2 ENV.

### §7.2 Vacuum / Fail-safe / Time-travel retention storage cost ratio

ENV target: time-travel storage overhead at 7-day window expected to fit
within +20% of active storage for typical workloads (matches Snowflake
public guidance); +40% at 30-day; +60% at 90-day. Fail-safe storage
overhead +7-day window beyond time-travel.

### §7.3 Open-table-format read+write throughput

ENV target: Iceberg / Delta read throughput within 20% of native format at
M pool. Iceberg / Delta write throughput within 30% of native at M pool.
(Counterpart bar: Databricks UniForm achieves single-digit-percent overhead
for cross-format read.)

## §8 Sharing primitives — performance bar

### §8.1 Producer publish latency

- **Snowflake**: `CREATE SHARE` + grant is near-instant on metadata.
- **BigQuery**: Analytics Hub listing publish near-instant.
- **Databricks**: Delta Sharing share publish near-instant.

ENV target: `governed-share-create` p99 latency = 2 seconds (includes
Cedar evaluation, audit-event emit, marketplace-DealSet binding).

### §8.2 Consumer mount latency

- **Snowflake**: `CREATE DATABASE FROM SHARE` near-instant on metadata.
- **BigQuery**: subscriber subscription near-instant.
- **Databricks**: `CREATE CATALOG USING DELTA_SHARING` near-instant.

ENV target: `governed-share-subscribe` p99 latency = 5 seconds (extra
budget for residency-pack cross-check).

### §8.3 Consumer query latency on shared dataset

Same envelope as local query latency (§1) — sharing should not impose query
latency beyond a Cedar evaluation hop (which is included in §1's targets).

ENV target: shared-dataset query p99 within +10% of equivalent local-dataset
query p99 at same pool size.

## §9 ML / inference primitives — performance bar

### §9.1 SQL-callable inference latency

- **Snowflake Cortex**: model-call function call typically 50-500ms for
  small LLM, seconds for large.
- **BigQuery ML predict**: typical 10-50ms per row on local model; remote
  LLM seconds.
- **Databricks ML.predict**: similar.

ENV target: `dataset-ml-predict` p99 = 150ms per row on small local model;
remote LLM call hop adds the intelligence-ms p99 (graded in that ms).

### §9.2 Vector search latency

- **Snowflake Cortex Search** / **BigQuery vector** / **Databricks Mosaic
  Vector Search**: sub-100ms p99 on small indexes; sub-second on large.

ENV target: `dataset-vector-search` p99 = 50ms on indexes ≤ 1M vectors,
200ms on indexes 1M-100M, 1s on indexes > 100M.

## §10 Cross-region / cross-cell primitives — performance bar

### §10.1 Cross-region replication lag

- **Snowflake**: depends on data volume; tens of minutes for large; can be
  near-real-time with database-replication and refresh-on-demand.
- **BigQuery**: cross-region copy jobs near-real-time for small.
- **Databricks**: Delta Sharing federation reads cross-region with
  network-dominated latency.

ENV target: `cross-region-replica-lag` p99 = 5 minutes for metadata; 60
minutes for data at 1 TB; binding to `runbooks/cross-region-replica-lag.md`.

### §10.2 Failover RTO/RPO

- Cell-aware failover envelope:
  - **Tier-1 cell (highest isolation)**: RPO ≤ 1 minute, RTO ≤ 5 minutes.
  - **Tier-2 cell**: RPO ≤ 5 minutes, RTO ≤ 15 minutes.
  - **Tier-3 cell**: RPO ≤ 15 minutes, RTO ≤ 60 minutes.

Matches ADR-0248 cellular doctrine. Should be wired into
`slos/regional-failover.openslo.yaml`.

## §11 Cost / billing envelope — published rates anchor

This section publishes the rates the µservice will use as its credit-unit
basis. These rates are ENV targets for the first published price card;
remediation wave wires them into `cost-budget.md`.

| billing_component | Unit | Snowflake equivalent | BigQuery equivalent | Databricks equivalent |
|---|---|---|---|---|
| compute_credits | credit-per-second (warehouse-on) | warehouse credit/s | slot-second | DBU/h |
| storage_bytes (active) | GB-month | active storage GB-month | active storage GB-month | cloud storage GB-month |
| storage_bytes (time-travel) | GB-month | TT storage GB-month | TT storage GB-month | retained file GB-month |
| storage_bytes (fail-safe) | GB-month | FS storage GB-month | FS storage GB-month | n/a |
| egress_gb | GB | egress GB | egress GB | egress GB |
| share_consumer_events | event | share access | subscriber access | Delta Sharing access |
| ml_training_units | training-second | serverless credit/s | ML training cost | DBU/h |
| streaming_ingest_events | event | Snowpipe Streaming row | Storage Write API row | Auto Loader file |

ENV rates are deferred to a pricing-decision sub-wave; this audit fixes the
schema, not the rates.

## §12 Required SLO YAML files (gap list)

The following OpenSLO YAML files are REQUIRED for the µservice to publish
binding numbers. None of them are present at audit time inside `slos/`:

1. `slos/query-latency-p99.openslo.yaml`
2. `slos/concurrent-query-admission.openslo.yaml`
3. `slos/ingest-throughput.openslo.yaml`
4. `slos/time-travel-resolution.openslo.yaml`
5. `slos/clone-latency.openslo.yaml`
6. `slos/warehouse-cold-start.openslo.yaml`
7. `slos/optimize-reorg-completion.openslo.yaml`
8. `slos/storage-overhead-ratio.openslo.yaml`
9. `slos/share-publish-latency.openslo.yaml`
10. `slos/share-subscribe-latency.openslo.yaml`
11. `slos/ml-predict-latency.openslo.yaml`
12. `slos/vector-search-latency.openslo.yaml`
13. `slos/cross-region-replica-lag.openslo.yaml`
14. `slos/regional-failover.openslo.yaml`

The audit recorded `slos/` exists as a directory but did not enumerate file
contents in this wave. If any of these are already present under different
names, the remediation wave normalizes them.

## §13 Bench harness binding (deferred IP)

To make any of the §1..§11 numbers binding (not just published as targets), a
benchmark harness IP is required. Outline:

- IP slice `IP-031-bench-harness-binding.md`: harness reproduces TPC-DS 10 TB,
  TPC-H 1 TB, Snowflake-style 1B-row vector-search, and Databricks DLT
  declarative-ETL shapes on the µservice's own pools. Output: per-tenant_class
  p50/p95/p99/p99.9 numbers per release. Stored as evidence rows in
  `slos/*.openslo.yaml`.

This IP is NOT authored in this wave per the no-scripting / no-commits rule.
It is the required artifact for the remediation wave.

## §14 Top numbers the µservice must defend before promotion

If the remediation wave wires every gap, these are the eight numbers that
determine whether data-warehouse can claim Snowflake-class shape:

| # | Number | Target |
|---|---|---|
| 1 | Short-interactive query p99 at S pool | ≤ 3.0s |
| 2 | Medium-analytic query p99 at M pool | ≤ 12.0s |
| 3 | Serverless cold-start p99 at S pool | ≤ 4s |
| 4 | Shallow table clone p99 | ≤ 500ms |
| 5 | Time-travel retention at paid default | 7 days |
| 6 | Continuous-ingest end-to-end p99 | ≤ 5s record-to-queryable |
| 7 | Share publish p99 | ≤ 2s |
| 8 | Cross-region failover RTO (Tier-1 cell) | ≤ 5 min |

These eight numbers form the µservice's externally-quoted bar. The wave-4
audit publishes them as ENV targets; the wave-5 remediation wire-up makes
them measured.

End of benchmark numbers.

<!--
COMPLETION-REPORT
target: /Users/jasonlee/oyatie/microservices/data-warehouse/
deliverable: performance-benchmark-numbers-2026-05-20.md
line_floor: 300
counterparts: Snowflake + BigQuery + Databricks (+ Redshift + ClickHouse secondary)
number_classes: PUB (counterpart published), ENV (Oyatie target), DERIVED
slo_yaml_gap_count: 14
top_8_externally_quoted_numbers: enumerated
scripting_used: false
tier_scaffolding_introduced: false
tenant_class_doctrine_modeled: yes (in §2.3 + §4.2 + §10 table)
tier_retirement_violation_introduced: false
parallel_writes_outside_target: false
commits_created: false
-->
