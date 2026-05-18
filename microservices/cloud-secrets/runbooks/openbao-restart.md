---
doc_class: Runbook
title: OpenBao restart + cluster recovery
microservice: cloud-secrets
owner_team: axis-cloud-secrets + ops-sre
date: 2026-05-17
severity_default: Sev-2 (single node); Sev-1 (quorum loss)
---

# Runbook: OpenBao restart + cluster recovery

## When to use

- Single OpenBao pod unhealthy; HPA + ReplicaSet should heal automatically.
- ≥2 OpenBao pods unhealthy → Raft quorum at risk.
- ≥3 OpenBao pods unhealthy → Raft quorum lost (Sev-1).
- Postgres backend corruption (Sev-1; see §"Backend recovery").
- Region failover for DR-pair packs (Sev-1; see §"Region failover").

## §A — Single pod restart (Sev-3)

Usually self-healing. If not:

```bash
kubectl -n cloud-secrets-<pack> delete pod openbao-<n>
# Wait for ReplicaSet to recreate
kubectl -n cloud-secrets-<pack> get pods -w
```

Verify Raft cluster healthy:

```bash
kubectl -n cloud-secrets-<pack> exec openbao-0 -- bao status
kubectl -n cloud-secrets-<pack> exec openbao-0 -- bao operator raft list-peers
```

## §B — Two-pod degradation (Sev-2)

Raft can tolerate 1 of 5 peer loss for indefinite period; 2 of 5 means leader election will succeed but quorum is fragile. Investigate root cause:

```bash
# Check pod events
kubectl -n cloud-secrets-<pack> describe pod openbao-<n>
kubectl -n cloud-secrets-<pack> logs openbao-<n> --previous --tail 200

# Check node health
kubectl get nodes
kubectl describe node <node-name>
```

Common causes:
- OOMKill (raise resource.limits; document why).
- PVC pending (storage issue; engage ops-storage).
- Network partition (engage ops-net).

Apply fix; restart pods; verify quorum.

## §C — Raft quorum loss (Sev-1)

≥3 of 5 peers unreachable. Cluster cannot accept writes.

### Step 1 — Confirm

```bash
# From any healthy peer (or external client with token)
bao operator raft list-peers
# Expect: ≥3 peers showing as "voter" + healthy
```

### Step 2 — Containment

Consumer SDKs have ≤60s cache TTL; resolution continues briefly. Extend cache TTL via emergency flag if needed:

```bash
# CAREFUL: emergency mode; only for >60s outage
cargo run -p oya-cloud-secrets-secret-reference-resolver-app -- admin emergency-cache-extend \
    --pack <pack> --ttl 300 --incident-id <ulid>
# Reverts automatically at incident close
```

### Step 3 — Diagnose

```bash
# Check Postgres backend
cargo run -p oya-cloud-secrets-openbao-operator-app -- backend status --pack <pack>

# Check HSM partition (auto-unseal dependency)
cargo run -p oya-cloud-secrets-hsm-integration-app -- partition status --pack <pack>

# Check node-level events
kubectl get events -n cloud-secrets-<pack> --sort-by='.lastTimestamp' | tail 50
```

### Step 4 — Recovery options

**Option A: Restart pods + rejoin (preferred)**

```bash
# Restart in order: followers first, leader last
for i in 4 3 2 1 0; do
  kubectl -n cloud-secrets-<pack> delete pod openbao-$i
  sleep 30
  kubectl -n cloud-secrets-<pack> exec openbao-$i -- bao status
done
```

Auto-unseal via HSM should engage if HSM healthy.

**Option B: Raft peer remove + reseed**

If a peer is permanently lost:

```bash
# From a healthy peer
bao operator raft remove-peer <lost-peer-id>

# Provision a replacement peer (via openbao-operator reconciler; or manual)
kubectl -n cloud-secrets-<pack> scale statefulset openbao --replicas=4
sleep 60
kubectl -n cloud-secrets-<pack> scale statefulset openbao --replicas=5
```

**Option C: Restore from backup**

If Raft snapshot corrupt:

See §"Backend recovery" below.

### Step 5 — Verify

```bash
bao operator raft list-peers
bao status
cargo run -p oya-cloud-secrets-secret-reference-resolver-app -- health-check --pack <pack>
```

## §D — Backend recovery (Postgres corruption)

### Step 1 — Confirm corruption

```bash
# Patroni state
kubectl -n cloud-secrets-<pack> exec postgres-0 -- patronictl list

# Postgres health
kubectl -n cloud-secrets-<pack> exec postgres-0 -- pg_isready
```

### Step 2 — Patroni failover

If primary corrupt but replicas healthy:

```bash
kubectl -n cloud-secrets-<pack> exec postgres-0 -- patronictl failover
```

### Step 3 — Restore from backup (if all replicas corrupt)

```bash
cargo run -p oya-cloud-secrets-openbao-operator-app -- backend restore \
    --pack <pack> \
    --from-backup-time "<last-known-good-utc>" \
    --target-cluster postgres-<pack>
```

OpenBao must restart to pick up restored backend:

```bash
kubectl -n cloud-secrets-<pack> rollout restart statefulset/openbao
```

Auto-unseal via HSM should engage.

### Step 4 — Reconciliation

After backend restore:
- OpenBao Raft state replays from Postgres; any in-flight writes lost (≤1h RPO).
- Trigger reconciliation of namespace-controller; per-tenant-namespace-controller will re-emit any missing TenantNamespace state.
- Trigger rotation-scheduler reconciliation; missed rotation jobs are re-queued.

## §E — Region failover (DR-pair packs only)

For pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa.

### Step 1 — Confirm primary region unrecoverable

Engage OCI support; confirm region is fully down (not a transient blip).

### Step 2 — Promote DR Postgres

```bash
# From DR region operator
cargo run -p oya-cloud-secrets-openbao-operator-app -- region promote-dr \
    --pack <pack> \
    --primary-region <down-region> \
    --new-primary-region <dr-region>
```

### Step 3 — Update DNS

```bash
# Service discovery for SDK consumers
cargo run -p oya-cloud-iac-app -- dns update \
    --record "openbao-<pack>.oyatie.dev" \
    --target-region <dr-region>
```

### Step 4 — Verify consumer SDK pickup

SDK consumers use a service-discovery client; on DNS change, new resolves go to DR region. Existing connections drain.

Monitor:

```bash
cargo run -p oya-cloud-secrets-secret-reference-resolver-app -- region status --pack <pack>
```

### Step 5 — Tenant notification

Sev-1 per `incident-response.md`. Notify tenants of DR failover.

### Step 6 — Plan return-to-primary

Once primary region recovers, plan return-to-primary at next maintenance window. Steps reverse: Patroni replication primary ← DR; OpenBao Raft re-formation; DNS revert.

## Verification (post-recovery)

```bash
# Cluster healthy
bao status
bao operator raft list-peers

# Resolve operations succeed
cargo run -p oya-cloud-secrets-secret-reference-resolver-app -- bench resolve \
    --pack <pack> --acceptance "p99 ≤ 25ms"

# Audit-chain consistent
cargo run -p oya-audit-chain-app -- verify-seal --pack <pack> --window "last 1h"
```

## References

- `microservices/cloud-secrets/failure-modes.md` FM-01 + FM-12
- `microservices/cloud-secrets/multi-region.md`
- `microservices/cloud-secrets/incident-response.md`
- OpenBao operator + Raft documentation
- Patroni HA documentation
- OCI region failover documentation
