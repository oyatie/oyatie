---
ip_id: cloud-k8s/IP-karpenter-bootstrap
authored: 2026-05-18
slice_owner: axis-cloud-k8s
related_adrs: [ADR-0064, ADR-0131, ADR-0152, ADR-0173-vendor-lock-in-avoidance-and-stack-ownership, ADR-0240-sovereign-cloud-per-regional-pack, ADR-0198, ADR-0199]
ip_status: planned
---

# IP — Karpenter bootstrap (4 NodePools per workload class)

## Why this slice

ADR-0198 establishes Karpenter 1.11 as the canonical K8s node
autoscaler, replacing Cluster Autoscaler. The bootstrap slice deploys
the controller HA + the four NodePool CRDs (app / batch / gpu /
regulatory) per ADR-0198 D-2.

## Acceptance criteria

1. Helm chart at `microservices/cloud-k8s/iac/helm/karpenter/` deploys
   to `dev`:
   - Controller HA (2 replicas).
   - Webhook live + serving.
   - ServiceMonitor scraping `:8000`.
2. Four NodePool CRDs created per ADR-0198 D-2:
   - `oya-app` (general-purpose; on-demand bias).
   - `oya-batch` (compute-optimized; spot-first; tainted).
   - `oya-gpu` (GPU on-demand only; tainted).
   - `oya-regulatory` (sovereign-region-pinned; on-demand only; tainted).
3. Cluster Autoscaler is REMOVED (per ADR-0198 D-1 strict).
4. Drift detection enabled fleet-wide.
5. Spot-to-spot consolidation enabled for app + batch NodePools.
6. Per-NodePool cost-attribution label `oya.io/workload-class` propagates
   to node labels; OpenCost reads it.

## File-level work plan

1. Helm chart (DONE this batch).
2. EC2NodeClass templates per workload class (FOLLOW-UP — backend
   binds at deploy time via per-cloud cloud-provider plugin).
3. Per-pack overlay for `regulatory` NodePool's region pin (FOLLOW-UP).
4. ArgoCD ApplicationSet entry (FOLLOW-UP).
5. CA removal verification (FOLLOW-UP).

## Risks

- Cluster Autoscaler removal is destructive; coordinate with on-call
  before the cutover.
- On-prem Cluster API integration is less mature; sovereign packs may
  fall back to manual NodePools until the Cluster API Karpenter
  provider matures.

## Out-of-scope

- The per-cloud cloud-provider plugin selection (separate IP per
  cloud).
- The on-prem Cluster API integration (separate IP).

## References

- ADR-0198 — K8s node autoscaling canonical.
- ADR-0199 — cost-attribution canonical.
