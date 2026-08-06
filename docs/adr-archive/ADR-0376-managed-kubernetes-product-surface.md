---
id: ADR-0376
status: Superseded
planning_impact: true
deciders: founder, council-architecture, ops-platform
date: 2026-05-27
owner: council-architecture
supersedes: []
superseded_by: [ADR-0701]
related: [ADR-0375, ADR-0148, ADR-0147, ADR-0338, ADR-0131, ADR-0132, ADR-0009, ADR-0306]
related_specs: [/specs/deployment-ops-contract.json, /specs/hyperscaler-architecture-invariants.json]
milestone: M-MANAGED-K8S-PRODUCT
depends_on: [ADR-0375]
door: two-way
affected_surfaces:
  crates: []
  microservices: []
  specs: [/specs/deployment-ops-contract.json]
deliverables:
  - id: ADR-0376-D1
    description: "Two-tier managed-Kubernetes product doctrine: hosted control plane (Kamaji, control planes as pods in Oyatie's management cluster) is the DEFAULT tier; a dedicated/sovereign full Talos spoke cluster per tenant (ADR-0375) is the PREMIUM tier. Tenant picks the tier; default is hosted. This ADR records the decision; no product code lands in this lane."
    exit_criteria: "docs/decisions/ADR-0376-managed-kubernetes-product-surface.md states both tiers, names hosted-as-default + dedicated-as-premium, and the tier-selection rule; the ADR index lists ADR-0376 as Accepted."
    verified_by: "cloud-ci/Rust gate packet adr-index"
  - id: ADR-0376-D2
    description: "Adopt Kamaji (github.com/clastix/kamaji) as a SECOND, ADDITIVE clusterctl-compliant Cluster API control-plane provider alongside the existing Talos control plane from ADR-0375. The management cluster runs both control-plane providers; hosted-tier tenant control planes are Kamaji-managed pods, dedicated-tier tenant control planes stay Talos CABPT/CACPPT spokes. Provider integration is BUILT in a later lane."
    exit_criteria: "the ADR cites Kamaji as a clusterctl-compliant CAPI control-plane provider (verified against the upstream CAPI provider list) and frames it as additive to ADR-0375's Talos control-plane provider, version-agnostic (no invented version pin)."
    verified_by: "cloud-ci/Rust gate packet adr-index"
  - id: ADR-0376-D3
    description: "Dogfood-first scope: the build target is the milestone where Oyatie provisions its OWN clusters via the cluster-CRUD API as tenant-zero (oyatie-dogfood-tenancy), with NO internal bypass of the tenant model. Billing, public SLA, DPIA, and external multi-tenant GA are explicitly DEFERRED to a follow-on ADR named as future work here, not designed in this ADR."
    exit_criteria: "the ADR states the dogfood-first build target + tenant-zero/no-bypass invariant and lists billing/SLA/DPIA/external-GA as deferred future-work legs with a named follow-on ADR placeholder."
    verified_by: "cloud-ci/Rust gate packet adr-index"
  - id: ADR-0376-D4
    description: "Name the four flat, single-concern microservices the product layer decomposes into (BUILT in later lanes, NOT now): oya-managed-k8s-cluster-lifecycle, oya-managed-k8s-tenant-quota, oya-managed-k8s-control-plane-host, oya-managed-k8s-sla-observability. Flat layout per ADR-0131/0132 (src/ canonical root, single-concern, no platform/bundle)."
    exit_criteria: "the ADR names all four microservices, asserts flat single-concern layout per ADR-0131/0132, and explicitly marks them as future-lane work."
    verified_by: "cloud-ci/Rust gate packet adr-index"
  - id: ADR-0376-D5
    description: "Resolve the placeholder-debt token `adr-0375-managed-k8s-product-surface`: repoint the registry/placeholder-debt/adr-follow-ups.yaml entry and the ADR-0375 back-reference at ADR-0376, since the product surface now has its own ADR."
    exit_criteria: "grep -rn \"adr-0375-managed-k8s-product-surface\" resolves to the repointed entry only; the follow-up entry's adr_when_landed cites ADR-0376; ADR-0375's Product-framing back-reference points at ADR-0376."
    verified_by: "oya-ci-required"
purpose: >
  Establish Oyatie's managed-Kubernetes offering — its own GKE/OKE/EKS
  equivalent — as a TWO-TIER product on top of the ADR-0375 substrate: a
  hosted control plane (Kamaji, control planes as pods in the management
  cluster) as the DEFAULT tier, and a dedicated/sovereign full Talos spoke
  cluster per tenant as the PREMIUM tier. Adopt Kamaji as a second, additive
  clusterctl-compliant CAPI control-plane provider alongside Talos. Scope is
  dogfood-first (Oyatie provisions its own clusters as tenant-zero, no internal
  bypass); billing, public SLA, DPIA, and external multi-tenant GA are deferred
  to a follow-on ADR. Names the four flat single-concern microservices the
  product layer will be built from in later lanes.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0376 — Oyatie managed-Kubernetes product surface (two-tier: hosted-default + dedicated-premium)

## Status
Accepted (2026-05-27). Builds on ADR-0375 (Talos + Cluster API + Argo CD fleet
substrate); supersedes nothing. References ADR-0148 (Cilium L3/L4 + Istio Ambient
L7), ADR-0147/0338 (Kata + Cloud Hypervisor untrusted worker pools), ADR-0131/0132
(flat single-concern microservice layout), ADR-0009 (per-tenant per-region cells),
and ADR-0306 (disaster-mode survival). The customer-facing product layer this ADR
authorizes is the realization of the follow-up tracked at
`registry/placeholder-debt/adr-follow-ups.yaml#adr-0375-managed-k8s-product-surface`,
now repointed at this ADR.

## Context
ADR-0375 landed the substrate primitive — bare-metal Talos + Cluster API + per-cell
Argo CD — and explicitly framed it as the foundation of a managed-Kubernetes product:
Oyatie's own OKE/GKE/EKS. ADR-0375 deliberately left the CUSTOMER-FACING product layer
(cluster-CRUD API, per-tenant quota/RBAC/isolation, control-plane multi-tenancy
hardening, SLA/observability contract) layered on top and tracked separately. This ADR
encodes the founder decision (2026-05-27) on the SHAPE of that product layer.

The core force is **control-plane economics**. The hyperscaler managed-k8s services
split into two economic models:

- **Dedicated control plane per cluster.** Each tenant cluster gets its own etcd and
  control-plane nodes (the ADR-0375 dedicated Talos spoke: 3 control-plane nodes + its
  own etcd). Strongest isolation and the only credible sovereign / air-gapped story, but
  it carries a per-tenant standing control-plane tax (~$73/tenant/month for the
  dedicated control-plane footprint alone) that does not scale to dense multi-tenant
  economics — most tenants do not need a dedicated three-node control plane.
- **Hosted (shared-substrate) control plane.** Tenant control planes run as workloads
  inside a shared management cluster rather than on dedicated machines. This is the
  GKE/EKS/OKE economic model: dense, provisions in seconds, collapses the per-tenant
  control-plane tax, while preserving strong one-way isolation (a tenant reaches its API
  server but never the management cluster or another tenant's control plane).

ADR-0375 already gives Oyatie the dedicated path. The gap is the hosted path and the
product framing that lets a tenant CHOOSE between them. `microservices/cloud-k8s/` is
RETIRED (it carried the ADR-0120/0121 kubeadm / on-prem doctrine that ADR-0375
superseded) and is NOT the home of this product; it is named here only to record that
it is not live.

## Decision

Oyatie's managed-Kubernetes offering is a **TWO-TIER product** on top of the ADR-0375
substrate. The tenant picks the tier; **the default is hosted**.

- **Hosted control plane = DEFAULT tier.** Tenant Kubernetes control planes run as pods
  inside Oyatie's management (control-plane) cluster via **Kamaji**
  (`github.com/clastix/kamaji`), a hosted-control-plane manager listed in the CNCF
  landscape. Kamaji is adopted as a **second, additive clusterctl-compliant Cluster API
  (CAPI) control-plane provider** — the upstream CAPI provider list publishes it as the
  Kamaji control-plane provider (`github.com/clastix/cluster-api-control-plane-provider-kamaji`)
  — installed ALONGSIDE the existing Talos control-plane provider (CABPT/CACPPT) from
  ADR-0375. The management cluster runs BOTH control-plane providers; a hosted-tier
  tenant cluster is a `Cluster` whose control plane is a Kamaji `TenantControlPlane`
  (control-plane pods + a tenant `etcd` datastore in the management cluster), with worker
  nodes joined via CAPI infra providers exactly as the dedicated tier does. This is dense,
  provisions in ~seconds, collapses the ~$73/tenant/month dedicated-control-plane tax,
  and preserves strong one-way isolation. It is the GKE/EKS/OKE economic model.
- **Dedicated / sovereign = PREMIUM tier.** A full Talos spoke cluster per tenant — its
  own etcd, three control-plane nodes — the path that ALREADY exists from ADR-0375
  (Talos CABPT/CACPPT + per-cell Argo CD + INV-CELL-ISOLATION). For sovereign, air-gapped,
  or strongest-isolation tenants. No new substrate is needed for this tier; it is the
  ADR-0375 spoke promoted to a product SKU.
- Both tiers inherit the ADR-0375 invariants: Cilium L3/L4 + Istio Ambient L7 with zero
  overlap (ADR-0148), Kata + Cloud Hypervisor worker pools for tenant-untrusted workloads
  (ADR-0147/0338), per-cell Argo CD pull model and independent failure domains
  (ADR-0009 cells, ADR-0306 disaster-mode). The difference between the tiers is WHERE the
  control plane lives, not how workloads are isolated or delivered.

**Scope = DOGFOOD-FIRST.** The build target for this product line is the milestone where
Oyatie provisions its OWN clusters through the cluster-CRUD API as **tenant-zero**
(`oyatie-dogfood-tenancy`), with **NO internal bypass** of the tenant model: Oyatie's
own microservices are provisioned by the same hosted/dedicated path that will serve
external tenants, so the product is proven by self-use before it is sold. Billing,
public SLA, DPIA, and external multi-tenant GA are **explicitly DEFERRED** to a
follow-on ADR (future work: an `oya-managed-k8s-commercial-ga` decision — not designed
here).

**Implementation decomposition (named here; BUILT in later lanes, NOT now).** The
product layer decomposes into **four flat, single-concern microservices**, each shipped
under `microservices/<ms>/` with `src/` as the canonical root, single-concern, no
platform/bundle, per ADR-0131/0132:

- `oya-managed-k8s-cluster-lifecycle` — the cluster-CRUD API (create / scale / upgrade /
  delete a tenant cluster as a first-class resource), wrapping the CAPI `Cluster`
  mechanism across both tiers.
- `oya-managed-k8s-tenant-quota` — per-tenant quota + RBAC + network isolation across
  tenant clusters.
- `oya-managed-k8s-control-plane-host` — the hosted-tier control-plane host concern:
  Kamaji `TenantControlPlane` lifecycle + management-cluster multi-tenancy hardening (a
  tenant must never reach the management cluster or another tenant's control plane).
- `oya-managed-k8s-sla-observability` — control-plane uptime, provisioning latency, and
  per-cluster health surfaced to the tenant.

No crate, microservice, or controller is created in this lane; this ADR records doctrine,
adopts the Kamaji provider, names the decomposition, and resolves the placeholder-debt
token. The four microservices land in their own IPs.

## Alternatives Considered

1. **Hosted-only (Kamaji for every tenant).** Rejected: no sovereign / air-gapped story.
   A shared management-cluster control plane cannot satisfy tenants that require physical
   control-plane isolation or an air-gapped etcd, and it concentrates blast radius in the
   management cluster. Density without a dedicated escape hatch is a non-starter for the
   regulated/sovereign segment Oyatie targets (ADR-0009 packs).
2. **Dedicated-only (the ADR-0375 spoke for every tenant).** Rejected: the
   ~$73/tenant/month standing dedicated-control-plane tax does not scale to dense
   multi-tenant economics. Most tenants do not need three control-plane nodes + their own
   etcd; charging every tenant for one prices Oyatie out of the GKE/EKS/OKE commodity
   tier and wastes management capacity.
3. **Two-tier hosted-default + dedicated-premium (CHOSEN).** Kamaji hosted control planes
   as the dense default; the ADR-0375 Talos spoke as the premium sovereign SKU. Captures
   the commodity economics AND the sovereign story with ONE substrate and two
   clusterctl-compliant control-plane providers, and lets the tenant choose. The
   additional operational surface (Kamaji in the management cluster) is bounded and
   additive, not a fork of the substrate.
4. **Gardener (SAP / Linux Foundation NeoNephos) as the hosted-control-plane mechanism.**
   Evaluated at founder challenge ("why Kamaji, not Gardener?") with a sourced comparison
   (2026-05-27). Gardener is the more battle-hardened managed-k8s PRODUCT — SAP, T-Systems,
   and STACKIT run it at tens-of-thousands-of-clusters scale; its seed/shoot "kubeception"
   delivers the FULL managed-k8s lifecycle out of the box (provisioning, upgrades, node OS,
   DNS, certs, autoscaling, monitoring); and it has neutral multi-vendor governance
   (LF / NeoNephos). Those are two axes (completeness, governance) on which Gardener beats
   Kamaji. Rejected for now on the DECIDING axis — substrate fit: Gardener is a different
   top-down paradigm (Garden→Seed→Shoot, gardenlet, its own machine-controller-manager +
   extensions) that is explicitly NOT CAPI-native (`cluster-api-provider-gardener`, Aug 2025,
   is an experimental migration bridge, not convergence). Adopting it would SUPPLANT most of
   the just-landed ADR-0375 CAPI + Talos + Argo substrate rather than extend it — a substrate
   U-turn, not a tier choice — and would make ~2 of the four product microservices
   (cluster-lifecycle, sla-observability) largely redundant. Kamaji instead drops into
   ADR-0375 as one more clusterctl-compliant control-plane provider, preserves the tenant-zero
   dogfood CAPI model, and keeps the four microservices as Oyatie's product differentiation.
   Re-evaluate if the strategic priority shifts to time-to-market for a full external
   managed-k8s product with minimal in-house platform engineering — Gardener would then be the
   rational choice and worth the substrate rewrite.

## Consequences

- **Enables** a credible managed-Kubernetes product on the existing ADR-0375 substrate:
  the dense commodity tier (hosted) and the sovereign tier (dedicated) share one CAPI
  control plane and one delivery model, differing only in control-plane placement.
- **Enables dogfooding**: Oyatie provisions its own clusters as tenant-zero through the
  same path external tenants will use, proving the product by self-use (no internal
  bypass).
- **New operational surface from Kamaji**: the management cluster now hosts tenant
  control-plane pods + per-tenant `etcd` datastores and must be run HA and hardened as a
  multi-tenant control-plane host (a tenant must never reach the management cluster or a
  peer tenant). Kamaji upgrade/patch cadence, per-tenant etcd backup/restore, and
  management-cluster capacity planning become standing operational concerns. This
  compounds the ADR-0375 known gap (the CAPI management control plane is single-site until
  run HA) — hosted-tier density makes management-cluster HA a hard prerequisite, not a
  follow-up nicety.
- **Deferred GA legs** (own follow-on ADR, NOT this one): billing/metering of managed
  clusters, the public SLA contract, the DPIA for hosting tenant control planes, and
  external multi-tenant GA. Until that ADR lands, the product is dogfood-only.
- **Placeholder debt resolved**: the
  `registry/placeholder-debt/adr-follow-ups.yaml#adr-0375-managed-k8s-product-surface`
  follow-up and the ADR-0375 Product-framing back-reference are repointed at ADR-0376; the
  product surface now has its own ADR.
- **Versioning**: Kamaji is adopted version-agnostically here. A concrete Kamaji /
  CAPI-provider version pin lands in `registry/lts-pins.yaml` when the
  `oya-managed-k8s-control-plane-host` lane integrates the provider, not in this ADR.
- **Tracked risk — Kamaji maturity + vendor concentration**: Kamaji is CNCF *Sandbox*
  with a single primary backer (Clastix). This is an accepted risk given the two-way door;
  re-evaluate at CNCF Incubation or if Clastix backing weakens. The drop-in fallback is
  **k0smotron** (Mirantis) — also a clusterctl-compliant CAPI control-plane provider
  (`K0smotronControlPlane`, CNCF Sandbox) — so switching the hosted-tier provider is
  low-cost and does not touch the substrate. Production adopters (NVIDIA DOCA/DPF, Rackspace,
  OVHcloud, IONOS) evidence Kamaji's hyperscaler-grade fitness today, which is why it passes
  the "would a hyperscaler use this dependency?" bar.

## Door
Two-way: Kamaji is an additive, replaceable clusterctl-compliant control-plane provider
(k0smotron is the drop-in fallback provider); the dedicated Talos tier remains fully
functional without it, and the two-tier model can collapse back to dedicated-only by
retiring the hosted provider without touching the substrate.

## References
- ADR-0375 — Talos + Cluster API + Argo CD fleet substrate (the substrate this product builds on).
- ADR-0148 — service mesh: Cilium L3/L4 + Istio Ambient L7 (zero overlap).
- ADR-0147 / ADR-0338 — Kata Containers + Cloud Hypervisor untrusted worker pools.
- ADR-0131 / ADR-0132 — flat single-concern microservice layout (src/ canonical root; no platform/bundle).
- ADR-0009 — per-tenant per-region cells; ADR-0306 — disaster-mode survival.
- Kamaji — `github.com/clastix/kamaji`; CAPI control-plane provider
  `github.com/clastix/cluster-api-control-plane-provider-kamaji` (listed in the upstream
  Cluster API provider reference at `cluster-api.sigs.k8s.io/reference/providers`).
- `registry/placeholder-debt/adr-follow-ups.yaml#adr-0375-managed-k8s-product-surface` —
  the follow-up this ADR realizes (repointed at ADR-0376).
- Evaluated alternative — Gardener (`gardener.cloud`, LF / NeoNephos); drop-in fallback
  provider — k0smotron (`docs.k0smotron.io`, Mirantis, CNCF Sandbox). See the 2026-05-27
  sourced Kamaji-vs-Gardener comparison that backs the Alternatives-Considered entry above.
