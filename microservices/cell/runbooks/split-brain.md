---
doc_class: Runbook
title: Split-brain — cell-registry HA failover with diverged state
microservice: cell
severity: "Sev-1 (data consistency risk)"
status: Accepted
owner_team: cloud-k8s + axis-cell-substrate + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/cell/failure-modes.md (FM-11 split-brain)
  - microservices/cell/multi-region.md
doc_status: published
---

# Runbook: Split-Brain in Cell-Registry HA

## Trigger

`postgres_write_quorum_break == 1` for the per-pack cell-registry Postgres cluster. CloudNativePG / Patroni detected loss of write quorum due to network partition between primary candidates.

## Severity

**Sev-1**. Even if no diverged writes occurred, this is a data-consistency risk class and must be treated as Sev-1.

## Pre-checks

1. Confirm split-brain: `kubectl -n cell-control-plane get cluster <pg-cluster> -o yaml` shows `quorumBreak: true`.
2. Identify candidate primaries: `kubectl -n cell-control-plane get pods -l postgres-cluster=<name>`. Two pods may both claim "leader" status.
3. Check network: `kubectl exec ... -- ping <other-pod>`. Are they actually partitioned, or is this a perceived partition due to controller bug?
4. Verify audit-chain seal log: `oya cell audit-log --window 30m` for any writes during the partition window.

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; open `#inc-<id>`; assign IC; engage ops-security + cloud-k8s + axis-cell-substrate. | ≤ 5 min |
| 2 | Force both candidates to read-only mode: `kubectl exec ... -- psql -c "ALTER SYSTEM SET default_transaction_read_only = on; SELECT pg_reload_conf();"` on both pods. | ≤ 5 min |
| 3 | Stop accepting new writes at PgBouncer: `kubectl -n cell-control-plane scale deployment pgbouncer --replicas=0`. | ≤ 1 min |
| 4 | Investigate diverged writes during partition window: compare `cell_assignments` table row checksums between pods. | ≤ 15 min |
| 5 | Verify audit-chain ledger: `registry/cell-assignment.jsonl` is the authoritative external record; reconcile diverged Postgres writes against this. | ≤ 30 min |

## Reconciliation

### If no diverged writes occurred during partition window

| Step | Action |
|---|---|
| 1 | Pick a preferred primary (highest LSN in WAL log; or most recent successful commit). |
| 2 | Force-fence the loser: `kubectl exec loser-pod -- pg_ctl stop -m immediate` + remove from cluster. |
| 3 | Loser rejoins as replica: `kubectl exec ... -- pg_basebackup ... -- ... --create-slot`. |
| 4 | Verify quorum restored: `kubectl get cluster <name>` shows `quorumBreak: false`. |
| 5 | Re-enable writes: PgBouncer scale back to 2; `ALTER SYSTEM SET default_transaction_read_only = off`. |

### If diverged writes occurred during partition window

| Step | Action |
|---|---|
| 1 | Engage council-architecture for operator decision: which writes to keep, which to reject. |
| 2 | Reconcile via union-merge ledger: `registry/cell-assignment.jsonl` is append-only; the writes there are authoritative. Postgres state diverged from ledger is provisional. |
| 3 | For each diverged Postgres write: check if a matching audit-chain seal exists. |
| 4a | If seal exists: keep the write; ensure both Postgres replicas converge on this row. |
| 4b | If no seal: reject the write; emit `RolledBackDivergedWrite` audit event. |
| 5 | Verify post-reconciliation: Postgres state matches union-merged ledger. |
| 6 | Force-fence the loser; rejoin as replica. |
| 7 | Re-enable writes. |
| 8 | Postmortem within 2 business days (more urgent than standard 5 days due to data-consistency severity). |

## Tenant Communications

| Phase | Action |
|---|---|
| Initial declaration (≤ 5 min) | Status page: "We are investigating a cell-substrate consistency issue in <pack>. New cell-assignment writes briefly paused. Existing tenants unaffected." |
| Mitigation in progress (≤ 30 min) | Status page updated; tenant operator email if Sev-1 escalates beyond 30 min. |
| Resolution | Status page: "Resolved." Tenant operator email confirming no data loss; per-changeset evidence published. |
| Regulatory notification | If diverged writes affected any tenant data: GDPR Art. 33 72h + KR PIPA Art. 34 72h + HIPAA §164.404 if applicable. Engage council-privacy. |

## Verification

After completion:
- Single primary in cluster; replicas synced.
- `postgres_write_quorum_break == 0`.
- Postgres state matches `registry/cell-assignment.jsonl` (union-merged ledger).
- No unreconciled diverged writes.
- New writes flowing; PgBouncer healthy.
- Audit-chain integrity verified.

## Post-incident updates

- Postmortem within 2 business days.
- Action items: typically include "why did the partition happen?", "why did quorum-loss detection take N minutes?", "should we enable additional fencing primitives?".
- Network topology review with cloud-k8s.
- Annual chaos-engineering drill: inject simulated partition; verify reconciliation procedure.

## References

- `microservices/cell/failure-modes.md` FM-11.
- `microservices/cell/multi-region.md`.
- CloudNativePG cluster recovery — `cloudnative-pg.io/documentation/current/cluster_conf/`.
- Postgres pg_basebackup — `postgresql.org/docs/current/app-pgbasebackup.html`.
- Patroni split-brain mitigation — `patroni.readthedocs.io/en/latest/`.
