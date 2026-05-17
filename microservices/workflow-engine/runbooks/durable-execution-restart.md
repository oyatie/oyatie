---
doc_class: Runbook
title: Durable execution cold-start + replay storm handling
microservice: workflow-engine
severity: "Sev-2 (operational degradation during cold-start)"
status: Accepted
owner_team: axis-workflow + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/workflow-engine/failure-modes.md (FM-03 replay storm)
  - microservices/workflow-engine/PRD.md (FR-07 deterministic replay; AC-03 durable execution)
  - microservices/workflow-engine/policy/spec-integrity.md
doc_status: published
---

# Runbook: Durable execution cold-start + replay storm

## Trigger

ONE of:

1. **Engine cold-start**: cluster-wide engine restart (rolling deploy in failure-mode; large-scale pod eviction; region failover).
2. **Replay storm**: cold-start tries to resume all in-flight runs simultaneously; engine worker queue depth spikes; lease wait p99 > 2s.
3. **Single-pod kill mid-run**: HA failover resumes the run on a different worker; durable-execution invariant tested.

## Severity

- Single-pod kill + HA failover successful within RTO: Sev-3.
- Cluster-wide cold-start with throttling working: Sev-2.
- Cold-start with replay storm overwhelming workers: Sev-2 (could escalate to Sev-1 if cascading).

## Pre-checks

1. Verify engine worker pod state: `kubectl -n workflow-engine get pods -l app=execution-engine-worker`.
2. Verify Postgres + Redis health.
3. Verify resume-rate-limit configuration is active.

## Recovery Path A — HA failover single-pod kill

This is the common case; engine is designed for it.

| Step | Action |
|---|---|
| 1 | Identify the killed pod: log inspection. |
| 2 | HA failover should be automatic: another worker acquires lease for the affected runs within 5s (lease TTL). |
| 3 | New worker reads run state from Postgres + replays event log from last checkpoint. |
| 4 | Deterministic-replay invariant: replay produces the same step sequence (verified at AC-02). |
| 5 | Verify run continues: completion within expected window. |
| 6 | No tenant action required. |

## Recovery Path B — Cluster-wide cold-start (rolling deploy)

| Step | Action |
|---|---|
| 1 | Rolling deploy in progress; some workers cold; in-flight runs resume on warm workers. |
| 2 | Resume-rate-limit caps replay at 100 runs/s/worker; verifies in `oya_workflow_engine_resume_rate_per_worker`. |
| 3 | New run-starts may experience start-latency degradation; SLO graceful. |
| 4 | HPA ramps workers as queue depth climbs. |
| 5 | Verify steady-state recovery within 10 min. |

## Recovery Path C — Replay storm with worker queue saturation

| Step | Action |
|---|---|
| 1 | Sev-2 declared; engage axis-workflow + ops-sre-reliability. |
| 2 | Verify resume rate-limit is in effect: `oya_workflow_engine_resume_throttle_active == 1`. |
| 3 | If rate-limit not engaging (config bug or override): manually apply via `cargo run -p oya-dev-cli -- workflow-engine apply-resume-throttle --rate 100`. |
| 4 | Scale workers above HPA max if needed: `kubectl scale deployment/execution-engine-worker --replicas=300`. |
| 5 | Defer new run-starts: tenant-facing 429 for 30s while backlog clears. |
| 6 | Verify recovery cadence: queue depth halving every 30s. |
| 7 | Postmortem: identify why the replay storm wasn't absorbed. |

## Recovery Path D — Region-failover cold-start (post-DR-failover)

| Step | Action |
|---|---|
| 1 | DR-pair region has warm Postgres replica + warm engine workers at 0.6× capacity. |
| 2 | After DR failover (per `multi-region.md`), engine workers scale up to 1.0× primary capacity. |
| 3 | Resume-rate-limit cadence cranked: 100 runs/s/worker; durable runs resume from last checkpoint. |
| 4 | Long-running paused workflows (24h+ paused) verified: they should resume identically per deterministic replay invariant. |
| 5 | Tenant notification per `multi-region.md` §"Tenant Notification". |

## Recovery Path E — Cold-start with Postgres degraded

If Postgres is the bottleneck (rare; usually Postgres scales first):

| Step | Action |
|---|---|
| 1 | Verify Postgres read replica is keeping up. |
| 2 | Engine read-only paths (state lookup) route to read replica during cold-start. |
| 3 | Write paths (state checkpointing) bottleneck on coordinator; if backed up, defer non-essential writes. |
| 4 | Postgres autoscale (where supported) ramps workers. |

## Verification

After recovery:
- Worker queue depth < 5k.
- Step dispatch latency p99 < 200ms (per PRD).
- In-flight run completion rate returns to baseline.
- Tenant-facing dashboard shows healthy state.
- No run terminated incorrectly during cold-start (verified by audit chain integrity check).

## Post-incident updates

- Postmortem within 5 business days.
- Action: harden the resume-rate-limit (e.g., add a circuit breaker that escalates to 50/s if worker queue grows mid-throttle).
- Action: extend AC-03 (durable execution restart) test coverage if a new failure mode was discovered.
- Action: verify long-running workflow resumption test still passes after engine changes.

## References

- `microservices/workflow-engine/failure-modes.md` FM-03.
- `microservices/workflow-engine/PRD.md` FR-07, AC-03, AC-04.
- `microservices/workflow-engine/multi-region.md`.
- `microservices/workflow-engine/policy/spec-integrity.md`.
- Temporal durability docs — `docs.temporal.io/dev-guide/durability`.
