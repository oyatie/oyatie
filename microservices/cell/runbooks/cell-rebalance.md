---
doc_class: Runbook
title: Cell rebalance — re-distribute tenants when utilization band breached
microservice: cell
severity: "Sev-3 (degraded) / Sev-2 (band breach > 1h)"
status: Accepted
owner_team: axis-cell-substrate + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/cell/failure-modes.md (FM-13 Cluster API outage; FM-07 cross-cell incident)
  - microservices/cell/policy/cell-boundary.md
doc_status: published
---

# Runbook: Cell Rebalance

## Trigger

ONE of:

1. **Automated**: scheduler detects a cell exceeds [80%] utilization band OR drops below [40%] for ≥ 1h; auto-invokes rebalance evaluation.
2. **Manual**: operator declares rebalance via `oya cell rebalance --pack <pack>`.
3. **Capacity exhaustion**: pack-wide cell-count exceeds threshold; scheduler creates new cells before rebalancing.

## Severity

- Band-breach < 1h: Sev-3 (band-policy variance).
- Band-breach > 1h or affecting > 50% of cells: Sev-2.

## Pre-checks

1. Identify cells in pack: `oya cell list --pack <pack>` → list of `(cell_id, utilization%, state)`.
2. Identify hot cells (> 80%) and cold cells (< 40%).
3. Verify no active migrations in flight: `oya cell migration-status --pack <pack>` returns empty.
4. Verify host-pool warm size ≥ 2: `oya cell host-pool-status --pack <pack>`.
5. Check no active Sev-1 / Sev-2 on adjacent dependencies (Postgres / SPIRE / Cluster API).

## Recovery Path A — Standard Rebalance

| Step | Action | Time |
|---|---|---|
| 1 | Run scheduler dry-run: `oya cell rebalance --pack <pack> --dry-run`. Output: list of (tenant, source_cell, target_cell, expected_band_after) proposals. | ≤ 1 min |
| 2 | Operator reviews proposal; verifies no cross-pack moves; verifies HIPAA-dedicated cells unaffected (or specifically authorised). | ≤ 5 min |
| 3 | Execute: `oya cell rebalance --pack <pack> --apply --max-concurrent 2`. Scheduler invokes tenant-migration use case for each proposal. | per-tenant ≤ 10 min p99 |
| 4 | Monitor migration progress via `oya cell migration-status --pack <pack>`. | continuous |
| 5 | Verify post-rebalance: all cells within [40%, 80%] band; `oya cell list --pack <pack>` confirms. | ≤ 5 min |
| 6 | Emit `CellRebalanced` event per migration; audit-chain seals. | – |

## Recovery Path B — Provisioning Timeout (FM-05)

Cause: lifecycle-manager started cell create but stuck in `provisioning` > 30 min.

| Step | Action |
|---|---|
| 1 | Identify stuck cell: `oya cell list --pack <pack> --state provisioning`. |
| 2 | Inspect lifecycle-manager logs: `kubectl logs -n cell-control-plane lifecycle-manager-worker` for the stuck cell_id. |
| 3 | If Cluster API CRD stuck: re-trigger reconcile via `oya cell reconcile --cell <id>`. |
| 4 | If still stuck after 30 min: transition to `decommissioning` to clean up; new cell create via `oya cell create --pack <pack>`. |
| 5 | File issue for root-cause investigation. |

## Recovery Path C — Cross-Cell Query Incident (FM-07)

Cause: Postgres RLS detected cross-cell query attempt.

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-security + axis-cell-substrate. |
| 2 | Identify offending workload µservice via Postgres audit log: `attempted_cell_id, source_spiffe_id, query_hash`. |
| 3 | Trace lineage of the offending code path (which PR shipped the bug). |
| 4 | If active impact: revert offending PR (`oya vcs rollback --microservice <ms> --env <env>`). |
| 5 | Audit lane gap: why did `oya-cell-boundary` lane miss this? Strengthen lane heuristic. |
| 6 | Postmortem within 5 business days. |

## Recovery Path D — Management Cluster Outage (FM-13)

Cause: K8s Cluster API management cluster down.

| Step | Action |
|---|---|
| 1 | Verify outage: `kubectl --kubeconfig=mgmt cluster-info` fails. |
| 2 | Declare Sev-2; engage cloud-k8s. |
| 3 | Existing cells unaffected (workload cluster control planes independent). |
| 4 | New cell create / delete queued by lifecycle-manager; flushes on recovery. |
| 5 | Recover management cluster (etcd / API server / controllers); see cloud-k8s runbook. |
| 6 | Validate Cluster API CRDs reconcile after recovery; replay queued requests. |

## Verification

After completion:
- All cells in pack within [40%, 80%] band (or specifically-flagged dedicated cells).
- No active migrations stuck.
- No pending audit-chain seals.
- `oya cell health --pack <pack>` returns green.
- Tenants notified if migrations involved their workload (per `incident-response.md`).

## Post-incident updates

- If recurring band-breach: revisit scheduler placement policy; consider higher band ceiling.
- If migration churn high: revisit migration cost-budget; consider longer min-duration before re-evaluation.

## References

- `microservices/cell/failure-modes.md` FM-13, FM-07, FM-05.
- `microservices/cell/policy/cell-boundary.md`.
- Bominal ADR-0019 (cell sharding).
- Kubernetes Cluster API — `cluster-api.sigs.k8s.io`.
