---
doc_class: CapacityModel
title: Capacity Sizing Model — Nodes × Pods × Pack
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-cloud
deciders: ops-sre-reliability, axis-cloud, council-architecture
related_adrs: [ADR-0117, ADR-0121, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-k8s/cost-budget.md
  - microservices/cloud-k8s/multi-region.md
  - microservices/cloud-k8s/policy/cluster-isolation.md (per-tenant limits)
  - microservices/cloud-k8s/PRD.md §"Horizontal Scalability"
review_cadence: quarterly + on every cluster-component-replica-set change
doc_status: published
---

# Capacity Sizing Model (cloud-k8s µservice)

## Purpose

Sizing formulas + reference baselines for every cluster component (kube-apiserver / kube-controller-manager / kube-scheduler / etcd / kubelet / containerd / Cilium / Istio / Envoy / CSI / Kyverno / kubernetes-api-proxy) per pack, per scale tier. Drives `cost-budget.md` and `multi-region.md`. Numbers cite CNCF reference + upstream Kubernetes / Istio / Cilium published guidance.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants in pack | `N_tenants` | OpenBao tenant-resolver |
| Workload µservices per pack | `M_microservices` | tenant deployments per pack |
| Pods per µservice avg | `P_pods_per_ms` | from each µservice's capacity-model |
| Active sidecars (mesh-enrolled pods) | `S_sidecars` | usually ~ pod count × 1.0 |
| API call rate (mutations) | `Q_api_mutate` | per kube-apiserver bench |
| API call rate (reads) | `Q_api_read` | per kube-apiserver bench |
| Cluster nodes (worker) | `W_nodes` | derived from pod requests |

## Cluster Component Sizing

### kube-apiserver

```
total_pods             = N_tenants × M_microservices × P_pods_per_ms
worker_nodes_needed    = ceil(total_pods / 100)  // 100 pods/node sustainable; CIS recommends 110 max
api_request_rate       = total_pods × 0.5 reqs/sec  // kubelet heartbeat + sidecar xDS + workload API
```

kube-apiserver replicas (after M04 HA; 1 at M01):

```
kube_apiserver_replicas = max(3, ceil(api_request_rate / 5000))
```

References: Kubernetes scaling guide (`kubernetes.io/docs/setup/best-practices/cluster-large/`); upstream conformance tested at 5 000 nodes + 150 000 pods.

### etcd

```
etcd_replicas = 3 (after M04 HA) or 1 (M01)
etcd_disk_iops = max(1000, total_pods × 0.1)  // per Kubernetes etcd-perf guide
etcd_disk_size = max(8 GB, total_pods × 0.001 GB)  // historical; well-bounded
etcd_memory   = max(8 GB, total_pods × 0.001 GB)
```

References: etcd hardware recommendations (`etcd.io/docs/v3.5/op-guide/hardware/`).

### kube-controller-manager + kube-scheduler

```
controller_manager_replicas = same as kube-apiserver (HA pair)
scheduler_replicas          = same as kube-apiserver
```

### kubelet + containerd (per worker node)

```
kubelet_cpu_reservation   = 100m + (pods_per_node × 10m)  // per kubelet sizing
kubelet_memory_reservation = 256MB + (pods_per_node × 4MB)
containerd_cpu_reservation = 50m  // baseline
containerd_memory_reservation = 128MB
```

References: kubelet sizing (`kubernetes.io/docs/concepts/configuration/manage-resources-containers/`).

### Cilium

```
cilium_agent_replicas    = 1 per worker node (DaemonSet)
cilium_operator_replicas = 2 (HA)
cilium_agent_cpu         = 100m baseline + (endpoints_on_node × 1m)
cilium_agent_memory      = 256MB baseline + (endpoints_on_node × 1MB)
```

References: Cilium scale (`docs.cilium.io/en/stable/operations/performance/`).

### Istio control plane (istiod)

```
istiod_replicas = max(3, ceil(S_sidecars / 1000))
istiod_cpu      = 1 core per replica
istiod_memory   = 1 GB per replica + 1 KB per sidecar
```

References: Istio scaling (`istio.io/latest/docs/ops/best-practices/scaling/`).

### Istio data plane (Envoy sidecars + ingress gateway)

```
envoy_sidecar_cpu    = 50m baseline (per pod)
envoy_sidecar_memory = 128MB baseline (per pod)
ingress_gateway_replicas = max(2, ceil(public_rps / 5000))
ingress_gateway_cpu  = 500m per replica baseline
```

### CSI provisioners

```
csi_block_volume_controller_replicas  = 2 (HA)
csi_object_controller_replicas        = 2 (HA)
csi_file_controller_replicas          = 2 (HA)
csi_node_plugin_replicas              = 1 per worker node (DaemonSet)
```

### Kyverno admission webhook

```
kyverno_admission_replicas = max(3, ceil(api_request_rate / 1000))
kyverno_background_replicas = 2
```

### kubernetes-api-proxy

```
api_proxy_replicas = max(3, ceil(api_request_rate / 2000))
api_proxy_cpu      = 500m per replica
api_proxy_memory   = 512MB per replica
```

## Reference-architecture baselines

| Scale tier | N_tenants | M_microservices | total_pods | worker_nodes | kube-apiserver replicas | etcd | istiod | Cilium agent (DaemonSet) | api-proxy |
|---|---|---|---|---|---|---|---|---|---|
| **XS** (M01 launch; pack-kr; ~20 tenants × ~36 µservices) | 20 | 36 | ~1 500 (avg 2 pods/ms) | 20 | 1 (M01) / 3 (HA) | 1 (M01) / 3 (HA) | 3 | 20 | 3 |
| **S** (~100 tenants × 50 µservices) | 100 | 50 | ~10 000 | 120 | 3 | 3 | 6 | 120 | 5 |
| **M** (~1k tenants × 60 µservices) | 1000 | 60 | ~100 000 | 1200 | 6 | 3 | 50 | 1200 | 20 |
| **L** (~10k tenants × 80 µservices) | 10000 | 80 | ~1 000 000 (multi-cluster necessary) | 5000 (per cluster) × N clusters | 9 per cluster | 3 per cluster | 200 per cluster | 5000 per cluster | 50 per cluster |

At L tier, single-cluster cap is ~150 000 pods (upstream conformance limit); multi-cluster federation per pack splits load. Per-pack-region multiplier: each pack has its own cluster sized at active-tenants-in-pack tier. DR-pair packs add 0.6× warm-standby.

## Per-pack DR-pair sizing

For DR-pair packs (pack-eu / pack-us / pack-au / pack-in / pack-br / pack-ae / pack-ksa):
- Primary cluster: 1.0× of pack's active capacity.
- DR-pair cluster: 0.6× warm-standby (snapshot-restore in ≤ 1h; HPA scales to 1.0× post-failover within 30 min).

## Pack-us-healthcare overlay

- Extended retention (≥ 6y audit log) bumps etcd snapshot cold-tier storage cost.
- Lower pods/node (50 instead of 110) for stronger isolation.
- Worker node multiplier: 1.4× of equivalent non-HC pack to handle the lower density.

## HPA + Autoscaling

| Component | HPA on | Min replicas | Max replicas | Cooldown |
|---|---|---|---|---|
| kube-apiserver | n/a (kubeadm fixed; HA after M04) | 1 (M01) / 3 (HA) | 6 | – |
| api-proxy | CPU > 70% OR `queue_depth > 50` | 3 | 50 | 60s |
| istiod | CPU > 70% | 3 | 20 | 60s |
| ingress-gateway | CPU > 70% OR connection-count | 2 | 100 | 60s |
| kyverno-admission | CPU > 70% | 3 | 20 | 30s |
| CSI controllers | CPU > 70% | 2 | 6 | 60s |
| Karpenter (M02-onward per ADR-0198) | `unschedulable_pods > 0` | min nodes set per pack | max nodes per pack | 30s consolidation TTL; bin-pack-first |

## Pre-warmed pool

- 2 idle worker nodes per cluster (cold-start budget ≤ 5 min from `kubeadm join` to `Ready`).
- 2 standby api-proxy replicas (warm).
- 2 standby istiod replicas (warm).

## Storage Costs (per pack region)

### etcd persistent volume (block storage)

- Size: 16 GB (XS) → 64 GB (M) → 256 GB (L); RPO-driven snapshot to object storage every 5 min.
- OCI Block Volume pricing: ~$0.0255/GB/month (balanced perf class).

### etcd snapshot storage (object)

- 14d hot + 90d cold per `policy/data-residency.md`.
- XS tier: ~3 GB/snapshot × 288 snapshots/day = 864 GB/day raw → ~13 TB/14d hot + ~80 TB/90d cold.

### Audit log storage (Loki)

- Per-pack: ~5 GB/day per medium-tenant (XS pack: ~100 GB/day).
- Retention 6y (pack-us-healthcare); 5y (pack-kr); 2y default.

### Container image registry (Harbor; cloud-iac provides)

- Per-pack mirror: ~2 TB hot for active images.

## Worked example: oyatie XS tier (M01 launch; pack-kr; 20 tenants)

```
total_pods           = 20 × 36 × 2 = 1 440
worker_nodes_needed  = ceil(1440 / 100) = 15 (with 2 pre-warm = 17)
api_request_rate     = 1440 × 0.5 = 720 reqs/sec
kube_apiserver_replicas (M01) = 1
etcd_replicas (M01)           = 1
istiod_replicas               = 3 (min HA)
cilium_agent_replicas         = 17 (DaemonSet)
api_proxy_replicas            = 3 (min HA)

etcd disk: 16 GB block volume; 5-min snapshot to object storage
etcd snapshot retention: 14d hot + 90d cold per data-residency

worker node: VM.Standard.E4 Flex 4 OCPU / 32 GB RAM
control-plane node: VM.Standard.E4 Flex 8 OCPU / 64 GB RAM

Monthly compute cost (per cost-budget.md): ~$5500
Monthly storage cost: ~$1000
TOTAL XS tier per pack region: ~$6500/month
```

## Verification

- `cargo run -p dev-cli -- gate validate capacity-conformance --microservice cloud-k8s` — exit 0; deployed replica counts ≥ formula minimums.
- Quarterly capacity review: actual usage vs forecast; recalibrate `P_pods_per_ms` averages.
- Annual reference-architecture refresh: re-verify against current Kubernetes / Istio / Cilium published sizing guides.

## References

- Kubernetes large-cluster best practices — `kubernetes.io/docs/setup/best-practices/cluster-large/`.
- etcd hardware — `etcd.io/docs/v3.5/op-guide/hardware/`.
- Istio scaling — `istio.io/latest/docs/ops/best-practices/scaling/`.
- Cilium scale — `docs.cilium.io/en/stable/operations/performance/`.
- kubelet sizing — `kubernetes.io/docs/concepts/configuration/manage-resources-containers/`.
- CNCF conformance — `cncf.io/certification/software-conformance/`.
- OCI pricing — `oracle.com/cloud/pricing/`.
- `microservices/cloud-k8s/cost-budget.md`.
- `microservices/cloud-k8s/multi-region.md`.
