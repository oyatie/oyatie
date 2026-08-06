---
id: ADR-0370
status: Superseded
deciders: founder, council-architecture
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-701]
related: [ADR-0147, ADR-0165, ADR-0130, ADR-0131, ADR-0363, ADR-0341]
planning_impact: true
milestone: M-PRODUCTION-FIDELITY-SUBSTRATE
depends_on: []
door: two-way
affected_surfaces:
  crates: []
  microservices: []
  specs: [/infra/talos/]
deliverables:
  - id: ADR-0370-D1
    description: "Kata sandboxing is a KNOWN LOCAL GAP (corrected after testing): /dev/kvm is present but cloud-hypervisor cannot create its microVM — Apple-Silicon nested virt is shallow (Hypervisor.framework gives the inner VMM no complete virtual EL2/vGIC; documented analog lima-vm/lima#4498). Confirmed RAM-independent on a 24GB worker. Kata cloud-hypervisor (ADR-0147) is validated in the REAL cloud, not this local substrate; local kata-pinned workloads relax to the default runtime. (No CLH flag fixes it; kata-qemu is an uncertain, medium-heavy maybe.)"
    exit_criteria: "the gap is documented; the entire substrate + dogfood (all non-kata workloads) run normally; cloud retains kata fidelity."
    verified_by: "infra/talos/smoke-kata.sh shows /dev/kvm present + the CLH microVM-create failure; .omx/plans/kata-nested-virt-research.md"
  - id: ADR-0370-D2
    description: "Real HA control plane: 3 control-plane nodes with embedded-etcd quorum that survives the loss of one node (impossible on single-node colima+k3s)."
    exit_criteria: "killing one control-plane node keeps the apiserver reachable via the VIP and etcd quorum intact."
    verified_by: "talosctl etcd members shows 3; apiserver stays up through a 1-node kill"
  - id: ADR-0370-D3
    description: "Chaos + anti-affinity fidelity (ADR-0165/ADR-0341): node-failure/partition drills run and a 3-replica Deployment spreads across nodes — the invariants single-node cannot exercise."
    exit_criteria: "a node-drain chaos scenario runs and a 3-replica workload spreads across distinct nodes."
    verified_by: "oya gate validate substrate-fidelity (to author) + Chaos Mesh drill green"
  - id: ADR-0370-D4
    description: "Everything-as-IaC: VMs created headlessly by prlctl (create-cluster.sh), Talos config + Cilium + Kata applied by bootstrap.sh, and the substrate fleet synced by an ArgoCD app-of-apps — no hand-rolled cluster state."
    exit_criteria: "the cluster stands up from infra/talos/ scripts + GitOps with no manual kubectl/helm outside the scripts."
    verified_by: "infra/talos/ scripts reproduce the cluster; ArgoCD app-of-apps Synced/Healthy"
  - id: ADR-0370-D5
    description: "Talos secrets (machine secrets, talosconfig PKI) live in OpenBao/sops, never in git — consistent with the no-plaintext-secrets directive (ADR-0043)."
    exit_criteria: "no Talos secret material is committed; secrets are sourced from OpenBao."
    verified_by: "secret-scan clean on infra/talos/; talosconfig sourced from OpenBao"
purpose: Choose the LOCAL production-fidelity Kubernetes substrate for dogfooding the platform on this Apple Silicon host. Decision — multi-node Talos Linux on Parallels Desktop 26 (nested virt for Kata cloud-hypervisor), replacing single-node colima+k3s — driven by the platform's own invariants (ADR-0147 Kata runtimeClass, ADR-0165 chaos drills, ADR-0341 anti-affinity) which single-node and container-node clusters structurally cannot honor. Challenged via best-practice-research + adversarial verification + empirical nested-virt proof, per ADR-0368 D6.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0370: Local production-fidelity substrate — multi-node Talos on Apple Silicon

## Status
Accepted — 2026-05-26.

## Context
The local cluster had been single-node **colima + k3s** (an unratified toolchain choice). That substrate
**structurally cannot dogfood the platform's own invariants**:

- **ADR-0147** pins `runtimeClassName: kata-cloud-hypervisor` across 10+ microservices (verified:
  identity, payments, emr, cloud-kms, api-gateway, …). Kata cloud-hypervisor needs **nested
  virtualization**, which **container-node clusters (kind/k3d/OrbStack) cannot provide on macOS**, and
  for which single-node has no spare node.
- **ADR-0165** mandates nightly **node-failure / partition chaos drills** (SLO breach = release
  blocker) and **ADR-0341/3-replica anti-affinity** — all of which require **≥3 real nodes**. A single
  node cannot fail a node, partition a cell, or spread replicas.

Best-practice research (`.omx/plans/local-cluster-substrate-decision.md`,
`.omx/plans/talos-on-apple-silicon-procedure.md`) evaluated colima+k3s, k3d, kind, OrbStack, k0s,
minikube, kubeadm, and Talos. The make-or-break question — does Apple Silicon expose nested virt to a
Linux guest — was **empirically confirmed** on this host (Apple M5 Max, macOS 26.4): a Talos node
created via Parallels Desktop 26 (`--nested-virt on`) shows **`/dev/kvm` present**, so Kata
cloud-hypervisor microVMs run nested. Apple exposes nested virt on M3+/macOS 15+ via
Virtualization.framework; Parallels (and UTM's `vz` backend) surface it; **raw/manual QEMU (HVF) does
not** — hence Parallels, not QEMU.

## Decision
The local production-fidelity substrate is **multi-node Talos Linux on Parallels Desktop 26**:

1. **Topology:** 3 control-plane (embedded-etcd HA, floating VIP) + 2 workers (Kata-capable), on the
   Parallels Shared net (10.211.55.0/24). Talos is immutable + API-managed (no SSH) — genuine prod
   fidelity, not a dev shim.
2. **Hypervisor = Parallels 26** (`prlctl` creates/configures/boots VMs **headlessly** with
   `--nested-virt on`) — the zero-hand-rolling VM layer. UTM `vz` is the free fallback but cannot
   create VMs headlessly.
3. **Kata cloud-hypervisor** via the Talos `siderolabs/kata-containers` system extension (baked into an
   Image Factory image; the extension ships **cloud-hypervisor** as its sole hypervisor). A
   RuntimeClass named `kata-cloud-hypervisor` aliases the extension's `kata` handler to match the
   platform's pinned `runtimeClassName`.
4. **Cilium CNI** (kube-proxy replacement via Talos KubePrism) — Talos ships no default CNI/kube-proxy.
5. **Everything-as-IaC** under `infra/talos/`: `create-cluster.sh` (prlctl VM creation),
   `bootstrap.sh` (gen-config → apply → bootstrap → Cilium → Kata), `smoke-kata.sh` (fidelity proof);
   the substrate fleet (Jenkins/GitHub/OpenBao/ArgoCD/Rollouts/observability/Valkey) syncs via an
   ArgoCD app-of-apps. `colima` is retired for the substrate (kept optional for the inner dev loop).

## Rejected alternatives
- **colima + k3s (single-node, status quo)** — rejected: no HA (no 3-node etcd quorum), no node-failure
  drills, no real anti-affinity spread, and no spare node for the Kata runtime class. Cannot honor
  ADR-0147/0165/0341.
- **kind / k3d / OrbStack (container-node clusters)** — rejected: container "nodes" share the host
  kernel and **cannot run Kata cloud-hypervisor nested on macOS**; fast DX but not prod-fidelity.
- **UTM (`vz` backend)** — viable + free + nested-virt default-on, but `utmctl` cannot create VMs
  headlessly (GUI/AppleScript only). Kept as the no-license fallback; Parallels wins on full `prlctl`
  IaC.
- **Manual QEMU / talosctl QEMU provisioner** — rejected: QEMU on Apple Silicon uses HVF, which does
  **not** expose nested virt (so no Kata), and the darwin/arm64 path has active blocking bugs.
- **Talos `docker` provisioner** — rejected: container nodes, no nested Kata.
- **Managed cloud K8s** — out of scope for the local dogfood host; the cloud substrate is a separate
  decision.

## Consequences
- Positive: the local cluster honestly dogfoods Kata cloud-hypervisor, HA etcd, chaos drills, and
  anti-affinity — the platform is exercised the way production is. Fully IaC + GitOps, reproducible.
- Negative/cost: 5 VMs (~72 GB) require **stopping colima** (96 GB) to fit 128 GB; Parallels is a paid
  dependency (UTM is the free fallback); a one-time `prlctl` VM-creation layer + the bootstrap scripts
  are maintenance surface. Talos's immutability means no SSH debugging (API-only) — intended.
- Neutral: the VCS/CI substrate (ADR-0363 git+Jenkins+GitHub) is unchanged; this is the compute
  substrate beneath it.

## Verification
Per-deliverable `verified_by`. **D1 CORRECTED after testing: Apple-Silicon nested virt is SHALLOW** —
`/dev/kvm` is present but cloud-hypervisor cannot create its microVM (no complete vEL2/vGIC to the inner
VMM; documented limit, lima-vm/lima#4498), confirmed RAM-independent on a 24GB worker. Kata is a known
LOCAL gap, validated in the real cloud (see `.omx/plans/kata-nested-virt-research.md`); the local
substrate + dogfood run as normal (non-kata) workloads. The cluster is otherwise production-fidelity:
HA control plane (3×apiserver/cm/scheduler + 3-member etcd), Cilium, all normal workloads. D2 (HA etcd
1-node-kill), D3 (chaos drill + 3-replica spread), D4 (IaC + ArgoCD app-of-apps reproduces the cluster),
and D5 (secrets in OpenBao) are validated as the substrate migrates onto the cluster. This ADR was
challenged per ADR-0368 D6: best-practice-research compared all substrates, the load-bearing claims
(Kata pin, chaos drills) were verified against the repo, and the make-or-break nested-virt assumption
was proven empirically rather than assumed.

## References
ADR-0147 (Kata cloud-hypervisor runtime ladder), ADR-0165 (chaos engineering substrate), ADR-0341
(cellular promotion / anti-affinity), ADR-0130/0131 (observability substrate + layout), ADR-0363
(git+Jenkins+GitHub substrate), ADR-0368 (north-star: even architecture is challenged). Research:
`.omx/plans/local-cluster-substrate-decision.md`, `.omx/plans/talos-on-apple-silicon-procedure.md`.
IaC: `infra/talos/`.
