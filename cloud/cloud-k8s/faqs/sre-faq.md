---
doc_class: FAQ
microservice: cloud-k8s
persona: sre
date: 2026-05-20
doc_status: published
---

# SRE FAQ — cloud-k8s

## Why are we on kubeadm + upstream Kubernetes instead of Rancher / k3s / OpenShift?

Per ADR-0121 §"Vanilla upstream + minimal patch surface". We need CNCF-conformant clusters so that any future hyperscaler-managed escape hatch (EKS / GKE / AKS / OKE / Tanzu) is a recompile of the IaC, not a re-architecture of the workload. Rancher RKE2 and OpenShift add control-plane components (Rancher cluster-controller, OpenShift OAuth/OLM) that we'd have to unwind during a multi-cluster federation; k3s drops too much (no audit-log API, single-binary etcd embed that doesn't snapshot via standard `etcdctl`). The patch surface we tolerate: containerd 2.3.0 LTS pin, Cilium 1.18 CNI, Istio 1.29.2 mesh, Envoy bundled-with-Istio. That's it.

## Why containerd not CRI-O or Docker?

containerd 2.3.0 LTS has the longest CRI track-record at the throughput envelopes oyatie cells run at (10 k+ pods per cluster; 250 k+ container-start events per day). CRI-O is a credible alternative but its multi-arch image-pulling story lags containerd on ARM64 + RISC-V (we have RISC-V edge cells per ADR-0254 footnote-3). Docker is not a CRI any more — Kubernetes 1.24 removed the dockershim. The decision is in ADR-0121 §"containerd selection rationale".

## etcd is on the control-plane nodes — at what scale do I split it out?

The split-out threshold is **50 worker nodes per cluster** OR **5 000 pods per cluster** OR **etcd_disk_wal_fsync_duration p99 > 25 ms for 7 consecutive days**, whichever fires first. paid dedicated-cloud cell_topology (per `capability-tiers/tier-matrix.md`) implies the split as a baseline. Don't pre-split demo_trial clusters — it adds 3 nodes of ops surface for no benefit at the demo_trial hardware envelope.

## Istio control-plane went 3 replicas → 1 replica during the drill. Was that a config drift?

Likely not — Istio's `istiod` Deployment has `replicas: 3` but a PodDisruptionBudget of `minAvailable: 1`, so during the drill the autoscaler may have scaled to 1 if the drill removed enough nodes. Check `istiod-pdb` budget — if it's `minAvailable: 2` (the production setting per `iac/istio-pdb.yaml`) then a 3 → 1 drop is a real degradation and worth a P2. If the drill cluster set `minAvailable: 1`, expected.

## When can I run `kubectl apply` against a production cluster manually?

Only from the break-glass jump-host with the on-call's break-glass kubeconfig — and only after declaring an incident via `oya incident open`. The audit-chain lane `cloud-k8s-manual-mutation` flags any `kubectl apply` against a production cluster that does not originate from the GitOps reconciler (`flux-system` namespace) or the cluster-API controller (`capi-system` namespace). The flag triggers a P2 review; if the mutation does not link to an open incident the P2 escalates to P1.

## Why does the pod-scheduling latency dashboard show two p99 lines (sidecar-injected vs not)?

Because Istio sidecar inject adds ~ 15 s to first-schedule latency (the Envoy sidecar pull + warm-up). The dashboard separates them so you can see the underlying scheduler latency without sidecar pollution. The `injection-vs-no-injection-p99-delta` panel surfaces the gap; if it widens beyond 25 s, it usually means the sidecar image pull is hitting a cold node — fix by warming the image on the kubeadm join phase via `containerd image pull --pre-pull-list iac/sidecar-prepull.yaml`.

## CSI rebuild — when do I actually do this?

Three triggers: (a) Ceph PG count drops > 5 % unexpectedly (the Ceph mon-quorum lost a member; rebuild = re-balance, not data loss), (b) the cluster's PV churn rate hits the `csi-pv-churn-budget` SLO red line, (c) Velero restore needs the CSI to be re-seeded from snapshot (rare). The runbook `runbooks/csi-rebuild.md` walks each branch; do not run a CSI rebuild on a hunch — the rebuild moves real bytes and the cluster runs degraded for the duration (RWO PVs unavailable on detached nodes during the rebalance).

## A tenant says their pod isn't scheduling and the events say "0/12 nodes are available". What do I check?

In order:

1. NodeAffinity / NodeSelector match (most common; tenant-pack pinning often mismatches).
2. Taints + Tolerations (per-pack taint `pack=kr-pipa:NoSchedule` requires explicit toleration in tenant manifests).
3. Resource fit (CPU + memory; check `kubectl describe node` for allocatable vs requests).
4. PV binding (the PVC's StorageClass might require a node-local rebuild from CSI).
5. Pod-anti-affinity (within-tenant high-cardinality often causes this in cells with < 6 worker nodes).

The on-call escalation tree at `runbooks/pod-not-scheduling.md` enumerates the diagnostic commands per branch.

## Why don't we use Karpenter for autoscaling?

Per ADR-0121 §"Autoscaler selection". Karpenter is excellent for AWS-managed nodes; on-prem we don't have the EC2 API surface Karpenter expects. Cluster Autoscaler with the cluster-api provider gives us the same surface (scale-from-zero, scale-to-zero, node-pool budgets) backed by our own cluster-api implementation. If we move a cell to AWS we'd revisit; on-prem Karpenter requires a third-party provider that's not LTS yet.

## What's the difference between this µservice and `cloud-iac`?

Per ADR-0131 split:

- `cloud-iac`: lays down the network (VLANs, switches, BGP), compute (bare-metal provisioning via Tinkerbell), storage hardware (Ceph mons + OSDs at the hardware-config level). Stops at "boxes are reachable on the management network".
- `cloud-k8s`: turns the boxes from `cloud-iac` into a Kubernetes cluster. Starts at "boxes are reachable" and ends at "kubectl apply works".

You shouldn't be debugging in `cloud-iac` from a `cloud-k8s` page; the boundary is the kubeadm-init prerequisites checklist at `iac/prerequisite-checklist.yaml`.
