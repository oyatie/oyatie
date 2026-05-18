# Analytics µservice — Backfill and Replay Procedure

**Authority:** ADR-0193, ADR-0153 outbox, ADR-0154 event schema versioning, ADR-0195 stream processing
**Owner:** council-analytics
**Last reviewed:** 2026-05-18

## 1. When to backfill

Three legitimate triggers:

1. **New MV deployed** — A new materialized view is added; the target table needs to be populated from historical source data.
2. **MV bug correction** — A bug is discovered in a deployed MV; the MV must be recomputed from canonical source.
3. **Schema migration** — A column is added or a partition strategy changes; data needs to be rewritten under the new schema.

Backfill is **never** used to "fix" missing source events — that is the source µservice's responsibility via the outbox guarantee.

## 2. Architecture

```
canonical source ──→ Pulsar topic ──→ Kafka engine table ──→ MV ──→ target table
        │                                                              │
        └──── replay job ──→ source-snapshot table ──→ backfill MV ────┘
```

The replay job reads from the canonical source (either a Postgres source-of-truth or a frozen S3 snapshot), projects through the MV logic, and writes to the target table — *without* republishing to Pulsar (which would double-count for any consumer that already processed the original events).

## 3. Procedure — new MV deployment

### Step 1: Pre-flight

1. Confirm the new MV's target table is created and empty:
   ```sql
   SELECT count() FROM tenant_${tid}.new_mv_target;
   -- Expect 0.
   ```
2. Confirm the MV's source table has the full historical window:
   ```sql
   SELECT min(emitted_at), max(emitted_at), count() FROM tenant_${tid}.source_events;
   ```
3. Document the backfill window in the runbook record (`evidence/backfill/<date>-<mv-name>.md`).

### Step 2: Snapshot the source

For tenant-isolated backfill (the common case):

```sql
-- Snapshot the source partition.
CREATE TABLE tenant_${tid}.source_events_backfill_snapshot
ENGINE = MergeTree
ORDER BY (tenant_id, emitted_at)
AS SELECT * FROM tenant_${tid}.source_events WHERE emitted_at < '${cutoff_ts}';
```

### Step 3: Replay through the MV logic

The MV definition itself does the projection. We invoke it manually:

```sql
INSERT INTO tenant_${tid}.new_mv_target
SELECT
    toStartOfHour(emitted_at) AS hour,
    tenant_id,
    countState() AS run_count,
    sumState(duration_ms) AS total_duration_ms,
    quantilesState(0.5, 0.95, 0.99)(duration_ms) AS duration_percentiles
FROM tenant_${tid}.source_events_backfill_snapshot
WHERE event_type = 'workflow.executed' AND tenant_id = '${tid}'
GROUP BY hour, tenant_id;
```

### Step 4: Activate the live MV

Only after backfill completes:

```sql
ATTACH MATERIALIZED VIEW mv_new_workflow_aggregate ...;
```

### Step 5: Reconcile

Verify the target table has both backfilled history and live updates:

```sql
SELECT
    min(hour) AS earliest,
    max(hour) AS latest,
    count() AS bucket_count
FROM tenant_${tid}.new_mv_target;
```

The `earliest` should match the backfill start; the `latest` should be moving forward as new live events arrive.

### Step 6: Drop the snapshot

```sql
DROP TABLE tenant_${tid}.source_events_backfill_snapshot;
```

## 4. Procedure — MV bug correction

### Step 1: Quarantine the broken target

```sql
RENAME TABLE tenant_${tid}.target_table TO tenant_${tid}.target_table_broken_${date};
```

### Step 2: Recreate the target

```sql
CREATE TABLE tenant_${tid}.target_table ...;  -- from the corrected DDL
```

### Step 3: Detach the broken MV

```sql
DETACH MATERIALIZED VIEW mv_broken;
```

### Step 4: Replay (as in §3)

### Step 5: Attach the corrected MV

```sql
ATTACH MATERIALIZED VIEW mv_corrected ...;
```

### Step 6: Deep verify

For a sample of buckets, compare backfilled values to expected:

```sql
-- Spot-check 10 random hours.
SELECT
    hour,
    countMerge(run_count) AS backfilled_count,
    (SELECT count() FROM tenant_${tid}.source_events
     WHERE event_type = 'workflow.executed'
       AND toStartOfHour(emitted_at) = t.hour) AS source_count
FROM tenant_${tid}.target_table AS t
WHERE hour IN (SELECT hour FROM ... ORDER BY rand() LIMIT 10);
-- backfilled_count and source_count should match.
```

### Step 7: Drop the quarantined broken table

After 7 days (in case of regression need):

```sql
DROP TABLE tenant_${tid}.target_table_broken_${date};
```

## 5. Procedure — schema migration

### Step 1: Author the new schema

New table with the new schema:

```sql
CREATE TABLE tenant_${tid}.events_v2 ( ... );
```

### Step 2: Backfill from old schema

```sql
INSERT INTO tenant_${tid}.events_v2
SELECT
    old_col_1,
    old_col_2,
    -- new column with default
    'default' AS new_col_3,
    emitted_at
FROM tenant_${tid}.events;
```

### Step 3: Cut over

1. Add the new schema to the canonical event contract (per ADR-0154).
2. Source µservices update to emit the new field.
3. The MV emits to the v2 table.
4. Old `events` table renamed to `events_pre_migration`.
5. After 30 days of green operation, drop the old table.

## 6. Per-tenant vs cluster-wide backfill

- **Per-tenant backfill** — preferred. Bounded blast radius. Used for individual tenant correction.
- **Cluster-wide backfill** — required for MV bug correction. Coordinated across all tenants; runs during a low-traffic window; backfill rate-limited via `max_insert_threads`.

## 7. Rate limiting

Backfill `INSERT ... SELECT` operations are throttled to avoid starving live ingest:

```sql
SET max_insert_threads = 4;
SET min_insert_block_size_rows = 1048576;
SET max_block_size = 1048576;
```

For very large backfills, partition by date and process one partition at a time, with a 30-second sleep between partitions.

## 8. Idempotency

The target tables use `ReplacingMergeTree` or `AggregatingMergeTree`; re-running the same backfill produces the same result (within merge eventual consistency). Backfill is therefore safe to retry on failure.

## 9. Audit trail

Every backfill emits a record at `evidence/backfill/<date>-<mv-name>.json`:

```json
{
  "mv": "mv_workflow_aggregate",
  "tenant_id": "ten_acme",
  "window": {"from": "2026-01-01T00:00:00Z", "to": "2026-05-18T00:00:00Z"},
  "trigger": "new_mv_deployment",
  "operator": "alice@oyatie",
  "duration_minutes": 47,
  "rows_processed": 1234567890,
  "verification": "spot_check_passed",
  "approval_ref": "pr/216"
}
```

## 10. Failure recovery

If a backfill fails mid-way:

1. Identify the last completed partition via `SELECT max(emitted_at) FROM target`.
2. Resume from that partition.
3. `ReplacingMergeTree` deduplicates if any partial state was committed.

## 11. References

- ADR-0193 (engine), ADR-0153 outbox, ADR-0154 event schema versioning, ADR-0195 stream processing.
- ClickHouse `INSERT ... SELECT` semantics: https://clickhouse.com/docs/sql-reference/statements/insert-into
- ClickHouse `AggregatingMergeTree` finalization: https://clickhouse.com/docs/engines/table-engines/mergetree-family/aggregatingmergetree
