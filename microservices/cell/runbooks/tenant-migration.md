---
doc_class: Runbook
title: Tenant migration — move tenant from cell A → cell B
microservice: cell
severity: "Sev-2 (regression) / Sev-3 (planned)"
status: Accepted
owner_team: axis-cell-substrate + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/cell/failure-modes.md (FM-06 migration race)
  - microservices/cell/PRD.md (FR-05 tenant migration)
  - Bominal ADR-0009 §"Live migration"
doc_status: published
---

# Runbook: Tenant Migration

## Trigger

ONE of:

1. **Automated**: scheduler rebalance triggers a migration plan (per `cell-rebalance.md` Recovery Path A).
2. **Manual**: operator declares migration via `oya cell migrate-tenant --tenant <id> --to <cell>`.
3. **Pack rehome**: rare; tenant changes HQ jurisdiction; cross-pack move with SCC + 2-person rule (see `policy/data-residency.md` §"SCC exception").

## Severity

- Operational migration (scale + rebalance): Sev-3 (planned recovery).
- Migration triggered by regression / incident: Sev-2.

## Pre-checks

1. Identify tenant: `oya cell get-assignment --tenant <id>` → current `(cell_id, pack, state)`.
2. Identify target cell: must be in same pack as tenant.pack (cross-pack move requires SCC path; see `policy/data-residency.md`).
3. Verify no active migration: `oya cell migration-status --tenant <id>` empty.
4. Verify target cell within [40%, 80%] band post-migration projection.
5. Verify SPIRE + Postgres + Cluster API healthy in the pack.
6. If cross-pack: confirm SCC on file at `legal/transfer-register.md`; confirm 2-person rule operators present.

## Migration Procedure (≤ 10 min p99 per Bominal ADR-0009)

### Phase 1: Plan + Lock (≤ 30s)

| Step | Action |
|---|---|
| 1 | CLI invokes `tenant-assignment-usecase::CreateMigrationPlan(tenant_id, source_cell, target_cell, reason)`. |
| 2 | Postgres advisory-lock acquired on `tenant_id` (FM-06 mitigation: prevents concurrent migration). |
| 3 | MigrationPlan row inserted with state `planned`; idempotency key returned. |
| 4 | Emit `MigrationPlanned` event; audit-chain seal. |

### Phase 2: Drain (≤ 2 min)

| Step | Action |
|---|---|
| 5 | Tenant's workload pods in source_cell drain new requests (LB stops sending traffic). |
| 6 | In-flight requests complete (drain budget 60s); reject after. |
| 7 | Tenant marked `draining` in cell_assignments table. |
| 8 | Emit `MigrationDraining` event. |

### Phase 3: Copy (≤ 5 min p99)

| Step | Action |
|---|---|
| 9 | Tenant Postgres schema copied source→target (logical-replication or `pg_dump` for small tenants). |
| 10 | Tenant S3 prefix copied source→target via OCI Object Storage CopyObject (CRC-validated). |
| 11 | Tenant OpenBao credentials rotated: old source-cell credentials revoked; new target-cell credentials issued. |
| 12 | Tenant workload pods scheduled in target_cell K8s namespace (per cell.spec). |
| 13 | Migration checkpoint persisted (resumable from this point if interrupted). |
| 14 | Emit `MigrationCopying` events at each major step. |

### Phase 4: Cutover (≤ 1 min)

| Step | Action |
|---|---|
| 15 | tenant_assignment row updated: `cell_id = target_cell` (Postgres transaction; RLS validates target cell.pack == tenant.pack). |
| 16 | LB routes tenant traffic to target_cell. |
| 17 | Workload µservice caches invalidated (TTL 60s; subscribers receive `CellAssigned` event). |
| 18 | MigrationPlan state → `cutover_complete`. |
| 19 | Emit `CellRebalanced` event; audit-chain seal. |

### Phase 5: Cleanup (≤ 2 min)

| Step | Action |
|---|---|
| 20 | Tenant pods in source_cell scaled to 0 (after 30s grace). |
| 21 | Tenant Postgres schema in source_cell marked for retention (30d soft-delete window). |
| 22 | Tenant S3 prefix in source_cell marked for retention. |
| 23 | MigrationPlan state → `completed`; advisory-lock released. |
| 24 | Emit `MigrationCompleted` event. |

## Recovery Path A — Migration stuck in Copy phase

| Step | Action |
|---|---|
| 1 | Inspect MigrationPlan checkpoint: `oya cell migration-status --tenant <id>` shows last successful step. |
| 2 | If pg_dump stuck: increase timeout; inspect for connection issues. |
| 3 | If S3 copy stuck: verify both source + target buckets healthy; verify CRC mismatch not blocking. |
| 4 | Resume from checkpoint: `oya cell migration-resume --tenant <id>`. |
| 5 | If resume fails repeatedly: abort migration via `oya cell migration-abort --tenant <id>`. Tenant remains in source_cell (no data loss). |

## Recovery Path B — Concurrent Migration (FM-06)

Cause: two operators issued migration simultaneously.

| Step | Action |
|---|---|
| 1 | Second migration attempt receives `MigrationInProgress` error with existing migration_id. |
| 2 | Verify existing migration is what was intended; if yes, observe progress. |
| 3 | If existing migration is wrong (different target_cell): abort it (`oya cell migration-abort`) then start correct one. |
| 4 | Postmortem: review operator coordination process. |

## Recovery Path C — Cutover Fails (Postgres RLS rejects)

Cause: target cell.pack ≠ tenant.pack (cross-pack attempt) OR target cell in invalid state.

| Step | Action |
|---|---|
| 1 | Cutover rolls back: tenant_assignment row reverts to source_cell. |
| 2 | Emit `MigrationFailed` event with reason. |
| 3 | Investigate: cross-pack? operator error or cedar bypass attempt? engage ops-security if latter. |
| 4 | Source-cell workload pods stay running; tenant unaffected (still in source_cell). |

## Recovery Path D — Cross-Pack Migration (Rare; SCC-required)

Per `policy/data-residency.md` §"SCC exception":

| Step | Action |
|---|---|
| 1 | Verify SCC on file; council-privacy approval; 2-person rule operators present. |
| 2 | Quorum acknowledgement via OpenBao JIT; both operators sign elevated principal. |
| 3 | Cross-pack scheduler write authorised via elevated principal carrying `quorum_acks >= 2 + scc_exception_id`. |
| 4 | Migration proceeds with audit-chain seal carrying SCC reference. |
| 5 | Tenant DPA updated to reflect new pack assignment. |

## Verification

After completion:
- `oya cell get-assignment --tenant <id>` returns target_cell.
- Tenant workload traffic flowing to target_cell (visible in observability dashboards).
- No active MigrationPlan rows for tenant.
- `MigrationCompleted` event sealed in audit-chain.
- Tenant notified per `incident-response.md` template (operational migrations: notification is informational).

## Post-incident updates

- If repeated migration churn: revisit scheduler placement policy.
- If pg_dump approach hits performance ceiling: evaluate logical-replication for larger tenants.
- Migration completion time trend tracked monthly; if p99 trends up, capacity-model revision.

## References

- Bominal ADR-0009 §"Live migration".
- `microservices/cell/PRD.md` FR-05.
- `microservices/cell/failure-modes.md` FM-06.
- `microservices/cell/policy/data-residency.md`.
- CloudNativePG logical-replication — `cloudnative-pg.io/documentation/current/logical_replication/`.
