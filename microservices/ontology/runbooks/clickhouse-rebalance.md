---
doc_class: Runbook
title: ClickHouse rebalance + lag recovery
microservice: ontology
severity: "Sev-3 (OLAP lag) / Sev-2 (rebalance during incident)"
status: Accepted
owner_team: axis-ontology + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/ontology/failure-modes.md (FM-10 ClickHouse lag)
  - microservices/ontology/capacity-model.md
doc_status: published
---

# Runbook: ClickHouse rebalance + lag recovery

## Trigger

Any of:
- ClickHouse mirror lag > 60 s (FM-10).
- Per-shard load imbalance ≥ 2× median.
- Cold-tier object storage saturation > 85 %.
- Tenant pack capacity migration.

## Severity

- Sev-3 for lag-only (OLAP reads stale).
- Sev-2 if lag > 5 min affects compliance reads or audit-evidence delivery.

## Pre-checks

1. ClickHouse shard health: `SELECT shard_num, replica_num, is_replica_active FROM system.clusters WHERE cluster = 'ontology';`
2. Mirror-consumer lag: `kafka-consumer-groups.sh --describe --group ontology-clickhouse-mirror` — LAG column.
3. Per-shard load: `SELECT count() FROM system.parts WHERE active GROUP BY partition ORDER BY count() DESC LIMIT 10`.
4. ClickHouse compactor backlog: `SELECT count() FROM system.parts_to_move`.

## Steps — Lag recovery

| Step | Action | Time |
|---|---|---|
| 1 | Identify the lagging mirror-consumer: pinpoint partition + offset | ≤ 5 min |
| 2 | Scale up ClickHouse mirror-ingester replicas: `kubectl scale deployment ontology-clickhouse-mirror -n ontology --replicas=8` | ≤ 5 min |
| 3 | Validate ingestion rate climbing: `clickhouse_mirror_ingest_rate_rows_per_sec` should rise | ≤ 5 min |
| 4 | If lag > 5 min: throttle OLAP read budget per-tenant via `clickhouse-set-quota` | ≤ 5 min |
| 5 | Monitor lag drain to baseline (≤ 60 s) | ≤ 30 min |
| 6 | Once lag normal: restore OLAP quota | ≤ 5 min |
| 7 | Postmortem if lag > 5 min duration | per timeline |

## Steps — Shard rebalance (capacity expansion)

| Step | Action | Time |
|---|---|---|
| 1 | Schedule maintenance window | ≤ 1 d |
| 2 | Confirm new shard nodes are healthy + joined the cluster | ≤ 30 min |
| 3 | Trigger rebalance: `clickhouse-rebalance --shard-target <new-shard-id>` | hours |
| 4 | Monitor `system.replication_queue` for any stuck replicas | continuous |
| 5 | Validate per-tenant query latency stable during rebalance | continuous |
| 6 | Once rebalance complete: update Helm values to register the new shard count | ≤ 5 min |

## Steps — Cold-tier saturation

| Step | Action | Time |
|---|---|---|
| 1 | Identify saturated bucket: `clickhouse-storage-usage --tier cold` | ≤ 5 min |
| 2 | Trigger aggressive compaction: `OPTIMIZE TABLE <ontology>.<table> FINAL` (off-peak only) | varies |
| 3 | Adjust archive policy: move 6mo+ data to deeper archive tier | ≤ 1 d |
| 4 | If still saturated: shard the bucket by tenant prefix (multi-bucket) | ≤ 1 w |
| 5 | FinOps review: capacity budget vs forecast | per timeline |

## Verification

After recovery:
- ClickHouse `clickhouse_mirror_lag_seconds` ≤ 60 s.
- OLAP Function p99 ≤ 500 ms.
- No stuck replicas in `system.replication_queue`.
- Per-shard `parts` count within 2× of median.
- Self-observability dashboard green.

## Post-incident updates

- Postmortem within 5 business days for Sev-2.
- If lag recurs frequently: consider raising baseline ingest-consumer replicas.
- Quarterly capacity-model refresh.

## References

- `microservices/ontology/failure-modes.md` FM-10.
- `microservices/ontology/capacity-model.md`.
- ClickHouse replication — `clickhouse.com/docs/en/engines/table-engines/mergetree-family/replication`.
- ClickHouse storage policies — `clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree#table_engine-mergetree-multiple-volumes`.
