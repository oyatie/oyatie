---
doc_class: Runbook
title: ClickHouse Rebalance (parity_analytics cluster)
microservice: foundry-eval
severity: "Sev-3 (analytics query latency elevated) / Sev-2 (analytics unavailable; downstream parity verdicts delayed)"
status: Accepted
owner_team: ops-sre-reliability + axis-foundry
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-06 ClickHouse rebalance)
  - microservices/intelligence/threat-model.md (T-D-02, T-I-01)
  - microservices/intelligence/capacity-model.md
doc_status: published
---

# Runbook: ClickHouse Rebalance

## Trigger

ONE of:

1. `clickhouse_query_latency_seconds{quantile="0.99",table="parity_analytics"} > 0.5` for ≥ 5 min.
2. ClickHouse shard imbalance > 20% size variance across shards.
3. Disk pressure on any ClickHouse node > 85%.
4. ZooKeeper coordination latency p99 > 100ms.
5. Cross-tenant query leak detected (T-I-01) — Sev-1 escalation.

## Severity

- Query latency elevated, no leak: **Sev-3**.
- Query unavailable for ≥ 10 min: **Sev-2**.
- Cross-tenant leak: **Sev-1** (escalate to ops-security incident playbook).

## Pre-checks

1. Confirm cluster state: `kubectl exec -n foundry-eval clickhouse-0 -- clickhouse-client -q "SELECT cluster, shard_num, replica_num, host_name FROM system.clusters WHERE cluster='parity_analytics'"`.
2. Confirm shard sizes: `clickhouse-client -q "SELECT shard_num, sum(bytes_on_disk)/1e9 AS gb FROM system.parts GROUP BY shard_num"`.
3. Confirm ZooKeeper quorum: `kubectl exec -n foundry-eval zookeeper-0 -- echo ruok | nc localhost 2181` returns `imok` on majority.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC; declare severity | ≤ 5 min |
| 2 | Pre-checks above | ≤ 10 min |
| 3 | For Sev-1 (cross-tenant leak): freeze affected query endpoint; engage ops-security incident playbook; cease normal operations; refer to `runbooks/parity-regression-triage.md` is NOT appropriate — refer to ops-security playbook | escalation |
| 4 | For Sev-2/3 (latency/imbalance): check query plan: `EXPLAIN PIPELINE` on slow queries; identify cardinality blowups | ≤ 15 min |
| 5 | If single hot shard (data skew): apply `ALTER TABLE parity_analytics MOVE PARTITION` per shard-balance script `iac/clickhouse/rebalance.sh` (2-person rule per `policy/two-person-admin-ops.md` for production cluster ops) | ≤ 1 h per partition |
| 6 | If disk pressure: trigger MergeTree compaction; archive cold partitions to S3 per `policy/data-residency.md` retention | ≤ 2 h |
| 7 | If ZooKeeper coordination slow: investigate ZK node health; restart unhealthy ZK replicas one-at-a-time preserving quorum | ≤ 30 min |
| 8 | If query plan inefficient: identify problematic query; add appropriate index OR rewrite query; for repeating offenders, file Issue against query author | per fix |
| 9 | Verify rebalance complete: shard size variance < 10%; query latency p99 ≤ 200ms | ≤ 1 h |
| 10 | If cluster scale-out needed: provision new shard + replicate via `iac/clickhouse/scale-out.sh`; 2-person rule | ≤ 4 h |
| 11 | Update `capacity-model.md` if forecast revised | — |
| 12 | Postmortem within 5 business days for Sev-1 / Sev-2 | — |

## Cross-tenant Leak Special Case

If ANY query result returns rows from `tenant_id != requesting_tenant_id` (T-I-01):
1. Freeze the affected endpoint immediately (cordon at Envoy).
2. Capture forensic trace: which query? which user? which session?
3. Engage ops-security incident response.
4. Begin breach-notification chain per GDPR Art. 33 / KR PIPA equivalent / HIPAA breach notification.
5. Audit-chain emission of leak detection.
6. Refer to `policy/tenant-isolation.md` for invariant violation handling.

## Verification

After completion:
- `clickhouse_query_latency_seconds{quantile="0.99"} <= 0.2` sustained ≥ 1h.
- Shard size variance < 10%.
- No `cross_tenant_query_leak_total > 0` (must remain zero).
- `ClickHouseRebalanced{shard_movements, partitions_compacted}` event in audit-chain.

## References

- `microservices/intelligence/failure-modes.md` FM-06.
- `microservices/intelligence/threat-model.md` T-D-02, T-I-01.
- `microservices/intelligence/policy/tenant-isolation.md`.
- `microservices/intelligence/policy/two-person-admin-ops.md`.
- `microservices/intelligence/capacity-model.md`.
- ClickHouse docs `clickhouse.com/docs/en/operations/`.
