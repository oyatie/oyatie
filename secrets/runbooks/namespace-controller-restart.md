---
doc_class: Runbook
title: Per-tenant namespace controller restart
microservice: cloud-secrets
owner_team: axis-cloud-secrets
date: 2026-05-17
severity_default: Sev-2
---

# Runbook: per-tenant-namespace-controller restart

## When to use

- `NamespaceProvisioningStuck` event fired.
- `cloud_secrets_namespace_provisioning_lag_seconds > 600` (10 min).
- Controller pods crash-looping.
- Tenant onboarding flow stalled (ops report).

## §A — Diagnosis

### Step 1 — Pod health

```bash
kubectl -n cloud-secrets-<pack> get pods -l app=per-tenant-namespace-controller -o wide
kubectl -n cloud-secrets-<pack> describe pod <pod>
kubectl -n cloud-secrets-<pack> logs <pod> --tail 200
kubectl -n cloud-secrets-<pack> logs <pod> --previous --tail 200
```

### Step 2 — Leader election

```bash
kubectl -n cloud-secrets-<pack> get lease per-tenant-namespace-controller-leader
```

If lease is held by a crashed pod, lease expiry (default 15s) lets a new replica acquire.

### Step 3 — Backlog

```bash
cargo run -p cloud-secrets-per-tenant-namespace-controller-app -- backlog --pack <pack>
```

Output: pending `TenantRegistered`, `TenantDeprovisioned`, `MicroserviceRegistered` events.

## §B — Common causes + fixes

### B.1 — RBAC drift

Controller needs Kubernetes ServiceAccount + OpenBao policy `namespace-admin`. Check:

```bash
# Kubernetes
kubectl -n cloud-secrets-<pack> get sa per-tenant-namespace-controller -o yaml
kubectl auth can-i --as=system:serviceaccount:cloud-secrets-<pack>:per-tenant-namespace-controller \
    get,list,patch namespaces

# OpenBao
vault token capabilities <controller-token> sys/namespaces/
# Expect: create, read, update, delete
```

If drift detected: re-apply IaC.

```bash
kubectl -n cloud-secrets-<pack> apply -k microservices/cloud-secrets/iac/kustomize/overlays/pack-<pack>
```

### B.2 — OpenBao policy error

Controller may panic on a malformed namespace policy template. Check logs for stack trace; identify offending template; fix in PR.

### B.3 — Workflow event consumer lag

Controller consumes `TenantRegistered` from `tenancy` µservice. If event-bus consumer lag is high:

```bash
cargo run -p cloud-secrets-per-tenant-namespace-controller-app -- consumer-lag --pack <pack>
```

If lag > 10min: scale controller replicas:

```bash
kubectl -n cloud-secrets-<pack> scale deployment per-tenant-namespace-controller --replicas 4
```

(Default is 2; max recommended 4 to avoid OpenBao API throttling.)

### B.4 — Idempotency bug (rare)

If controller crash-loops on a specific event:

```bash
# Identify the event
kubectl -n cloud-secrets-<pack> logs <pod> --previous | grep "event_id"

# Quarantine the event (skip and re-enqueue manually after fix)
cargo run -p cloud-secrets-per-tenant-namespace-controller-app -- skip-event \
    --event-id <id> \
    --reason "bug-quarantine" \
    --reroute-to "dead-letter-queue"
```

File bug + fix; re-enqueue from DLQ after fix.

## §C — Restart sequence

```bash
# Drain existing controller
kubectl -n cloud-secrets-<pack> rollout restart deployment/per-tenant-namespace-controller

# Watch
kubectl -n cloud-secrets-<pack> rollout status deployment/per-tenant-namespace-controller
```

Reconciliation is idempotent; restart resumes from last-applied state. Pending `TenantRegistered` events drain at controller's reconcile cadence.

## §D — If both replicas down

```bash
kubectl -n cloud-secrets-<pack> scale deployment per-tenant-namespace-controller --replicas 0
sleep 10
kubectl -n cloud-secrets-<pack> scale deployment per-tenant-namespace-controller --replicas 2
```

Validate first pod elects leader; second pod becomes standby.

## §E — Rollback if recent deploy

```bash
kubectl -n cloud-secrets-<pack> rollout undo deployment/per-tenant-namespace-controller
```

## Verification

```bash
# Lag clears
cargo run -p cloud-secrets-per-tenant-namespace-controller-app -- backlog --pack <pack>
# Expect: < 10

# Tenant onboard test
cargo run -p cloud-secrets-per-tenant-namespace-controller-app -- onboard-test \
    --pack <pack> \
    --tenant-id tenant:test-onboard-<ulid> \
    --expect-completion-within 30s

# Audit-chain has NamespaceProvisioned events
cargo run -p audit-chain-app -- query \
    --event-type NamespaceProvisioned \
    --since "10 minutes ago"
```

## References

- `microservices/cloud-secrets/failure-modes.md` FM-05
- `microservices/cloud-secrets/IP-012-per-tenant-namespace-controller.md`
- `microservices/cloud-secrets/PRD.md` FR-04
