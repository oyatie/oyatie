---
doc_class: Runbook
title: State-lock break (OpenTofu advisory lock)
microservice: cloud-iac
severity: "Sev-3 (transient) / Sev-2 (persistent)"
status: Accepted
owner_team: axis-cloud-iac
date: 2026-05-17
related_artifacts:
  - microservices/cloud-iac/failure-modes.md (FM-04)
  - microservices/cloud-iac/incident-response.md
doc_status: published
---

# Runbook: State-lock break (OpenTofu advisory lock)

## Trigger

OpenTofu state-lock held > 10min OR `iac_state_lock_timeout_total > 0`.

## Severity

- Lock release within 15min (lock-holder still alive but slow): Sev-3.
- Lock-holder dead (no corresponding applier pod) and lock not released: Sev-2.
- Cluster-wide state-lock contention pattern: Sev-2.

## Pre-checks

1. Identify the lock: cloud-native IaC controller/API `state-lock status` workflow shows (microservice, pack, env, lock_id, holder, acquired_at).
2. Verify lock-holder pod is alive: `kubectl -n cloud-iac get pod <holder-pod>`.
3. Determine lock age: if held > 10min, abnormal.
4. Check Postgres advisory-lock state: `SELECT pid, locktype, mode FROM pg_locks WHERE locktype = 'advisory'`.

## Recovery Path A — Lock-holder alive but slow

| Step | Action |
|---|---|
| 1 | Let it complete; lock will release when apply finishes OR timeout at 10min |
| 2 | If apply times out at 10min: applier auto-aborts; lock auto-released |
| 3 | Verify subsequent applier acquires the lock |

## Recovery Path B — Lock-holder dead (orphaned lock)

| Step | Action |
|---|---|
| 1 | Confirm holder pod is gone: `kubectl get pod <holder-pod>` returns NotFound |
| 2 | Confirm lock is stale: lock acquired_at > 10min ago AND no corresponding live applier |
| 3 | Force-unlock: cloud-native IaC controller/API `state-lock force-unlock` workflow. **REQUIRES JIT + 2-person rule**. The CLI: <br>  a. Validates the lock is truly orphaned (re-checks lock state + pod state); <br>  b. Captures the rationale in audit-chain; <br>  c. Releases the Postgres advisory lock; <br>  d. Emits `state_lock_force_unlock` audit event |
| 4 | Verify subsequent applier acquires the lock |
| 5 | Postmortem: why did the lock-holder die? Was it an OOM, eviction, kubelet failure? |

## Recovery Path C — Cluster-wide state-lock contention pattern

Cause: many appliers competing for state-locks across different (microservice, pack, env) tuples (typically a Postgres CPU or connection-pool issue masquerading as state-lock contention).

| Step | Action |
|---|---|
| 1 | Verify Postgres health: `psql -c 'SELECT count(*) FROM pg_stat_activity'` + check CPU |
| 2 | If Postgres CPU > 80%: scale Postgres vertically OR add pgBouncer |
| 3 | If connection-pool exhausted: increase `max_connections` (with corresponding memory bump) |
| 4 | Throttle applier replicas temporarily |
| 5 | Verify lock wait latency p99 returns to ≤ 30s |

## Verification

After recovery:
- `iac_state_lock_wait_seconds_p99 < 30` for ≥ 30min.
- No orphan locks in `pg_locks` advisory lock listing.
- Applier queue depth returns to baseline.
- `iac_state_lock_timeout_total` rate < 1 / hour.

## Post-incident updates

- Postmortem (Sev-2+) within 5 business days.
- If force-unlock was used: review the JIT trail; verify rationale is captured + complete.
- If Postgres bottleneck pattern: capacity-model.md update + cost-budget.md review.
- Consider replacing Postgres advisory lock with Kubernetes Lease object if churn pattern suggests it.

## References

- `microservices/cloud-iac/failure-modes.md` FM-04.
- `microservices/cloud-iac/incident-response.md`.
- Postgres advisory-lock docs — `postgresql.org/docs/current/explicit-locking.html#ADVISORY-LOCKS`.
- OpenTofu state-lock docs — `opentofu.org/docs/language/state/locking/`.
