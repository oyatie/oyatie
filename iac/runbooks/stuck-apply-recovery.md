---
doc_class: Runbook
title: Stuck-apply recovery
microservice: cloud-iac
severity: "Sev-2 (single-µservice) / Sev-1 (cluster-wide pattern)"
status: Accepted
owner_team: axis-cloud-iac + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - iac/failure-modes.md (FM-01, FM-04)
  - iac/incident-response.md
doc_status: published
---

# Runbook: Stuck-apply recovery

## Trigger

An iac-applier-worker job hangs > 15min, exceeding the p999 apply latency target. Manifests as `oya_cloud_iac_apply_duration_seconds{quantile="0.99"} > 900` or a specific `ApplyJob` in `Running` state > 15min.

## Severity

- Single-µservice stuck < 1h: Sev-2.
- Cluster-wide pattern (> 3 stuck applies in 1h) OR persistent > 1h: Sev-1.

## Pre-checks

1. Identify the stuck apply: cloud-native IaC controller/API `status --microservice` workflow.
2. Inspect applier-worker logs: `kubectl -n cloud-iac logs -l app=iac-applier-worker --tail=200`.
3. Verify Kubernetes apiserver health from applier pod: `kubectl exec <pod> -- curl -s https://<apiserver>/livez`.
4. Verify OpenTofu state-lock state: cloud-native IaC controller/API `state-lock status` workflow.

## Recovery Path A — k8s apiserver resource conflict / finalizer loop

| Step | Action |
|---|---|
| 1 | Identify offending resource: applier log shows `resource conflict on kind=X name=Y` |
| 2 | Inspect finalizer state: `kubectl get <kind> <name> -o yaml \| yq '.metadata.finalizers'` |
| 3 | If a finalizer is hung: identify owning controller; restart owning controller pod |
| 4 | If finalizer is orphaned (controller gone): manually remove via `kubectl patch <kind> <name> -p '{"metadata":{"finalizers":[]}}' --type=merge` (requires JIT + 2-person rule for production-tier) |
| 5 | Abort stuck apply via cloud-native IaC controller/API `abort --job` workflow; applier retries with backoff |
| 6 | Verify apply completes within RTO; if not, escalate |

## Recovery Path B — OpenTofu state-lock not released (FM-04)

| Step | Action |
|---|---|
| 1 | Identify stale lock: cloud-native IaC controller/API `state-lock status` workflow shows lock-holder + age |
| 2 | If holder is dead (no corresponding applier pod): force-unlock via cloud-native IaC controller/API `state-lock force-unlock` workflow. Requires JIT + 2-person rule |
| 3 | Force-unlock emits audit-chain seal; never use without justification |
| 4 | Re-trigger apply; verify lock acquired by new applier |

## Recovery Path C — Webhook hang (admission/conversion/validation webhook timing out)

| Step | Action |
|---|---|
| 1 | Identify webhook from applier log: error mentions webhook URL |
| 2 | Check webhook pod health: `kubectl -n <webhook-namespace> get pods -l <webhook-selector>` |
| 3 | Restart webhook pods if crashloop; check cert validity if TLS error |
| 4 | If webhook is owned by another µservice: engage that µservice's on-call |
| 5 | Last resort: temporarily set webhook to `failurePolicy: Ignore` (with audit-chain seal); requires ops-security approval |

## Recovery Path D — Persistent cluster-wide pattern (Sev-1)

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage axis-cloud-iac + ops-sre-reliability + ops-security |
| 2 | Open `#inc-<id>` Slack channel; assign IC |
| 3 | If cluster apiserver is the bottleneck: scale apiserver replicas; engage cloud-k8s µservice on-call |
| 4 | If ArgoCD reconciler is the bottleneck: see `runbooks/gitops-reconciler-restart.md` |
| 5 | If multiple µservices stuck on same resource: identify the contention point; consider temporarily disabling drift-reconcile for that resource via Cedar policy entitlement |

## Verification

After recovery:
- `oya_cloud_iac_apply_duration_seconds{quantile="0.99"}` returns to ≤ 5min steady state for ≥ 30min.
- Stuck-apply count `oya_cloud_iac_stuck_apply_total` rate < 1 / 1h.
- All ApplyJobs reach `Completed` or `Failed` (no `Running` > 15min).
- Self-SLO returns to green: `https://grafana-<pack>.oyatie.dev/d/cloud-iac-self/overview`.

## Post-incident updates

- Postmortem within 5 business days.
- If apiserver bottleneck: action item for cloud-k8s to increase apiserver capacity.
- If finalizer-loop pattern: file Issue against owning controller; harden controller's finalizer-cleanup path.
- If force-unlock was used: audit-chain seal review by ops-security; ensure JIT trail is complete.

## References

- `iac/failure-modes.md` FM-01 + FM-04.
- `iac/incident-response.md`.
- Kubernetes finalizer docs — `kubernetes.io/docs/concepts/overview/working-with-objects/finalizers/`.
- OpenTofu state-lock docs — `opentofu.org/docs/language/state/locking/`.
