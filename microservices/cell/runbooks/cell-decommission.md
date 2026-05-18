---
doc_class: Runbook
title: Cell decommission — terminal-state cell + tenant migration out + delete
microservice: cell
severity: "Sev-3 (planned) / Sev-1 (surprise / residency breach recovery)"
status: Accepted
owner_team: axis-cell-substrate + ops-security + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/cell/failure-modes.md (FM-09 cross-pack; FM-10 soft-delete expiry)
  - microservices/cell/policy/cell-boundary.md §"Cell Identity Model"
  - microservices/cell/policy/data-residency.md §"DSR Cascade"
doc_status: published
---

# Runbook: Cell Decommission

## Trigger

ONE of:

1. **Planned** (operator-initiated): cell is end-of-life (e.g., new cell version supersedes; pack capacity rebalanced down). Invoked via `oya cell decommission --cell <id>`.
2. **DSR cascade**: tenant deprovisioning → if cell hosted only this tenant + `cell_scope` is `dedicated`/`hipaa-dedicated`, cell auto-enters decommission flow.
3. **Residency breach recovery**: cell was somehow created in wrong pack; immediate emergency decommission with regulator notification (per `incident-response.md`).
4. **Soft-delete window expiry**: 30 days have passed since decommission initiated; finalisation actions execute.

## Severity

- Planned decommission: Sev-3.
- DSR-cascade decommission: Sev-3.
- Residency breach recovery: Sev-1 (regulator-notifiable).
- Surprise / unintended decommission: Sev-1.

## Pre-checks

1. Identify cell: `oya cell describe --cell <id>` shows `(state, tenants_bound_count, cell_scope, pack)`.
2. **Critical: verify zero tenants still bound**: `oya cell list-tenants --cell <id>` MUST return empty before proceeding past Phase 2.
3. Verify 2-person rule: TWO operators must invoke (or one operator with explicit 2-person quorum elevation via OpenBao JIT).
4. Verify audit-chain healthy: needed for sealing all decommission events.
5. Verify no in-flight migrations referencing this cell.

## Decommission Procedure

### Phase 1: Declare Intent (≤ 30 min)

| Step | Action |
|---|---|
| 1 | Operator-1 issues `oya cell decommission --cell <id> --reason "<rfc>"`. |
| 2 | CLI requires 2-person rule: prompts for Operator-2 sign-off via OpenBao JIT. |
| 3 | Both operators sign elevated principal carrying `quorum_acks: 2 + decommission_reason`. |
| 4 | Cell-state machine transitions `ready → draining-tenants`. |
| 5 | Emit `CellDecommissionDeclared` event; audit-chain Ed25519 seal. |
| 6 | Tenant communications (if any tenants still bound): per `incident-response.md` template Sev-3. |

### Phase 2: Drain Tenants (≤ 6h)

| Step | Action |
|---|---|
| 7 | For each tenant bound to this cell, scheduler initiates migration to a target cell in the same pack via `tenant-migration.md` runbook. |
| 8 | Migration concurrency capped at 2 to limit user-visible impact. |
| 9 | Drain progress observable via `oya cell drain-status --cell <id>`. |
| 10 | If a tenant migration fails repeatedly: pause decommission; engage axis-cell-substrate to root-cause; abort decommission if needed (`oya cell decommission-abort --cell <id>`). |
| 11 | Once tenants_bound_count = 0: cell transitions `draining-tenants → drained`. |

### Phase 3: Drain Operator Workloads (≤ 30 min)

| Step | Action |
|---|---|
| 12 | Per-cell operator pods scaled down: `oya cell scale-operators --cell <id> --replicas 0`. |
| 13 | Cell-resident workload µservice pods scaled down. |
| 14 | K8s namespace marked for retention but operators not running. |

### Phase 4: Soft-Delete Window (30 days)

| Step | Action |
|---|---|
| 15 | Cell state: `draining → decommissioning-soft-delete`. |
| 16 | Postgres logical schema marked for retention (not yet dropped). |
| 17 | S3 prefix marked for retention (lifecycle policy lock in place). |
| 18 | OpenBao cell credentials revoked; SVIDs marked terminal. |
| 19 | Daily reminder: cell will be finalized in N days. Operator may abort: `oya cell decommission-abort --cell <id>`. |
| 20 | If aborted during soft-delete window: cell can be revived; tenants must be migrated back manually. Revival emits `CellDecommissionAborted` event. |

### Phase 5: Finalisation (FM-10; ≤ 1h after 30-day window)

| Step | Action |
|---|---|
| 21 | 2-person rule re-verified: operators reconfirm intent. |
| 22 | Postgres logical schema dropped: `DROP SCHEMA cell_<hashed-id> CASCADE`. |
| 23 | S3 prefix permanently deleted (lifecycle policy invokes delete). |
| 24 | K8s namespace deleted: `kubectl delete namespace cell-<hashed-id>`. |
| 25 | Cell-registry row state: `decommissioning-soft-delete → decommissioned` (terminal). |
| 26 | Audit-chain final seal: `CellDecommissioned`. |
| 27 | Cell-substrate metrics emit final `oya_cell_decommission_finalized_total`. |

## Recovery Paths

### Path A — Surprise decommission (unintended)

Cause: operator error; bypass of 2-person rule via stolen credential.

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-security + axis-cell-substrate + council-privacy. |
| 2 | If still in Phase 1–4 (soft-delete window): abort decommission immediately (`oya cell decommission-abort`). |
| 3 | If already finalised: data may be unrecoverable; depends on backup state. Engage workload owners for backup restore. |
| 4 | Postmortem within 2 business days; root-cause the bypass. |

### Path B — Residency breach recovery (FM-09)

Cause: cell was created in wrong pack; data committed to wrong region.

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage council-privacy. |
| 2 | Immediate tenant migration to correct-pack cell (per `tenant-migration.md` Recovery Path D cross-pack). |
| 3 | Once migration complete: this runbook's Phase 1–5 to fully delete the wrong-pack cell. |
| 4 | Regulator notification chain: GDPR Art. 33 / KR PIPA Art. 34 / HIPAA §164.404 / etc. per `incident-response.md`. |
| 5 | Postmortem within 24h. |

## Tenant Communications

| Phase | Action |
|---|---|
| Phase 1 declaration | Status page (if any tenants impacted): "Scheduled cell decommission in <pack> on <date>. Affected tenants will be auto-migrated; no action required." |
| Phase 2 in progress | Tenant operator email if migration affects them. |
| Phase 5 finalisation | Audit log entry only; no tenant comms (tenants migrated out before this point). |

## Verification

After completion:
- Cell state: `decommissioned` (terminal).
- `oya cell describe --cell <id>` returns "decommissioned" with finalisation timestamp.
- No tenant-assignments reference this cell.
- Postgres schema dropped (confirmed via `\dn` in psql).
- S3 prefix empty (confirmed via OCI Object Storage list).
- K8s namespace deleted.
- All audit-chain events sealed.

## Post-incident updates

- If repeated decommissions trigger from same root cause (e.g., capacity-band misconfig): revisit scheduler policy.
- If tenant migration during decommission was slow: revisit migration concurrency cap.
- Audit-chain immutability verified annually.

## References

- `microservices/cell/failure-modes.md` FM-09, FM-10.
- `microservices/cell/policy/cell-boundary.md`.
- `microservices/cell/policy/data-residency.md` §"DSR Cascade".
- `microservices/cell/runbooks/tenant-migration.md`.
- `microservices/cell/incident-response.md`.
- Bominal ADR-0009; ADR-0019.
