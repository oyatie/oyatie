---
doc_class: Runbook
title: State-machine deadlock detection + cycle-tear-down
microservice: workflow-engine
severity: "Sev-2 (multi-tenant cascade) / Sev-3 (single-tenant)"
status: Accepted
owner_team: axis-workflow
date: 2026-05-17
related_artifacts:
  - microservices/workflow-engine/failure-modes.md (FM-01 deadlock)
  - microservices/workflow-engine/PRD.md (state-machine BC)
  - microservices/workflow-engine/policy/spec-integrity.md
doc_status: published
---

# Runbook: State-machine deadlock resolution

## Trigger

ONE of:

1. **Auto-detected**: cycle detector emits `oya_workflow_engine_deadlock_cycle_detected_total > 0`; SLA timer on stuck runs fires.
2. **Manual report**: tenant operator reports runs stuck in `WAITING` state with cross-run wait dependency.

## Severity

- Single-tenant cycle: Sev-3.
- Cycle spans multiple tenants (via shared event-bus subscription dependency): Sev-2.
- Production-tier cascade affecting > 100 runs: Sev-2 (could escalate).

## Pre-checks

1. Identify the cycle: `cargo run -p oya-dev-cli -- workflow-engine inspect-deadlock --tenant <hash>` returns the wait-graph cycle.
2. Identify the runs in the cycle: each is in `WAITING` state with `waiting_on` pointing to another run in the cycle.
3. Determine cycle origin:
   - **Spec-induced**: spec author wrote cross-run wait without exit path.
   - **Race condition**: runs raced into a state that wasn't designed as a cycle.
   - **Subscriber dependency**: runs wait on event subscribers that themselves are stuck on these runs.

## Recovery Path A — Auto cycle-tear-down (preferred; engine-built-in)

The engine's cycle detector automatically tears down cycles by failing the youngest run:

| Step | Action |
|---|---|
| 1 | Engine detects cycle in wait-graph. |
| 2 | Tear-down primitive: identify the youngest run in the cycle (by `started_at`). |
| 3 | Fail the youngest run with `DeadlockBroken` reason; emit audit-chain seal. |
| 4 | Other runs in the cycle are unblocked; complete normally. |
| 5 | Tenant notified via subscription event. |

Verification: `oya_workflow_engine_deadlock_cycle_detected_total` decrements; affected run status `failed` with reason `DeadlockBroken`.

## Recovery Path B — Manual cycle-tear-down (when auto-detection lags or fails)

| Step | Action |
|---|---|
| 1 | Operator confirms the cycle via inspection command. |
| 2 | Manual tear-down: `cargo run -p oya-dev-cli -- workflow-engine break-cycle --tenant <hash> --run-id <youngest-run-id> --reason "<rfc>" --two-person-signature <s1> <s2>`. |
| 3 | Verify cycle broken: other runs in cycle should unblock. |
| 4 | Audit-chain seal emitted. |
| 5 | Engage spec author for permanent fix. |

## Recovery Path C — Cycle with subscriber dependency

Cross-tenant cycles where a subscription on tenant-A waits on a tenant-B publish:

| Step | Action |
|---|---|
| 1 | Engage both tenant operators; explain the cross-tenant dependency. |
| 2 | Identify the proper tear-down point: typically the dependency is in one tenant's spec; tear that one down. |
| 3 | Apply Path A or B. |
| 4 | Followup: cross-tenant dependency patterns should be re-reviewed in tenant onboarding; surface as anti-pattern. |

## Recovery Path D — Recurring cycle (spec author bug)

If the same spec_id keeps producing cycles:

| Step | Action |
|---|---|
| 1 | Engage spec-author team; identify the root cause. |
| 2 | Apply Path A or B to current cycle. |
| 3 | Deprecate the offending spec version (`runbooks/spec-rollback.md` Path A). |
| 4 | New spec version with fixed wait-graph published. |
| 5 | LEAN check `oya-governance-spec-construct-conformance` extended to catch the pattern at PR review. |

## Verification

After recovery:
- `oya_workflow_engine_deadlock_cycle_detected_total == 0`.
- Affected runs: youngest is `failed`; others complete normally OR are operator-resumed.
- Tenant-facing dashboard reflects resolution.
- Audit-chain seal log includes the cycle-break event with cause attribution.

## Post-incident updates

- Postmortem within 5 business days.
- Action: verify cycle detector lag (time-to-detect should be < 30s).
- Action: extend LEAN check at spec authoring time for the failure pattern.
- Action: tenant onboarding doc updates explaining the cross-run dependency anti-pattern.

## References

- `microservices/workflow-engine/failure-modes.md` FM-01.
- `microservices/workflow-engine/PRD.md` state-machine BC.
- `microservices/workflow-engine/policy/spec-integrity.md`.
- `microservices/workflow-engine/runbooks/spec-rollback.md`.
- Deadlock detection algorithms — Coffman conditions.
