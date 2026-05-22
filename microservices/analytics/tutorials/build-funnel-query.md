---
doc_class: Tutorial
microservice: analytics
persona: tenant-analyst + data-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a 5-step conversion funnel query in oyatie analytics

You will: load 50 k synthetic workflow events, author a funnel query using `windowFunnel()`, visualise the drop-off in the analytics workbook surface, then export the result to a tenant dashboard. Total time ≤ 60 minutes.

## Pre-requisites

- A tenant cell with tenant_class `paid` or `demo_trial` quota headroom, with at least one onboarded tenant.
- `oya-dev-cli` ≥ 1.42.0.
- `clickhouse-client` 26.3 installed locally (`brew install clickhouse` or `apt install clickhouse-client`).
- Cedar principal in your tenant's `analyst` role.

## Step 1 — Seed the synthetic data (≤ 10 min)

```sh
oya synthetic emit \
    --tenant drill-acme \
    --table outbox.workflow_event \
    --rows 50000 \
    --shape funnel-tutorial \
    --funnel-stages workflow_started,form_submitted,payment_intent_created,payment_succeeded,workflow_completed \
    --drop-off-rates 1.0,0.85,0.70,0.55,0.48
```

The `--shape funnel-tutorial` flag generates events with realistic drop-off across 5 stages over a 7-day window. The 5 drop-off rates control the funnel cardinality: 50k → 42.5k → 35k → 27.5k → 24k events at each stage.

Wait ~ 30 s for ingest to settle:

```sh
clickhouse-client --host <your-cell-clickhouse-endpoint> \
    --query "SELECT count() FROM tenant_drill_acme.workflow_event WHERE event_time > now() - INTERVAL 7 DAY"
```

Expected: ~ 178 500 (= 50k + 42.5k + 35k + 27.5k + 24k, since each "user" emits one event per stage they reach).

## Step 2 — Author the funnel query (≤ 15 min)

ClickHouse's `windowFunnel(t)` aggregates accept an ordered sequence of conditions and return the maximum stage each user reached within the window `t` seconds.

```sql
SELECT
    level,
    count() AS users_at_level
FROM (
    SELECT
        user_id,
        windowFunnel(86400)(
            event_time,
            event_name = 'workflow_started',
            event_name = 'form_submitted',
            event_name = 'payment_intent_created',
            event_name = 'payment_succeeded',
            event_name = 'workflow_completed'
        ) AS level
    FROM tenant_drill_acme.workflow_event
    WHERE event_time >= now() - INTERVAL 7 DAY
    GROUP BY user_id
)
GROUP BY level
ORDER BY level;
```

Notes:

- `windowFunnel(86400)` — 24-hour window; users who hit all 5 stages within 24 h count toward `level=5`.
- The inner GROUP BY produces one row per user with their max level (0 = no events, 1 = only stage 1, …, 5 = all stages).
- The outer GROUP BY rolls up to per-level counts.

Expected output (approximately, since synthetic drop-off is stochastic):

| level | users_at_level |
|---:|---:|
| 0 | (residual users with no funnel events) |
| 1 | 7 500 |
| 2 | 7 500 |
| 3 | 7 500 |
| 4 | 3 500 |
| 5 | 24 000 |

(Reading: 24 000 users completed all 5; 3 500 stopped at stage 4; 7 500 each stopped at stages 1, 2, 3.)

## Step 3 — Convert to a drop-off chart (≤ 10 min)

ClickHouse can compute the conversion rate at each stage in one query:

```sql
WITH funnel AS (
    SELECT
        windowFunnel(86400)(
            event_time,
            event_name = 'workflow_started',
            event_name = 'form_submitted',
            event_name = 'payment_intent_created',
            event_name = 'payment_succeeded',
            event_name = 'workflow_completed'
        ) AS level
    FROM tenant_drill_acme.workflow_event
    WHERE event_time >= now() - INTERVAL 7 DAY
    GROUP BY user_id
)
SELECT
    stage,
    users_reached,
    round(users_reached * 100.0 / first_value(users_reached) OVER (), 1) AS conversion_pct
FROM (
    SELECT
        1 AS stage, sumIf(1, level >= 1) AS users_reached FROM funnel UNION ALL
    SELECT 2, sumIf(1, level >= 2) FROM funnel UNION ALL
    SELECT 3, sumIf(1, level >= 3) FROM funnel UNION ALL
    SELECT 4, sumIf(1, level >= 4) FROM funnel UNION ALL
    SELECT 5, sumIf(1, level >= 5) FROM funnel
)
ORDER BY stage;
```

Expected output:

| stage | users_reached | conversion_pct |
|---:|---:|---:|
| 1 | 50 000 | 100.0 |
| 2 | 42 500 | 85.0 |
| 3 | 35 000 | 70.0 |
| 4 | 27 500 | 55.0 |
| 5 | 24 000 | 48.0 |

The `conversion_pct` column matches the `--drop-off-rates` we passed in Step 1. The query latency for a 50 k-user funnel in the paid tenant_class quota envelope is ~ 80 ms p95.

## Step 4 — Save as a Materialized View for dashboard freshness (≤ 10 min)

If the tenant queries this funnel every page-load, recompute on every read wastes capacity. Author a daily MV that pre-aggregates per stage:

```sql
USE tenant_drill_acme;

CREATE MATERIALIZED VIEW workflow_funnel_daily
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(day)
ORDER BY day
TTL day + INTERVAL 730 DAY
AS
SELECT
    toDate(event_time) AS day,
    uniqExactStateIf(user_id, event_name = 'workflow_started')        AS stage1_state,
    uniqExactStateIf(user_id, event_name = 'form_submitted')          AS stage2_state,
    uniqExactStateIf(user_id, event_name = 'payment_intent_created')  AS stage3_state,
    uniqExactStateIf(user_id, event_name = 'payment_succeeded')       AS stage4_state,
    uniqExactStateIf(user_id, event_name = 'workflow_completed')      AS stage5_state
FROM workflow_event
GROUP BY day;
```

Then query the MV at dashboard read-time:

```sql
SELECT
    day,
    uniqExactMerge(stage1_state) AS s1,
    uniqExactMerge(stage2_state) AS s2,
    uniqExactMerge(stage3_state) AS s3,
    uniqExactMerge(stage4_state) AS s4,
    uniqExactMerge(stage5_state) AS s5
FROM workflow_funnel_daily
WHERE day >= today() - INTERVAL 7 DAY
GROUP BY day
ORDER BY day;
```

Query latency drops from ~ 80 ms (funnel from raw) to ~ 12 ms (funnel from MV). The trade-off: this MV approximates the funnel (`uniqExact` of users-who-hit-stage-N) without preserving the strict ordering that `windowFunnel` does — so out-of-order user behaviour (rare) inflates conversion. Acceptable for a daily dashboard; use the raw `windowFunnel` for ad-hoc analyst exploration.

## Step 5 — Pin the query to a tenant dashboard (≤ 5 min)

```sh
oya analytics dashboard pin \
    --tenant drill-acme \
    --query-id workflow_funnel_daily_v1 \
    --query-file ./funnel-mv.sql \
    --refresh-interval 5m \
    --visibility tenant-analyst,tenant-admin
```

The dashboard now renders the funnel with 5-minute refresh. The Cedar gate `analytics::dashboard::pin` is checked at pin-time and at every render.

## Step 6 — Audit-chain verification (≤ 5 min)

Every query emitted an audit event. Query the chain:

```sh
oya audit query --tenant drill-acme --since 1h --event-class analytics_query_*
```

Expected events:

- `analytics_query_executed` (5-10 — one per ad-hoc query above, plus one per dashboard render).
- `analytics_mv_created` (1 — your `workflow_funnel_daily`).
- `analytics_dashboard_pinned` (1 — your funnel dashboard).

All events Ed25519-signed against the `oyatie.analytics.runtime` key:

```sh
oya audit verify-chain --tenant drill-acme --since 1h
```

Output: `chain verified, N events, no signature gaps`.

## What you've learned

- The `windowFunnel()` + `*State()`/`*Merge()` two-step.
- The trade-off between strict funnel order (`windowFunnel`) and approximate stage cardinality (`uniqExact` per stage).
- The Materialized View pattern for dashboard query latency.
- The Cedar + audit-chain enforcement boundary on tenant queries.

Next tutorial: `tutorials/cross-tenant-aggregate-via-ops.md` — fleet-aggregate ops dashboards that read across tenants without violating per-tenant isolation.
