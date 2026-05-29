# Runbook — Ingest Lag Burn

**Authority:** ADR-0195 stream processing, ADR-0186 observability, IP-004, IP-005
**Owner:** ops-sre-reliability + council-analytics
**Trigger:** `analytics-clickhouse-ingest-lag-burnrate-fastburn` (14.4× in 1 h) OR mediumburn (6× in 6 h).
**Severity:** Sev 2 (customer-visible dashboard freshness degradation)

## What broke

The MV ingest lag from outbox emit to target-table visibility is > 5 s p99 (SLO budget). At burn rate 14.4×, the 30-day budget is exhausted in ~2 days.

## Quick diagnosis

### Step 1: Consumer lag

```
clickhouse-client --query "
SELECT
    table,
    assignments.partition_id AS partition,
    assignments.current_offset AS current,
    assignments.committed_offset AS committed,
    (current - committed) AS lag
FROM system.kafka_consumers
ARRAY JOIN assignments
WHERE table = 'oya_events_kafka_source'
ORDER BY lag DESC
LIMIT 10
"
```

If `lag` is growing → consumer is behind Pulsar.

### Step 2: MV target merge backlog

```
clickhouse-client --query "
SELECT database, table, length(merges) AS in_flight, formatReadableSize(size) AS size
FROM system.merges
WHERE database LIKE 'tenant_%'
ORDER BY in_flight DESC
LIMIT 10
"
```

If `in_flight > 5` per shard → merge backlog; downstream of insert pressure.

### Step 3: Pulsar broker health

```
kubectl get pods -n observability -l app=pulsar-broker
```

If any pulsar broker is NotReady → upstream broker outage. See `runbooks/incident-response.md §5.6`.

## Decision tree

### Symptom: Consumer lag growing, but cluster otherwise healthy

**Likely cause:** consumer parallelism insufficient for current burst.

**Action:**

1. Increase `kafka_num_consumers`:
   ```sql
   ALTER TABLE oya_events_kafka_source MODIFY SETTING kafka_num_consumers = 12;  -- up from 6
   ```
2. Watch lag for 10 minutes.
3. If still growing: add server replicas (the consumer threads live on the replicas).

### Symptom: Merge backlog on target tables

**Likely cause:** insertion rate exceeds merge throughput.

**Action:**

1. Identify the offending table(s):
   ```sql
   SELECT
       database,
       table,
       count() AS parts,
       sum(rows) AS rows
   FROM system.parts
   WHERE active AND database LIKE 'tenant_%'
   GROUP BY database, table
   HAVING parts > 100
   ORDER BY parts DESC
   ```
2. If small parts (lots of small inserts), consolidate via:
   ```sql
   OPTIMIZE TABLE tenant_${tid}.${table} ON CLUSTER oya-cell FINAL;
   ```
3. Throttle ingest upstream (per-tenant QUOTA tightened) until merges catch up.

### Symptom: Specific MV slow

**Likely cause:** MV does expensive transform per insert.

**Action:**

1. Identify which MV: check `system.query_log` for slow MV SELECTs.
2. Profile the MV SELECT:
   ```
   clickhouse-client --query "EXPLAIN PIPELINE SELECT ... FROM ..."
   ```
3. If a JSON extraction is hot, pre-extract to a column at insert time.
4. Consider partitioning the MV target differently.

### Symptom: Pulsar broker failover ongoing

**Likely cause:** Pulsar broker restart caused consumer rebalance.

**Action:**

1. Wait for rebalance to complete (typically <2 min).
2. Verify consumer resumes from last committed offset:
   ```
   kafka-consumer-groups --bootstrap-server pulsar-kafka:9092 --describe --group analytics-clickhouse
   ```
3. If consumer fails to resume, force restart:
   ```sql
   SYSTEM RESTART CONSUMER 'oya_events_kafka_source';
   ```

## Verification

After mitigation:

1. Lag p99 returns < 5 s within 15 minutes.
2. Burn rate trends downward.
3. SLO budget remains within window (no false-positive alert resolve).

## Long-term mitigations

- Auto-scale Kafka consumers based on lag (deferred to phase 2 — finops controller).
- Predictive merge scheduling (ClickHouse 27+ feature; not in 26.3 LTS).
- Per-tenant ingest rate-limit at the source µservice outbox publisher.

## References

- ADR-0195 §"Default tier", ADR-0186 Stage 4, IP-004, IP-005.
- ClickHouse Kafka engine docs: https://clickhouse.com/docs/engines/table-engines/integrations/kafka
