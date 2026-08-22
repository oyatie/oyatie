---
runbook_id: cloud-k8s/karpenter-scale-up-stall
authored: 2026-05-18
oncall: axis-cloud-k8s
adr_authority: ADR-0198
---

# Runbook — Karpenter scale-up stall

## When this fires

- `KarpenterScaleUpStalled` (pending pods unscheduled for ≥ 5 min on a
  NodePool that has not provisioned a new node).
- p99 pod-scheduling latency on any NodePool > 5 min.

## First five minutes

1. **Identify** the stalled NodePool:
   ```sh
   kubectl describe nodepool app batch gpu regulatory
   ```
   Look for `Status.Conditions` indicating provisioning issues.
2. **Check** capacity-type availability for the NodePool's requirements
   (spot vs on-demand vs region pin):
   ```sh
   kubectl get nodeclaims -A | head -30
   ```
3. **Check** the cloud-provider plugin logs:
   ```sh
   kubectl -n cloud-k8s logs deployment/karpenter --tail=200 | grep -i error
   ```

## Likely root causes

| Cause                                       | Confirm via                           | Action                              |
|---------------------------------------------|---------------------------------------|-------------------------------------|
| Spot capacity exhausted (batch NodePool)    | nodeclaim status `Unavailable`        | toggle on-demand fallback per pack  |
| NodePool `limits` reached                   | `Status.Resources.Used` near limits   | raise limits via Helm upgrade       |
| Disruption budget blocking provisioning     | `Disruption.Budgets[].Schedule`       | confirm intended; wait or override  |
| Cloud-provider IAM policy missing           | controller log shows `AccessDenied`   | fix IAM; restart controller         |
| Region pin (regulatory) unavailable         | NodeClaim error `InvalidParameter`    | confirm with regulatory-pack owner  |
| GPU instance shortage (gpu NodePool)        | EC2 / GCE quota dashboard             | open quota increase request         |

## Escalation

- If scale-up stall affects `regulatory` NodePool > 30 min: SEV-1 (per
  ADR-0240-sovereign — regulatory workloads cannot scale out of
  region).
- If scale-up stall affects `app` NodePool > 1 h: SEV-2.
- If stall is a cloud-provider quota limit: open the quota request
  with the cloud-provider account team.

## Evidence

- Audit-chain class `AutoscalerIncident` sealed with the incident
  identifier and the affected NodePool.

## References

- ADR-0198 — Karpenter canonical.
- ADR-0240-sovereign-cloud-per-regional-pack.
- ADR-0152 — workload class tier mapping.
