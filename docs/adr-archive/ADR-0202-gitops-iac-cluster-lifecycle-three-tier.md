---
id: ADR-0202
status: Superseded
superseded_by: [ADR-0709]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0202 — GitOps + IaC + Cluster lifecycle: three-tier separation

- Status: Accepted
- Date: 2026-05-18
- Deciders: Substrate architecture authority (oya-architecture-authority)
- Tags: substrate, gitops, iac, cluster-lifecycle, supply-chain
- Supersedes: none (refines tier boundaries already implicit in
  ADR-0171)
- Superseded by: none
- Related: ADR-0147 (container sandboxing runtime ladder),
  ADR-0171 (multi-cluster federation — picked ArgoCD + Cluster
  API; this ADR completes the third tier),
  ADR-0173 (vendor lock-in avoidance — OpenTofu vs Terraform BSL),
  ADR-0181 (container image promotion pipeline),
  ADR-0183 (policy engine separation — Kyverno admission policy is
  applied during GitOps reconciliation, not via IaC).

## Context

Three concerns are routinely confused into a single "infrastructure
as code" lane:

1. **Application deployment** — pushing app manifests (Helm,
   Kustomize) into running clusters.
2. **Cloud-side resource lifecycle** — provisioning the VPC, IAM,
   DNS, KMS, RDS-equivalent, namespaces, ArgoCD project boots.
3. **Kubernetes cluster lifecycle itself** — create / upgrade /
   scale / delete clusters.

A single tool that owns all three either (a) drifts in scope until
it becomes a bespoke control plane, (b) imports nominal cloud-IAM
support that breaks the moment a non-canonical primitive shows up,
or (c) re-implements GitOps reconciliation worse than ArgoCD does.

ADR-0171 already chose ArgoCD (federation) + Cluster API (cluster
lifecycle). This ADR completes the third tier by selecting
OpenTofu for cloud-side resources, and defines the tier boundary
table that prevents drift between the three.

## Decision

Three tools, three tiers, zero overlap.

### Tier A — GitOps app deployment: ArgoCD

ArgoCD is the canonical Tier-A engine (Intuit-origin, donated to
the CNCF; graduated 2022; Apache-2.0). Per ADR-0171 it is also
the federation engine.

- Owns: K8s app manifests, Helm releases, Kustomize overlays,
  ArgoCD `Application` and `ApplicationSet` CRs.
- Does NOT own: cloud-side primitives (VPC, IAM, RDS-equivalent,
  KMS keys, DNS zones). Even when an ArgoCD CR references one of
  those primitives, the primitive itself is owned by Tier B.
- Source-of-truth: per-µservice `iac/argocd/` directory inside the
  µservice's flat layout (ADR-0131).

### Tier B — Cloud-side resources: OpenTofu

OpenTofu is the Linux Foundation-hosted fork of Terraform created
after HashiCorp's 2023 BSL relicensing. MPL-2.0. Active community,
multi-provider support compatible with Terraform's provider
ecosystem.

- Owns: VPC, subnets, IAM roles + policies, RDS-equivalent
  instances, KMS keys, DNS zones, OpenBao secret bootstrap, K8s
  namespace bootstrap, ArgoCD project bootstrap (so Tier A has a
  project to push into).
- Does NOT own: per-pod manifests, Helm release manifests, or any
  intra-cluster app surface. Producing a Deployment / StatefulSet
  / DaemonSet from OpenTofu is a discipline violation.
- OpenTofu (Linux Foundation, MPL-2.0, fork of Terraform after
  Terraform's BSL relicensing in 2023) is selected over Terraform
  per ADR-0173 vendor lock-in avoidance.
- Canonical modules live under
  `microservices/cloud-iac/tofu/modules/`:
  - `cloud-account/` — root-of-trust account + organization
    setup
  - `vpc/` — VPC + subnets + routing
  - `dns/` — DNS zone + record bootstrap
  - `kms/` — KMS keys (per tenant, per region)
  - `secrets-bootstrap/` — OpenBao initial seed + bootstrap
    secrets
  - `k8s-namespace-bootstrap/` — namespace + RBAC + network
    policy seed (so ArgoCD can land app manifests safely)

### Tier C — Cluster lifecycle: Cluster API

- Owns: K8s cluster creation, upgrade, scale, delete via Cluster
  API CRDs + provider implementations (per ADR-0171).
- Does NOT own: cloud-side resources outside the cluster (those
  are Tier B), nor app manifests inside the cluster (those are
  Tier A).
- ClusterClass templates live under
  `microservices/cloud-k8s/iac/cluster-api/`.

### Boundary table

| Resource kind                | Owner Tier | Tool      |
| ---------------------------- | ---------- | --------- |
| ArgoCD Application CR        | A          | ArgoCD    |
| Helm release manifest        | A          | ArgoCD    |
| K8s Deployment / StatefulSet | A          | ArgoCD    |
| K8s namespace creation       | B          | OpenTofu  |
| K8s RBAC bootstrap           | B          | OpenTofu  |
| ArgoCD project bootstrap     | B          | OpenTofu  |
| VPC / subnet / route         | B          | OpenTofu  |
| IAM role + policy            | B          | OpenTofu  |
| KMS key                      | B          | OpenTofu  |
| DNS zone / record            | B          | OpenTofu  |
| RDS-equivalent instance      | B          | OpenTofu  |
| OpenBao initial bootstrap    | B          | OpenTofu  |
| K8s cluster (the cluster)    | C          | ClusterAPI|
| Cluster upgrade plan         | C          | ClusterAPI|

## Alternatives considered

- **Terraform (HashiCorp BSL)** — BSL relicensing (2023) prevents
  oyatie's commercial-use posture from depending on it. ADR-0173
  forbids this. Rejected.
- **Pulumi** — Multi-language IaC (TS, Python, Go, .NET) is
  attractive for product teams already in those stacks, but
  increases stack complexity (additional language runtime per
  provisioning step), and the licensing posture is
  commercial-leaning for the SaaS state-management surface.
  Rejected as canonical default; remains acceptable as a
  per-tenant exception.
- **Crossplane** — K8s-native IaC with strong story for primitives
  modeled as CRDs. Immature for non-K8s primitives that oyatie
  requires (DNS provider variance, secret-store bootstrap,
  cross-cloud root-of-trust). Rejected for canonical.
- **CDK for Terraform (CDKTF)** — Still Terraform under the hood;
  inherits BSL risk. Rejected.
- **Single tool spanning all three tiers** — Repeatedly drifts in
  every prior attempt across industry. Rejected on principle.

## Consequences

- Migration ADR addendum (T+90d): existing `.tf` files convert to
  OpenTofu (`.tofu` or `.tf` files run by `tofu`). The two
  syntaxes are intentionally compatible for the duration of the
  90-day window.
- `oya-check-iac-tier-discipline` is the advisory gate enforcing
  the boundary table. Violations:
  - OpenTofu module declares a per-pod / per-Deployment manifest.
  - ArgoCD Application references a cloud-side primitive directly
    (vs. referencing the OpenTofu-bootstrapped namespace).
- Cluster API ClusterClass templates standardize cluster shape
  across regions per ADR-0171.
- Standards doc
  `docs/standards/gitops-iac-cluster-tier-boundaries.md` is the
  reading-list anchor for new contributors.
- Existing `microservices/cloud-iac/` µservice gains the
  `tofu/modules/` canonical module set; existing IPs (IP-002
  OpenTofu IaC) remain authoritative for renderer/applier code.

## Standards anchor

- `docs/standards/gitops-iac-cluster-tier-boundaries.md`
- `crates/oya-check-iac-tier-discipline/src/lib.rs`
- `microservices/cloud-iac/tofu/modules/` (canonical modules)
- `microservices/cloud-iac/iac/helm/argocd/` (ArgoCD bootstrap
  chart)

## Migration

- T+0 (this ADR): boundary table + canonical OpenTofu module
  skeletons + standards doc.
- T+30d: All new IaC lands as OpenTofu.
- T+60d: Existing `.tf` files migrated to `tofu`.
- T+90d: CI lane flips to BLOCKER on residual Terraform usage.

## In-house roadmap

All three tiers in this ADR ARE the community standards. No
Phase-2 in-house substitute is planned; in-house effort focuses
on the integration + tier-boundary discipline that hyperscalers
themselves do not provide off-the-shelf.

### Keep as community standards (no replacement planned)

- **ArgoCD** — Tier A. Intuit-origin, donated to CNCF, graduated.
  Apache-2.0. Used in production at scale by Intuit, IBM,
  BlackRock, Tesla. This is the community standard for GitOps.
  Adopting ArgoCD *is* the in-house posture.
- **OpenTofu** — Tier B. Linux Foundation, MPL-2.0. The community
  fork that exists precisely because the previous incumbent
  (Terraform) violated open-source posture in 2023. Adopting
  OpenTofu over Terraform IS the in-house / community-aligned
  decision.
- **Cluster API** — Tier C. Kubernetes SIG Cluster Lifecycle.
  Upstream-native. No fork.

### What oyatie owns in-house (the integration layer)

The in-house investment is the *boundary table itself* + the
`oya-check-iac-tier-discipline` gate that enforces it. The boundary
table is the value oyatie adds on top of the three community
substrates. Hyperscalers (AWS Proton, GCP Config Connector, Azure
Bicep) each ship their own opinionated overlap of the three tiers;
oyatie's contribution is the discipline that keeps them separated.

### No Phase 2 in-house replacement

Building an in-house GitOps engine, IaC engine, or cluster-lifecycle
engine would (a) fragment from upstream, (b) violate ADR-0173 by
introducing Oya-exclusive lock-in, and (c) re-invent already-mature
substrates. Rejected.

### In-house contribution path

Bugs / features that arise in our integration that belong upstream
are contributed back to ArgoCD / OpenTofu / Cluster API. Per
ADR-0173 contribution-back policy.

## Open questions

- Cross-cloud root-of-trust (oyatie running across AWS + GCP +
  Azure + sovereign clouds simultaneously) is acknowledged but
  deferred to a follow-up ADR that builds on this tier table.
- Pulumi as a per-tenant exception path: opt-in mechanism +
  audit-chain emission shape pending follow-up.
