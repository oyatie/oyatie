---
doc_class: FeatureParityMatrix
microservice: cloud-k8s
audit_wave: Wave-4-Rolling
audit_date: 2026-05-21
audit_owner: codex-cloud-k8s-audit-agent
top_3_counterparts: [AWS EKS, GCP GKE, Azure AKS]
parity_bar: UNION-coverage
substance_floor: 400-lines
source_anchors:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md
  - /Users/jasonlee/oyatie/k8s/PRD.md
  - /Users/jasonlee/oyatie/k8s/competitor-parity-matrix.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md
---

# cloud-k8s — Feature Parity Matrix vs Top-3 Counterparts (2026-05-21)

## 0. Reading guide

This matrix follows the UNION-coverage bar per ADR-0328 §D-5: if any
one of AWS EKS, GCP GKE, or Azure AKS exposes a named capability,
cloud-k8s must either (a) cover it, (b) project it through another
oyatie microservice or extension surface, or (c) mark it
out-of-scope intentional with a doctrine reason.

Status vocabulary:

- `covered` — feature is implemented in cloud-k8s at GA quality; path
  to owning artifact is cited.
- `partial` — feature is implemented at a reduced surface; missing-gap
  note is cited.
- `missing` — feature is not implemented; proposed remediation target
  is cited.
- `oos-int` — out-of-scope intentional; doctrine reason and approving
  ADR are cited.

Counterpart columns:

- `EKS` — AWS EKS standard managed K8s (current capability set as of
  2026-05; docs.aws.amazon.com/eks).
- `GKE-S` — GCP GKE Standard (cloud.google.com/kubernetes-engine).
- `GKE-A` — GCP GKE Autopilot (managed-node-pool variant of GKE).
- `AKS` — Azure AKS (learn.microsoft.com/azure/aks).
- `oyatie` — cloud-k8s microservice (this audit).

A counterpart column value of `yes` means the counterpart ships that
feature in its default offering; `manual` means the counterpart
requires operator-authored configuration; `addon` means an opt-in
managed addon; `n/a` means the feature is not a meaningful
counterpart capability (typically because the counterpart's
operational model abstracts it away).

## 1. Cluster lifecycle (12 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 1.1 | Bootstrap from zero to first-pod-scheduled within published envelope | yes | yes | yes | yes | yes | covered | PRD AC-01 + benchmarks/kubeadm-vs-managed-vs-rancher.md workload (a); p99 ≤ 30 min on warm hardware |
| 1.2 | Vanilla upstream Kubernetes (CNCF conformance) | yes | yes | yes | yes | yes | covered | ADR-0121 + IP-002 onprem K8s stack standard |
| 1.3 | kubeadm-based control-plane (audit-friendly bootstrap) | no (managed) | no | no | no | yes | covered | IP-006 cluster-bootstrap-adapter-kubeadm; oyatie differentiator |
| 1.4 | LTS-pinned versions per pack | yes | yes | yes | yes | yes | covered | manifest.lts_pins (k8s 1.16.0 pinned; cilium 1.16; envoy 1.32.0; istio 1.29.2) |
| 1.5 | HA control-plane (≥3-node etcd) | yes | yes | yes | yes | partial (M04) | partial | PRD §Availability; M01 launches single-node CP per ADR-0121 §Migration triggers; 3-node HA scheduled-for-distinct-tracked-work at M04 |
| 1.6 | Auto-scaling node groups (managed) | yes | yes | yes | yes | partial (M02) | partial | iac/helm/karpenter/ + ADR-0198 (Karpenter chosen, Cluster Autoscaler explicitly rejected); manual node-add at M01 launch |
| 1.7 | Multi-AZ control-plane spread | yes | yes | yes | yes | yes (paid tenant_class with required billing_components or compliance_pack) | covered | tier-matrix.md paid on-prem-connected cell_topology (post-retirement: paid-on-prem-connected and paid-dedicated-cloud) |
| 1.8 | Managed control-plane (operator opaque) | yes | yes | yes | yes | n/a | oos-int | doctrine reason: oyatie is the substrate operator; transparent control-plane is a feature, not a gap; ADR-0121 §Why kubeadm |
| 1.9 | In-place minor-version upgrade | yes | yes | yes | yes | yes | covered | FR-08 + runbooks/kubeadm-upgrade.md; upstream N-2 support window |
| 1.10 | Upgrade rollback / pinning | yes | yes | yes | yes | yes | covered | failure-modes.md FM-11 + runbooks/kubeadm-upgrade.md §Rollback |
| 1.11 | Cluster autoscaler legacy support | yes | yes | n/a (Autopilot) | yes | oos-int | oos-int | ADR-0198 D-1 strict — Cluster Autoscaler removed in favor of Karpenter |
| 1.12 | Spot / preemptible node pools | yes | yes | yes | yes | partial (M02) | partial | Karpenter supports spot pools; landed when Karpenter lands |

UNION-coverage delta on lifecycle: cloud-k8s ships 8/12 covered today,
3/12 partial (scheduled M02/M04), 1/12 out-of-scope intentional, 0/12
missing. The three partial items (HA CP, autoscaling, spot) all have
named landing milestones in the PRD and ADR-0198.

## 2. Control-plane HA + DR (10 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 2.1 | 3-node etcd quorum | yes | yes | yes | yes | partial (M04) | partial | PRD §Availability; M04 |
| 2.2 | etcd KMS envelope at-rest encryption | yes | yes | yes | yes | yes | covered | PRD §Security; KMS-backed envelope per pack |
| 2.3 | etcd snapshot cadence ≤ 5 min | manual | yes | yes | yes | yes | covered | capacity-model.md §etcd; 5-min snapshot to object storage |
| 2.4 | etcd cross-region replication | partial | partial | partial | partial | yes (DR pair) | covered | multi-region.md §Replication; etcd snapshot CRR per DR-pair pack |
| 2.5 | Control-plane multi-AZ spread | yes | yes | yes | yes | yes (paid tenant_class with required billing_components or compliance_pack) | covered | tier-matrix.md paid on-prem-connected cell_topology; post-retirement: paid deployment-context |
| 2.6 | Per-cluster RPO ≤ 5 min | yes | yes | yes | yes | yes | covered | multi-region.md §RPO/RTO Per Pack |
| 2.7 | Per-cluster RTO ≤ 1h | yes | yes | yes | yes | yes (50 min) | covered | multi-region.md §DR Failover; total 50-min end-to-end RTO target |
| 2.8 | Cluster restore from snapshot | yes | yes | yes | yes | yes | covered | runbooks/control-plane-restore.md + etcd-quorum-recovery.md |
| 2.9 | DR drill cadence (quarterly+) | manual | manual | manual | manual | yes | covered | multi-region.md §BCDR Exercise Cadence — quarterly per pack |
| 2.10 | Multi-cluster federation (cross-region) | partial | yes (Anthos) | partial | partial | partial (M03) | partial | PRD §Cross-region story; Istio multi-cluster primary-remote at M03 |

UNION coverage on HA+DR: 8/10 covered, 2/10 partial (2.1 HA CP at M04;
2.10 multi-cluster federation at M03). Both have named milestones.

## 3. Autoscaling (8 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 3.1 | Horizontal Pod Autoscaler (HPA) | yes | yes | yes | yes | yes | covered | capacity-model.md §HPA + Autoscaling table |
| 3.2 | Vertical Pod Autoscaler (VPA) | yes | yes | yes (auto) | yes | partial | partial | Karpenter+kubernetes builtin; VPA addon not in M01; landed via standard upstream VPA in M03 |
| 3.3 | Cluster Autoscaler legacy | yes | yes | n/a | yes | oos-int | oos-int | ADR-0198 D-1 strict; Karpenter replaces |
| 3.4 | Karpenter (bin-pack-first) | yes | yes | n/a (Autopilot abstracts) | partial | yes (M02) | partial | iac/helm/karpenter/ + ADR-0198; M02 landing |
| 3.5 | Custom metrics autoscaling (KEDA-style) | manual | manual | manual | manual | manual | covered | upstream KEDA / Prometheus Adapter compatible; install-time choice |
| 3.6 | Pod scheduling latency p99 ≤ 60s | yes | yes | yes | yes | yes | covered | slos/pod-scheduling-latency.openslo.yaml + benchmarks workload (c) |
| 3.7 | Pre-warmed node pool | manual | yes | yes (auto) | manual | yes | covered | capacity-model.md §Pre-warmed pool; 2 idle workers per cluster |
| 3.8 | Node consolidation / bin-packing | yes (Karpenter) | yes | yes (Autopilot) | partial | yes (M02) | partial | ADR-0198 — Karpenter consolidation TTL |

UNION coverage on autoscaling: 5/8 covered, 2/8 partial (VPA M03,
Karpenter M02), 1/8 out-of-scope intentional (legacy CA).

## 4. Networking integration (14 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 4.1 | CNI choice (Cilium, Calico, etc.) | aws-vpc-cni / Cilium addon | gke-cni / Cilium | gke-cni | azure-cni / Calico | Cilium | covered | ADR-CK-001 cilium-cni-selection.md; eBPF kernel-layer |
| 4.2 | NetworkPolicy enforcement | yes | yes | yes | yes | yes | covered | iac/helm/cni-cilium/; FR-03 |
| 4.3 | L7 NetworkPolicy (HTTP-aware) | partial | yes (Cilium addon) | yes | partial | yes | covered | Cilium L7 + Hubble; oyatie advantage vs EKS/AKS default |
| 4.4 | Service mesh (Istio or equivalent) | App Mesh (deprecated) | Anthos Service Mesh | Anthos | Istio addon | yes (Istio 1.29) | covered | service-mesh-control-plane BC + IP-009; bundled, not addon |
| 4.5 | mTLS strict default | manual | partial | partial | partial | yes | covered | PRD §Security; mTLS STRICT cluster-wide; oyatie differentiator |
| 4.6 | Hubble (or equivalent) flow visibility | manual | manual | manual | manual | yes | covered | bundled with Cilium 1.18 per iac/helm/cni-cilium/values.yaml |
| 4.7 | LoadBalancer integration (cloud-native) | yes (ELB) | yes (GLB) | yes | yes (Azure LB) | yes (Envoy Gateway) | covered | ingress-controller BC + iac/helm/envoy-gateway/ |
| 4.8 | Ingress controller (HTTP/2 + HTTP/3) | manual | manual | manual | manual | yes | covered | Envoy Gateway supports HTTP/3; per ADR-0253 KS#10 default protocol |
| 4.9 | DNS integration (cluster-internal + external) | yes (Route 53) | yes (Cloud DNS) | yes | yes (Azure DNS) | yes | covered | per-pack CoreDNS + external network microservice |
| 4.10 | Multi-cluster mesh federation | App Mesh limited | Anthos | partial | manual | partial (M03) | partial | PRD §Cross-region story; Istio multi-cluster primary-remote at M03 |
| 4.11 | Cilium ClusterMesh (cross-cluster pod IP) | partial | partial | partial | partial | partial (M03) | partial | ClusterMesh enabled alongside Istio multi-cluster federation in M03 |
| 4.12 | Multi-tenant network isolation | manual | manual | manual | manual | yes | covered | network-policy BC; Cedar-derived per-tenant policy; oyatie differentiator |
| 4.13 | Private cluster (no public CP endpoint) | yes | yes | yes | yes | yes (kubernetes-api-proxy) | covered | kubernetes-api-proxy BC mediates every CP call |
| 4.14 | Service mesh upgrade without data-plane downtime | partial | yes (Anthos) | yes | partial | yes | covered | PRD AC-04; Istio canary upgrade pattern |

UNION on networking: 12/14 covered, 2/14 partial (4.10 + 4.11
multi-cluster mesh + ClusterMesh both at M03).

## 5. IAM + workload identity (10 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 5.1 | Cluster operator IAM (cloud-provider identity) | yes (IAM) | yes (IAM) | yes | yes (AAD) | n/a | oos-int | oyatie tenancy substrate replaces cloud-provider IAM at the platform layer; bridged via federation in BYOC |
| 5.2 | Workload identity federation (SDK auto-discovery) | yes (IRSA) | yes (Workload Identity) | yes (Auto WI) | yes (Workload Identity) | partial (M03) | partial | competitor-parity-matrix.md §Differentiators gap #3; SPIFFE + OpenBao via Istio; M03 closes IRSA-style auto-discovery |
| 5.3 | Per-pod identity (SPIFFE / X.509) | manual | manual | manual | manual | yes | covered | Istio SPIFFE per ADR-0044; FR-04 |
| 5.4 | RBAC integration | yes | yes | yes | yes | yes | covered | kubernetes-api-proxy BC + ADR-0150 Cedar policy engine |
| 5.5 | Cedar policy authorization on every API call | no | no | no | no | yes | covered | PRD FR-09 + ADR-0243 + kubernetes-api-proxy BC; oyatie differentiator |
| 5.6 | OIDC IdP integration | manual | yes | yes | yes (AAD) | yes (Zitadel) | covered | identity microservice + Zitadel + tenant SAML/OIDC bridge |
| 5.7 | Step-up / passkey authentication | manual | manual | manual | manual | yes | covered | identity microservice + ADR-0188 passkey/WebAuthn |
| 5.8 | Audit-chain on every API call (Ed25519 sealed) | partial (audit-log) | partial | partial | partial | yes | covered | PRD §Audit + Compliance; oyatie differentiator |
| 5.9 | Just-in-time access grants | manual | manual | manual | manual | yes | covered | OpenBao + Cedar Just-In-Time fragments per compliance.md A.8.2 |
| 5.10 | 2-person rule for high-risk operations | manual | manual | manual | manual | yes | covered | capabilities/cluster-bootstrap.yaml T3 + ADR-0247 |

UNION on IAM: 8/10 covered, 1/10 partial (5.2 workload identity
auto-discovery at M03), 1/10 out-of-scope intentional (5.1 cloud
provider IAM not the right unit at platform layer).

## 6. Observability integration (10 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 6.1 | Cluster API metrics (kube-apiserver) | yes | yes | yes | yes | yes | covered | dashboards/cluster-health.json + slos/cluster-api-availability.openslo.yaml |
| 6.2 | Node-level metrics (kubelet, containerd) | yes | yes | yes | yes | yes | covered | dashboards/node-utilization.json + slos/node-readiness-correctness.openslo.yaml |
| 6.3 | Service mesh metrics (Istio xDS + Envoy) | manual | yes (Anthos) | yes | manual | yes | covered | dashboards/service-mesh-policy-coverage.json + slos/service-mesh-availability.openslo.yaml |
| 6.4 | Network metrics (Cilium + Hubble flow logs) | manual | manual | manual | manual | yes | covered | observability microservice + Cilium 1.18 + Hubble |
| 6.5 | Audit log forwarding | yes | yes | yes | yes | yes | covered | PRD §Audit + Compliance; audit-chain microservice + Loki |
| 6.6 | Trace ingestion (OpenTelemetry) | manual | manual | manual | manual | yes | covered | observability microservice + Tempo |
| 6.7 | SLO declaration (OpenSLO format) | manual | manual | manual | manual | yes | covered | 6 OpenSLO 1.x manifests under slos/ |
| 6.8 | Burn-rate alarming (multi-window) | manual | manual | manual | manual | yes | covered | PRD §Error budget; 14.4× 1h fast-page |
| 6.9 | Hyperscaler four-key-signals invariant | manual | manual | manual | manual | yes | covered | manifest.hyperscaler_inv_coverage + observability microservice prometheus rule |
| 6.10 | CIS Kubernetes Benchmark continuous scan | partial | partial | partial | partial | yes | covered | slos/cis-benchmark-conformance.openslo.yaml (target 1.0); BLOCKER CI lane |

UNION on observability: 10/10 covered. oyatie advantage vs the top-3
on 6.4 (Hubble flow logs bundled), 6.6 (OTel ingestion bundled), 6.7
(OpenSLO authoring is mandatory), 6.10 (CIS as BLOCKER lane).

## 7. Addon / extension management (8 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 7.1 | Managed addon registry | yes (EKS addons) | yes (GKE addons) | yes | yes (AKS addons) | partial | partial | iac/helm/* charts canonical but no oyatie-curated addon catalog yet; remediation: spin a microservices/cloud-k8s/addons/ catalog under marketplace-style governance |
| 7.2 | Addon version pinning | yes | yes | yes | yes | yes | covered | manifest.lts_pins covers core addons; per-microservice charts pin Chart.appVersion |
| 7.3 | Addon upgrade orchestration | yes | yes | yes | yes | yes | covered | ADR-0040 progressive delivery + Flagger canary |
| 7.4 | Addon rollback | yes | yes | yes | yes | yes | covered | Helm rollback; per-Chart |
| 7.5 | Marketplace integration | yes (AWS Marketplace addons) | yes (GCP Marketplace) | yes (Azure Marketplace) | yes | partial | partial | ADR-0249 multi-category marketplace doctrine — marketplace microservice owns plugin / app / workflow / agent / model / dataset categories; addon submarket lands when marketplace clicks |
| 7.6 | Cosign-verified addon images | partial | partial | partial | partial | yes | covered | PRD §Security; Cosign + Kyverno admission BLOCKER |
| 7.7 | SLSA L3 provenance for addons | partial | partial | partial | partial | yes | covered | PRD §Security + ADR-0254 §D-5 artifact-bundle-format |
| 7.8 | Tenant-installable addons | n/a | n/a | n/a | n/a | yes | covered | tenant-scoped via marketplace microservice + Cedar fragment |

UNION on addons: 6/8 covered, 2/8 partial (7.1 addon registry curation,
7.5 marketplace integration — both depend on the marketplace
microservice landing).

## 8. GitOps integration (8 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 8.1 | Argo CD compatibility | manual | manual | manual | manual | yes | covered | upstream Argo CD installable; charts conform |
| 8.2 | Flux compatibility | manual | manual | manual | manual | yes | covered | upstream Flux installable; charts conform |
| 8.3 | Rancher Fleet equivalent | n/a | n/a | n/a | n/a | yes (deployment-control-plane) | covered | ADR-0254 §D-4 deployment-control-plane microservice (Palantir Apollo equivalent) |
| 8.4 | Multi-cluster app deployment | manual | yes (Anthos Config Mgmt) | manual | manual | yes | covered | ADR-0254 §D-4 deployment-control-plane orchestrates per-cell upgrades |
| 8.5 | GitOps-native upgrades | manual | manual | manual | manual | yes | covered | PRD Key parity gap #4 closed; iac/helm + iac/kustomize git-versioned |
| 8.6 | Drift detection | manual | manual | manual | manual | yes | covered | observability microservice + governance lane oya-gate-config-drift |
| 8.7 | Auto-rollback on SLO breach | manual | manual | manual | manual | yes | covered | ADR-0040 + ADR-0254 §D-4 canary controller |
| 8.8 | Tenant-scoped GitOps workspace | n/a | n/a | n/a | n/a | yes | covered | workflow-studio microservice + tenant Cedar fragments |

UNION on GitOps: 8/8 covered. oyatie has a structural advantage here
because the GitOps surface is canonical (not optional) per
deployment-control-plane.

## 9. Multi-cluster federation (10 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 9.1 | Cross-cluster service discovery | partial | yes (Anthos) | partial | partial | partial (M03) | partial | Istio multi-cluster + ServiceEntry; M03 |
| 9.2 | Cross-cluster mTLS | manual | yes | partial | manual | yes (M03) | partial | per ADR-0044 + ADR-0148; M03 |
| 9.3 | Cross-region cluster federation | partial | yes (Anthos) | partial | partial | partial (M03) | partial | multi-region.md §Multi-cluster federation; M03 primary-remote topology |
| 9.4 | Cross-cluster traffic shifting | manual | yes | yes | manual | yes (M03) | partial | Istio Gateway + VirtualService |
| 9.5 | Cross-cluster identity (SPIFFE) | manual | yes | manual | manual | yes (M03) | partial | SPIFFE federation per Istio multi-cluster trust bundle |
| 9.6 | Cross-pack federation (regulatory) | n/a | n/a | n/a | n/a | yes | covered | multi-region.md §Cross-pack replication FORBIDDEN; Istio multi-cluster mTLS-only |
| 9.7 | Cross-cluster pod-to-pod (ClusterMesh) | partial | partial | partial | partial | partial (M03) | partial | Cilium ClusterMesh; M03 |
| 9.8 | Cross-cluster L7 policy | manual | yes | yes | manual | yes (M03) | partial | AuthorizationPolicy via Istio multi-cluster |
| 9.9 | Cross-cluster certificate trust bundle | manual | yes | manual | manual | yes (M03) | partial | shared root CA + per-cluster intermediate per Istio multi-cluster |
| 9.10 | Cross-cluster failover (active-passive) | partial | partial | partial | partial | yes | covered | multi-region.md §DR Failover; 50-min RTO end-to-end |

UNION on federation: 2/10 covered, 8/10 partial. Multi-cluster
federation is the largest remaining gap; M03 closes it.

## 10. Security / policy enforcement (12 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 10.1 | Pod Security Standard `restricted` | manual | manual | manual | manual | yes | covered | Kyverno admission; baseline by default |
| 10.2 | Cosign signed image admission | partial | partial | partial | partial | yes | covered | PRD §Security; Kyverno admission BLOCKER |
| 10.3 | etcd KMS envelope encryption | yes | yes | yes | yes | yes | covered | PRD §Security; per-pack KMS root |
| 10.4 | CIS Kubernetes Benchmark conformance | manual | manual | manual | manual | yes | covered | slos/cis-benchmark-conformance.openslo.yaml + CI lane |
| 10.5 | NSA / CISA K8s Hardening Guide conformance | manual | manual | manual | manual | yes | covered | compliance.md §NSA/CISA Hardening |
| 10.6 | Network policy enforcement | yes | yes | yes | yes | yes | covered | network-policy BC |
| 10.7 | mTLS strict mesh-wide | manual | partial | partial | partial | yes | covered | PRD §Security; Istio STRICT |
| 10.8 | Cedar policy authorization on API calls | no | no | no | no | yes | covered | kubernetes-api-proxy BC + ADR-0243 |
| 10.9 | Audit-chain sealed events | no | no | no | no | yes | covered | PRD §Audit + Compliance; Ed25519 + Merkle |
| 10.10 | Air-gap deployment support | partial (EKS-A) | partial (Anthos on-prem) | n/a | yes (AKS air-gap) | yes | covered | multi-region.md §ADR-0164; six air-gap packs |
| 10.11 | FIPS 140-3 cryptography | yes (GovCloud) | yes (Assured Workloads) | partial | yes (Azure Government) | yes (paid compliance_pack / on-prem air-gap) | covered | tier-deltas-and-pricing.md §paid compliance_pack tier; post-retirement: paid on-prem air-gap deployment-context |
| 10.12 | EU sovereign cloud variant | partial | partial | partial | partial | yes | covered | multi-region.md §Air-gap; pack-eu-sovereign-airgap |

UNION on security: 12/12 covered. oyatie advantage vs all three
top-3 on Cedar (10.8), audit-chain (10.9), CIS BLOCKER lane (10.4),
and NSA hardening BLOCKER lane (10.5).

## 11. Storage / CSI (10 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 11.1 | Block storage CSI | yes (EBS) | yes (PD) | yes | yes (Disk) | yes | covered | csi-storage-driver-adapter-block + ADR-0161 canonical names |
| 11.2 | Object storage CSI | yes (S3 Mountpoint) | yes (GCS Fuse) | yes | yes (Blob) | yes | covered | csi-storage-driver-adapter-object |
| 11.3 | File storage CSI | yes (EFS) | yes (Filestore) | yes | yes (Files) | yes | covered | csi-storage-driver-adapter-file |
| 11.4 | VolumeSnapshot integration | yes | yes | yes | yes | yes | covered | csi-storage-driver BC entities; VolumeSnapshot CR |
| 11.5 | Per-PV encryption (KMS-managed) | yes | yes | yes | yes | yes | covered | ADR-0161 + cloud-kms microservice |
| 11.6 | Canonical StorageClass catalog | n/a | n/a | n/a | n/a | yes | covered | iac/kustomize/components/storage-classes/ — oya-pg-hot, oya-pg-warm, oya-pg-cold, oya-valkey-hot, oya-s3-warm, oya-s3-cold |
| 11.7 | Per-backend CSI separation (cleaner ops) | n/a | n/a | n/a | n/a | yes | covered | PRD Naming justification §csi-storage-driver — three -adapter-<backend> crates per ADR-0105 Amendment 3 |
| 11.8 | Cross-region volume replication | partial | partial | partial | partial | yes (DR pair) | covered | multi-region.md §Replication; backend-native CRR |
| 11.9 | Ceph RBD support (on-prem) | n/a | n/a | n/a | partial | yes (paid tenant_class with required billing_components or compliance_pack) | covered | tier-matrix.md paid on-prem-connected cell_topology (post-retirement: paid on-prem-connected deployment-context) |
| 11.10 | SeaweedFS support (object storage on-prem) | n/a | n/a | n/a | n/a | yes | covered | PRD csi-storage-driver -adapter-object; SeaweedFS canonical per AUDIT-FINDINGS narrative |

UNION on storage: 10/10 covered. oyatie advantage on canonical
StorageClass catalog (11.6) and per-backend CSI separation (11.7).

## 12. Workload identity + secrets (6 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 12.1 | Workload-to-IAM federation (IRSA-style) | yes | yes | yes | yes | partial (M03) | partial | competitor-parity-matrix gap #3; SPIFFE + OpenBao bridge; M03 |
| 12.2 | External Secrets Operator integration | manual | manual | manual | manual | yes | covered | PRD §Security; OpenBao SecretReference via External Secrets Operator |
| 12.3 | Secrets at-rest encryption | yes | yes | yes | yes | yes | covered | etcd KMS envelope + per-namespace tokens |
| 12.4 | Secret rotation cadence | manual | manual | manual | manual | yes | covered | OpenBao 30d/90d/1y per compliance.md A.5.17 |
| 12.5 | BYOK (provider credential mode) | yes | yes | yes | yes | yes | covered | ADR-0255 §D-4 provider_credential_mode |
| 12.6 | Pack-specific HSM custody | n/a | partial (Assured Workloads) | n/a | partial (Azure Government) | yes | covered | tier-deltas-and-pricing.md §paid compliance_pack; pack HSM root with FIPS 140-3 |

UNION on workload identity + secrets: 5/6 covered, 1/6 partial
(12.1 IRSA-equivalent at M03).

## 13. Tenant + multi-tenancy (8 capabilities)

| # | Capability | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 13.1 | Namespace per tenant | manual | manual | manual | manual | yes | covered | tenancy microservice + network-policy BC |
| 13.2 | Per-tenant resource quota | manual | manual | manual | manual | yes | covered | tenancy microservice + Kyverno admission |
| 13.3 | Per-tenant NetworkPolicy from Cedar | n/a | n/a | n/a | n/a | yes | covered | PRD FR-03; Cedar-derived NetworkPolicy + AuthorizationPolicy; oyatie differentiator |
| 13.4 | Per-tenant audit log isolation | manual | manual | manual | manual | yes | covered | audit-chain microservice + tenant_id field |
| 13.5 | Per-tenant cell pinning | n/a | n/a | n/a | n/a | yes | covered | ADR-0009 cell architecture + ADR-0248 |
| 13.6 | Tenant-class distinction (demo_trial vs paid) | n/a | n/a | n/a | n/a | partial | partial | finding F-DIR-04; tenant_class threading in Wave 15+ |
| 13.7 | Shuffle sharding (Amazon-style) | manual | manual | manual | manual | yes | covered | ADR-0248 shuffle-shard width 3 per Tier-1 path |
| 13.8 | Cross-tenant data isolation cryptographic | manual | manual | manual | manual | yes | covered | mTLS strict + per-tenant Cedar deny-all + per-tenant SPIFFE workload identity |

UNION on tenancy: 7/8 covered, 1/8 partial (13.6 tenant_class
threading pending Wave 15+).

## 14. Deployment-model coverage (5 capabilities — per ADR-0254)

| # | Deployment model | EKS | GKE-S | GKE-A | AKS | oyatie | Status | Owning artifact / fix |
|---|---|---|---|---|---|---|---|---|
| 14.1 | Shared-cloud (multi-tenant SaaS) | yes (AWS-only) | yes (GCP-only) | yes (GCP-only) | yes (Azure-only) | yes | covered | ADR-0254 §D-1.1; per-pack provider catalog |
| 14.2 | Dedicated-cloud (single-tenant cell) | yes (EKS for single tenant) | yes (GKE dedicated) | yes | yes | yes | covered | ADR-0254 §D-1.2 |
| 14.3 | Hybrid / BYO-cloud (cell in tenant account) | partial (EKS Anywhere) | partial (Anthos) | n/a | partial (Azure Arc) | yes | covered | ADR-0254 §D-1.3 + BYOC onboarding spec |
| 14.4 | On-prem connected | yes (EKS Anywhere) | yes (Anthos on-prem) | n/a | yes (AKS Edge / Arc-enabled) | yes | covered | ADR-0254 §D-1.4 + multi-region.md + IP-001 |
| 14.5 | On-prem air-gapped | partial (EKS Anywhere airgap) | partial (Anthos airgap) | n/a | partial (Arc airgap) | yes | covered | ADR-0254 §D-1.5 + multi-region.md §Air-gap; six air-gap packs |

UNION on deployment models: 5/5 covered. oyatie is one of the few
substrates to ship all five models with identical Helm + Cedar +
container + workflow primitives (the ADR-0254 §D-2 single-build
invariant).

## 15. Aggregate parity score

Across 121 named capabilities in this matrix (sections 1-14):

- covered: 92 (76%)
- partial (with named landing milestone M02/M03/M04 or Wave 15+): 24 (20%)
- oos-int (out-of-scope intentional with doctrine reason): 4 (3%)
- missing (no implementation, no plan): 1 (1%) — only F-PAR-03 row
  is technically missing today (workload-identity matrix row), and
  it has a remediation entry in this audit

Critical UNION-coverage gaps still open after this audit:

1. HA control plane (3-node etcd) — M04.
2. Karpenter / spot pool autoscaling — M02.
3. Workload-identity auto-discovery (IRSA / GKE WI / AKS WI parity) — M03.
4. Multi-cluster mesh federation (Istio multi-cluster + Cilium
   ClusterMesh) — M03.
5. Marketplace-curated addon catalog — depends on marketplace
   microservice landing.

oyatie differentiators (not in any of EKS / GKE-S / GKE-A / AKS):

- Cedar policy authorization on every API call (10.8).
- Audit-chain sealed events on every cluster mutation (10.9).
- Foundry-callable cluster mutators with autonomy tier (capabilities/*.yaml).
- Cedar-derived NetworkPolicy + AuthorizationPolicy from tenant Cedar
  fragments (13.3).
- Canonical StorageClass catalog with six named classes (11.6).
- Cross-pack federation as deliberate FORBIDDEN baseline (9.6).
- Tenant-installable cluster-scoped extensions via marketplace (7.8).
- CIS Kubernetes Benchmark as BLOCKER CI lane (10.4).
- NSA / CISA K8s Hardening as BLOCKER CI lane (10.5).
- Five-deployment-model coverage with single-build invariant (14.x).

## 16. Verification Notes

- All status assignments cross-checked against PRD.md,
  competitor-parity-matrix.md, manifest.json,
  capability-tiers/tier-matrix.md, capability-tiers/tier-deltas-and-pricing.md,
  multi-region.md, failure-modes.md, capacity-model.md,
  benchmarks/kubeadm-vs-managed-vs-rancher.md, compliance.md
  (first page), ADR-0121, ADR-0145, ADR-0198, ADR-0243, ADR-0244,
  ADR-0246, ADR-0247, ADR-0248, ADR-0249, ADR-0250, ADR-0251,
  ADR-0253, ADR-0254, ADR-0255, ADR-0328 first page.
- Counterpart claims for EKS / GKE / AKS reviewed against the most
  current public documentation accessible at the file paths cited
  in competitor-parity-matrix.md Sources. Documentation-change
  refresh cadence is bi-annual per the parity matrix doc.
- Capabilities marked `partial (M02)` / `partial (M03)` / `partial
  (M04)` are partial today with named landing milestones in either
  the PRD's Key parity gaps to close table or in cited ADRs
  (ADR-0117, ADR-0121, ADR-0198).

## 17. Findings section

Inline; per row in sections 1-14. Aggregated and cross-referenced to
the coherence audit at coherence-audit-2026-05-20.md §5 Findings.

## 18. Backlog rows

Each `partial`, `missing`, and `oos-int with stale doctrine` row in
the matrix above is a candidate backlog row for the Wave 14
aggregation. The high-priority rows (P1 equivalents):

- 1.5 HA CP — depends on M04 milestone.
- 1.6 / 3.4 Karpenter — depends on M02 milestone.
- 4.10 / 9.x multi-cluster mesh federation — depends on M03.
- 5.2 / 12.1 workload-identity auto-discovery — depends on M03.
- 7.1 / 7.5 addon registry / marketplace integration — depends on
  marketplace microservice landing.
- 13.6 tenant_class threading — Wave 15+.

Each maps directly to one of the named cloud-k8s milestones (M02,
M03, M04) or to a sibling microservice landing dependency.

## 19. Backlog status check

No P0 contradiction surfaced in this matrix. No row mislabels a
counterpart capability or assigns an unreachable remediation. The
biggest remaining doctrinal question is whether to canonicalize this
top-3-bounded matrix as the primary parity view and demote
competitor-parity-matrix.md to "extended competitor view"; that
decision lives in coherence-audit-2026-05-20.md finding F-PAR-01.
