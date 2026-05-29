---
doc_class: FAQ
microservice: analytics
persona: data-engineer + analytics-platform-engineer
date: 2026-05-20
doc_status: published
---

# Data Engineer FAQ — analytics

## Why ClickHouse and not Snowflake / BigQuery / Redshift / Druid / Pinot?

Per ADR-0193. Snowflake and BigQuery are managed services with per-query pricing that does not align with our multi-tenant model (we expose tenant_class plus paid billing_components). Redshift is fine for AWS-only workloads; we need bare-metal + multi-hyperscaler portability. Druid is great for time-series streaming aggregates but its query model is JSON-shaped and lacks Joins; tenants need SQL. Pinot is competitive with ClickHouse on latency at scale but its operational maturity for multi-tenant CRUD-of-tenant-database operations lags ClickHouse's by a clear margin (we surveyed 2025-Q4 Pinot multi-tenancy guides — required custom controllers; ClickHouse ships the per-database + per-quota primitives).

The patch surface we tolerate on ClickHouse: 26.3 LTS pin, Keeper not ZooKeeper, custom backup daemon (`clickhouse-backup`), the per-tenant `CREATE QUOTA` DDL applied at onboarding. That's it.

## Why does our outbox flow go through Pulsar's Kafka-protocol endpoint instead of Kafka directly?

Per ADR-0078: Pulsar is our canonical event substrate. Pulsar's Kafka-protocol layer (KoP) gives us the ClickHouse `Kafka` engine support out of the box without running a separate Kafka cluster. KoP-on-Pulsar adds ~ 2 ms tail latency over native Pulsar producers; ClickHouse `Kafka` engine reads at the consumer-batch level so the per-event tail is amortised in the batch — no practical difference at our 100 k rows/sec envelope.

## A tenant query times out at 30 s. What do I check?

In order:

1. Is the query hitting cold-tier partitions? Check `system.query_log` for `read_bytes_from_s3 > 0`. Cold queries are 3-5× slower; the tenant may need an MV or a hot-window-bounded query.
2. Is the partitioning pruning effectively? Check `EXPLAIN ESTIMATE` — if `parts` is high relative to `total parts`, the WHERE clause isn't pruning. Common cause: predicate on a non-partition column.
3. Is the tenant quota close to exhausted? Check `system.quota_usage` — if `read_rows` is near limit, ClickHouse throttles the query.
4. Is there partition skew? Check `system.parts` for parts-per-partition; a skew of > 5× between partitions starves the parallel-query plan.
5. Cross-shard parallel-replicas misconfigured? Verify `parallel_replicas_count` is set on the Distributed engine table for paid tenant_class workloads that require cross-shard query parallelism.

The escalation tree at `runbooks/query-timeout.md` enumerates the diagnostic commands per branch.

## When do I split a tenant cell vs scale the current tenant cell up?

The split-out trigger is **3 TiB hot-tier capacity used** OR **400 qps sustained at p99 > 800 ms** OR **ingest backpressure > 30 s on the Pulsar consumer**, whichever fires first. Paid tenant_class workloads with per_usage billing can use a 3-shard × 2-replica baseline when the quota envelope requires it. Do not pre-split demo_trial cells — it adds 4 nodes of ops surface before the tenant_class usage cap requires it.

## Why do we author Materialized Views as `*State()` instead of `*()`?

Because MV refresh in ClickHouse is per-insert (each insert into the source table triggers the MV `SELECT` against the new rows only — there's no full re-scan). `countState()`, `avgState()`, `quantileState()` etc. emit an intermediate aggregation state that's mergeable across MV inserts; the read-time `countMerge()`, `avgMerge()`, `quantileMerge()` finalises across all state rows. Without the `*State()` pattern, the MV would re-aggregate the same rows on every read — defeating the purpose.

Read `decisions/ADR-AN-005-materialized-view-cadence.md` § "Why AggregatingMergeTree over SummingMergeTree" for the choice of MergeTree variant.

## The MV lag dashboard shows two lines (raw-table vs MV-table); they're 20 s apart. Is that bad?

20 s is within the paid tenant_class MV budget (`slos/mv-lag.openslo.yaml` p99 ≤ 30 s). Above 60 s p99 sustained, the MV's source table is probably being inserted into faster than the MV can keep up — common cause: source table has `parts_to_throw_insert` triggering on the MV target. Fix: tune the MV target's `parts_to_throw_insert` higher (default 300; we set 600 for high-cardinality MVs) and verify `parts_to_delay_insert` is proportional (default 150; we set 300).

## When can I run a manual `ALTER TABLE … DROP PARTITION` against a production cell?

Only from the break-glass jump-host with the on-call's break-glass ClickHouse credentials — and only after declaring an incident via `oya incident open`. The audit-chain lane `analytics-manual-mutation` flags any `ALTER TABLE` that does not originate from the analytics tenant-onboarding controller or the TTL-driven background merge. The flag triggers a P2 review; if the mutation does not link to an open incident the P2 escalates to P1.

## Why are tenant queries forbidden from cross-tenant federation but ops dashboards permitted?

Per ADR-0193 § "Federation rules". Tenant queries run in the `tenant_query` Cedar principal and the policy denies cross-database access. Ops dashboards run in the `analytics_ops_aggregate` principal and the policy permits access only to the fleet-aggregate database `ops_fleet` (which is fed by per-tenant MVs that strip PII at MV-author time). Tenant queries cannot impersonate ops aggregation; ops aggregation cannot drill into tenant-specific rows. The boundary is policy-enforced + audit-emitted; defense in depth.

## What's the difference between this µservice and `observability`?

Per ADR-0184 + ADR-0186:

- `observability`: owns fleet-internal SRE telemetry (Prometheus + Mimir + Loki + Tempo). Metrics, traces, logs. SRE-facing. Retention: 90 d default; 1 y archive on opt-in.
- `analytics`: owns tenant-facing business data (workflow execution dashboards, audit-log search, billing rollups, ops-portal aggregations). OLAP shape. Tenant-facing. Retention: 90 d hot + cold tier to 7 y for audit + billing.

A tenant query for "show me my audit log for the last 7 days" hits `analytics`. An SRE query for "show me the kubelet error rate fleet-wide" hits `observability`. The boundary is the principal kind (tenant vs SRE) + the data class (tenant-visible business vs fleet-internal SRE).

## A tenant says their Mixpanel-style funnel query returns wrong numbers. What do I check?

ClickHouse has native `windowFunnel()` + `sequenceMatch()` aggregations. Common causes of wrong numbers:

1. The funnel window is wider than the time range filtered — tenants often write `WHERE event_time > now() - INTERVAL 7 DAY` but the funnel window in `windowFunnel(7200)` (2 h) is unrelated; both must be set.
2. The funnel ordering doesn't match the tenant's mental model — `windowFunnel` requires events in chronological order; if the tenant emits events out-of-order (rare; outbox enforces order per-key), you get spurious results.
3. The funnel deduplicates on the first ORDER BY key (typically `user_id`) — tenants forget this and double-count via session.

Walk the tenant through `tutorials/build-funnel-query.md`.

## Karpenter equivalent — how do we autoscale ClickHouse?

ClickHouse server nodes do not autoscale at the node level — adding a shard requires a re-partition or a re-replication step that's measured in hours. We pre-provision per the tenant_class quota matrix. Within a shard, the query-thread pool autoscales via `max_threads` and `max_concurrent_queries` per-tenant (set via `CREATE QUOTA … MAX queries=N/SECOND`). At the cell level, we add a 4th shard when the 3-shard envelope is > 70 % capacity-used for ≥ 14 d.
