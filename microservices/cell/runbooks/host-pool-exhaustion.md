---
doc_class: Runbook
title: Host-pool exhaustion + drain stuck
microservice: cell
severity: "Sev-2 (exhaustion) / Sev-3 (drain stuck)"
status: Accepted
owner_team: ops-sre-reliability + cloud-k8s
date: 2026-05-17
related_artifacts:
  - microservices/cell/failure-modes.md (FM-04 pool exhaustion; FM-08 drain stuck)
  - microservices/cell/capacity-model.md §"Warm-Pool Sizing"
doc_status: published
---

# Runbook: Host-Pool Exhaustion + Drain Stuck

## Trigger

ONE of:

1. **Exhaustion**: `oya_cell_warm_pool_size{pack="<pack>"} < 2` for ≥ 5 min.
2. **Drain stuck**: `host_pool_drain_duration_seconds > 1800` (30 min).

## Severity

- Pool exhaustion blocking onboarding: Sev-2 (new-tenant onboarding stalls).
- Drain stuck blocking hardware retirement: Sev-3 (no tenant impact unless drain blocks recovery).

## Pre-checks

1. Identify pack: `oya cell host-pool-status --pack <pack>` shows `(available_nodes, draining_nodes, total_pool_size)`.
2. Check hyperscaler API health: OCI status page; recent quota changes; recent provisioning errors.
3. Check workload cluster autoscaler: `kubectl logs -n kube-system cluster-autoscaler` for provisioning errors.
4. Check tenant onboarding queue depth: `oya cell placement-queue --pack <pack>`.

## Recovery Path A — Pool Exhaustion (FM-04)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-2; open `#inc-<id>`; assign IC. | ≤ 5 min |
| 2 | Verify hyperscaler quota: `oci compute quota list --compartment-id <cid>` shows headroom. | ≤ 5 min |
| 3 | If quota OK: trigger immediate cluster-autoscaler scale-up. `kubectl scale ...` or via Cluster API: `kubectl patch machinedeployment <md> --type='json' -p='[{"op":"replace","path":"/spec/replicas","value":<N+2>}]'`. | ≤ 1 min |
| 4 | If quota exhausted: request OCI quota increase (engage ops-finops for budget). | varies |
| 5 | Tighten incoming placement-request rate-limit: `oya cell placement-rate-limit --pack <pack> --max-concurrent 1`. | ≤ 1 min |
| 6 | Monitor pool refill: `oya cell host-pool-status --pack <pack>` should show ≥ 2 within 5 min of scale-up. | ≤ 5 min |
| 7 | If 5 min passes without refill: escalate to cloud-k8s lead; investigate hyperscaler-side provisioning issue. | ≤ 15 min |
| 8 | Onboarding queue drains; tenant-onboarding resume. | continuous |

## Recovery Path B — Drain Stuck (FM-08)

Cause: Pod eviction blocked by PDB / finalizer / PVC detach failure.

| Step | Action |
|---|---|
| 1 | Identify stuck host: `oya cell host-status --pack <pack>` shows host in `draining` for > 30 min. |
| 2 | Inspect K8s drain log: `kubectl drain <node> --dry-run --ignore-daemonsets`. Identifies pods blocking. |
| 3 | For each blocking pod, identify cause: PDB violation? Stuck finalizer? PVC not detaching? |
| 4a | If PDB violation: engage workload owner to scale up replicas or relax PDB temporarily. |
| 4b | If finalizer stuck: investigate finalizer-owning controller; manually remove finalizer ONLY after ops-security approval (irreversible). |
| 4c | If PVC not detaching: inspect OCI Block Volume detach state; force-detach via OCI console (cloud-k8s on-call). |
| 5 | Once blocker removed: drain completes automatically. |
| 6 | Verify host transitions to `decommissioned`; cell-substrate operator pods replanned to other hosts. |
| 7 | Last resort (escalation): force-delete stuck pods (data loss possible; ops-security 2-person rule); document in postmortem. |

## Recovery Path C — Mass Drain Suspect (T-S-03 in threat-model)

Cause: alarm fires on rate > 1 drain/min (anomalous mass-drain pattern).

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-security + cloud-k8s. |
| 2 | Identify drain authority: audit-chain log shows `(host_id, drained_by_spiffe_id, timestamp)` per drain. |
| 3 | If unauthorised principal: revoke credentials immediately; freeze further drains. |
| 4 | If authorised but suspicious pattern (e.g., one operator draining many hosts): contact operator out-of-band; verify intent. |
| 5 | Block further drains pending investigation: `oya cell host-pool-block-drains --pack <pack>`. |
| 6 | Postmortem; assess audit-chain integrity. |

## Verification

After completion:
- `oya_cell_warm_pool_size{pack="<pack>"} ≥ 2`.
- Onboarding queue depth = 0.
- No stuck-draining hosts.
- Cluster autoscaler emits success metrics on next scale event.

## Post-incident updates

- If exhaustion recurring: revisit warm-pool sizing formula in `capacity-model.md` §"Warm-Pool Sizing"; consider raising baseline `W_warm_per_pack`.
- If drain-stuck recurring: revisit PDB defaults; engage workload owners.
- Trend analysis: monthly pool-utilization curve.

## References

- `microservices/cell/failure-modes.md` FM-04, FM-08.
- `microservices/cell/capacity-model.md`.
- Kubernetes drain — `kubernetes.io/docs/tasks/administer-cluster/safely-drain-node/`.
- OCI compute quota — `docs.oracle.com/en-us/iaas/Content/General/Concepts/servicelimits.htm`.
