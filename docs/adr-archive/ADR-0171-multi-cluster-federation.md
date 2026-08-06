---
id: ADR-0171
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-cloud-iac, axis-cloud-k8s, axis-regional-packs
date: 2026-05-18
owner: axis-cloud-iac
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0009, ADR-0010, ADR-0049, ADR-0117, ADR-0121, ADR-0131, ADR-0148]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/per-microservice-flat-layout.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0171 — Multi-cluster federation via ArgoCD ApplicationSets + Cluster API

## Status

Accepted (2026-05-18). Authorizes ArgoCD ApplicationSets as the canonical multi-cluster federation surface and Cluster API (CAPI) as the canonical cluster-lifecycle controller, with a federation control plane managing per-pack regional clusters and cross-region routing. Tier C "nice-to-have" hyperscaler pattern per `/specs/hyperscaler-architecture-invariants.json` audit Row C5.

## Context

Oyatie operates per-pack regional clusters per ADR-0010 (regional pack architecture) and per ADR-0009 (cell architecture per-tenant per-region). The fleet shape at M01 (foundation) milestone:

- ≥3 packs at GA: global, EU, KR. Each pack ships its own cluster(s).
- Each pack has at least one PROD cluster + DR cluster.
- Foundry's per-tenant GPU pool runs on cell-isolated clusters per ADR-0009.

By M02 we project ≥12 clusters under management. Managing 12+ clusters without a federation surface produces:

1. **Drift between clusters** — `kustomize` overlays applied per-cluster diverge silently.
2. **Manual-orchestration toil** — every release requires N kubectl-context switches.
3. **Cross-region routing complexity** — failover between primary and DR clusters requires manual DNS / mesh-config changes.
4. **Cluster lifecycle by hand** — provisioning a new pack cluster takes ~2 days of manual Terraform-runbook.

The hyperscaler-reference is well-established:

- **GKE Multi-Cluster Ingress / Anthos** — Google's federation surface; per-cluster Application + multi-cluster Ingress.
- **AWS EKS + ArgoCD ApplicationSets** — Spinnaker / ArgoCD as the federation control plane.
- **Azure Arc + ArgoCD** — Microsoft's federated-cluster surface.
- **Cluster API (CAPI)** — CNCF graduated; Kubernetes-native cluster lifecycle.
- **ArgoCD ApplicationSets** — Argo project; declarative N-cluster application deployment.
- **Karmada (CNCF)** — multi-cluster scheduling; alternative to ApplicationSets.

ADR-0148 already commits to Istio as the canonical service-mesh; Istio's `multi-primary` topology covers in-mesh cross-cluster discovery. What's missing is:

- **Application deployment across N clusters** — ArgoCD ApplicationSets.
- **Cluster lifecycle (create / upgrade / delete)** — Cluster API.
- **Cross-region routing for tenant traffic** — DNS-based + GeoIP routing via the `cloud-network` µservice; multi-cluster Ingress pattern from GKE adapted to on-prem.

## Decision

Oyatie adopts a three-component multi-cluster federation substrate:

### Component 1: ArgoCD ApplicationSets (application deployment across N clusters)

Every µservice's `iac/helm/<ms>/` chart is referenced from a single ArgoCD `ApplicationSet` declaration. The ApplicationSet uses a cluster-list generator (`generators.list[]`) or a cluster-decision-resource generator to fan out to each target cluster with per-cluster value overrides.

Per-pack example:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: meet
spec:
  generators:
  - clusters:
      selector:
        matchLabels:
          oya.dev/pack-tier: production
  template:
    metadata:
      name: 'meet-{{name}}'
    spec:
      project: default
      source:
        repoURL: https://github.com/oyatie/oya.git
        targetRevision: HEAD
        path: microservices/meet/iac/helm/meet
        helm:
          valueFiles:
          - 'values-{{metadata.labels.oya.dev/pack}}.yaml'
      destination:
        server: '{{server}}'
        namespace: meet
```

Per-pack overlays (e.g. `values-kr.yaml`, `values-eu.yaml`, `values-global.yaml`) live under `microservices/<ms>/iac/helm/<ms>/`.

### Component 2: Cluster API (CAPI) (cluster lifecycle)

Cluster creation, upgrade, and deletion are declarative via Cluster API. Each cluster is described by a CAPI `Cluster` + provider-specific (`AzureCluster` / `AWSCluster` / `OpenStackCluster` / `MetalCluster`) CRD. The management cluster runs the CAPI controllers and provisions workload clusters.

CAPI providers per-environment:

- **On-prem (sovereign packs)** — `cluster-api-provider-metal3` (bare-metal) per ADR-0117.
- **EU pack** — `cluster-api-provider-azure` or `cluster-api-provider-openstack` per ADR-0049 cross-region replication + residency.
- **KR pack** — sovereign on-prem; metal3.
- **Foundry GPU pools** — cell-isolated CAPI clusters per ADR-0009.

CAPI versions tracked per ADR-0121 (canonical hyperscaler-bar standards index).

### Component 3: Federation control plane (cross-region routing)

A dedicated "federation" cell hosts:

- The ArgoCD federation control plane (1 ArgoCD instance manages N application clusters).
- The CAPI management cluster.
- The Karmada-or-multi-cluster-Ingress controller for cross-region routing.
- The `cloud-network` µservice's GeoDNS + global load balancer (per pack).

Cross-region routing pattern adapted from GKE Multi-Cluster Ingress:

1. Tenant DNS resolves to a GeoDNS pool.
2. Each pack publishes its public IP to the GeoDNS pool with a residency-aware label.
3. Per-tenant residency (per ADR-0010) constrains which pack receives the request.
4. Failover within a pack: pack-local DR cluster receives traffic via DNS-failover (TTL ≤60s).
5. Failover across packs: ONLY for non-residency-bound tenants; otherwise per ADR-0008 data-use boundary the request is failed-closed.

### Per-pack cluster topology

Each pack ships:

- Primary application cluster (PROD workload).
- DR application cluster (warm-standby; same data-residency).
- Optional cell clusters for tenant-isolated workloads per ADR-0009.

The federation control plane lives in a meta-pack ("federation") that is NOT a tenant data residency boundary — it carries only ApplicationSets, CAPI controllers, and routing config; no tenant data.

## Alternatives considered

### A. Single global cluster (no federation)
- Pros: simplest possible topology; one kubectl context.
- Cons: violates ADR-0009 cell architecture; violates ADR-0010 regional pack architecture; violates ADR-0049 cross-region residency; cluster blast radius is fleet-wide; no DR isolation.
- **Rejected**: violates four prior ADRs; insufficient for the hyperscaler shape.

### B. Per-cluster kubectl-orchestrated deployment (no ArgoCD federation)
- Pros: no federation tier to manage.
- Cons: manual N-cluster orchestration toil; drift between clusters; no declarative source-of-truth for "what's deployed where".
- **Rejected**: ops toil dominates; GitOps source-of-truth lost.

### C. Karmada as the federation control plane (not ArgoCD ApplicationSets)
- Pros: CNCF Sandbox-tier; native multi-cluster scheduling primitive; per-cluster placement policies.
- Cons: smaller community than ArgoCD ApplicationSets; less mature plugin ecosystem; team has prior ArgoCD competency from ADR-0040 + ADR-0121.
- **Rejected**: ArgoCD ecosystem maturity + team competency win.

### D. ArgoCD ApplicationSets WITHOUT Cluster API (manual cluster lifecycle)
- Pros: only one new substrate.
- Cons: cluster provisioning still ~2-day manual runbook; no declarative cluster lifecycle; cluster drift visible to operators only via inspection.
- **Rejected**: half-solution; cluster lifecycle is the dominant remaining toil.

### E. GKE Multi-Cluster Ingress / Anthos directly (Google-managed)
- Pros: turnkey; well-documented federation surface.
- Cons: GCP-bound (violates ADR-0117 sovereign on-prem invariant); residency violation for KR pack; per-cluster GCP cost dominates at scale.
- **Rejected**: provider lock-in + residency violation.

### F. Per-cell ArgoCD instances (no federation control plane)
- Pros: per-cell isolation; no cross-cell coupling.
- Cons: cross-pack release coordination becomes manual; per-cell ArgoCD instances drift on plugin versions; engineering cost dominates.
- **Partial accept**: per-CELL ArgoCD for tenant-isolated cells per ADR-0009; per-PACK ArgoCD for pack-scope deployments. Federation control plane governs pack-level + cross-pack rollouts; cells get their own per-tenant ArgoCD.

## Consequences

### Positive

1. **Hyperscaler-parity** — Oyatie's federation surface matches GKE Multi-Cluster Ingress + AWS EKS + ArgoCD industry-default. Audit Row C5 closed.
2. **Declarative cluster lifecycle** — CAPI provides "kubectl apply → cluster exists in 30min" — vs the 2-day manual runbook.
3. **Single ApplicationSet per µservice** — one declaration deploys to all packs; per-pack overlay handles localization.
4. **Cross-region routing canonical** — GeoDNS + multi-cluster Ingress pattern documented; per-tenant residency constraints enforced.
5. **Cell-isolation preserved** — federation control plane is meta-pack; tenant data never traverses the federation tier.

### Negative

1. **Federation control plane is a single-point-of-coordination** — outage of the federation tier blocks NEW deployments fleet-wide (existing deployments continue running). DR for the federation tier: per-region warm-standby federation cluster.
2. **CAPI version skew** — each provider (metal3 / azure / aws / openstack) versions independently; quarterly upgrade cadence audited.
3. **ApplicationSet generator complexity** — multi-generator chains (cluster + matrix + git-file) require careful authoring; per-µservice CI lane validates generator output via dry-run.
4. **Cross-region DNS TTL tradeoffs** — TTL ≤60s for failover-friendliness vs DNS-query load. Per-pack GeoDNS scaled accordingly.

### Operational

1. `microservices/cloud-iac/PRD.md` updated to declare ArgoCD ApplicationSets as the canonical multi-cluster federation surface (per this ADR).
2. CAPI providers tracked in `registry/cluster-api-providers.json` (new registry entry).
3. Federation control plane runs in a dedicated meta-pack ("federation"); not a tenant-data residency boundary.
4. Per-µservice migration: each µservice's helm chart references switch from per-cluster `kubectl apply` to ApplicationSet-rendered `kubectl apply` over a single migration window (one PR per µservice).
5. GeoDNS + multi-cluster Ingress: managed by the `cloud-network` µservice; per-pack public IPs published with residency labels.
6. SLO: federation control plane availability 99.95% (one nine below the platform — federation outage degrades new-deploy velocity, not tenant-facing traffic).

## References

- ArgoCD ApplicationSets — https://argo-cd.readthedocs.io/en/stable/operator-manual/applicationset/ — canonical multi-cluster application deployment.
- ArgoCD ApplicationSets Generators — https://argo-cd.readthedocs.io/en/stable/operator-manual/applicationset/Generators/ — cluster / matrix / git-file generators we use.
- Cluster API (CAPI) — https://cluster-api.sigs.k8s.io — CNCF graduated; Kubernetes-native cluster lifecycle.
- CAPI Provider — Metal3 (bare-metal) — https://github.com/metal3-io/cluster-api-provider-metal3 — on-prem pack provider.
- GKE Multi-Cluster Ingress — https://cloud.google.com/kubernetes-engine/docs/concepts/multi-cluster-ingress — cross-region routing pattern we adapt to on-prem.
- AWS EKS + ArgoCD federation — https://aws.amazon.com/blogs/containers/multi-cluster-gitops-using-amazon-eks-with-argocd/ — reference architecture.
- Anthos Multi-Cluster — https://cloud.google.com/anthos/clusters — Google's federated-cluster surface.
- Karmada — https://karmada.io — CNCF Sandbox alternative considered + rejected.
- Istio multi-primary topology — https://istio.io/latest/docs/setup/install/multicluster/multi-primary/ — in-mesh cross-cluster discovery (ADR-0148).
- ADR-0009 — cell architecture per-tenant per-region (cell-isolated clusters preserved by federation tier).
- ADR-0010 — regional pack architecture (per-pack cluster topology this ADR operationalizes).
- ADR-0049 — cross-region replication + residency (residency constraints honored by GeoDNS pool labels).
- ADR-0117 — cloud-native infrastructure (sovereign on-prem authority).
- ADR-0121 — canonical hyperscaler-bar standards index (ArgoCD + CAPI cited here).
- ADR-0131 — per-microservice flat layout (helm charts under per-µservice `iac/helm/<ms>/`).
- ADR-0148 — service-mesh Istio (Istio multi-primary complements ApplicationSet-driven app deploy).
- `/specs/hyperscaler-architecture-invariants.json` — audit Row C5 closes here.
