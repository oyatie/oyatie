---
id: ADR-0375
status: Accepted
planning_impact: true
deciders: founder, council-architecture, ops-platform
date: 2026-05-27
owner: council-architecture
supersedes: [ADR-0120, ADR-0121]
superseded_by: []
related: [ADR-0009, ADR-0147, ADR-0148, ADR-0306, ADR-0338, ADR-0339, ADR-0363, ADR-0370, ADR-0371, ADR-0158, ADR-0171, ADR-0198]
related_specs: [/specs/deployment-ops-contract.json, /specs/hyperscaler-architecture-invariants.json]
milestone: M-FLEET-SUBSTRATE
depends_on: [ADR-0370]
door: two-way
affected_surfaces:
  crates: [oya-dev-cli]
  microservices: []
  specs: [/specs/deployment-ops-contract.json]
deliverables:
  - id: ADR-0375-D1
    description: "Bare-metal Talos USB zero-touch installer generator (infra/talos/usb): a `hub` preset with the machine config baked offline via the imager `--embedded-config-path`, and a `node` preset (Kata extension + `talos.config` fetch) built via the Image Factory. Substrate (on-prem/colo) does not split the media; cloud nodes use no USB (CAPI cloud images)."
    exit_criteria: "gen-usb.sh hub and node each produce a bootable ISO9660 image; the hub image embeds the controlplane config; talosctl gen config applies the cni:none + proxy.disabled + schedulable-CP patches."
    verified_by: "infra/talos/usb/gen-usb.sh hub (ISO is bootable; config baked)"
  - id: ADR-0375-D2
    description: "Cluster API control plane (infra/capi): clusterctl provider pins (Talos CABPT/CACPPT + OCI/AWS/Metal3), init.sh (clusterctl init onto the USB-formed hub, no kind), and a ClusterResourceSet (crs/) that bootstraps Cilium + Argo CD onto each cluster at provision time."
    exit_criteria: "clusterctl init installs the Talos + OCI/AWS/Metal3 providers on the hub; the CRS applies Cilium + Argo CD to clusters labelled oya.io/bootstrap=true."
    verified_by: "KUBECONFIG=<hub> infra/capi/init.sh (providers Ready) + clusterctl describe"
  - id: ADR-0375-D3
    description: "Per-substrate spoke Cluster CR templates (infra/capi/clusters/{oci,aws,metal3}): one Talos cell each, cni:none + Cilium-via-CRS, dedicated CP + Kata worker pools (ADR-0147/0338). Each cell is an independent failure domain (INV-CELL-ISOLATION)."
    exit_criteria: "a filled spoke template validates with kubectl --dry-run=server post-init and provisions a 3-CP HA Talos cluster; worker pool carries katacontainers.io/kata-runtime."
    verified_by: "kubectl --dry-run=server -f infra/capi/clusters/<substrate>/cluster.yaml"
  - id: ADR-0375-D4
    description: "Per-cell Argo CD app-of-apps Helm chart (infra/gitops), pull model: delivers Forgejo, Jenkins, OpenBao, observability, Kyverno, Istio Ambient. Cilium L3/L4 + Istio Ambient L7 zero-overlap (ADR-0148). Source = GitHub at bootstrap, flips to Forgejo post-cutover (ADR-0247)."
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

# ADR-0375 — Talos + Cluster API + Argo CD fleet substrate (retire Omni / OCI-TF / on-prem)

## Status
Accepted (2026-05-27). Supersedes ADR-0120 (rust-first on-prem tooling) and ADR-0121
(on-prem k8s stack: kubeadm + containerd + istio-envoy). Amends the deployment-ops-contract
(`specs/deployment-ops-contract.json` v2.0.0) accordingly.

## Context
The local substrate needed a production-fidelity, hyperscaler-shaped foundation for ~8
cross-region cells (ADR-0009, per-region packs). Three approaches were evaluated this cycle:

1. **Sidero Omni** — a proprietary fleet manager. Rejected: not how hyperscalers provision,
   proprietary/commercial, and operationally fiddly (SideroLink networking, join-token flow,
   API behind CF Access, omnictl-only).
2. **libvirt VMs on a Debian host (kubeadm/k3s-style)** — the prior on-prem model (ADR-0120/0121).
   Rejected: hand-provisioning tax (qemu perms, Docker↔libvirt iptables, mutable host), and a
   single Debian host can't be a multi-node fleet without nested-virt complexity.
3. **Talos + Cluster API + Argo CD** — the CNCF-standard declarative cluster lifecycle + GitOps.
   Adopted.

## Decision
- **Node OS:** Talos (immutable, API-managed). Bare-metal nodes auto-install zero-touch from a
  **USB** image (config baked for the hub; fetched for spoke nodes). Cloud nodes use CAPI cloud images.
- **Cluster lifecycle:** **Cluster API** — declarative `Cluster`/`MachineDeployment` CRs in git,
  reconciled by controllers. The management/hub cluster runs CAPI core + Talos providers
  (CABPT/CACPPT) + infra providers (CAPOCI / CAPA / Metal3). Provisioned out-of-band (no maintained
  libvirt CAPI provider); `clusterctl init` runs directly onto the USB-formed Talos hub (no kind).
- **Day-1 bootstrap:** a **CAPI ClusterResourceSet** installs Cilium (CNI) + Argo CD onto every
  cluster at provision time.
- **App delivery:** **per-cell Argo CD (pull model)**, Helm-first app-of-apps. Each cell
  self-reconciles from git, independent of the hub (INV-CELL-ISOLATION; ADR-0306 disaster-mode).
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
- **Known gap (not a pattern flaw):** the CAPI management hub is single-site (SPOF) until run HA
  across hosts/regions — tracked as hub-HA hardening.

## Door
Two-way: CAPI/Talos/Argo CD are replaceable OSS layers; clusters are declarative and re-provisionable.
