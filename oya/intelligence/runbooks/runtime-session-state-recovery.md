---
doc_class: Runbook
title: Session-state recovery (Postgres replica + cold-restore latency + Postgres outage)
microservice: foundry-runtime
severity: "Sev-2 (Postgres outage) / Sev-3 (replica fail or latency spike)"
status: Accepted
owner_team: ops-sre-reliability + axis-foundry-runtime
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-09, FM-10)
  - microservices/intelligence/multi-region.md
  - microservices/intelligence/capacity-model.md
doc_status: published
---

# Runbook: Session-state recovery

## Trigger

ONE of:
- Postgres replica fail (FM-09): `pg_replication_lag_seconds > 60` OR replica unreachable.
- Cold-restore latency spike (FM-10): `oya_foundry_runtime_session_cold_restore_duration_seconds{quantile="0.99"} > 100ms`.
- Postgres primary outage (broader Sev-2): no writes accepted; reads served by replicas at lag.

## Severity

- Replica fail: Sev-3.
- Cold-restore latency: Sev-3.
- Primary outage: Sev-2.

## Postgres replica fail (FM-09)

| Step | Action | Time |
|---|---|---|
| 1 | Identify failing replica: `kubectl -n foundry-runtime get pods -l role=postgres-replica` | ≤2min |
| 2 | Check replication health: connect to primary; `SELECT * FROM pg_stat_replication` | ≤5min |
| 3 | Restart replica pod: `kubectl rollout restart sts/oya-intelligence-runtime-postgres-replica-N` | ≤5min |
| 4 | Verify replication catches up: `pg_replication_lag_seconds < 30` for ≥5min | ≤15min |
| 5 | If pattern (multiple replica fails in 24h): engage cloud-secrets / OCI support; scale Postgres vertically | ≤30min |

## Cold-restore latency spike (FM-10)

| Step | Action | Time |
|---|---|---|
| 1 | Identify cause: `pg_stat_activity` for long-running queries; `pg_locks` for lock contention | ≤5min |
| 2 | If autovacuum behind: tune `autovacuum_vacuum_scale_factor`; manual `VACUUM ANALYZE` on hot tables | ≤30min |
| 3 | If IO saturation: scale Postgres vertically (more cores + faster volume) | ≤30min |
| 4 | If query-plan regression after pg_stats refresh: `ANALYZE` on affected tables | ≤10min |
| 5 | Verify recovery: `cold_restore_duration_seconds{quantile="0.99"} < 100ms` for ≥15min | – |

## Postgres primary outage

| Step | Action | Time |
|---|---|---|
| 1 | Engage Sev-2; open `#inc-<id>` Slack | ≤5min |
| 2 | Verify primary state: `kubectl -n foundry-runtime get pods -l role=postgres-primary` | ≤2min |
| 3 | If primary host failed: promote synchronous replica to primary (Patroni / Stolon orchestrator) | ≤5min |
| 4 | Update DNS / Service to point at promoted replica | ≤1min |
| 5 | Verify writes: invocation lifecycle records resuming | ≤5min |
| 6 | Verify session cold restores: latency back within target | ≤5min |
| 7 | If pack has DR pair (per `multi-region.md`) AND outage > 30min: initiate DR failover | ≤35min DR |
| 8 | Tenant comms per `incident-response.md` template | ≤30min |

## Verification

After recovery:
- `pg_replication_lag_seconds < 30` for all replicas for ≥5min.
- `cold_restore_duration_seconds{quantile="0.99"} < 100ms` for ≥15min.
- Session resume succeeds in synthetic probe.
- Self-observability dashboard green.

## Post-incident updates

- Postmortem within 5 business days.
- For repeated FM-09: investigate replica instance type; consider RAM upgrade.
- For repeated FM-10: revisit table partitioning; tune autovacuum globally.

## References

- `microservices/intelligence/failure-modes.md` FM-09, FM-10.
- `microservices/intelligence/multi-region.md` §"DR Failover".
- `microservices/intelligence/capacity-model.md`.
- Postgres 16 LTS HA — `postgresql.org/docs/16/high-availability.html`.
- Patroni — `github.com/zalando/patroni`.
