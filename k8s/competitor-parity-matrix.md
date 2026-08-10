---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cloud + council-architecture
deciders: axis-cloud, council-architecture, gtm-customer-success
related_adrs: [ADR-0117, ADR-0121, ADR-0123]
related_artifacts:
  - k8s/PRD.md (§"Competitive Benchmark")
  - /specs/hyperscaler-gates.json (HG-CLOUD-K8S gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (cloud-k8s µservice)

## Purpose

Quantitative + qualitative parity comparison vs the industry-leading managed Kubernetes + on-prem Kubernetes distributions. Drives the `oya-governance-hyperscaler-maturity-claims` gate (per ADR-0123 HG-CLOUD-K8S) and tells gtm-customer-success what to claim. Re-validated bi-annually.

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| AWS EKS | Managed Kubernetes | Global region coverage; IRSA workload identity; ALB integration | `docs.aws.amazon.com/eks/` |
| GCP GKE | Managed Kubernetes | Autopilot; Workload Identity; Anthos Service Mesh | `cloud.google.com/kubernetes-engine/docs/` |
| Azure AKS | Managed Kubernetes | AAD integration; Azure CNI; Istio add-on | `learn.microsoft.com/azure/aks/` |
| Oracle OKE | Managed Kubernetes | KR + global; same control-plane code path | `docs.oracle.com/en-us/iaas/Content/ContEng/` |
| Rancher RKE2 | On-prem K8s distro | Hardened defaults; air-gapped | `docs.rke2.io` |
| OpenShift | On-prem K8s distro | OpenShift-specific operators; Red Hat support | `docs.openshift.com` |
| Tanzu Kubernetes Grid | On-prem K8s distro | VMware integration; cluster-API | `docs.vmware.com/en/VMware-Tanzu-Kubernetes-Grid/` |
| Talos Linux + Omni | On-prem K8s distro | API-driven OS for K8s | `talos.dev` |

## Feature Parity Matrix

### Cluster lifecycle

| Capability | oyatie | EKS | GKE | AKS | OKE | RKE2 | OpenShift | TKG |
|---|---|---|---|---|---|---|---|---|
| Bootstrap to Ready ≤ 30min p99 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Vanilla upstream Kubernetes (CNCF conformance) | ✅ | ✅ | ✅ | ✅ | ✅ | partial | partial (OCP additions) | ✅ |
| kubeadm-based (audit-friendly) | ✅ | ❌ (managed CP) | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| LTS-pinned versions per pack | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| HA control-plane (3-node etcd) | M04 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Auto-scaling node groups | M02 | ✅ | ✅ (Autopilot) | ✅ | ✅ | partial | partial | ✅ |
| Multi-AZ control plane | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Networking

| Capability | oyatie | EKS | GKE | AKS | OKE | RKE2 | OpenShift | TKG |
|---|---|---|---|---|---|---|---|---|
| CNI choice | Cilium (eBPF) | aws-vpc-cni / Cilium | gke-cni / Cilium | azure-cni / Calico | OCI VCN-native | Canal / Cilium | OpenShift SDN / OVN | Antrea / Calico |
| NetworkPolicy enforcement | Cilium kernel-layer | aws-vpc-cni / Calico | gke-cni | Azure NetworkPolicy | OCI native | Canal | OVN-K8s | Antrea |
| Service mesh native | Istio 1.29 | partial (App Mesh) | Anthos Service Mesh (Istio) | Istio add-on | none-built-in | none-built-in | OpenShift Service Mesh (Istio) | Tanzu Service Mesh |
| mTLS strict default | ✅ | manual | partial | partial | manual | manual | partial | manual |
| Multi-cluster mesh | M03 (Istio multi-cluster) | partial (App Mesh) | Anthos | partial | partial | manual | ✅ | ✅ |
| Hubble flow logs | ✅ | partial | partial | partial | partial | partial | partial | partial |

### Security / Supply Chain

| Capability | oyatie | EKS | GKE | AKS | OKE | RKE2 | OpenShift | TKG |
|---|---|---|---|---|---|---|---|---|
| Cosign + Kyverno admission | ✅ | partial | partial | partial | partial | partial | partial | partial |
| Pod Security Standard `restricted` enforced | ✅ | manual | manual | manual | manual | ✅ | ✅ | manual |
| etcd KMS envelope at-rest | ✅ | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ |
| CIS Kubernetes Benchmark v1.9 (BLOCKER lane) | ✅ | manual scan | manual scan | manual scan | manual scan | ✅ | ✅ | manual |
| NSA/CISA Hardening Guide lane | ✅ | manual | manual | manual | manual | partial | partial | manual |
| Cedar policy on every API call | ✅ | ❌ (IAM only) | ❌ (IAM only) | ❌ (IAM only) | ❌ (IAM only) | ❌ | ❌ | ❌ |
| Audit-chain Ed25519 seal per mutation | ✅ | ❌ (audit-log only) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| kubernetes-api-proxy mediation (no direct 6443) | ✅ | partial (IAM-gated) | partial | partial | partial | ❌ | partial | ❌ |

### Storage / CSI

| Capability | oyatie | EKS | GKE | AKS | OKE | RKE2 | OpenShift | TKG |
|---|---|---|---|---|---|---|---|---|
| Block / Object / File CSI drivers per backend | ✅ (per pack) | ✅ (EBS, S3, EFS) | ✅ (PD, GCS, Filestore) | ✅ (Azure Disk, Blob, Files) | ✅ (OCI Block, Object, File) | manual | ✅ | manual |
| VolumeSnapshot integration | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Per-PV encryption (KMS-managed) | ✅ | ✅ | ✅ | ✅ | ✅ | partial | ✅ | partial |

### Operations / Cost

| Capability | oyatie | EKS | GKE | AKS | OKE | RKE2 | OpenShift | TKG |
|---|---|---|---|---|---|---|---|---|
| Per-pack cluster (data-residency) | ✅ (11 packs) | regional (28 regions) | regional (40 regions) | regional (60 regions) | regional (44 regions) | self-hosted | self-hosted | self-hosted |
| HIPAA BAA | conditional | ✅ | ✅ | ✅ | ✅ | n/a | n/a | n/a |
| KR-CSAP / FSS | conditional | ❌ | ❌ | ❌ | conditional | n/a | n/a | n/a |
| Cost: managed cluster fee | $0 (self-hosted) | $73/month/cluster | $73/month/cluster | $73/month/cluster | $0 (free) | $0 | subscription | subscription |
| Cost: worker nodes | OCI Block + worker VM | EC2 | GCE | Azure VM | OCI VM | self-hosted | self-hosted | self-hosted |
| FinOps automation | M02 (per cost-budget.md) | partial (Cost Explorer) | partial (Cost Mgmt) | partial | partial | ❌ | partial | partial |

### Agent Operability

| Capability | oyatie | EKS | GKE | AKS | OKE | RKE2 | OpenShift | TKG |
|---|---|---|---|---|---|---|---|---|
| Foundry-callable cluster mutators | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Autonomy-tier per capability | ✅ (T1/T2/T3) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| 2-person rule for T3 ops | ✅ | manual | manual | manual | manual | manual | manual | manual |
| Cedar-derived NetworkPolicy + AuthorizationPolicy from tenant fragments | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

## Quantitative Performance Parity

(30-day rolling-window evaluations on equivalent workloads.)

| Metric | oyatie target | EKS reference | GKE reference | Notes |
|---|---|---|---|---|
| Cluster bootstrap (control-plane Ready) p99 | ≤ 30 min | ≤ 15 min (managed CP) | ≤ 5 min (Autopilot) / ≤ 15 min (Standard) | oyatie includes Istio + Cilium + CSI install in bootstrap; EKS/GKE bootstrap is CP-only |
| Node-join (Ready) p99 | ≤ 5 min | ≤ 3 min | ≤ 3 min | parity |
| NetworkPolicy propagation p99 | ≤ 30 s | ≤ 10 s (Cilium) | ≤ 10 s | parity (same Cilium) |
| Istio xDS push latency p99 | ≤ 10 s | n/a (App Mesh diff) | ≤ 10 s (Anthos) | parity |
| kube-apiserver request p99 | ≤ 500 ms | ≤ 500 ms (managed) | ≤ 500 ms | parity |
| api-proxy decision latency (Cedar + audit emit) | ≤ 50 ms | n/a | n/a | oyatie unique |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | HA control plane (3-node etcd) | axis-cloud + ops-sre-reliability | M04 |
| 2 | Cluster Autoscaler integration | axis-cloud | M02 |
| 3 | Multi-cluster mesh federation | axis-cloud | M03 |
| 4 | Workload Identity Federation (SPIFFE ↔ IRSA / GKE WI / AKS WI) | axis-cloud + ops-security | M03 |
| 5 | Karpenter / fast-bin-pack node provisioning | axis-cloud | M04 |
| 6 | Region coverage (oyatie 11 packs vs EKS 28 regions) | axis-cloud + ops-finops | progressive; per pack activation |

## Key oyatie Differentiators (NOT in any competitor)

1. **Foundry-callable cluster mutators**: every cluster operation is a Foundry capability with autonomy ceiling + audit-chain seal + 2-person rule for T3. No competitor exposes this shape.
2. **Cedar-derived NetworkPolicy / AuthorizationPolicy from tenant fragments**: tenant Cedar policy → atomic NetworkPolicy + AuthorizationPolicy CR pair. Competitors require operators to hand-author CRs.
3. **kubernetes-api-proxy with Cedar policy on every API call + audit-chain emission**: competitors expose raw kube-apiserver (with RBAC + audit-log only).
4. **Sovereignty-grade per-pack cluster boundary + cross-pack mTLS-only federation**: zero cross-pack PV / etcd replication by design.
5. **CIS K8s Benchmark v1.9 + NSA/CISA Hardening Guide as BLOCKER CI lanes**: not just scanned post-deploy.

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "cloud-k8s is the only Kubernetes substrate with cryptographic audit-chain over every cluster mutation" (true as of 2026-05-17; review bi-annually).
- ✅ "oyatie's per-pack cluster boundary exceeds Datadog / Grafana Cloud / single-region competitors on data-residency" (true).
- ✅ "Vanilla upstream Kubernetes (no Rancher / OpenShift-specific bits)" (true; ADR-0121).
- ✅ "kubeadm + containerd + Istio + Envoy: same components OKE workers run; portable to multi-cluster federation without re-validation" (true).

Sales claims FORBIDDEN (per ADR-0123):
- ❌ "oyatie is faster than EKS on bootstrap" (no published benchmark; comparison apples-vs-oranges due to bundled mesh + CNI install).
- ❌ "oyatie is HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation).
- ❌ "We beat OKE on cost" (depends on workload shape).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-cloud |
| 3. Re-run quantitative benchmarks against current versions | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary rule updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `k8s/PRD.md` §"Competitive Benchmark".
- `/specs/hyperscaler-gates.json` HG-CLOUD-K8S gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0121 (on-prem k8s stack).
- Competitor docs as cited inline.
- CNCF Conformance Program — `cncf.io/certification/software-conformance/`.
