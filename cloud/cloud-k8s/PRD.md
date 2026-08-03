---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-cloud-k8s
microservice: cloud-k8s
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0117
  - ADR-0120
  - ADR-0121
  - ADR-0139
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json, /specs/hyperscaler-gates.json]
date: 2026-05-17
owner_team: axis-cloud
doc_status: published
---

# PRD-cloud-k8s: On-prem Kubernetes Cluster Substrate

## Purpose

The `cloud-k8s` microservice is oyatie's **on-prem Kubernetes substrate** — the layer that owns vanilla kubeadm clusters, the containerd runtime, the Istio service mesh control plane, the Envoy data plane, and the CNI / CRI / CSI integrations that every other oyatie µservice runs on top of. Per ADR-0121, the stack is **vanilla upstream Kubernetes 1.35 LTS + containerd 2.3.0 LTS + Istio 1.29.2 + Envoy (Istio-bundled) + CSI drivers per storage backend**; no Rancher / k3s / k0s on the primary cell. Per ADR-0131 (Cloud split), `cloud-k8s` is the sibling to `cloud-iac` (which lays down the underlying network + compute + storage on bare-metal / OCI) — `cloud-iac` makes the box; `cloud-k8s` turns it into a Kubernetes cluster.

This µservice is **shared substrate**, not a hero product. It hosts every other oyatie µservice. Its existence is the precondition for ADR-0117's cloud-native progression and ADR-0139's gate-driven promotion (the SLO engine needs a cluster to schedule on). It is consumed by `cell` (tenant cell-scheduling), `cloud-iac` (applies bootstrap manifests against it), `observability` (deploys Grafana stack onto it), and every workload µservice that schedules pods.

This µservice has no Bominal equivalent and originates in oyatie per the 2026-05-16 on-prem cell directive.

## Tenant Value

Tenants do not consume `cloud-k8s` directly. Tenant value is **indirect, structural, and load-bearing**:

- **Tenant Outcome 1 — Sovereignty-grade isolation.** Per-pack physical cluster boundary (one `cloud-k8s` cluster per regional pack); cross-pack mTLS only via Istio multi-cluster federation; no tenant data routes through a foreign jurisdiction's API server.
- **Tenant Outcome 2 — Operational durability.** Cluster bootstrap completes within 30 min (vs hours for hand-rolled clusters); node-join p99 ≤ 5 min; service-mesh policy propagates p99 ≤ 30 s. Tenants experience consistent latency floors.
- **Tenant Outcome 3 — Audit-grade provenance.** Every cluster mutation (node-join, network-policy-apply, Istio-policy-change, kubeadm upgrade) emits an Ed25519-signed audit-chain record per Bominal ADR-0028; auditors get a sealed, provable, point-in-time cluster topology.
- **Internal Outcome 4 — Hyperscaler-parity substrate.** Same control-plane code that AWS EKS / GCP GKE / Azure AKS / Oracle OKE / Rancher RKE2 / OpenShift / Tanzu Kubernetes Grid run; CNCF conformance by construction; no Rancher-specific bits to unwind at multi-cluster federation.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | cluster operator (cloud-iac) | to bootstrap a kubeadm-control-plane onto a fresh node fleet via a single Foundry capability invocation | a pack cluster comes up in ≤ 30 min without manual SSH | cluster-bootstrap | Must |
| FR-02 | node-lifecycle controller | to add / cordon / drain / remove worker nodes idempotently | node-fleet auto-scales without operator toil | node-lifecycle | Must |
| FR-03 | tenant-scheduling integration (cell µservice) | to apply per-tenant Cedar-derived NetworkPolicy + AuthorizationPolicy to a tenant's namespace | tenants cannot cross-talk inside or across pack clusters | network-policy | Must |
| FR-04 | Istio control-plane operator | to install / upgrade Istio's control plane (istiod) without service-mesh data-plane downtime | services keep mTLS during the upgrade | service-mesh-control-plane | Must |
| FR-05 | Envoy ingress operator | to apply VirtualService / Gateway / DestinationRule for any namespace | tenant-facing HTTPS routes exist with TLS termination + per-route policy | ingress-controller | Must |
| FR-06 | workload µservice owner | to provision a PVC via a per-backend CSI driver (block-volume / object / file) | stateful workloads have durable storage with declared QoS class | csi-storage-driver | Must |
| FR-07 | Foundry agent | to call `cloud-k8s.cluster.bootstrap`, `cloud-k8s.node.add`, `cloud-k8s.networkpolicy.apply` capabilities under autonomy-ceiling | clusters are agent-operated, audit-chain-emitting, and reversible | cluster-bootstrap + node-lifecycle + network-policy | Must |
| FR-08 | kubeadm-upgrade controller | to upgrade Kubernetes minor versions (N → N+1) within the upstream N-2 support window | clusters never drop out of CNCF-supported window | cluster-bootstrap | Must |
| FR-09 | Kubernetes-API proxy | to mediate every kubectl / API call (operator + agent) with Cedar-policy authorization + audit-chain emission | direct kube-apiserver access is impossible; every call is policy-checked | kubernetes-api-proxy | Must |
| FR-10 | aggregation index | to regenerate `docs/prds/INDEX.md`, `registry/catalog/<crate>.yaml` union, and machine-readable views from per-microservice sources | central indices are never hand-edited; per-microservice folders are source of truth | (cross-cutting) | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Cluster bootstrap (kubeadm init → ready, single control-plane node) | ≤ 10 min | ≤ 30 min | ≤ 45 min | per ADR-0121 §"Setup time"; matches OKE bootstrap envelope |
| Node-join (kubeadm join → Ready) | ≤ 2 min | ≤ 5 min | ≤ 10 min | per upstream kubeadm benchmarks |
| Node cordon + drain (graceful) | ≤ 2 min | ≤ 5 min | — | per-pod TerminationGracePeriodSeconds bounded ≤ 60 s |
| NetworkPolicy / AuthorizationPolicy propagation (Cilium / Istio) | ≤ 5 s | ≤ 30 s | ≤ 60 s | per Istio xDS publish + Cilium cilium-agent xfer |
| Istio control-plane upgrade (canary istiod) | ≤ 10 min | ≤ 30 min | — | zero data-plane downtime |
| Envoy config propagation (VirtualService → all sidecars) | ≤ 2 s | ≤ 10 s | ≤ 30 s | per Istio xDS benchmark |
| CSI volume provision (block-volume) | ≤ 5 s | ≤ 30 s | ≤ 60 s | per OCI Block Volume + Ceph RBD benchmark |
| kubeadm minor-version upgrade (whole cluster) | ≤ 30 min | ≤ 90 min | — | per upstream guidance + Istio rolling upgrade overlap |

### Security

- mTLS strict mode by default across every namespace per ADR-0044 (scheduled-for-distinct-tracked-work from M02b-substrate-P22; landed on-prem via ADR-0121); permissive mode allowed only during a documented Istio rollout window.
- All kube-apiserver access is mediated by the `kubernetes-api-proxy` BC (no direct port-6443 from operator workstations); Cedar policy + audit-chain on every call.
- etcd encrypted at rest with KMS-backed envelope encryption per pack; etcd peer + client mTLS strict.
- containerd config: no privileged-by-default; seccomp `runtime/default` baseline; AppArmor profiles per workload class.
- Supply-chain: every container image consumed by control-plane components verified via Cosign signatures per ADR-0117 §"Supply chain"; admission controller refuses unsigned images.
- Secrets: OpenBao SecretReference materialised via the External Secrets Operator + per-namespace ServiceAccount-bound tokens (no raw secrets in YAML).

### Audit + Compliance

- Every `ClusterBootstrapped`, `NodeJoined`, `NodeDrained`, `NetworkPolicyApplied`, `IstioPolicyChanged`, `KubeadmUpgraded` event emits an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028).
- Audit-chain seal latency ≤ 1 s per event.
- CIS Kubernetes Benchmark v1.9 compliance verified continuously by `oya-check-cis-k8s-benchmark` CI lane.
- Kubernetes API audit log: forwarded to `audit-chain` µservice; retention ≥ 6 y for pack-us-healthcare (HIPAA §164.316(b)(2)), ≥ 5 y for pack-kr (KR commercial code), ≥ 2 y default.

### Availability + SLO

- Control-plane availability target: 99.99 % monthly per pack (3-node HA control-plane after M04 multi-node HA per ADR-0121 §"Migration triggers"; single control-plane at M01 launch with documented downgrade to 99.9 %).
- Data-plane availability target: 99.99 % monthly (Envoy sidecars survive control-plane outage by config).
- RTO: ≤ 30 min for control-plane restore (etcd snapshot + kubeadm init from snapshot). RPO: ≤ 5 min (etcd snapshot cadence).

### Data residency

- Cluster components, etcd state, and PV objects pinned per pack jurisdiction per ADR-0117. No cross-pack PV replication. Multi-cluster Istio federation strictly mTLS-only (no payload replication).

### DR posture (ADR-0343)

- Declared target: RTO <= 1800 seconds and RPO <= 300 seconds, matching the current Availability + SLO section. `manifest.json` currently lacks a `dr` block, so D-2 backfill must mirror these values.
- Applicable floors: HIPAA-2024 (3600/300, multi-region), SOC2-T2 (14400/900), ISO27001-2022 (14400/3600), KR-CSAP-v3.1 (3600/900, multi-region), KR-PIPA-2023-amendment (14400/900), and EU-AI-ACT-2024-HIGH-RISK when the cluster hosts Annex-III workloads (1800/300, multi-region). Effective strictness is RTO 1800 seconds and RPO 300 seconds.
- Failover runbook reference: `runbooks/control-plane-restore.md` for kubeadm/etcd restore, `runbooks/etcd-quorum-recovery.md` for quorum loss, and `runbooks/kubeadm-upgrade.md` for upgrade recovery.
- multi_region_active_active posture: false for a single etcd writer/control plane; true at the pack/cell layer only through separate clusters and Istio multi-cluster federation, with no cross-pack PV replication.
- WHY: tenant workloads need bounded control-plane recovery while preserving Kubernetes' single-cluster consistency model and per-pack jurisdiction boundaries.

### Capacity model (ADR-0340)

- Per-tenant baseline: D-2 has not populated `capacity_model`; the current PRD uses substrate envelope values instead: cluster bootstrap <= 30 minutes, node join p99 <= 5 minutes, 10 baseline nodes for launch clusters, and up to 5,000 nodes per cluster once Karpenter NodePools are active.
- Scaling dimension: `per_capability` for cluster bootstrap, node lifecycle, network policy, service mesh, ingress, CSI, and kubernetes-api-proxy; the Karpenter IP adds workload-class NodePools (`oya-app`, `oya-batch`, `oya-gpu`, `oya-regulatory`) as the runtime scaling primitive.
- Cell placement class: Tier-2, matching the manifest's `criticality_tier: T2`, because cloud-k8s is shared substrate with tenant workload placement responsibility but not the canonical commercial, identity, or key ledger.
- Autoscaling boundaries: controller HA starts at two replicas for Karpenter, control plane targets three nodes after HA promotion, NodePools expand by workload class, and regulatory NodePools remain sovereign-region pinned/on-demand only.
- WHY: capacity tracks cluster and node churn, not application request rate; the model keeps pack clusters available for every µservice while preserving regulatory placement constraints.

### Sustainability + cost attribution (ADR-0344)

- Every `ClusterBootstrapped`, `NodeJoined`, `NodeDrained`, `NetworkPolicyApplied`, `IstioPolicyChanged`, `KubeadmUpgraded`, and kubernetes-api-proxy audit row also emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the tenant/product/capability/provider/cell/compliance_pack axes.
- Provider routing is carbon-aware for non-urgent node placement, batch NodePool scale-out, and planned control-plane maintenance. It is not carbon-routed for HIPAA emergency mode, PCI realtime-fraud dependency recovery, control-plane restore, or regulatory NodePool region pins.
- Tenant transparency surface: finops-portal node/workload-class cost view, OpenCost labels such as `oya.io/workload-class`, and per-pack infrastructure allocation reports.
- WHY: CSRD, SB-253, SEC climate-disclosure, and FinOps reporting require cluster substrate costs and energy to be explainable by tenant, workload class, cell, and compliance pack.

### API versioning posture (ADR-0342)

- Public API version model: cluster lifecycle, node lifecycle, network-policy, CSI, ingress, and kubernetes-api-proxy contracts use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and proto3 version field.
- SDK semver model: operator and automation SDKs use major.minor.patch, with major bumps reserved for generated type or supported carrier breaks.
- Support window: last N=3 external control-plane API versions are supported for at least 180 days.
- Per-tenant pinning: supported for paid/regulatory tenants during cluster maintenance and audit windows; demo_trial follows the platform default.
- Internal-mesh exemption: yes; in-cluster gRPC and Kubernetes watch paths preserve ADR-0145 direct semantics while the API proxy enforces version carriers on external/operator boundaries.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), layers used by this µservice are: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-kubeadm`, `adapter-containerd`, `adapter-istio`, `adapter-envoy`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `cluster-bootstrap` | `oya-cloud-k8s-cluster-bootstrap-{kernel,domain,usecase,api,adapter,adapter-kubeadm,rest,worker,sdk,app}` | kubeadm init + join orchestration; cluster lifecycle; minor-version upgrade | `Cluster`, `ControlPlaneNode`, `KubeadmConfig`, `EtcdSnapshot`, `BootstrapEvidence` |
| `node-lifecycle` | `oya-cloud-k8s-node-lifecycle-{kernel,domain,usecase,api,adapter,rest,worker,app}` | node add / cordon / drain / remove; node health attestation; node taint+toleration management | `Node`, `NodeRole`, `NodeAttestation`, `CordonReason`, `DrainPlan` |
| `network-policy` | `oya-cloud-k8s-network-policy-{kernel,domain,usecase,api,adapter,rest,worker,app}` | Cedar-derived NetworkPolicy + AuthorizationPolicy emission; per-tenant namespace policy ; cross-pack federation policy | `NetworkPolicy`, `AuthorizationPolicy`, `PeerSelector`, `TenantNamespace` |
| `service-mesh-control-plane` | `oya-cloud-k8s-service-mesh-control-plane-{kernel,domain,usecase,api,adapter,adapter-istio,rest,worker,app}` | Istio control-plane install / upgrade / configuration; istiod lifecycle; multi-cluster mesh federation | `IstioRevision`, `MeshConfig`, `Telemetry`, `ProxyConfig`, `MultiClusterPeer` |
| `ingress-controller` | `oya-cloud-k8s-ingress-controller-{kernel,domain,usecase,api,adapter,adapter-envoy,rest,worker,app}` | Envoy Gateway / VirtualService / DestinationRule / TLS termination; public ingress; SNI routing | `Gateway`, `VirtualService`, `DestinationRule`, `TlsCertificate`, `SniRoute` |
| `csi-storage-driver` | `oya-cloud-k8s-csi-storage-driver-{kernel,domain,usecase,api,adapter,adapter-block,adapter-object,adapter-file,rest,worker,app}` | per-backend CSI driver shims (block-volume, object, file); per-pack PV provisioning; QoS-class enforcement | `StorageClass`, `PersistentVolume`, `PersistentVolumeClaim`, `VolumeSnapshot`, `CsiBackend` |
| `kubernetes-api-proxy` | `oya-cloud-k8s-kubernetes-api-proxy-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Cedar-policy + audit-chain wrapper around kube-apiserver; operator + agent + CI access gateway | `ApiCall`, `CallerPrincipal`, `PolicyDecision`, `AuditRecord` |

Naming justification — `cluster-bootstrap`:

```
NAME: oya-cloud-k8s-cluster-bootstrap-<layer>
JUSTIFICATION:
- microservice = cloud-k8s: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice folder.
- bc-tokens = cluster-bootstrap: primary BC; cluster lifecycle (init → join → upgrade → reset).
  Sibling BCs (node-lifecycle, network-policy, service-mesh-control-plane, ingress-controller,
  csi-storage-driver, kubernetes-api-proxy) justify explicit BC token per ADR-0056 v4.1
  BC-optionality rule.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + sealed-trait + entity types (Cluster, ControlPlaneNode, KubeadmConfig,
    EtcdSnapshot, BootstrapEvidence). Zero I/O. data_class annotated.
  - domain: pure cluster-version arithmetic, kubeadm-version compatibility checks, etcd-snapshot
    integrity computation.
  - usecase (per ADR-0106; replaces legacy 'application'): orchestrators wrapping kubeadm
    init / join / upgrade via ports.
  - api: protocol-neutral typed I/O contracts (cluster.create / node.add / upgrade.plan).
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-kubeadm: backend-qualified adapter (per ADR-0105 Amendment 3 *-adapter-<backend>
    pattern); shells out to kubeadm CLI; reads /etc/kubernetes/*.
  - rest: HTTP handler/route layer; OpenAPI-defined REST surface.
  - worker: long-lived bootstrap-watcher; emits ClusterBootstrapped events.
  - sdk: client library (Rust + future TS / Py / Go).
  - app: composition root binary.
- exemptions claimed: none.
```

Naming justification — `service-mesh-control-plane`:

```
NAME: oya-cloud-k8s-service-mesh-control-plane-<layer>
JUSTIFICATION:
- microservice = cloud-k8s.
- bc-tokens = service-mesh-control-plane: explicit BC token; sibling BC ingress-controller
  is the data-plane peer; this BC is the control-plane (istiod, MeshConfig, multi-cluster).
- layer = <layer>: 9 layers per ADR-0105 +
  - adapter-istio: backend-qualified adapter; shells out to istioctl + reads/writes IstioOperator CR.
- exemptions claimed: none.
```

Naming justification — `csi-storage-driver`:

```
NAME: oya-cloud-k8s-csi-storage-driver-<layer>
JUSTIFICATION:
- microservice = cloud-k8s.
- bc-tokens = csi-storage-driver.
- layer = <layer>: includes THREE backend-qualified adapters per ADR-0105 Amendment 3:
  - adapter-block (OCI Block Volume + Ceph RBD)
  - adapter-object (OCI Object Storage + SeaweedFS)
  - adapter-file (OCI File Storage + CephFS)
  Each is a distinct *-adapter-<backend> crate; no exception required.
```

Naming justifications for the remaining BCs follow the same pattern (microservice = cloud-k8s; bc-tokens explicit; layers per ADR-0105; no exemptions claimed).

Layer mapping per BC (13-layer canonical enum; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-* | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|
| `cluster-bootstrap` | ✅ | ✅ | ✅ | ✅ | ✅ | `-adapter-kubeadm` | ✅ | ✅ | ✅ | ✅ |
| `node-lifecycle` | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ | ✅ | — | ✅ |
| `network-policy` | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ | ✅ | — | ✅ |
| `service-mesh-control-plane` | ✅ | ✅ | ✅ | ✅ | ✅ | `-adapter-istio` | ✅ | ✅ | — | ✅ |
| `ingress-controller` | ✅ | ✅ | ✅ | ✅ | ✅ | `-adapter-envoy` | ✅ | ✅ | — | ✅ |
| `csi-storage-driver` | ✅ | ✅ | ✅ | ✅ | ✅ | `-adapter-block`, `-adapter-object`, `-adapter-file` | ✅ | ✅ | — | ✅ |
| `kubernetes-api-proxy` | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ | ✅ | ✅ | ✅ |

Total crates introduced by this µservice: **62** (10 cluster-bootstrap + 8 node-lifecycle + 8 network-policy + 9 service-mesh-control-plane + 9 ingress-controller + 11 csi-storage-driver + 10 kubernetes-api-proxy − 3 sdk omissions for non-tenant-facing BCs = 62).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `KubeadmCommander` | `oya-cloud-k8s-cluster-bootstrap-kernel` | `-adapter-kubeadm` (shell-out to kubeadm CLI with bounded scope) | `AUDIT` (kubeadm config + node-join token) |
| `EtcdSnapshotter` | `oya-cloud-k8s-cluster-bootstrap-kernel` | `-adapter` (etcdctl wrapper) | `AUDIT` (etcd state snapshot) |
| `NodeRegistry` | `oya-cloud-k8s-node-lifecycle-kernel` | `-adapter` (kube-apiserver client; bounded RBAC) | `INTERNAL_ONLY` (node names + labels) |
| `NodeDrainer` | `oya-cloud-k8s-node-lifecycle-kernel` | `-adapter` (eviction API; PDB-aware) | `INTERNAL_ONLY` |
| `NetworkPolicyEmitter` | `oya-cloud-k8s-network-policy-kernel` | `-adapter` (kube-apiserver client) | `BEHAVIORAL_TENANT_PRODUCT` (tenant-namespace identifier) |
| `AuthorizationPolicyEmitter` | `oya-cloud-k8s-network-policy-kernel` | `-adapter` (Istio AuthorizationPolicy CR write) | `BEHAVIORAL_TENANT_PRODUCT` |
| `IstioCommander` | `oya-cloud-k8s-service-mesh-control-plane-kernel` | `-adapter-istio` (istioctl + IstioOperator CR) | `INTERNAL_ONLY` |
| `EnvoyConfigurer` | `oya-cloud-k8s-ingress-controller-kernel` | `-adapter-envoy` (Gateway / VirtualService CR) | `INTERNAL_ONLY` |
| `CsiProvisioner` | `oya-cloud-k8s-csi-storage-driver-kernel` | `-adapter-block`, `-adapter-object`, `-adapter-file` | `BEHAVIORAL_TENANT_PRODUCT` (PVC owner) |
| `ApiCallMediator` | `oya-cloud-k8s-kubernetes-api-proxy-kernel` | `-adapter` (HTTP reverse-proxy with Cedar + audit-chain) | `AUDIT` (every API call) |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `cloud-k8s` MUST NOT import any other product µservice crate at any layer. Workload events flow through Workflow events (`ClusterBootstrapped`, `NodeJoined`, `NetworkPolicyApplied`, `IstioPolicyChanged`); reads happen through Ontology (`Cluster`, `Node`, `NetworkPolicy` object types). `cloud-iac` and `cell` are explicit sibling cloud-* µservices and exchange data only through Workflow + Ontology. LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice cloud-k8s` — dependency-direction
- `oya gate validate lean-a2 --microservice cloud-k8s` — cross-product-refusal
- `oya gate validate port-location --microservice cloud-k8s` — ports in kernel
- `oya gate validate layer-correctness --microservice cloud-k8s` — layer enum match
- `oya gate validate per-microservice-layout --microservice cloud-k8s` — ADR-0131 conformance
- `oya gate validate statelessness --microservice cloud-k8s`
- `oya gate validate shardability --microservice cloud-k8s`
- `oya gate validate cis-k8s-benchmark --microservice cloud-k8s` — CIS Kubernetes Benchmark v1.9

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `ClusterBootstrapped` | kubeadm-init completes + first node Ready | `cell`, `observability`, `audit-chain`, `cloud-iac` | cluster-state-machine per `/specs/cloud-k8s-cluster-state.json` |
| `NodeJoined` | kubeadm-join succeeds + Node enters Ready | `cell` (capacity allocation), `observability` (node SLI), `audit-chain` | — |
| `NodeFailed` | node-lifecycle worker detects NotReady for > 5 min OR attestation fails | `cell` (re-schedule pods), `observability`, `audit-chain` | — |
| `NodeDrained` | drain completes; node ready for removal | `cell`, `audit-chain` | — |
| `NetworkPolicyApplied` | NetworkPolicy / AuthorizationPolicy CR write succeeds | `audit-chain`, `observability` | — |
| `IstioPolicyChanged` | IstioOperator / Telemetry / MeshConfig CR change | `audit-chain`, `observability`, `workflow-engine` (for downstream policy re-eval) | — |
| `KubeadmUpgraded` | minor-version upgrade completes; control-plane + worker nodes upgraded | `audit-chain`, `observability` | — |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `IacResourcePlanned` | `cloud-iac` (OpenTofu plan accepted) | `cluster-bootstrap` | discover the new node fleet; pre-stage kubeadm join token; emit `ReadyForBootstrap` |
| `TenantOnboarded` | `tenancy` | `network-policy` | derive Cedar policy fragment → emit per-tenant NetworkPolicy + AuthorizationPolicy in the tenant's namespace |
| `CellProvisionRequested` | `cell` | `cluster-bootstrap` + `node-lifecycle` | allocate the right pack cluster + place the cell on the right worker nodes |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Cluster{pack, region, version, control_plane_nodes, status}` | `cluster_for→Pack` | `cluster-bootstrap` | Ed25519 |
| `Node{cluster_id, role, az, kernel, runtime, status}` | `node_in→Cluster` | `node-lifecycle` | Ed25519 |
| `NetworkPolicy{namespace, peer_selector, ports, applied_at}` | `policy_for→Namespace` | `network-policy` | Ed25519 |
| `IstioRevision{cluster_id, revision, status}` | `istio_for→Cluster` | `service-mesh-control-plane` | Ed25519 |
| `Gateway{cluster_id, hosts, tls_secret_ref}` | `gateway_for→Cluster` | `ingress-controller` | Ed25519 |
| `StorageClass{cluster_id, backend, qos_class, reclaim_policy}` | `class_for→Cluster` | `csi-storage-driver` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Pack` (catalog) | `cluster-bootstrap` | `filter(active=true)` to enumerate packs requiring a cluster |
| `Tenant{tenant_id, pack, scope}` | `network-policy` | `filter(pack=<this_pack>)` to enumerate tenants requiring namespace policy |
| `Microservice{name, pack_assignment}` | `cluster-bootstrap` | `filter(pack=<this_pack>)` to inform sizing |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| AWS EKS | Managed Kubernetes | control-plane HA, node-group lifecycle, IRSA for IAM, CNI flexibility, Calico/Cilium support | `docs.aws.amazon.com/eks/` |
| GCP GKE | Managed Kubernetes | control-plane HA, autopilot, Workload Identity, mesh add-on | `cloud.google.com/kubernetes-engine/docs/` |
| Azure AKS | Managed Kubernetes | control-plane HA, AAD integration, Azure CNI, Istio add-on | `learn.microsoft.com/azure/aks/` |
| Oracle OKE | Managed Kubernetes | KR-Seoul + global regions; native OCI integration | `docs.oracle.com/en-us/iaas/Content/ContEng/` |
| Rancher RKE2 | On-prem Kubernetes distro | hardened defaults; air-gapped install | `docs.rke2.io` |
| OpenShift | On-prem Kubernetes distro | OpenShift-specific operators; Red Hat support | `docs.openshift.com` |
| Tanzu Kubernetes Grid | On-prem Kubernetes distro | VMware-friendly; cluster-API based | `docs.vmware.com/en/VMware-Tanzu-Kubernetes-Grid/` |

Key parity gaps to close (ordered by priority):

1. **Multi-cluster mesh** — EKS / GKE / AKS all have managed Istio or Anthos Service Mesh; oyatie ships Istio on day 1 but multi-cluster federation lands in M03 per ADR-0117.
2. **Auto-scaling node groups** — managed cloud providers offer Karpenter (canonical per ADR-0198) and the legacy Cluster Autoscaler; oyatie ships manual node-add at M01 launch and Karpenter integration in M02 (Cluster Autoscaler is explicitly rejected per ADR-0198 — bin-pack-first node provisioning + heterogeneous instance-type selection are required).
3. **Workload identity (IRSA / GKE WI / AKS WI)** — workload-to-IAM federation; oyatie integrates with OpenBao + SPIFFE per ADR-0028 but is not yet hyperscaler-parity for AWS / GCP / Azure SDK auto-discovery.
4. **GitOps-native** — Argo CD / Flux integration is canonical in M01; matches Rancher Fleet + OpenShift GitOps.

Key oyatie differentiators (NOT in any competitor):

1. **Foundry-callable cluster mutators** — every cluster operation is a Foundry capability with autonomy ceiling + audit-chain seal; no competitor exposes this shape.
2. **Cedar-derived NetworkPolicy / AuthorizationPolicy** — tenant Cedar policy → NetworkPolicy CR + Istio AuthorizationPolicy CR; competitors require operators to hand-author CRs.
3. **Kubernetes-API proxy with Cedar + audit-chain** — every kubectl call gets Cedar-authorized + audit-chain-emitted; competitors expose raw kube-apiserver (with RBAC + audit-log only).

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Cluster bootstrap | ≤ 10 min | ≤ 30 min | ≤ 45 min | per ADR-0121 |
| Node-join | ≤ 2 min | ≤ 5 min | ≤ 10 min | — |
| Node drain | ≤ 2 min | ≤ 5 min | — | — |
| NetworkPolicy propagation | ≤ 5 s | ≤ 30 s | ≤ 60 s | — |
| Istio control-plane upgrade | ≤ 10 min | ≤ 30 min | — | zero data-plane downtime |
| CSI volume provision | ≤ 5 s | ≤ 30 s | ≤ 60 s | — |
| API-proxy decision latency (Cedar + audit emit) | ≤ 10 ms | ≤ 50 ms | ≤ 200 ms | per kube-apiserver call |

Error budget:
- Monthly error budget for control-plane availability: 0.01 % (≈ 4.3 min/month) at 99.99 % target.
- Burn-rate alarms on Mimir aggregations (`oya_cloud_k8s_control_plane_*`) with 14.4× burn / 1h fast-page.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `stateless | postgres | object-storage | persistent-volume | mixed` → **`mixed`**. Rationale: control-plane is etcd (persistent-volume; 3-node Raft after M04 HA); worker components are stateless; CSI provisioners delegate state to backend (block-volume / object / file); api-proxy is stateless; istiod is stateless (config via CR).

**Active-active compatibility**: control plane uses etcd Raft (active-passive within cluster, active-active across packs via Istio multi-cluster federation in M03). Worker components stateless-compatible.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Nodes per cluster | 10 | 5000 | per upstream kubeadm support; CNCF tested at 5k nodes |
| Pods per node | 110 | 250 | per kubelet default; pack-us-healthcare lowered to 50 for isolation |
| Pods per cluster | 1100 | 150 000 | per upstream |
| NetworkPolicies per namespace | 10 | 1000 | Cilium scale envelope |
| CSI volumes per cluster | 100 | 50 000 | per backend |
| Istio sidecars per cluster | 1100 | 150 000 | per upstream; xDS scales linearly |

Scale-out policy:
- Per-pack cluster boundary: one cluster per pack; multi-cluster federation via Istio multi-cluster install at M03.
- Karpenter (M02 per ADR-0198): scales worker nodes on `unschedulable_pods > 0` with bin-pack-first heterogeneous-instance selection. Cluster Autoscaler is explicitly rejected (taint-based ASG model does not meet hyperscaler bin-pack latency target).
- Pre-warmed pool: 2 idle worker nodes per cluster; cold-start budget ≤ 5 min (full node-join).

Cross-region story:
- M01 launch: single KR cluster on the on-prem cell per ADR-0121.
- M03: per ADR-0117, OCI OKE clusters per pack join via Istio multi-cluster federation.
- Multi-cluster federation uses Istio's primary-remote topology for the on-prem cell + OCI peers.

Sharding:
- Cluster identity sharded by `pack`; intra-cluster sharding by `namespace` (tenant ownership).
- `oya-check-shardability-cli` CI lane verifies partition key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | `kubeadm init` completes against a fresh node fleet in ≤ 30 min p99 | timed e2e drill under `microservices/cloud-k8s/tests/e2e/cluster-bootstrap.rs` |
| AC-02 | A `kubeadm join` completes in ≤ 5 min p99 against an active cluster | timed e2e drill |
| AC-03 | NetworkPolicy / AuthorizationPolicy propagates to all Cilium / Istio agents within ≤ 30 s p99 of CR write | xDS-publish latency probe |
| AC-04 | Istio control-plane canary upgrade (`istioctl upgrade`) completes with zero data-plane downtime | e2e drill with synthetic mTLS traffic during upgrade |
| AC-05 | Cosign-unsigned image is refused at admission | admission-webhook integration test |
| AC-06 | Direct kube-apiserver access (port 6443) is refused; kubernetes-api-proxy is the only path | NetworkPolicy + e2e probe |
| AC-07 | `oya-check-cis-k8s-benchmark` lane passes against bootstrapped cluster | LEAN lane exit 0 |
| AC-08 | All IaC manifests (Helm + OpenTofu + Kustomize) deploy clean against a kind cluster | CI lane `oya-cloud-k8s-iac-smoke` |
| AC-09 | `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice cloud-k8s` exit 0 | ADR-0131 lane |
| AC-10 | `cargo run -p oya-dev-cli -- gate validate authority-cohesion` exit 0 | ADR-0123 lane; HG-CLOUD-K8S registered |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | M01 control-plane HA: single node (faster) vs 3-node (resilient)? | axis-cloud + ops-sre-reliability | Resolved: single-node M01 launch per ADR-0121 §"Migration triggers"; 3-node at M04. |
| 2 | CNI choice: Calico vs Cilium for primary | axis-cloud | Resolved: Cilium per ADR-0117 §"CNI" (eBPF, NetworkPolicy + Hubble + multi-cluster mesh primitives). |
| 3 | CSI driver split: per-backend crates vs single multiplexed adapter | axis-cloud | Resolved: per-backend adapter (`-adapter-block`, `-adapter-object`, `-adapter-file`) per ADR-0105 Amendment 3. |
| 4 | Kubernetes-API proxy: HTTP-reverse-proxy vs in-process Cedar+apiserver fork | axis-cloud + ops-security | Resolved: HTTP-reverse-proxy at M01; in-process variant scheduled-for-distinct-tracked-work to M04. |
| 5 | Multi-cluster mesh federation: M02 or M03? | axis-cloud + ops-sre-reliability | M03 per ADR-0117. |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application → usecase rename | usecase naming |
| ADR-0117 | Cloud-native infrastructure progression | high-level cloud strategy |
| ADR-0120 | Rust-first on-prem tooling | tooling discipline |
| ADR-0121 | On-prem k8s stack: kubeadm + containerd + Istio + Envoy | this PRD's substrate decision |
| ADR-0139 | Agentic SLO-gated promotion | gating consumer of cluster |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0132 | Product platform + bundle dissolution | no-grouping forward policy |
| ADR-0133 | Industry-best-practice conformance program | CIS / NSA / NIST framework conformance |
| ADR-0123 | Hyperscaler maturity claim gate | HG-CLOUD-K8S registers here |
| ADR-0116 | Retire external agent-coordination tooling | oya vcs primitives used throughout |

## ADR-0164 Update — Sovereign Cloud / Air-Gapped Deployment Variant

Per ADR-0164 (2026-05-18), the cloud-k8s µservice ships a per-pack air-gap variant for sovereign packs. See `multi-region.md` for the full statement.

Highlights:
- **In-cell container registry**: Harbor 2.x at `registry.{cell}.svc.cluster.local`. Image references rewritten via kustomize component. Sigstore + Kyverno admission verification.
- **No external API egress**: NetworkPolicy + Cilium L7 egress deny all external hosts. DNS via in-cell CoreDNS, NTP via in-cell chrony, OCSP/CRL via in-cell PKI.
- **Telemetry confinement**: external observability SaaS (Datadog, Honeycomb, New Relic) forbidden in air-gap mode; only in-cell observability backend permitted.
- **In-region CI runner option**: per-pack overlay points deploy pipeline at in-region self-hosted runners.

Per-pack overlays at `iac/kustomize/components/air-gap-{ksa,kr-fsc,kr-public,uae,eu-sovereign-airgap,us-gov}/` flip the air-gap mode. Pre-flight mirror job templated at `microservices/cloud-iac/iac/helm/harbor-mirror/`.

CI lane `oya gate validate air-gap-overlay` enforces sovereign-pack containment.

## ADR-0161 Update — Canonical StorageClass Catalog

Per ADR-0161 (2026-05-18), this µservice ships the canonical StorageClass catalog at `iac/kustomize/components/storage-classes/` and the per-pack overlay surface at `iac/kustomize/components/pack-{name}/`.

Canonical names workload µservices reference: `oya-pg-hot`, `oya-pg-warm`, `oya-pg-cold`, `oya-valkey-hot`, `oya-s3-warm`, `oya-s3-cold`. Per-pack overlay binds each canonical name to a concrete CSI driver per the matrix in `/specs/csi-storage-class-canonical.json`.

CI lane `oya gate validate storage-class-canonical` enforces (a) every workload µservice chart references only canonical names, (b) every active pack populates the full matrix.

## ADR-0158 Update — Active-Passive Disposition

Per ADR-0158 (2026-05-18), the cloud-k8s µservice's cluster control-plane is declared `active_passive` per cell. See `multi-region.md` for the full disposition statement.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is legacy/local-feedback provenance only after ADR-0515; protected merge authority is `oya-ci-required`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, while ArgoCD remains separately authorized CD evidence with cosign, tenant namespace, and audit-chain controls.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `cloud-k8s` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `cloud-k8s` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 8 module pin(s) across 4 context(s).
- Scaling input: `per_capability` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
