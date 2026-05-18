# ClickHouse Runbook — Observability

**Authority:** ADR-0193
**Owner:** axis-observability + ops-sre-reliability
**Last reviewed:** 2026-05-18

## Cluster health quick-check

```bash
# All ClickHouse server pods
kubectl get pods -n observability -l clickhouse.altinity.com/chi=oya-observability-clickhouse

# Keeper quorum
kubectl exec -n observability clickhouse-keeper-0 -- clickhouse-keeper-client -p 9181 -q 'stat'

# Verify version + connectivity
kubectl exec -n observability chi-oya-observability-clickhouse-oya-cell-0-0-0 -- \
  clickhouse-client --query 'SELECT version()'

# Replication lag
kubectl exec -n observability chi-oya-observability-clickhouse-oya-cell-0-0-0 -- \
  clickhouse-client --query 'SELECT table, replica_name, absolute_delay FROM system.replicas WHERE absolute_delay > 0'
```

## Keeper leader lost

`ClickHouseKeeperLeaderLost` alert:

1. Confirm: `kubectl exec clickhouse-keeper-0 -- clickhouse-keeper-client -q 'stat' | grep Mode`.
2. If no pod returns "Mode: leader" within 60s, restart the keeper-0 pod.
3. Verify leader election: `for i in 0 1 2; do kubectl exec clickhouse-keeper-$i -- clickhouse-keeper-client -q 'stat' | grep Mode; done` — exactly one should be "leader".

## Replication lag > 60s

1. Identify lagging table: `SELECT * FROM system.replicas WHERE absolute_delay > 60`.
2. Check network between replicas: `kubectl exec ... -- traceroute clickhouse-server-N`.
3. If sustained, scale source-side ingest rate down or scale destination replica's CPU/memory up.

## Part-merge backlog

`ClickHousePartMergeBacklog` (>100 parts queued):

1. `SELECT count() FROM system.parts WHERE active=1 GROUP BY table ORDER BY count() DESC`.
2. If a single table dominates, check `system.merge_tree_settings.parts_to_throw_insert` — may be hitting insert throttle.
3. Consider increasing `parts_to_throw_insert` temporarily; investigate why partition count is so high (under-partitioned data, schema design).

## Cold-tier S3 latency

Cold-tier query p99 > 2s:

1. Check SeaweedFS S3 endpoint latency from a ClickHouse pod.
2. Verify CSI fast-class disk has not run out of space (hot tier).
3. If pattern is sustained, consider extending the hot window (TTL update; requires partition migration).

## Escalation

- Page → PagerDuty `observability-oncall` + Opsgenie `observability-oncall`.
- Slack: `#observability-incidents`.
