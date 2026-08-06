---
id: ADR-0198
status: Superseded
deciders: council-architecture, axis-cloud-k8s, ops-sre-reliability, ops-finops
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0704]
related: [ADR-0064, ADR-0131, ADR-0152, ADR-0161, ADR-0173-vendor-lock-in-avoidance-and-stack-ownership, ADR-0240-sovereign-cloud-per-regional-pack, ADR-0184, ADR-0186, ADR-0199]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0198 — Kubernetes node autoscaling: Karpenter primary, NodePool per workload class

## Status

Accepted (2026-05-18). Mandates Karpenter 1.11 (kubernetes-sigs / CNCF) as
the canonical Kubernetes node autoscaler. Cluster Autoscaler is removed
from the substrate. NodePool CRDs are declared per workload class.

## Context

The hyperscaler reference shape for K8s node autoscaling has shifted from
Cluster Autoscaler (CA) to Karpenter in the 2024-2026 window:

- **AWS** — Karpenter graduated to v1.0 in 2024; AWS now recommends
  Karpenter as the default for EKS clusters; CA is the legacy fallback
  for clusters with mixed-mode constraints.
- **Pinterest, Anthropic, Stripe** — public engineering blogs document
  Karpenter adoption with multi-second scale-up and per-workload-class
  NodePools.
- **CNCF** — Karpenter is now a kubernetes-sigs project (vendor-neutral)
  with cloud-provider plugins (AWS, Azure, GCP, on-prem via Cluster API).

Cluster Autoscaler's limitations vs Karpenter:

| Concern              | Cluster Autoscaler         | Karpenter               |
|----------------------|----------------------------|-------------------------|
| Scale-up latency     | ~10 min (per ASG hot path) | ~30-60 s                |
| Bin-packing          | Per-ASG; greedy            | Cluster-wide; optimal   |
| Mixed instance types | Limited (per ASG)          | Native; first-class     |
| Spot integration     | Bolted-on                  | First-class             |
| Drift detection      | None                       | Native (compares Node-  |
|                      |                            | Class to actual node)   |
| API surface          | ConfigMap / ASG tags       | CRDs (NodePool, NodeClass) |

The shift is industry-wide and the operational gap (sub-minute scale-up)
is material for oyatie's bursty workloads (Foundry capability invocations,
Workflow Studio runs, audit-chain emit spikes).

## Decision

### D-1. Karpenter 1.11 is the canonical node autoscaler

- **License:** Apache 2.0.
- **Source:** kubernetes-sigs/karpenter (CNCF; vendor-neutral core) +
  cloud-provider plugins.
- **Cluster Autoscaler is removed** from the substrate. There is no
  fallback; if Karpenter fails, the manually-fixed nodepool (per
  NodePool CRD) survives and absorbs steady-state load.
- **Deployment:** Helm chart at
  `microservices/cloud-k8s/iac/helm/karpenter/` with HA controller (2
  replicas) + drift detection + spot-to-spot consolidation enabled.

### D-2. NodePool CRD per workload class

Four canonical NodePools, mirroring ADR-0152 workload classes:

#### app-tier (general-purpose)

- Instance categories: c, m, r; generation > 5; amd64 + arm64.
- Capacity: on-demand bias with spot allowed for cost optimization (75/25
  split as steady-state target via consolidation policy).
- Disruption: `WhenEmptyOrUnderutilized`, 30 s consolidation delay.
- Disruption budgets: 10 % nodes always; 5 nodes max per hour.

#### batch-tier (compute-optimized + spot-first)

- Instance categories: c, m; CPU > 3.
- Capacity: **spot only** (batch workloads tolerate interruption).
- Taint: `oya.io/batch-only=true:NoSchedule` (batch pods MUST tolerate).
- Disruption: `WhenEmptyOrUnderutilized`, 60 s delay.
- Disruption budgets: 20 % nodes always.

#### gpu-tier (GPU on-demand only)

- Instance families: g5, g6, p4d, p5.
- Capacity: **on-demand only** (GPU spot interruption mid-training is
  catastrophic).
- Taint: `nvidia.com/gpu=true:NoSchedule`.
- Disruption: `WhenEmpty` (never consolidate-underutilized for GPU);
  blocked during business hours via the disruption-budget schedule.

#### regulatory-tier (sovereign-region-pinned)

- Topology pinned to the regulatory region (`${REGULATORY_REGION}` via
  per-pack overlay: `kr-central`, `eu-frankfurt`, `us-gov-east`, etc.).
- Capacity: **on-demand only** (no spot for regulatory workloads).
- Taint: `oya.io/regulatory-only=true:NoSchedule`.
- Highest weight (110) — preempts other NodePool placement.

### D-3. Disruption controls

- **Budgets** — every NodePool declares a disruption budget; production
  outages from autoscaler-initiated consolidation are bounded.
- **Drift detection** enabled fleet-wide; nodes drifted from NodeClass
  spec are gracefully drained + replaced.
- **Spot-to-spot consolidation** enabled for app + batch; saves cost
  without service interruption.
- **Maintenance windows** — disruption budgets honor `cron` schedules so
  business-hour disruption is bounded.

### D-4. On-prem + multi-cloud via Cluster API

- For on-prem sovereign packs (KR CSAP, EU GAIA-X), Karpenter's
  Cluster-API cloud provider drives node provisioning against the
  on-prem hypervisor (KubeVirt / Equinix / OpenStack).
- For multi-cloud, separate Karpenter cloud-provider plugins (AWS, Azure,
  GCP, OCI) run side-by-side; each NodePool declares its
  `nodeClassRef.kind` to the relevant cloud's NodeClass.

### D-5. FinOps integration (per ADR-0199)

- Every NodePool's nodes carry the canonical cost-attribution labels
  (`oya.io/cost-center`, `oya.io/workload-class`); OpenCost aggregates
  cost by these labels.
- Karpenter metrics (node lifecycle, consolidation savings, spot vs
  on-demand share) ship to Mimir per ADR-0186.

## Alternatives considered

### (a) Cluster Autoscaler (CA) — REJECTED

- **Pros:** mature; known operator skill; no migration cost.
- **Cons:** sub-minute scale-up is impossible (per-ASG hot path is
  ~10 min); bin-packing is per-ASG and suboptimal; spot integration is
  bolted-on; no drift detection. The operational gap is material for
  oyatie's bursty workloads.
- **Rejected:** latency + bin-packing inadequate.

### (b) Cast.ai / Komodor / Akuity commercial autoscalers — REJECTED

- **Pros:** managed UI, vendor-specific cost optimization tooling.
- **Cons:** vendor lock-in (per ADR-0173); commercial licensing scales
  with cluster size; sovereign packs require on-prem-portable tooling.
- **Rejected:** vendor lock-in.

### (c) Manual NodePools (no autoscale) — REJECTED

- **Pros:** simplest; predictable cost.
- **Cons:** capacity-planning lag triggers tail-latency regressions during
  spikes; over-provisioning is required as a margin → cost burn. Not
  hyperscaler-grade.
- **Rejected:** cannot meet performance invariants.

### (d) Mix CA + Karpenter — REJECTED

- **Pros:** "best of both" intuition.
- **Cons:** real-world experience (AWS public docs, Pinterest blog) is
  that the two compete for placement decisions; conflicts cause flapping.
- **Rejected:** incompatible operational shape.

### (e) **CHOSEN:** Karpenter 1.11 primary, four NodePools per workload
class, no CA fallback.

## Consequences

### Positive

- ~30-60 s scale-up vs CA's ~10 min — material p99 latency win for bursty
  workloads.
- Bin-packing is cluster-wide → tangible cost savings (Pinterest reports
  15-25 % cost reduction vs CA).
- Drift detection catches config-divergence early.
- Per-workload-class NodePools cleanly separate concerns; batch lives on
  spot, GPU on on-demand, regulatory on sovereign region.

### Negative

- Karpenter is younger than CA; ops familiarity is lower. Mitigation:
  graduated CNCF project + AWS-recommended default since 2024; hiring
  pool is growing.
- On-prem support via Cluster API is less mature than AWS-native. Mitigation:
  per-pack overlay can still use manual NodePools (Karpenter without
  autoscale) as a safety net.

### Neutral

- One autoscaler binary; one CRD surface; ops simplicity.

## In-house roadmap

Per the user directive "wherever possible, support in-house tech stack —
like AWS, Google, Microsoft, Oracle" (2026-05-18), Karpenter occupies a
specific category that warrants KEEP (no in-house rebuild planned):

### Why Karpenter is KEEP (no Phase 2 rebuild)

- **Karpenter is a kubernetes-sigs project under CNCF governance.**
  Although it was originated by AWS, it is now a vendor-neutral
  kubernetes-sigs project with cloud-provider plugins for AWS, Azure,
  GCP, and on-prem (Cluster API). The governance model matches CNCF's
  vendor-neutral standard.
- **AWS uses Karpenter themselves** for EKS Auto Mode — the same code
  path oyatie consumes. There is no commercial-tier feature gating
  asymmetry (the failure mode ADR-0173 forbids).
- **Apache 2.0 license** with no contagion or commercial-feature
  cliff.
- **Community standard at hyperscaler scale** — Pinterest, Anthropic,
  Stripe public references confirm 15-25 % cost reduction vs CA at
  scale. The replacement bar is high.

### Phase 0 (TODAY)

- Karpenter 1.11 via Helm at `microservices/cloud-k8s/iac/helm/
  karpenter/`.
- NodePool CRDs per workload class (app / batch / gpu / regulatory).
- Per ADR-0173 best practice, even community-standard tools sit behind
  a thin adapter trait when their surface is operationally complex.
  Karpenter's surface (NodePool CRD + EC2NodeClass) is CRD-driven and
  declarative; the seam here is the **NodePool template** itself,
  managed via Helm, not a Rust trait.

### Phase 1 — multi-cloud + on-prem hardening (M02-M03 horizon)

- Per-cloud-provider plugin matrix: AWS provider, Azure provider, GCP
  provider, Cluster-API provider for on-prem sovereign packs.
- Per-pack NodeClass overlay (`kr-central`, `eu-frankfurt`, etc.).
- Drift detection + spot-to-spot consolidation enabled fleet-wide.

### Phase 2 — NOT PLANNED

- **No in-house Karpenter rebuild is planned.** The cost / benefit is
  unfavorable: rewriting a CNCF kubernetes-sigs project that AWS
  themselves consume is a poor use of engineering capital. The
  in-house posture vis-à-vis hyperscalers here is "use what AWS uses,
  because AWS uses it for the same reasons we do."
- **Boundary at which oyatie would reconsider:** if Karpenter
  governance shifts away from CNCF vendor neutrality (e.g. AWS-only
  feature divergence, or a commercial fork that fragments the
  community). Today, that boundary is far away.

### Parallel to hyperscalers' in-house posture

- **AWS** — Karpenter IS AWS's in-house tool, donated to CNCF; oyatie
  consumes it directly. No fork needed.
- **Google** — historically uses Cluster Autoscaler; recent moves
  toward GKE Autopilot use Google-proprietary autoscaling. Oyatie's
  posture matches the AWS pattern, not the Google pattern, because
  CNCF Karpenter is the open-standard direction.
- **Microsoft Azure** — AKS supports Karpenter via the Azure provider
  plugin; same as AWS.
- **Oracle OKE** — supports Karpenter via the OCI cloud provider.

### What stays "in-house" for autoscaling

- **NodeClass templates** are Oya-authored (per-workload-class,
  per-pack); never inherited from upstream defaults verbatim.
- **Disruption budgets + maintenance windows** are Oya-authored;
  business-hour blackout policy is per-µservice.
- **Cost-attribution labels on every node** are the canonical block
  per ADR-0199.

The "in-house" surface is the declarative configuration, not the
controller binary. This matches AWS's own posture for EKS.

## Industry sources

- **AWS Karpenter 1.0 announcement** (2024-08): *Announcing Karpenter
  1.0*, AWS Containers blog,
  <https://aws.amazon.com/blogs/containers/announcing-karpenter-1-0/>.
- **Karpenter at Pinterest** — engineering blog on Karpenter adoption,
  15-25 % cost reduction vs CA.
- **Karpenter docs** — <https://karpenter.sh/docs/>; *Upgrade guide* +
  *NodePool* + *Disruption* topics.
- **kubernetes-sigs/karpenter** — <https://github.com/kubernetes-sigs/karpenter/releases>.
- **CNCF projects** — Karpenter graduated under kubernetes-sigs in 2024.
- **Anthropic engineering** — public references to Karpenter for ML
  workload autoscaling.

## Verification

- Helm chart at `microservices/cloud-k8s/iac/helm/karpenter/` renders.
- NodePool CRDs render per workload class with correct taints + capacity-
  type + topology constraints.
- Karpenter metrics (`karpenter_nodes_created`, `karpenter_disruption_*`)
  scrape into Mimir.
- OpenCost aggregates cost per `oya.io/workload-class` label.

## Footnotes (versions verified 2026-05-18)

- Karpenter 1.11.x: <https://github.com/kubernetes-sigs/karpenter/releases>.
- AWS provider 1.11.1: <https://github.com/aws/karpenter-provider-aws/releases>.
- Cluster API Karpenter provider: <https://github.com/kubernetes-sigs/cluster-api-provider-karpenter> (for on-prem).
