---
id: ADR-0381
status: Superseded
planning_impact: true
deciders: founder, ops-platform, council-architecture
date: 2026-05-28
owner: ops-platform
supersedes: []
superseded_by: []
related: [ADR-0378, ADR-0349, ADR-0380, ADR-0148, ADR-0083, ADR-0375]
related_specs: [/specs/deployment-ops-contract.json]
milestone: M-LOCAL-CI-SUBSTRATE
depends_on: [ADR-0378]
door: two-way
affected_surfaces:
  crates: []
  microservices: [ci-webhook-gateway]
  specs: []
deliverables:
  - id: ADR-0381-D1
    description: "Replace Kaniko (Google Container Tools — placed into maintenance/archive in 2024; GitHub repo is read-only) with BuildKit (Moby, Apache 2 — what Docker itself uses) as the in-cluster image-build substrate. Rewrite infra/ci-webhook-gateway/kaniko-build.yaml as buildkit-build.yaml (a buildkitd Deployment on the CI specialty pool from D2 + a buildctl client invoked from the Jenkins agent pod template); update infra/registry/registry.k8s.yaml and microservices/ci-webhook-gateway/Dockerfile so the build path is buildctl-driven. Wire BuildKit's `s3` cache backend to SeaweedFS-on-Talos (per ADR-0349) once D4 (storage pool + SeaweedFS) lands. Hyperscaler-lens: BuildKit is Apache 2, actively maintained, used by Docker / GitHub Actions / Cloud Build / Earthly — passes (a)-(d)."
    exit_criteria: "All Kaniko references in infra/ + microservices/ci-webhook-gateway/ are replaced with BuildKit equivalents (git grep -i kaniko returns 0 matches in those trees); an end-to-end in-cluster image build produces an OCI image consumable by ArgoCD without using Kaniko."
    verified_by: "git grep -i 'kaniko' -- infra/ microservices/ci-webhook-gateway/ shows 0 matches; a buildkit-built image is pulled + deployed via ArgoCD in a smoke run."
  - id: ADR-0381-D2
    description: "Multi-node Talos cell topology — replace the current single-node Talos VM with a multi-pool cluster: 3 control-plane nodes (etcd quorum, 2 vCPU + 2 GiB each, taint=node-role.kubernetes.io/control-plane:NoSchedule), 2 worker nodes (tenant workloads, PSA restricted, 4 vCPU + 8 GiB each), 1 CI specialty pool node (cargo build agents + buildkitd from D1, 6 vCPU + 16 GiB, label=oya.cell/ci=true + taint=dedicated=ci:NoSchedule), 1 storage specialty pool node (SeaweedFS data nodes from D4, 2 vCPU + 8 GiB + 100 GiB disk, label=oya.cell/storage=true). All nodes are Talos VMs managed by vfkit (per ADR-0378). Total recommended: ~22 vCPU + ~46 GiB — sizeable but pageable on a 32-GiB+ macOS host. Dial-down knobs documented for 16-GiB hosts (1 CP + 1 worker + 1 CI specialty + SeaweedFS co-located on worker — loses CP HA + storage-pool isolation, kept for resource-constrained dev). Hyperscaler-lens: Talos is Apache 2, actively maintained (Sidero Labs quarterly releases + CAPI integration), and the multi-pool topology IS what GKE/EKS/AKS themselves run — passes (a)-(d)."
    exit_criteria: "kubectl get nodes shows >=6 nodes with correct labels + taints; a CI-tier pod schedules onto the CI specialty node and never onto a CP or worker; etcd quorum survives the cordoning of one CP node (the cluster stays writable)."
    verified_by: "kubectl get nodes -o wide + a CP-failure drill (kubectl cordon + drain one CP, verify cluster remains healthy and a new pod schedules successfully)."
  - id: ADR-0381-D3
    description: "Cell-boundary enforcement: Cilium L3/L4 NetworkPolicies (per ADR-0148) restrict cross-cell traffic to explicit seams (oya.cell/foundation - oya.cell/tenant - oya.cell/ci - oya.cell/storage); ADR-0083 pod-runtime-tier annotations are translated to nodeSelector + tolerations so Tier-1 (tenant) pods land on the worker pool, Tier-3 (CI batch) on the CI specialty pool, foundation/control plane on the CP pool. No tenant pod can reach a CI agent pod; no CI agent can reach a tenant workload (only the foundation control plane spans cells, with explicit allowed seams)."
    exit_criteria: "kubectl exec from a tenant pod to a CI agent pod fails (NetworkPolicy denial); a pod annotated with runtime-tier=1 schedules onto a worker (not a CP, not a CI specialty); kubectl describe verifies the nodeSelector + tolerations match the tier."
    verified_by: "NetworkPolicy denial test (kubectl exec curl across cells fails); a tier-1 + tier-3 pod scheduling-test confirms node-pool affinity."
  - id: ADR-0381-D4
    description: "Restore SeaweedFS-on-Talos as the cluster-internal S3-API-compatible object store (per ADR-0349); deploy onto the storage specialty pool from D2; back BuildKit's `s3` cache backend (from D1) and Jenkins agent sccache (the deferred-half of ADR-0380 D6) on the same SeaweedFS instance. Single object-store substrate serves CI cache + image-registry overlay + artifact storage. Zero AWS S3 dependency anywhere in the data path. Hyperscaler-lens: SeaweedFS is Apache 2, actively maintained, S3-API-compatible — the OSS analogue of S3/Colossus/Blob; passes (a)-(d)."
    exit_criteria: "SeaweedFS pods run on storage specialty nodes (label oya.cell/storage=true); BuildKit cache writes succeed against the SeaweedFS S3 backend (cache-miss → write → cache-hit → read works end-to-end); Jenkins agent sccache can be re-enabled and shares the same SeaweedFS substrate."
    verified_by: "kubectl exec into a SeaweedFS pod + check buckets exist; a buildkit build with --cache-from registry,ref=s3://... + --cache-to type=s3 demonstrates write+hit cycle; ADR-0380 D6 sccache wiring can be smoke-tested off this substrate."
purpose: >
  Capture the two substrate corrections caught during the ADR-0380 amendment cycle (2026-05-28):
  Kaniko archived (migrate to BuildKit) + single-node Talos parallelism/HA bottleneck (migrate to
  multi-node CP/Worker/Specialty cell topology). Both decisions are validated against the
  standing hyperscaler-grade self-hosted-substrate lens (memory:
  hyperscaler-lens-architectural-filter): active upstream, license-clean, fully self-hostable,
  hyperscaler-internal-equivalent. Status is Proposed because this ADR captures the decision-
  space + recommended choices + hyperscaler-lens validation; the implementation IPs are
  authored as follow-ons, gated on the ADR-0380 amendment landing first.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0381: Kaniko → BuildKit migration + multi-node Talos cell topology

## Status

Proposed (2026-05-28). This ADR captures the decision-space + recommended choice
+ hyperscaler-lens validation for two substrate corrections caught mid-cycle in
ADR-0380's amendment. Implementation IPs follow once the ADR-0380 amendment
(this branch) lands on dev.

## Context

Two substrate-correctness problems surfaced during the ADR-0380 amendment cycle
(see ADR-0380 amendment (5)):

1. **Kaniko is archived.** Google Container Tools placed Kaniko into
   maintenance/archive in 2024; the GitHub repo is read-only. Current Oyatie
   references: `infra/ci-webhook-gateway/kaniko-build.yaml`,
   `infra/registry/registry.k8s.yaml`, and `microservices/ci-webhook-gateway/Dockerfile`.
   Running our image-build substrate on an archived upstream is operationally and
   security-wise wrong: no CVE patches, no new features, no community to escalate
   to.

2. **Local Talos is single-node** (~6 vCPU). This caps concurrent agent pods
   (the ADR-0380 D6 max-parallelism ceiling is gated on multi-node capacity),
   provides zero control-plane HA (single etcd, single apiserver), provides no
   cell-pattern enforcement (every pod lands on the same node — Cilium L3/L4
   per ADR-0148 + ADR-0083 pod-runtime-tier have nowhere to apply), and does not
   validate the production topology locally before deployment.

Both corrections must apply the standing **hyperscaler-grade self-hosted-substrate
lens** (per the `hyperscaler-lens-architectural-filter` feedback memory):
(a) active upstream — no archived projects;
(b) license-clean — Apache 2 / MIT / BSD / LGPL only;
(c) fully self-hostable — no managed-service dependency;
(d) hyperscaler-internal-equivalent — the OSS substrate a hyperscaler would
itself run, not a thing that only exists as their managed offering.

## Decision

### Image-build substrate: BuildKit

**Choice:** BuildKit (Moby project, Apache 2, https://github.com/moby/buildkit).

**Hyperscaler-lens validation:**
- **(a) Active upstream:** Moby/Docker maintains it; multiple releases per year;
  used by `docker buildx` itself. Passes.
- **(b) License-clean:** Apache 2. Passes.
- **(c) Fully self-hostable:** `buildkitd` runs as a Deployment in our cluster
  (on the CI specialty pool per D2); `buildctl` is the client invoked from
  Jenkins agent pods. No managed-service dependency. Passes.
- **(d) Hyperscaler-internal-equivalent:** What GCP Cloud Build, GitHub Actions
  build cache, Earthly, Depot.dev, Docker Hub itself use — the de-facto
  industry-standard daemonless container builder. Passes.

**Why not Buildah?** Strong alternative (Red Hat, Apache 2, daemonless, rootless-
first). It also passes the lens. BuildKit wins on adoption + cache-backend
ecosystem (BuildKit has native `registry / s3 / inline / gha` backends; the `s3`
backend is a perfect fit for SeaweedFS-on-Talos per D4). Buildah remains the
fallback if BuildKit hits an operational issue.

**Why not Kaniko?** Lens (a) fails — archived.

**Why not in-pod Docker (`docker:dind`)?** Requires `privileged: true` pods —
PSA-restricted incompatible; security regression.

### Multi-node Talos topology: CP / Worker / Specialty pools

**Topology** (recommended baseline; dial-down knobs documented for resource-
constrained hosts):

| Pool | Count | Spec | Purpose | Cell label |
|------|-------|------|---------|------------|
| Control plane | 3 | 2 vCPU + 2 GiB each | etcd quorum + apiserver HA | `oya.cell/foundation=true` |
| Worker (tenant) | 2 | 4 vCPU + 8 GiB each | tenant workloads, PSA-restricted | `oya.cell/tenant=true` |
| CI specialty | 1 | 6 vCPU + 16 GiB | cargo build agents + buildkitd | `oya.cell/ci=true` + taint `dedicated=ci:NoSchedule` |
| Storage specialty | 1 | 2 vCPU + 8 GiB + 100 GiB disk | SeaweedFS data nodes | `oya.cell/storage=true` |
| **Total** | **7 VMs** | **~22 vCPU + ~46 GiB** | | |

All nodes are Talos VMs managed by vfkit (per ADR-0378). Cilium L3/L4 enforces
cell boundaries (per ADR-0148 — ztunnel/Ambient layered above per the same ADR).
ADR-0083 pod-runtime-tier annotations are translated to nodeSelector +
tolerations:

- Tier-0 (foundation / control plane): CP-only or special foundation worker.
- Tier-1 (tenant workloads): worker pool.
- Tier-3 (CI build, batch, deferred): CI specialty pool.

**Hyperscaler-lens validation:**
- **(a) Active upstream:** Talos / Sidero Labs ships quarterly; CAPI integration
  current. Passes.
- **(b) License-clean:** Apache 2. Passes.
- **(c) Fully self-hostable:** runs as VMs on any host. Passes.
- **(d) Hyperscaler-internal-equivalent:** Talos is the OSS analogue of GKE's
  Container-Optimized OS / EKS's Bottlerocket / AKS's CBL-Mariner — minimal,
  immutable, security-hardened node OS. The multi-pool topology IS what GKE node
  pools / EKS managed node groups / AKS node pools provide. Passes.

**Dial-down for 16-GiB macOS hosts:** 1 CP + 1 worker + 1 CI specialty;
SeaweedFS co-locates on the worker. Loses CP HA + storage-pool isolation;
documented as a known compromise. Targeted for dev hosts < 32 GiB.

### Cell-boundary enforcement (D3)

Cilium NetworkPolicy seams (per ADR-0148):

- **tenant ↛ ci**: a tenant pod cannot reach a CI agent pod.
- **ci ↛ tenant**: a CI agent cannot reach a tenant workload pod.
- **storage ↛ tenant**: SeaweedFS pods are reachable from CI + foundation, not
  directly from tenant.
- **foundation → all (allowed seams only)**: control plane reaches what it needs.

ADR-0083 pod-runtime-tier → node-pool affinity is the scheduler-side
counterpart: tier-1 pods cannot schedule onto CI specialty (nodeSelector
mismatch), tier-3 batch cannot schedule onto worker (taint mismatch).

### Object-store substrate (D4)

SeaweedFS-on-Talos (per ADR-0349 restoration) is the cluster-internal
S3-API-compatible object store. It serves three workloads simultaneously:

1. BuildKit `s3` cache backend (D1).
2. Jenkins agent sccache (the deferred half of ADR-0380 D6).
3. ArgoCD artifact / image-registry overlay storage.

A single object-store substrate; zero AWS dependency.

## Consequences

**Positive:**
- Build substrate has no archived dependency.
- Local cluster matches production topology (CP / Worker / Specialty cells) —
  Cilium + ADR-0083 wiring is validated locally before prod.
- ADR-0380 D6 max-parallelism ceiling lifts (more CI specialty nodes =
  more concurrent agents; storage specialty pool enables cache reuse).
- Single object-store substrate (D4) consolidates cache + artifact storage on
  Oyatie-owned, hyperscaler-lens-clean foundation.

**Negative:**
- Resource cost on dev macOS host (22 vCPU + 46 GiB recommended baseline;
  dial-down available but with documented compromises).
- Migration friction: each existing manifest (kaniko-build.yaml, etc.) must be
  rewritten; multi-node bring-up time exceeds single-node by minutes per VM.
- BuildKit + SeaweedFS cache wiring is a new integration; cache-miss semantics
  must be tested before sccache is re-enabled (ADR-0380 D6 depends on D4
  landing first).

## Alternatives Considered

- **Buildah** (image build): strong alternative; lost on adoption + cache-backend
  ecosystem maturity. Kept as fallback.
- **img** (image build): also archived; lens (a) fails.
- **k3s with multi-node**: lighter than Talos but contradicts ADR-0378 (Talos
  canonical). Stay with Talos.
- **Single-node + bigger VM**: gets vertical parallelism but no HA, no
  cell-pattern enforcement, doesn't validate production topology locally.
- **AWS-managed equivalents** (CodeBuild, ECR, EKS managed nodes): fails lens
  (c) + (d) — hyperscaler-managed-service dependency contradicts Oyatie's
  positioning as itself a cloud provider.

## Related

- ADR-0378: vfkit + Talos canonical local substrate.
- ADR-0349: CI farm + SeaweedFS object store (canonical).
- ADR-0380: CI-loop closure on Talos (D1-D5 MVP + D6 max-parallelism path);
  this ADR is the substrate-correctness companion.
- ADR-0148: Service mesh: Cilium L3/L4 + Istio Ambient L7.
- ADR-0083: Pod runtime tier panic policy.
- ADR-0375: Talos + CAPI + Argo CD fleet substrate.

## Memory references

- `hyperscaler-lens-architectural-filter`: the standing meta-rule used as the
  choice-filter for D1-D4 above (every choice validated against
  (a) active upstream, (b) clean license, (c) self-hostable, (d) hyperscaler-
  internal-equivalent).
- `vfkit-talos-canonical-local-substrate`: the substrate fact this ADR builds on.
- `talos-local-stack-state`: the resumable single-node baseline this ADR
  migrates away from.
