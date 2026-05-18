---
doc_class: Runbook
title: Node cordon + drain
microservice: cloud-k8s
severity: "Sev-2 (single node) / Sev-3 (planned)"
status: Accepted
owner_team: ops-sre-reliability + axis-cloud
date: 2026-05-17
related_artifacts:
  - microservices/cloud-k8s/failure-modes.md (FM-04, FM-05)
doc_status: published
---

# Runbook: Node cordon + drain

## Trigger

- Worker node failure (FM-05): kernel panic, hardware, kubelet crashloop
- Control-plane node partition (FM-04)
- Planned: kubeadm upgrade, hardware refresh

## Severity

Sev-2 (single node failure) / Sev-3 (planned + small fleet) / Sev-1 (CP node + M01 single-CP).

## Procedure (planned, graceful)

| Step | Action | Time |
|---|---|---|
| 1 | Identify node: `kubectl get nodes -l <selector>` | ≤ 1 min |
| 2 | Verify PDB compliance: `kubectl get pdb -A` — ensure draining the node won't violate budgets | ≤ 5 min |
| 3 | Pre-cordon check: workload re-balance feasible? Sufficient surviving nodes? | ≤ 5 min |
| 4 | Cordon: `kubectl cordon <node>` | ≤ 1 min |
| 5 | Drain with eviction API: `kubectl drain <node> --ignore-daemonsets --delete-emptydir-data --grace-period=60 --timeout=600s` | ≤ 10 min |
| 6 | Verify pods rescheduled: `kubectl get pods --all-namespaces -o wide | grep <node>` empty | ≤ 5 min |
| 7 | Verify PDBs still respected: `kubectl get pdb -A` no violations | ≤ 2 min |
| 8 | Audit-chain emit `NodeDrained` event via cloud-k8s.node.drain capability | (automatic) |
| 9 | Decommission: `kubeadm reset` on node (if removing); otherwise leave cordoned for repair | ≤ 5 min |

Total: ≤ 30 min planned drain.

## Procedure (emergency, failed node)

| Step | Action | Time |
|---|---|---|
| 1 | Detect failed: NotReady ≥ 5 min OR kubelet attestation fails | ≤ 5 min (automatic) |
| 2 | Auto-cordon via node-lifecycle worker | ≤ 1 min |
| 3 | Taint-based eviction begins (kubelet's `node-monitor-grace-period`); pods auto-rescheduled per PDB | ≤ 15 min |
| 4 | Verify rescheduling: `kubectl get events --field-selector type=Normal,reason=Scheduled` shows new placements | ≤ 5 min |
| 5 | Investigate root cause: BMC console, kernel log, hardware diag | varies |
| 6 | If hardware: replace node; cloud-iac re-provisions; kubeadm join via cluster-bootstrap capability | ≤ 30 min |
| 7 | If software (kubelet OOM): adjust kubelet --kube-reserved/--system-reserved; restart node | ≤ 15 min |
| 8 | Audit-chain emit `NodeFailed` + `NodeReplaced` events | (automatic) |

Total RTO: ≤ 15 min (pods rescheduled to surviving fleet); replacement node up: ≤ 30 min.

## PDB-aware drain failure handling

If drain fails because evicting would violate PDB:
1. Identify the offending workload + its PDB
2. Coordinate with workload µservice owner: scale up replicas first, then re-drain
3. If urgent (security incident, kernel CVE): operator forces via `--disable-eviction` with ops-security 2-person rule + audit-chain emit

## Verification

- Drained node: `kubectl get pods -o wide -A | grep <node>` empty (except DaemonSets)
- Surviving fleet capacity sufficient: `kubectl top nodes` headroom ≥ 20%
- Workload SLOs unaffected: observability Mimir shows tenant SLI stable

## References

- `microservices/cloud-k8s/failure-modes.md` FM-04, FM-05.
- Kubernetes node drain — `kubernetes.io/docs/tasks/administer-cluster/safely-drain-node/`.
- PodDisruptionBudget — `kubernetes.io/docs/concepts/workloads/pods/disruptions/`.
