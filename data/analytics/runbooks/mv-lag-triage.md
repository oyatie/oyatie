# Runbook — Materialized View Lag Triage

**Authority:** ADR-0195, IP-005, IP-004
**Owner:** council-analytics
**Trigger:** Per-MV freshness SLO burn (any MV in `microservices/analytics/slos/`).
**Severity:** Sev 2

## What an MV is

A ClickHouse `MATERIALIZED VIEW` is a trigger: when rows insert into the source table, the MV's `SELECT` runs and inserts the projected rows into the target table. Lag is between (a) source insert and (b) target visibility.

## Diagnosis

### Step 1: Confirm the MV is firing

```sql
SELECT
    database,
    table AS mv_name,
    is_active,
    last_exception
FROM system.materialized_views
WHERE database = 'tenant_${tid}' AND table = '${mv_name}';
```

If `is_active=0` or `last_exception` is non-empty: the MV has failed and rows are not being projected.

### Step 2: Compare source vs target counts

```sql
-- Source row count for the window.
SELECT count() FROM oya_events_kafka_source
WHERE event_type = '...' AND emitted_at > now() - INTERVAL 1 HOUR;

-- Target row count.
SELECT countMerge(...) FROM tenant_${tid}.target_table
WHERE hour > now() - INTERVAL 1 HOUR;
```

Sustained source > target by a large gap → MV is lagging or has a bug.

### Step 3: Per-MV merge backlog

```sql
SELECT
    database,
    table,
    estimated_finish_time - now() AS time_to_finish
FROM system.merges
WHERE database = 'tenant_${tid}' AND table = '${mv_name}'
ORDER BY estimated_finish_time DESC
LIMIT 5;
```

## Decision tree

### MV not firing (is_active=0)

1. Reattach:
   ```sql
   DETACH MATERIALIZED VIEW tenant_${tid}.${mv_name};
   ATTACH MATERIALIZED VIEW tenant_${tid}.${mv_name};
   ```
2. Watch for `last_exception` to clear.
3. If `last_exception` references a schema mismatch (column missing in target), align the target schema and retry.

### MV firing but produces wrong values (logic bug)

Refer to `microservices/analytics/backfill-replay.md` §4 (MV bug correction).

1. Quarantine: rename the broken target.
2. Recreate target.
3. Detach broken MV; deploy fixed MV.
4. Backfill from canonical source.
5. Verify.

### MV firing but lag growing

See `runbooks/ingest-lag-burn.md` — root cause is upstream ingest pressure or downstream merge backlog.

## Verification

1. `is_active=1` on the MV.
2. Source count and target merge-count agree within 5 s.
3. SLO burn returns < 1×.

## Caveat — MV chains

Chained MVs (MV A → table B → MV C → table D) compound lag. If lag is high on a chained MV, check each link.

## References

- ADR-0195 §"Default tier", IP-005, `microservices/analytics/backfill-replay.md`.
- ClickHouse MV semantics: https://clickhouse.com/docs/sql-reference/statements/create/view#materialized-view
