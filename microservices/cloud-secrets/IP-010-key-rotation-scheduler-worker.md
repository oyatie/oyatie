---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-010-key-rotation-scheduler-worker
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [rotation-e2e, cascade-e2e]
---

# IP-010: key-rotation-scheduler worker

## Intent

Ship the rotation scheduler: cron-driven rotation of secrets per declared policy; cascade rotation of dependents; stuck-rotation detection.

## ChangeSet boundary

Seven new crates: kernel, domain, usecase, api, adapter, worker, app.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-key-rotation-scheduler-kernel/` | `RotationPolicy`, `RotationJob`, `CascadeDependency`, `RotationStateMachine` |
| `…/oya-cloud-secrets-key-rotation-scheduler-domain/` | pure topo-sort over cascade DAG; jitter arithmetic |
| `…/oya-cloud-secrets-key-rotation-scheduler-usecase/` | orchestrators: schedule, execute, cascade |
| `…/oya-cloud-secrets-key-rotation-scheduler-api/` | typed contracts |
| `…/oya-cloud-secrets-key-rotation-scheduler-adapter/` | OpenBao + audit-emitter adapter wiring |
| `…/oya-cloud-secrets-key-rotation-scheduler-worker/` | long-lived worker binary |
| `…/oya-cloud-secrets-key-rotation-scheduler-app/` | composition root |
| 7× catalog yamls | create |

## Rotation State Machine

```text
Scheduled → InProgress → Rotated → CascadeQueued → CascadeInProgress → Completed
                ↓                          ↓
             Failed (retry ×3 → Overdue → Page)
```

## Acceptance Gates

```bash
cargo nextest run -p 'oya-cloud-secrets-key-rotation-scheduler-*'
# Rotation e2e
cargo nextest run -p oya-cloud-secrets-key-rotation-scheduler-worker --features e2e
# Cascade e2e (chain of 3 dependents)
cargo nextest run --features cascade-e2e
```

## Test Plan

- Single-rotation: schedule → complete within SLA.
- Cascade DAG: rotate root → leaves rotate in topo order.
- Stuck detection: simulated HSM unavailability → RotationOverdue event after T+1d.
- Storm: 100 concurrent rotations → throttle respected.

## Halt Conditions

- Cascade rotation breaks dependent µservices — BLOCKER.

## Next IP

`IP-011-hsm-integration-adapter-hsm.md`
