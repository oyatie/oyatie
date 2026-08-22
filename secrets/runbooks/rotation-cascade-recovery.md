---
doc_class: Runbook
title: Rotation cascade recovery
microservice: cloud-secrets
owner_team: axis-cloud-secrets
date: 2026-05-17
severity_default: Sev-2
---

# Runbook: Rotation cascade recovery

## When to use

- `RotationOverdue` event fired.
- `cloud_secrets_rotation_overdue_total > 0` for ≥5 min.
- Cascade rotation stuck (parent rotated; dependents not catching up).
- Scheduler worker pod crash-looping.

## §A — Single rotation overdue (Sev-2)

### Step 1 — Identify

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- list-overdue \
    --pack <pack> --since "1 hour ago"
```

### Step 2 — Diagnose

Common causes:
- HSM partition unavailable → see `hsm-key-rotation.md`.
- OpenBao policy mis-author → see `secret-leak-detected.md` §"Policy mis-author".
- Dependent rotation deadlock → see §B below.
- Scheduler worker crash → see §C below.

### Step 3 — Manual rotation (after diagnosis)

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- rotate \
    --path "secret/<tenant>/<microservice>/<name>" \
    --priority high \
    --reason "manual-recovery"
```

Audit-emit captures manual rotation event.

## §B — Cascade deadlock

Cascade dependencies: KEK → DEKs → derived signing keys → API keys. A cycle in the dependency graph (which is a bug) would create deadlock. More commonly: a parent rotation stalls, blocking all dependents.

### Step 1 — Examine cascade DAG

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- cascade-dag \
    --root "secret/<tenant>/<microservice>/<name>" \
    --visualize
```

Output: GraphViz DOT of the dependency DAG; identify the stalled node.

### Step 2 — Manual unblock

Options:
- Force-complete parent rotation (rotation-state-machine has manual override; audit-emit override).
- Or, break cycle if true cycle detected (file bug; should be impossible at scheduler-design level).

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- override-rotation-state \
    --path "secret/<tenant>/<microservice>/<name>" \
    --to-state "completed" \
    --justification "<text>" \
    --approval-witness-1 <spiffe> \
    --approval-witness-2 <spiffe>
```

### Step 3 — Re-trigger dependents

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- cascade-retry \
    --root "secret/<tenant>/<microservice>/<name>"
```

## §C — Scheduler worker crash-loop

### Step 1 — Inspect

```bash
kubectl -n cloud-secrets-<pack> get pods -l app=key-rotation-scheduler
kubectl -n cloud-secrets-<pack> logs <pod> --previous --tail 200
```

### Step 2 — Identify root cause

Common patterns:
- Panic on malformed RotationPolicy YAML → fix policy via PR.
- HSM client init failure → see `hsm-key-rotation.md`.
- OpenBao auth failure → SPIFFE/SVID issue; engage ops-security.

### Step 3 — Rollback if recent deploy

```bash
kubectl -n cloud-secrets-<pack> rollout undo deployment/key-rotation-scheduler
```

### Step 4 — Restart + verify

```bash
kubectl -n cloud-secrets-<pack> rollout restart deployment/key-rotation-scheduler
```

Monitor:

```bash
# Pod health
kubectl -n cloud-secrets-<pack> get pods -l app=key-rotation-scheduler -w

# Queue depth
cargo run -p cloud-secrets-key-rotation-scheduler-app -- queue-depth --pack <pack>
```

## §D — Rotation storm (too many concurrent rotations)

If many secrets become due simultaneously (e.g., after a long rotation pause):

### Step 1 — Throttle

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- set-concurrency \
    --pack <pack> --max 50  # tune based on observed HSM headroom
```

### Step 2 — Drain in priority order

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- drain-queue \
    --pack <pack> \
    --order-by priority,due_at
```

### Step 3 — Apply jitter for future storms

Verify scheduler config has `±10% jitter` enabled to prevent recurrence.

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- config show --pack <pack> | grep jitter
```

If absent, file PR to enable.

## Verification (post-recovery)

```bash
# No overdue rotations
cargo run -p cloud-secrets-key-rotation-scheduler-app -- list-overdue --pack <pack>
# Expect: empty

# Scheduler queue drained
cargo run -p cloud-secrets-key-rotation-scheduler-app -- queue-depth --pack <pack>
# Expect: < 10

# Audit-chain has RotationCompleted events for each
cargo run -p audit-chain-app -- query --event-type SecretRotated --since "1 hour ago" --pack <pack>
```

## References

- `microservices/cloud-secrets/failure-modes.md` FM-03 + FM-04
- `microservices/cloud-secrets/threat-model.md` T-T-01 + T-D-04
- `microservices/cloud-secrets/IP-010-key-rotation-scheduler-worker.md`
- `microservices/cloud-secrets/policy/data-residency.md` "KEK Lifecycle"
