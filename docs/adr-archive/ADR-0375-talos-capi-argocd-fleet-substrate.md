---
id: ADR-0375
status: Superseded
planning_impact: true
deciders: founder, council-architecture, ops-platform
date: 2026-05-27
owner: council-architecture
supersedes: [ADR-0120, ADR-0121]
superseded_by: [ADR-701]
related: [ADR-0009, ADR-0147, ADR-0148, ADR-0306, ADR-0338, ADR-0339, ADR-0363, ADR-0370, ADR-0371, ADR-0158, ADR-0171, ADR-0198]
related_specs: [/specs/deployment-ops-contract.json, /specs/hyperscaler-architecture-invariants.json, /specs/talos-001-substrate-slice.json]
milestone: M-FLEET-SUBSTRATE
depends_on: [ADR-0370]
door: two-way
affected_surfaces:
  crates: [oya-dev-cli]
  microservices: []
  specs: [/specs/deployment-ops-contract.json]
deliverables:
  - id: ADR-0375-D1
    description: "Bare-metal Talos installation-media zero-touch installer generator (infra/talos/installation-media): a `control-plane` preset with the machine config baked offline via the imager `--embedded-config-path`, and a `node` preset (Kata extension + `talos.config` fetch) built via the Image Factory. Substrate (on-prem/colo) does not split the media; cloud nodes use no installation media (CAPI cloud images)."
    exit_criteria: "gen-media.sh control-plane and node each produce a bootable ISO9660 image; the control plane image embeds the controlplane config; talosctl gen config applies the cni:none + proxy.disabled + schedulable-CP patches."
    verified_by: "infra/talos/installation-media/gen-media.sh control-plane (ISO is bootable; config baked)"
  - id: ADR-0375-D2
    description: "Cluster API control plane (infra/capi): clusterctl provider pins (Talos CABPT/CACPPT + OCI/AWS/Metal3), init.sh (clusterctl init onto the installation-media-formed control plane, no kind), and a ClusterResourceSet (crs/) that bootstraps Cilium + Argo CD onto each cluster at provision time."
    exit_criteria: "clusterctl init installs the Talos + OCI/AWS/Metal3 providers on the control plane; the CRS applies Cilium + Argo CD to clusters labelled oya.io/bootstrap=true."
    verified_by: "KUBECONFIG=<control-plane> infra/capi/init.sh (providers Ready) + clusterctl describe"
  - id: ADR-0375-D3
    description: "Parameterized spoke-cell Helm chart (infra/capi/clusters): a cell is a values entry (substrate oci|aws|metal3), not a copied file. Renders the Talos Cluster CR set per cell — cni:none + Cilium-via-CRS, dedicated CP + Kata worker pools (ADR-0147/0338). Each cell is an independent failure domain (INV-CELL-ISOLATION)."
    exit_criteria: "helm lint + helm template (values-example.yaml) render valid CR sets for all three substrates; a rendered cell validates with kubectl --dry-run=server post-init and provisions a 3-CP HA Talos cluster; worker pool carries katacontainers.io/kata-runtime."
    verified_by: "helm lint infra/capi/clusters && helm template oya-spokes infra/capi/clusters -f infra/capi/clusters/values-example.yaml"
  - id: ADR-0375-D4
    description: "Per-cell Argo CD app-of-apps Helm chart (infra/gitops), pull model: delivers GitHub, Jenkins, OpenBao, observability, Kyverno, Istio Ambient. Cilium L3/L4 + Istio Ambient L7 zero-overlap (ADR-0148). Source = GitHub at bootstrap, flips to GitHub post-cutover (ADR-0247)."
    exit_criteria: "helm lint + helm template render valid Argo CD Applications; per-cell `cell` value override works."
    verified_by: "helm lint infra/gitops && helm template oya-platform infra/gitops"
purpose: >
  Adopt the canonical hyperscaler-pattern OSS substrate — bare-metal Talos
  (USB zero-touch) + Cluster API for cluster lifecycle + per-cell Argo CD for
  app delivery — for the ~8 cross-region cell fleet, and RETIRE the prior
  OCI/on-prem deployment model: the Sidero Omni evaluation, the kubeadm/
  containerd/istio-envoy on-prem stack (ADR-0121), the rust-first on-prem shell
  tooling (ADR-0120), and the infra/oci OpenTofu + infra/onprem shell trees.
  OpenTofu now owns only the Cloudflare edge; the cluster fleet is declarative
  CAPI/Talos/Argo CD.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0375 — Talos + Cluster API + Argo CD fleet substrate (retire Omni / OCI-TF / on-prem)

## Status
Accepted (2026-05-27). Supersedes ADR-0120 (rust-first on-prem tooling) and ADR-0121
(on-prem k8s stack: kubeadm + containerd + istio-envoy). Amends the deployment-ops-contract
(`specs/deployment-ops-contract.json` v2.0.0) accordingly. The TALOS-001 substrate-slice
validation matrix (authorities, claim boundaries, and required-surface source paths spanning
this ADR plus ADR-0370/0371/0376/0378/0382) is specified in the dedicated
`specs/talos-001-substrate-slice.json`, enforced by the
`//ci/facade/contract-slice-conformance` gate.

## Product framing — Oyatie's own OKE/GKE/EKS

This substrate is not only how Oyatie runs its OWN services — it is the foundation
of a **managed-Kubernetes product**: Oyatie's equivalent of OKE / GKE / EKS. The
architecture is deliberately the same shape the hyperscalers use for managed k8s:

- A **management (control-plane) cluster** runs Cluster API + the Talos
  bootstrap/control-plane providers + infra providers (CAPOCI/CAPA/Metal3). This is
  the managed-control-plane plane — the thing a customer never SSHes into.
- **Tenant clusters are CAPI spokes**: each `cells[]` entry in
  `infra/capi/clusters` is a fully-isolated Talos cluster (its own CP + Kata worker
  pools, `cni:none` + Cilium, per-cell Argo CD). Provisioning a customer cluster is
  adding a cell — declarative, git-driven, controller-reconciled — exactly the
  OKE/GKE/EKS "create cluster" primitive, vendor-neutral across OCI/AWS/bare-metal.
- **Oyatie's own platform is tenant zero** (`oyatie-dogfood-tenancy`): the same
  provisioning path that will serve external tenants runs Oyatie's microservices
  first, so the product is proven by self-use before it is sold. No internal bypass.
- **Per-cell Argo CD (pull model) + INV-CELL-ISOLATION** give each tenant cluster
  an independent failure domain and disaster-mode survival (ADR-0306) — the
  blast-radius isolation a managed-k8s SLA requires.

What this ADR lands is the substrate primitive (provision/lifecycle/delivery). The
customer-facing managed-k8s surface on top of it — the cluster-CRUD API, per-tenant
quota/billing/RBAC, control-plane multi-tenancy hardening, and the SLA/observability
contract — is the **product layer**, decided in **ADR-0376** (two-tier hosted-default +
dedicated-premium, Kamaji as an additive CAPI control-plane provider, dogfood-first). The
`registry/placeholder-debt/adr-follow-ups.yaml#adr-0375-managed-k8s-product-surface`
follow-up is repointed at ADR-0376.

## Context
The local substrate needed a production-fidelity, hyperscaler-shaped foundation for ~8
cross-region cells (ADR-0009, per-region packs) AND to BE the managed-Kubernetes
product surface (see Product framing above). Three approaches were evaluated this cycle:

1. **Sidero Omni** — a proprietary fleet manager. Rejected: not how hyperscalers provision,
   proprietary/commercial, and operationally fiddly (SideroLink networking, join-token flow,
   API behind Cloudflare Access, omnictl-only).
2. **libvirt VMs on a Debian host (kubeadm/k3s-style)** — the prior on-prem model (ADR-0120/0121).
   Rejected: hand-provisioning tax (qemu perms, Docker↔libvirt iptables, mutable host), and a
   single Debian host can't be a multi-node fleet without nested-virt complexity.
3. **Talos + Cluster API + Argo CD** — the CNCF-standard declarative cluster lifecycle + GitOps.
   Adopted.

## Decision
- **Node OS:** Talos (immutable, API-managed). Bare-metal nodes auto-install zero-touch from a
  **USB** image (config baked for the control plane; fetched for spoke nodes). Cloud nodes use CAPI cloud images.
- **Cluster lifecycle:** **Cluster API** — declarative `Cluster`/`MachineDeployment` CRs in git,
  reconciled by controllers. The management/control-plane cluster runs CAPI core + Talos providers
  (CABPT/CACPPT) + infra providers (CAPOCI / CAPA / Metal3). Provisioned out-of-band (no maintained
  libvirt CAPI provider); `clusterctl init` runs directly onto the installation-media-formed Talos control plane (no kind).
- **Day-1 bootstrap:** a **CAPI ClusterResourceSet** installs Cilium (CNI) + Argo CD onto every
  cluster at provision time.
- **App delivery:** **per-cell Argo CD (pull model)**, Helm-first app-of-apps. Each cell
  self-reconciles from git, independent of the control plane (INV-CELL-ISOLATION; ADR-0306 disaster-mode).
- **Mesh:** Cilium L3/L4 + Istio Ambient L7, zero overlap (ADR-0148, Cilium 1.19.x).
- **Runtime tiers:** Kata + Cloud Hypervisor worker pools for tenant-untrusted workloads (ADR-0147/0338).
- **Deployment authority:** OpenTofu owns only the Cloudflare edge (`infra/cloudflare`); the cluster
  fleet is CAPI/Talos/Argo CD. `specs/deployment-ops-contract.json` updated to v2.0.0.

## Consequences
- **Retired:** `infra/omni`, `infra/oci` (OpenTofu OCI stage0 trees), `infra/onprem` (kubeadm/
  containerd/istio/k3s/openbao/foundry shell stack), the OCI/libvirt helper scripts, and the
  Makefile's OCI-tenancy targets. ADR-0120/0121 superseded. The `oya onprem` / `oya ops oci-*`
  Rust subcommands remain dormant (no longer contract-enforced) pending removal in a follow-up.
- **App-layer concerns stay layered on top** (own ADRs): cell-routing/shuffle-sharding
  (ADR-0009/0248/0351), global LB/anycast/multi-region (ADR-0158/0171/0253; anycast = Cloudflare),
  capacity placement/autoscaling (ADR-0198 Karpenter, ADR-0340).
- **Known gap (not a pattern flaw):** the CAPI management control plane is single-site (SPOF) until run HA
  across hosts/regions — tracked as control-plane-HA hardening.

## Door
Two-way: CAPI/Talos/Argo CD are replaceable OSS layers; clusters are declarative and re-provisionable.

## Historical residual from ADR-120 (E3 fold 2026-08-06)

**Title:** ADR-0120-rust-first-onprem-tooling-with-paired-uninstall

**Preserved decision gist:** Two coupled rules: ### Rule 1 — Limited shell surface; Rust elsewhere. The on-prem tooling collapses to **one binary** plus a small bootstrap layer: ``` crates/oya-onprem-cli ← Rust binary `oya-onprem` ├── install <component> ← installs one component (idempotent) ├── uninstall <component> ← reverses one install (idempotent) ├── status ← machine-readable diagnostics ├── scan ← security scan (delegates to gitleaks/trivy/...) ├── cleanup ← apt autoremove + agent-state reap └── doctor ← runs status + suggests fixes ``` Authorized shell scripts (the **bootstrap layer**, capped at 3 files total): 1.

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-121 (E3 fold 2026-08-06)

**Title:** ADR-0121-onprem-k8s-stack-kubeadm-containerd-istio-envoy

**Preserved decision gist:** The on-prem Kubernetes stack on the KR primary cell is: | Layer | Component | Why | |---|---|---| | Container runtime | **containerd** (CRI) | Same runtime OCI OKE uses; canonical upstream choice. Aligns with ADR-0117. | | Kubernetes distribution | **kubeadm** (vanilla upstream) | Maximum OKE-parity; no Rancher-specific bits to unwind at M03 promotion; CNCF conformance by construction. | | Service mesh control plane | **Istio** (minimal profile initially) | Per ADR-0044 service-mesh + mTLS decision; canonical Envoy operator with strong ecosystem. | | Service mesh data plane | **Envoy** (Istio 

_Source file archived after fold; full body in git history / docs/adr-archive/._
