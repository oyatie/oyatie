---
doc_class: Runbook
title: Stuck workflow run recovery
microservice: workflow-engine
severity: "Sev-3 (single-tenant run)"
status: Accepted
owner_team: axis-workflow
date: 2026-05-17
related_artifacts:
  - microservices/workflow-engine/failure-modes.md (FM-11)
  - microservices/workflow-engine/incident-response.md
  - microservices/workflow-engine/PRD.md (FR-09 SLA timer; FR-12 operator pause/resume/cancel)
doc_status: published
---

# Runbook: Stuck workflow run recovery

## Trigger

`oya_workflow_engine_run_stuck_seconds > 3600` for one or more runs OR explicit tenant-operator report via support channel.

## Severity

- Single run stuck < 24h: Sev-3.
- Multiple runs (>10) stuck for same spec_id: Sev-2 (suggests spec defect; consider rollback).
- Production-tier critical workflow stuck > 24h with tenant escalation: Sev-2.

## Impact

Affected run(s) will never complete without intervention. Downstream subscribers waiting on completion event remain blocked until cancel + alternative path or operator override.

## Pre-checks

1. Confirm the run is genuinely stuck (not just slow): `cargo run -p oya-dev-cli -- workflow-engine inspect-run --tenant <hash> --run-id <id>`.
2. Identify the stuck step: review run state's `current_step_index` and `current_state`; the step body should expose a `waiting_on` field for transparency.
3. Determine root cause class:
   - **Wait-on-event that will never fire** (subscriber dependency upstream is dead).
   - **Spec-induced infinite-wait** (spec author bug; e.g., guard condition never satisfiable).
   - **SLA timer arming race** (timer didn't fire when it should have).
   - **External-action retry exhaustion without dead-letter** (poorly-configured retry policy).

## Recovery Path A — Single run, root cause is recoverable (e.g., transient downstream failure)

| Step | Action |
|---|---|
| 1 | Identify the failed transient dep; fix upstream. |
| 2 | Signal the run: `cargo run -p oya-dev-cli -- workflow-engine signal-run --tenant <hash> --run-id <id> --signal retry` |
| 3 | Verify the run advances: state should transition out of `WAITING` within 30s. |
| 4 | Monitor through completion. |

## Recovery Path B — Single run, root cause is unrecoverable (spec bug; will-never-complete)

| Step | Action |
|---|---|
| 1 | Notify tenant + obtain cancel approval. |
| 2 | Operator-initiated cancel: `cargo run -p oya-dev-cli -- workflow-engine cancel-run --tenant <hash> --run-id <id> --reason "<rfc>" --two-person-signature <signer1> <signer2>` |
| 3 | Verify cancel succeeded; `oya_workflow_engine_workflow_cancelled_total` increments; audit-chain seal emitted. |
| 4 | File spec-fix task with the tenant; new runs against fixed spec version (per `policy/spec-integrity.md` immutability — old runs against old spec are not retro-fixed). |
| 5 | Postmortem if production-tier impact: identify if spec authoring lane could have caught the bug pre-publication. |

## Recovery Path C — Many runs stuck under same spec_id (spec-defect class)

| Step | Action |
|---|---|
| 1 | Declare Sev-2; engage axis-workflow + spec-author team. |
| 2 | Verify all affected runs share the same spec_id + version_sha. |
| 3 | Deprecate the spec version: `cargo run -p oya-dev-cli -- workflow-engine deprecate-spec --tenant <hash> --spec-id <id> --version-sha <sha> --reason "<rfc>"` |
| 4 | Bulk-cancel affected runs with audit emission: `cargo run -p oya-dev-cli -- workflow-engine bulk-cancel-runs --tenant <hash> --spec-id <id> --version-sha <sha> --reason "<rfc>" --two-person-signature <s1> <s2>` |
| 5 | Tenant notification including impact + remediation. |
| 6 | If production-tier with regulatory implication (HIPAA/PIPA/etc.): engage Privacy Lead + start notification chain per `incident-response.md`. |

## Recovery Path D — SLA timer didn't fire when it should have

| Step | Action |
|---|---|
| 1 | Verify the timer record in Valkey: `redis-cli HGETALL oya:workflow:sla_timer:<run_id>` |
| 2 | If timer is missing: indicates a Valkey Sentinel outage during arming. Re-arm via `cargo run -p oya-dev-cli -- workflow-engine rearm-timer --run-id <id> --reason "<rfc>"` |
| 3 | If timer exists but expired without firing: indicates a worker-leader-election failure. Restart timer-firing worker pods. |
| 4 | Postmortem: harden the timer-firing path (e.g., add Postgres-backed durable timer mirror as Valkey fallback). |

## Recovery Path E — Operator override for genuine stuck-cannot-cancel cases

If steps above fail (rare; root cause should be identifiable), operator-override is the last resort with full audit:

| Step | Action |
|---|---|
| 1 | Two operators independently confirm cancellation justification (`rfc` describing the reason). |
| 2 | Force-cancel via emergency primitive: `cargo run -p oya-dev-cli -- workflow-engine force-cancel-run --tenant <hash> --run-id <id> --reason "<rfc>" --emergency-override --two-person-signature <s1> <s2>` |
| 3 | Audit-chain seal emitted with reason; tenant notified; postmortem mandatory. |

## Verification

After recovery:
- Stuck runs counter `oya_workflow_engine_run_stuck_seconds_max == 0` for affected scope.
- Tenant-facing run history reflects the resolution (completion or cancellation).
- Audit-chain seal log includes the resolution event.
- If spec-fix path: new spec version published; tenant migrates new runs forward.

## Post-incident updates

- Postmortem within 5 business days.
- Action: harden spec author tooling (CI lane to catch obvious infinite-wait patterns at PR review).
- Action: extend deterministic-replay test suite to include the failure pattern as a guard against regression.
- Action: tenant onboarding doc updates if a common pattern.

## References

- `microservices/workflow-engine/failure-modes.md` FM-11.
- `microservices/workflow-engine/incident-response.md`.
- `microservices/workflow-engine/PRD.md` FR-09 + FR-12.
- `microservices/workflow-engine/policy/spec-integrity.md` (spec lifecycle).
