---
doc_class: Runbook
title: Fleet-state recovery (Postgres / CRD divergence)
microservice: foundry-supervisor
severity: "Sev-2 (escalates to Sev-1 if drift is wide-scale or affects autonomy decisions)"
status: Accepted
owner_team: axis-foundry-control-plane + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-03, FM-11)
  - microservices/intelligence/policy/supervisor-isolation.md (TI-K-05 drift detector)
doc_status: published
---

# Runbook: Fleet-state recovery

## Trigger

ONE of:

1. **Nightly drift detector** emits `oya_supervisor_fleet_state_divergence_total > 0`.
2. **Operator alert** — manual observation of divergence (e.g., REST returns stale fleet count vs `kubectl get agents`).
3. **Drain stuck** (FM-11) — in-flight workers not completing within drain grace period.

## Severity

- Single-tenant divergence: Sev-2.
- Multi-tenant or autonomy-affecting divergence: Sev-1.
- Drain-stuck on small fleet: Sev-2; large fleet: Sev-1.

## Source-of-truth invariant

**Kubernetes CRDs are source-of-truth.** Postgres + Valkey are derived state. Reconcile always flows CRD → Postgres / Valkey, never the reverse.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>` Slack; assign IC; declare severity | ≤ 5 min |
| 2 | Inspect divergence scope: `kubectl get agentdeployment -A` vs `SELECT * FROM deployment_history WHERE ...` | ≤ 5 min |
| 3 | Block writes via REST circuit-breaker: `cargo run -p oya-dev-cli -- supervisor circuit-breaker open --reason "fleet-state-recovery"` | ≤ 1 min |
| 4 | Trigger reconcile replay: `cargo run -p oya-dev-cli -- supervisor reconcile --scope <ns-pattern> --source crd` (replays all CRD events through Operator → Postgres) | ≤ 30 min depending on fleet size |
| 5 | Verify divergence cleared: drift detector returns 0 | ≤ 5 min |
| 6 | Close REST circuit-breaker: `cargo run -p oya-dev-cli -- supervisor circuit-breaker close` | ≤ 1 min |
| 7 | Verify normal traffic: REST QPS recovers; reconcile lag returns to baseline | ≤ 10 min |
| 8 | If divergence pattern suggests Postgres corruption: engage ops-security; consider WAL replay | escalation |
| 9 | Postmortem within 5 business days | – |

## Drain stuck (FM-11)

| Step | Action |
|---|---|
| 1 | Inspect `DrainHandle.in_flight_count` for the affected fleet |
| 2 | Identify hung agents: `kubectl get pods -n foundry-tenant-<id> -l drain-state=in-flight` |
| 3 | Engage tenant DPO (their workload is hung; they may have visibility) |
| 4 | Wait grace period (default 30 min); if no completion, proceed |
| 5 | Force-terminate: `cargo run -p oya-dev-cli -- supervisor force-drain --fleet <id> --reason "drain-stuck-grace-period-exceeded"`. Emits `AgentEvicted{reason=force_drain}` for each. |
| 6 | Verify drain completes; `DrainHandle.in_flight_count == 0` |
| 7 | Audit-chain seal + per-changeset evidence |

## Verification

- Drift detector returns 0.
- `oya_supervisor_fleet_state_divergence_total == 0` for ≥ 1 h.
- CRD + Postgres + Valkey all consistent.
- `oya_supervisor_drain_in_flight_count` matches expected.
- Per-changeset evidence updated.

## Post-incident updates

- Postmortem with 5-whys against FM-ID.
- Action items: typically include "why did Postgres + CRD diverge?" and "should the drift detector cadence increase?".

## References

- `failure-modes.md` FM-03, FM-11.
- `policy/supervisor-isolation.md` TI-K-05.
- `incident-response.md` §"Sev-2 response".
- Kubernetes Operator reconcile pattern.
