---
doc_class: Runbook
title: Redis Sentinel failover for lease coordinator
microservice: workflow-engine
severity: "Sev-1 (cluster-wide impact when quorum lost)"
status: Accepted
owner_team: axis-workflow + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/workflow-engine/failure-modes.md (FM-05 Redis outage)
  - microservices/workflow-engine/PRD.md (execution-engine adapter-redis)
  - microservices/workflow-engine/multi-region.md
doc_status: published
---

# Runbook: Redis Sentinel failover for lease coordinator

## Trigger

ONE of:

1. **Auto**: Redis Sentinel quorum lost; `oya_workflow_engine_redis_sentinel_quorum_healthy == 0` for ≥ 30s.
2. **Auto**: Step claim failures spike; `oya_workflow_engine_step_claim_failure_rate > 0.1`.
3. **Manual**: ops-sre-reliability triggers controlled failover (e.g., for maintenance).

## Severity

- Single AZ Redis pod failure with quorum preserved: Sev-3 (operational).
- Quorum lost; cluster-wide step dispatch halted: Sev-1.
- Auto-recovery successful within 5min: Sev-2 retroactively.

## Impact

When Redis is unhealthy, the engine's lease coordinator is unavailable. New step dispatch fails; in-flight runs pause at next step boundary. Long-running workflows are unaffected (their state is in Postgres; resumption resumes once Redis is back). Tenant impact: step latency spike + transient run-start refusal.

## Pre-checks

1. Verify Redis Sentinel pod state: `kubectl -n workflow-engine get pods -l app=redis-sentinel`.
2. Verify Sentinel quorum: `redis-cli -p 26379 sentinel masters`.
3. Verify primary Redis reachability: `redis-cli -h redis-primary.workflow-engine.svc PING`.
4. Verify engine worker step claim health: `oya_workflow_engine_step_claim_failure_rate`.

## Recovery Path A — Single pod failure with quorum preserved

This is the common case; Sentinel handles automatically.

| Step | Action |
|---|---|
| 1 | Sentinel detects pod failure; promotes a replica to primary if needed (within ≤ 30s per Sentinel default). |
| 2 | Engine clients reconnect to new primary; transparent. |
| 3 | Verify step claim rate returns to baseline. |
| 4 | Postmortem: investigate why the pod failed (resource exhaustion? bad node?). |

## Recovery Path B — Quorum lost (Sev-1)

| Step | Action |
|---|---|
| 1 | Sev-1 declared; engage axis-workflow + ops-sre-reliability on-call. |
| 2 | Activate Postgres advisory-lock fallback: `cargo run -p oya-dev-cli -- workflow-engine activate-postgres-lock-fallback --reason "<rfc>"`. Engine workers acquire step claims via Postgres advisory locks (degraded latency but available). |
| 3 | Verify step dispatch resumes via fallback path; latency degraded (claim acquisition now ~50ms vs ~5ms via Redis). |
| 4 | Investigate quorum loss: AZ outage? Network partition? Resource exhaustion? |
| 5 | Restore Redis: scale up new Sentinel pods; verify quorum reached. |
| 6 | Deactivate Postgres fallback: `cargo run -p oya-dev-cli -- workflow-engine deactivate-postgres-lock-fallback`. Engine clients reconnect to Redis. |
| 7 | Tenant notification + status page update. |

## Recovery Path C — Sentinel + Redis both lost (extreme failure)

| Step | Action |
|---|---|
| 1 | Sev-1; engage director-level. |
| 2 | Postgres fallback (Path B step 2) active immediately. |
| 3 | Rebuild Redis cluster from Helm chart: `kubectl rollout restart deployment/redis-sentinel deployment/redis-primary`. |
| 4 | Verify cluster healthy + Sentinel quorum re-established. |
| 5 | Lease state in Redis is regenerable from Postgres (engine knows which runs are in-flight); engine workers re-claim leases automatically. |
| 6 | Deactivate Postgres fallback once Redis is fully recovered. |

## Recovery Path D — DR failover triggered Redis transition

When DR failover is in progress (per `multi-region.md`), Redis Sentinel must be promoted in the DR pair:

| Step | Action |
|---|---|
| 1 | DR failover initiates (Sev-1 or Sev-2 from multi-region drill). |
| 2 | DR-pair Redis Sentinel cluster pre-warmed; engine clients connect to DR-pair endpoints (DNS failover). |
| 3 | Lease state in DR-pair Redis is empty (cross-pack replication forbidden); engine workers regenerate leases from DR-pair Postgres state. |
| 4 | Verify step claim resumption within RTO window (≤ 35 min per `multi-region.md`). |

## Recovery Path E — Controlled failover (maintenance)

| Step | Action |
|---|---|
| 1 | Schedule maintenance window. |
| 2 | Activate Postgres fallback proactively. |
| 3 | Cycle Redis pods (drain + drain-and-replace); Sentinel handles primary election transparently. |
| 4 | Verify quorum stable on new pods. |
| 5 | Deactivate Postgres fallback. |

## Verification

After recovery:
- `oya_workflow_engine_redis_sentinel_quorum_healthy == 1`.
- Step claim failure rate returns to baseline (< 0.01).
- In-flight runs resume normally.
- Postgres fallback not active.
- Tenant-facing dashboard healthy.

## Post-incident updates

- Postmortem within 5 business days.
- Action: improve detection latency for quorum loss (current target ≤ 30s).
- Action: verify Postgres fallback path SLO (latency + correctness under load).
- Action: review Redis capacity if quorum was lost due to resource exhaustion.

## References

- `microservices/workflow-engine/failure-modes.md` FM-05.
- `microservices/workflow-engine/PRD.md` execution-engine BC + adapter-redis.
- `microservices/workflow-engine/multi-region.md`.
- Redis Sentinel — `redis.io/topics/sentinel`.
- Postgres advisory locks — `postgresql.org/docs/current/explicit-locking.html#ADVISORY-LOCKS`.
