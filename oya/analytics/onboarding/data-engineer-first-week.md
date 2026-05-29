---
doc_class: Onboarding
microservice: analytics
persona: data-engineer + analytics-platform-engineer
related_adrs: [ADR-0193, ADR-0184, ADR-0195, ADR-0131, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# Data Engineer onboarding — first 5 working days on `analytics`

Audience: a new data engineer or analytics-platform engineer joining the `analytics` rotation. By Day-5 they will have: spun up a demo_trial analytics cell, ingested a tenant outbox stream, authored a Materialized View, executed the cross-tenant isolation drill, and walked a real query-budget-exhausted incident from the runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 30 min) + `ARCHITECTURE.md` § per-tenant-isolation + § ingest-pipeline (∼ 45 min).
2. Open the Grafana folder `analytics`. Identify the four primary boards — `analytics-query-latency`, `analytics-ingest-freshness`, `analytics-quota-burn`, `analytics-mv-lag`.
3. Walk the runbook index `runbooks/README.md`. The on-call runbooks are: `clickhouse-keeper-quorum-loss.md`, `mv-stale-partition.md`, `tenant-quota-exhausted.md`, `cross-region-restore.md`, `cold-tier-rehydrate-storm.md`, `pulsar-consumer-lag-spike.md`, `partition-skew-rebalance.md`, `tenant-onboarding-failed.md`.
4. Sit in on Wed's data-substrate handoff. Watch how the outgoing rotation reads the past-week query-budget-burn ledger and hands the pager.

Acceptance: you can sketch on a whiteboard the request path: tenant API → analytics-query-gateway → Cedar gate → ClickHouse Distributed engine → per-shard MergeTree → optional cold-tier S3 fetch.

## Day 2 — demo_trial analytics cell bootstrap

```sh
cargo run -p oya-dev-cli -- analytics bootstrap \
    --tenant-class demo_trial \
    --cell drill-syd-1 \
    --keeper-nodes drill-cks-1,drill-cks-2,drill-cks-3 \
    --clickhouse-nodes drill-ch-1,drill-ch-2 \
    --pulsar-endpoint pulsar://drill-pulsar-syd-1:6650 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 20 min. Watch the bootstrap log for the phases (in order): Keeper quorum elected, ClickHouse server-1 schema applied, ClickHouse server-2 replica synced, Pulsar consumer slots provisioned, MV set applied, smoke-test rollups verified.

After bootstrap, verify:

```sh
clickhouse-client --host drill-ch-1 --query "SELECT name, total_rows FROM system.tables WHERE database LIKE 'tenant_%'"
```

Should return zero rows (no tenants onboarded yet) but the system catalog should be live.

Acceptance: cluster up, you can describe the role of each Keeper Raft member from the logs.

## Day 3 — Tenant onboarding + ingest stream

Onboard a synthetic drill tenant:

```sh
cargo run -p oya-dev-cli -- analytics tenant-onboard \
    --cell drill-syd-1 \
    --tenant-id drill-acme \
    --pack us-default \
    --tenant-class paid \
    --tables outbox.workflow_event,outbox.audit_event,outbox.billing_event
```

The onboarding controller:

1. Calls Cedar `analytics::tenant_db::create` — must `allow`.
2. Issues `CREATE DATABASE tenant_drill_acme`.
3. Applies per-table schemas + per-tenant quota DDL (`CREATE QUOTA tenant_drill_acme MAX queries=100/HOUR, MAX read_rows=10_000_000/HOUR`).
4. Provisions Pulsar consumer slots for the 3 outbox topics.
5. Emits `tenant_db_created` to the audit-chain.

Verify ingest:

```sh
oya synthetic emit --tenant drill-acme --table outbox.workflow_event --rows 10000
sleep 30
clickhouse-client --host drill-ch-1 --query "SELECT count() FROM tenant_drill_acme.workflow_event"
```

Should return ~ 10000 within 30 s (demo_trial freshness budget). The audit chain should also show 1 `partition_ingested` event per shard.

Acceptance: tenant DB exists, ingest flows, quota is enforced (try emitting > quota — should see Pulsar consumer pause with `quota_exceeded`).

## Day 4 — Materialized Views + recalc-cadence

Author a Materialized View on the drill tenant. Read `decisions/ADR-AN-005-materialized-view-cadence.md` first — it explains the AggregatingMergeTree pattern + the recalc-cadence tradeoff.

```sql
-- Author the MV in the drill tenant scope
USE tenant_drill_acme;

CREATE MATERIALIZED VIEW workflow_event_hourly_rollup
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(hour)
ORDER BY (hour, workflow_id, status)
TTL hour + INTERVAL 90 DAY
AS
SELECT
    toStartOfHour(event_time) AS hour,
    workflow_id,
    status,
    countState() AS event_count_state,
    avgState(duration_ms) AS avg_duration_state
FROM workflow_event
GROUP BY hour, workflow_id, status;
```

Verify the MV catches new rows:

```sh
oya synthetic emit --tenant drill-acme --table outbox.workflow_event --rows 5000 --status mixed
sleep 60
clickhouse-client --host drill-ch-1 --query "
    SELECT hour, workflow_id, status, countMerge(event_count_state) AS events
    FROM tenant_drill_acme.workflow_event_hourly_rollup
    GROUP BY hour, workflow_id, status
    ORDER BY events DESC
    LIMIT 10
"
```

Acceptance: MV lag p99 ≤ 15 s; rollup rows match raw events within 0.1 %; you can explain why `countState()` + `countMerge()` is the right two-step (state at write time, merge at read time).

## Day 5 — Cross-tenant isolation drill + query-budget-exhausted incident walk

Read `runbooks/tenant-quota-exhausted.md` end-to-end.

Run the cross-tenant drill:

```sh
oya analytics drill cross-tenant-isolation \
    --cell drill-syd-1 \
    --attacker-tenant drill-attacker \
    --victim-tenant drill-acme
```

The drill provisions `drill-attacker` with normal credentials, then attempts:

1. Direct query against `tenant_drill_acme.workflow_event` → expected: Cedar denies (`analytics::tenant_db::select` evaluated as `attacker` against `victim` database).
2. UNION ALL across tenant DBs → expected: ClickHouse engine refuses cross-database access for non-superuser accounts.
3. Side-channel timing inference (low-cardinality predicate sweep) → expected: per-tenant quota throttles to query-budget-exhausted before useful signal is extracted.

Verify each `attempt` lands in the audit chain with `result=denied` + a Cedar attribution.

Now walk a real quota-exhaustion incident. Read `runbooks/tenant-quota-exhausted.md` step-by-step. The recovery path:

1. Identify the offending tenant from `analytics-quota-burn` Grafana panel.
2. Examine query log via `clickhouse-client … SELECT query_id, query_duration_ms, read_rows FROM system.query_log WHERE user = '{tenant_id}' ORDER BY query_duration_ms DESC LIMIT 20`.
3. Decide: ad-hoc spike or pathological query? If ad-hoc, temporary 2× burst quota; if pathological, page tenant + recommend MV.
4. Re-enable consumer slot.

Target end-to-end recovery: ≤ 15 min for the drill (production target ≤ 30 min per `slos/tenant-quota-recovery.openslo.yaml`).

Acceptance: cross-tenant attempts all denied + audited; quota-exhausted recovery path executed; you can articulate why we use Cedar at the gateway layer rather than ClickHouse-native ACLs (Cedar gives us policy-as-data + audit, ClickHouse ACLs are config-as-code without audit).

## What you've learned

- The demo_trial bootstrap profile end-to-end + the tenant-onboarding controller's contract.
- The MV authoring pattern + `*State()` / `*Merge()` two-step.
- The cross-tenant isolation invariant (Cedar + database-per-tenant; defense in depth).
- The quota-exhausted recovery runbook (the single most-likely page on this rotation).

Next week: demo_trial to paid conversion drill (3-shard cluster bring-up), paid tenant_class multi-region backup-restore drill, cold-tier-rehydrate-storm tabletop, and your first production shadow.
