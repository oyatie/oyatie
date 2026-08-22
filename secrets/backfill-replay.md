---
doc_class: BackfillReplay
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: axis-cloud-secrets + ops-sre
related_artifacts:
  - microservices/cloud-secrets/runbooks/openbao-restart.md
  - microservices/cloud-secrets/runbooks/audit-emission-backlog.md
  - microservices/cloud-secrets/failure-modes.md
doc_status: published
---

# Backfill + Replay: cloud-secrets µservice

## Purpose

Define the deterministic backfill + replay procedures for cloud-secrets state when:
- Postgres backend restored from backup (RPO ≤1h gap).
- audit-emitter bridge lag drains after audit-chain recovery.
- per-tenant-namespace-controller reconciles missing state after region failover.
- key-rotation-scheduler resumes after scheduler outage.

cloud-secrets is **mostly stateless at the application layer** — OpenBao is the source of truth for secrets; Postgres-HA backs OpenBao; HSM holds KEK material. Backfill is bounded by RPO of the backing stores.

## Replay Classes

### Class 1: OpenBao Raft snapshot restore

When Raft snapshot or backend storage corrupt, restore from a known-good Raft snapshot.

```bash
cargo run -p cloud-secrets-openbao-operator-app -- backend restore \
    --pack <pack> \
    --from-backup-time "<utc>" \
    --target-cluster openbao-<pack>
```

Properties:
- RPO ≤1h (incremental backup cadence).
- Audit events for the gap window are durable on local audit-device files; bridge re-injects them on bridge restart.
- Active SecretReferences with versions in the gap may be re-issued (lost-write); affected versions are NEVER re-used (new version increment).

### Class 2: audit-emitter bridge replay

When audit-chain recovers from outage, audit-emitter replays from local audit-device file:

```bash
cargo run -p cloud-secrets-audit-emitter-app -- replay \
    --pack <pack> \
    --from-file /var/log/openbao/audit/audit.log.YYYY-MM-DD \
    --bridge-endpoint audit-chain.<pack>.svc.cluster.local:443
```

Properties:
- Local audit-device file is durable (capped at 10 GiB; rotates).
- Replay is idempotent: audit-chain dedup-keys on `(event_id, signature)`.
- Replay throughput: ≥10k events/sec; drains a 24h backlog within ~1h.

### Class 3: per-tenant-namespace-controller reconcile

After region failover or controller restart, reconcile namespace state:

```bash
cargo run -p cloud-secrets-per-tenant-namespace-controller-app -- reconcile \
    --pack <pack> \
    --tenancy-source tenancy.<pack>.svc.cluster.local
```

Properties:
- Reconciliation is idempotent.
- Compares tenancy µservice's authoritative tenant list against OpenBao namespace inventory.
- Missing namespaces re-provisioned; orphan namespaces flagged for review (NOT auto-deleted; manual review).

### Class 4: key-rotation-scheduler missed-rotation reconcile

After scheduler outage, identify missed rotations and re-queue:

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- reconcile-missed \
    --pack <pack> \
    --grace-window 30m
```

Properties:
- Missed rotations within the grace window are re-queued at next normal slot.
- Missed rotations beyond grace window are queued with `priority=high` and emit `RotationOverdue`.

### Class 5: revocation-push consumer catch-up

When a consumer SDK reconnects after disconnect:

```rust
// SDK behaviour (illustrative)
let last_event_id = sdk.last_seen_revocation_event_id();
sdk.reconnect_revocations(since = last_event_id);
// → server replays revocation events since last_event_id
```

Properties:
- OpenBao revocation-push channel retains a ring buffer of recent revocations (last 10 min by default; configurable).
- If consumer offline > ring-buffer-window: full cache flush on reconnect (defensive); audit-emit `consumer_full_flush`.

## Replay Verification

After replay:

```bash
# OpenBao state
cargo run -p cloud-secrets-openbao-operator-app -- cluster verify --pack <pack>

# audit-chain integrity
cargo run -p audit-chain-app -- verify-seal --pack <pack> --window <replay-window>

# Namespace reconciliation
cargo run -p cloud-secrets-per-tenant-namespace-controller-app -- inventory-diff --pack <pack>

# Rotation queue depth
cargo run -p cloud-secrets-key-rotation-scheduler-app -- queue-depth --pack <pack>
```

## RPO + RTO Summary

| Replay class | RPO | RTO |
|---|---|---|
| OpenBao Raft snapshot restore | ≤1h | ≤30 min |
| audit-emitter bridge replay | ≤1s (local device durable) | ≤1h to drain 24h backlog |
| namespace-controller reconcile | 0 (state in OpenBao + tenancy) | ≤10 min |
| rotation-scheduler reconcile-missed | 0 (policy in OpenBao; state derivable) | ≤30 min |
| revocation-push consumer catch-up | ≤10 min (ring buffer) | seconds |

## Non-replayable State

Some state is intentionally not replayable:
- HSM-resident KEK material: NEVER replicated outside HSM; lost partition = ceremony required.
- Resolved secret values: transient in consumer process memory; not replayed.
- Cache contents: TTL ≤60s; not replayed.

## References

- `microservices/cloud-secrets/multi-region.md`
- `microservices/cloud-secrets/failure-modes.md` FM-01, FM-04, FM-12
- `microservices/cloud-secrets/runbooks/openbao-restart.md`
- `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`
