---
doc_class: Runbook
title: Citus rebalance + Postgres / Patroni failover
microservice: tenancy
severity: "Sev-2 (single component) / Sev-1 (DCS quorum loss)"
status: Accepted
owner_team: ops-sre-reliability + axis-tenancy
date: 2026-05-17
related_artifacts:
  - microservices/tenancy/failure-modes.md (FM-01 Postgres primary, FM-03 Citus coordinator, FM-08 rebalance stuck, FM-12 DCS outage)
  - microservices/tenancy/capacity-model.md
  - microservices/tenancy/multi-region.md
doc_status: published
---

# Runbook: Citus rebalance + Postgres / Patroni failover

## Trigger

Any of:
- Citus rebalance scheduled (capacity-driven; shard utilisation > 80%).
- Citus rebalance hung (FM-08): `oya_tenancy_rebalance_duration_seconds{quantile="0.99"} > 3600` sustained.
- Postgres primary outage (FM-01); Patroni auto-failover in progress.
- Citus coordinator outage (FM-03); Patroni-managed coordinator failover.
- Patroni DCS outage (FM-12); quorum loss.

## Severity

- Scheduled rebalance: not an incident (tracked as ordinary ops).
- Stuck rebalance / coordinator outage: Sev-2.
- DCS quorum loss: Sev-1 (HA broken; cluster cannot elect leader).

## Citus rebalance (normal)

| Step | Action | Time budget |
|---|---|---|
| 1 | cell-assignment-worker schedules rebalance when shard utilisation > 80% on any worker. | – |
| 2 | Emit `CellRebalanceStarted` Workflow event; capture pre-rebalance row checksums per shard. | ≤ 5 s |
| 3 | Citus shard-move via `citus_move_shard_placement(<shard_id>, <source>, <target>)` (logical-replication-backed; transactional cut-over). | seconds-minutes per shard |
| 4 | Verify post-move row count + checksum match pre-move. | ≤ 1 min per shard |
| 5 | Emit `CellRebalanceCompleted` event; audit-chain seal. | ≤ 5 s |

## Recovery Path A — Rebalance hung (FM-08)

Cause: logical-replication lag; coordinator restart mid-rebalance.

| Step | Action |
|---|---|
| 1 | Verify rebalance state: `SELECT * FROM citus_get_active_worker_nodes();` + `citus_rebalance_status();` |
| 2 | Identify stuck shard from the metrics + `pg_stat_subscription`. |
| 3 | If safe to abort (pre-rebalance state still recoverable): `SELECT citus_rebalance_stop();` |
| 4 | Verify pre-rebalance row counts + checksums still match (no partial move corruption). |
| 5 | Resume from clean state in next cycle once root cause identified. |
| 6 | If unsafe to abort (mid-cut-over): wait for completion (Citus is transactional; eventually succeeds) OR engage DBA JIT for manual intervention. |

## Recovery Path B — Postgres primary failover (FM-01)

Cause: Postgres primary pod loss; Patroni triggers auto-failover.

| Step | Action |
|---|---|
| 1 | Verify Patroni state: `patronictl list` shows `Leader` on a different node within ≤ 10s. |
| 2 | Verify sync replica was promoted (no async replica should be promoted given quorum). |
| 3 | tenancy connection-pool re-routes; verify `oya_tenancy_postgres_primary_alive == 1` on new primary. |
| 4 | Verify Patroni replication lag among remaining replicas ≤ 5s. |
| 5 | Spin up replacement async replica (Patroni provisions automatically; verify cluster_size returns to declared replicas). |
| 6 | Validate downstream: cell-assignment worker resumes; activation worker queue drains. |
| 7 | Re-warm Valkey cache from new primary (cache will fill within 5min). |

## Recovery Path C — Citus coordinator failover (FM-03)

Cause: Citus coordinator pod loss; Patroni-managed.

| Step | Action |
|---|---|
| 1 | Verify Patroni-managed Citus coordinator state: `patronictl list -c citus-coord`. |
| 2 | Sync replica should be promoted within ≤ 30s. |
| 3 | Verify writes resume (new tenant activations + lifecycle mutations). |
| 4 | Verify worker connections re-establish to new coordinator. |
| 5 | Spin up replacement sync replica. |

## Recovery Path D — DCS quorum loss (FM-12)

Cause: 2 of 3 etcd pods down; Patroni can't elect leader.

| Step | Action |
|---|---|
| 1 | Engage ops-sre-reliability + cloud-k8s on-call. Declare Sev-1. |
| 2 | Restore etcd quorum: scale up etcd replicas; verify `etcd_server_has_leader == 1`. |
| 3 | If etcd cluster un-restorable in-place: engage etcd disaster-recovery (restore from snapshot — Patroni's last-known state is in etcd, so a stale snapshot may require Patroni cluster re-initialise; ops-sre-reliability owns). |
| 4 | Once DCS quorum restored: Patroni resumes leader election; primary stable within 30s. |
| 5 | If primary unreachable during quorum loss: write path was halted; reads from sync replicas continued. |
| 6 | Post-incident: review DCS topology — should etcd cluster size grow to 5 for hyperscaler-tier packs? |

## Recovery Path E — Citus + Postgres major-version upgrade (planned)

Cause: scheduled upgrade.

| Step | Action |
|---|---|
| 1 | Schedule maintenance window with tenant-facing communication ≥ 30d advance. |
| 2 | Snapshot Postgres data + WAL archive. |
| 3 | Upgrade per-node rolling (Patroni-managed): replica nodes first, then coordinated primary failover, then primary node. |
| 4 | Per-Citus version: review breaking changes; validate shard-distribution compatibility. |
| 5 | Validate downstream after upgrade: run synthetic activation drill + cross-tenant probe. |

## Verification

After completion:
- Patroni `patronictl list` shows healthy cluster (1 leader + expected replicas).
- Citus `SELECT * FROM citus_get_active_worker_nodes()` shows all expected workers.
- Postgres write path latency p99 ≤ baseline.
- Validate hot-path latency p99 ≤ 5ms (cache may need 5min to warm).
- No drift in `oya_tenancy_rls_drift_total` (RLS preserved through failover; FORCE attribute survives).
- Audit-chain seal log captures the event(s).

## Post-incident updates

- Postmortem within 5 business days (Sev-2+).
- For DCS quorum loss: review etcd topology; consider larger cluster size for HA-critical packs.
- For rebalance hung: review per-shard size limits; consider tighter shard-utilisation threshold for proactive splits.

## References

- `microservices/tenancy/failure-modes.md` FM-01 + FM-03 + FM-08 + FM-12.
- `microservices/tenancy/capacity-model.md`.
- `microservices/tenancy/multi-region.md`.
- Patroni HA documentation — `patroni.readthedocs.io`.
- Citus operational guide — `docs.citusdata.com/en/stable/admin_guide/cluster_management.html`.
- etcd operations — `etcd.io/docs/`.
