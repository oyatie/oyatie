---
id: ADR-0378
status: Superseded
planning_impact: true
deciders: founder, ops-platform, council-architecture
date: 2026-05-27
owner: ops-platform
supersedes: []
superseded_by: [ADR-701]
related: [ADR-0375, ADR-0376, ADR-0363, ADR-0349, ADR-0130, ADR-0131, ADR-0132]
related_specs: [/specs/deployment-ops-contract.json]
milestone: M-LOCAL-CI-SUBSTRATE
depends_on: [ADR-0375]
door: two-way
affected_surfaces:
  crates: []
  microservices: []
  specs: []
deliverables:
  - id: ADR-0378-D1
    description: "Standardize the local developer/CI substrate on vfkit + Talos Linux (immutable, API-driven, no-SSH; upstream Kubernetes v1.36.1 + Cilium CNI) and RETIRE colima (Lima VM running k3s/docker). One substrate, production-fidelity, dogfood-correct per oyatie-dogfood-tenancy. Records the founder decision; no product code in this lane."
    exit_criteria: "docs/decisions/ADR-0378-*.md names vfkit+Talos as the single canonical local substrate, names colima as retired, and states the upstream-k8s-parity rationale; the ADR index lists ADR-0378 as Accepted."
    verified_by: "cloud-ci/Rust gate packet adr-index"
  - id: ADR-0378-D2
    description: "Canonicalize cluster access: config home ~/.oya/talos-local/ (talosconfig/kubeconfig/controlplane.yaml); kube context admin@oya-local merged into ~/.kube/config as the DEFAULT so kubectl targets the Talos cluster (not the near-empty colima context that previously caused wrong-cluster checks). Manage the single node directly via talosctl."
    exit_criteria: "kubectl config current-context resolves to admin@oya-local and `kubectl get pods -n oya-ci-jenkins` shows the running Jenkins pod; ~/.oya/talos-local/ holds the canonical configs."
    verified_by: "operator check: kubectl config current-context == admin@oya-local"
  - id: ADR-0378-D3
    description: "Retire colima (stop then delete the default + omni profiles) and Sidero Omni (fleet-management overkill for a single local node), recovering reserved host resources. Container image builds run in-cluster (cloud-ci on Talos) or via a buildkit pod, not a host docker daemon."
    exit_criteria: "colima list shows no running profile; no host workflow depends on a colima docker context."
    verified_by: "operator check: colima list shows no Running profile"
  - id: ADR-0378-D4
    description: "Future-work leg (named, not built here): close the CI loop on this substrate — GitHub push -> cloud-ci pipeline -> commit-status back -> gated merge — which retires the temporary GitHub admin-merge seam (ADR-0363). Tracked as the CI-webhook + ed25519 lane."
    exit_criteria: "the ADR names CI-loop closure (GitHub->cloud-ci commit-status) as the deliverable that retires the admin-merge seam, with a follow-on lane reference."
    verified_by: "cloud-ci/Rust gate packet adr-index"
purpose: >
  Standardize the LOCAL developer/CI substrate on vfkit + Talos Linux — the same
  immutable, API-driven, upstream-Kubernetes OS Oyatie operates and ships per
  ADR-0375 — and RETIRE colima (Lima/k3s/docker dev-convenience). One substrate,
  production-fidelity, dogfood-correct. The real local stack (cloud-ci, GitHub,
  Cilium, local-path storage) already runs on the Talos node; this ADR makes it
  canonical, fixes kube-context access so the cluster cannot be mistaken for "down",
  and removes the colima divergence. This GitHub+cloud-ci-on-Talos substrate is the
  end-state that retires the GitHub admin-merge seam (ADR-0363) once CI gates merges.
---

# ADR-0378 — Canonical local substrate: vfkit + Talos Linux (retire colima)

## Status
Accepted (2026-05-27). Founder decision ("colima or vfkit? pick one and commit").
Builds on ADR-0375 (Talos + Cluster API + Argo CD fleet substrate) and ADR-0363
(git + cloud-ci + GitHub (interim) CI substrate); extends them to the local box.

## Context
Two local substrates had accreted on the development MacBook, both on Apple
Virtualization.framework:

1. **vfkit + Talos Linux** — Talos (immutable, API-driven, no SSH/shell, declarative
   machine config) running upstream Kubernetes v1.36.1 + Cilium 1.19.4 CNI, on which
   the real local stack runs: Jenkins (`oya-ci-jenkins/oya-jenkins-0`), GitHub
   (`oya-forge/github-*` + `oya-forge/wave3-github-webhook-sink`), local-path storage.
2. **colima** — a Lima VM running docker/k3s as a developer convenience: one stopped
   profile (14 vCPU / 96 GiB / 256 GiB reserved) and one running `omni` docker profile
   hosting Sidero Omni.

`kubectl` defaulted to the near-empty colima context, so the live cloud-ci/GitHub
cluster could appear "down" when it was up — a fidelity and operability hazard. Oyatie
builds cloud infrastructure and a managed-Kubernetes product (ADR-0375/0376); its own
local substrate must be the production article, not a divergent convenience box.

## Decision
**vfkit + Talos is the single canonical local substrate; colima is retired.** The
distinction is the guest, not the hypervisor (both use Apple VZ): Talos is the
production Kubernetes OS; colima is a laptop k3s/docker box.

1. Talos node (vfkit VM, 192.168.64.3) managed directly via `talosctl`; canonical
   config home `~/.oya/talos-local/`. Kube context `admin@oya-local` is the default.
2. Bring-up is declarative ArgoCD app-of-apps (`infra/gitops/bootstrap-sync.yaml`,
   ADR-0515 D3; `bring-up.sh` eliminated). Apply the manifest after ArgoCD
   bootstrap; ArgoCD syncs Cilium -> local-path -> VCS substrate in wave order.
3. colima and Sidero Omni are retired; a single local node needs only `talosctl`.
4. Upstream Kubernetes parity (not k3s) so local Cilium/Kyverno/ArgoCD/manifest
   behavior matches what Oyatie operates and ships.

## Rejected alternatives
- **colima (Lima + k3s/docker)** — rejected: k3s diverges from upstream (flannel,
  traefik, klipper-lb, embedded datastore, trimmed alpha surfaces); a developer box,
  not the production substrate; a dogfood mismatch for a company shipping managed-k8s.
- **Run both** (colima for host docker builds, Talos for runtime) — rejected: the
  directive is one substrate, and dual substrates caused exactly the wrong-context
  hazard observed; container builds run in-cluster (cloud-ci on Talos) or in a buildkit pod.

## Consequences
- Positive: production fidelity, dogfood correctness, a single source of truth for the
  local cluster, and recovered host resources (the colima reservations).
- Negative/cost: Talos has a steeper on-ramp than `colima start`, and there is no
  host `docker` daemon (use in-cluster builds or a buildkit pod).
- Neutral: this substrate is the end-state that retires the GitHub admin-merge seam
  (ADR-0363) once GitHub->cloud-ci commit-status gates merges (ADR-0378-D4).

## Verification
Per-deliverable `verified_by`: `kubectl config current-context` == `admin@oya-local`
with cloud-ci/GitHub/Cilium Running on the Talos node; `colima list` shows no running
profile after retirement; the ADR index lists ADR-0378 as Accepted.

## References
ADR-0375 (Talos + Cluster API + Argo CD fleet substrate), ADR-0376 (managed-Kubernetes
product surface), ADR-0363 (git + cloud-ci + GitHub (interim) substrate), ADR-0349
(CI farm), ADR-0130 (observability substrate), ADR-0131/0132 (flat single-concern
microservice layout). External: Talos Linux (https://www.talos.dev/), Cilium
(https://cilium.io/), colima (https://github.com/abiosoft/colima), Sidero Omni
(https://omni.siderolabs.com/), Apple Virtualization.framework
(https://developer.apple.com/documentation/virtualization).
